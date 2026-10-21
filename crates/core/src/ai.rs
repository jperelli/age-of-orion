//! The classic computer players' turn: fleets, colonization, transports, production
//! sliders, ship design and spending (1oom `game_ai_classic.c`: `turn_p1`, `turn_p2`,
//! `turn_p3`, `new_game_init`, `tech_next`). `ai_id` 0 is MOO1's AI (with its bugs, as
//! 1oom "Classic"), 1 is 1oom's "Classic+". Diplomacy is in `ai_diplo.rs`, battles in
//! `battle_ai.rs`.

use crate::data::{ARMOR, COMPS, ENGINES, JAMMERS, SHIELDS, SPECIALS, WEAPONS};
use crate::design::{ShipDesign, BUILD_STARGATE, HULL_LARGE, HULL_MEDIUM, HULL_SMALL, MAX_DESIGNS};
use crate::diplo::{self, SpyMode, Treaty};
use crate::fleet::{self, FLEET_ENROUTE_AI_MAX, FLEET_ENROUTE_MAX};
use crate::misc;
use crate::state::*;
use crate::turn::year_n;
use crate::types::*;
use crate::umath::dist_fast;

const AI_CLASSIC: u8 = 0;
const PLANET_NONE: usize = usize::MAX;
/// 1oom `game_num_tbl_hull_w`.
const HULL_W: [u64; 4] = [1, 5, 25, 125];
use crate::diplo::EXPANSIONIST;

const SP_FUEL: u8 = 1;
const SP_COLONY_FIRST: u8 = 2;
const SP_COLONY_LAST: u8 = 8;
const SP_SCANNER: u8 = 9;
const SP_REPULSOR: u8 = 11;
const SP_WARPDIS: u8 = 12;
const SP_ENERGY_PULSAR: u8 = 13;
const SP_INERTIAL_STABILIZER: u8 = 14;
const SP_HIGH_ENERGY_FOCUS: u8 = 20;
const SP_IONIC_PULSAR: u8 = 21;
const SP_ORACLE: u8 = 29;
const WEAPON_LASER: u8 = 2;

fn classic(st: &State) -> bool {
    st.ai_id == AI_CLASSIC
}

fn treaty(st: &State, a: usize, b: usize) -> Treaty {
    st.empires[a].rel.get(b).map_or(Treaty::None, |r| r.treaty)
}

fn frange(st: &State, p: PlanetId, pi: usize) -> u8 {
    st.planets[p].within_frange.get(pi).copied().unwrap_or(0)
}

fn hrf(st: &State, pi: usize, si: usize) -> bool {
    st.empires[pi]
        .have_reserve_fuel
        .get(si)
        .copied()
        .unwrap_or(false)
}

fn has_tech(st: &State, pi: usize, field: usize, tech: u8) -> bool {
    tech != 0 && st.empires[pi].tech.has(field, tech)
}

/// 1oom `game_ai_best_speed`: (best propulsion tech of the engine series) + 3.
fn best_speed(st: &State, pi: usize) -> i32 {
    let mut best = 0;
    let mut t = 6;
    while t < 50 {
        if has_tech(st, pi, FIELD_PROPULSION, t) {
            best = t as i32;
        }
        t += 6;
    }
    best + 3
}

/// 1oom `game_ai_classic_tech_next`.
pub fn tech_next(st: &mut State, tbl: &[u8]) -> u8 {
    if (st.rng.one_to(6) as i32) < st.difficulty as i32 {
        tbl[tbl.len() - 1]
    } else {
        tbl[st.rng.below(tbl.len() as u16) as usize]
    }
}

/// 1oom `game_ai_classic_new_game_init`: sliders of a computer player's home.
pub fn new_game_init(st: &mut State, player: usize, home: PlanetId) {
    let (hx, hy) = (st.planets[home].x, st.planets[home].y);
    let n = st
        .planets
        .iter()
        .filter(|q| {
            q.kind as i32 > PlanetType::Minimal as i32 - 1 && dist_fast(q.x, q.y, hx, hy) <= 30
        })
        .count();
    let p3 = st.rng.one_to(6) as i32 + 14;
    let p2 = st.rng.one_to(10) as i32;
    let e = &mut st.empires[player];
    e.ai_p3_countdown = p3;
    e.ai_p2_countdown = p2;
    if n > 1 {
        st.planets[home].sliders = [10, 0, 90, 0, 0];
        st.empires[player].tech.alloc = [20, 20, 0, 40, 20, 0];
    } else {
        st.empires[player].ai_p3_countdown = st.rng.one_to(6) as i32 + 24;
        st.planets[home].sliders = [0, 0, 60, 10, 40];
        st.empires[player].tech.alloc = [0, 0, 0, 0, 100, 0];
    }
    let e = &mut st.empires[player];
    e.shipi_colony = 1;
    e.shipi_bomber = 4;
}

/// 1oom `game_ai_classic_new_game_tech`: first projects of the computer players.
pub fn new_game_tech(st: &mut State) {
    for pi in 0..st.empires.len() {
        if !st.empires[pi].ai {
            continue;
        }
        for field in 0..NUM_FIELDS {
            let tech = loop {
                let t = st.empires[pi].tech.list[field][0][st.rng.below(3) as usize];
                if t != 0 {
                    break t;
                }
            };
            let t = &mut st.empires[pi].tech;
            t.project[field] = tech;
            t.cost[field] = tech as u32 * tech as u32 * 50;
        }
    }
}

struct TurnP1 {
    have_colonizable: bool,
    need_conquer: bool,
    shipthreat: Vec<[u64; MAX_DESIGNS]>,
    xcenter: Vec<i32>,
    ycenter: Vec<i32>,
    force_own: Vec<u64>,
    num_fronts: usize,
    front_relation: [i32; PLAYER_NUM],
    front_planet: [usize; PLAYER_NUM],
    force_own_sum: u64,
    planet_own: Vec<(usize, i32)>,
    planet_en: Vec<(usize, i32)>,
}

fn fleet_count(st: &State) -> usize {
    st.fleets.len()
}

/// 1oom `game_ai_classic_turn_p1_send_scout`.
fn send_scout(st: &mut State, ait: &mut TurnP1, pi: usize) {
    let np = st.planets.len();
    let mut ignore = vec![false; np];
    for (i, ig) in ignore.iter_mut().enumerate() {
        let p = &st.planets[i];
        let mut ships: u32 = 0;
        for pi2 in 0..st.empires.len() {
            if pi != pi2 && treaty(st, pi, pi2) != Treaty::Alliance {
                let e2 = &st.empires[pi2];
                for j in 0..e2.designs.len() {
                    ships = ships.wrapping_add(e2.orbit[i][j]);
                }
            }
        }
        if st.empires[pi].explored[i]
            || frange(st, i, pi) == 0
            || (year_n(st) < 150 && st.orion == i)
            || ships > 0
        {
            *ig = true;
        }
        let _ = p;
    }
    for f in &st.fleets {
        if f.owner as usize == pi {
            ignore[f.dest] = true;
        }
    }
    ait.have_colonizable = false;
    ait.need_conquer = st.rng.below(8 - st.difficulty as u16) == 0;
    let colony_for = st.empires[pi].ab.colony_for as i32;
    let mut scout = Vec::new();
    for (i, p) in st.planets.iter().enumerate() {
        if p.owner.is_none() {
            if !ignore[i] && st.orion != i {
                scout.push(i);
            }
            if p.kind as i32 >= colony_for && (st.orion != i || !st.guardian) {
                ait.have_colonizable = true;
                ait.need_conquer = false;
            }
        }
    }
    if st.winner.is_some() || st.council.final_war {
        ait.need_conquer = false;
    }
    let numsd = st.empires[pi].designs.len();
    let mut ownorbit: Vec<u32> = (0..np)
        .map(|i| {
            st.empires[pi].orbit[i]
                .iter()
                .take(numsd)
                .fold(0u32, |a, &b| a.wrapping_add(b))
        })
        .collect();
    for pli in scout {
        let p = &st.planets[pli];
        let (px, py) = (p.x, p.y);
        let mut mindist = 10000;
        let mut mini = PLANET_NONE;
        for (j, &o) in ownorbit.iter().enumerate() {
            if o > 0 {
                let p2 = &st.planets[j];
                let d = dist_fast(px, py, p2.x, p2.y);
                if d < mindist {
                    mindist = d;
                    mini = j;
                }
            }
        }
        if mini == PLANET_NONE || fleet_count(st) >= FLEET_ENROUTE_AI_MAX {
            continue;
        }
        let mut shipi: i32 = -1;
        for j in 0..numsd {
            let t = st.empires[pi].orbit[mini][j];
            if t != 0 && (shipi == -1 || !hrf(st, pi, shipi as usize)) {
                shipi = j as i32;
            }
        }
        if shipi == -1 {
            continue;
        }
        let fr = frange(st, pli, pi);
        if !(fr == 1 || (fr == 2 && hrf(st, pi, shipi as usize))) {
            continue;
        }
        let si = shipi as usize;
        let num_send = (st.empires[pi].orbit[mini][si] & 0xffff) / 4 + 1;
        let speed = st.empires[pi].designs[si].engine as i32 + 1;
        let mut ships = vec![0u32; MAX_DESIGNS];
        ships[si] = num_send;
        fleet::push_fleet(st, pi as u8, mini, pli, speed, ships);
        st.empires[pi].orbit[mini][si] = st.empires[pi].orbit[mini][si].wrapping_sub(num_send);
        ownorbit[mini] = ownorbit[mini].wrapping_sub(num_send);
    }
}

/// 1oom `game_ai_classic_turn_p1_front_find_planet`.
fn front_find_planet(st: &State, pi: usize, x: i32, y: i32) -> usize {
    let mut mini = 0;
    let mut mindist = 10000;
    for (i, p) in st.planets.iter().enumerate() {
        if p.owner == Some(pi as u8) {
            let d = dist_fast(p.x, p.y, x, y);
            if d < mindist {
                mindist = d;
                mini = i;
            }
        }
    }
    let mut mindist = 10000;
    let e = &st.empires[pi];
    for (i, p) in st.planets.iter().enumerate() {
        if p.owner == Some(pi as u8) {
            let mut defense: u32 = if p.bases != 0 { 1 } else { 0 };
            let mut j = 0;
            while j < e.designs.len() && defense == 0 {
                defense = defense.wrapping_add(e.orbit[i][j]);
                j += 1;
            }
            if defense == 1 {
                let d = dist_fast(p.x, p.y, x, y);
                if d < mindist {
                    mindist = d;
                    mini = i;
                }
            }
        }
    }
    mini
}

/// 1oom `game_ai_classic_turn_p1_front`.
fn front(st: &State, ait: &mut TurnP1, pi: usize) {
    let bestspeed = best_speed(st, pi) * 10 + 10;
    let mut tx = [0i32; PLAYER_NUM + 1];
    let mut ty = [0i32; PLAYER_NUM + 1];
    ait.num_fronts = 0;
    let final_war = st.council.final_war;
    for pi2 in 0..st.empires.len() {
        if pi != pi2
            && st.empires[pi].rel[pi2].contact
            && (!final_war || st.council.refuse.get(pi2).copied().unwrap_or(false))
        {
            let nf = ait.num_fronts;
            ait.front_relation[nf] = 0;
            tx[nf] = (ait.xcenter[pi2] * 4 + ait.xcenter[pi] * 6) / 10;
            ty[nf] = (ait.ycenter[pi2] * 4 + ait.ycenter[pi] * 6) / 10;
            let mut vc = 0;
            for i in 0..nf {
                if dist_fast(tx[i], ty[i], tx[nf], ty[nf]) <= bestspeed {
                    vc = i + 1;
                }
            }
            let v8 = st.empires[pi].rel[pi2].relation.min(0);
            if vc == 0 {
                ait.front_relation[nf] = v8;
                ait.num_fronts += 1;
            }
        }
    }
    if ait.num_fronts == 0 {
        tx[0] = ait.xcenter[pi];
        ty[0] = ait.ycenter[pi];
        ait.front_relation[0] = 80;
        ait.num_fronts = 1;
    }
    for i in 0..ait.num_fronts {
        ait.front_planet[i] = front_find_planet(st, pi, tx[i], ty[i]);
    }
    for _ in 0..PLAYER_NUM - 1 {
        let mut i = 0;
        while i < ait.num_fronts {
            let mut m: Option<usize> = None;
            for j in i + 1..ait.num_fronts {
                if ait.front_planet[i] == ait.front_planet[j] {
                    m = Some(j);
                }
            }
            if let Some(m) = m {
                ait.front_relation[i] += ait.front_relation[m];
                for j in m..ait.num_fronts - 1 {
                    ait.front_relation[j] = ait.front_relation[j + 1];
                    ait.front_planet[j] = ait.front_planet[j + 1];
                }
                ait.num_fronts -= 1;
            } else {
                i += 1;
            }
        }
    }
    let e = &st.empires[pi];
    let mut sum: u64 = 0;
    for i in 0..st.planets.len() {
        let p = &st.planets[i];
        ait.force_own[i] = 0;
        if p.owner == Some(pi as u8) || p.owner.is_none() {
            for j in 0..e.designs.len() {
                let n = e.orbit[i][j] as u64;
                if n != 0 {
                    let mut v = ait.shipthreat[pi][j].wrapping_mul(n);
                    sum = sum.wrapping_add(v);
                    if classic(st) {
                        v &= 0xffff_ffff;
                    }
                    ait.force_own[i] = ait.force_own[i].wrapping_add(v);
                }
            }
        }
    }
    if classic(st) {
        sum &= 0xffff_ffff;
    }
    for i in 0..ait.num_fronts {
        if let Some(add) = ait.force_own[ait.front_planet[i]]
            .wrapping_mul(100)
            .checked_div(sum)
        {
            ait.front_relation[i] = ait.front_relation[i].wrapping_add(add as i32);
        }
    }
    ait.force_own_sum = sum / 25;
}

/// 1oom `game_ai_classic_turn_p1_have_colony_ship`.
fn have_colony_ship(st: &mut State, ait: &TurnP1, pi: usize) -> bool {
    let shipi = st.empires[pi].shipi_colony;
    if ait.num_fronts == 0 || shipi == -1 || st.empires[pi].total_prod == 0 {
        return false;
    }
    let si = shipi as usize;
    let mut shipn = st.empires[pi].shipcount.get(si).copied().unwrap_or(0);
    // 1oom always applies MOO1 v1.3a's fix of the "4 colony ships" curse.
    let planeti = ait.front_planet[st.rng.below(ait.num_fronts as u16) as usize];
    if shipn > 3 {
        return true;
    }
    let mut prod = ((st.empires[pi].total_prod as u32).wrapping_mul(2) / 5).clamp(1, 500);
    if (st.difficulty as i32) < Difficulty::Average as i32
        && st.rng.below(6) as i32 > st.difficulty as i32
    {
        prod = 0;
    }
    if !ait.have_colonizable || st.rng.one_to(500) as u32 > prod {
        return shipn > 0;
    }
    shipn += 1;
    if let Some(c) = st.empires[pi].shipcount.get_mut(si) {
        *c = shipn;
    }
    st.empires[pi].orbit[planeti][si] += 1;
    true
}

/// 1oom `game_turn_fleet_send`: all ships of an orbit leave (as MOO1 does, even those
/// without the fuel to go).
fn fleet_send(st: &mut State, ait: &mut TurnP1, pi: usize, from: usize, dest: usize) {
    if from == dest {
        return;
    }
    ait.force_own[from] = 0;
    if fleet_count(st) == FLEET_ENROUTE_MAX {
        return;
    }
    let fr = frange(st, dest, pi);
    let numsd = st.empires[pi].designs.len();
    let mut ships = vec![0u32; MAX_DESIGNS];
    let mut speed = 50;
    let mut types = 0;
    for i in 0..numsd {
        let mut n = st.empires[pi].orbit[from][i];
        if fr == 0 || (fr == 2 && !hrf(st, pi, i)) {
            n = 0;
        }
        if n > 0 {
            ships[i] = n;
            if !classic(st) {
                st.empires[pi].orbit[from][i] = 0;
            }
            types += 1;
            speed = speed.min(st.empires[pi].designs[i].engine as i32);
        }
    }
    let (pf, pt) = (&st.planets[from], &st.planets[dest]);
    if pt.owner == pf.owner && pt.stargate && pf.stargate {
        speed = 100;
    }
    if types > 0 {
        fleet::push_fleet(st, pi as u8, from, dest, speed + 1, ships);
    }
    if classic(st) {
        for n in st.empires[pi].orbit[from].iter_mut() {
            *n = 0;
        }
    }
}

/// 1oom `game_ai_classic_turn_p1_send_colony_ships`.
fn send_colony_ships(st: &mut State, ait: &mut TurnP1, pi: usize) {
    let range = st.empires[pi].ab.fuel_range * 15;
    let si = st.empires[pi].shipi_colony as usize;
    if si >= st.empires[pi].designs.len() {
        return;
    }
    let np = st.planets.len();
    let mut ignore: Vec<bool> = (0..np)
        .map(|i| st.planets[i].owner.is_some() || frange(st, i, pi) == 0)
        .collect();
    for f in &st.fleets {
        if f.owner as usize == pi && f.ships.get(si).is_some_and(|&n| n > 0) {
            ignore[f.dest] = true;
        }
    }
    let mut can_colonize = PlanetType::Minimal as i32;
    for &s in &st.empires[pi].designs[si].specials {
        if (SP_COLONY_FIRST..=SP_COLONY_LAST).contains(&s) {
            can_colonize = PlanetType::Minimal as i32 - (s as i32 - SP_COLONY_FIRST as i32);
        }
    }
    if st.empires[pi].race == Race::Silicoid {
        can_colonize = PlanetType::Radiated as i32;
    }
    let targets: Vec<usize> = (0..np)
        .filter(|&i| {
            let p = &st.planets[i];
            p.owner.is_none()
                && !ignore[i]
                && (st.orion != i || !st.guardian)
                && p.kind as i32 >= can_colonize
        })
        .collect();
    let mut orbit: Vec<u32> = (0..np).map(|i| st.empires[pi].orbit[i][si]).collect();
    for pli in targets {
        let (px, py) = (st.planets[pli].x, st.planets[pli].y);
        let fr = frange(st, pli, pi);
        let mut mini = PLANET_NONE;
        if fr == 1 || (fr == 2 && hrf(st, pi, si)) {
            let mut mindist = 10000;
            for (j, &o) in orbit.iter().enumerate() {
                if o > 0 {
                    let p2 = &st.planets[j];
                    let d = dist_fast(px, py, p2.x, p2.y);
                    if d < mindist {
                        mindist = d;
                        mini = j;
                    }
                }
            }
        }
        if mini == PLANET_NONE {
            continue;
        }
        let p2 = &st.planets[mini];
        if (!classic(st) || dist_fast(px, py, p2.x, p2.y) <= range)
            && fleet_count(st) < FLEET_ENROUTE_AI_MAX
        {
            fleet_send(st, ait, pi, mini, pli);
            if !classic(st) {
                orbit[mini] = st.empires[pi].orbit[mini][si];
            }
        }
    }
}

/// 1oom `game_ai_classic_turn_p1_planet_w`: own planets and targets by worth.
fn planet_w(st: &State, ait: &mut TurnP1, pi: usize) {
    ait.planet_en.clear();
    ait.planet_own.clear();
    let final_war = st.council.final_war;
    for (i, p) in st.planets.iter().enumerate() {
        let sp = p.special as i32;
        let v4 = (sp - 2) * 20 + sp * 10 + 20;
        let mut owner = p.owner.map(|o| o as usize);
        if owner.is_none() {
            for pi2 in 0..st.empires.len() {
                if owner.is_some() {
                    break;
                }
                if pi2 != pi {
                    let e2 = &st.empires[pi2];
                    if (0..e2.designs.len()).any(|j| e2.orbit[i][j] != 0) {
                        owner = Some(pi2);
                    }
                }
            }
        }
        match owner {
            Some(o) if o == pi => ait.planet_own.push((i, p.pop - p.bases * 5 + v4)),
            Some(o) => {
                let e = &st.empires[pi];
                if (treaty(st, pi, o) >= Treaty::War || ait.need_conquer || final_war)
                    && frange(st, i, pi) == 1
                    && p.kind as i32 >= e.ab.colony_for as i32
                    && (year_n(st) > 120 || st.orion != i)
                {
                    ait.planet_en.push((i, p.pop - p.bases * 10 + v4));
                }
            }
            None => {}
        }
    }
    // MOO1's bubble sort, which may stop early.
    let n = ait.planet_en.len();
    let mut work_left = true;
    let mut i = 0;
    while i < n && work_left {
        for j in 0..n.saturating_sub(1) {
            work_left = false;
            if ait.planet_en[j].1 < ait.planet_en[j + 1].1 {
                ait.planet_en.swap(j, j + 1);
                work_left = true;
            }
        }
        i += 1;
    }
}

fn treaty_of(st: &State, a: usize, b: usize) -> Treaty {
    treaty(st, a, b)
}

/// 1oom `game_ai_classic_turn_p1_send_attack`.
fn send_attack(st: &mut State, ait: &mut TurnP1, pi: usize) {
    let range = st.empires[pi].ab.fuel_range * 15;
    for i in 0..ait.num_fronts {
        let pfrom = ait.front_planet[i];
        let mut pto2 = PLANET_NONE;
        let mut j = 0;
        while j < ait.planet_en.len() && pto2 == PLANET_NONE {
            let (pto, w) = ait.planet_en[j];
            let (ptx, pty, pt_owner) =
                (st.planets[pto].x, st.planets[pto].y, st.planets[pto].owner);
            if w != -1000
                && dist_fast(ptx, pty, st.planets[pfrom].x, st.planets[pfrom].y) < range
                && (st.rng.one_to(100) < 40 || ait.planet_en.len() < 2)
            {
                let mut weight: u64 = 0;
                for pi2 in 0..st.empires.len() {
                    if pi2 != pi {
                        let e2 = &st.empires[pi2];
                        for l in 0..e2.designs.len() {
                            let n = e2.orbit[pto][l] as u64;
                            if n != 0 {
                                weight =
                                    weight.wrapping_add(n.wrapping_mul(ait.shipthreat[pi2][l]));
                            }
                        }
                    }
                }
                if let Some(o) = pt_owner {
                    let e2 = &st.empires[o as usize];
                    weight = weight.wrapping_add(
                        ((e2.tech.level[FIELD_WEAPON] as u64 * 5)
                            + (e2.ab.planet_shield as u64 + 10) * 10)
                            * st.planets[pto].bases as u64,
                    );
                }
                if classic(st) {
                    weight &= 0xffff_ffff;
                }
                if ait.force_own[pfrom] > weight {
                    pto2 = pto;
                    match pt_owner {
                        None => fleet_send(st, ait, pi, pfrom, pto2),
                        Some(o) => {
                            let o = o as usize;
                            let t = treaty_of(st, o, pi);
                            let expansionist = st.empires[pi].trait2 == EXPANSIONIST;
                            if t == Treaty::Alliance {
                                if st.rng.one_to(4) == 1 || (st.rng.one_to(2) == 1 && expansionist)
                                {
                                    diplo::act(
                                        st,
                                        -10000,
                                        o as u8,
                                        pi as u8,
                                        32,
                                        pto2,
                                        pto2 as i32,
                                    );
                                    diplo::break_treaty(st, pi as u8, o as u8);
                                    if st.empires[pi].rel[o].relation > 30 {
                                        st.empires[pi].rel[o].relation = 30;
                                        st.empires[o].rel[pi].relation = 30;
                                    }
                                }
                            } else if t == Treaty::NonAggression {
                                if st.rng.one_to(2) == 1 || expansionist {
                                    diplo::act(
                                        st,
                                        -10000,
                                        o as u8,
                                        pi as u8,
                                        32,
                                        pto2,
                                        pto2 as i32,
                                    );
                                    diplo::break_treaty(st, pi as u8, o as u8);
                                    if st.empires[pi].rel[o].relation > 30 {
                                        st.empires[pi].rel[o].relation = 20;
                                        st.empires[o].rel[pi].relation = 20;
                                    }
                                }
                            } else {
                                fleet_send(st, ait, pi, pfrom, pto2);
                            }
                        }
                    }
                    ait.planet_en[j].1 = -1000;
                }
            }
            j += 1;
        }
    }
    for pfrom in 0..st.planets.len() {
        let p = &st.planets[pfrom];
        if !(p.owner.is_some() && p.owner != Some(pi as u8) && ait.force_own[pfrom] != 0) {
            continue;
        }
        for j in 0..ait.planet_en.len() {
            if ait.planet_en[j].1 <= -1000 {
                continue;
            }
            let pto = ait.planet_en[j].0;
            let (ptx, pty) = (st.planets[pto].x, st.planets[pto].y);
            if dist_fast(ptx, pty, st.planets[pfrom].x, st.planets[pfrom].y) < range
                && st.rng.one_to(100) < 60
            {
                let mut weight: u32 = 0;
                for pi2 in 0..st.empires.len() {
                    if pi2 != pi {
                        let e2 = &st.empires[pi2];
                        for k in 0..e2.designs.len() {
                            weight = weight.wrapping_add(
                                e2.orbit[pto][k]
                                    .wrapping_mul(HULL_W[e2.designs[k].hull as usize] as u32),
                            );
                        }
                    }
                }
                if ait.force_own[pfrom].wrapping_mul(3) / 2 > weight as u64 {
                    fleet_send(st, ait, pi, pfrom, pto);
                    ait.planet_en[j].1 = -1000;
                    break;
                }
            }
        }
    }
}

/// 1oom `game_ai_classic_turn_p1_send_defend`.
fn send_defend(st: &mut State, ait: &mut TurnP1, pi: usize) {
    let bestspeed = best_speed(st, pi) * 30 + 20;
    let np = st.planets.len();
    let mut threat = vec![0u64; np];
    for f in &st.fleets {
        if f.owner as usize != pi && st.planets[f.dest].owner == Some(pi as u8) {
            let e2 = &st.empires[f.owner as usize];
            for j in 0..e2.designs.len() {
                let n = f.ships.get(j).copied().unwrap_or(0) as u64;
                if n != 0 {
                    threat[f.dest] =
                        threat[f.dest].wrapping_add(n * HULL_W[e2.designs[j].hull as usize]);
                }
            }
        }
    }
    for t in &st.transports {
        if t.owner as usize != pi && st.planets[t.dest].owner == Some(pi as u8) {
            threat[t.dest] =
                threat[t.dest].wrapping_add(t.pop as u64 * HULL_W[HULL_MEDIUM as usize]);
        }
    }
    for i in 0..np {
        if st.planets[i].bases < 5 && ait.force_own[i] == 0 {
            threat[i] = 50;
        }
    }
    if classic(st) {
        for t in threat.iter_mut() {
            *t &= 0xffff_ffff;
        }
    }
    let defend: Vec<usize> = (0..np)
        .filter(|&i| {
            st.planets[i].owner == Some(pi as u8)
                && ait.force_own[i].wrapping_mul(4) / 3 < threat[i]
        })
        .collect();
    for i in 0..np {
        if !(ait.force_own[i] > 0 && threat[i] == 0) {
            continue;
        }
        let (fx, fy) = (st.planets[i].x, st.planets[i].y);
        let pto = defend
            .iter()
            .copied()
            .find(|&pli| dist_fast(fx, fy, st.planets[pli].x, st.planets[pli].y) <= bestspeed);
        if let Some(pto) = pto {
            if threat[pto].wrapping_mul(5) / 3 < ait.force_own[i] {
                fleet_send(st, ait, pi, i, pto);
                threat[pto] = 100_000_000;
            }
        }
    }
}

/// 1oom `game_ai_classic_turn_p1_send_idle`.
fn send_idle(st: &mut State, ait: &mut TurnP1, pi: usize) {
    let bestspeed = best_speed(st, pi) * 20 + 20;
    for i in 0..ait.num_fronts {
        ait.force_own[ait.front_planet[i]] = 0;
    }
    // MOO1 compares an unsigned difference with 0 here: never true.
    if st.council.held {
        ait.force_own_sum /= 2;
    }
    if st.council.final_war {
        ait.force_own_sum /= 2;
    }
    for f in ait.force_own.iter_mut() {
        if *f < ait.force_own_sum {
            *f = 0;
        }
    }
    for i in 0..st.planets.len() {
        if ait.force_own[i] == 0 {
            continue;
        }
        let (fx, fy) = (st.planets[i].x, st.planets[i].y);
        let mut pto = PLANET_NONE;
        let mut minv = 10000;
        for j in 0..ait.num_fronts {
            let pli = ait.front_planet[j];
            let pt = &st.planets[pli];
            let v = (dist_fast(pt.x, pt.y, fx, fy) * 10) / bestspeed + ait.front_relation[j];
            if v < minv {
                minv = v;
                pto = pli;
            }
        }
        if pto != PLANET_NONE {
            fleet_send(st, ait, pi, i, pto);
        }
    }
}

/// 1oom `game_ai_classic_turn_p1_trans_en`: invade planets held by own ships.
fn trans_en(st: &mut State, pi: usize) {
    let bestspeed = best_speed(st, pi) * 30 + 30;
    let np = st.planets.len();
    let mut to = vec![false; np];
    let mut from = vec![false; np];
    for i in 0..np {
        let p = &st.planets[i];
        let e = &st.empires[pi];
        let mut have_orbit = false;
        if let Some(o) = p.owner.filter(|&o| o as usize != pi) {
            let o = o as usize;
            let ceasefire = st.empires[o].rel.get(pi).map_or(0, |r| r.ceasefire);
            if (st.empires[o].ai || ceasefire <= 0)
                && treaty(st, pi, o) != Treaty::Alliance
                && e.ab.colony_for as i32 <= p.kind as i32
            {
                have_orbit = (0..e.designs.len()).any(|j| e.orbit[i][j] != 0);
            }
        }
        to[i] = have_orbit;
        from[i] = p.owner == Some(pi as u8)
            && p.pop >= p.max_pop / 2
            && p.pop > 20
            && (classic(st) || p.unrest != Unrest::Rebellion);
    }
    for t in &st.transports {
        if t.owner as usize == pi {
            to[t.dest] = false;
        }
    }
    for i in 0..np {
        if !to[i] {
            continue;
        }
        let (tx, ty) = (st.planets[i].x, st.planets[i].y);
        let mut mindist = 10000;
        let mut pfrom = PLANET_NONE;
        for j in 0..np {
            if from[j] {
                let d = dist_fast(tx, ty, st.planets[j].x, st.planets[j].y);
                if d < mindist {
                    mindist = d;
                    pfrom = j;
                }
            }
        }
        if pfrom != PLANET_NONE && mindist < bestspeed {
            let pf = &mut st.planets[pfrom];
            pf.trans_num = pf.pop / 2;
            pf.trans_dest = i;
            from[pfrom] = false;
        }
    }
}

/// 1oom `game_ai_classic_turn_p1_trans_own`: move colonists to underpopulated planets.
fn trans_own(st: &mut State, pi: usize) {
    let bestspeed = best_speed(st, pi) * 20 + 20;
    let np = st.planets.len();
    let mut to = vec![false; np];
    let mut from = vec![false; np];
    for i in 0..np {
        let p = &st.planets[i];
        if p.owner == Some(pi as u8) {
            if p.pop < p.max_pop / 3 || p.unrest == Unrest::Rebellion {
                to[i] = true;
            }
            if p.pop > (p.max_pop * 3) / 4 && (classic(st) || p.unrest != Unrest::Rebellion) {
                from[i] = true;
            }
        }
    }
    for t in &st.transports {
        if t.owner as usize == pi {
            to[t.dest] = false;
        }
    }
    for i in 0..np {
        if !to[i] {
            continue;
        }
        let (tx, ty) = (st.planets[i].x, st.planets[i].y);
        for j in 0..np {
            if from[j] && dist_fast(tx, ty, st.planets[j].x, st.planets[j].y) < bestspeed {
                let pf = &mut st.planets[j];
                pf.trans_num = pf.pop / 3;
                pf.trans_dest = i;
                from[j] = false;
                break;
            }
        }
    }
}

/// 1oom `game_ai_classic_turn_p1_build_defending_ships`.
fn build_defending_ships(st: &mut State, pi: usize) {
    let np = st.planets.len();
    let mut incoming = vec![false; np];
    for f in &st.fleets {
        let o = f.owner as usize;
        if o != pi
            && (classic(st) || treaty(st, pi, o) != Treaty::Alliance)
            && st.planets[f.dest].owner == Some(pi as u8)
        {
            incoming[f.dest] = true;
        }
    }
    for (i, inc) in incoming.into_iter().enumerate() {
        if !inc {
            continue;
        }
        let sl = &mut st.planets[i].sliders;
        sl[SLIDER_SHIP] += sl[SLIDER_DEF] + sl[SLIDER_IND] + sl[SLIDER_TECH];
        sl[SLIDER_DEF] = 0;
        sl[SLIDER_IND] = 0;
        sl[SLIDER_TECH] = 0;
        add_reserve(st, i, pi);
    }
}

fn add_reserve(st: &mut State, planet: PlanetId, pi: usize) {
    let v = st.empires[pi].reserve / 5;
    st.empires[pi].reserve -= v;
    st.planets[planet].reserve += v;
}

/// 1oom `game_ai_classic_turn_p1_fund_developing`.
fn fund_developing(st: &mut State, pi: usize) {
    for i in 0..st.planets.len() {
        if st.planets[i].owner == Some(pi as u8) && st.planets[i].pop < 20 {
            add_reserve(st, i, pi);
        }
    }
}

/// 1oom `game_ai_classic_turn_p1`: fleets, colony ships, transports, taxes.
pub fn turn_p1(st: &mut State) {
    let n = st.empires.len();
    let mut shipthreat = vec![[0u64; MAX_DESIGNS]; n];
    for (pi, th) in shipthreat.iter_mut().enumerate() {
        let e = &st.empires[pi];
        for (i, sd) in e.designs.iter().enumerate() {
            let mut v: u32 = 0;
            for j in 0..4 {
                let wt = sd.weapons[j] as usize;
                if !WEAPONS[wt].bomb {
                    v = v.wrapping_add(WEAPONS[wt].tech as u32 * sd.weapon_counts[j] as u32);
                }
            }
            if v != 0 {
                v = v.wrapping_add(((sd.shield as u32 + 10) * sd.hp) / 50);
            }
            for &s in &sd.specials {
                if s >= SP_SCANNER {
                    v = v.wrapping_add(SPECIALS[s as usize].tech as u32 * 2);
                }
            }
            if e.ai {
                if st.council.held {
                    v = v.wrapping_mul(2);
                }
                if st.council.final_war {
                    v = v.wrapping_mul(2);
                }
            }
            if classic(st) {
                v &= 0xffff;
            }
            th[i] = v as u64;
        }
    }
    misc::update_maint_costs(st);
    let mut xcenter = vec![0i32; n];
    let mut ycenter = vec![0i32; n];
    for pi in 0..n {
        if !st.empires[pi].alive {
            continue;
        }
        let (mut xs, mut ys, mut num) = (0i32, 0i32, 0i32);
        for p in st.planets.iter().filter(|p| p.owner == Some(pi as u8)) {
            xs += p.x;
            ys += p.y;
            num += 1;
        }
        if num != 0 {
            xcenter[pi] = (xs / num) & 0xffff;
            ycenter[pi] = (ys / num) & 0xffff;
        }
    }
    let np = st.planets.len();
    let mut ait = TurnP1 {
        have_colonizable: false,
        need_conquer: false,
        shipthreat,
        xcenter,
        ycenter,
        force_own: vec![0; np],
        num_fronts: 0,
        front_relation: [0; PLAYER_NUM],
        front_planet: [0; PLAYER_NUM],
        force_own_sum: 0,
        planet_own: Vec::new(),
        planet_en: Vec::new(),
    };
    for pi in 0..n {
        if !st.empires[pi].ai {
            continue;
        }
        let mut send_colony = true;
        if st.empires[pi].trait2 != EXPANSIONIST {
            let mut num = 0;
            let mut developing = 0;
            for p in st.planets.iter().filter(|p| p.owner == Some(pi as u8)) {
                num += 1;
                if p.bases < p.max_pop / 20 && p.pop < (p.max_pop * 2) / 3 {
                    developing += 1;
                }
            }
            if num / 2 < developing {
                for pi2 in 0..n {
                    if !st.empires[pi2].ai && st.empires[pi2].rel[pi].contact {
                        send_colony = false;
                        break;
                    }
                }
            }
        }
        send_scout(st, &mut ait, pi);
        front(st, &mut ait, pi);
        if have_colony_ship(st, &ait, pi) && send_colony {
            send_colony_ships(st, &mut ait, pi);
        }
        front(st, &mut ait, pi);
        planet_w(st, &mut ait, pi);
        if !ait.planet_en.is_empty() {
            send_attack(st, &mut ait, pi);
        }
        send_defend(st, &mut ait, pi);
        let num_enroute = st.fleets.iter().filter(|f| f.owner as usize == pi).count();
        if num_enroute < 8 {
            send_idle(st, &mut ait, pi);
        }
        trans_en(st, pi);
        trans_own(st, pi);
        build_defending_ships(st, pi);
        fund_developing(st, pi);
        st.empires[pi].tax = if year_n(st) >= 20 {
            st.rng.one_to(10) as i32 + st.difficulty as i32 + 2
        } else {
            0
        };
    }
}

// ------------------------------------------------------------------ ship design

/// 1oom `struct game_design_s`: the design being built with the designer's tech levels.
struct Gd {
    sd: ShipDesign,
    percent: [u32; NUM_FIELDS],
    player: usize,
    names: [String; 4],
}

impl Gd {
    fn space(&self) -> i32 {
        self.sd.free_space(&self.percent)
    }
    fn cost(&self) -> u32 {
        self.sd.cost(&self.percent)
    }
    /// 1oom `game_ai_classic_design_update_engines_space`: the space is stored in a
    /// 16-bit unsigned field, so an overfull design reads as very roomy.
    fn update_engines_space(&mut self) -> i32 {
        self.sd.update_engines();
        self.sd.space = self.space();
        self.sd.space as u16 as i32
    }
}

/// 1oom `game_get_random_shipnames`: a free name per hull (four random numbers).
fn random_shipnames(st: &mut State, pi: usize) -> [String; 4] {
    let e = &st.empires[pi];
    let names: Vec<String> = e.ship_names.clone();
    let num = names.len().max(1);
    let mut unused: Vec<bool> = (0..num)
        .map(|n| {
            !e.designs
                .iter()
                .any(|d| names.get(n).is_some_and(|nm| &d.name == nm))
        })
        .collect();
    let mut out: [String; 4] = Default::default();
    for i in (0..4).rev() {
        let mut namei = i * 3 + st.rng.below(3) as usize;
        let mut found = unused.get(namei).copied().unwrap_or(false);
        if !found {
            while !found && namei < num {
                if unused[namei] {
                    found = true;
                } else {
                    namei += 1;
                }
            }
            if !found {
                let mut k = namei as i64 - 1;
                while !found && k >= 0 {
                    if unused[k as usize] {
                        found = true;
                    } else {
                        k -= 1;
                    }
                }
                namei = if found { k as usize } else { 0 };
            }
        }
        if namei < num {
            unused[namei] = false;
        }
        out[i] = names.get(namei).cloned().unwrap_or_default();
    }
    out
}

/// 1oom `game_design_prepare_ai`.
fn prepare_ai(st: &mut State, pi: usize, hull: u8, look: u8) -> Gd {
    let names = random_shipnames(st, pi);
    let mut sd = ShipDesign {
        name: names[hull as usize].clone(),
        hull,
        look,
        weapons: [0; 4],
        weapon_counts: [0; 4],
        engine: 0,
        engines: 0,
        specials: [0; 3],
        shield: 0,
        jammer: 0,
        comp: 0,
        armor: 0,
        man: 0,
        hp: 0,
        cost_bc: 0,
        space: 0,
    };
    let percent = st.empires[pi].tech.level;
    sd.update_engines();
    let mut gd = Gd {
        sd,
        percent,
        player: pi,
        names,
    };
    gd.sd.space = gd.space();
    gd.sd.cost_bc = gd.cost();
    gd
}

/// Generic 1oom `game_design_build_tbl_fit_*` over a part table.
#[allow(clippy::too_many_arguments)]
fn fit_table(
    st: &State,
    gd: &mut Gd,
    buf: &mut [i8],
    len: usize,
    start: usize,
    get: fn(&ShipDesign) -> u8,
    set: fn(&mut ShipDesign, u8),
    tech: impl Fn(usize) -> (usize, u8),
) -> usize {
    let act = get(&gd.sd);
    let mut last = 0;
    buf[0] = 1;
    for i in start..len {
        let (field, t) = tech(i);
        if has_tech(st, gd.player, field, t) {
            set(&mut gd.sd, i as u8);
            gd.sd.update_engines();
            buf[i] = if gd.space() >= 0 { 1 } else { 0 };
            last = i;
        } else {
            buf[i] = -1;
        }
    }
    set(&mut gd.sd, act);
    gd.sd.update_engines();
    last
}

fn fit_comp(st: &State, gd: &mut Gd, buf: &mut [i8]) -> usize {
    fit_table(
        st,
        gd,
        buf,
        COMPS.len(),
        1,
        |s| s.comp,
        |s, v| s.comp = v,
        |i| (FIELD_COMPUTER, COMPS[i].tech),
    )
}
fn fit_shield(st: &State, gd: &mut Gd, buf: &mut [i8]) -> usize {
    fit_table(
        st,
        gd,
        buf,
        SHIELDS.len(),
        1,
        |s| s.shield,
        |s, v| s.shield = v,
        |i| (FIELD_FORCE_FIELD, SHIELDS[i].tech),
    )
}
fn fit_jammer(st: &State, gd: &mut Gd, buf: &mut [i8]) -> usize {
    fit_table(
        st,
        gd,
        buf,
        JAMMERS.len(),
        1,
        |s| s.jammer,
        |s, v| s.jammer = v,
        |i| (FIELD_COMPUTER, JAMMERS[i].tech),
    )
}
fn fit_armor(st: &State, gd: &mut Gd, buf: &mut [i8]) -> usize {
    fit_table(
        st,
        gd,
        buf,
        ARMOR.len(),
        1,
        |s| s.armor,
        |s, v| s.armor = v,
        |i| (FIELD_CONSTRUCTION, ARMOR[i].tech),
    )
}
fn fit_engine(st: &State, gd: &mut Gd, buf: &mut [i8]) -> usize {
    fit_table(
        st,
        gd,
        buf,
        ENGINES.len(),
        0,
        |s| s.engine,
        |s, v| s.engine = v,
        |i| (FIELD_PROPULSION, ENGINES[i].tech),
    )
}

fn fit_man(gd: &mut Gd, buf: &mut [i8]) -> usize {
    let act = gd.sd.man;
    for i in 0..=gd.sd.engine {
        gd.sd.man = i;
        gd.sd.update_engines();
        buf[i as usize] = if gd.space() >= 0 { 1 } else { 0 };
    }
    gd.sd.man = act;
    gd.sd.update_engines();
    gd.sd.engine as usize
}

fn fit_weapon(st: &State, gd: &mut Gd, buf: &mut [i8], slot: usize) -> usize {
    let (actt, actn) = (gd.sd.weapons[slot], gd.sd.weapon_counts[slot]);
    let mut last = 0;
    buf[0] = 1;
    for i in 1..WEAPONS.len() {
        let field = if WEAPONS[i].bio {
            FIELD_PLANETOLOGY
        } else {
            FIELD_WEAPON
        };
        if has_tech(st, gd.player, field, WEAPONS[i].tech) {
            gd.sd.weapons[slot] = i as u8;
            let mut n = 1u8;
            while n < 100 {
                gd.sd.weapon_counts[slot] = n;
                gd.sd.update_engines();
                if gd.space() < 0 {
                    break;
                }
                n += 1;
            }
            buf[i] = (n - 1) as i8;
            last = i;
        } else {
            buf[i] = -1;
        }
    }
    gd.sd.weapons[slot] = actt;
    gd.sd.weapon_counts[slot] = actn;
    gd.sd.update_engines();
    last
}

fn fit_special(st: &State, gd: &mut Gd, buf: &mut [i8], slot: usize) -> usize {
    let act = gd.sd.specials[slot];
    let mut last = 0;
    let other: Vec<u8> = (0..3)
        .filter(|&j| j != slot)
        .map(|j| SPECIALS[gd.sd.specials[j] as usize].kind)
        .collect();
    buf[0] = 1;
    for i in 1..SPECIALS.len() {
        if has_tech(st, gd.player, SPECIALS[i].field as usize, SPECIALS[i].tech) {
            let kind = SPECIALS[i].kind;
            if other.contains(&kind) {
                buf[i] = 0;
            } else {
                gd.sd.specials[slot] = i as u8;
                gd.sd.update_engines();
                buf[i] = if gd.space() >= 0 { 1 } else { 0 };
            }
            last = i;
        } else {
            buf[i] = -1;
        }
    }
    gd.sd.specials[slot] = act;
    gd.sd.update_engines();
    last
}

fn count_have(tbl: &[i8], last: usize) -> usize {
    tbl[..last].iter().filter(|&&v| v > 0).count()
}

fn find_have(tbl: &[i8], num: usize) -> usize {
    let mut num = num;
    let mut i = 0;
    while num > 0 {
        if tbl[i] > 0 {
            num -= 1;
        }
        i += 1;
    }
    if i > 0 {
        i - 1
    } else {
        0
    }
}

/// 1oom `game_ai_classic_design_ship_get_item`.
fn get_item(st: &mut State, mut num: usize, mut chance: i32) -> usize {
    while st.rng.one_to(100) as i32 > chance && num > 1 {
        chance *= 2;
        num -= 1;
    }
    num
}

const CHANCE_SPECIAL: [[i32; 4]; 31] = [
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 30, 80],
    [0, 10, 50, 80],
    [0, 0, 25, 80],
    [0, 0, 20, 70],
    [0, 5, 20, 60],
    [20, 30, 50, 80],
    [0, 10, 40, 70],
    [0, 0, 25, 50],
    [0, 0, 20, 40],
    [20, 30, 40, 70],
    [0, 0, 20, 30],
    [10, 20, 25, 35],
    [0, 0, 20, 30],
    [0, 0, 20, 40],
    [10, 20, 30, 40],
    [0, 10, 30, 50],
    [0, 0, 30, 60],
    [0, 0, 0, 50],
    [0, 10, 20, 30],
    [10, 20, 40, 60],
    [10, 20, 40, 60],
    [10, 20, 40, 60],
];

struct TurnP2 {
    shiptype: u8,
    have_pulsar: bool,
    have_repulwarp: bool,
}

/// 1oom `game_ai_classic_design_ship_base`.
fn design_base(st: &mut State, ait: &TurnP2, gd: &mut Gd) {
    let mut have = [0i8; 64];
    let hull = gd.sd.hull as usize;
    {
        let v = fit_engine(st, gd, &mut have);
        let mut v = count_have(&have, v);
        if classic(st) || v < 2 {
            v = get_item(st, v, 60);
        } else if v > 2 {
            let best = find_have(&have, v);
            let next = find_have(&have, v - 1);
            if next > 1 && best - next < 3 {
                v = get_item(st, v, 45 + (best as i32 - next as i32) * 15);
            }
        }
        gd.sd.engine = find_have(&have, v) as u8;
    }
    let space = gd.update_engines_space() / 3;
    if ait.shiptype == 0 {
        let v = fit_special(st, gd, &mut have, 0);
        have[SP_FUEL as usize] = 0;
        for h in have.iter_mut().take(v + 1).skip(SP_SCANNER as usize) {
            *h = 0;
        }
        let v = count_have(&have, v);
        let v = if v >= 2 {
            find_have(&have, v).max(SP_COLONY_FIRST as usize)
        } else {
            SP_COLONY_FIRST as usize
        };
        gd.sd.specials[0] = v as u8;
        if gd.percent[FIELD_CONSTRUCTION] > 5 {
            gd.sd.specials[1] = SP_FUEL;
        }
    }
    {
        let chance = [10, 20, 35, 50];
        let v = fit_comp(st, gd, &mut have);
        let v = count_have(&have, v);
        let v = get_item(st, v, chance[hull]);
        gd.sd.comp = find_have(&have, v) as u8;
    }
    if gd.update_engines_space() < space {
        return;
    }
    {
        let chance = [5, 15, 40, 70];
        let v = fit_shield(st, gd, &mut have);
        let v = count_have(&have, v);
        let v = get_item(st, v, chance[hull]);
        gd.sd.shield = find_have(&have, v) as u8;
    }
    if gd.update_engines_space() < space {
        return;
    }
    {
        let chance = [1, 2, 4, 8];
        let mut v = fit_armor(st, gd, &mut have);
        if !(st.rng.one_to(100) as i32 <= chance[hull] && (v & 1) == 1) {
            v &= !1;
        }
        gd.sd.armor = v as u8;
    }
    if gd.update_engines_space() < space {
        return;
    }
    if ait.shiptype == 0 {
        return;
    }
    for si in 0..=1 {
        let mut stype = 0u8;
        let v = fit_special(st, gd, &mut have, si);
        for h in have
            .iter_mut()
            .take(SP_SCANNER as usize)
            .skip(SP_FUEL as usize)
        {
            *h = 0;
        }
        for i in 0..=v {
            if st.rng.one_to(100) as i32 <= CHANCE_SPECIAL[i][hull] && have[i] > 0 {
                stype = i as u8;
            }
        }
        gd.sd.specials[si] = stype;
        if gd.update_engines_space() < space {
            return;
        }
    }
    {
        let chance = [30, 20, 20, 15];
        let v = fit_man(gd, &mut have);
        let v = count_have(&have, v);
        let v = get_item(st, v, chance[hull]);
        gd.sd.man = find_have(&have, v) as u8;
    }
    if gd.update_engines_space() < space {
        return;
    }
    {
        let chance = [2, 8, 20, 30];
        let v = fit_jammer(st, gd, &mut have);
        let v = count_have(&have, v);
        let v = get_item(st, v, chance[hull]);
        gd.sd.jammer = find_have(&have, v) as u8;
    }
    {
        let mut stype = 0u8;
        let v = fit_special(st, gd, &mut have, 2);
        for h in have
            .iter_mut()
            .take(SP_SCANNER as usize)
            .skip(SP_FUEL as usize)
        {
            *h = 0;
        }
        for i in 0..=v {
            if st.rng.one_to(100) as i32 <= CHANCE_SPECIAL[i][hull] && have[i] > 0 {
                stype = i as u8;
            }
        }
        gd.sd.specials[2] = stype;
    }
}

/// 1oom `game_ai_classic_design_ship_sub2`.
fn design_sub2(st: &State, gd: &mut Gd) {
    let mut have = [0i8; 64];
    if gd.sd.specials[2] != 0 || gd.sd.hull < HULL_LARGE {
        return;
    }
    fit_special(st, gd, &mut have, 2);
    if have[SP_SCANNER as usize] > 0 {
        gd.sd.specials[2] = SP_SCANNER;
    }
    if gd.percent[FIELD_PROPULSION] < 30 || gd.sd.specials[1] != 0 {
        return;
    }
    fit_special(st, gd, &mut have, 2);
    if have[SP_INERTIAL_STABILIZER as usize] > 0 {
        gd.sd.specials[1] = SP_INERTIAL_STABILIZER;
    }
}

/// 1oom `game_ai_classic_design_ship_weapon`.
fn design_weapon(
    st: &mut State,
    ait: &TurnP2,
    gd: &mut Gd,
    slot: usize,
    numshots_ignore: i8,
    c1: i32,
    c2: i32,
) {
    let mut have = [0i8; 64];
    let mut v = fit_weapon(st, gd, &mut have, slot);
    if slot == 0 && WEAPONS[v].bomb && ait.shiptype != 2 {
        v = v.wrapping_sub(1);
    }
    let v = v.max(1);
    for i in 0..v {
        if have[i] > 0
            && (WEAPONS[i].shots == numshots_ignore
                || (ait.have_repulwarp && WEAPONS[i].range == 1))
        {
            have[i] = 0;
        }
    }
    for i in 0..slot {
        have[gd.sd.weapons[i] as usize] = 0;
    }
    let v = count_have(&have, v);
    if v > 1 {
        let v = find_have(&have, v);
        gd.sd.weapons[slot] = v as u8;
        let w = &WEAPONS[v];
        let r = if w.fire > 0 || w.streams > 0 { 41 } else { 11 };
        let roll = st.rng.one_to((c2 - c1 + r) as u16) as i32 + c1;
        gd.sd.weapon_counts[slot] = ((have[v] as i32 * roll) / 100) as u8;
    }
}

/// 1oom `game_ai_classic_design_ship_weapons`.
fn design_weapons(st: &mut State, ait: &TurnP2, gd: &mut Gd) {
    let numshots_ignore = if st.rng.below(2) != 0 { 2 } else { 5 };
    let hull = gd.sd.hull as usize;
    let mut weapnum = 1;
    if st.rng.one_to(100) as i32 <= [0, 50, 75, 100][hull] {
        weapnum = 2;
        if st.rng.one_to(100) as i32 <= [0, 0, 50, 100][hull] {
            weapnum = 3;
            if st.rng.one_to(100) as i32 <= [0, 0, 25, 50][hull] {
                weapnum = 4;
            }
        }
    }
    const C1: [[i32; 4]; 4] = [
        [100, 100, 100, 100],
        [60, 100, 100, 100],
        [40, 40, 100, 100],
        [40, 40, 60, 100],
    ];
    const C2: [[i32; 4]; 4] = [
        [100, 100, 100, 100],
        [80, 100, 100, 100],
        [70, 70, 100, 100],
        [60, 60, 80, 100],
    ];
    for i in 0..weapnum {
        design_weapon(
            st,
            ait,
            gd,
            i,
            numshots_ignore,
            C1[weapnum - 1][i],
            C2[weapnum - 1][i],
        );
    }
}

/// 1oom `game_ai_classic_design_ship_get_look`.
fn design_look(st: &State, pi: usize, hull: u8) -> u8 {
    let e = &st.empires[pi];
    let lookbase = 6 * hull as u32 + e.banner as u32 * 24;
    let lookmax = lookbase + 5;
    let mut used = [false; 6];
    for d in &e.designs {
        let l = d.look as u32;
        if l >= lookbase && l <= lookmax {
            used[(l - lookbase) as usize] = true;
        }
    }
    let mut look = 0;
    while look < 6 && used[look] {
        look += 1;
    }
    if look > 5 {
        look = 0;
    }
    (look as u32 + lookbase) as u8
}

/// 1oom `game_ai_classic_design_ship`.
fn design_ship(st: &mut State, ait: &mut TurnP2, pi: usize) {
    let temp_repulwarp = ait.have_repulwarp;
    let mut loops = 0;
    let gd = 'again: loop {
        loops += 1;
        if loops >= 100_000 {
            panic!("design_ship looped");
        }
        let hull = if ait.shiptype != 0 {
            const HULLDIFF: [i32; 10] = [0, 0, 3, 0, 0, -3, -3, 3, 3, 0];
            const HULLS: [u8; 12] = [0, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3];
            let mut v = st.rng.below(12) as i32 + HULLDIFF[st.empires[pi].race.index()];
            v = v.max(0);
            if ait.have_pulsar {
                v += 1;
            }
            HULLS[v.min(11) as usize]
        } else {
            HULL_LARGE
        };
        let look = design_look(st, pi, hull);
        let mut gd: Option<Gd> = None;
        let mut loops2 = 0;
        while gd.as_ref().is_none_or(|g| g.sd.weapon_counts[0] == 0) && loops2 < 100_000 {
            let mut g = prepare_ai(st, pi, hull, look);
            design_base(st, ait, &mut g);
            design_sub2(st, &mut g);
            design_weapons(st, ait, &mut g);
            g.sd.set_hp();
            g.sd.compact();
            if ait.shiptype == 0 {
                g.sd.name = "Colony Ship".to_string();
                if g.sd.weapon_counts[0] == 0 {
                    g.sd.weapons[0] = WEAPON_LASER;
                    g.sd.weapon_counts[0] = 1;
                }
            }
            g.sd.cost_bc = g.cost();
            gd = Some(g);
            loops2 += 1;
        }
        let gd = gd.expect("designed");
        if gd.sd.weapon_counts[0] == 0 {
            if ait.have_repulwarp {
                if hull != HULL_SMALL {
                    ait.have_repulwarp = false;
                }
                continue 'again;
            }
            panic!("design_ship loops2");
        }
        let wt = &WEAPONS[gd.sd.weapons[0] as usize];
        let is_missile = wt.damage_min == wt.damage_max;
        let mut again = false;
        if (is_missile || wt.bomb) && gd.sd.specials.contains(&SP_HIGH_ENERGY_FOCUS) {
            again = true;
        }
        if is_missile && gd.sd.specials.contains(&SP_ORACLE) {
            again = true;
        }
        if again {
            continue 'again;
        }
        break gd;
    };
    let _ = &gd.names;
    let e = &mut st.empires[pi];
    if e.designs.len() < MAX_DESIGNS {
        e.designs.push(gd.sd);
        let y = year_n_of(st);
        let e = &mut st.empires[pi];
        e.design_year.resize(MAX_DESIGNS, 0);
        let idx = e.designs.len() - 1;
        e.design_year[idx] = y;
    }
    ait.have_repulwarp = temp_repulwarp;
}

fn year_n_of(st: &State) -> i32 {
    year_n(st)
}

/// 1oom `game_design_scrap` (`for_new` false for the AI).
pub fn design_scrap(st: &mut State, pi: usize, shipi: usize, for_new: bool) {
    misc::update_maint_costs(st);
    let num = st.empires[pi].designs.len();
    if num <= 1 || shipi >= num {
        return;
    }
    let e = &mut st.empires[pi];
    e.design_year.resize(MAX_DESIGNS, 0);
    for i in shipi..MAX_DESIGNS - 1 {
        e.design_year[i] = e.design_year[i + 1];
    }
    for o in e.orbit.iter_mut() {
        for j in shipi..MAX_DESIGNS - 1 {
            o[j] = o[j + 1];
        }
        o[MAX_DESIGNS - 1] = 0;
    }
    for f in st.fleets.iter_mut().filter(|f| f.owner as usize == pi) {
        f.ships.resize(MAX_DESIGNS, 0);
        for j in shipi..MAX_DESIGNS - 1 {
            f.ships[j] = f.ships[j + 1];
        }
        f.ships[MAX_DESIGNS - 1] = 0;
    }
    let e = &mut st.empires[pi];
    let refund = (e.shipcount.get(shipi).copied().unwrap_or(0) as i64
        * e.designs[shipi].cost_bc as i64
        / 4) as i32;
    e.reserve += refund;
    let human = !e.ai;
    for p in st.planets.iter_mut().filter(|p| p.owner == Some(pi as u8)) {
        let bs = p.build;
        if bs != BUILD_STARGATE {
            if bs == shipi {
                p.build = if !human || !for_new { 0 } else { num - 1 };
            } else if bs > shipi {
                p.build = bs - 1;
            }
        }
    }
    st.empires[pi].designs.remove(shipi);
    fleet::remove_empty(st);
    misc::update_visibility(st);
    misc::update_maint_costs(st);
    misc::update_have_reserve_fuel(st);
}

/// 1oom `game_ai_classic_design_scrap`.
fn ai_design_scrap(st: &mut State, pi: usize, shipi: usize) {
    design_scrap(st, pi, shipi, false);
    let e = &mut st.empires[pi];
    for si in [&mut e.shipi_colony, &mut e.shipi_bomber] {
        if *si == shipi as i32 {
            *si = -1;
        } else if *si > shipi as i32 {
            *si -= 1;
        }
    }
}

fn is_colony_special(s: u8) -> bool {
    (SP_COLONY_FIRST..=SP_COLONY_LAST).contains(&s)
}

/// 1oom `game_ai_classic_turn_p2_do`: scrap old designs, design new ones, choose what to build.
fn turn_p2_do(st: &mut State, pi: usize) {
    let mut ait = TurnP2 {
        shiptype: 0,
        have_pulsar: false,
        have_repulwarp: false,
    };
    for p2 in 0..st.empires.len() {
        if st.empires[p2].ai || !st.empires[p2].alive {
            continue;
        }
        for d in &st.empires[p2].designs {
            for &s in &d.specials {
                if s == SP_ENERGY_PULSAR || s == SP_IONIC_PULSAR {
                    ait.have_pulsar = true;
                }
                if s == SP_REPULSOR || s == SP_WARPDIS {
                    ait.have_repulwarp = true;
                }
            }
        }
    }
    st.empires[pi].ai_p2_countdown = st.rng.one_to(12) as i32 + 8;
    misc::update_maint_costs(st);
    let year = year_n(st);
    {
        let e = &mut st.empires[pi];
        e.design_year.resize(MAX_DESIGNS, 0);
        e.shipi_colony = -1;
        e.shipi_bomber = -1;
    }
    let mut num_non0 = 0;
    for i in 0..st.empires[pi].designs.len() {
        let e = &st.empires[pi];
        if e.shipcount.get(i).copied().unwrap_or(0) != 0 {
            num_non0 += 1;
        }
        let colony = e.designs[i].specials.iter().any(|&s| is_colony_special(s));
        let bomber = WEAPONS[e.designs[i].weapons[0] as usize].bomb;
        let e = &mut st.empires[pi];
        if colony {
            e.shipi_colony = i as i32;
        }
        if bomber {
            e.shipi_bomber = i as i32;
        }
    }
    let mut i = 0;
    while i < st.empires[pi].designs.len() && num_non0 > 1 {
        let dy = st.empires[pi].design_year[i];
        if (year < 100 && (year - dy) > 50) || (year >= 100 && (year - dy) > 100) {
            ai_design_scrap(st, pi, i);
            num_non0 -= 1;
        }
        i += 1;
    }
    let e = &st.empires[pi];
    let num_non0 = (0..e.designs.len())
        .filter(|&i| e.shipcount.get(i).copied().unwrap_or(0) != 0)
        .count();
    let mut remove: Option<usize> = None;
    if num_non0 > 4 {
        let mut oldest = 0xffff;
        for i in 0..e.designs.len() {
            let sy = e.design_year[i];
            if sy < oldest {
                oldest = sy;
                remove = Some(i);
            }
        }
    } else if e.designs.len() > 1 {
        remove = (0..e.designs.len()).find(|&i| e.shipcount.get(i).copied().unwrap_or(0) == 0);
    }
    if let Some(r) = remove {
        ai_design_scrap(st, pi, r);
    }
    let mut loops = 0;
    while st.empires[pi].designs.len() < MAX_DESIGNS && loops < 10_000 {
        let si = st.empires[pi].designs.len() as i32;
        let e = &mut st.empires[pi];
        if e.shipi_colony == -1 {
            e.shipi_colony = si;
            ait.shiptype = 0;
        } else if e.shipi_bomber == -1 {
            e.shipi_bomber = si;
            ait.shiptype = 2;
        } else {
            ait.shiptype = 1;
        }
        design_ship(st, &mut ait, pi);
        loops += 1;
    }
    for i in 0..st.empires[pi].designs.len() {
        let e = &st.empires[pi];
        let colony = e.designs[i].specials.iter().any(|&s| is_colony_special(s));
        let bomber = WEAPONS[e.designs[i].weapons[0] as usize].bomb;
        let e = &mut st.empires[pi];
        if colony {
            e.shipi_colony = i as i32;
        }
        if bomber {
            e.shipi_bomber = i as i32;
        }
    }
    let e = &st.empires[pi];
    let mut tbl_i: [usize; MAX_DESIGNS] = [0, 1, 2, 3, 4, 5];
    let mut tbl_year = [0i32; MAX_DESIGNS];
    for (i, y) in tbl_year.iter_mut().enumerate() {
        *y = e.design_year.get(i).copied().unwrap_or(0) & 0xffff;
        if i as i32 == e.shipi_colony {
            *y = 0;
        }
    }
    for _ in 0..MAX_DESIGNS - 1 {
        for i in 0..MAX_DESIGNS - 1 {
            if tbl_year[i + 1] > tbl_year[i] {
                tbl_year.swap(i, i + 1);
                tbl_i.swap(i, i + 1);
            }
        }
    }
    let stargates = e.ab.stargates;
    for i in 0..st.planets.len() {
        if st.planets[i].owner != Some(pi as u8) {
            continue;
        }
        let n = if !st.planets[i].stargate && stargates && st.rng.below(2) != 0 {
            BUILD_STARGATE
        } else {
            tbl_i[st.rng.below(3) as usize]
        };
        st.planets[i].build = n;
    }
}

/// 1oom `game_ai_classic_turn_p2`.
pub fn turn_p2(st: &mut State) {
    for pi in 0..st.empires.len() {
        if st.empires[pi].ai && st.empires[pi].alive {
            st.empires[pi].ai_p2_countdown -= 1;
            if st.empires[pi].ai_p2_countdown <= 0 {
                turn_p2_do(st, pi);
            }
        }
    }
}

// ------------------------------------------------------------------ sliders, spending

/// 1oom `ai_p3_tbl_w`: slider weights per objective, the last row at war.
const P3_W: [[i32; 9]; 7] = [
    [1, 2, 4, 1, 2, 2, 4, 2, 3],
    [4, 1, 2, 1, 3, 1, 4, 2, 1],
    [2, 4, 1, 1, 1, 1, 3, 5, 2],
    [1, 2, 3, 1, 4, 2, 3, 2, 2],
    [1, 2, 4, 1, 1, 5, 4, 1, 2],
    [1, 1, 2, 4, 1, 2, 4, 1, 5],
    [4, 1, 1, 1, 2, 2, 4, 2, 1],
];

/// 1oom `game_ai_classic_turn_p3_sub1`: what the spies should do next.
fn turn_p3_spies(st: &mut State, pi: usize) {
    for pi2 in 0..st.empires.len() {
        st.empires[pi].rel[pi2].spymode_next = SpyMode::Hide;
        let human_cf = !st.empires[pi2].ai && st.empires[pi2].rel[pi2].ceasefire > 0;
        if human_cf {
            continue;
        }
        if pi != pi2 && st.empires[pi].rel[pi2].contact {
            if treaty(st, pi, pi2) >= Treaty::War {
                st.empires[pi].rel[pi2].spymode_next = SpyMode::Sabotage;
            } else if st.empires[pi].race == Race::Darlok || st.rng.below(2) != 0 {
                let r = st.empires[pi].rel[pi2].relation;
                if st.rng.one_to(200) as i32 > r * 2 + 200 {
                    st.empires[pi].rel[pi2].spymode_next = SpyMode::Espionage;
                }
            }
        }
    }
}

/// 1oom `game_ai_classic_turn_p3`: planet sliders, research focus, security and spies.
pub fn turn_p3(st: &mut State) {
    let n = st.empires.len();
    for pi in 0..n {
        if !st.empires[pi].ai {
            continue;
        }
        let race = st.empires[pi].race;
        let mut ti = st.empires[pi].trait2 as usize;
        for pi2 in 0..n {
            if treaty(st, pi, pi2) >= Treaty::War {
                ti = 6;
            }
        }
        st.empires[pi].ai_p3_countdown -= 1;
        if st.empires[pi].ai_p3_countdown <= 0 {
            st.empires[pi].ai_p3_countdown = st.rng.one_to(5) as i32 + 1;
            misc::update_eco_on_waste(st, pi as u8, true);
            let w_ship = P3_W[ti][0];
            let w_def = w_ship + P3_W[ti][1];
            let w_ind = w_def + P3_W[ti][2];
            let w_eco = w_ind + P3_W[ti][2];
            for i in 0..st.planets.len() {
                if st.planets[i].owner != Some(pi as u8) {
                    continue;
                }
                let (pop, max_pop, max_pop2, special) = {
                    let p = &st.planets[i];
                    (p.pop, p.max_pop, p.base_max_pop, p.special)
                };
                {
                    let sl = &mut st.planets[i].sliders;
                    sl[SLIDER_SHIP] = 0;
                    sl[SLIDER_DEF] = 0;
                    sl[SLIDER_IND] = 0;
                    sl[SLIDER_TECH] = 0;
                    if sl[SLIDER_ECO] < 96 {
                        sl[SLIDER_SHIP] = 5;
                    }
                    if sl[SLIDER_ECO] < 91 {
                        sl[SLIDER_DEF] = 5;
                    }
                    if sl[SLIDER_ECO] < 81 {
                        sl[SLIDER_IND] = 10;
                    }
                    if sl[SLIDER_ECO] < 71 {
                        sl[SLIDER_TECH] = 25;
                    }
                    if pop < (max_pop * 3) / 4 {
                        sl[SLIDER_IND] = 50;
                    }
                    if (race == Race::Bulrathi || race == Race::Sakkra) && pop < max_pop2 {
                        sl[SLIDER_ECO] += 10;
                    }
                    if race == Race::Klackon && pop < max_pop2 {
                        sl[SLIDER_ECO] += 15;
                    }
                    if race == Race::Meklar {
                        sl[SLIDER_IND] += 20;
                    }
                    if race == Race::Psilon {
                        sl[SLIDER_TECH] += 20;
                    }
                    if race == Race::Mrrshan {
                        sl[SLIDER_SHIP] += 15;
                    }
                    if special == Special::Artifacts || special == Special::Orion {
                        sl[SLIDER_TECH] += 20;
                    }
                    if special as i32 >= Special::Rich as i32 {
                        sl[SLIDER_IND] += 20;
                        sl[SLIDER_SHIP] += 10;
                    }
                }
                let mut left = 100 - st.planets[i].sliders.iter().sum::<i32>();
                while left > 0 {
                    let r1 = st.rng.one_to(13) as i32;
                    let r2 = (st.rng.one_to(8) as i32 + 2).min(left);
                    left -= r2;
                    let sl = &mut st.planets[i].sliders;
                    if r1 <= w_ship {
                        sl[SLIDER_SHIP] += r2;
                    } else if r1 <= w_def {
                        sl[SLIDER_DEF] += r2;
                    } else if r1 <= w_ind {
                        sl[SLIDER_IND] += r2;
                    } else if r1 <= w_eco {
                        sl[SLIDER_ECO] += r2;
                    } else {
                        sl[SLIDER_TECH] += r2;
                    }
                }
            }
        }
        let cof = st.empires[pi].ab.colonist_oper_factories;
        let have_shield = st.empires[pi].ab.planet_shield;
        let diff = st.difficulty as i32;
        for p in st.planets.iter_mut().filter(|p| p.owner == Some(pi as u8)) {
            let sl = &mut p.sliders;
            if p.factories >= p.pop * cof {
                let si = sl[SLIDER_IND] & 0xff;
                sl[SLIDER_DEF] += si / 2;
                sl[SLIDER_TECH] += si / 2 + si % 2;
                sl[SLIDER_IND] = 0;
            }
            if p.bases > 4 && (p.bases * 3) >= (p.pop / (5 - diff)) && p.shield >= have_shield {
                // MOO1 bug: doubles the ship slider, which builds very large fleets.
                let v = sl[SLIDER_SHIP] & 0xffff;
                sl[SLIDER_SHIP] += v;
                sl[SLIDER_DEF] = 0;
            }
        }
        if (year_n(st) % 10) as usize == pi {
            let mut sl = [5u32; NUM_FIELDS];
            if year_n(st) < 30 && st.rng.below(2) != 0 {
                sl[FIELD_PROPULSION] = 75;
            } else {
                let focus_own = st.rng.below(4) == 0;
                if (race == Race::Bulrathi || race == Race::Mrrshan) && focus_own {
                    sl[FIELD_WEAPON] = 75;
                } else if race == Race::Darlok && focus_own {
                    sl[FIELD_COMPUTER] = 75;
                } else if race == Race::Meklar && focus_own {
                    let f = if st.rng.below(2) != 0 {
                        FIELD_CONSTRUCTION
                    } else {
                        FIELD_PLANETOLOGY
                    };
                    sl[f] = 75;
                } else if race == Race::Alkari && focus_own {
                    sl[FIELD_PROPULSION] = 75;
                } else if race == Race::Sakkra && focus_own {
                    sl[FIELD_PLANETOLOGY] = 75;
                } else {
                    let r1 = st.rng.one_to(16) as i32;
                    let w_comp = P3_W[ti][4];
                    let w_cons = w_comp + P3_W[ti][5];
                    let w_ff = w_cons + P3_W[ti][6];
                    let w_prop = w_ff + P3_W[ti][8];
                    let w_plan = w_prop + P3_W[ti][7];
                    let f = if r1 <= w_comp {
                        FIELD_COMPUTER
                    } else if r1 <= w_cons {
                        FIELD_CONSTRUCTION
                    } else if r1 <= w_ff {
                        FIELD_FORCE_FIELD
                    } else if r1 <= w_prop {
                        FIELD_PROPULSION
                    } else if r1 <= w_plan {
                        FIELD_PLANETOLOGY
                    } else {
                        FIELD_WEAPON
                    };
                    sl[f] = 75;
                }
            }
            st.empires[pi].tech.alloc = sl;
        }
        if year_n(st) > 30 {
            let mut totalspies: u32 = 0;
            for pi2 in 0..n {
                if st.empires[pi].rel[pi2].contact {
                    totalspies = totalspies.wrapping_add(st.empires[pi2].rel[pi].spies as u32);
                }
            }
            let mut sec: i32 = if totalspies == 0 {
                0
            } else {
                (st.rng.below(2) as u32).wrapping_mul(totalspies) as i16 as i32
            };
            sec += diff * 8;
            st.empires[pi].security = sec.min(100);
            turn_p3_spies(st, pi);
            for pi2 in 0..n {
                st.empires[pi].rel[pi2].spying = 0;
                st.empires[pi].rel[pi2].spymode = SpyMode::Hide;
                let next = st.empires[pi].rel[pi2].spymode_next;
                if pi != pi2
                    && st.empires[pi].rel[pi2].contact
                    && next != SpyMode::Hide
                    && (!st.empires[pi2].ai || st.rng.below(3) == 0)
                {
                    st.empires[pi].rel[pi2].spymode = next;
                    st.empires[pi].rel[pi2].spying = st.rng.one_to(5) as i32 + 5;
                }
            }
        }
    }
}

/// 1oom `game_ai_classic_bomb`: bomb when the planet has more people than the invaders.
pub fn bomb(st: &State, player: EmpireId, planet: PlanetId, pop_inbound: i32) -> bool {
    let mut pop_inbound = pop_inbound;
    if st.empires[player as usize].race == Race::Bulrathi {
        if classic(st) {
            pop_inbound += pop_inbound / 5;
        } else {
            pop_inbound = (pop_inbound * 215) / 100;
        }
    }
    let p = &st.planets[planet];
    let mut flag = p.pop > pop_inbound;
    if let Some(o) = p.owner {
        if !st.empires[o as usize].ai && st.empires[o as usize].rel[player as usize].ceasefire > 0 {
            flag = false;
        }
    }
    flag
}

/// 1oom `game_ai_classic_crank_tech` (plague, nova): everything into research.
pub fn crank_tech(st: &mut State, planet: PlanetId) {
    let Some(o) = st.planets[planet]
        .owner
        .filter(|&o| st.empires[o as usize].ai)
    else {
        return;
    };
    let sl = &mut st.planets[planet].sliders;
    let v = sl[SLIDER_SHIP] + sl[SLIDER_IND] + sl[SLIDER_DEF];
    sl[SLIDER_TECH] += v;
    sl[SLIDER_SHIP] = 0;
    sl[SLIDER_IND] = 0;
    sl[SLIDER_DEF] = 0;
    add_reserve(st, planet, o as usize);
}

/// 1oom `game_ai_classic_crank_ship` (comet, pirates): everything into ships.
pub fn crank_ship(st: &mut State, planet: PlanetId) {
    let Some(o) = st.planets[planet]
        .owner
        .filter(|&o| st.empires[o as usize].ai)
    else {
        return;
    };
    let sl = &mut st.planets[planet].sliders;
    let v = sl[SLIDER_TECH] + sl[SLIDER_IND] + sl[SLIDER_DEF];
    sl[SLIDER_SHIP] += v;
    sl[SLIDER_TECH] = 0;
    sl[SLIDER_IND] = 0;
    sl[SLIDER_DEF] = 0;
    add_reserve(st, planet, o as usize);
}
