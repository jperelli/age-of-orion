//! The complete game state. It serializes to JSON, which is also the save format.

use serde::{Deserialize, Serialize};

use crate::rng::Rng;
use crate::rules::{Abilities, Rules};
use crate::types::*;

pub const SAVE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Planet {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub star: StarType,
    pub kind: PlanetType,
    pub special: Special,
    pub growth: Growth,
    /// Picture index (1oom `infogfx`; PLANETS.LBX entry in the classic theme).
    pub image: u8,
    /// Natural maximum population (1oom `max_pop2`).
    pub base_max_pop: i32,
    /// Current maximum population including terraforming (1oom `max_pop3`).
    pub max_pop: i32,
    pub owner: Option<EmpireId>,
    pub pop: i32,
    pub pop_tenths: i32,
    pub factories: i32,
    pub waste: i32,
    pub bases: i32,
    pub shield: i32,
    /// Colonists per factory this colony has been refitted for.
    pub pop_oper_fact: i32,
    /// Spending in percent: ship, defense, industry, ecology, technology.
    pub sliders: [i32; NUM_SLIDERS],
    /// Design index built with the ship slider.
    pub build: usize,
    /// Production after maintenance, last turn.
    pub prod: i32,
    pub bc_to_ship: i32,
    pub bc_to_factory: i32,
    pub bc_to_base: i32,
    pub bc_to_shield: i32,
    pub bc_to_refit: i32,
}

impl Planet {
    pub fn habitable(&self) -> bool {
        self.kind != PlanetType::NotHabitable
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tech {
    pub level: [u32; NUM_FIELDS],
    /// Research points accumulated towards the next level.
    pub invested: [u32; NUM_FIELDS],
    /// Share of research per field in percent.
    pub alloc: [u32; NUM_FIELDS],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Empire {
    pub leader: String,
    pub race: Race,
    pub banner: u8,
    pub ai: bool,
    pub alive: bool,
    pub tech: Tech,
    pub reserve: i32,
    pub total_prod: i32,
    pub research: i32,
    pub maintenance: i32,
    /// Ships in orbit, per planet and design.
    pub orbit: Vec<Vec<u32>>,
    pub explored: Vec<bool>,
    pub home: PlanetId,
}

impl Empire {
    pub fn abilities(&self) -> Abilities {
        Abilities::new(&self.tech.level, self.race)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fleet {
    pub id: u32,
    pub owner: EmpireId,
    pub x: f64,
    pub y: f64,
    pub dest: PlanetId,
    pub ships: Vec<u32>,
    pub speed: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transport {
    pub id: u32,
    pub owner: EmpireId,
    pub x: f64,
    pub y: f64,
    pub dest: PlanetId,
    pub pop: i32,
    pub speed: i32,
}

/// Something that happened during a turn. The UI turns it into text with the key
/// `event.<type>` of the active theme.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    ColonyFounded {
        planet: PlanetId,
    },
    ShipsBuilt {
        planet: PlanetId,
        design: usize,
        count: u32,
    },
    FactoriesComplete {
        planet: PlanetId,
    },
    TechAdvance {
        field: TechField,
        level: u32,
    },
    Battle {
        planet: PlanetId,
        sides: Vec<i32>,
        winner: Option<i32>,
        /// Ships lost per side (same order as `sides`, -1 = guardian).
        losses: Vec<u32>,
        bases_lost: i32,
    },
    Bombed {
        planet: PlanetId,
        attacker: EmpireId,
        pop_killed: i32,
        factories_destroyed: i32,
    },
    TransportsLost {
        planet: PlanetId,
        owner: EmpireId,
        pop: i32,
    },
    GroundBattle {
        planet: PlanetId,
        attacker: EmpireId,
        defender: EmpireId,
        won: bool,
        attackers_left: i32,
        defenders_left: i32,
    },
    PlanetCaptured {
        planet: PlanetId,
        from: EmpireId,
        to: EmpireId,
    },
    ColonyDestroyed {
        planet: PlanetId,
        owner: EmpireId,
    },
    GuardianDefeated {
        planet: PlanetId,
        by: EmpireId,
    },
    OrionColonized {
        planet: PlanetId,
        by: EmpireId,
    },
    EmpireEliminated {
        empire: EmpireId,
    },
    Victory {
        empire: EmpireId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Empires that see this event (empty = everyone).
    pub to: Vec<EmpireId>,
    #[serde(flatten)]
    pub kind: EventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub version: u32,
    pub rng: Rng,
    pub seed: u32,
    pub turn: u32,
    pub year: u32,
    pub difficulty: Difficulty,
    pub galaxy_size: GalaxySize,
    pub width: i32,
    pub height: i32,
    pub planets: Vec<Planet>,
    pub empires: Vec<Empire>,
    pub fleets: Vec<Fleet>,
    pub transports: Vec<Transport>,
    pub orion: PlanetId,
    pub guardian_hp: i32,
    pub winner: Option<EmpireId>,
    pub next_id: u32,
    /// Events of the last processed turn.
    pub events: Vec<Event>,
    pub rules: Rules,
}

impl State {
    pub fn dist(&self, a: PlanetId, b: PlanetId) -> f64 {
        let (p, q) = (&self.planets[a], &self.planets[b]);
        (((p.x - q.x).pow(2) + (p.y - q.y).pow(2)) as f64).sqrt()
    }

    pub fn dist_xy(&self, x: f64, y: f64, b: PlanetId) -> f64 {
        let q = &self.planets[b];
        ((x - q.x as f64).powi(2) + (y - q.y as f64).powi(2)).sqrt()
    }

    /// Distance in parsecs (rounded up), the unit of fuel range and speed.
    pub fn parsecs(&self, a: PlanetId, b: PlanetId) -> i32 {
        (self.dist(a, b) / self.rules.parsec).ceil() as i32
    }

    /// Parsecs from `planet` to the nearest colony of `empire`.
    pub fn range_to(&self, empire: EmpireId, planet: PlanetId) -> i32 {
        self.planets
            .iter()
            .enumerate()
            .filter(|(_, p)| p.owner == Some(empire))
            .map(|(i, _)| self.parsecs(i, planet))
            .min()
            .unwrap_or(i32::MAX)
    }

    pub fn eta(&self, x: f64, y: f64, dest: PlanetId, speed: i32) -> u32 {
        let d = self.dist_xy(x, y, dest);
        (d / (self.rules.parsec * speed.max(1) as f64))
            .ceil()
            .max(1.0) as u32
    }

    pub fn colonies(&self, empire: EmpireId) -> impl Iterator<Item = (PlanetId, &Planet)> {
        self.planets
            .iter()
            .enumerate()
            .filter(move |(_, p)| p.owner == Some(empire))
    }

    pub fn guardian_alive(&self) -> bool {
        self.guardian_hp > 0
    }

    pub fn push_event(&mut self, to: Vec<EmpireId>, kind: EventKind) {
        self.events.push(Event { to, kind });
    }

    pub fn alloc_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    pub fn ship_cost(&self, empire: EmpireId, design: usize) -> i32 {
        let a = self.empires[empire as usize].abilities();
        (self.rules.designs[design].cost * a.cost_pct / 100).max(1)
    }

    pub fn base_cost(&self, empire: EmpireId) -> i32 {
        let e = &self.empires[empire as usize];
        let a = e.abilities();
        let mut c = (self.rules.base_cost * a.cost_pct / 100)
            + 2 * e.tech.level[TechField::Weapon as usize] as i32
            + 2 * e.tech.level[TechField::ForceField as usize] as i32;
        if e.ai {
            c = match self.difficulty {
                Difficulty::Easy => c * 9 / 10,
                Difficulty::Average => c * 8 / 10,
                Difficulty::Hard => c * 7 / 10,
                Difficulty::Impossible => c / 2,
                Difficulty::Simple => c,
            };
        }
        c.max(50)
    }
}
