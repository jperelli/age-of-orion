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
use crate::umath;

pub const SAVE_VERSION: u32 = 4;

/// 1oom `FLEET_SPEED_STARGATE`: the speed of fleets travelling through stargates.
pub const SPEED_STARGATE: i32 = 35;
/// 1oom `PLAYER_NUM`.
pub const PLAYER_NUM: usize = 6;
/// 1oom `FINISHED_*`: factories, max population, soil/atmosphere, stargate, shield, ship.
pub const FINISHED_NUM: usize = 6;
pub const FINISHED_FACT: usize = 0;
pub const FINISHED_POPMAX: usize = 1;
pub const FINISHED_SOILATMOS: usize = 2;
pub const FINISHED_STARGATE: usize = 3;
pub const FINISHED_SHIELD: usize = 4;
pub const FINISHED_SHIP: usize = 5;

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
    /// Picture variant and animation frame on the classic map (1oom `look`, `frame`).
    #[serde(default)]
    pub look: u8,
    #[serde(default)]
    pub frame: u8,
    /// Battle background 0..4, 0 = in a nebula (1oom `battlebg`).
    #[serde(default)]
    pub battlebg: u8,
    #[serde(default)]
    pub prev_owner: Option<EmpireId>,
    #[serde(default)]
    pub pop_prev: i32,
    /// Production before the empire-wide maintenance share (1oom `total_prod`).
    #[serde(default)]
    pub total_prod: i32,
    #[serde(default)]
    pub slider_lock: [bool; NUM_SLIDERS],
    #[serde(default)]
    pub bc_upgrade_base: i32,
    /// Colonists to send at the start of the next turn (1oom `trans_num`, `trans_dest`).
    #[serde(default)]
    pub trans_num: i32,
    #[serde(default)]
    pub trans_dest: PlanetId,
    /// Per empire: 0 out of fuel range, 1 in range, 2 in range with reserve tanks.
    #[serde(default)]
    pub within_frange: Vec<u8>,
    /// Per empire: inside scanner range.
    #[serde(default)]
    pub within_srange: Vec<bool>,
    /// Per empire: ships here ran out of fuel this turn.
    #[serde(default)]
    pub unrefuel: Vec<bool>,
    /// Projects finished this turn (`FINISHED_*`).
    #[serde(default)]
    pub finished: [bool; FINISHED_NUM],
    /// Turn temporaries (1oom `inbound`, `total_inbound`, `artifact_looter`).
    #[serde(default)]
    pub inbound: Vec<i32>,
    #[serde(default)]
    pub total_inbound: Vec<i32>,
    #[serde(default)]
    pub artifact_looter: Option<EmpireId>,
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
    /// Research sliders the player locked (1oom `slider_lock`, an interface aid).
    #[serde(default)]
    pub slider_lock: [bool; NUM_FIELDS],
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
            slider_lock: [false; NUM_FIELDS],
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
    /// Values derived from the completed techs, updated when 1oom updates them
    /// (`game_update_tech_util`).
    #[serde(default)]
    pub ab: Abilities,
    /// Missile base parts (1oom `base_shield`, `base_comp`, `base_weapon`), updated each turn.
    #[serde(default)]
    pub base_shield: usize,
    #[serde(default)]
    pub base_comp: usize,
    #[serde(default)]
    pub base_weapon: usize,
    /// 1oom `ship_maint_bc`, `bases_maint_bc`, `total_maint_bc`, `percent_prod_total_to_actual`.
    #[serde(default)]
    pub ship_maint: i32,
    #[serde(default)]
    pub bases_maint: i32,
    #[serde(default)]
    pub percent_prod_total_to_actual: i32,
    /// Classic AI countdowns (1oom `ai_p2_countdown`, `ai_p3_countdown`).
    #[serde(default)]
    pub ai_p2_countdown: i32,
    #[serde(default)]
    pub ai_p3_countdown: i32,
    /// Classic AI: the colony ship and bomber designs, -1 none (1oom `shipi_colony`, `shipi_bomber`).
    #[serde(default)]
    pub shipi_colony: i32,
    #[serde(default)]
    pub shipi_bomber: i32,
    /// Per design: year it was designed, ships existing, has reserve fuel tanks.
    #[serde(default)]
    pub design_year: Vec<i32>,
    #[serde(default)]
    pub shipcount: Vec<u32>,
    #[serde(default)]
    pub have_reserve_fuel: Vec<bool>,
    /// What this empire last saw of each planet (1oom `seen`).
    #[serde(default)]
    pub seen: Vec<Seen>,
}

/// 1oom `seen_t`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Seen {
    pub owner: Option<EmpireId>,
    pub pop: i32,
    pub bases: i32,
    pub factories: i32,
}

impl Empire {
    pub fn abilities(&self) -> Abilities {
        self.ab
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fleet {
    pub id: u32,
    pub owner: EmpireId,
    pub x: i32,
    pub y: i32,
    pub dest: PlanetId,
    pub ships: Vec<u32>,
    /// Warp speed, or `SPEED_STARGATE` through stargates (arrives next turn).
    pub speed: i32,
    #[serde(default)]
    pub retreat: bool,
    /// Per empire: this fleet is visible to it.
    #[serde(default)]
    pub visible: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transport {
    pub id: u32,
    pub owner: EmpireId,
    pub x: i32,
    pub y: i32,
    pub dest: PlanetId,
    pub pop: i32,
    pub speed: i32,
    #[serde(default)]
    pub visible: Vec<bool>,
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
    /// A human's scouts or scanners explored a star system (1oom `ui_explore`).
    Explored {
        planet: PlanetId,
        by_scanner: bool,
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
        /// Galaxy statistics newscast: empires from first to last (1oom `game_news_get_msg`).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        ranking: Vec<EmpireId>,
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
        /// Troops at the start (transports landed / sent for the attacker).
        #[serde(default)]
        attackers: i32,
        #[serde(default)]
        defenders: i32,
        #[serde(default)]
        landed_of: i32,
        /// Invaders against rebels on one's own planet.
        #[serde(default)]
        rebels: bool,
        /// Who fell, in order (0 an attacker, 1 a defender), for the ground battle screen
        /// (only when a human player takes part).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        deaths: Vec<u8>,
        /// Battle suit, personal shield and hand weapon techs of each side (0: none).
        #[serde(default)]
        gear: [Vec<(u8, u8)>; 2],
        /// Armor tier of each side (the armor part is `ARMOR[tier * 2]`).
        #[serde(default)]
        armor: [u8; 2],
        /// Techs found in the captured factories.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        captured: Vec<(u8, u8)>,
        #[serde(default)]
        factories: i32,
    },
    PlanetCaptured {
        planet: PlanetId,
        from: EmpireId,
        by: EmpireId,
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
    /// A turn interrupted by a human battle or question.
    #[serde(default)]
    pub progress: Option<Progress>,
    /// The question the turn waits for, and its answer once given.
    #[serde(default)]
    pub question: Option<Question>,
    /// What each empire had and did, year by year (the stats screen).
    #[serde(default)]
    pub stats: crate::stats::Stats,
    #[serde(default)]
    pub answer: Option<bool>,
    /// 1oom `ai_id`: 0 classic (as MOO1), 1 classic+ (1oom's improved AI).
    #[serde(default)]
    pub ai_id: u8,
    /// 1oom `galaxy_seed`, `galaxy_w`, `galaxy_h`.
    #[serde(default)]
    pub galaxy_seed: u32,
    #[serde(default)]
    pub galaxy_w: i32,
    #[serde(default)]
    pub galaxy_h: i32,
    /// 1oom `evn.ceasefire[human][ai]` lives in `Relation::ceasefire`; this is the year of
    /// the next MOO1 copy protection roll (1oom `copyprot_status`, never acted upon here).
    #[serde(default)]
    pub copyprot_status: i32,
    /// Who killed the Guardian this turn (1oom `guardian_killer`): loot and news follow.
    #[serde(default)]
    pub guardian_killer: Option<EmpireId>,
    /// Parsecs between planets (1oom `gaux->star_dist`), derived from the positions.
    #[serde(skip)]
    pub star_dist: Vec<Vec<u8>>,
}

/// Where an interrupted turn resumes (after the battles of `battle_planet`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Progress {
    /// Contacts at the start of the turn (1oom `old_contact`).
    pub old_contact: Vec<Vec<bool>>,
    pub battle_planet: usize,
    /// Parties left out of the battles at `battle_planet` (treaties, ceasefires).
    pub skipped: Vec<i32>,
    /// Step of the turn: 0 battles, 1 exploration, 2 bombing.
    pub stage: u8,
    /// Planet and empire where the exploration or the bombing goes on after a question.
    pub at: (usize, usize),
}

/// A question 1oom asks a human player in the middle of the turn; the turn waits for the
/// answer (`Command::Answer`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Question {
    /// 1oom `ui_explore` with a colony ship in orbit: build a colony there?
    Colonize {
        empire: EmpireId,
        planet: PlanetId,
        /// The design that settles it.
        ship: usize,
        /// The planet was not explored before.
        first: bool,
    },
    /// 1oom `ui_bomb_ask`: bomb the enemy colony held by our ships?
    Bomb {
        empire: EmpireId,
        planet: PlanetId,
        /// What the bombs would destroy (1oom computes it before asking).
        pop: i32,
        factories: i32,
        bio: i32,
        /// Troops on their way to the planet.
        inbound: i32,
    },
}

impl Question {
    pub fn empire(&self) -> EmpireId {
        match self {
            Question::Colonize { empire, .. } | Question::Bomb { empire, .. } => *empire,
        }
    }
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
    /// Parsecs between two planets (1oom `gaux->star_dist`, `util_math_dist_steps`).
    pub fn parsecs(&self, a: PlanetId, b: PlanetId) -> i32 {
        match self.star_dist.get(a).and_then(|r| r.get(b)) {
            Some(&d) => d as i32,
            None => {
                let (p, q) = (&self.planets[a], &self.planets[b]);
                umath::dist_steps(p.x, p.y, q.x, q.y) as u8 as i32
            }
        }
    }

    /// 1oom `init_star_dist`: fills the parsec table; call after planets move or on load.
    pub fn init_star_dist(&mut self) {
        let n = self.planets.len();
        let mut t = vec![vec![0u8; n]; n];
        for i in 0..n {
            for j in i + 1..n {
                let (p, q) = (&self.planets[i], &self.planets[j]);
                let d = umath::dist_steps(p.x, p.y, q.x, q.y) as u8;
                t[i][j] = d;
                t[j][i] = d;
            }
        }
        self.star_dist = t;
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
    pub fn in_nebula(&self, x: i32, y: i32) -> bool {
        self.nebulae.iter().any(|n| {
            n.rects
                .iter()
                .any(|r| x >= r[0] && x <= r[2] && y >= r[1] && y <= r[3])
        })
    }

    /// 1oom `game_calc_eta_ship`: turns for ships at `speed` from (x0, y0) to `dest`.
    pub fn eta(&self, x0: i32, y0: i32, dest: PlanetId, speed: i32) -> u32 {
        let (x1, y1) = (self.planets[dest].x, self.planets[dest].y);
        self.eta_ship(speed, x0, y0, x1, y1) as u32
    }

    pub fn eta_ship(&self, speed: i32, x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
        let (mut x, mut y, mut num) = (x0, y0, 0);
        while x != x1 || y != y1 {
            let mut v = speed;
            let mut first = true;
            loop {
                // Inside nebulae, ships move on frame 0 and then on every odd frame.
                if first || !self.in_nebula(x, y) {
                    umath::go_line_dist(&mut x, &mut y, x1, y1, 5);
                }
                umath::go_line_dist(&mut x, &mut y, x1, y1, 6);
                first = false;
                v -= 1;
                if v <= 0 {
                    break;
                }
            }
            num += 1;
        }
        num
    }

    /// 1oom `game_calc_eta_trans`.
    pub fn eta_trans(&self, speed: i32, x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
        let (mut x, mut y, mut num) = (x0, y0, 0);
        while x != x1 || y != y1 {
            let mut v = speed;
            loop {
                umath::go_line_dist(&mut x, &mut y, x1, y1, 5);
                if !self.in_nebula(x, y) {
                    umath::go_line_dist(&mut x, &mut y, x1, y1, 6);
                }
                v -= 1;
                if v <= 0 {
                    break;
                }
            }
            num += 1;
        }
        num
    }

    pub fn colonies(&self, empire: EmpireId) -> impl Iterator<Item = (PlanetId, &Planet)> {
        self.planets
            .iter()
            .enumerate()
            .filter(move |(_, p)| p.owner == Some(empire))
    }

    pub fn push_event(&mut self, to: Vec<EmpireId>, kind: EventKind) {
        crate::stats::count(self, &to, &kind);
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

    /// 1oom `game_get_base_cost` (with the stored base parts `base_weapon` etc.).
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
        let (shield, comp) = (e.base_shield, e.base_comp);
        let jammer = (0..JAMMERS.len())
            .filter(|&i| t.has(FIELD_COMPUTER, JAMMERS[i].tech))
            .max()
            .unwrap_or(0);
        let mut cost = (ARMOR[armor].cost[l] as u32 + HULLS[l].cost as u32)
            * reduce(lv[FIELD_CONSTRUCTION])
            / 1500;
        cost +=
            WEAPONS[e.base_weapon].cost as u32 * ((reduce(lv[FIELD_WEAPON]) * 9) & 0xffff) / 1000;
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
