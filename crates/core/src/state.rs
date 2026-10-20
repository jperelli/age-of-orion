//! The complete game state. It serializes to JSON, which is also the save format.

use serde::{Deserialize, Serialize};

use crate::audience::{Deal, Incoming};
use crate::council::Council;
use crate::data::*;
use crate::data_types::REDUCE_50;
use crate::design::{ShipDesign, HULL_LARGE};
use crate::diplo::{Relation, Treaty};
use crate::events::EventState;
use crate::rng::Rng;
use crate::rules::{Abilities, Rules};
use crate::spy::{SabotageKind, SpyChance};
use crate::tech::{Source, TIERS};
use crate::types::*;

pub const SAVE_VERSION: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unrest {
    #[default]
    Normal,
    Rebellion,
    /// The rebellion was put down this turn.
    Resolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Victory {
    Conquest,
    Council,
}

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
    /// Size of the planet before soil enrichment (1oom `max_pop1`).
    pub size: i32,
    /// Natural maximum population (1oom `max_pop2`).
    pub base_max_pop: i32,
    /// Current maximum population including terraforming (1oom `max_pop3`).
    pub max_pop: i32,
    /// Inside a nebula: no shields, slower travel.
    pub nebula: bool,
    pub owner: Option<EmpireId>,
    pub pop: i32,
    pub pop_tenths: i32,
    pub factories: i32,
    pub waste: i32,
    pub bases: i32,
    pub shield: i32,
    pub stargate: bool,
    /// Colonists per factory this colony has been refitted for.
    pub pop_oper_fact: i32,
    /// Spending in percent: ship, defense, industry, ecology, technology.
    pub sliders: [i32; NUM_SLIDERS],
    /// Design index built with the ship slider, or `design::BUILD_STARGATE`.
    pub build: usize,
    /// Where newly built ships go (1oom `reloc`); `None` = stay in orbit.
    pub rally: Option<PlanetId>,
    /// Production after maintenance, last turn.
    pub prod: i32,
    pub bc_to_ship: i32,
    pub bc_to_factory: i32,
    pub bc_to_base: i32,
    pub bc_to_shield: i32,
    pub bc_to_refit: i32,
    pub bc_to_ecoproj: i32,
    /// BC sent from the empire's reserve, added to production (1oom `reserve`).
    #[serde(default)]
    pub reserve: i32,
    #[serde(default)]
    pub rebels: i32,
    #[serde(default)]
    pub unrest: Unrest,
    #[serde(default)]
    pub unrest_reported: bool,
    /// Empire with ships here or owning it, for diplomacy after battles (1oom `claim`).
    #[serde(default)]
    pub claim: Option<EmpireId>,
    /// Asteroids in battles here: 0 none, 1 some, 2 many (1oom `rocks`).
    #[serde(default)]
    pub rocks: u8,
}

impl Planet {
    pub fn habitable(&self) -> bool {
        self.kind != PlanetType::NotHabitable
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tech {
    /// Completed tech numbers per field, ascending (1oom `researchcompleted`).
    pub completed: Vec<Vec<u8>>,
    /// Tech being researched per field, 0 = none.
    pub project: [u8; NUM_FIELDS],
    /// Cost of the project in research points.
    pub cost: [u32; NUM_FIELDS],
    pub invested: [u32; NUM_FIELDS],
    /// Share of research per field in percent.
    pub alloc: [u32; NUM_FIELDS],
    /// Tech level per field (1oom `tech.percent`).
    pub level: [u32; NUM_FIELDS],
    /// Techs this empire can research: per field, tiers of up to 3 (1oom `researchlist`).
    pub list: Vec<[[u8; 3]; TIERS]>,
}

impl Default for Tech {
    fn default() -> Self {
        Self::new()
    }
}

impl Tech {
    /// Every field starts with tech 1 known (1oom `game_generate_research`).
    pub fn new() -> Self {
        Tech {
            completed: vec![vec![1]; NUM_FIELDS],
            project: [0; NUM_FIELDS],
            cost: [0; NUM_FIELDS],
            invested: [0; NUM_FIELDS],
            alloc: [17, 17, 16, 17, 17, 16],
            level: [1; NUM_FIELDS],
            list: vec![[[0; 3]; TIERS]; NUM_FIELDS],
        }
    }

    pub fn has(&self, field: usize, n: u8) -> bool {
        self.completed[field].binary_search(&n).is_ok()
    }

    pub fn add(&mut self, field: usize, n: u8) {
        if let Err(i) = self.completed[field].binary_search(&n) {
            self.completed[field].insert(i, n);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Empire {
    pub leader: String,
    pub race: Race,
    pub banner: u8,
    pub ai: bool,
    pub alive: bool,
    pub tech: Tech,
    pub designs: Vec<ShipDesign>,
    /// Bomb enemy planets when holding their orbit (human choice; AIs decide each time).
    pub bombard: bool,
    /// Names offered for new designs (1oom `game_str_tbl_ship_names`, 3 per hull).
    pub ship_names: Vec<String>,
    pub reserve: i32,
    pub total_prod: i32,
    pub research: i32,
    pub maintenance: i32,
    /// Ships in orbit, per planet and design.
    pub orbit: Vec<Vec<u32>>,
    pub explored: Vec<bool>,
    pub home: PlanetId,
    /// Relations with every empire (index = other empire).
    #[serde(default)]
    pub rel: Vec<Relation>,
    /// Personality and objective (`diplo::XENOPHOBIC`.. and `diplo::DIPLOMAT`..).
    #[serde(default)]
    pub trait1: u8,
    #[serde(default)]
    pub trait2: u8,
    /// Spending on internal security and taxes, tenths of a percent of production.
    #[serde(default)]
    pub security: i32,
    #[serde(default)]
    pub tax: i32,
    #[serde(default)]
    pub total_trade: i32,
    #[serde(default)]
    pub spying_maint: i32,
}

impl Empire {
    pub fn abilities(&self) -> Abilities {
        Abilities::new(&self.tech, self.race)
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
    /// Travels through stargates: arrives next turn.
    #[serde(default)]
    pub gate: bool,
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

/// A nebula: up to four rectangles `[x0, y0, x1, y1]` (1oom `tbl_nebula_data`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Nebula {
    pub kind: u8,
    pub x: i32,
    pub y: i32,
    pub rects: Vec<[i32; 4]>,
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
    StargateBuilt {
        planet: PlanetId,
    },
    FactoriesComplete {
        planet: PlanetId,
    },
    EnvironmentImproved {
        planet: PlanetId,
    },
    TechDiscovered {
        field: TechField,
        tech: u8,
        source: Source,
        #[serde(default)]
        from: Option<EmpireId>,
    },
    TechStolen {
        field: TechField,
        tech: u8,
        by: Option<EmpireId>,
    },
    SpiesCaught {
        spy: EmpireId,
        target: EmpireId,
        count: i32,
    },
    Sabotaged {
        planet: PlanetId,
        kind: SabotageKind,
        count: i32,
        by: Option<EmpireId>,
    },
    /// A random event (`news.<news>.<stage>`).
    News {
        news: String,
        stage: u8,
        planet: Option<PlanetId>,
        empire: Option<EmpireId>,
        other: Option<EmpireId>,
        n1: i32,
        n2: i32,
    },
    Council {
        candidates: Vec<EmpireId>,
    },
    CouncilResult {
        candidates: Vec<EmpireId>,
        votes: Vec<i32>,
        total: i32,
        elected: Option<EmpireId>,
    },
    FinalWar {
        against: EmpireId,
    },
    RebelsDefeated {
        planet: PlanetId,
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
    pub nebulae: Vec<Nebula>,
    pub empires: Vec<Empire>,
    pub fleets: Vec<Fleet>,
    pub transports: Vec<Transport>,
    pub orion: PlanetId,
    /// The Guardian of Orion is alive.
    pub guardian: bool,
    pub winner: Option<EmpireId>,
    pub next_id: u32,
    /// Events of the last processed turn.
    pub events: Vec<Event>,
    pub rules: Rules,
    #[serde(default)]
    pub victory: Option<Victory>,
    #[serde(default)]
    pub council: Council,
    /// Random events and monsters.
    #[serde(default)]
    pub evn: EventState,
    /// Decisions waiting for human players.
    #[serde(default)]
    pub pending: Pending,
    /// The battle a human is fighting; the turn waits for it.
    #[serde(default)]
    pub battle: Option<Box<crate::battle::Battle>>,
    /// A turn interrupted by a human battle.
    #[serde(default)]
    pub progress: Option<Progress>,
}

/// Where an interrupted turn resumes (after the battles of `battle_planet`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Progress {
    pub arrived: Vec<Transport>,
    pub battle_planet: usize,
    /// Parties left out of the battles at `battle_planet` (treaties, ceasefires).
    pub skipped: Vec<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pending {
    pub messages: Vec<Incoming>,
    pub deals: Vec<Deal>,
    pub spy: Vec<(EmpireId, SpyChance)>,
}

/// 1oom `get_tech_reduce_50` (by tech level 1..).
fn reduce(level: u32) -> u32 {
    REDUCE_50[(level.clamp(1, 50) - 1) as usize] as u32
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

    /// Parsecs from `planet` to the nearest colony of `empire` or of its allies
    /// (1oom `game_update_within_range`).
    pub fn range_to(&self, empire: EmpireId, planet: PlanetId) -> i32 {
        let e = &self.empires[empire as usize];
        self.planets
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.owner.is_some_and(|o| {
                    o == empire
                        || e.rel
                            .get(o as usize)
                            .is_some_and(|r| r.treaty == Treaty::Alliance)
                })
            })
            .map(|(i, _)| self.parsecs(i, planet))
            .min()
            .unwrap_or(i32::MAX)
    }

    /// 1oom `game_xy_is_in_nebula`.
    pub fn in_nebula(&self, x: f64, y: f64) -> bool {
        self.nebulae.iter().any(|n| {
            n.rects.iter().any(|r| {
                x >= r[0] as f64 && x <= r[2] as f64 && y >= r[1] as f64 && y <= r[3] as f64
            })
        })
    }

    /// Turns to reach `dest`; ships inside nebulae move at half speed (1oom `game_calc_eta_ship`).
    pub fn eta(&self, x: f64, y: f64, dest: PlanetId, speed: i32) -> u32 {
        let (tx, ty) = (self.planets[dest].x as f64, self.planets[dest].y as f64);
        let (mut x, mut y) = (x, y);
        let mut turns = 0;
        while (x - tx).abs() > 1e-9 || (y - ty).abs() > 1e-9 {
            let step = self.move_step(x, y, speed);
            let d = ((tx - x).powi(2) + (ty - y).powi(2)).sqrt();
            if d <= step + 1e-9 {
                x = tx;
                y = ty;
            } else {
                x += (tx - x) * step / d;
                y += (ty - y) * step / d;
            }
            turns += 1;
            if turns > 500 {
                break;
            }
        }
        turns.max(1)
    }

    /// Distance covered in one turn from `(x, y)`.
    pub fn move_step(&self, x: f64, y: f64, speed: i32) -> f64 {
        let step = self.rules.parsec * speed.max(1) as f64;
        if self.in_nebula(x, y) {
            step / 2.0
        } else {
            step
        }
    }

    pub fn colonies(&self, empire: EmpireId) -> impl Iterator<Item = (PlanetId, &Planet)> {
        self.planets
            .iter()
            .enumerate()
            .filter(move |(_, p)| p.owner == Some(empire))
    }

    pub fn push_event(&mut self, to: Vec<EmpireId>, kind: EventKind) {
        self.events.push(Event { to, kind });
    }

    pub fn alloc_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    pub fn ship_cost(&self, empire: EmpireId, design: usize) -> i32 {
        let e = &self.empires[empire as usize];
        e.designs
            .get(design)
            .map_or(1, |d| d.cost(&e.tech.level) as i32)
    }

    /// Speed of a group of ships: the slowest design.
    pub fn fleet_speed(&self, empire: EmpireId, ships: &[u32]) -> i32 {
        let e = &self.empires[empire as usize];
        ships
            .iter()
            .enumerate()
            .filter(|(_, &c)| c > 0)
            .filter_map(|(d, _)| e.designs.get(d).map(|d| d.speed()))
            .min()
            .unwrap_or(1)
    }

    /// 1oom `game_get_base_weapon`: the best two-shot missile, for missile bases.
    pub fn base_weapon(&self, empire: EmpireId) -> usize {
        let t = &self.empires[empire as usize].tech;
        let level = t.level[FIELD_WEAPON];
        let mut r = 2;
        for (i, w) in WEAPONS.iter().enumerate() {
            if w.tech as u32 <= level
                && w.damage_min == w.damage_max
                && w.shots == 2
                && w.miss_type == 0
                && !w.bio
                && w.streams == 1
                && t.has(FIELD_WEAPON, w.tech)
            {
                r = i;
            }
        }
        r
    }

    /// 1oom `game_get_base_weapon_2`: scatter packs, for the second base weapon.
    pub fn base_weapon_2(&self, empire: EmpireId) -> usize {
        let t = &self.empires[empire as usize].tech;
        let mut r = self.base_weapon(empire);
        for (i, w) in WEAPONS.iter().enumerate().take(60) {
            if w.streams > 1 && t.has(FIELD_WEAPON, w.tech) {
                r = i;
            }
        }
        r
    }

    /// Best shield, computer and jammer for bases (1oom `game_get_best_*`).
    pub fn base_parts(&self, empire: EmpireId) -> (usize, usize, usize) {
        let t = &self.empires[empire as usize].tech;
        let best = |techs: &mut dyn Iterator<Item = u8>, field: usize| -> usize {
            let level = t.level[field];
            techs
                .enumerate()
                .filter(|&(_, n)| n as u32 <= level && t.has(field, n))
                .map(|(i, _)| i)
                .filter(|&i| i > 0)
                .max()
                .unwrap_or(0)
        };
        (
            best(&mut SHIELDS.iter().map(|s| s.tech), FIELD_FORCE_FIELD),
            best(&mut COMPS.iter().map(|s| s.tech), FIELD_COMPUTER),
            best(&mut JAMMERS.iter().map(|s| s.tech), FIELD_COMPUTER),
        )
    }

    /// 1oom `game_get_base_cost`.
    pub fn base_cost(&self, empire: EmpireId) -> i32 {
        let e = &self.empires[empire as usize];
        let t = &e.tech;
        let lv = &t.level;
        let l = HULL_LARGE as usize;
        let mut armor = (0..ARMOR.len())
            .filter(|&i| t.has(FIELD_CONSTRUCTION, ARMOR[i].tech))
            .max()
            .unwrap_or(0);
        armor = armor.saturating_sub(1);
        let (shield, comp, _) = self.base_parts(empire);
        let jammer = (0..JAMMERS.len())
            .filter(|&i| t.has(FIELD_COMPUTER, JAMMERS[i].tech))
            .max()
            .unwrap_or(0);
        let mut cost = (ARMOR[armor].cost[l] as u32 + HULLS[l].cost as u32)
            * reduce(lv[FIELD_CONSTRUCTION])
            / 1500;
        cost += WEAPONS[self.base_weapon(empire)].cost as u32 * reduce(lv[FIELD_WEAPON]) * 9 / 1000;
        cost += SHIELDS[shield].cost[l] as u32 * reduce(lv[FIELD_FORCE_FIELD]) / 1000
            + SHIELDS[shield].power[l] as u32 / 10;
        cost += COMPS[comp].cost[l] as u32 * reduce(lv[FIELD_COMPUTER]) / 1000
            + COMPS[comp].power[l] as u32 / 10;
        cost += JAMMERS[jammer].cost[l] as u32 * reduce(lv[FIELD_COMPUTER]) / 1000
            + JAMMERS[jammer].power[l] as u32 / 10;
        let mut cost = (cost * 3 / 5) as i32;
        if e.ai {
            cost = match self.difficulty {
                Difficulty::Easy => cost * 9 / 10,
                Difficulty::Average => cost * 8 / 10,
                Difficulty::Hard => cost * 7 / 10,
                Difficulty::Impossible => cost / 2,
                Difficulty::Simple => cost,
            };
        }
        cost.max(50)
    }
}
