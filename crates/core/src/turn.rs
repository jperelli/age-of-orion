//! Turn processing, a port of 1oom `game_turn.c` (`game_turn_process` and its steps) with
//! 1oom's default rules (none of its optional "fix" switches).
//!
//! A battle with a human player interrupts the turn: `st.battle` holds the battle and
//! `st.progress` where to resume; calling `process` again once the battle is over
//! continues the turn.

use crate::combat;
use crate::design::BUILD_STARGATE;
use crate::diplo::Treaty;
use crate::fleet;
use crate::ground;
use crate::misc;
use crate::state::*;
use crate::tech;
use crate::types::*;
use crate::umath;
use crate::{ai, ai_diplo, audience, council, diplo, events, spy};

/// 1oom's year counter (2300 is year 1).
pub fn year_n(st: &State) -> i32 {
    st.year as i32 - st.rules.start_year as i32 + 1
}

const LIMIT_SHIPS: u32 = 32000;

/// 1oom `game_turn_limit_ships`.
fn limit_ships(st: &mut State) {
    for f in st.fleets.iter_mut() {
        for s in f.ships.iter_mut() {
            *s = (*s).min(LIMIT_SHIPS);
        }
        f.retreat = false;
    }
    for e in st.empires.iter_mut() {
        for o in e.orbit.iter_mut() {
            for s in o.iter_mut() {
                *s = (*s).min(LIMIT_SHIPS);
            }
        }
    }
    fleet::remove_empty(st);
}

/// 1oom `game_turn_init_z_finished`.
fn init_z_finished(st: &mut State) {
    for p in st.planets.iter_mut() {
        p.finished = [false; FINISHED_NUM];
        p.unrefuel = vec![false; PLAYER_NUM];
    }
    for e in st.empires.iter_mut() {
        for r in e.rel.iter_mut() {
            r.contact_broken = false;
        }
    }
}

/// 1oom `game_turn_send_transport`.
fn send_transports(st: &mut State) {
    for i in 0..st.planets.len() {
        if st.planets[i].owner.is_some() && st.planets[i].trans_num > 0 {
            fleet::send_transport(st, i);
        }
    }
}

/// 1oom `game_turn_build_eco`: waste, eco cleanup, terraforming, population growth.
fn build_eco(st: &mut State) {
    let atmos_cost = st.rules.atmos_cost;
    let soil_cost = st.rules.soil_cost;
    let adv_soil_cost = st.rules.adv_soil_cost;
    let max_pop = st.rules.max_pop;
    for i in 0..st.planets.len() {
        let owner = st.planets[i].owner;
        st.planets[i].pop_prev = if owner.is_some() {
            st.planets[i].pop
        } else {
            0
        };
        let Some(owner) = owner else { continue };
        if st.planets[i].kind == PlanetType::NotHabitable || st.planets[i].pop == 0 {
            continue;
        }
        let e = &st.empires[owner as usize];
        let (race, ab) = (e.race, e.ab);
        let cof = ab.colonist_oper_factories;
        let p = &mut st.planets[i];
        {
            let fact = p.factories.min(p.pop * cof);
            let waste = (fact * ab.ind_waste_scale) / 10;
            p.waste += ((100 - ((p.waste * 100) / p.base_max_pop)) * waste) / 100;
        }
        if p.unrest == Unrest::Rebellion {
            p.waste = 0;
        }
        let ecorestore = if race != Race::Silicoid {
            p.waste / ab.eco_restoration
        } else {
            0
        };
        {
            // MOO1 adds waste a second time (1oom `game_turn_build_eco_sub1`).
            let fact = p.factories.min(p.pop * cof);
            let v = (fact * cof) / 10;
            p.waste += ((100 - ((p.waste * 100) / p.base_max_pop)) * v) / 100;
        }
        let mut ecoprod = (p.sliders[SLIDER_ECO] * p.prod) / 100;
        if race != Race::Silicoid {
            if ecorestore > ecoprod {
                p.waste -= ab.eco_restoration * ecoprod;
                ecoprod = 0;
            } else {
                ecoprod -= ecorestore;
                p.waste = 0;
            }
        }
        p.waste = p.waste.max(0);
        if (p.base_max_pop - p.waste) < 10 {
            p.waste = p.base_max_pop - 10;
        }
        p.waste = p.waste.max(0);
        let mut finished = false;
        if ecoprod > 0 && ab.atmos_terraform && p.growth == Growth::Hostile {
            p.bc_to_ecoproj += ecoprod;
            if p.bc_to_ecoproj >= atmos_cost {
                misc::atmos_tform(p);
                ecoprod = p.bc_to_ecoproj - atmos_cost;
                p.bc_to_ecoproj = 0;
                finished = true;
            } else {
                ecoprod = 0;
            }
        }
        if ecoprod > 0 && ab.soil_enrich && p.growth == Growth::Normal {
            p.bc_to_ecoproj += ecoprod;
            if p.bc_to_ecoproj > soil_cost {
                misc::soil_enrich(p, ab.terraform, false, max_pop);
                ecoprod = p.bc_to_ecoproj - soil_cost;
                p.bc_to_ecoproj = p.bc_to_ecoproj.min(soil_cost);
                finished = true;
            } else {
                ecoprod = 0;
            }
        }
        if ecoprod > 0
            && ab.adv_soil_enrich
            && p.growth < Growth::Gaia
            && p.growth > Growth::Hostile
        {
            p.bc_to_ecoproj += ecoprod;
            if p.bc_to_ecoproj > adv_soil_cost {
                misc::soil_enrich(p, ab.terraform, true, max_pop);
                ecoprod = p.bc_to_ecoproj - adv_soil_cost;
                p.bc_to_ecoproj = 0;
                finished = true;
            } else {
                ecoprod = 0;
            }
        }
        if finished {
            p.finished[FINISHED_SOILATMOS] = true;
        }
        if ecoprod > 0 && (p.max_pop - p.base_max_pop) < ab.terraform && p.max_pop < max_pop {
            let v = ecoprod / ab.terraform_cost;
            let tmax = ab.terraform + p.base_max_pop;
            if tmax < p.max_pop + v {
                ecoprod -= (tmax - p.max_pop) * ab.terraform_cost;
                p.max_pop = tmax;
            } else {
                p.max_pop += v;
                ecoprod = 0;
            }
        }
        p.max_pop = p.max_pop.min(max_pop);
        let ecoprod = ecoprod.max(0);
        let p = &st.planets[i];
        let v = misc::pop_growth_max(st, p, p.max_pop)
            + misc::pop_growth_for_eco(st, p, ecoprod)
            + p.pop_tenths;
        let p = &mut st.planets[i];
        p.pop += v / 10;
        p.pop_tenths = v % 10;
        p.pop = p.pop.clamp(0, p.max_pop);
    }
}

/// 1oom `game_turn_build_def`: missile bases, their upgrades and planetary shields.
fn build_def(st: &mut State) {
    let n = st.empires.len();
    let mut cost_new = vec![0; n];
    let mut cost_diff = vec![0; n];
    for i in 0..n {
        let cost_old = st.base_cost(i as u8);
        let lv = st.empires[i].tech.level;
        let w = st.base_weapon(i as u8);
        let _ = lv;
        let (sh, cp, _) = st.base_parts(i as u8);
        let e = &mut st.empires[i];
        e.base_weapon = w;
        e.base_shield = sh;
        e.base_comp = cp;
        cost_new[i] = st.base_cost(i as u8);
        cost_diff[i] = (cost_new[i] - cost_old).max(0);
    }
    let pshield_cost = st.rules.shield_cost;
    for i in 0..st.planets.len() {
        let Some(owner) = st.planets[i].owner else {
            continue;
        };
        let o = owner as usize;
        let (shield_cost, have_shield) = (
            st.empires[o].ab.planet_shield_cost,
            st.empires[o].ab.planet_shield,
        );
        let in_nebula = st.in_nebula(st.planets[i].x, st.planets[i].y);
        let p = &mut st.planets[i];
        let mut prod =
            misc::adjust_prod_by_special((p.sliders[SLIDER_DEF] * p.prod) / 100, p.special);
        prod += p.bc_to_base;
        let mut toup = cost_diff[o] * p.bases + p.bc_upgrade_base;
        if toup >= prod {
            toup -= prod;
            p.bc_upgrade_base = toup;
            p.bc_to_base = 0;
        } else {
            prod -= toup;
            p.bc_upgrade_base = 0;
            let mut tosh = (shield_cost - p.bc_to_shield).max(0);
            if p.battlebg == 0 || in_nebula {
                tosh = 0;
                p.bc_to_shield = 0;
            }
            if prod <= tosh {
                p.bc_to_shield += prod;
                prod = 0;
            } else {
                prod -= tosh;
                p.bc_to_shield = shield_cost;
                while prod >= cost_new[o] {
                    prod -= cost_new[o];
                    p.bases += 1;
                }
            }
            p.bc_to_base = prod;
        }
        let curcost = shield_cost.min(p.bc_to_shield);
        let mut newshield = 0;
        for c in pshield_cost.iter().skip(1) {
            if curcost >= *c {
                newshield += 5;
            } else {
                break;
            }
        }
        if newshield != p.shield {
            p.shield = newshield;
            if newshield == have_shield {
                p.finished[FINISHED_SHIELD] = true;
            }
        }
    }
}

/// 1oom `game_turn_build_ship`.
fn build_ship(st: &mut State) {
    misc::update_maint_costs(st);
    for e in st.empires.iter_mut() {
        let lv = e.tech.level;
        for d in e.designs.iter_mut() {
            d.cost_bc = d.cost(&lv);
        }
    }
    let gate_cost = st.rules.stargate_cost;
    for i in 0..st.planets.len() {
        let Some(owner) = st.planets[i].owner else {
            continue;
        };
        let o = owner as usize;
        let p = &mut st.planets[i];
        let mut prod =
            misc::adjust_prod_by_special((p.sliders[SLIDER_SHIP] * p.prod) / 100, p.special);
        prod += p.bc_to_ship;
        // MOO1 bug: a 1 BC bonus for having the slider above 0.
        if p.sliders[SLIDER_SHIP] > 0 {
            prod += 1;
        }
        let si = p.build;
        if si == BUILD_STARGATE {
            if prod >= gate_cost {
                p.stargate = true;
                p.build = 0;
                p.bc_to_ship = prod - gate_cost;
                p.finished[FINISHED_STARGATE] = true;
            } else {
                p.bc_to_ship = prod;
            }
            continue;
        }
        let cost = st.empires[o]
            .designs
            .get(si)
            .map_or(1, |d| d.cost_bc.max(1)) as i32;
        let mut shipnum = 0u32;
        while prod >= cost {
            shipnum += 1;
            prod -= cost;
        }
        let count = st.empires[o].shipcount.get(si).copied().unwrap_or(0);
        if shipnum + count > LIMIT_SHIPS {
            shipnum = LIMIT_SHIPS.saturating_sub(count);
        }
        if let Some(c) = st.empires[o].shipcount.get_mut(si) {
            *c += shipnum;
        }
        let dest = st.planets[i].rally.unwrap_or(i);
        if dest == i {
            st.empires[o].orbit[i][si] += shipnum;
            if shipnum > 0 {
                st.planets[i].finished[FINISHED_SHIP] = true;
            }
        } else if shipnum > 0 {
            fleet::send_reloc(st, owner, i, dest, si, shipnum);
        }
        if shipnum > 0 {
            st.push_event(
                vec![owner],
                EventKind::ShipsBuilt {
                    planet: i,
                    design: si,
                    count: shipnum,
                },
            );
        }
        st.planets[i].bc_to_ship = prod;
    }
    misc::update_visibility(st);
}

/// 1oom `game_turn_reserve`: planets use up their reserve, taxes fill the empire's.
fn reserve(st: &mut State) {
    for p in st.planets.iter_mut().filter(|p| p.owner.is_some()) {
        let v = p.prod / 2;
        p.reserve = if v < p.reserve { p.reserve - v } else { 0 };
    }
    for e in st.empires.iter_mut() {
        e.reserve += (e.total_prod as i64 * e.tax as i64 / 2000) as i32;
    }
}

/// 1oom `game_turn_finished_switch_slider`.
fn switch_slider(st: &State, p: &mut Planet, from: usize, to: usize) -> bool {
    if from == to {
        return false;
    }
    let mut v = p.sliders[from];
    if from == SLIDER_ECO {
        v -= misc::waste_percent(st, p, false).0;
        v = v.max(0);
    }
    p.sliders[from] -= v;
    p.sliders[to] += v;
    true
}

/// 1oom `game_turn_finished_disable_slider`.
fn disable_slider(st: &State, p: &mut Planet, from: usize, build_ind: bool) {
    let first = if build_ind { SLIDER_IND } else { SLIDER_TECH };
    let _ = switch_slider(st, p, from, first)
        || switch_slider(st, p, from, SLIDER_TECH)
        || switch_slider(st, p, from, SLIDER_SHIP)
        || switch_slider(st, p, from, SLIDER_DEF)
        || switch_slider(st, p, from, SLIDER_IND)
        || switch_slider(st, p, from, SLIDER_ECO);
}

fn with_planet(st: &mut State, i: PlanetId, f: impl FnOnce(&State, &mut Planet)) {
    let mut p = st.planets[i].clone();
    f(st, &mut p);
    st.planets[i] = p;
}

/// 1oom `game_turn_build_ind`.
fn build_ind(st: &mut State) {
    let max_fact = st.rules.max_factories;
    for i in 0..st.planets.len() {
        let Some(owner) = st.planets[i].owner else {
            continue;
        };
        let o = owner as usize;
        let (race, ab) = (st.empires[o].race, st.empires[o].ab);
        let cost = ab.factory_adj_cost.max(1);
        let p = &mut st.planets[i];
        let factold = p.factories & 0xffff;
        let mut fact = factold;
        let mut prod =
            misc::adjust_prod_by_special((p.sliders[SLIDER_IND] * p.prod) / 100, p.special);
        prod += p.bc_to_factory;
        let bonus = if race == Race::Meklar { 2 } else { 0 };
        let mut v = ab.colonist_oper_factories - p.pop_oper_fact - bonus;
        if v > 0 {
            v *= ab.factory_cost / 2;
        } else {
            v = 0;
        }
        if (prod / cost + fact) > (p.pop * p.pop_oper_fact) {
            p.bc_to_refit += prod;
            if p.bc_to_refit >= v {
                p.pop_oper_fact = ab.colonist_oper_factories;
                prod = p.bc_to_refit - v;
                p.bc_to_refit = 0;
            } else {
                prod = 0;
            }
        }
        let mut num = (prod / cost) & 0xffff;
        p.bc_to_factory = prod % cost;
        let mut v = p.max_pop * ab.colonist_oper_factories;
        if v < fact + num {
            v = ((fact + num) - v).max(0).min(num);
            num -= v;
        } else {
            v = 0;
        }
        let refund = (v * cost) / 2;
        fact += num;
        fact = fact.min(max_fact);
        p.factories = fact;
        let done = fact != factold && fact >= p.max_pop * ab.colonist_oper_factories;
        st.empires[o].reserve += refund;
        if done {
            if !st.empires[o].ai && st.planets[i].sliders[SLIDER_IND] > 0 {
                with_planet(st, i, |st, p| disable_slider(st, p, SLIDER_IND, false));
            }
            st.planets[i].finished[FINISHED_FACT] = true;
            st.push_event(vec![owner], EventKind::FactoriesComplete { planet: i });
        }
    }
}

/// 1oom `game_turn_move_ships`: fleets move in steps of 5 and 6 map units (two steps per
/// warp), at half speed inside nebulae; monsters move one parsec.
fn move_ships(st: &mut State) {
    let mut flag_more = true;
    let mut frame = 0;
    while frame < 20 && flag_more {
        let odd = frame & 1 == 1;
        flag_more = false;
        for fi in 0..st.fleets.len() {
            let f = &st.fleets[fi];
            if f.speed * 2 <= frame {
                continue;
            }
            let (mut x, mut y) = (f.x, f.y);
            let in_nebula = if odd || frame == 0 {
                false
            } else {
                st.in_nebula(x, y)
            };
            if odd || frame == 0 || !in_nebula {
                let p = &st.planets[f.dest];
                if f.speed == SPEED_STARGATE {
                    x = p.x;
                    y = p.y;
                } else {
                    flag_more = true;
                    umath::go_line_dist(&mut x, &mut y, p.x, p.y, if odd { 6 } else { 5 });
                }
                let f = &mut st.fleets[fi];
                f.x = x;
                f.y = y;
            } else if in_nebula {
                flag_more = true;
            }
        }
        for ti in 0..st.transports.len() {
            let t = &st.transports[ti];
            if t.speed * 2 <= frame {
                continue;
            }
            let (mut x, mut y) = (t.x, t.y);
            let in_nebula = if !odd { false } else { st.in_nebula(x, y) };
            if !odd || !in_nebula {
                let p = &st.planets[t.dest];
                if t.speed == SPEED_STARGATE {
                    x = p.x;
                    y = p.y;
                } else {
                    flag_more = true;
                    umath::go_line_dist(&mut x, &mut y, p.x, p.y, if odd { 6 } else { 5 });
                }
                let t = &mut st.transports[ti];
                t.x = x;
                t.y = y;
            } else if in_nebula {
                flag_more = true;
            }
        }
        for kind in [events::MonsterKind::Crystal, events::MonsterKind::Amoeba] {
            if frame >= 2 {
                break;
            }
            let Some(mi) = st.evn.monsters.iter().position(|m| m.kind == kind) else {
                continue;
            };
            let m = &st.evn.monsters[mi];
            if m.counter > 0 {
                continue;
            }
            let (x1, y1) = (st.planets[m.dest].x, st.planets[m.dest].y);
            let (mut x, mut y) = (m.x, m.y);
            if x != x1 || y != y1 {
                umath::go_line_dist(&mut x, &mut y, x1, y1, if odd { 6 } else { 5 });
                let m = &mut st.evn.monsters[mi];
                m.x = x;
                m.y = y;
            }
        }
        frame += 1;
    }
    let mut i = 0;
    while i < st.fleets.len() {
        let f = &st.fleets[i];
        let p = &st.planets[f.dest];
        if f.x == p.x && f.y == p.y {
            let f = st.fleets.swap_remove(i);
            let o = &mut st.empires[f.owner as usize].orbit[f.dest];
            for (j, n) in f.ships.iter().enumerate() {
                o[j] = (o[j] + n).min(LIMIT_SHIPS);
            }
        } else {
            i += 1;
        }
    }
    misc::update_visibility(st);
}

/// 1oom `game_turn_explore`: ships reveal planets and colony ships settle them. A human is
/// asked before a colony is built (`Question::Colonize`): the exploration stops there and
/// goes on from the same planet and empire once answered; true while it waits.
fn explore(st: &mut State) -> bool {
    let n = st.empires.len();
    let (mut p0, mut i0) = st.progress.as_ref().map_or((0, 0), |p| p.at);
    if let Some(Question::Colonize {
        empire,
        planet,
        ship,
        ..
    }) = st.question.clone()
    {
        let Some(yes) = st.answer.take() else {
            return true;
        };
        st.question = None;
        if yes {
            colonize(st, empire as usize, planet, ship);
        }
        (p0, i0) = if empire as usize + 1 < n {
            (planet, empire as usize + 1)
        } else {
            (planet + 1, 0)
        };
    }
    for pli in p0..st.planets.len() {
        for i in if pli == p0 { i0 } else { 0 }..n {
            if i == 0 {
                st.planets[pli].artifact_looter = None;
            }
            let explored_i = st.empires[i].explored[pli];
            if explored_i && st.planets[pli].owner.is_some() {
                continue;
            }
            let e = &st.empires[i];
            let mut visible = e.orbit[pli].iter().take(e.designs.len()).any(|&s| s > 0);
            let mut by_scanner = false;
            if !visible && e.ab.adv_scanner {
                let p = &st.planets[pli];
                for p2 in &st.planets {
                    if p2.owner == Some(i as u8) && umath::dist_fast(p.x, p.y, p2.x, p2.y) <= 110 {
                        visible = true;
                        break;
                    }
                }
                by_scanner = true;
            }
            if !visible {
                continue;
            }
            let first = st.empires.iter().all(|e| !e.explored[pli]);
            if st.planets[pli].special == Special::Artifacts && !by_scanner && first {
                st.planets[pli].artifact_looter = Some(i as u8);
            }
            let mut best_colonize = 200;
            let mut best_ship = 0;
            for (j, sd) in e.designs.iter().enumerate() {
                let mut can = 200;
                for &s in &sd.specials {
                    if (2..=8).contains(&s) {
                        can = PlanetType::Minimal as i32 - (s as i32 - 2);
                    }
                }
                if can < 200 && e.orbit[pli][j] > 0 {
                    if can < best_colonize {
                        best_colonize = can;
                        best_ship = j;
                    }
                    if e.race == Race::Silicoid {
                        best_colonize = PlanetType::Radiated as i32;
                    }
                }
            }
            let p = &st.planets[pli];
            if best_colonize == 200
                || p.owner.is_some()
                || p.kind == PlanetType::NotHabitable
                || (p.kind as i32) < best_colonize
            {
                best_colonize = 0;
            }
            st.empires[i].explored[pli] = true;
            if !st.empires[i].ai {
                if best_colonize != 0 {
                    st.question = Some(Question::Colonize {
                        empire: i as u8,
                        planet: pli,
                        ship: best_ship,
                        first: !explored_i,
                    });
                    st.answer = None;
                    if let Some(p) = st.progress.as_mut() {
                        p.at = (pli, i);
                    }
                    return true;
                }
                if !explored_i {
                    st.push_event(
                        vec![i as u8],
                        EventKind::Explored {
                            planet: pli,
                            by_scanner,
                        },
                    );
                }
            } else if best_colonize != 0 {
                colonize(st, i, pli, best_ship);
            }
        }
    }
    false
}

/// Empire `i` builds a colony on `pli` with a ship of design `ship` from its orbit.
fn colonize(st: &mut State, i: usize, pli: PlanetId, ship: usize) {
    let p = &mut st.planets[pli];
    p.owner = Some(i as u8);
    p.pop = 2;
    st.empires[i].orbit[pli][ship] -= 1;
    st.push_event(vec![i as u8], EventKind::ColonyFounded { planet: pli });
    if pli == st.orion {
        st.push_event(
            vec![],
            EventKind::OrionColonized {
                planet: pli,
                by: i as u8,
            },
        );
    }
}

/// 1oom `game_turn_coup` is in `events::coup`; `game_turn_check_end` decides the winner.
fn check_end(st: &mut State) -> bool {
    let n = st.empires.len();
    for ei in 0..n {
        if st.empires[ei].alive && st.colonies(ei as u8).next().is_none() {
            let e = &mut st.empires[ei];
            e.alive = false;
            for o in e.orbit.iter_mut() {
                o.iter_mut().for_each(|c| *c = 0);
            }
            fleet::remove_player(st, ei as u8);
            st.push_event(vec![], EventKind::EmpireEliminated { empire: ei as u8 });
        }
    }
    if st.winner.is_some() {
        return true;
    }
    let owners: Vec<usize> = (0..n)
        .filter(|&i| st.planets.iter().any(|p| p.owner == Some(i as u8)))
        .collect();
    if owners.len() == 1 {
        st.winner = Some(owners[0] as u8);
        st.victory = Some(Victory::Conquest);
        st.push_event(
            vec![],
            EventKind::Victory {
                empire: owners[0] as u8,
            },
        );
        return true;
    }
    let humans: Vec<usize> = (0..n).filter(|&i| !st.empires[i].ai).collect();
    if !humans.is_empty() && !humans.iter().any(|&h| owners.contains(&h)) {
        return true;
    }
    false
}

/// 1oom `game_turn_claim`: owner, or an empire with ships at an empty planet.
fn claim(st: &mut State) {
    for pli in 0..st.planets.len() {
        let mut c = st.planets[pli].owner;
        if c.is_none() {
            for (e, emp) in st.empires.iter().enumerate() {
                if emp.orbit[pli]
                    .iter()
                    .take(emp.designs.len())
                    .any(|&n| n != 0)
                {
                    c = Some(e as u8);
                }
            }
        }
        st.planets[pli].claim = c;
    }
}

/// 1oom `game_turn_finished_slider`: humans' sliders move on when a project is done.
fn finished_slider(st: &mut State) {
    for pli in 0..st.planets.len() {
        let owner = st.planets[pli].owner;
        let Some(pi) = owner.filter(|&o| !st.empires[o as usize].ai) else {
            st.planets[pli].finished = [false; FINISHED_NUM];
            continue;
        };
        let ab = st.empires[pi as usize].ab;
        let race = st.empires[pi as usize].race;
        with_planet(st, pli, |st, p| {
            let cond_build_ind = p.factories < p.pop * ab.colonist_oper_factories;
            if p.finished[FINISHED_SOILATMOS] {
                disable_slider(st, p, SLIDER_ECO, cond_build_ind);
            }
            if p.pop >= p.max_pop && p.unrest == Unrest::Normal {
                let mut v = (p.prod * p.sliders[SLIDER_ECO]) / 100;
                let pending = v > 0
                    && ((ab.atmos_terraform && p.growth == Growth::Hostile)
                        || (ab.soil_enrich && p.growth == Growth::Normal)
                        || (ab.adv_soil_enrich
                            && (p.growth == Growth::Normal || p.growth == Growth::Fertile))
                        || (p.max_pop - p.base_max_pop) < ab.terraform);
                if !pending {
                    let (w, waste) = misc::waste_percent(st, p, false);
                    v = (v - waste).max(0);
                    if (misc::pop_growth_for_eco(st, p, v) / 10 > 0
                        || (p.sliders[SLIDER_ECO] != 0 && race == Race::Silicoid))
                        && (p.pop > p.pop_prev || p.sliders[SLIDER_ECO] > w + 7)
                    {
                        disable_slider(st, p, SLIDER_ECO, cond_build_ind);
                    }
                    if p.sliders[SLIDER_ECO] > 0
                        && (race == Race::Silicoid || (ab.ind_waste_scale == 0 && p.waste == 0))
                    {
                        disable_slider(st, p, SLIDER_ECO, cond_build_ind);
                    }
                }
            }
            if p.finished[FINISHED_SHIELD] {
                disable_slider(st, p, SLIDER_DEF, false);
            }
            if p.finished[FINISHED_STARGATE] {
                disable_slider(st, p, SLIDER_SHIP, false);
            }
        });
    }
}

/// 1oom `game_turn_update_have_met`.
fn update_have_met(st: &mut State) {
    misc::update_empire_contact(st);
    let n = st.empires.len();
    for i in 0..n {
        if st.empires[i].ai {
            continue;
        }
        for j in 0..n {
            if i != j && st.empires[i].rel[j].have_met == 0 && st.empires[i].rel[j].contact {
                st.empires[i].rel[j].have_met = 1;
                st.empires[i].rel[j].treaty = Treaty::None;
                st.empires[j].rel[i].treaty = Treaty::None;
            }
        }
    }
}

/// Processes one turn for every empire, in the order of 1oom `game_turn_process`; the
/// computer players give their orders inside (1oom `game_ai->turn_p1..p3`).
pub fn process(st: &mut State) {
    if st.progress.is_none() {
        let old_contact = before_battles(st);
        st.progress = Some(Progress {
            old_contact,
            ..Default::default()
        });
    }
    if let Some(bt) = st.battle.as_ref() {
        if bt.winner.is_none() {
            return;
        }
        combat::finish_battle(st);
        st.battle = None;
    }
    let stage = |st: &State| st.progress.as_ref().map_or(0, |p| p.stage);
    let next = |st: &mut State, stage: u8| {
        if let Some(p) = st.progress.as_mut() {
            p.stage = stage;
            p.at = (0, 0);
        }
    };
    if stage(st) == 0 {
        if combat::battles(st) {
            return;
        }
        spy::turn(st);
        tech::research(st);
        if st.guardian {
            let o = st.orion;
            for e in st.empires.iter_mut() {
                e.orbit[o][0] = 0;
            }
        }
        next(st, 1);
    }
    if stage(st) == 1 {
        if explore(st) {
            return;
        }
        next(st, 2);
    }
    if combat::bomb(st) {
        return;
    }
    let old_contact = st
        .progress
        .take()
        .map(|p| p.old_contact)
        .unwrap_or_default();
    after_battles(st, old_contact);
    crate::stats::sample(st);
}

fn before_battles(st: &mut State) -> Vec<Vec<bool>> {
    st.events.clear();
    audience::expire(st);
    council::expire(st);
    st.pending.spy.clear();
    limit_ships(st);
    let old_contact: Vec<Vec<bool>> = st
        .empires
        .iter()
        .map(|e| e.rel.iter().map(|r| r.contact).collect())
        .collect();
    if year_n(st) > 40 {
        // MOO1 copy protection roll (never acted upon).
        st.rng.below(30);
    }
    diplo::countdown_ceasefire(st);
    diplo::update_mood_blunder(st);
    misc::update_have_reserve_fuel(st);
    ai::turn_p1(st);
    ai::turn_p2(st);
    misc::update_have_reserve_fuel(st);
    ai::turn_p3(st);
    init_z_finished(st);
    send_transports(st);
    misc::update_production(st);
    build_eco(st);
    misc::update_total_research(st);
    misc::update_have_reserve_fuel(st);
    diplo::update_trade(st);
    spy::build(st);
    misc::update_production(st);
    diplo::diplo_adjust(st);
    build_def(st);
    build_ship(st);
    reserve(st);
    build_ind(st);
    move_ships(st);
    limit_ships(st);
    fleet::remove_empty(st);
    spy::report(st);
    old_contact
}

/// The rest of the turn, after battles, exploration and bombing.
fn after_battles(st: &mut State, old_contact: Vec<Vec<bool>>) {
    ground::land(st);
    ground::resolve_all(st);
    events::coup(st);
    if check_end(st) {
        return finish_year(st);
    }
    let since = st.events.len();
    events::new_event(st);
    events::run(st, since);
    if check_end(st) {
        return finish_year(st);
    }
    for pli in 0..st.planets.len() {
        if let Some(looter) = st.planets[pli].artifact_looter {
            tech::artifact_loot(st, looter);
        }
    }
    update_have_met(st);
    ai_diplo::turn_p1(st);
    if council::due(st) {
        council::convene(st);
    }
    if check_end(st) {
        return finish_year(st);
    }
    ai_diplo::turn_p2(st);
    audience::collect(st);
    if check_end(st) {
        return finish_year(st);
    }
    diplo::contact_broken(st, &old_contact);
    fleet::unrefuel(st);
    misc::update_production(st);
    for i in 0..st.empires.len() {
        misc::update_eco_on_waste(st, i as u8, false);
    }
    misc::update_seen(st);
    diplo::mood_relax(st);
    st.turn += 1;
    st.year += 1;
    tech::update_tech_util(st);
    for e in st.empires.iter_mut() {
        for f in 0..NUM_FIELDS {
            e.tech.level[f] = tech::field_level(&e.tech, f);
        }
    }
    finished_slider(st);
    claim(st);
    misc::update_within_range(st);
    misc::update_visibility(st);
    diplo::update_final_war(st);
    fleet::remove_empty(st);
    misc::update_home(st);
}

/// The end of a turn that ended the game.
fn finish_year(st: &mut State) {
    st.turn += 1;
    st.year += 1;
}

/// Ecology slider (percent) that keeps a colony clean (the UI's "clean" button).
pub fn clean_eco_percent(st: &State, p: &Planet) -> i32 {
    misc::waste_percent(st, p, false).0
}

/// Whether the empire could settle `planet` with its colony technology.
pub fn can_colonize(st: &State, empire: EmpireId, planet: PlanetId) -> bool {
    let e = &st.empires[empire as usize];
    let p = &st.planets[planet];
    p.owner.is_none()
        && p.habitable()
        && p.kind as i32 >= e.ab.colony_for as i32
        && !(planet == st.orion && st.guardian)
}
