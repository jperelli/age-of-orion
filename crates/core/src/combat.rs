//! Space battles, orbital bombing, transports and ground combat.
//!
//! Battles follow 1oom `game_battle_handle_all`: two parties at a time per planet. A human
//! fights on the tactical grid (`battle.rs`, the turn pauses until the battle is over);
//! computers resolve their battles from the strength of both sides.
//! Bombing (`game_bomb.c`), shooting at transports (`game_turn_transport_shoot`) and
//! ground combat (`game_ground.c`) are direct ports.

use crate::battle::{self, Battle};
use crate::data::*;
use crate::design::WEAPON_SLOTS;
use crate::diplo::{self, Treaty};
use crate::events::{self, MonsterKind};
use crate::rng::Rng;
use crate::state::*;
use crate::types::*;

/// Battle parties: empires, or -1 the Guardian, -2 the crystal, -3 the amoeba.
const GUARDIAN: i32 = -1;

fn monster_party(kind: MonsterKind) -> i32 {
    match kind {
        MonsterKind::Crystal => -2,
        MonsterKind::Amoeba => -3,
    }
}

/// 1oom party order: empires first, then crystal, amoeba, Guardian.
fn party_order(p: i32) -> i32 {
    match p {
        -2 => 100,
        -3 => 101,
        -1 => 102,
        e => e,
    }
}

fn participants(st: &State, planet: PlanetId, ids: &[i32]) -> Vec<EmpireId> {
    let mut v: Vec<EmpireId> = ids.iter().filter(|&&i| i >= 0).map(|&i| i as u8).collect();
    if let Some(o) = st.planets[planet].owner {
        if !v.contains(&o) {
            v.push(o);
        }
    }
    v
}

/// Ships of a side that lost a battle retreat to the nearest own colony
/// (1oom `game_battle_post`), or are lost if there is none.
fn retreat(st: &mut State, empire: EmpireId, from: PlanetId) {
    let ships = st.empires[empire as usize].orbit[from].clone();
    let (fx, fy) = (st.planets[from].x, st.planets[from].y);
    let mut dest = None;
    let mut mindist = 10000;
    for (i, p) in st.planets.iter().enumerate() {
        if i != from && p.owner == Some(empire) {
            let d = crate::umath::dist_fast(fx, fy, p.x, p.y);
            if d < mindist {
                mindist = d;
                dest = Some(i);
            }
        }
    }
    if let Some(to) = dest {
        let pairs: Vec<(usize, u32)> = ships
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, c)| c > 0)
            .collect();
        crate::fleet::send_retreat(st, empire, from, to, &pairs);
    }
    st.empires[empire as usize].orbit[from]
        .iter_mut()
        .for_each(|c| *c = 0);
}

/// Parties with forces at a planet (1oom `tbl_have_force`): an owner with missile bases,
/// empires with ships, monsters there.
fn forces(st: &State, planet: PlanetId) -> Vec<i32> {
    let p = &st.planets[planet];
    let mut v: Vec<i32> = (0..st.empires.len())
        .filter(|&i| {
            (p.owner == Some(i as u8) && p.bases > 0)
                || st.empires[i].orbit[planet].iter().any(|&c| c > 0)
        })
        .map(|i| i as i32)
        .collect();
    if let Some(mi) = events::monster_at(st, planet) {
        v.push(monster_party(st.evn.monsters[mi].kind));
    }
    if planet == st.orion && st.guardian {
        v.push(GUARDIAN);
    }
    v.sort_by_key(|&p| party_order(p));
    v
}

fn is_human(st: &State, party: i32) -> bool {
    party >= 0 && !st.empires[party as usize].ai
}

/// 1oom `game_battle_handle_all`: battles at every planet, in turn, two parties at a time.
/// Returns true when a battle with a human player is waiting in `st.battle`.
pub fn battles(st: &mut State) -> bool {
    let (start, mut skipped) = st
        .progress
        .as_ref()
        .map(|p| (p.battle_planet, p.skipped.clone()))
        .unwrap_or((0, Vec::new()));
    for pli in start..st.planets.len() {
        let mut guard = 0;
        loop {
            guard += 1;
            if guard > 50 {
                break;
            }
            let f: Vec<i32> = forces(st, pli)
                .into_iter()
                .filter(|p| !skipped.contains(p))
                .collect();
            if f.len() < 2 {
                break;
            }
            let owner = st.planets[pli].owner.map(|o| o as i32);
            let def = match owner {
                Some(o) if f.contains(&o) => o,
                _ => f[0],
            };
            if def < 0 {
                break;
            }
            let att = *f.iter().find(|&&p| p != def).expect("two parties");
            if att >= 0 {
                let t = diplo::treaty(st, def as u8, att as u8);
                if t == Treaty::Alliance || (t == Treaty::NonAggression && Some(def) != owner) {
                    skipped.push(att);
                    continue;
                }
            }
            if !is_human(st, def) && !is_human(st, att) {
                resolve_ai(st, att, def, pli);
                continue;
            }
            if att >= 0 && !is_human(st, att) && diplo::rel(st, def as u8, att as u8).ceasefire > 0
            {
                skipped.push(att);
                retreat(st, att as u8, pli);
                continue;
            }
            let (l, r) = if is_human(st, def) {
                (def, att)
            } else {
                (att, def)
            };
            let bt = Battle::new_human(st, r, l, pli);
            st.battle = Some(Box::new(bt));
            if let Some(p) = st.progress.as_mut() {
                p.battle_planet = pli;
                p.skipped = skipped;
            }
            return true;
        }
        skipped.clear();
    }
    if let Some(p) = st.progress.as_mut() {
        p.battle_planet = st.planets.len();
        p.skipped.clear();
    }
    false
}

/// 1oom `game_ai_classic_battle_ai_ai_get_weights`.
fn weights(st: &State, party: i32) -> Vec<i32> {
    if party < 0 {
        return Vec::new();
    }
    const HULL_W: [i32; 4] = [1, 5, 25, 125];
    let e = &st.empires[party as usize];
    e.designs
        .iter()
        .map(|sd| {
            let mut w = sd.shield as i32 * 5
                + sd.comp as i32 * 5
                + sd.weapons[0] as i32
                + sd.armor as i32 * 10;
            w += sd.specials.iter().map(|&s| s as i32 * 2).sum::<i32>();
            w += e.tech.level[FIELD_CONSTRUCTION] as i32 / 2;
            if matches!(e.race, Race::Mrrshan | Race::Alkari) {
                w += if st.ai_id == 0 { 15 } else { 20 };
            }
            w.wrapping_mul(HULL_W[sd.hull.min(3) as usize])
        })
        .collect()
}

/// 1oom `game_ai_classic_battle_ai_ai_resolve`: computer against computer (or a monster),
/// resolved from the strength of both sides (32-bit, as MOO1), then `game_battle_finish`
/// and `_post`.
fn resolve_ai(st: &mut State, party_r: i32, party_l: i32, planet: PlanetId) {
    let mut bt = Battle::new(st, party_r, party_l, planet);
    let wl_tbl = weights(st, party_l);
    let wr_tbl = weights(st, party_r);
    let force = |st: &State, party: i32, tbl: &[i32]| -> i32 {
        if party < 0 {
            return if party == GUARDIAN { 100_000 } else { 30_000 };
        }
        let e = &st.empires[party as usize];
        let mut w: i32 = 0;
        for d in 0..e.designs.len() {
            let n = e.orbit[planet][d];
            if n > 0 {
                w = w.wrapping_add((n as i32).wrapping_mul(tbl[d]));
            }
        }
        w
    };
    let base = |st: &State, party: i32| -> i32 {
        let t = &st.empires[party as usize].tech.level;
        (t[FIELD_WEAPON] + t[FIELD_FORCE_FIELD]) as i32
    };
    let mut wl = if party_l >= 0 {
        force(st, party_l, &wl_tbl)
    } else {
        0
    };
    let mut wr = if party_r >= 0 {
        force(st, party_r, &wr_tbl)
    } else {
        0
    };
    match bt.planet_side {
        battle::Side::L => {
            wl = wl.wrapping_add(base(st, party_l).wrapping_mul(75).wrapping_mul(bt.bases))
        }
        battle::Side::R => {
            wr = wr.wrapping_add(base(st, party_r).wrapping_mul(75).wrapping_mul(bt.bases))
        }
        _ => {}
    }
    if party_l < 0 {
        wl = force(st, party_l, &[]);
    }
    if party_r < 0 {
        wr = force(st, party_r, &[]);
    }
    let (mut wl2, mut wr2) = (wl, wr);
    while wl2 > 0 && wr2 > 0 {
        let v = ((st.rng.one_to(10) as i32).wrapping_mul(wl) / 100).max(1);
        wr2 -= v;
        let v = ((st.rng.one_to(10) as i32).wrapping_mul(wr) / 100).max(1);
        wl2 -= v;
    }
    let (wl2, wr2) = (wl2.max(0), wr2.max(0));
    let r_won = wr2 > 0;
    for (k, party, ws, ws2) in [(0, party_l, wl, wl2), (1, party_r, wr, wr2)] {
        if party < 0 {
            continue;
        }
        let orbit = st.empires[party as usize].orbit[planet].clone();
        bt.s[k].ships = orbit
            .iter()
            .map(|&n| {
                if ws == 0 {
                    0
                } else {
                    // 32-bit as MOO1: large fleets overflow (to be capped next turn).
                    ((n as i32).wrapping_mul(ws2) / ws) as u16 as u32
                }
            })
            .collect();
    }
    if st.ai_id != 0 {
        bt.bases = match bt.planet_side {
            battle::Side::L if wl > 0 => (bt.bases as i64 * wl2 as i64 / wl as i64) as i32,
            battle::Side::R if wr > 0 => (bt.bases as i64 * wr2 as i64 / wr as i64) as i32,
            battle::Side::None => 0,
            _ => 0,
        };
    }
    bt.winner = Some(if r_won {
        battle::Side::R
    } else {
        battle::Side::L
    });
    apply(st, &bt);
}

/// 1oom `game_battle_finish` and `game_battle_post`: puts the result of a finished battle
/// into the game.
pub fn apply(st: &mut State, bt: &Battle) {
    let planet = bt.planet;
    let winner = bt.winner.unwrap_or(battle::Side::R);
    let bases_before = st.planets[planet].bases;
    if bt.planet_side != battle::Side::None {
        let p = &mut st.planets[planet];
        p.max_pop = (p.max_pop - bt.biodamage).max(10);
        p.bases = bt.bases;
        if p.pop == 0 && bt.s[1].party >= 0 {
            destroy_colony(st, planet);
        }
    }
    let mut losses = Vec::new();
    let mut lost_force = [0i32; 2];
    for k in 0..2 {
        let party = bt.s[k].party;
        if party < 0 {
            let dead = (k == 1 && winner == battle::Side::L) as u32;
            losses.push(dead);
            continue;
        }
        let e = party as usize;
        let mut lost = 0;
        let mut remaining_force = 0;
        for d in 0..st.empires[e].designs.len() {
            let before = st.empires[e].orbit[planet][d];
            if before == 0 {
                continue;
            }
            let after = bt.s[k].ships.get(d).copied().unwrap_or(0);
            lost += before.saturating_sub(after);
            remaining_force += (st.empires[e].designs[d].hull as i32 + 1) * after as i32;
            st.empires[e].orbit[planet][d] = after;
        }
        lost_force[k] = bt.s[k].apparent_force - remaining_force;
        losses.push(lost);
    }
    let (pl, pr) = (bt.s[0].party, bt.s[1].party);
    if pl >= 0 && pr >= 0 {
        let pop_killed = bt.pop - st.planets[planet].pop;
        diplo::battle_finish(
            st,
            pl as u8,
            pr as u8,
            pop_killed,
            lost_force[0],
            bt.biodamage,
            lost_force[1],
            planet,
        );
    }
    let (loser, win) = if winner == battle::Side::R {
        (pl, pr)
    } else {
        (pr, pl)
    };
    match loser {
        GUARDIAN => {
            st.guardian = false;
            // The loot comes with the news (1oom `game_event_run`, `guardian_killer`).
            st.guardian_killer = (win >= 0).then_some(win as u8);
        }
        -2 | -3 => {
            if let Some(m) = st
                .evn
                .monsters
                .iter_mut()
                .find(|m| monster_party(m.kind) == loser && m.dest == planet)
            {
                m.killer = (win >= 0).then_some(win as u8);
            }
        }
        e => retreat(st, e as u8, planet),
    }
    // A wounded monster keeps its damage.
    if pr <= -2 && winner == battle::Side::R {
        if let Some(it) = bt
            .item
            .iter()
            .find(|i| i.side == battle::Side::R && i.design.is_none())
        {
            if let Some(m) = st
                .evn
                .monsters
                .iter_mut()
                .find(|m| monster_party(m.kind) == pr && m.dest == planet)
            {
                m.hp = (it.hp1 * it.num.max(1) - it.hploss).max(1) as u32;
            }
        }
    }
    let sides = vec![pl, pr];
    st.push_event(
        participants(st, planet, &sides),
        EventKind::Battle {
            planet,
            sides,
            winner: Some(win),
            losses,
            bases_lost: bases_before - st.planets[planet].bases,
        },
    );
}

/// Ends the current human battle once it is over: applies its result.
pub fn finish_battle(st: &mut State) {
    let Some(mut bt) = st.battle.take() else {
        return;
    };
    if bt.winner.is_some() && !bt.applied {
        let p = &mut st.planets[bt.planet];
        p.pop = bt.pop;
        p.factories = bt.fact;
        bt.bases = bt.item[0].num.max(0);
        if bt.planet_side == battle::Side::None {
            bt.bases = st.planets[bt.planet].bases;
        }
        apply(st, &bt);
        bt.applied = true;
    }
    st.battle = Some(bt);
}

/// 1oom `game_turn_bomb_damage`: (population killed, factories destroyed, bio damage).
fn bomb_damage(st: &mut State, planet: PlanetId, attacker: EmpireId) -> (i32, i32, i32) {
    let p = &st.planets[planet];
    let owner = p.owner.expect("owned planet");
    let pshield = p.shield;
    let antidote = st.empires[owner as usize].abilities().antidote;
    let e = &st.empires[attacker as usize];
    let mut tbl = vec![0u32; WEAPONS.len()];
    let mut maxcomp = 0;
    for (d, sd) in e.designs.iter().enumerate() {
        let n = e.orbit[planet][d];
        if n == 0 {
            continue;
        }
        maxcomp = maxcomp.max(sd.comp as i32);
        for s in 0..WEAPON_SLOTS {
            let v = n * sd.weapon_counts[s] as u32;
            if v != 0 {
                let ns = WEAPONS[sd.weapons[s] as usize].shots;
                tbl[sd.weapons[s] as usize] += v * if ns == -1 { 30 } else { ns as u32 };
            }
        }
    }
    let complevel = ((maxcomp - 1) * 2 + 6).clamp(1, 20) as u16;
    let (mut total, mut bio) = (0i64, 0i32);
    let rng = &mut st.rng;
    for (i, &count) in tbl.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let w = &WEAPONS[i];
        let mut vcur = count.min(st.rules.max_bomb_dmg);
        let v = if vcur < 10 {
            1
        } else {
            let v = vcur / 10;
            vcur = 10;
            v
        };
        let (dmin, mut dmax) = (w.damage_min as i32, w.damage_max as i32);
        if dmin == dmax {
            if w.bio {
                for _ in 0..vcur {
                    if rng.one_to(20) <= complevel {
                        let dmg = (rng.below(dmin as u16 + 1) as i32 - antidote).max(0);
                        bio = (bio + v as i32 * dmg).min(st.rules.max_bio_dmg);
                    }
                }
            } else {
                if w.miss_type != 0 {
                    dmax /= 2;
                }
                dmax *= w.streams as i32;
                if dmax > pshield {
                    for _ in 0..vcur {
                        if rng.one_to(20) <= complevel {
                            total += v as i64 * (dmax - pshield) as i64;
                        }
                    }
                }
            }
        } else {
            let (mut dmin, mut dmax) = (dmin, dmax);
            if !w.bomb {
                dmin /= 2;
                dmax /= 2;
            }
            if dmax > pshield {
                let range = (dmax - dmin + 1) as u16;
                let base = dmin - 1 - pshield;
                let shots = complevel as u32 * vcur * v / 20;
                for _ in 0..shots {
                    let dmg = rng.one_to(range) as i32 + base;
                    if dmg > 0 {
                        total += dmg as i64;
                    }
                }
            }
        }
    }
    let p = &st.planets[planet];
    let roll = |rng: &mut Rng, hp: i32| {
        let h = hp / 5;
        (rng.one_to((h * 2).max(1) as u16) as i32 + hp - h).max(1)
    };
    let fact_hp = st.rules.fact_hp;
    let pop_hp = st.rules.pop_hp;
    let fact = ((total / roll(&mut st.rng, fact_hp) as i64) as i32).min(p.factories);
    let pop = ((total / roll(&mut st.rng, pop_hp) as i64) as i32 + bio).min(p.pop);
    (pop, fact, bio)
}

/// 1oom `game_turn_bomb`: ships in orbit of an enemy colony may bomb it.
pub fn bomb(st: &mut State) {
    for planet in 0..st.planets.len() {
        for ei in 0..st.empires.len() {
            let Some(owner) = st.planets[planet].owner else {
                break;
            };
            if ei == owner as usize || st.empires[ei].orbit[planet].iter().all(|&c| c == 0) {
                continue;
            }
            // 1oom game_turn_bomb: allies never bomb, computers respect pacts.
            let t = diplo::treaty(st, ei as u8, owner);
            if t == Treaty::Alliance || (t == Treaty::NonAggression && st.empires[ei].ai) {
                continue;
            }
            let (pop_dmg, fact_dmg, bio) = bomb_damage(st, planet, ei as u8);
            if pop_dmg == 0 && fact_dmg == 0 {
                continue;
            }
            let e = &st.empires[ei];
            let inbound: i32 = st
                .transports
                .iter()
                .filter(|t| t.owner as usize == ei && t.dest == planet)
                .map(|t| t.pop)
                .sum();
            let go = if e.ai {
                crate::ai::bomb(st, ei as u8, planet, inbound)
            } else {
                e.bombard
            };
            if !go {
                continue;
            }
            let p = &mut st.planets[planet];
            if bio > 0 {
                p.max_pop = (p.max_pop - bio).max(10);
            }
            p.pop -= pop_dmg;
            p.factories -= fact_dmg;
            st.push_event(
                vec![ei as u8, owner],
                EventKind::Bombed {
                    planet,
                    attacker: ei as u8,
                    pop_killed: pop_dmg,
                    factories_destroyed: fact_dmg,
                },
            );
            let destroyed = st.planets[planet].pop <= 0;
            if destroyed {
                destroy_colony(st, planet);
            }
            if destroyed && !st.empires[ei].ai {
                let dv = if diplo::treaty(st, ei as u8, owner) < Treaty::War {
                    diplo::start_war(st, owner, ei as u8);
                    13
                } else {
                    10
                };
                let v = -50 - st.rng.one_to(50) as i32;
                diplo::act(st, v, ei as u8, owner, dv, planet, 0);
            } else {
                diplo::battle_finish(st, owner, ei as u8, pop_dmg, 0, bio, 0, planet);
            }
        }
    }
}

fn destroy_colony(st: &mut State, planet: PlanetId) {
    let p = &mut st.planets[planet];
    let Some(owner) = p.owner.take() else { return };
    p.pop = 0;
    p.factories = 0;
    p.bases = 0;
    p.shield = 0;
    p.waste = 0;
    p.stargate = false;
    st.push_event(vec![], EventKind::ColonyDestroyed { planet, owner });
}
