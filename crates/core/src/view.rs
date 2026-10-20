//! What one empire knows about the game: the data the UI renders.

use serde::Serialize;

use crate::rules::Abilities;
use crate::state::*;
use crate::turn::{clean_eco_percent, factory_cost, tech_prod};
use crate::types::*;

#[derive(Debug, Clone, Serialize)]
pub struct Colony {
    pub sliders: [i32; NUM_SLIDERS],
    /// BC per turn going to each slider.
    pub spending: [i32; NUM_SLIDERS],
    pub build: usize,
    pub bc_to_ship: i32,
    pub factory_cost: i32,
    pub max_factories: i32,
    pub research: i32,
    pub waste: i32,
    /// Ecology percent that keeps the colony clean.
    pub clean_eco: i32,
    pub pop_oper_fact: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanetView {
    pub id: PlanetId,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub star: StarType,
    pub explored: bool,
    /// Parsecs to the nearest own colony.
    pub range: i32,
    pub orion: bool,
    pub guardian: bool,
    pub image: Option<u8>,
    pub kind: Option<PlanetType>,
    pub special: Option<Special>,
    pub growth: Option<Growth>,
    pub max_pop: Option<i32>,
    pub owner: Option<EmpireId>,
    pub pop: Option<i32>,
    pub factories: Option<i32>,
    pub bases: Option<i32>,
    pub shield: Option<i32>,
    pub prod: Option<i32>,
    pub colonizable: bool,
    pub colony: Option<Colony>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FleetView {
    /// `None` for ships in orbit.
    pub id: Option<u32>,
    pub owner: EmpireId,
    /// Planet orbited, for ships in orbit.
    pub at: Option<PlanetId>,
    pub x: f64,
    pub y: f64,
    pub dest: Option<PlanetId>,
    pub eta: Option<u32>,
    pub ships: Vec<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransportView {
    pub id: u32,
    pub owner: EmpireId,
    pub x: f64,
    pub y: f64,
    pub dest: PlanetId,
    pub eta: u32,
    /// Only known for own transports.
    pub pop: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmpireView {
    pub id: EmpireId,
    pub leader: String,
    pub race: Race,
    pub banner: u8,
    pub alive: bool,
    pub ai: bool,
    pub known_colonies: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignView {
    pub key: String,
    pub cost: i32,
    pub attack: i32,
    pub hp: i32,
    pub bombs: i32,
    pub colony: bool,
    pub range: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct TechView {
    pub level: [u32; NUM_FIELDS],
    pub invested: [u32; NUM_FIELDS],
    pub next_cost: [u32; NUM_FIELDS],
    pub alloc: [u32; NUM_FIELDS],
}

#[derive(Debug, Clone, Serialize)]
pub struct View {
    pub turn: u32,
    pub year: u32,
    pub player: EmpireId,
    pub width: i32,
    pub height: i32,
    pub alive: bool,
    pub winner: Option<EmpireId>,
    pub reserve: i32,
    pub total_prod: i32,
    pub maintenance: i32,
    pub research: i32,
    pub tech: TechView,
    pub abilities: Abilities,
    pub base_cost: i32,
    pub planets: Vec<PlanetView>,
    pub empires: Vec<EmpireView>,
    pub fleets: Vec<FleetView>,
    pub transports: Vec<TransportView>,
    pub designs: Vec<DesignView>,
    pub events: Vec<Event>,
}

fn sees(st: &State, me: EmpireId, x: f64, y: f64) -> bool {
    let e = &st.empires[me as usize];
    let range = e.abilities().scanner_range as f64 * st.rules.parsec;
    st.colonies(me).any(|(i, _)| st.dist_xy(x, y, i) <= range)
        || (0..st.planets.len()).any(|p| {
            e.orbit[p].iter().any(|&c| c > 0) && st.dist_xy(x, y, p) <= st.rules.parsec * 2.0
        })
        || st.fleets.iter().any(|f| {
            f.owner == me && ((f.x - x).powi(2) + (f.y - y).powi(2)).sqrt() <= st.rules.parsec * 2.0
        })
}

pub fn view(st: &State, me: EmpireId) -> View {
    let e = &st.empires[me as usize];
    let a = e.abilities();
    let planets = st
        .planets
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let ex = e.explored[i];
            let mine = p.owner == Some(me);
            let colony = mine.then(|| Colony {
                sliders: p.sliders,
                spending: std::array::from_fn(|s| p.sliders[s] * p.prod / 100),
                build: p.build,
                bc_to_ship: p.bc_to_ship,
                factory_cost: factory_cost(p, e),
                max_factories: p.max_pop * a.colonist_oper_factories,
                research: tech_prod(p, e.race),
                waste: p.waste,
                clean_eco: clean_eco_percent(p, e),
                pop_oper_fact: p.pop_oper_fact,
            });
            PlanetView {
                id: i,
                name: p.name.clone(),
                x: p.x,
                y: p.y,
                star: p.star,
                explored: ex,
                range: st.range_to(me, i),
                orion: i == st.orion,
                guardian: i == st.orion && st.guardian_alive(),
                image: ex.then_some(p.image),
                kind: ex.then_some(p.kind),
                special: ex.then_some(p.special),
                growth: ex.then_some(p.growth),
                max_pop: ex.then_some(p.max_pop),
                owner: if ex { p.owner } else { None },
                pop: if ex && p.owner.is_some() {
                    Some(p.pop)
                } else {
                    None
                },
                factories: if ex && p.owner.is_some() {
                    Some(p.factories)
                } else {
                    None
                },
                bases: if ex && p.owner.is_some() {
                    Some(p.bases)
                } else {
                    None
                },
                shield: if mine { Some(p.shield) } else { None },
                prod: mine.then_some(p.prod),
                colonizable: ex && crate::turn::can_colonize(st, me, i),
                colony,
            }
        })
        .collect();

    let mut fleets = Vec::new();
    for (ei, other) in st.empires.iter().enumerate() {
        for (pi, o) in other.orbit.iter().enumerate() {
            if o.iter().all(|&c| c == 0) {
                continue;
            }
            let (x, y) = (st.planets[pi].x as f64, st.planets[pi].y as f64);
            if ei == me as usize || sees(st, me, x, y) {
                fleets.push(FleetView {
                    id: None,
                    owner: ei as u8,
                    at: Some(pi),
                    x,
                    y,
                    dest: None,
                    eta: None,
                    ships: o.clone(),
                });
            }
        }
    }
    for f in &st.fleets {
        let mine = f.owner == me;
        if mine || sees(st, me, f.x, f.y) {
            fleets.push(FleetView {
                id: Some(f.id),
                owner: f.owner,
                at: None,
                x: f.x,
                y: f.y,
                dest: Some(f.dest),
                eta: Some(st.eta(f.x, f.y, f.dest, f.speed)),
                ships: f.ships.clone(),
            });
        }
    }
    let transports = st
        .transports
        .iter()
        .filter(|t| t.owner == me || sees(st, me, t.x, t.y))
        .map(|t| TransportView {
            id: t.id,
            owner: t.owner,
            x: t.x,
            y: t.y,
            dest: t.dest,
            eta: st.eta(t.x, t.y, t.dest, t.speed),
            pop: (t.owner == me).then_some(t.pop),
        })
        .collect();
    let empires = st
        .empires
        .iter()
        .enumerate()
        .map(|(i, o)| EmpireView {
            id: i as u8,
            leader: o.leader.clone(),
            race: o.race,
            banner: o.banner,
            alive: o.alive,
            ai: o.ai,
            known_colonies: st
                .planets
                .iter()
                .enumerate()
                .filter(|(pi, p)| p.owner == Some(i as u8) && e.explored[*pi])
                .count(),
        })
        .collect();
    let designs = st
        .rules
        .designs
        .iter()
        .enumerate()
        .map(|(d, des)| DesignView {
            key: des.key.clone(),
            cost: st.ship_cost(me, d),
            attack: des.attack * a.attack_pct / 100,
            hp: des.hp * a.hp_pct / 100,
            bombs: des.bombs * a.attack_pct / 100,
            colony: des.colony,
            range: a.fuel_range + des.extra_range,
        })
        .collect();
    let next_cost = std::array::from_fn(|f| {
        st.rules
            .research_cost(e.race, f, e.tech.level[f] + 1, e.ai, st.difficulty)
    });
    View {
        turn: st.turn,
        year: st.year,
        player: me,
        width: st.width,
        height: st.height,
        alive: e.alive,
        winner: st.winner,
        reserve: e.reserve,
        total_prod: e.total_prod,
        maintenance: e.maintenance,
        research: st.colonies(me).map(|(_, p)| tech_prod(p, e.race)).sum(),
        tech: TechView {
            level: e.tech.level,
            invested: e.tech.invested,
            next_cost,
            alloc: e.tech.alloc,
        },
        abilities: a,
        base_cost: st.base_cost(me),
        planets,
        empires,
        fleets,
        transports,
        designs,
        events: st
            .events
            .iter()
            .filter(|ev| ev.to.is_empty() || ev.to.contains(&me))
            .cloned()
            .collect(),
    }
}
