//! What one empire knows about the game: the data the UI renders.

use serde::Serialize;

use crate::audience::{self, AudienceOptions, Incoming};
use crate::council::Election;
use crate::design::{self, DesignStats, KnownParts, ShipDesign};
use crate::diplo::{self, SpyMode, TechRef, Treaty};
use crate::events::MonsterKind;
use crate::rules::Abilities;
use crate::spy::SpyChance;
use crate::state::*;
use crate::tech;
use crate::turn::{clean_eco_percent, factory_cost, ship_totals, tech_prod};
use crate::types::*;

#[derive(Debug, Clone, Serialize)]
pub struct Colony {
    pub sliders: [i32; NUM_SLIDERS],
    /// BC per turn going to each slider.
    pub spending: [i32; NUM_SLIDERS],
    /// Design index, or 6 for a stargate.
    pub build: usize,
    pub build_cost: i32,
    pub bc_to_ship: i32,
    pub rally: Option<PlanetId>,
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
    pub nebula: bool,
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
    pub stargate: bool,
    pub prod: Option<i32>,
    pub colonizable: bool,
    pub colony: Option<Colony>,
    /// Own planets: rebels and the reserve sent to it.
    pub rebels: Option<i32>,
    pub rebellion: bool,
    pub reserve: Option<i32>,
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
    /// Design names of the owner, for enemy fleets.
    pub names: Vec<String>,
    pub speed: i32,
    pub range: i32,
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

/// This empire's dealings with another one.
#[derive(Debug, Clone, Serialize)]
pub struct RelationView {
    pub empire: EmpireId,
    pub contact: bool,
    pub met: bool,
    pub treaty: Treaty,
    /// How the other (computer) empire feels about us, -100..100.
    pub relation: i32,
    pub trade_bc: i32,
    pub trade_percent: i32,
    pub ceasefire: i32,
    pub spies: i32,
    pub spying: i32,
    pub spy_mode: SpyMode,
    /// Their spies we know of (caught this turn are in the events).
    pub personality: Option<u8>,
    pub objective: Option<u8>,
    /// Tech levels and newest techs, known while we have spies there.
    pub spy_levels: Option<Vec<u32>>,
    pub spy_report: Option<Vec<TechRef>>,
    pub spy_report_year: Option<u32>,
    pub audience: Option<AudienceOptions>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonsterView {
    pub kind: MonsterKind,
    pub x: f64,
    pub y: f64,
    pub dest: PlanetId,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignView {
    #[serde(flatten)]
    pub design: ShipDesign,
    pub stats: DesignStats,
    /// Ships of this design in service.
    pub count: u32,
    pub range: i32,
    pub colony: Option<PlanetType>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TechChoice {
    pub tech: u8,
    pub key: String,
    pub cost: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldView {
    pub field: TechField,
    pub level: u32,
    pub project: u8,
    pub project_key: String,
    pub cost: u32,
    pub invested: u32,
    pub alloc: u32,
    pub choices: Vec<TechChoice>,
    pub completed: Vec<TechChoice>,
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
    pub bombard: bool,
    pub tech: Vec<FieldView>,
    pub abilities: Abilities,
    pub base_cost: i32,
    pub stargate_cost: i32,
    pub planets: Vec<PlanetView>,
    pub nebulae: Vec<Nebula>,
    pub empires: Vec<EmpireView>,
    pub fleets: Vec<FleetView>,
    pub transports: Vec<TransportView>,
    pub designs: Vec<DesignView>,
    pub parts: KnownParts,
    pub events: Vec<Event>,
    pub security: i32,
    pub tax: i32,
    pub total_trade: i32,
    pub spying_maint: i32,
    pub relations: Vec<RelationView>,
    /// Messages from computer empires, with their index for `AnswerMessage`.
    pub messages: Vec<(usize, Incoming)>,
    /// Spy successes waiting for a choice, with their index.
    pub spy_chances: Vec<(usize, SpyChance)>,
    pub election: Option<Election>,
    pub final_war: bool,
    pub high_master: Option<EmpireId>,
    pub victory: Option<Victory>,
    pub monsters: Vec<MonsterView>,
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

fn field_view(st: &State, me: EmpireId, f: usize) -> FieldView {
    let t = &st.empires[me as usize].tech;
    let choice = |n: u8| TechChoice {
        tech: n,
        key: tech::key(f, n),
        cost: tech::cost_of(st, me, f, n),
    };
    FieldView {
        field: TechField::ALL[f],
        level: t.level[f],
        project: t.project[f],
        project_key: tech::key(f, t.project[f]),
        cost: t.cost[f],
        invested: t.invested[f],
        alloc: t.alloc[f],
        choices: tech::next_techs(t, f).into_iter().map(choice).collect(),
        completed: t.completed[f].iter().map(|&n| choice(n)).collect(),
    }
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
                build_cost: if p.build == design::BUILD_STARGATE {
                    st.rules.stargate_cost
                } else {
                    st.ship_cost(me, p.build)
                },
                bc_to_ship: p.bc_to_ship,
                rally: p.rally,
                factory_cost: factory_cost(p, e),
                max_factories: p.max_pop * a.colonist_oper_factories,
                research: tech_prod(p, e.race),
                waste: p.waste,
                clean_eco: clean_eco_percent(p, e),
                pop_oper_fact: p.pop_oper_fact,
            });
            let known_owner = ex && p.owner.is_some();
            PlanetView {
                id: i,
                name: p.name.clone(),
                x: p.x,
                y: p.y,
                star: p.star,
                explored: ex,
                nebula: p.nebula,
                range: st.range_to(me, i),
                orion: i == st.orion,
                guardian: i == st.orion && st.guardian,
                image: ex.then_some(p.image),
                kind: ex.then_some(p.kind),
                special: ex.then_some(p.special),
                growth: ex.then_some(p.growth),
                max_pop: ex.then_some(p.max_pop),
                owner: if ex { p.owner } else { None },
                pop: known_owner.then_some(p.pop),
                factories: known_owner.then_some(p.factories),
                bases: known_owner.then_some(p.bases),
                shield: mine.then_some(p.shield),
                stargate: ex && p.stargate,
                prod: mine.then_some(p.prod),
                colonizable: ex && crate::turn::can_colonize(st, me, i),
                colony,
                rebels: mine.then_some(p.rebels),
                rebellion: (mine || known_owner) && p.unrest == Unrest::Rebellion,
                reserve: mine.then_some(p.reserve),
            }
        })
        .collect();

    let names = |o: &Empire| o.designs.iter().map(|d| d.name.clone()).collect::<Vec<_>>();
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
                    names: names(other),
                    speed: st.fleet_speed(ei as u8, o),
                    range: crate::commands::fleet_range(st, ei as u8, o),
                });
            }
        }
    }
    for f in &st.fleets {
        if f.owner == me || sees(st, me, f.x, f.y) {
            fleets.push(FleetView {
                id: Some(f.id),
                owner: f.owner,
                at: None,
                x: f.x,
                y: f.y,
                dest: Some(f.dest),
                eta: Some(if f.gate {
                    1
                } else {
                    st.eta(f.x, f.y, f.dest, f.speed)
                }),
                ships: f.ships.clone(),
                names: names(&st.empires[f.owner as usize]),
                speed: f.speed,
                range: crate::commands::fleet_range(st, f.owner, &f.ships),
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
    let totals = ship_totals(st, me);
    let designs = e
        .designs
        .iter()
        .enumerate()
        .map(|(d, sd)| DesignView {
            design: sd.clone(),
            stats: design::stats(sd, &e.tech),
            count: totals[d],
            range: a.fuel_range + sd.extra_range(),
            colony: sd.colony_type(),
        })
        .collect();
    let relations = (0..st.empires.len() as u8)
        .filter(|&o| o != me)
        .map(|o| {
            let r = diplo::rel(st, me, o);
            let other = &st.empires[o as usize];
            let spied = r.spies > 0;
            RelationView {
                empire: o,
                contact: r.contact,
                met: r.have_met > 0 || r.contact,
                treaty: r.treaty,
                relation: r.relation,
                trade_bc: r.trade_bc,
                trade_percent: r.trade_percent,
                ceasefire: r.ceasefire,
                spies: r.spies,
                spying: r.spying,
                spy_mode: r.spymode,
                personality: r.contact.then_some(other.trait1),
                objective: (r.contact && other.ai).then_some(other.trait2),
                spy_levels: spied.then(|| other.tech.level.to_vec()),
                spy_report: (r.spy_report_year > 0).then(|| {
                    (0..NUM_FIELDS)
                        .map(|f| (f as u8, r.spy_report[f]))
                        .collect()
                }),
                spy_report_year: (r.spy_report_year > 0).then_some(r.spy_report_year),
                audience: (r.contact && other.ai && other.alive && e.alive)
                    .then(|| audience::options(st, me, o)),
            }
        })
        .collect();
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
        bombard: e.bombard,
        tech: (0..NUM_FIELDS).map(|f| field_view(st, me, f)).collect(),
        abilities: a,
        base_cost: st.base_cost(me),
        stargate_cost: st.rules.stargate_cost,
        planets,
        nebulae: st.nebulae.clone(),
        empires,
        fleets,
        transports,
        designs,
        parts: design::known_parts(&e.tech),
        events: st
            .events
            .iter()
            .filter(|ev| ev.to.is_empty() || ev.to.contains(&me))
            .cloned()
            .collect(),
        security: e.security,
        tax: e.tax,
        total_trade: e.total_trade,
        spying_maint: e.spying_maint,
        relations,
        messages: st
            .pending
            .messages
            .iter()
            .enumerate()
            .filter(|(_, m)| m.to == me)
            .map(|(i, m)| (i, m.clone()))
            .collect(),
        spy_chances: st
            .pending
            .spy
            .iter()
            .enumerate()
            .filter(|(_, (o, _))| *o == me)
            .map(|(i, (_, c))| (i, c.clone()))
            .collect(),
        election: st.council.pending.clone(),
        final_war: st.council.final_war,
        high_master: st.council.winner,
        victory: st.victory,
        monsters: st
            .evn
            .monsters
            .iter()
            .filter(|m| sees(st, me, m.x, m.y))
            .map(|m| MonsterView {
                kind: m.kind,
                x: m.x,
                y: m.y,
                dest: m.dest,
            })
            .collect(),
    }
}
