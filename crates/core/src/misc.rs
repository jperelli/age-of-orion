//! Per-turn bookkeeping shared by the turn, the AI and the UI: production, maintenance,
//! research totals, ranges, contact and visibility (1oom `game_misc.c`), and the planet
//! helpers of 1oom `game_planet.c`.

use crate::data::SPECIALS;
use crate::design::KIND_FUEL;
use crate::diplo::Treaty;
use crate::state::*;
use crate::types::*;
use crate::umath;

/// 1oom `game_planet_get_fact_adj_cost` (without the factory cost fix).
pub fn fact_adj_cost(st: &State, p: &Planet) -> i32 {
    p.owner
        .map_or(10, |o| st.empires[o as usize].ab.factory_adj_cost)
}

/// 1oom `game_update_have_reserve_fuel`.
pub fn update_have_reserve_fuel(st: &mut State) {
    for e in st.empires.iter_mut() {
        e.have_reserve_fuel = (0..crate::design::MAX_DESIGNS)
            .map(|i| {
                e.designs.get(i).is_some_and(|d| {
                    d.specials
                        .iter()
                        .any(|&s| SPECIALS[s as usize].kind == KIND_FUEL && s != 0)
                })
            })
            .collect();
    }
}

/// Ships of every design an empire has, in orbit and in flight.
pub fn ship_totals(st: &State, pi: usize) -> Vec<u32> {
    let e = &st.empires[pi];
    let numsd = e.designs.len();
    let mut t = vec![0u32; crate::design::MAX_DESIGNS];
    for (si, ts) in t.iter_mut().enumerate().take(numsd) {
        for o in &e.orbit {
            *ts = ts.wrapping_add(o[si]);
        }
    }
    for f in st.fleets.iter().filter(|f| f.owner as usize == pi) {
        for (si, ts) in t.iter_mut().enumerate().take(numsd) {
            *ts = ts.wrapping_add(f.ships[si]);
        }
    }
    t
}

/// 1oom `game_update_maint_costs`.
pub fn update_maint_costs(st: &mut State) {
    for pi in 0..st.empires.len() {
        let t = ship_totals(st, pi);
        let mut total: u32 = 0;
        for (si, &n) in t.iter().enumerate() {
            let cost = st.empires[pi].designs.get(si).map_or(0, |d| d.cost_bc);
            total = total.wrapping_add(n.wrapping_mul(cost));
        }
        let max = st.rules.max_ship_maint as u32;
        let mut total = (total / 50).min(max);
        let mut bases: u32 = 0;
        for p in st.planets.iter().filter(|p| p.owner == Some(pi as u8)) {
            if p.stargate {
                total += st.rules.stargate_maint as u32;
            }
            bases = (bases + p.bases as u32) & 0xffff;
        }
        let total = total.min(max);
        let base_cost = st.base_cost(pi as u8) as u32;
        let e = &mut st.empires[pi];
        e.shipcount = t;
        e.ship_maint = total as i32;
        e.bases_maint = (bases * base_cost / 50) as i32;
    }
}

/// 1oom `game_update_production`.
pub fn update_production(st: &mut State) {
    update_maint_costs(st);
    let n = st.empires.len();
    for e in st.empires.iter_mut() {
        e.total_prod = 0;
    }
    for pi in 0..n {
        for pi2 in 0..n {
            let contact = st.empires[pi].rel.get(pi2).is_some_and(|r| r.contact);
            if pi == pi2 || !contact {
                if let Some(r) = st.empires[pi].rel.get_mut(pi2) {
                    r.spying = 0;
                }
            }
        }
    }
    for i in 0..st.planets.len() {
        let Some(owner) = st.planets[i].owner else {
            continue;
        };
        let p = &st.planets[i];
        let e = &st.empires[owner as usize];
        let pop = p.pop;
        let popx = (p.pop - p.rebels).max(0);
        let mut v = popx * e.ab.colonist_oper_factories;
        let factories = (p.factories & 0xffff).min(v);
        let mut extra = (pop * (e.tech.level[FIELD_PLANETOLOGY] as i32 * 3 + 50)) / 100;
        if e.race == Race::Klackon {
            extra <<= 1;
        }
        v = factories + extra;
        if e.ai {
            match st.difficulty {
                Difficulty::Easy => v = (v << 2) / 5,
                Difficulty::Average => v += v / 4,
                Difficulty::Hard => v += v / 2,
                Difficulty::Impossible => v += v,
                Difficulty::Simple => {}
            }
        }
        let reserve = (p.reserve as u32).min(v as u32);
        v = (v as u32).wrapping_add(reserve) as i32;
        if p.unrest == Unrest::Rebellion {
            v = 0;
        }
        let v = v.max(0);
        let pm = &mut st.planets[i];
        pm.prod = v & 0xffff;
        pm.total_prod = v & 0xffff;
        let e = &mut st.empires[owner as usize];
        e.total_prod = (e.total_prod as u32).wrapping_add(v as u32) as i32;
    }
    for pi in 0..n {
        let e = &st.empires[pi];
        let mut spysum: u32 = 0;
        let mut spymaint: u32 = 0;
        for pi2 in 0..n {
            let s = e.rel[pi2].spying as u32;
            spysum = (spysum + s) & 0xffff;
            spymaint = (spymaint + (e.total_prod as u32).wrapping_mul(s) / 1000) & 0xffff;
        }
        let bonus = if e.race == Race::Human { 25 } else { 0 };
        let mut trade: i32 = 0;
        for pi2 in 0..n {
            trade += (e.rel[pi2].trade_bc * (e.rel[pi2].trade_percent + bonus)) / 100;
        }
        let tp = e.total_prod as i64;
        let mut actual = (tp * (1000 - e.security as i64 - spysum as i64)) / 1000 + trade as i64
            - e.ship_maint as i64
            - e.bases_maint as i64
            - (tp * e.tax as i64) / 1000;
        if actual < 1 {
            actual = 1;
        }
        let actual = actual as i32;
        let percent = if e.total_prod != 0 {
            (actual as i64 * 100 / e.total_prod as i64) as i32 & 0xffff
        } else {
            0
        };
        let e = &mut st.empires[pi];
        e.spying_maint = spymaint as i32;
        e.total_trade = trade;
        e.maintenance = e.total_prod - actual;
        e.percent_prod_total_to_actual = percent;
        for p in st.planets.iter_mut().filter(|p| p.owner == Some(pi as u8)) {
            let v = (p.prod * percent) / 100 - p.trans_num;
            p.prod = v.max(0);
        }
    }
}

/// 1oom `game_get_tech_prod`.
pub fn tech_prod(prod: i32, slider: i32, race: Race, special: Special) -> i32 {
    let mut v = (prod * slider) / 100;
    if race == Race::Psilon {
        v += v / 2;
    }
    match special {
        Special::Artifacts => v *= 2,
        Special::Orion => v *= 4,
        _ => {}
    }
    v.min(0x7fff)
}

/// 1oom `game_update_total_research`.
pub fn update_total_research(st: &mut State) {
    for e in st.empires.iter_mut() {
        e.research = 0;
    }
    for (i, p) in st.planets.iter().enumerate() {
        let Some(owner) = p.owner else { continue };
        let plague = st.evn.plague.as_ref().is_some_and(|l| l.planet == i);
        let nova = st.evn.nova.as_ref().is_some_and(|l| l.planet == i);
        if plague || nova {
            continue;
        }
        let e = &st.empires[owner as usize];
        let v = tech_prod(p.prod, p.sliders[SLIDER_TECH], e.race, p.special);
        st.empires[owner as usize].research += v;
    }
}

/// 1oom `game_planet_get_waste_percent`, returning (percent, waste).
pub fn waste_percent(st: &State, p: &Planet, subtract_transports: bool) -> (i32, i32) {
    let Some(owner) = p.owner else {
        return (0, 0);
    };
    let e = &st.empires[owner as usize];
    let mut pop = p.pop;
    if subtract_transports {
        pop -= p.trans_num;
    }
    let fact = p.factories.min(pop * e.ab.colonist_oper_factories);
    let waste = if e.race == Race::Silicoid {
        0
    } else {
        ((fact * e.ab.ind_waste_scale) / 10 + p.waste) / e.ab.eco_restoration
    };
    let prod = if p.prod == 0 { 1000 } else { p.prod };
    let w = ((waste * 100) + prod - 1) / prod;
    (w.clamp(0, 100), waste)
}

/// 1oom `game_update_eco_on_waste` (without slider locks: they only exist for humans with
/// the `slider_respects_locks` option).
pub fn update_eco_on_waste(st: &mut State, player: EmpireId, force_adjust: bool) {
    if st.empires[player as usize].race == Race::Silicoid {
        return;
    }
    for i in 0..st.planets.len() {
        if st.planets[i].owner != Some(player) {
            continue;
        }
        let (v, _) = waste_percent(st, &st.planets[i], false);
        let p = &mut st.planets[i];
        let sl = &mut p.sliders;
        if sl[SLIDER_ECO] < v || force_adjust {
            let eco_diff = v - sl[SLIDER_ECO];
            let sum = 100;
            sl[SLIDER_ECO] = v;
            sl[SLIDER_SHIP] -= (sl[SLIDER_SHIP] * eco_diff) / sum;
            sl[SLIDER_DEF] -= (sl[SLIDER_DEF] * eco_diff) / sum;
            sl[SLIDER_IND] -= (sl[SLIDER_IND] * eco_diff) / sum;
        }
        sl[SLIDER_SHIP] = sl[SLIDER_SHIP].max(0);
        sl[SLIDER_DEF] = sl[SLIDER_DEF].max(0);
        sl[SLIDER_IND] = sl[SLIDER_IND].max(0);
        sl[SLIDER_ECO] = sl[SLIDER_ECO].clamp(0, 100);
        let left =
            (100 - sl[SLIDER_SHIP] - sl[SLIDER_DEF] - sl[SLIDER_IND] - sl[SLIDER_ECO]).max(0);
        sl[SLIDER_TECH] = left;
    }
}

fn set_seen(st: &mut State, pi: usize, i: PlanetId) {
    let p = &st.planets[i];
    let s = Seen {
        owner: p.owner,
        pop: p.pop,
        bases: p.bases,
        factories: p.factories,
    };
    let e = &mut st.empires[pi];
    if e.seen.len() < st.planets.len() {
        e.seen.resize(st.planets.len(), Seen::default());
    }
    e.seen[i] = s;
}

/// 1oom `game_update_seen_by_orbit`.
pub fn update_seen_by_orbit(st: &mut State, pi: usize) {
    for i in 0..st.planets.len() {
        let in_orbit = st.empires[pi].orbit[i].iter().any(|&n| n != 0);
        if st.planets[i].owner == Some(pi as u8) || in_orbit {
            set_seen(st, pi, i);
        }
    }
}

/// 1oom `game_turn_update_seen`.
pub fn update_seen(st: &mut State) {
    update_visibility(st);
    for i in 0..st.planets.len() {
        for pi in 0..st.empires.len() {
            let p = &st.planets[i];
            let mut can_see =
                p.within_srange.get(pi).copied().unwrap_or(false) || p.owner == Some(pi as u8);
            if !can_see {
                can_see = st.empires[pi].orbit[i].iter().any(|&n| n != 0);
            }
            if can_see {
                set_seen(st, pi, i);
            } else if let Some(o) = p.owner {
                if st.empires[pi].rel[o as usize].contact {
                    let n = st.planets.len();
                    let e = &mut st.empires[pi];
                    if e.seen.len() < n {
                        e.seen.resize(n, Seen::default());
                    }
                    e.seen[i].owner = Some(o);
                }
            }
        }
    }
}

/// 1oom `game_update_within_range`.
pub fn update_within_range(st: &mut State) {
    let n = st.empires.len();
    let np = st.planets.len();
    for p in st.planets.iter_mut() {
        p.within_frange.resize(PLAYER_NUM, 0);
        p.within_srange.resize(PLAYER_NUM, false);
    }
    for pi in 0..n {
        let e = &st.empires[pi];
        let mut alliance = [false; PLAYER_NUM];
        for (i, a) in alliance.iter_mut().enumerate().take(n) {
            *a = e.rel[i].treaty == Treaty::Alliance;
        }
        alliance[pi] = true;
        let frange = e.ab.fuel_range;
        let frangep3 = frange + 3;
        let srange = e.ab.scanner_range;
        let srange2 = match srange {
            5 => 10,
            7 => 20,
            9 => 30,
            _ => 0,
        };
        let tbl: Vec<usize> = (0..np)
            .filter(|&i| st.planets[i].owner.is_some_and(|o| alliance[o as usize]))
            .collect();
        for i in 0..np {
            if st.planets[i].owner == Some(pi as u8) {
                st.planets[i].within_frange[pi] = 1;
                st.planets[i].within_srange[pi] = true;
                continue;
            }
            let (mut mindist1, mut mindist2) = (0x2710i32, 0x2710i32);
            for &j in &tbl {
                if mindist1 <= frange && mindist2 <= srange {
                    break;
                }
                let dist = st.parsecs(i, j);
                mindist1 = mindist1.min(dist);
                if dist < mindist2 && st.planets[j].owner == Some(pi as u8) {
                    mindist2 = dist;
                }
            }
            st.planets[i].within_frange[pi] = if mindist1 <= frange {
                1
            } else if mindist1 <= frangep3 {
                2
            } else {
                0
            };
            st.planets[i].within_srange[pi] = mindist2 <= srange;
            // Fleets scan too (MOO1 v1.3a's ship scanner fix, always on in 1oom).
            if !st.planets[i].within_srange[pi] {
                let (px, py) = (st.planets[i].x, st.planets[i].y);
                let mut mindist1 = 10000;
                for f in &st.fleets {
                    if mindist1 <= srange2 {
                        break;
                    }
                    if f.owner as usize == pi {
                        mindist1 = mindist1.min(umath::dist_fast(f.x, f.y, px, py));
                    }
                }
                if mindist1 <= srange2 {
                    st.planets[i].within_srange[pi] = true;
                }
            }
        }
    }
    for i in 0..np {
        for pi in 0..n {
            if st.empires[pi].orbit[i].iter().any(|&s| s != 0) {
                st.planets[i].within_srange[pi] = true;
            }
        }
    }
}

/// 1oom `game_update_empire_contact`.
pub fn update_empire_contact(st: &mut State) {
    let n = st.empires.len();
    for pi1 in 0..n {
        for pi2 in pi1 + 1..n {
            let frange = st.empires[pi1]
                .ab
                .fuel_range
                .max(st.empires[pi2].ab.fuel_range);
            let mut mindist = 0xffi32;
            for (i1, p1) in st.planets.iter().enumerate() {
                if p1.owner != Some(pi1 as u8) {
                    continue;
                }
                for (i2, p2) in st.planets.iter().enumerate() {
                    if p2.owner == Some(pi2 as u8) {
                        mindist = mindist.min(st.parsecs(i1, i2));
                    }
                }
            }
            let c = mindist <= frange;
            st.empires[pi1].rel[pi2].contact = c;
            st.empires[pi2].rel[pi1].contact = c;
        }
    }
}

/// 1oom `game_check_coord_is_visible`.
pub fn coord_visible(st: &State, pi: usize, range: i32, x: i32, y: i32) -> bool {
    let range = range * 10;
    for p in &st.planets {
        if p.owner == Some(pi as u8) && umath::dist_fast(x, y, p.x, p.y) <= range {
            return true;
        }
    }
    let range = (range - 30) / 2;
    for (i, p) in st.planets.iter().enumerate() {
        let snum: u32 = st.empires[pi].orbit[i].iter().sum();
        if snum != 0 && umath::dist_fast(x, y, p.x, p.y) <= range {
            return true;
        }
    }
    st.fleets
        .iter()
        .any(|f| f.owner as usize == pi && umath::dist_fast(x, y, f.x, f.y) <= range)
}

/// 1oom `game_update_visibility` (fleets and transports; orbits are always shown).
pub fn update_visibility(st: &mut State) {
    let n = st.empires.len();
    let mut fv = Vec::with_capacity(st.fleets.len());
    for f in &st.fleets {
        fv.push(
            (0..n)
                .map(|pi| {
                    f.owner as usize == pi
                        || coord_visible(st, pi, st.empires[pi].ab.scanner_range, f.x, f.y)
                })
                .collect::<Vec<bool>>(),
        );
    }
    for (f, v) in st.fleets.iter_mut().zip(fv) {
        f.visible = v;
    }
    let mut tv = Vec::with_capacity(st.transports.len());
    for t in &st.transports {
        tv.push(
            (0..n)
                .map(|pi| {
                    t.owner as usize == pi
                        || coord_visible(st, pi, st.empires[pi].ab.scanner_range, t.x, t.y)
                })
                .collect::<Vec<bool>>(),
        );
    }
    for (t, v) in st.transports.iter_mut().zip(tv) {
        t.visible = v;
    }
}

/// 1oom `game_adjust_prod_by_special`.
pub fn adjust_prod_by_special(prod: i32, special: Special) -> i32 {
    match special {
        Special::UltraPoor => prod / 3,
        Special::Poor => prod / 2,
        Special::Rich => prod * 2,
        Special::UltraRich => prod * 3,
        _ => prod,
    }
}

/// 1oom `game_get_pop_growth_max`.
pub fn pop_growth_max(st: &State, p: &Planet, max_pop3: i32) -> i32 {
    let e = &st.empires[p.owner.expect("owned") as usize];
    let mut v;
    if e.race == Race::Silicoid {
        v = (100 - (p.pop * 100) / max_pop3) / 2;
    } else {
        v = 100 - ((p.pop + p.waste) * 100) / max_pop3;
        match p.growth {
            Growth::Hostile => {
                if v >= 0 {
                    v /= 2;
                } else {
                    v *= 2;
                }
            }
            Growth::Normal => {}
            Growth::Fertile => {
                if v >= 0 {
                    v += v / 2;
                } else {
                    v = (v * 2) / 3;
                }
            }
            Growth::Gaia => {
                if v >= 0 {
                    v *= 2;
                } else {
                    v /= 2;
                }
            }
        }
        if e.race == Race::Sakkra {
            v *= 2;
        }
    }
    let mut r = (v * p.pop + 5) / 100;
    r += p.pop_tenths;
    if v > 0 && r < 1 {
        r = 1;
    }
    if (p.pop + r / 10) < 1 {
        r = 1;
    }
    r
}

/// 1oom `game_get_pop_growth_for_eco`.
pub fn pop_growth_for_eco(st: &State, p: &Planet, eco: i32) -> i32 {
    let e = &st.empires[p.owner.expect("owned") as usize];
    let mut v = (eco * 10) / e.ab.inc_pop_cost;
    if e.race == Race::Sakkra {
        v *= 2;
    } else if e.race == Race::Silicoid {
        v /= 2;
    }
    let vmax = (p.pop * 10) / 4;
    v.min(vmax)
}

/// 1oom `game_turn_atmos_tform`.
pub fn atmos_tform(p: &mut Planet) {
    let inc = if (p.kind as i32) < PlanetType::Dead as i32 {
        20
    } else if (p.kind as i32) < PlanetType::Barren as i32 {
        10
    } else {
        0
    };
    p.kind = PlanetType::Minimal;
    p.size += inc;
    p.base_max_pop += inc;
    p.max_pop += inc;
    p.growth = Growth::Normal;
}

/// 1oom `game_turn_soil_enrich`.
pub fn soil_enrich(p: &mut Planet, best_tform: i32, advanced: bool, max_pop: i32) {
    p.growth = if advanced {
        Growth::Gaia
    } else {
        Growth::Fertile
    };
    let mut inc = if advanced {
        let mut v = (p.size / 10) * 5;
        if (p.size / 10) % 5 != 0 {
            v += 5;
        }
        v
    } else {
        let mut v = (p.size / 20) * 5;
        if (p.size / 20) % 5 != 0 {
            v += 5;
        }
        v
    };
    inc = inc.max(5);
    let old = p.base_max_pop;
    p.base_max_pop = p.size + inc;
    p.max_pop += p.base_max_pop - old;
    p.max_pop = p.max_pop.min(best_tform + p.base_max_pop);
    p.max_pop = p.max_pop.min(max_pop);
}

/// 1oom `game_planet_get_random` (consumes a random number only when the empire has
/// planets).
pub fn planet_random(st: &mut State, owner: EmpireId) -> Option<PlanetId> {
    let tbl: Vec<PlanetId> = (0..st.planets.len())
        .filter(|&i| st.planets[i].owner == Some(owner))
        .collect();
    if tbl.is_empty() {
        None
    } else {
        Some(tbl[st.rng.below(tbl.len() as u16) as usize])
    }
}

/// 1oom `game_planet_destroy`: the colony is gone, the planet stays.
pub fn planet_destroy(st: &mut State, planet: PlanetId, _attacker: Option<EmpireId>) {
    let owner = st.planets[planet].owner;
    if let Some(o) = owner {
        let n = st.planets.len();
        let e = &mut st.empires[o as usize];
        if e.seen.len() < n {
            e.seen.resize(n, Seen::default());
        }
        e.seen[planet] = Seen::default();
    }
    let p = &mut st.planets[planet];
    p.rebels = 0;
    p.unrest = Unrest::Normal;
    p.reserve = 0;
    p.prev_owner = owner;
    p.owner = None;
    p.bc_to_ecoproj = 0;
    p.pop = 0;
    p.pop_prev = 0;
    p.prod = 0;
    p.bc_to_ship = 0;
    p.inbound = vec![0; PLAYER_NUM];
    p.build = 0;
    p.sliders = [0, 0, 100, 0, 0];
    p.slider_lock = [false; NUM_SLIDERS];
    p.rally = None;
    p.bases = 0;
    p.bc_upgrade_base = 0;
    p.bc_to_base = 0;
    p.trans_num = 0;
    p.bc_to_factory = 0;
    p.stargate = false;
    p.shield = 0;
    p.bc_to_shield = 0;
    p.finished = [false; FINISHED_NUM];
    for q in st.planets.iter_mut() {
        if q.rally == Some(planet) {
            q.rally = None;
        }
    }
}

/// 1oom `game_planet_update_home`.
pub fn update_home(st: &mut State) {
    for i in 0..st.planets.len() {
        if let Some(o) = st.planets[i].owner {
            let e = &mut st.empires[o as usize];
            if !e.alive {
                e.alive = true;
                e.home = i;
            }
        }
    }
}

/// 1oom `game_stat_prod`: production on a doubling scale.
pub fn stat_prod(st: &State, pi: usize) -> i32 {
    let mut prod = st.empires[pi].total_prod.max(0) as u32;
    let (mut va, mut ve, mut si) = (100u32, 0u32, 0i32);
    while prod > va {
        prod -= va;
        si += 1;
        ve = va;
        va <<= 1;
    }
    ((prod * 10) / (va - ve)) as i32 + si * 10
}

/// 1oom `game_stat_pop`.
pub fn stat_pop(st: &State, pi: usize) -> i32 {
    let num = st.planets.len() as i32;
    let sum: i32 = st
        .planets
        .iter()
        .filter(|p| p.owner == Some(pi as u8))
        .map(|p| p.pop)
        .sum();
    sum / ((num * 3) / 2 + 1)
}

/// 1oom `game_stat_tech`.
pub fn stat_tech(st: &State, pi: usize) -> i32 {
    st.empires[pi].tech.level.iter().sum::<u32>() as i32
}

/// The empires from first to last in production (0), population (1), technology (2) or
/// fleet strength (3), for the GNN statistics newscast (1oom `game_news_get_msg`).
pub fn stats_ranking(st: &State, kind: u8) -> Vec<EmpireId> {
    let mut tbl: Vec<(EmpireId, i32)> = Vec::new();
    for i in 0..st.empires.len() {
        if !st.empires[i].alive {
            continue;
        }
        let v = match kind {
            0 => stat_prod(st, i),
            1 => stat_pop(st, i),
            2 => stat_tech(st, i),
            _ => crate::diplo::stat_fleet(st, i as u8),
        }
        .max(1);
        tbl.push((i as u8, v));
    }
    let num = tbl.len();
    for _ in 0..num {
        for i in 0..num.saturating_sub(1) {
            if tbl[i].1 < tbl[i + 1].1 {
                tbl.swap(i, i + 1);
            }
        }
    }
    tbl.into_iter().map(|(e, _)| e).collect()
}

#[cfg(test)]
mod tests {
    use crate::{Game, NewGameOptions};

    #[test]
    fn stats_rank_every_living_empire() {
        let mut g = Game::new(&NewGameOptions {
            seed: 9,
            players: 4,
            all_ai: true,
            ..Default::default()
        })
        .unwrap();
        for _ in 0..30 {
            g.end_turn_auto();
        }
        let st = g.state();
        for kind in 0..4 {
            let r = super::stats_ranking(st, kind);
            assert_eq!(r.len(), 4);
            if kind == 1 {
                let v: Vec<i32> = r
                    .iter()
                    .map(|&e| super::stat_pop(st, e as usize).max(1))
                    .collect();
                assert!(v.windows(2).all(|w| w[0] >= w[1]));
            }
        }
    }
}
