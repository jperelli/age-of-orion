//! Turn processing. Economy formulas are ports of 1oom `game_turn.c` / `game_misc.c`
//! (with the 1oom "fix" options for the waste, population tenths and refit bugs).

use crate::combat;
use crate::state::*;
use crate::types::*;

fn adjust_by_special(prod: i32, special: Special) -> i32 {
    match special {
        Special::UltraPoor => prod / 3,
        Special::Poor => prod / 2,
        Special::Rich => prod * 2,
        Special::UltraRich => prod * 3,
        _ => prod,
    }
}

fn spend(p: &Planet, slider: usize) -> i32 {
    adjust_by_special(p.sliders[slider] * p.prod / 100, p.special)
}

/// Research output of a colony (1oom `game_get_tech_prod`).
pub fn tech_prod(p: &Planet, race: Race) -> i32 {
    let mut v = p.prod * p.sliders[SLIDER_TECH] / 100;
    if race == Race::Psilon {
        v += v / 2;
    }
    match p.special {
        Special::Artifacts => v * 2,
        Special::Orion => v * 4,
        _ => v,
    }
}

/// Factory cost for this colony (1oom `game_planet_get_fact_adj_cost`).
pub fn factory_cost(p: &Planet, e: &Empire) -> i32 {
    let a = e.abilities();
    if e.race == Race::Meklar {
        a.factory_cost
    } else {
        (a.factory_cost * p.pop_oper_fact / 2).max(1)
    }
}

/// Ecology spending in BC that keeps a colony free of pollution next turn.
pub fn eco_needed(p: &Planet, e: &Empire) -> i32 {
    if e.race == Race::Silicoid {
        return 0;
    }
    let a = e.abilities();
    let fact = p.factories.min(p.pop * p.pop_oper_fact);
    let waste_new = fact * a.ind_waste_scale / 10;
    let mp2 = p.base_max_pop.max(1);
    let projected = p.waste + ((100 - (p.waste * 100) / mp2) * waste_new) / 100;
    (projected + a.eco_restoration - 1) / a.eco_restoration
}

/// Ecology slider (percent) that keeps a colony clean.
pub fn clean_eco_percent(p: &Planet, e: &Empire) -> i32 {
    if p.prod <= 0 {
        return 0;
    }
    ((eco_needed(p, e) * 100 + p.prod - 1) / p.prod).clamp(0, 100)
}

/// Ships in orbit and en route of an empire, per design.
pub fn ship_totals(st: &State, empire: EmpireId) -> Vec<u32> {
    let n = st.rules.designs.len();
    let mut t = vec![0u32; n];
    for o in &st.empires[empire as usize].orbit {
        for (i, c) in o.iter().enumerate() {
            t[i] += c;
        }
    }
    for f in st.fleets.iter().filter(|f| f.owner == empire) {
        for (i, c) in f.ships.iter().enumerate() {
            t[i] += c;
        }
    }
    t
}

/// 1oom `game_update_production`: industry + population output, AI difficulty bonus,
/// then ship and base maintenance taken proportionally from every colony.
pub fn update_production(st: &mut State) {
    let n = st.empires.len();
    let mut raw_total = vec![0i32; n];
    let mut raw = vec![0i32; st.planets.len()];
    for (i, p) in st.planets.iter().enumerate() {
        let Some(o) = p.owner else { continue };
        let e = &st.empires[o as usize];
        let workers = p.pop * p.pop_oper_fact;
        let mut extra =
            p.pop * (e.tech.level[TechField::Planetology as usize] as i32 * 3 + 50) / 100;
        if e.race == Race::Klackon {
            extra *= 2;
        }
        let mut v = p.factories.min(workers) + extra;
        if e.ai {
            v = match st.difficulty {
                Difficulty::Easy => (v * 4) / 5,
                Difficulty::Average => v + v / 4,
                Difficulty::Hard => v + v / 2,
                Difficulty::Impossible => v * 2,
                Difficulty::Simple => v,
            };
        }
        raw[i] = v.max(0);
        raw_total[o as usize] += raw[i];
    }
    for ei in 0..n {
        let e = ei as u8;
        let ships: i64 = ship_totals(st, e)
            .iter()
            .enumerate()
            .map(|(d, &c)| c as i64 * st.ship_cost(e, d) as i64)
            .sum();
        let bases: i32 = st.colonies(e).map(|(_, p)| p.bases).sum();
        let maint = (ships / 50) as i32 + bases * st.base_cost(e) / 50;
        let em = &mut st.empires[ei];
        em.maintenance = maint.min(raw_total[ei]);
        em.total_prod = raw_total[ei];
    }
    for (i, p) in st.planets.iter_mut().enumerate() {
        let Some(o) = p.owner else {
            p.prod = 0;
            continue;
        };
        let e = &st.empires[o as usize];
        p.prod = if e.total_prod > 0 {
            (raw[i] as i64 * (e.total_prod - e.maintenance) as i64 / e.total_prod as i64) as i32
        } else {
            0
        };
    }
}

fn pop_growth_max(p: &Planet, race: Race, max_pop: i32) -> i32 {
    let mut v;
    if race == Race::Silicoid {
        v = (100 - (p.pop * 100) / max_pop) / 2;
    } else {
        v = 100 - ((p.pop + p.waste) * 100) / max_pop;
        match p.growth {
            Growth::Hostile => v = if v >= 0 { v / 2 } else { v * 2 },
            Growth::Fertile => v = if v >= 0 { v + v / 2 } else { (v * 2) / 3 },
            Growth::Gaia => v = if v >= 0 { v * 2 } else { v / 2 },
            Growth::Normal => {}
        }
        if race == Race::Sakkra {
            v *= 2;
        }
    }
    let mut r = (v * p.pop + 5) / 100;
    if v > 0 && r < 1 {
        r = 1;
    }
    if p.pop + r / 10 < 1 {
        r = 1;
    }
    r
}

fn pop_growth_for_eco(p: &Planet, race: Race, inc_pop_cost: i32, eco: i32) -> i32 {
    let mut v = (eco * 10) / inc_pop_cost.max(1);
    match race {
        Race::Sakkra => v *= 2,
        Race::Silicoid => v /= 2,
        _ => {}
    }
    v.min((p.pop * 10) / 4)
}

/// 1oom `game_turn_build_eco`: waste, eco cleanup, terraforming, population growth.
fn build_eco(st: &mut State) {
    let max_pop_cap = st.rules.max_pop;
    for p in st.planets.iter_mut() {
        let Some(o) = p.owner else { continue };
        if p.pop == 0 || !p.habitable() {
            continue;
        }
        let e = &st.empires[o as usize];
        let a = e.abilities();
        let fact = p.factories.min(p.pop * p.pop_oper_fact);
        let waste = (fact * a.ind_waste_scale) / 10;
        let mp2 = p.base_max_pop.max(1);
        p.waste += ((100 - (p.waste * 100) / mp2) * waste) / 100;
        let silicoid = e.race == Race::Silicoid;
        let ecorestore = if silicoid {
            0
        } else {
            p.waste / a.eco_restoration
        };
        let mut ecoprod = (p.sliders[SLIDER_ECO] * p.prod) / 100;
        if !silicoid {
            if ecorestore > ecoprod {
                p.waste -= a.eco_restoration * ecoprod;
                ecoprod = 0;
            } else {
                ecoprod -= ecorestore;
                p.waste = 0;
            }
        }
        p.waste = p.waste.max(0);
        if mp2 - p.waste < 10 {
            p.waste = (mp2 - 10).max(0);
        }
        if ecoprod > 0 && (p.max_pop - p.base_max_pop) < a.terraform && p.max_pop < max_pop_cap {
            let v = ecoprod / a.terraform_cost;
            let tmax = a.terraform + p.base_max_pop;
            if tmax < p.max_pop + v {
                ecoprod -= (tmax - p.max_pop) * a.terraform_cost;
                p.max_pop = tmax;
            } else {
                p.max_pop += v;
                ecoprod = 0;
            }
        }
        p.max_pop = p.max_pop.min(max_pop_cap);
        let ecoprod = ecoprod.max(0);
        let v = pop_growth_max(p, e.race, p.max_pop.max(1))
            + pop_growth_for_eco(p, e.race, a.inc_pop_cost, ecoprod)
            + p.pop_tenths;
        p.pop += v / 10;
        p.pop_tenths = v % 10;
        p.pop = p.pop.clamp(0, p.max_pop);
    }
}

fn research(st: &mut State) {
    for ei in 0..st.empires.len() {
        let race = st.empires[ei].race;
        let total: i32 = st.colonies(ei as u8).map(|(_, p)| tech_prod(p, race)).sum();
        st.empires[ei].research = total;
        let alloc = st.empires[ei].tech.alloc;
        let asum: u32 = alloc.iter().sum::<u32>().max(1);
        for f in 0..NUM_FIELDS {
            let rp = total as u32 * alloc[f] / asum;
            st.empires[ei].tech.invested[f] += rp;
            loop {
                let e = &st.empires[ei];
                let level = e.tech.level[f];
                if level >= 50 {
                    st.empires[ei].tech.invested[f] = 0;
                    break;
                }
                let cost = st
                    .rules
                    .research_cost(race, f, level + 1, e.ai, st.difficulty);
                if e.tech.invested[f] < cost {
                    break;
                }
                let e = &mut st.empires[ei];
                e.tech.invested[f] -= cost;
                e.tech.level[f] += 1;
                let level = e.tech.level[f];
                st.push_event(
                    vec![ei as u8],
                    EventKind::TechAdvance {
                        field: TechField::ALL[f],
                        level,
                    },
                );
            }
        }
    }
}

/// 1oom `game_turn_build_def`: planetary shield first, then missile bases.
fn build_def(st: &mut State) {
    for i in 0..st.planets.len() {
        let Some(o) = st.planets[i].owner else {
            continue;
        };
        let base_cost = st.base_cost(o);
        let a = st.empires[o as usize].abilities();
        let shield_cost = st.rules.shield_cost[(a.planet_shield / 5) as usize];
        let costs = st.rules.shield_cost;
        let p = &mut st.planets[i];
        let mut prod = spend(p, SLIDER_DEF) + p.bc_to_base;
        let tosh = (shield_cost - p.bc_to_shield).max(0);
        if prod <= tosh {
            p.bc_to_shield += prod;
            prod = 0;
        } else {
            prod -= tosh;
            p.bc_to_shield = shield_cost;
            p.bases += prod / base_cost;
            prod %= base_cost;
        }
        p.bc_to_base = prod;
        let cur = shield_cost.min(p.bc_to_shield);
        p.shield = 5 * (1..5).filter(|&j| cur >= costs[j]).count() as i32;
    }
}

/// 1oom `game_turn_build_ship`.
fn build_ships(st: &mut State) {
    for i in 0..st.planets.len() {
        let Some(o) = st.planets[i].owner else {
            continue;
        };
        let p = &st.planets[i];
        let design = p.build.min(st.rules.designs.len() - 1);
        let cost = st.ship_cost(o, design);
        let mut prod = spend(p, SLIDER_SHIP) + p.bc_to_ship;
        if p.sliders[SLIDER_SHIP] > 0 {
            prod += 1;
        }
        let count = (prod / cost) as u32;
        prod -= count as i32 * cost;
        st.planets[i].bc_to_ship = prod;
        if count > 0 {
            st.empires[o as usize].orbit[i][design] += count;
            st.push_event(
                vec![o],
                EventKind::ShipsBuilt {
                    planet: i,
                    design,
                    count,
                },
            );
        }
    }
}

/// Moves the industry slider elsewhere once a colony has all its factories.
fn release_slider(p: &mut Planet) {
    let v = p.sliders[SLIDER_IND];
    p.sliders[SLIDER_IND] = 0;
    p.sliders[SLIDER_TECH] += v;
}

/// 1oom `game_turn_build_ind` (with the refit fix: colonies refit one step at a time).
fn build_ind(st: &mut State) {
    let max_fact = st.rules.max_factories;
    for i in 0..st.planets.len() {
        let Some(o) = st.planets[i].owner else {
            continue;
        };
        let e = &st.empires[o as usize];
        let a = e.abilities();
        let (race, human) = (e.race, !e.ai);
        let cost = factory_cost(&st.planets[i], e);
        let mut reserve = 0;
        let p = &mut st.planets[i];
        let old = p.factories;
        let mut fact = p.factories;
        let mut prod = spend(p, SLIDER_IND) + p.bc_to_factory;
        let cap = p.pop * p.pop_oper_fact;
        if p.pop_oper_fact < a.colonist_oper_factories && prod / cost + fact > cap {
            if fact < cap {
                prod -= (cap - fact) * cost;
                fact = cap;
            }
            let refit = if race == Race::Meklar {
                0
            } else {
                a.factory_cost * fact / 2
            };
            p.bc_to_refit += prod;
            if p.bc_to_refit >= refit {
                p.pop_oper_fact = if race == Race::Meklar {
                    a.colonist_oper_factories
                } else {
                    (p.pop_oper_fact + 1).min(a.colonist_oper_factories)
                };
                prod = p.bc_to_refit - refit;
                p.bc_to_refit = 0;
            } else {
                prod = 0;
            }
        }
        let cost = if race == Race::Meklar {
            a.factory_cost
        } else {
            (a.factory_cost * p.pop_oper_fact / 2).max(1)
        };
        let mut num = prod / cost;
        p.bc_to_factory = prod % cost;
        let limit = p.max_pop * p.pop_oper_fact;
        if fact + num > limit {
            let excess = (fact + num - limit).clamp(0, num);
            num -= excess;
            reserve += excess * cost / 2;
        }
        fact = (fact + num).min(max_fact);
        p.factories = fact;
        let done = fact != old && fact >= p.max_pop * a.colonist_oper_factories;
        if done && human && p.sliders[SLIDER_IND] > 0 {
            release_slider(p);
        }
        st.empires[o as usize].reserve += reserve;
        if done {
            st.push_event(vec![o], EventKind::FactoriesComplete { planet: i });
        }
    }
}

fn step_towards(st: &State, x: &mut f64, y: &mut f64, dest: PlanetId, speed: i32) -> bool {
    let (tx, ty) = (st.planets[dest].x as f64, st.planets[dest].y as f64);
    let d = ((tx - *x).powi(2) + (ty - *y).powi(2)).sqrt();
    let step = st.rules.parsec * speed.max(1) as f64;
    if d <= step + 1e-9 {
        *x = tx;
        *y = ty;
        true
    } else {
        *x += (tx - *x) * step / d;
        *y += (ty - *y) * step / d;
        false
    }
}

/// Moves fleets and transports; fleets that arrive join the orbit of their destination.
/// Returns transports that arrived this turn.
fn move_ships(st: &mut State) -> Vec<Transport> {
    let mut fleets = std::mem::take(&mut st.fleets);
    fleets.retain_mut(|f| {
        let (mut x, mut y) = (f.x, f.y);
        let arrived = step_towards(st, &mut x, &mut y, f.dest, f.speed);
        f.x = x;
        f.y = y;
        if arrived {
            let o = &mut st.empires[f.owner as usize].orbit[f.dest];
            for (i, c) in f.ships.iter().enumerate() {
                o[i] += c;
            }
        }
        !arrived
    });
    st.fleets = fleets;
    let mut transports = std::mem::take(&mut st.transports);
    let mut arrived = Vec::new();
    transports.retain_mut(|t| {
        let (mut x, mut y) = (t.x, t.y);
        let done = step_towards(st, &mut x, &mut y, t.dest, t.speed);
        t.x = x;
        t.y = y;
        if done {
            arrived.push(t.clone());
        }
        !done
    });
    st.transports = transports;
    arrived
}

/// Sliders for a newly founded colony: grow industry, keep it clean.
pub fn new_colony_sliders() -> [i32; NUM_SLIDERS] {
    [0, 0, 70, 20, 10]
}

pub fn found_colony(st: &mut State, planet: PlanetId, owner: EmpireId, pop: i32) {
    let cof = st.empires[owner as usize]
        .abilities()
        .colonist_oper_factories;
    let p = &mut st.planets[planet];
    p.owner = Some(owner);
    p.pop = pop.min(p.max_pop);
    p.pop_tenths = 0;
    p.factories = 0;
    p.waste = 0;
    p.bases = 0;
    p.shield = 0;
    p.pop_oper_fact = cof;
    p.sliders = new_colony_sliders();
    p.build = 1;
    p.bc_to_ship = 0;
    p.bc_to_factory = 0;
    p.bc_to_base = 0;
    p.bc_to_shield = 0;
    p.bc_to_refit = 0;
    st.empires[owner as usize].explored[planet] = true;
    st.push_event(vec![owner], EventKind::ColonyFounded { planet });
    if planet == st.orion {
        let bonus = st.rules.orion_tech_bonus;
        let e = &mut st.empires[owner as usize];
        for l in e.tech.level.iter_mut() {
            *l = (*l + bonus).min(50);
        }
        st.push_event(vec![], EventKind::OrionColonized { planet, by: owner });
    }
}

pub fn can_colonize(st: &State, empire: EmpireId, planet: PlanetId) -> bool {
    let p = &st.planets[planet];
    let a = st.empires[empire as usize].abilities();
    p.owner.is_none()
        && p.habitable()
        && p.kind >= a.colony_for
        && !(planet == st.orion && st.guardian_alive())
}

/// Colony ships in orbit settle their planet when nothing hostile is armed there.
fn colonize(st: &mut State) {
    for planet in 0..st.planets.len() {
        for ei in 0..st.empires.len() {
            let e = ei as u8;
            let Some(d) = (0..st.rules.designs.len())
                .find(|&d| st.rules.designs[d].colony && st.empires[ei].orbit[planet][d] > 0)
            else {
                continue;
            };
            if !can_colonize(st, e, planet) || combat::hostile_armed_at(st, planet, e) {
                continue;
            }
            st.empires[ei].orbit[planet][d] -= 1;
            let pop = st.rules.colony_ship_pop;
            found_colony(st, planet, e, pop);
        }
    }
}

/// Planets within scanner range of a colony, or with own ships in orbit, are explored.
pub fn update_exploration(st: &mut State) {
    for ei in 0..st.empires.len() {
        let range = st.empires[ei].abilities().scanner_range;
        for p in 0..st.planets.len() {
            if st.empires[ei].explored[p] {
                continue;
            }
            let seen =
                st.empires[ei].orbit[p].iter().any(|&c| c > 0) || st.range_to(ei as u8, p) <= range;
            if seen {
                st.empires[ei].explored[p] = true;
            }
        }
    }
}

fn check_end(st: &mut State) {
    for ei in 0..st.empires.len() {
        if st.empires[ei].alive && st.colonies(ei as u8).next().is_none() {
            let e = &mut st.empires[ei];
            e.alive = false;
            for o in e.orbit.iter_mut() {
                o.iter_mut().for_each(|c| *c = 0);
            }
            st.fleets.retain(|f| f.owner as usize != ei);
            st.transports.retain(|t| t.owner as usize != ei);
            st.push_event(vec![], EventKind::EmpireEliminated { empire: ei as u8 });
        }
    }
    let alive: Vec<usize> = (0..st.empires.len())
        .filter(|&i| st.empires[i].alive)
        .collect();
    if alive.len() == 1 && st.winner.is_none() {
        st.winner = Some(alive[0] as u8);
        st.push_event(
            vec![],
            EventKind::Victory {
                empire: alive[0] as u8,
            },
        );
    }
}

/// Processes one turn for every empire. AI orders must have been given before.
pub fn process(st: &mut State) {
    st.events.clear();
    update_production(st);
    build_eco(st);
    research(st);
    build_def(st);
    build_ships(st);
    build_ind(st);
    let arrived = move_ships(st);
    combat::battles(st);
    combat::bomb(st);
    combat::land_transports(st, arrived);
    colonize(st);
    update_exploration(st);
    check_end(st);
    update_production(st);
    st.turn += 1;
    st.year += 1;
}
