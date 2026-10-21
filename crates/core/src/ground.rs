//! Transports landing and ground combat: 1oom `game_turn_transport`,
//! `game_turn_transport_shoot` and `game_ground.c` (`game_turn_ground_resolve_all`).

use crate::data::{COMPS, WEAPONS};
use crate::diplo::{self, Treaty};
use crate::rules::{
    group_tier, GROUP_ARMOR, GROUP_PERSONAL_ARMOR, GROUP_PERSONAL_SHIELD, GROUP_PERSONAL_WEAPONS,
};
use crate::state::*;
use crate::tech::{self, Source};
use crate::types::*;

const MAX_TRANS_DMG: u32 = 32000;
const MAX_INBOUND: i32 = 300;

/// 1oom `game_turn_transport_shoot`: transports killed by the planet's bases or by ships
/// in orbit.
#[allow(clippy::too_many_arguments)]
fn transport_shoot(
    st: &mut State,
    planet: PlanetId,
    rowner: EmpireId,
    num: i32,
    speed: i32,
    attacker: EmpireId,
    bases: i32,
    basewpnt: usize,
) -> i32 {
    let ed = &st.empires[rowner as usize];
    let bestarmor = group_tier(&ed.tech, FIELD_CONSTRUCTION, GROUP_ARMOR).0;
    let transport_hp = (bestarmor + 1) * 15;
    let total_hp = transport_hp * num;
    let ea = &st.empires[attacker as usize];
    let mut tbl = vec![0u32; WEAPONS.len()];
    tbl[basewpnt] = (bases * 24) as u32;
    let mut bestcomp: i32 = 0;
    for (i, sd) in ea.designs.iter().enumerate() {
        let comp = sd.comp as i32;
        if comp > speed {
            bestcomp = comp;
        }
        for j in 0..4 {
            let wpnt = sd.weapons[j] as usize;
            let mut v = ea.orbit[planet][i].wrapping_mul(sd.weapon_counts[j] as u32);
            if v != 0 {
                let w = &WEAPONS[wpnt];
                let ns = if w.shots == -1 { 4 } else { w.shots as u32 };
                v = v.wrapping_mul(ns);
                if w.streams != 0 {
                    v = v.wrapping_mul(w.streams as u32);
                }
                v = v.min(MAX_TRANS_DMG);
                if w.bomb || w.bio {
                    v = 0;
                }
                if w.miss_type != 0 {
                    v /= 2;
                }
                tbl[wpnt] = tbl[wpnt].wrapping_add(v * 2 / speed.max(1) as u32);
            }
        }
    }
    if bases > 0 {
        let e = &st.empires[rowner as usize];
        let level = e.tech.level[FIELD_COMPUTER];
        bestcomp = (0..COMPS.len())
            .filter(|&i| COMPS[i].tech as u32 <= level && e.tech.has(FIELD_COMPUTER, COMPS[i].tech))
            .max()
            .unwrap_or(0) as i32;
    }
    let complevel = ((bestcomp - speed) * 2 + 12).clamp(1, 20);
    let mut totaldmg: i32 = 0;
    // 1oom always counts every weapon here (MOO1 v1.3a fix).
    for i in 0..WEAPONS.len() {
        let vcur = tbl[i];
        if vcur == 0 {
            continue;
        }
        let w = &WEAPONS[i];
        let (dmin, dmax) = (w.damage_min as i32, w.damage_max as i32);
        if dmin == dmax {
            for _ in 0..vcur {
                if st.rng.one_to(20) as i32 <= complevel {
                    totaldmg += dmax;
                }
            }
        } else {
            let range = dmax - dmin + 1;
            let dmin = dmin - 1;
            let n = (complevel as u32).wrapping_mul(vcur) / 20;
            for _ in 0..n {
                let dmg = st.rng.one_to(range as u16) as i32 + dmin;
                if dmg > 0 {
                    totaldmg += dmg;
                }
            }
        }
        if totaldmg >= total_hp {
            break;
        }
    }
    let mut killed = (totaldmg / transport_hp).min(1000);
    let p = &st.planets[planet];
    for m in &st.evn.monsters {
        if m.killer.is_none() && m.x == p.x && m.y == p.y {
            killed = 1000;
        }
    }
    killed
}

/// 1oom `game_turn_transport`: transports that arrived are shot at, then land.
pub fn land(st: &mut State) {
    let n = st.empires.len();
    for p in st.planets.iter_mut() {
        p.inbound = vec![0; PLAYER_NUM];
        p.total_inbound = vec![0; PLAYER_NUM];
    }
    let mut i = 0;
    while i < st.transports.len() {
        let t = st.transports[i].clone();
        let dest = t.dest;
        let (px, py) = (st.planets[dest].x, st.planets[dest].y);
        if !(t.x == px && t.y == py) {
            i += 1;
            continue;
        }
        let owner = t.owner;
        let pop2 = t.pop;
        let mut pop3 = t.pop;
        for j in 0..n {
            if j == owner as usize {
                continue;
            }
            let tr = st.empires[owner as usize].rel[j].treaty;
            let powner = st.planets[dest].owner;
            if Some(j as u8) == powner || tr == Treaty::None || tr >= Treaty::War {
                if Some(j as u8) == powner {
                    let (bases, w) = (st.planets[dest].bases, st.empires[j].base_weapon);
                    pop3 -= transport_shoot(st, dest, owner, pop3, t.speed, j as u8, bases, w);
                } else {
                    let e = &st.empires[j];
                    if (0..e.designs.len()).any(|k| e.orbit[dest][k] > 0) {
                        pop3 -= transport_shoot(st, dest, owner, pop3, t.speed, j as u8, 0, 0);
                    }
                }
            }
        }
        if st.guardian && dest == st.orion {
            pop3 = 0;
        }
        for m in &st.evn.monsters {
            if m.x == px && m.y == py {
                pop3 = 0;
            }
        }
        pop3 = pop3.max(0);
        if st.empires[owner as usize].ab.combat_transporters {
            let lost = pop2 - pop3;
            let interdictor = st.planets[dest]
                .owner
                .is_some_and(|o| st.empires[o as usize].ab.sub_space_interdictor);
            if interdictor {
                // MOO1 bug kept (1oom `game_num_combat_trans_fix` off): transporters only
                // work when the planet owner has an interdictor, then save 62.5% of the
                // transports shot down. See docs/MOO1_QUIRKS.md.
                for _ in 0..lost {
                    if st.rng.below(4) == 0 || st.rng.below(2) == 0 {
                        pop3 += 1;
                    }
                }
            }
        }
        let powner = st.planets[dest].owner;
        if pop3 <= 0 {
            let to = std::iter::once(owner).chain(powner).collect();
            st.push_event(
                to,
                EventKind::TransportsLost {
                    planet: dest,
                    owner,
                    pop: pop2,
                },
            );
        } else if powner.is_none() {
            st.push_event(
                vec![owner],
                EventKind::TransportsLost {
                    planet: dest,
                    owner,
                    pop: pop3,
                },
            );
        } else {
            let po = powner.unwrap();
            let p = &mut st.planets[dest];
            if p.pop == 0 {
                p.pop = pop3;
                p.bc_to_refit = 0;
                p.pop_oper_fact = 1;
                p.owner = Some(owner);
            } else if po == owner {
                if p.unrest == Unrest::Rebellion {
                    p.inbound[owner as usize] = (p.inbound[owner as usize] + pop3).min(MAX_INBOUND);
                    p.total_inbound[owner as usize] += pop3;
                } else {
                    p.pop = (p.pop + pop3).min(p.max_pop);
                }
            } else if st.empires[owner as usize].rel[po as usize].treaty != Treaty::Alliance {
                let p = &mut st.planets[dest];
                p.inbound[owner as usize] = (p.inbound[owner as usize] + pop3).min(MAX_INBOUND);
                p.total_inbound[owner as usize] += pop2;
            }
        }
        st.transports.swap_remove(i);
    }
}

struct Side {
    player: EmpireId,
    human: bool,
    force: i32,
    pop1: i32,
}

/// 1oom `game_ground_resolve_init`: the ground forces' bonuses.
fn resolve_init(st: &State, s: &mut [Side; 2], flag_swap: bool) {
    for side in s.iter_mut() {
        let e = &st.empires[side.player as usize];
        side.human = !e.ai;
        let armor = group_tier(&e.tech, FIELD_CONSTRUCTION, GROUP_ARMOR).0;
        let suit = group_tier(&e.tech, FIELD_CONSTRUCTION, GROUP_PERSONAL_ARMOR).0;
        side.force = armor * 5 + suit * 10;
        let shield = group_tier(&e.tech, FIELD_FORCE_FIELD, GROUP_PERSONAL_SHIELD).0;
        if shield != 0 {
            side.force += shield * 10;
        }
        let mut weap = group_tier(&e.tech, FIELD_WEAPON, GROUP_PERSONAL_WEAPONS).0;
        if weap != 0 {
            if weap > 2 {
                weap += 1;
            }
            side.force += weap * 5;
        }
        if e.race == Race::Bulrathi {
            side.force += 25;
        }
    }
    s[if flag_swap { 0 } else { 1 }].force += 5;
}

/// 1oom `game_ai_classic_ground`: what the defender thinks of the invasion.
fn ai_ground(
    st: &mut State,
    def: EmpireId,
    att: EmpireId,
    planet: PlanetId,
    pop_killed: i32,
    owner_changed: bool,
) {
    if !st.empires[def as usize].ai {
        return;
    }
    if st.empires[att as usize].ai {
        // MOO1: computer against computer always counts one colonist killed.
        let v = -(st.rng.one_to(4) as i32 + 4);
        diplo::act(st, v, att, def, 0xa, planet, 0);
    } else if owner_changed {
        if st.empires[att as usize].rel[def as usize].treaty < Treaty::War {
            let v = -50 - st.rng.one_to(50) as i32;
            diplo::act(st, v, att, def, 0xd, planet, 0);
            // 1oom `game_diplo_start_war_swap`.
            if st.ai_id == 0 {
                diplo::start_war(st, def, att);
            } else {
                diplo::start_war(st, att, def);
            }
        } else {
            let v = -50 - st.rng.one_to(50) as i32;
            diplo::act(st, v, att, def, 0xa, planet, 0);
        }
    } else {
        let v = -((st.rng.one_to(5) as i32 + 5) * pop_killed);
        diplo::act(st, v, att, def, 0xa, planet, 0);
    }
}

/// 1oom `game_turn_ground_resolve_all`: invaders fight the defenders (or rebels) to the
/// last; the winner of an invasion may capture technology from the factories.
pub fn resolve_all(st: &mut State) {
    let n = st.empires.len();
    for pli in 0..st.planets.len() {
        let mut powner = st.planets[pli].owner;
        for i in 0..n {
            let rebellion = st.planets[pli].unrest == Unrest::Rebellion;
            if Some(i as u8) == powner && !rebellion {
                continue;
            }
            let inbound = st.planets[pli].inbound.get(i).copied().unwrap_or(0);
            if let (Some(po), true) = (powner, inbound > 0) {
                let p = &st.planets[pli];
                let flag_rebel = rebellion && !st.empires[i].ai && p.owner == Some(i as u8);
                let pop_planet0 = if flag_rebel { p.rebels } else { p.pop };
                let mut s = [
                    Side {
                        player: i as u8,
                        human: false,
                        force: 0,
                        pop1: inbound,
                    },
                    Side {
                        player: po,
                        human: false,
                        force: 0,
                        pop1: pop_planet0,
                    },
                ];
                let flag_swap = !st.empires[i].ai || !st.empires[po as usize].ai;
                if flag_swap {
                    s.swap(0, 1);
                }
                resolve_init(st, &mut s, flag_swap);
                let fact = st.planets[pli].factories;
                let mut seed = st.rng;
                if s[0].pop1 != 0 && s[1].pop1 != 0 {
                    let human = s[0].human || s[1].human;
                    let att_side = usize::from(flag_swap);
                    let start = if flag_swap {
                        [s[1].pop1, s[0].pop1]
                    } else {
                        [s[0].pop1, s[1].pop1]
                    };
                    let mut deaths = Vec::new();
                    while s[0].pop1 != 0 && s[1].pop1 != 0 {
                        // 1oom rolls on a copy of the seed (not in its random number log).
                        let v1 = seed.one_to_quiet(100) as i32 + s[0].force;
                        let v2 = seed.one_to_quiet(100) as i32 + s[1].force;
                        let death = if v1 <= v2 { 0 } else { 1 };
                        s[death].pop1 -= 1;
                        if human {
                            deaths.push(u8::from(death != att_side));
                        }
                    }
                    if flag_swap {
                        s.swap(0, 1);
                    }
                    st.rng = seed;
                    let p = &st.planets[pli];
                    let info = Report {
                        start,
                        deaths,
                        landed_of: p.total_inbound.get(i).copied().unwrap_or(0),
                    };
                    finish(st, pli, &s, flag_rebel, fact, info);
                    let mut pop_planet = pop_planet0 - s[1].pop1;
                    pop_planet = pop_planet.max(1);
                    let changed = st.planets[pli].owner != powner;
                    ai_ground(st, s[1].player, s[0].player, pli, pop_planet, changed);
                }
            }
            powner = st.planets[pli].owner;
        }
    }
}

/// What the ground battle screen shows.
struct Report {
    start: [i32; 2],
    deaths: Vec<u8>,
    landed_of: i32,
}

/// Battle suit, personal shield and hand weapon of an empire's troops, tech 0 for none
/// (1oom `game_ground_resolve_init`, for the ground battle screen).
fn gear(st: &State, e: EmpireId) -> Vec<(u8, u8)> {
    let t = &st.empires[e as usize].tech;
    [
        (FIELD_CONSTRUCTION, GROUP_PERSONAL_ARMOR),
        (FIELD_FORCE_FIELD, GROUP_PERSONAL_SHIELD),
        (FIELD_WEAPON, GROUP_PERSONAL_WEAPONS),
    ]
    .into_iter()
    .map(|(f, g)| (f as u8, group_tier(t, f, g).1))
    .collect()
}

/// 1oom `game_ground_finish`.
fn finish(st: &mut State, pli: PlanetId, s: &[Side; 2], flag_rebel: bool, fact: i32, info: Report) {
    let (att, def) = (s[0].player, s[1].player);
    let won = s[0].pop1 > 0;
    let mut captured = Vec::new();
    if won {
        if flag_rebel {
            let p = &mut st.planets[pli];
            p.unrest = Unrest::Resolved;
            p.pop = p.pop - p.rebels + s[0].pop1;
            p.rebels = 0;
            st.push_event(vec![att], EventKind::RebelsDefeated { planet: pli });
        } else {
            let mut chance = 0;
            for _ in 0..fact {
                if st.rng.below(50) == 0 {
                    chance += 1;
                }
            }
            let offers = crate::spy::pick_techs(st, att, def, 0, 0);
            let num = offers.len().min(chance);
            for o in offers.iter().take(num) {
                captured.push((o.field, o.tech));
                tech::grant_from(
                    st,
                    att,
                    o.field as usize,
                    o.tech,
                    Source::Espionage,
                    Some(def),
                );
            }
            crate::misc::planet_destroy(st, pli, Some(att));
            let p = &mut st.planets[pli];
            p.owner = Some(att);
            p.pop = s[0].pop1;
            p.bc_to_refit = 0;
            p.pop_oper_fact = 1;
            st.empires[att as usize].explored[pli] = true;
            st.push_event(
                vec![],
                EventKind::PlanetCaptured {
                    planet: pli,
                    from: def,
                    by: att,
                },
            );
        }
    } else if flag_rebel {
        let p = &mut st.planets[pli];
        p.rebels = s[1].pop1;
        if p.rebels == 0 {
            p.unrest = Unrest::Resolved;
        }
    } else {
        st.planets[pli].pop = s[1].pop1;
    }
    let p = &mut st.planets[pli];
    p.pop = p.pop.min(p.max_pop);
    let g = [gear(st, att), gear(st, def)];
    let armor = [att, def].map(|e| {
        group_tier(
            &st.empires[e as usize].tech,
            FIELD_CONSTRUCTION,
            GROUP_ARMOR,
        )
        .0 as u8
    });
    st.push_event(
        vec![att, def],
        EventKind::GroundBattle {
            planet: pli,
            attacker: att,
            defender: def,
            won,
            attackers_left: s[0].pop1,
            defenders_left: s[1].pop1,
            attackers: info.start[0],
            defenders: info.start[1],
            landed_of: info.landed_of,
            rebels: flag_rebel,
            deaths: info.deaths,
            gear: g,
            armor,
            captured,
            factories: if won && !flag_rebel { fact } else { 0 },
        },
    );
}
