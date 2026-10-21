//! Fleets and transports leaving planets (1oom `game_fleet.c`).
//!
//! The fleet and transport lists keep 1oom's order: new entries go to the end and removed
//! ones are replaced by the last entry (`util_table_remove_item_any_order`), because the
//! computer players walk these lists in order.

use crate::design::MAX_DESIGNS;
use crate::state::*;
use crate::types::*;

pub const FLEET_ENROUTE_MAX: usize = 260;
/// 1oom `FLEET_ENROUTE_AI_MAX`: the AI stops sending scouts and colony ships beyond this.
pub const FLEET_ENROUTE_AI_MAX: usize = 208;
pub const TRANSPORT_MAX: usize = 100;
const LIMIT_SHIPS: u32 = 32000;

fn new_fleet(
    st: &mut State,
    owner: EmpireId,
    x: i32,
    y: i32,
    dest: PlanetId,
    speed: i32,
    retreat: bool,
) -> Fleet {
    let id = st.alloc_id();
    let mut visible = vec![false; PLAYER_NUM];
    visible[owner as usize] = true;
    Fleet {
        id,
        owner,
        x,
        y,
        dest,
        ships: vec![0; MAX_DESIGNS],
        speed,
        retreat,
        visible,
    }
}

/// 1oom `game_send_fleet_do`: merges into a fleet already leaving the same planet for the
/// same destination at the same speed.
fn send_do(
    st: &mut State,
    owner: EmpireId,
    from: PlanetId,
    dest: PlanetId,
    ships: &[(usize, u32)],
    retreat: bool,
) -> bool {
    if !ships.iter().any(|&(_, n)| n != 0) {
        return false;
    }
    let (pf, pt) = (&st.planets[from], &st.planets[dest]);
    let speed = if pt.owner == Some(owner) && pf.owner == Some(owner) && pt.stargate && pf.stargate
    {
        SPEED_STARGATE
    } else {
        let e = &st.empires[owner as usize];
        ships
            .iter()
            .filter(|&&(_, n)| n > 0)
            .map(|&(t, _)| e.designs[t].engine as i32 + 1)
            .fold(100, i32::min)
    };
    let (fx, fy) = (pf.x, pf.y);
    for f in st.fleets.iter_mut() {
        if f.owner == owner && f.x == fx && f.y == fy && f.dest == dest && f.speed == speed {
            for &(t, n) in ships {
                if n > 0 {
                    f.ships[t] = (f.ships[t] + n).min(LIMIT_SHIPS);
                }
            }
            return true;
        }
    }
    if st.fleets.len() == FLEET_ENROUTE_MAX {
        return false;
    }
    let mut f = new_fleet(st, owner, fx, fy, dest, speed, retreat);
    for &(t, n) in ships {
        if n > 0 {
            f.ships[t] = n;
        }
    }
    st.fleets.push(f);
    true
}

fn from_orbit_do(
    st: &mut State,
    owner: EmpireId,
    from: PlanetId,
    dest: PlanetId,
    ships: &[(usize, u32)],
    retreat: bool,
) -> bool {
    if !send_do(st, owner, from, dest, ships, retreat) {
        return false;
    }
    let o = &mut st.empires[owner as usize].orbit[from];
    for &(t, n) in ships {
        o[t] = o[t].wrapping_sub(n);
    }
    true
}

/// 1oom `game_send_fleet_from_orbit`: `ships` are (design, count) pairs.
pub fn send_from_orbit(
    st: &mut State,
    owner: EmpireId,
    from: PlanetId,
    dest: PlanetId,
    ships: &[(usize, u32)],
) -> bool {
    if dest == from {
        return false;
    }
    from_orbit_do(st, owner, from, dest, ships, false)
}

/// 1oom `game_send_fleet_retreat`.
pub fn send_retreat(
    st: &mut State,
    owner: EmpireId,
    from: PlanetId,
    dest: PlanetId,
    ships: &[(usize, u32)],
) -> bool {
    from_orbit_do(st, owner, from, dest, ships, true)
}

/// 1oom `game_send_fleet_reloc`: newly built ships to the rally point.
pub fn send_reloc(
    st: &mut State,
    owner: EmpireId,
    from: PlanetId,
    dest: PlanetId,
    si: usize,
    n: u32,
) -> bool {
    send_do(st, owner, from, dest, &[(si, n)], false)
}

/// A fleet appended directly, as the AI does (1oom `game_turn_fleet_send`, scouts).
pub fn push_fleet(
    st: &mut State,
    owner: EmpireId,
    from: PlanetId,
    dest: PlanetId,
    speed: i32,
    ships: Vec<u32>,
) {
    let (x, y) = (st.planets[from].x, st.planets[from].y);
    let mut f = new_fleet(st, owner, x, y, dest, speed, false);
    f.ships = ships;
    st.fleets.push(f);
}

/// 1oom `game_send_transport`.
pub fn send_transport(st: &mut State, from: PlanetId) -> bool {
    if st.transports.len() == TRANSPORT_MAX {
        return false;
    }
    let pf = &st.planets[from];
    let dest = pf.trans_dest;
    let owner = pf.owner.expect("owned");
    let pt = &st.planets[dest];
    let mut speed = st.empires[owner as usize].ab.engine_warp;
    if pt.owner == Some(owner) && pt.stargate && pf.stargate {
        speed = SPEED_STARGATE;
    }
    let (x, y, n) = (pf.x, pf.y, pf.trans_num);
    let id = st.alloc_id();
    let mut visible = vec![false; PLAYER_NUM];
    visible[owner as usize] = true;
    st.transports.push(Transport {
        id,
        owner,
        x,
        y,
        dest,
        pop: n & 0xffff,
        speed,
        visible,
    });
    let pf = &mut st.planets[from];
    pf.pop = (pf.pop - n).max(1);
    pf.rebels = (pf.rebels - (n / 2 + 1)).max(0);
    pf.trans_num = 0;
    true
}

/// 1oom `game_remove_empty_fleets`.
pub fn remove_empty(st: &mut State) {
    let mut i = 0;
    while i < st.fleets.len() {
        if st.fleets[i].ships.iter().all(|&n| n == 0) {
            st.fleets.swap_remove(i);
        } else {
            i += 1;
        }
    }
    let mut i = 0;
    while i < st.transports.len() {
        if st.transports[i].pop == 0 {
            st.transports.swap_remove(i);
        } else {
            i += 1;
        }
    }
}

/// 1oom `game_remove_player_fleets`.
pub fn remove_player(st: &mut State, owner: EmpireId) {
    let mut i = 0;
    while i < st.fleets.len() {
        if st.fleets[i].owner == owner {
            st.fleets.swap_remove(i);
        } else {
            i += 1;
        }
    }
    let mut i = 0;
    while i < st.transports.len() {
        if st.transports[i].owner == owner {
            st.transports.swap_remove(i);
        } else {
            i += 1;
        }
    }
    for o in st.empires[owner as usize].orbit.iter_mut() {
        o.iter_mut().for_each(|n| *n = 0);
    }
}

/// 1oom `game_fleet_unrefuel`: ships out of fuel range are lost.
pub fn unrefuel(st: &mut State) {
    for pi in 0..st.empires.len() {
        for pli in 0..st.planets.len() {
            let wr = st.planets[pli].within_frange.get(pi).copied().unwrap_or(0);
            if wr == 1 {
                continue;
            }
            let mut lost = false;
            let nd = st.empires[pi].designs.len();
            for si in 0..nd {
                let reserve = st.empires[pi]
                    .have_reserve_fuel
                    .get(si)
                    .copied()
                    .unwrap_or(false);
                let r = &mut st.empires[pi].orbit[pli];
                if r[si] != 0 && (wr == 0 || (wr == 2 && !reserve)) {
                    lost = true;
                    r[si] = 0;
                }
            }
            if lost {
                let p = &mut st.planets[pli];
                p.unrefuel.resize(PLAYER_NUM, false);
                p.unrefuel[pi] = true;
            }
        }
    }
}

/// 1oom `game_fleet_redirect` (without the stargate redirection fix): a fleet sent back
/// to the planet it is leaving rejoins the orbit.
pub fn redirect(st: &mut State, fi: usize, pfrom: Option<PlanetId>, pto: PlanetId) {
    if pfrom == Some(pto) {
        let f = st.fleets.swap_remove(fi);
        let os = &mut st.empires[f.owner as usize].orbit[pto];
        for (i, n) in f.ships.iter().enumerate() {
            os[i] += n;
        }
    } else {
        st.fleets[fi].dest = pto;
    }
}
