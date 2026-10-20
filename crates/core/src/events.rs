//! Random events, space monsters, rebellions and coups (1oom `game_event.c`,
//! `game_turn_coup`). Each event reports with `EventKind::News` (`news.<kind>.<stage>`).

use serde::{Deserialize, Serialize};

use crate::diplo::{self, Treaty};
use crate::state::*;
use crate::tech::{self, Source};
use crate::turn::tech_prod;
use crate::types::*;

pub const EVENT_NUM: usize = 17;
const PLAGUE: u16 = 1;
const QUAKE: u16 = 2;
const NOVA: u16 = 3;
const ACCIDENT: u16 = 4;
const ASSASSIN: u16 = 5;
const VIRUS: u16 = 6;
const COMET: u16 = 7;
const PIRATES: u16 = 8;
const DERELICT: u16 = 9;
const REBELLION: u16 = 10;
const CRYSTAL: u16 = 11;
const AMOEBA: u16 = 12;
const ENVIRO: u16 = 13;
const RICH: u16 = 14;
const SUPPORT: u16 = 15;
const POOR: u16 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonsterKind {
    Crystal,
    Amoeba,
}

/// A space monster roaming the map (1oom `monster_t`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Monster {
    pub kind: MonsterKind,
    pub x: f64,
    pub y: f64,
    pub dest: PlanetId,
    /// Turns it waits at a planet before destroying it.
    pub counter: i32,
    pub nuked: i32,
    pub killer: Option<EmpireId>,
    pub hp: u32,
}

/// An event that lasts several turns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lasting {
    pub planet: PlanetId,
    pub player: EmpireId,
    /// Research points still needed (plague, nova) or hit points (comet, pirates).
    pub val: i32,
    pub years: i32,
    pub damage: i32,
    /// 1 just started, 2 going on.
    pub stage: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EventState {
    /// Game year (counted from 1) of the last event (1oom `evn.year`).
    pub last_year: i32,
    pub done: Vec<bool>,
    pub plague: Option<Lasting>,
    pub nova: Option<Lasting>,
    pub accident: Option<Lasting>,
    pub comet: Option<Lasting>,
    pub pirates: Option<Lasting>,
    pub monsters: Vec<Monster>,
    pub report_stars: u8,
}

impl EventState {
    pub fn new() -> Self {
        EventState {
            last_year: 40,
            done: vec![false; EVENT_NUM],
            ..Default::default()
        }
    }
}

/// 1oom `tbl_monster`: hit points, defense and absorb per difficulty.
pub fn monster_stats(kind: MonsterKind, diff: Difficulty) -> (u32, i32, i32) {
    let d = diff as u32;
    match kind {
        MonsterKind::Crystal => (3000 + 1000 * d, d as i32 + 1, 5),
        MonsterKind::Amoeba => (1000 + 1000 * d, d as i32 + 1, 0),
    }
}

fn year_n(st: &State) -> i32 {
    (st.year as i32 - st.rules.start_year as i32 + 1).max(1)
}

#[allow(clippy::too_many_arguments)]
fn news(
    st: &mut State,
    to: Vec<EmpireId>,
    kind: &str,
    stage: u8,
    planet: Option<PlanetId>,
    empire: Option<EmpireId>,
    n1: i32,
    n2: i32,
) {
    st.push_event(
        to,
        EventKind::News {
            news: kind.to_string(),
            stage,
            planet,
            empire,
            other: None,
            n1,
            n2,
        },
    );
}

fn planets_of(st: &State) -> Vec<i32> {
    let mut v = vec![0; st.empires.len()];
    for p in &st.planets {
        if let Some(o) = p.owner {
            v[o as usize] += 1;
        }
    }
    v
}

/// 1oom `game_event_new_get_victim`: empires with more planets are hit more often; humans
/// five times as often.
fn victim(st: &mut State) -> EmpireId {
    let mut t: Vec<i32> = planets_of(st)
        .iter()
        .enumerate()
        .map(|(i, &n)| {
            if n < 4 {
                0
            } else if !st.empires[i].ai {
                n * 5
            } else {
                n
            }
        })
        .collect();
    let min = if t.len() < 6 {
        0
    } else {
        t.iter().copied().min().unwrap_or(0)
    }
    .max(5);
    let mut sum = 0;
    for v in t.iter_mut() {
        *v = (*v / min).max(1);
        sum += *v;
    }
    let mut r = st.rng.below(sum as u16) as i32;
    let mut player = 0;
    while r >= 0 && player < t.len() - 1 {
        r -= t[player];
        if r >= 0 {
            player += 1;
        }
    }
    player as u8
}

fn random_planet(st: &mut State, owner: EmpireId) -> Option<PlanetId> {
    let v: Vec<PlanetId> = st.colonies(owner).map(|(i, _)| i).collect();
    if v.is_empty() {
        None
    } else {
        Some(v[st.rng.below(v.len() as u16) as usize])
    }
}

/// 1oom `game_monster_set_start`: monsters enter from an edge of the map.
fn monster_start(st: &mut State, kind: MonsterKind) -> Monster {
    let (w, h) = st.galaxy_size.grid();
    let (w, h) = (w as i32, h as i32);
    let (x, y, dest) = match st.rng.below(4) {
        0 => {
            let v = st.rng.below(h as u16) as i32;
            (0, (v - 1) * 32 + st.rng.one_to(12) as i32 + 10, v * w)
        }
        1 => {
            let v = st.rng.below(w as u16) as i32;
            ((v - 1) * 28 + st.rng.one_to(12) as i32 + 30, 0, v)
        }
        2 => {
            let v = st.rng.below(h as u16) as i32;
            (
                w * 28,
                (v - 1) * 32 + st.rng.one_to(12) as i32 + 10,
                (v + 1) * w - 1,
            )
        }
        _ => {
            let v = st.rng.below(w as u16) as i32;
            (
                (v - 1) * 28 + st.rng.one_to(12) as i32 + 30,
                h * 32,
                (h - 1) * w + v,
            )
        }
    };
    let dest = (dest.max(0) as usize).min(st.planets.len() - 1);
    Monster {
        kind,
        x: x as f64,
        y: y as f64,
        dest,
        counter: 2,
        nuked: 0,
        killer: None,
        hp: monster_stats(kind, st.difficulty).0,
    }
}

/// 1oom `game_monster_set_next_dest`: wander to a neighbouring star.
fn monster_next(st: &mut State, m: &mut Monster) {
    let (w, h) = st.galaxy_size.grid();
    let (w, h) = (w as i32, h as i32);
    let old = m.dest as i32;
    let n = st.planets.len() as i32;
    let mut dest = old;
    let mut tries = 0;
    while dest == old && tries < 100 {
        tries += 1;
        dest = old;
        let x = old % w;
        if x == 0 {
            dest += st.rng.below(2) as i32;
        } else {
            dest += st.rng.one_to(if x == w - 1 { 2 } else { 3 }) as i32 - 2;
        }
        if old < w {
            dest += st.rng.below(2) as i32 * w;
        } else {
            dest += st.rng.one_to(if w * h - w - 1 < old { 2 } else { 3 }) as i32 * w - w * 2;
        }
        if dest < 0
            || dest >= n
            || !st.planets[dest as usize].habitable()
            || dest as usize == st.orion
        {
            dest = old;
        }
    }
    m.dest = dest as usize;
}

/// Monsters move one parsec per turn (1oom `game_turn_move_ships`).
pub fn move_monsters(st: &mut State) {
    let parsec = st.rules.parsec;
    let planets = st.planets.clone();
    for m in st.evn.monsters.iter_mut() {
        if m.counter > 0 {
            continue;
        }
        let (tx, ty) = (planets[m.dest].x as f64, planets[m.dest].y as f64);
        let d = ((tx - m.x).powi(2) + (ty - m.y).powi(2)).sqrt();
        if d <= parsec {
            m.x = tx;
            m.y = ty;
        } else {
            m.x += (tx - m.x) * parsec / d;
            m.y += (ty - m.y) * parsec / d;
        }
    }
}

/// The monster sitting at `planet`, if any.
pub fn monster_at(st: &State, planet: PlanetId) -> Option<usize> {
    let p = &st.planets[planet];
    st.evn.monsters.iter().position(|m| {
        m.killer.is_none()
            && m.dest == planet
            && (m.x - p.x as f64).abs() < 0.5
            && (m.y - p.y as f64).abs() < 0.5
    })
}

/// 1oom `game_event_new`: maybe start a new event.
pub fn new_event(st: &mut State) {
    let year = year_n(st);
    let mut chance = (year - st.evn.last_year).max(0);
    chance = match st.difficulty {
        Difficulty::Simple => chance / 2,
        Difficulty::Easy => chance * 2 / 3,
        Difficulty::Average => chance * 3 / 4,
        Difficulty::Hard => chance * 4 / 5,
        Difficulty::Impossible => chance,
    };
    if st.rng.one_to(512) as i32 > chance {
        return;
    }
    let player = victim(st);
    let Some(planet) = random_planet(st, player) else {
        return;
    };
    if planet == st.orion {
        return;
    }
    let e = &st.empires[player as usize];
    let human = !e.ai;
    let race = e.race;
    let p = st.planets[planet].clone();
    let monster_alive = !st.evn.monsters.is_empty();
    let trader = (0..st.empires.len())
        .any(|i| i != player as usize && e.rel[i].contact && e.rel[i].trade_bc != 0);
    let partner = (0..st.empires.len() as u8).rev().find(|&i| {
        i != player
            && st.empires[i as usize].ai
            && matches!(
                e.rel[i as usize].treaty,
                Treaty::NonAggression | Treaty::Alliance
            )
    });
    let home = st.empires[player as usize].home;
    let mut kind = 0;
    for _ in 0..5 {
        kind = st.rng.one_to(EVENT_NUM as u16 - 1);
        let bad = st.evn.done[kind as usize]
            || (kind == REBELLION && (!human || home == planet))
            || (kind == QUAKE && p.pop < 50)
            || ((kind == CRYSTAL || kind == AMOEBA) && (year < 200 || monster_alive))
            || (kind == DERELICT && human)
            || (kind == ASSASSIN && (st.council.final_war || partner.is_none()))
            || (kind == COMET && year < 100)
            || (kind == PIRATES && !trader)
            || ((kind == ACCIDENT || kind == PLAGUE) && race == Race::Silicoid)
            || (kind == ENVIRO
                && (p.kind < PlanetType::Minimal
                    || p.growth >= Growth::Fertile
                    || p.special != Special::Normal))
            || (kind == RICH && p.special != Special::Normal)
            || (kind == POOR && p.special != Special::Normal);
        if bad {
            kind = 0;
        } else {
            break;
        }
    }
    if kind == 0 {
        return;
    }
    st.evn.last_year = year;
    st.evn.done[kind as usize] = true;
    let diff = st.difficulty as i32;
    let lasting = |val: i32, years: i32| Lasting {
        planet,
        player,
        val,
        years,
        damage: 0,
        stage: 1,
    };
    match kind {
        PLAGUE => {
            let v = (st.rng.one_to(8) as i32 + diff * 2) * p.prod;
            st.evn.plague = Some(lasting(v, 0));
        }
        QUAKE => {
            let vp = ((st.rng.one_to(10) as i32 + 20) * p.pop / 100).min(p.pop);
            let vf = ((st.rng.one_to(50) as i32 + 30) * p.factories / 100).min(p.factories);
            let pl = &mut st.planets[planet];
            pl.pop -= vp;
            pl.factories -= vf;
            news(
                st,
                vec![],
                "quake",
                if human { 0 } else { 4 },
                Some(planet),
                Some(player),
                vp,
                vf,
            );
        }
        NOVA => {
            let years = st.rng.one_to(5) as i32 + 10 - diff;
            let val = (st.rng.one_to(5) as i32 + 10 - diff) * p.prod;
            st.evn.nova = Some(lasting(val, years));
        }
        ACCIDENT => st.evn.accident = Some(lasting(0, 0)),
        ASSASSIN => {
            if let Some(p2) = partner {
                diplo::start_war(st, p2, player);
                if human {
                    let r = diplo::rel_mut(st, player, p2);
                    r.diplo_type = diplo::msg::WAR_DECLARED;
                    r.diplo_val = 100;
                }
                st.push_event(
                    vec![],
                    EventKind::News {
                        news: "assassin".into(),
                        stage: 0,
                        planet: None,
                        empire: Some(player),
                        other: Some(p2),
                        n1: 0,
                        n2: 0,
                    },
                );
            }
        }
        VIRUS => {
            let f = [
                FIELD_COMPUTER,
                FIELD_FORCE_FIELD,
                FIELD_PROPULSION,
                FIELD_WEAPON,
            ][st.rng.below(4) as usize];
            let lost = st.empires[player as usize].tech.invested[f] as i32;
            st.empires[player as usize].tech.invested[f] = 0;
            news(st, vec![], "virus", 4, None, Some(player), f as i32, lost);
        }
        COMET => {
            let years = st.rng.one_to(5) as i32 + 10 - diff;
            let hp = (st.rng.one_to(5) as i32 + 10 + diff) * 25;
            st.evn.comet = Some(lasting(hp, years));
        }
        PIRATES => {
            let pl = st.rng.below(st.planets.len() as u16) as usize;
            let hp = (st.rng.one_to(5) as i32 + 10 + diff) * 30;
            st.evn.pirates = Some(Lasting {
                planet: pl,
                player,
                val: hp,
                years: 0,
                damage: 0,
                stage: 1,
            });
        }
        DERELICT => {
            for f in [FIELD_WEAPON, FIELD_FORCE_FIELD] {
                let tier = st.empires[player as usize].tech.level[f] as usize / 5;
                for i in 0..(tier + 2).min(crate::tech::TIERS) {
                    let t = st.empires[player as usize].tech.list[f][i][0];
                    tech::grant(st, player, f, t, Source::Research);
                }
            }
            news(st, vec![], "derelict", 0, None, Some(player), 0, 0);
        }
        REBELLION => {
            let pl = &mut st.planets[planet];
            pl.rebels = pl.pop / 2;
            pl.unrest = Unrest::Rebellion;
            pl.unrest_reported = false;
        }
        CRYSTAL | AMOEBA => {
            let k = if kind == CRYSTAL {
                MonsterKind::Crystal
            } else {
                MonsterKind::Amoeba
            };
            let m = monster_start(st, k);
            st.evn.monsters.push(m);
            news(
                st,
                vec![],
                if kind == CRYSTAL { "crystal" } else { "amoeba" },
                0,
                None,
                None,
                0,
                0,
            );
        }
        ENVIRO => {
            let tf = st.planets[planet]
                .owner
                .map_or(0, |o| st.empires[o as usize].abilities().terraform);
            let max = st.rules.max_pop;
            crate::turn::soil_enrich(&mut st.planets[planet], tf, false, max);
            news(
                st,
                vec![],
                "enviro",
                if human { 0 } else { 4 },
                Some(planet),
                Some(player),
                0,
                0,
            );
        }
        RICH => {
            st.planets[planet].special = Special::Rich;
            news(
                st,
                vec![],
                "rich",
                if human { 0 } else { 4 },
                Some(planet),
                Some(player),
                0,
                0,
            );
        }
        SUPPORT => {
            let v = (year * 10 / 100 * 100).max(500);
            st.empires[player as usize].reserve += v;
            news(
                st,
                vec![],
                "support",
                if human { 0 } else { 4 },
                None,
                Some(player),
                v,
                0,
            );
        }
        POOR => {
            st.planets[planet].special = Special::Poor;
            news(
                st,
                vec![],
                "poor",
                if human { 0 } else { 4 },
                Some(planet),
                Some(player),
                0,
                0,
            );
        }
        _ => {}
    }
}

/// Planets whose research does not count this turn (fighting a plague or a nova).
pub fn blocked_research(st: &State, planet: PlanetId) -> bool {
    st.evn.plague.as_ref().is_some_and(|l| l.planet == planet)
        || st.evn.nova.as_ref().is_some_and(|l| l.planet == planet)
}

/// 1oom `game_event_run`: lasting events, monsters, rebellions.
pub fn run(st: &mut State) {
    if let Some(mut l) = st.evn.plague.take() {
        let p = &st.planets[l.planet];
        if p.owner == Some(l.player) {
            let race = st.empires[l.player as usize].race;
            let mut killed = 0;
            if l.stage != 1 {
                l.val -= tech_prod(p, race);
                killed = ((st.rng.one_to(6) as i32 + 4) * p.pop / 100).min(p.pop);
                st.planets[l.planet].pop -= killed;
            }
            let human = !st.empires[l.player as usize].ai;
            let done = l.val <= 0;
            let stage = match (l.stage, done) {
                (_, true) => Some(if human { 2 } else { 5 }),
                (1, _) => Some(if human { 0 } else { 4 }),
                _ => (human && st.rng.below(10) == 0).then_some(1),
            };
            if let Some(s) = stage {
                news(
                    st,
                    vec![],
                    "plague",
                    s,
                    Some(l.planet),
                    Some(l.player),
                    killed,
                    l.val,
                );
            }
            if st.empires[l.player as usize].ai {
                let pl = &mut st.planets[l.planet];
                pl.sliders = [
                    0,
                    0,
                    0,
                    pl.sliders[SLIDER_ECO].min(30),
                    100 - pl.sliders[SLIDER_ECO].min(30),
                ];
            }
            if !done {
                l.stage = 2;
                st.evn.plague = Some(l);
            }
        }
    }
    if let Some(mut l) = st.evn.nova.take() {
        let owner = st.planets[l.planet].owner;
        if l.stage != 1 {
            if let Some(o) = owner {
                let race = st.empires[o as usize].race;
                l.val -= tech_prod(&st.planets[l.planet], race);
                l.years -= 1;
            }
        }
        let mut finished = l.val <= 0;
        let mut exploded = false;
        if l.years <= 0 && !finished {
            let v = st.rng.one_to(10) as i32 + 10;
            let pl = &mut st.planets[l.planet];
            pl.base_max_pop = v;
            pl.max_pop = v;
            pl.kind = PlanetType::Radiated;
            pl.growth = Growth::Hostile;
            let keep = st.rng.one_to(v as u16) as i32;
            let pl = &mut st.planets[l.planet];
            pl.pop = pl.pop.min(keep);
            if owner.is_none() {
                pl.pop = 0;
            }
            let f = (st.rng.one_to(3) as i32 + 1) * st.planets[l.planet].factories / 20;
            let pl = &mut st.planets[l.planet];
            pl.factories = pl.factories.min(f);
            finished = true;
            exploded = true;
        }
        let human = owner.is_some_and(|o| !st.empires[o as usize].ai);
        let stage = if finished {
            Some(if human { 2 } else { 5 } + u8::from(exploded))
        } else if l.stage == 1 {
            Some(if human { 0 } else { 4 })
        } else {
            (human && (st.rng.below(10) == 0 || l.years < 3)).then_some(1)
        };
        if let Some(s) = stage {
            news(st, vec![], "nova", s, Some(l.planet), owner, l.years, l.val);
        }
        if let Some(o) = owner.filter(|&o| st.empires[o as usize].ai) {
            let _ = o;
            let pl = &mut st.planets[l.planet];
            pl.sliders = [0, 0, 0, 0, 100];
        }
        if !finished {
            l.stage = 2;
            st.evn.nova = Some(l);
        }
    }
    if let Some(mut l) = st.evn.accident.take() {
        let human = !st.empires[l.player as usize].ai;
        if l.stage == 1 {
            let pl = &mut st.planets[l.planet];
            pl.max_pop /= 2;
            pl.base_max_pop = pl.base_max_pop.min(pl.max_pop);
            pl.waste = pl.max_pop - 10;
            pl.kind = PlanetType::Radiated;
            pl.growth = Growth::Hostile;
            news(
                st,
                vec![],
                "accident",
                if human { 0 } else { 4 },
                Some(l.planet),
                Some(l.player),
                0,
                0,
            );
            l.stage = 2;
            st.evn.accident = Some(l);
        } else if st.planets[l.planet].waste <= 0 {
            news(
                st,
                vec![],
                "accident",
                if human { 1 } else { 5 },
                Some(l.planet),
                Some(l.player),
                0,
                0,
            );
        } else {
            st.evn.accident = Some(l);
        }
    }
    if let Some(mut l) = st.evn.comet.take() {
        let owner = st.planets[l.planet].owner;
        const HULL_W: [u16; 4] = [1, 5, 25, 125];
        let mut dmg = 0;
        if let Some(o) = owner {
            let designs = st.empires[o as usize].designs.clone();
            for (d, sd) in designs.iter().enumerate() {
                let n = st.empires[o as usize].orbit[l.planet][d] as i32;
                if n > 0 {
                    dmg += st.rng.one_to(HULL_W[sd.hull.min(3) as usize]) as i32 * n;
                }
            }
        }
        let mut finished = false;
        let mut hit = false;
        if l.stage != 1 && owner.is_some() {
            dmg += l.damage;
            if dmg > l.val {
                l.damage = l.val;
                finished = true;
            } else {
                l.damage = dmg;
            }
            l.years -= 1;
        }
        if l.years <= 0 && !finished {
            destroy_planet(st, l.planet);
            let pl = &mut st.planets[l.planet];
            pl.kind = PlanetType::NotHabitable;
            pl.growth = Growth::Hostile;
            finished = true;
            hit = true;
        }
        let human = owner.is_some_and(|o| !st.empires[o as usize].ai);
        let stage = if finished {
            Some(if human { 2 } else { 5 } + u8::from(hit))
        } else if l.stage == 1 {
            Some(if human { 0 } else { 4 })
        } else {
            (human && (st.rng.below(10) == 0 || l.years < 3)).then_some(1)
        };
        if let Some(s) = stage {
            let pct = l.damage * 100 / l.val.max(1);
            news(st, vec![], "comet", s, Some(l.planet), owner, l.years, pct);
        }
        if let Some(o) = owner.filter(|&o| st.empires[o as usize].ai) {
            let _ = o;
            st.planets[l.planet].sliders = [100, 0, 0, 0, 0];
        }
        if !finished {
            l.stage = 2;
            st.evn.comet = Some(l);
        }
    }
    if let Some(mut l) = st.evn.pirates.take() {
        const HULL_W: [u16; 4] = [1, 5, 25, 125];
        let mut dmg = 0;
        for o in 0..st.empires.len() {
            let designs = st.empires[o].designs.clone();
            for (d, sd) in designs.iter().enumerate() {
                let n = st.empires[o].orbit[l.planet][d] as i32;
                if n > 0 {
                    dmg += st.rng.one_to(HULL_W[sd.hull.min(3) as usize]) as i32 * n;
                }
            }
            dmg = dmg.min(32000);
        }
        l.val = (l.val - dmg).max(0);
        let setback = l.val / 6;
        let finished = l.val == 0;
        let n = st.empires.len();
        for a in 0..n {
            if st.range_to(a as u8, l.planet) > st.empires[a].abilities().fuel_range {
                continue;
            }
            for b in 0..n {
                if a == b {
                    continue;
                }
                for (x, y) in [(a, b), (b, a)] {
                    let r = &mut st.empires[x].rel[y];
                    r.trade_percent = r.trade_percent * (100 - setback) / 100;
                }
            }
        }
        let stage = if finished {
            Some(2)
        } else if l.stage == 1 {
            Some(0)
        } else {
            (st.rng.below(10) == 0).then_some(1)
        };
        if let Some(s) = stage {
            news(st, vec![], "pirates", s, Some(l.planet), None, setback, 0);
        }
        if !finished {
            l.stage = 2;
            st.evn.pirates = Some(l);
        }
    }
    // Rebellions (1oom `game_event_run` unrest reports).
    for pli in 0..st.planets.len() {
        let p = &st.planets[pli];
        let Some(o) = p.owner else { continue };
        if p.unrest_reported {
            continue;
        }
        let ai = st.empires[o as usize].ai;
        match p.unrest {
            Unrest::Rebellion => {
                let many = st
                    .colonies(o)
                    .filter(|(_, q)| q.unrest == Unrest::Rebellion)
                    .count()
                    > 1;
                let s = u8::from(many) + if ai { 3 } else { 0 };
                let rebels = p.rebels;
                st.planets[pli].unrest_reported = true;
                news(st, vec![], "rebellion", s, Some(pli), Some(o), rebels, 0);
            }
            Unrest::Resolved => {
                let pl = &mut st.planets[pli];
                pl.unrest = Unrest::Normal;
                pl.unrest_reported = true;
                news(
                    st,
                    vec![],
                    "rebellion",
                    2 + if ai { 3 } else { 0 },
                    Some(pli),
                    Some(o),
                    0,
                    0,
                );
            }
            Unrest::Normal => {}
        }
    }
    // Monsters.
    let mut monsters = std::mem::take(&mut st.evn.monsters);
    monsters.retain_mut(|m| {
        let kname = match m.kind {
            MonsterKind::Crystal => "crystal",
            MonsterKind::Amoeba => "amoeba",
        };
        if let Some(k) = m.killer {
            news(st, vec![], kname, 1, Some(m.dest), Some(k), 0, 0);
            return false;
        }
        if m.nuked > 4 {
            news(st, vec![], kname, 2, None, None, 0, 0);
            return false;
        }
        let p = &st.planets[m.dest];
        let at = (m.x - p.x as f64).abs() < 0.5 && (m.y - p.y as f64).abs() < 0.5;
        if at && m.counter <= 0 {
            let owner = p.owner;
            let colony = p.pop > 0 && owner.is_some();
            let dest = m.dest;
            if m.kind == MonsterKind::Amoeba {
                let pl = &mut st.planets[dest];
                pl.factories /= 10;
                pl.kind = PlanetType::Radiated;
                let mp = st.rng.one_to(3) as i32 * 5 + 5;
                let pl = &mut st.planets[dest];
                pl.base_max_pop = mp;
                pl.max_pop = mp;
                destroy_planet(st, dest);
            } else {
                let pl = &mut st.planets[dest];
                pl.pop = 0;
                pl.waste = pl.max_pop - 10;
                destroy_planet(st, dest);
            }
            m.nuked += 1;
            monster_next(st, m);
            m.counter = 4;
            if colony {
                news(st, vec![], kname, 3, Some(dest), owner, 0, 0);
            }
        } else {
            m.counter -= 1;
        }
        true
    });
    st.evn.monsters = monsters;
    stars_report(st);
}

/// 1oom: "the largest empire controls a quarter/half... of the galaxy".
fn stars_report(st: &mut State) {
    let counts = planets_of(st);
    let (top, max) = counts
        .iter()
        .enumerate()
        .max_by_key(|&(i, &c)| (c, std::cmp::Reverse(i)))
        .map(|(i, &c)| (i as u8, c))
        .unwrap_or((0, 0));
    let half = st.planets.len() as i32 / 2;
    let rs = st.evn.report_stars as i32;
    if (((rs + 1) * half) / 4 <= max
        || (rs == 3 && half - 2 <= max && max < half && st.winner.is_none()))
        && rs < 4
    {
        st.evn.report_stars += 1;
        news(st, vec![], "stars", rs as u8, None, Some(top), max, 0);
    }
}

/// Removes a colony without changing the planet's owner bookkeeping elsewhere.
fn destroy_planet(st: &mut State, planet: PlanetId) {
    let p = &mut st.planets[planet];
    p.pop = 0;
    if let Some(owner) = p.owner.take() {
        p.factories = 0;
        p.bases = 0;
        p.shield = 0;
        p.stargate = false;
        p.rebels = 0;
        p.unrest = Unrest::Normal;
        st.push_event(vec![], EventKind::ColonyDestroyed { planet, owner });
    }
}

/// 1oom `game_turn_coup`: half the planets in rebellion topple the government.
pub fn coup(st: &mut State) {
    let n = st.empires.len();
    let mut total = vec![0; n];
    let mut rebel = vec![0; n];
    for p in &st.planets {
        if let Some(o) = p.owner {
            total[o as usize] += 1;
            if p.unrest == Unrest::Rebellion {
                rebel[o as usize] += 1;
            }
        }
    }
    for i in 0..n {
        if !st.empires[i].alive || total[i] == 0 || rebel[i] < (total[i] + 1) / 2 {
            continue;
        }
        st.empires[i].trait2 = st.rng.below(6) as u8;
        st.empires[i].trait1 = st.rng.below(6) as u8;
        for j in 0..n {
            let r = st.empires[i].rel[j].base_relation;
            st.empires[i].rel[j].relation = r;
            if i != j {
                st.empires[j].rel[i].relation = r;
                if !st.empires[j].ai {
                    diplo::break_treaty(st, i as u8, j as u8);
                    diplo::break_trade(st, i as u8, j as u8);
                }
            }
        }
        for p in st.planets.iter_mut().filter(|p| p.owner == Some(i as u8)) {
            p.unrest = Unrest::Normal;
            p.rebels = 0;
        }
        let humans = st.empires.iter().filter(|e| !e.ai && e.alive).count();
        if !st.empires[i].ai && humans == 1 {
            st.empires[i].alive = false;
            st.push_event(
                vec![],
                EventKind::News {
                    news: "coup".into(),
                    stage: 2,
                    planet: None,
                    empire: Some(i as u8),
                    other: None,
                    n1: 0,
                    n2: 0,
                },
            );
        } else {
            let (t1, t2) = (st.empires[i].trait1 as i32, st.empires[i].trait2 as i32);
            let s = st.rng.below(2) as u8;
            st.push_event(
                vec![],
                EventKind::News {
                    news: "coup".into(),
                    stage: s,
                    planet: None,
                    empire: Some(i as u8),
                    other: None,
                    n1: t1,
                    n2: t2,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Game, NewGameOptions};

    #[test]
    fn events_happen_in_a_long_game() {
        let mut g = Game::new(&NewGameOptions {
            seed: 5,
            players: 4,
            all_ai: true,
            galaxy_size: GalaxySize::Medium,
            ..Default::default()
        })
        .unwrap();
        let mut seen = 0;
        for _ in 0..250 {
            g.end_turn_auto();
            seen += g
                .state()
                .events
                .iter()
                .filter(|e| matches!(e.kind, EventKind::News { .. }))
                .count();
        }
        assert!(seen > 0);
        assert!(g.state().evn.done.iter().any(|&d| d));
    }

    #[test]
    fn monsters_wander_and_strike() {
        let mut g = Game::new(&NewGameOptions::default()).unwrap();
        let st = g.state_mut();
        let m = monster_start(st, MonsterKind::Amoeba);
        st.evn.monsters.push(m);
        for _ in 0..40 {
            move_monsters(st);
            run(st);
        }
        assert!(
            st.evn.monsters.is_empty()
                || st.evn.monsters[0].nuked > 0
                || st.evn.monsters[0].counter < 2
        );
    }

    #[test]
    fn fleets_fight_monsters() {
        let mut g = Game::new(&NewGameOptions::default()).unwrap();
        let st = g.state_mut();
        let home = st.empires[0].home;
        let (x, y) = (st.planets[home].x as f64, st.planets[home].y as f64);
        let mut m = monster_start(st, MonsterKind::Amoeba);
        m.x = x;
        m.y = y;
        m.dest = home;
        m.hp = 50;
        st.evn.monsters.push(m);
        assert_eq!(monster_at(st, home), Some(0));
        st.empires[0].orbit[home][1] = 40;
        st.planets[home].bases = 20;
        assert!(crate::combat::battles(st));
        g.command(
            0,
            &crate::Command::StartBattle {
                mode: crate::BattleMode::Auto,
            },
        )
        .unwrap();
        let st = g.state_mut();
        assert!(st
            .events
            .iter()
            .any(|e| matches!(&e.kind, EventKind::Battle { sides, .. } if sides.contains(&-3))));
        assert_eq!(
            st.evn.monsters[0].killer,
            Some(0),
            "missile bases kill a wounded amoeba"
        );
    }
}
