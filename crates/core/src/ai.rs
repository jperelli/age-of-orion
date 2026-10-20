//! Computer players. A compact strategy written for this port (the 4,500-line classic AI
//! of 1oom `game_ai_classic.c` is on the roadmap): expand with colony ships, scout, build
//! industry, research, design better ships as techs arrive, then attack weak neighbours and
//! invade them with transports. Research choices and slider focus follow the classic AI.

use crate::commands::{apply, fleet_range, Command};
use crate::data::{SHIELDS, WEAPONS};
use crate::design::{
    auto_design, Role, ShipDesign, HULL_LARGE, HULL_MEDIUM, HULL_SMALL, MAX_DESIGNS,
};
use crate::state::*;
use crate::tech;
use crate::turn::{can_colonize, clean_eco_percent};
use crate::types::*;

#[derive(Debug, Default)]
struct Roles {
    scout: Option<usize>,
    colony: Option<usize>,
    bomber: Option<usize>,
    warship: Option<usize>,
    cheap_warship: Option<usize>,
}

fn role_of(d: &ShipDesign) -> Role {
    if d.colony_type().is_some() {
        Role::Colony
    } else if d.bombs() {
        Role::Bomber
    } else if d.armed() {
        Role::Warship
    } else {
        Role::Scout
    }
}

fn roles(st: &State, me: EmpireId) -> Roles {
    let e = &st.empires[me as usize];
    let lv = &e.tech.level;
    let of = |r: Role| (0..e.designs.len()).filter(move |&i| role_of(&e.designs[i]) == r);
    Roles {
        scout: of(Role::Scout).find(|&i| e.designs[i].extra_range() > 0),
        colony: of(Role::Colony).min_by_key(|&i| e.designs[i].colony_type()),
        bomber: of(Role::Bomber).max_by_key(|&i| e.designs[i].strength()),
        warship: of(Role::Warship).max_by_key(|&i| {
            e.designs[i].strength() * 100 / e.designs[i].cost(lv).max(1) as i64
                + e.designs[i].cost(lv) as i64
        }),
        cheap_warship: of(Role::Warship).min_by_key(|&i| e.designs[i].cost(lv)),
    }
}

/// Combat value of ships of an empire: damage per turn times hit points.
fn strength(e: &Empire, ships: &[u32]) -> i64 {
    ships
        .iter()
        .zip(&e.designs)
        .map(|(&c, d)| c as i64 * d.strength())
        .sum()
}

fn defence_strength(st: &State, me: usize, planet: PlanetId) -> i64 {
    let p = &st.planets[planet];
    let mut s: i64 = 0;
    if planet == st.orion && st.guardian {
        s += 4_000_000 * (st.difficulty as i64 + 1);
    }
    if let Some(o) = p.owner.filter(|&o| o as usize != me) {
        let w = &WEAPONS[st.base_weapon(o)];
        let (shield, _, _) = st.base_parts(o);
        let dmg = w.damage_max as i64 * 6;
        let hp = st.rules.base_hp[st.empires[o as usize].abilities().armor_tier.min(6)] as i64
            + (SHIELDS[shield].absorb as i64 + p.shield as i64) * 5;
        s += p.bases as i64 * dmg * hp;
    }
    for (ei, e) in st.empires.iter().enumerate() {
        if ei != me {
            s += strength(e, &e.orbit[planet]);
        }
    }
    s
}

fn armed_only(e: &Empire, ships: &[u32]) -> Vec<u32> {
    ships
        .iter()
        .enumerate()
        .map(|(d, &c)| {
            if e.designs
                .get(d)
                .is_some_and(|d| d.armed() && d.colony_type().is_none())
            {
                c
            } else {
                0
            }
        })
        .collect()
}

/// A name from the race's list that no current design uses (1oom `game_get_random_shipnames`).
fn ship_name(st: &mut State, me: EmpireId, hull: u8) -> String {
    let pick = hull as usize * 3 + st.rng.below(3) as usize;
    let e = &st.empires[me as usize];
    let names = &e.ship_names;
    let pick = pick.min(names.len().saturating_sub(1));
    for i in (pick..names.len()).chain((0..pick).rev()) {
        if !e.designs.iter().any(|d| d.name == names[i]) {
            return names[i].clone();
        }
    }
    format!(
        "{} {}",
        names.get(pick).cloned().unwrap_or_default(),
        st.turn
    )
}

/// Creates better designs as techs arrive (every tenth turn per empire).
fn redesign(st: &mut State, me: EmpireId) {
    if st.turn % 10 != me as u32 % 10 {
        return;
    }
    let total_prod = st.empires[me as usize].total_prod;
    let hull = if total_prod > 400 {
        HULL_LARGE
    } else if total_prod > 80 {
        HULL_MEDIUM
    } else {
        HULL_SMALL
    };
    for role in [Role::Warship, Role::Bomber, Role::Colony] {
        let hull = match role {
            Role::Colony => HULL_LARGE,
            Role::Bomber => hull.max(HULL_MEDIUM),
            _ => hull,
        };
        let e = &st.empires[me as usize];
        let name = String::new();
        let mut d = auto_design(&e.tech, role, hull, name);
        if !d.armed() && role != Role::Colony {
            continue;
        }
        let lv = e.tech.level;
        let totals = crate::turn::ship_totals(st, me);
        let e = &st.empires[me as usize];
        let same: Vec<usize> = (0..e.designs.len())
            .filter(|&i| role_of(&e.designs[i]) == role)
            .collect();
        let better = match role {
            Role::Colony => same
                .iter()
                .all(|&i| d.colony_type() < e.designs[i].colony_type()),
            _ => same.iter().all(|&i| {
                d.strength() * 100 / d.cost(&lv).max(1) as i64
                    > e.designs[i].strength() * 120 / e.designs[i].cost(&lv).max(1) as i64
            }),
        };
        if !better {
            continue;
        }
        d.name = ship_name(st, me, hull);
        let e = &st.empires[me as usize];
        let slot = if e.designs.len() < MAX_DESIGNS {
            None
        } else {
            // Replace the least used design of the same role (or any warship).
            let pool: Vec<usize> = if same.is_empty() {
                (0..e.designs.len())
                    .filter(|&i| role_of(&e.designs[i]) == Role::Warship)
                    .collect()
            } else {
                same.clone()
            };
            pool.into_iter().min_by_key(|&i| totals[i])
        };
        if e.designs.len() >= MAX_DESIGNS && slot.is_none() {
            continue;
        }
        let _ = apply(st, me, &Command::SaveDesign { slot, design: d });
    }
}

/// The classic AI's research focus, every ten years (1oom `game_ai_classic_turn_p3`).
fn research_focus(st: &mut State, me: EmpireId) {
    let race = st.empires[me as usize].race;
    if st.turn % 10 == me as u32 % 10 {
        let mut sl = [5u32; NUM_FIELDS];
        let early = st.turn < 30 && st.rng.below(2) == 1;
        let own = st.rng.below(4) == 0;
        let focus = if early {
            FIELD_PROPULSION
        } else {
            match race {
                Race::Bulrathi | Race::Mrrshan if own => FIELD_WEAPON,
                Race::Darlok if own => FIELD_COMPUTER,
                Race::Meklar if own => {
                    if st.rng.below(2) == 1 {
                        FIELD_CONSTRUCTION
                    } else {
                        FIELD_PLANETOLOGY
                    }
                }
                Race::Alkari if own => FIELD_PROPULSION,
                Race::Sakkra if own => FIELD_PLANETOLOGY,
                _ => [
                    FIELD_COMPUTER,
                    FIELD_CONSTRUCTION,
                    FIELD_CONSTRUCTION,
                    FIELD_FORCE_FIELD,
                    FIELD_PROPULSION,
                    FIELD_PLANETOLOGY,
                    FIELD_PLANETOLOGY,
                    FIELD_WEAPON,
                    FIELD_WEAPON,
                ][st.rng.below(9) as usize],
            }
        };
        sl[focus] = 75;
        let _ = apply(st, me, &Command::SetResearch { alloc: sl });
    }
    for f in 0..NUM_FIELDS {
        if st.empires[me as usize].tech.project[f] == 0 {
            tech::ai_choose(st, me, f);
        }
    }
}

fn set_economy(st: &mut State, me: EmpireId, r: &Roles, want_colony_ships: i32) {
    let e = &st.empires[me as usize];
    let a = e.abilities();
    let threatened: Vec<bool> = (0..st.planets.len())
        .map(|p| {
            st.fleets.iter().any(|f| {
                f.owner != me
                    && f.dest == p
                    && armed_only(&st.empires[f.owner as usize], &f.ships)
                        .iter()
                        .any(|&c| c > 0)
            }) || st
                .planets
                .iter()
                .enumerate()
                .any(|(q, pl)| pl.owner.is_some_and(|o| o != me) && st.parsecs(p, q) <= 4)
        })
        .collect();
    let mut colony_builders = 0;
    let mut by_prod: Vec<PlanetId> = st.colonies(me).map(|(i, _)| i).collect();
    by_prod.sort_by_key(|&i| -st.planets[i].prod);
    let mut orders = Vec::new();
    for &i in &by_prod {
        let p = &st.planets[i];
        let prod = p.prod.max(1);
        let mut eco = clean_eco_percent(p, &st.empires[me as usize]).min(60);
        if p.pop < p.max_pop / 2 {
            eco += 5;
        }
        let def = if threatened[i] && p.bases < 2 + st.turn as i32 / 15 {
            20
        } else if st.turn > 60 && p.bases < p.pop / 10 {
            5
        } else {
            0
        };
        let mut left = 100 - eco - def;
        let target_fact = p.max_pop * a.colonist_oper_factories;
        let wants_colony =
            r.colony.is_some() && colony_builders < want_colony_ships && p.prod >= 20;
        let (ship, ind);
        if p.factories < target_fact && p.pop > 0 {
            let s = if wants_colony || i == st.empires[me as usize].home {
                35
            } else {
                10
            };
            ship = left * s / 100;
            left -= ship;
            ind = left * 70 / 100;
            left -= ind;
        } else {
            ship = left * 50 / 100;
            left -= ship;
            ind = 0;
        }
        let tech = left.max(0);
        let build = if wants_colony {
            colony_builders += 1;
            r.colony
        } else if st.turn < 4 && r.scout.is_some() && i == st.empires[me as usize].home {
            r.scout
        } else if prod < 25 {
            r.cheap_warship.or(r.warship)
        } else if st.turn % 4 == 0 {
            r.bomber.or(r.warship)
        } else {
            r.warship.or(r.cheap_warship)
        };
        orders.push(Command::SetSliders {
            planet: i,
            sliders: [ship, def, ind, eco, tech],
        });
        if let Some(d) = build {
            orders.push(Command::SetBuild {
                planet: i,
                design: d,
            });
        }
    }
    for o in orders {
        let _ = apply(st, me, &o);
    }
}

fn colony_targets(st: &State, me: EmpireId, r: &Roles) -> Vec<(PlanetId, i64)> {
    let e = &st.empires[me as usize];
    let Some(cd) = r.colony else {
        return Vec::new();
    };
    let worst = e.designs[cd].colony_type().unwrap_or(PlanetType::Minimal);
    let range = e.abilities().fuel_range;
    let mut v: Vec<(PlanetId, i64)> = (0..st.planets.len())
        .filter(|&p| {
            let pl = &st.planets[p];
            e.explored[p]
                && can_colonize(st, me, p)
                && (pl.kind >= worst || e.race == Race::Silicoid)
                && st.range_to(me, p) <= range
                && !crate::combat::hostile_armed_at(st, p, me)
        })
        .map(|p| {
            let pl = &st.planets[p];
            let mult = match pl.special {
                Special::Rich | Special::Artifacts => 150,
                Special::UltraRich | Special::Orion => 200,
                Special::Poor => 70,
                Special::UltraPoor => 40,
                Special::Normal => 100,
            };
            let score = pl.max_pop as i64 * mult / (2 + st.range_to(me, p) as i64);
            (p, score)
        })
        .collect();
    v.sort_by_key(|&(p, s)| (-s, p));
    v
}

fn send(st: &mut State, me: EmpireId, from: PlanetId, to: PlanetId, ships: Vec<u32>) -> bool {
    apply(st, me, &Command::SendFleet { from, to, ships }).is_ok()
}

fn move_colony_ships(st: &mut State, me: EmpireId, r: &Roles, targets: &[(PlanetId, i64)]) {
    let Some(cd) = r.colony else { return };
    let mut taken: Vec<PlanetId> = st
        .fleets
        .iter()
        .filter(|f| f.owner == me && f.ships[cd] > 0)
        .map(|f| f.dest)
        .collect();
    for from in 0..st.planets.len() {
        let mut n = st.empires[me as usize].orbit[from][cd];
        if n > 0 && targets.iter().any(|&(t, _)| t == from) {
            taken.push(from);
            n -= 1;
        }
        for _ in 0..n {
            let Some(&(to, _)) = targets
                .iter()
                .find(|(t, _)| !taken.contains(t) && *t != from)
            else {
                break;
            };
            let mut ships = vec![0; MAX_DESIGNS];
            ships[cd] = 1;
            if send(st, me, from, to, ships) {
                taken.push(to);
            }
        }
    }
}

fn move_scouts(st: &mut State, me: EmpireId, r: &Roles) {
    let Some(sd) = r.scout else { return };
    let mut taken: Vec<PlanetId> = st
        .fleets
        .iter()
        .filter(|f| f.owner == me)
        .map(|f| f.dest)
        .collect();
    let mut ships = vec![0; MAX_DESIGNS];
    ships[sd] = 1;
    let range = fleet_range(st, me, &ships);
    for from in 0..st.planets.len() {
        for _ in 0..st.empires[me as usize].orbit[from][sd] {
            let e = &st.empires[me as usize];
            let best = (0..st.planets.len())
                .filter(|&p| !e.explored[p] && !taken.contains(&p) && st.range_to(me, p) <= range)
                .min_by_key(|&p| (st.dist(from, p) as i64, p));
            let Some(to) = best else { break };
            if send(st, me, from, to, ships.clone()) {
                taken.push(to);
            }
        }
    }
}

fn military(st: &mut State, me: EmpireId, need_conquer: bool) {
    let mi = me as usize;
    let home = st.empires[mi].home;
    let staging = if st.planets[home].owner == Some(me) {
        home
    } else {
        match st.colonies(me).max_by_key(|(_, p)| p.pop) {
            Some((i, _)) => i,
            None => return,
        }
    };
    for from in 0..st.planets.len() {
        if from == staging || st.planets[from].owner != Some(me) {
            continue;
        }
        let ships = armed_only(&st.empires[mi], &st.empires[mi].orbit[from]);
        if ships.iter().any(|&c| c > 0) {
            send(st, me, from, staging, ships);
        }
    }
    let force = armed_only(&st.empires[mi], &st.empires[mi].orbit[staging]);
    let mine = strength(&st.empires[mi], &force);
    if mine == 0 {
        return;
    }
    let range = fleet_range(st, me, &force);
    let already: Vec<PlanetId> = st
        .fleets
        .iter()
        .filter(|f| f.owner == me)
        .map(|f| f.dest)
        .collect();
    let target = (0..st.planets.len())
        .filter(|&p| {
            let pl = &st.planets[p];
            // 1oom turn_p1_planet_w: attack empires at war, or anyone without a treaty once
            // there is nothing left to colonize.
            let enemy = pl.owner.is_some_and(|o| {
                let t = crate::diplo::treaty(st, me, o);
                o != me
                    && (t >= crate::diplo::Treaty::War
                        || (need_conquer && t == crate::diplo::Treaty::None))
            });
            let orion = p == st.orion && st.guardian;
            st.empires[mi].explored[p]
                && (enemy || orion)
                && !already.contains(&p)
                && st.range_to(me, p) <= range
        })
        .map(|p| (p, defence_strength(st, mi, p)))
        .filter(|&(_, d)| mine > d * 3 + 50)
        .min_by_key(|&(p, d)| (d / 1000 + st.parsecs(staging, p) as i64 * 10, p));
    if let Some((to, _)) = target {
        send(st, me, staging, to, force);
    }
}

fn invade(st: &mut State, me: EmpireId) {
    let mi = me as usize;
    let range = st.empires[mi].abilities().fuel_range;
    for target in 0..st.planets.len() {
        let Some(owner) = st.planets[target].owner else {
            continue;
        };
        let holding = armed_only(&st.empires[mi], &st.empires[mi].orbit[target])
            .iter()
            .any(|&c| c > 0);
        if owner == me || !holding || st.planets[target].bases > 0 {
            continue;
        }
        if matches!(
            crate::diplo::treaty(st, me, owner),
            crate::diplo::Treaty::NonAggression | crate::diplo::Treaty::Alliance
        ) {
            continue;
        }
        if st
            .transports
            .iter()
            .any(|t| t.owner == me && t.dest == target)
        {
            continue;
        }
        let a = st.empires[mi].abilities().ground_bonus;
        let d = st.empires[owner as usize].abilities().ground_bonus + 5;
        let mut need = st.planets[target].pop * (100 + (d - a).max(0) * 2) / 100 * 3 / 2 + 2;
        let mut sources: Vec<PlanetId> = st
            .colonies(me)
            .filter(|(i, p)| p.pop >= 10 && st.parsecs(*i, target) <= range)
            .map(|(i, _)| i)
            .collect();
        sources.sort_by_key(|&i| -st.planets[i].pop);
        let available: i32 = sources.iter().map(|&i| st.planets[i].pop / 2).sum();
        if available < need {
            continue;
        }
        for from in sources {
            if need <= 0 {
                break;
            }
            let pop = (st.planets[from].pop / 2).min(need);
            if apply(
                st,
                me,
                &Command::SendTransport {
                    from,
                    to: target,
                    pop,
                },
            )
            .is_ok()
            {
                need -= pop;
            }
        }
    }
}

/// 1oom `game_ai_classic_turn_p1_tax` and `_fund_developing`.
fn taxes(st: &mut State, me: EmpireId) {
    let tax = if st.turn >= 20 {
        st.rng.one_to(10) as i32 + st.difficulty as i32 + 2
    } else {
        0
    };
    st.empires[me as usize].tax = tax;
    let small: Vec<PlanetId> = st
        .colonies(me)
        .filter(|(_, p)| p.pop < 20)
        .map(|(i, _)| i)
        .collect();
    for i in small {
        let v = st.empires[me as usize].reserve / 5;
        st.empires[me as usize].reserve -= v;
        st.planets[i].reserve += v;
    }
}

/// Gives this turn's orders for an AI empire.
pub fn play(st: &mut State, me: EmpireId) {
    if !st.empires[me as usize].alive {
        return;
    }
    research_focus(st, me);
    redesign(st, me);
    taxes(st, me);
    crate::ai_diplo::spying(st, me);
    let r = roles(st, me);
    let targets = colony_targets(st, me, &r);
    let have = r
        .colony
        .map_or(0, |c| crate::turn::ship_totals(st, me)[c] as i32);
    let want = (targets.len() as i32 - have).clamp(0, 2);
    move_colony_ships(st, me, &r, &targets);
    move_scouts(st, me, &r);
    military(st, me, targets.is_empty());
    invade(st, me);
    set_economy(st, me, &r, want);
}
