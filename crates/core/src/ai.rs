//! Computer players. A compact strategy written for this port (the 4,500-line classic AI
//! of 1oom `game_ai_classic.c` is on the roadmap): expand with colony ships, scout, build
//! industry, research, then attack weak neighbours and invade them with transports.

use crate::commands::{apply, fleet_range, Command};
use crate::state::*;
use crate::turn::{can_colonize, clean_eco_percent};
use crate::types::*;

struct Roles {
    scout: Option<usize>,
    colony: Option<usize>,
    bomber: Option<usize>,
    warship: Option<usize>,
    cheap_warship: Option<usize>,
}

fn roles(st: &State) -> Roles {
    let d = &st.rules.designs;
    let find = |f: &dyn Fn(&crate::rules::Design) -> bool| (0..d.len()).find(|&i| f(&d[i]));
    let armed: Vec<usize> = (0..d.len())
        .filter(|&i| d[i].attack > 0 && !d[i].colony && d[i].bombs == 0)
        .collect();
    Roles {
        scout: find(&|x| x.extra_range > 0 && !x.armed()),
        colony: find(&|x| x.colony),
        bomber: find(&|x| x.bombs > 0),
        warship: armed
            .iter()
            .copied()
            .max_by_key(|&i| d[i].attack * d[i].hp * 100 / d[i].cost.max(1) + d[i].cost),
        cheap_warship: armed.iter().copied().min_by_key(|&i| d[i].cost),
    }
}

/// Combat strength used to compare forces: total attack x total hit points.
fn strength(st: &State, empire: usize, ships: &[u32]) -> i64 {
    let a = st.empires[empire].abilities();
    let (mut att, mut hp) = (0i64, 0i64);
    for (d, &c) in ships.iter().enumerate() {
        let des = &st.rules.designs[d];
        att += (des.attack.max(des.bombs / 4) * a.attack_pct / 100) as i64 * c as i64;
        hp += (des.hp * a.hp_pct / 100) as i64 * c as i64;
    }
    att * hp
}

fn defence_strength(st: &State, me: usize, planet: PlanetId) -> i64 {
    let p = &st.planets[planet];
    let mut s: i64 = 0;
    if planet == st.orion && st.guardian_alive() {
        s += st.rules.guardian_attack as i64 * st.guardian_hp as i64;
    }
    if let Some(o) = p.owner.filter(|&o| o as usize != me) {
        let a = st.empires[o as usize].abilities();
        let b = p.bases as i64;
        s += (b * (st.rules.base_attack * a.attack_pct / 100) as i64)
            * (b * (st.rules.base_hp * a.hp_pct / 100) as i64);
    }
    for (ei, e) in st.empires.iter().enumerate() {
        if ei != me {
            s += strength(st, ei, &e.orbit[planet]);
        }
    }
    s
}

fn armed_only(st: &State, ships: &[u32]) -> Vec<u32> {
    ships
        .iter()
        .zip(&st.rules.designs)
        .map(|(&c, d)| if d.armed() && !d.colony { c } else { 0 })
        .collect()
}

fn set_economy(st: &mut State, me: EmpireId, r: &Roles, want_colony_ships: i32) {
    let e = &st.empires[me as usize];
    let a = e.abilities();
    let threatened: Vec<bool> = (0..st.planets.len())
        .map(|p| {
            st.fleets.iter().any(|f| {
                f.owner != me && f.dest == p && armed_only(st, &f.ships).iter().any(|&c| c > 0)
            }) || st
                .planets
                .iter()
                .enumerate()
                .any(|(q, pl)| pl.owner.is_some_and(|o| o != me) && st.parsecs(p, q) <= 4)
        })
        .collect();
    let mut colony_builders = 0;
    let colonies: Vec<PlanetId> = st.colonies(me).map(|(i, _)| i).collect();
    let mut by_prod = colonies.clone();
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
        let wants_colony = colony_builders < want_colony_ships && p.prod >= 20;
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
            r.cheap_warship
        } else if st.turn % 4 == 0 {
            r.bomber.or(r.warship)
        } else {
            r.warship
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

fn colony_targets(st: &State, me: EmpireId) -> Vec<(PlanetId, i64)> {
    let e = &st.empires[me as usize];
    let range = e.abilities().fuel_range;
    let mut v: Vec<(PlanetId, i64)> = (0..st.planets.len())
        .filter(|&p| {
            e.explored[p]
                && can_colonize(st, me, p)
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
        if n > 0 && can_colonize(st, me, from) {
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
            let mut ships = vec![0; st.rules.designs.len()];
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
    let mut ships = vec![0; st.rules.designs.len()];
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

fn military(st: &mut State, me: EmpireId) {
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
    // Gather armed ships idling at own colonies at the staging planet.
    for from in 0..st.planets.len() {
        if from == staging || st.planets[from].owner != Some(me) {
            continue;
        }
        let ships = armed_only(st, &st.empires[mi].orbit[from]);
        if ships.iter().any(|&c| c > 0) {
            send(st, me, from, staging, ships);
        }
    }
    let force = armed_only(st, &st.empires[mi].orbit[staging]);
    let mine = strength(st, mi, &force);
    if mine == 0 {
        return;
    }
    let range = st.empires[mi].abilities().fuel_range;
    let already: Vec<PlanetId> = st
        .fleets
        .iter()
        .filter(|f| f.owner == me)
        .map(|f| f.dest)
        .collect();
    let target = (0..st.planets.len())
        .filter(|&p| {
            let pl = &st.planets[p];
            let enemy = pl.owner.is_some_and(|o| o != me);
            let orion = p == st.orion && st.guardian_alive();
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
        let holding = armed_only(st, &st.empires[mi].orbit[target])
            .iter()
            .any(|&c| c > 0);
        if owner == me || !holding || st.planets[target].bases > 0 {
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

/// Gives this turn's orders for an AI empire.
pub fn play(st: &mut State, me: EmpireId) {
    if !st.empires[me as usize].alive {
        return;
    }
    let r = roles(st);
    let targets = colony_targets(st, me);
    let have = r.colony.map_or(0, |c| {
        let t = crate::turn::ship_totals(st, me);
        t[c] as i32
    });
    let want = (targets.len() as i32 - have).clamp(0, 2);
    move_colony_ships(st, me, &r, &targets);
    move_scouts(st, me, &r);
    military(st, me);
    invade(st, me);
    set_economy(st, me, &r, want);
    let w = st.rules.research_weights_ai;
    let _ = apply(st, me, &Command::SetResearch { alloc: w });
}
