//! New game generation (port of 1oom `game_new.c`, without nebulae).

use serde::{Deserialize, Serialize};

use crate::design::MAX_DESIGNS;
use crate::rng::Rng;
use crate::rules::Rules;
use crate::state::*;
use crate::types::*;

/// Star names of MOO1 (1oom `game_str_tbl_planet_names`). Themes usually pass their own.
pub const STAR_NAMES: [&str; 110] = [
    "Paranar",
    "Denubius",
    "Antares",
    "Draconis",
    "Zoctan",
    "Rigel",
    "Talas",
    "Moro",
    "Quayal",
    "Neptunus",
    "Jinga",
    "Argus",
    "Escalon",
    "Berel",
    "Collassa",
    "Whynil",
    "Nordia",
    "Tau Cygni",
    "Phyco",
    "Firma",
    "Keeta",
    "Arietis",
    "Rhilus",
    "Willow",
    "Mu Delphi",
    "Stalaz",
    "Gorra",
    "Beta Ceti",
    "Spica",
    "Omicron",
    "Rha",
    "Kailis",
    "Vulcan",
    "Centauri",
    "Herculis",
    "Endoria",
    "Kulthos",
    "Hyboria",
    "Zhardan",
    "Yarrow",
    "Incedius",
    "Paladia",
    "Romulas",
    "Galos",
    "Uxmai",
    "Thrax",
    "Laan",
    "Imra",
    "Selia",
    "Seidon",
    "Tao",
    "Rana",
    "Vox",
    "Maalor",
    "Xudax",
    "Helos",
    "Crypto",
    "Gion",
    "Phantos",
    "Reticuli",
    "Maretta",
    "Toranor",
    "Exis",
    "Tyr",
    "Ajax",
    "Obaca",
    "Dolz",
    "Drakka",
    "Ryoun",
    "Vega",
    "Anraq",
    "Gienah",
    "Rotan",
    "Proxima",
    "Mobas",
    "Iranha",
    "Celtsi",
    "Dunatis",
    "Morrig",
    "Primodius",
    "Nyarl",
    "Ukko",
    "Crius",
    "Hyades",
    "Kronos",
    "Guradas",
    "Rayden",
    "Kakata",
    "Misha",
    "Xendalla",
    "Artemis",
    "Aurora",
    "Proteus",
    "Esper",
    "Darrian",
    "Trax",
    "Xengara",
    "Nitzer",
    "Simius",
    "Bootis",
    "Pollus",
    "Cygni",
    "Aquilae",
    "Volantis",
    "Tauri",
    "Regulus",
    "Klystron",
    "Lyae",
    "Capella",
    "Alcor",
];

/// Home world names per race (1oom `game_str_tbl_home_names`).
pub const HOME_NAMES: [&str; 10] = [
    "Sol", "Fierias", "Cryslon", "Sssla", "Mentar", "Altair", "Kholdan", "Ursa", "Meklon", "Nazin",
];

/// Ship names per race, 3 per hull size (1oom `game_str_tbl_ship_names`).
pub const SHIP_NAMES: [[&str; 12]; 10] = [
    [
        "Fighter",
        "Gunboat",
        "Courier",
        "Frigate",
        "Destroyer",
        "Corvette",
        "Cruiser",
        "Escort",
        "Warship",
        "Dreadnought",
        "Battleship",
        "Dreadstar",
    ],
    [
        "Wolverine",
        "Griffin",
        "Ferret",
        "Lynx",
        "Cheetah",
        "Bobcat",
        "Panther",
        "Leopard",
        "Warcat",
        "Sabretooth",
        "Puma",
        "Tiger",
    ],
    [
        "Piranha",
        "Stingray",
        "Minnow",
        "Manta",
        "Barracuda",
        "Morey",
        "Shark",
        "Mako",
        "Whale",
        "Kraken",
        "Monitor",
        "Polaris",
    ],
    [
        "Goblin",
        "Cyclops",
        "Daemon",
        "Banshee",
        "Spectre",
        "Spirit",
        "Juggernaut",
        "Valkyrie",
        "Hydra",
        "Titan",
        "Colossus",
        "Dragon",
    ],
    [
        "Starfighter",
        "Comet",
        "Seeker",
        "Star Wing",
        "Star Blazer",
        "Star Streak",
        "Star Blade",
        "Dark Star",
        "Coraona",
        "Sun Fire",
        "Star Fury",
        "Nova",
    ],
    [
        "Foxbat",
        "Sparrowhawk",
        "Pelican",
        "Falcon",
        "Sky Hawk",
        "Space Gull",
        "Warbird",
        "Warhawk",
        "Wareagle",
        "Condor",
        "Dreadwing",
        "Sky Master",
    ],
    [
        "Sabre",
        "Cutlass",
        "Dagger",
        "Lancer",
        "Pegasus",
        "Horseman",
        "Ranger",
        "Knight",
        "Avenger",
        "Dragoon",
        "Paladin",
        "Excaliber",
    ],
    [
        "Paw",
        "Panda",
        "Patrol",
        "Hunter",
        "Gladiator",
        "Sentinel",
        "Claw",
        "Tooth",
        "Warbear",
        "Grizzly",
        "Punisher",
        "Rhino",
    ],
    [
        "Marauder",
        "Hyperion",
        "Drachma",
        "Intruder",
        "Penetrator",
        "Tormenter",
        "Ajax",
        "Tornado",
        "Nexus",
        "Nemesis",
        "Devastator",
        "Annihilator",
    ],
    [
        "Spider",
        "Bumblebee",
        "Needle",
        "Hornet",
        "Scorpion",
        "Venom",
        "Tarantula",
        "Viper",
        "Cobra",
        "King Cobra",
        "Black Widow",
        "Death Wyn",
    ],
];

/// 1oom `tbl_planet_type_infogfx` (1-based PLANETS.LBX pictures per planet type).
const PLANET_IMAGES: [[u8; 6]; 14] = [
    [0x01, 0x01, 0x01, 0x01, 0x01, 0x01],
    [0x05, 0x05, 0x14, 0x14, 0x22, 0x22],
    [0x04, 0x04, 0x06, 0x06, 0x17, 0x17],
    [0x0b, 0x0b, 0x0d, 0x0d, 0x18, 0x18],
    [0x07, 0x07, 0x08, 0x08, 0x23, 0x23],
    [0x20, 0x20, 0x20, 0x1e, 0x1e, 0x1e],
    [0x02, 0x02, 0x1a, 0x1a, 0x20, 0x20],
    [0x09, 0x09, 0x09, 0x19, 0x19, 0x19],
    [0x11, 0x11, 0x11, 0x1d, 0x1d, 0x1d],
    [0x0f, 0x0f, 0x12, 0x12, 0x1f, 0x1f],
    [0x15, 0x15, 0x15, 0x1c, 0x1c, 0x1c],
    [0x0e, 0x0e, 0x0e, 0x1b, 0x1b, 0x1b],
    [0x03, 0x03, 0x01, 0x10, 0x0c, 0x16],
    [0x01, 0x01, 0x0a, 0x0a, 0x13, 0x21],
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NewGameOptions {
    pub seed: u32,
    pub galaxy_size: GalaxySize,
    pub difficulty: Difficulty,
    /// Number of empires including the human player (2..=6).
    pub players: u8,
    /// Race of the human player (empire 0); random if `None`.
    pub race: Option<Race>,
    pub banner: Option<u8>,
    /// No human player: every empire is run by the AI (simulations, tests).
    pub all_ai: bool,
    /// 1oom AI: 0 classic (as MOO1, the default), 1 classic+.
    #[serde(default)]
    pub ai_id: u8,
    pub star_names: Vec<String>,
    /// Home world name per race (10 entries).
    pub home_names: Vec<String>,
    /// Leader name per race (10 entries).
    pub leader_names: Vec<String>,
    pub leader: Option<String>,
    pub orion_name: Option<String>,
    /// Names of the starting designs (5 entries).
    pub start_design_names: Vec<String>,
    /// Ship names per race (10 lists of 12, 3 per hull size), offered for new designs.
    pub ship_names: Vec<Vec<String>>,
    pub rules: Option<Rules>,
}

impl Default for NewGameOptions {
    fn default() -> Self {
        NewGameOptions {
            seed: 1,
            galaxy_size: GalaxySize::Small,
            difficulty: Difficulty::Easy,
            players: 4,
            race: None,
            banner: None,
            all_ai: false,
            ai_id: 0,
            star_names: Vec::new(),
            home_names: Vec::new(),
            leader_names: Vec::new(),
            leader: None,
            orion_name: None,
            start_design_names: Vec::new(),
            ship_names: Vec::new(),
            rules: None,
        }
    }
}

fn dist_fast(x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
    let (dx, dy) = ((x1 - x0).abs(), (y1 - y0).abs());
    if dx > dy {
        dx + dy / 2
    } else {
        dy + dx / 2
    }
}

fn empty_planet(x: i32, y: i32) -> Planet {
    Planet {
        name: String::new(),
        x,
        y,
        star: StarType::Yellow,
        kind: PlanetType::NotHabitable,
        special: Special::Normal,
        growth: Growth::Normal,
        image: 0,
        size: 10,
        base_max_pop: 10,
        max_pop: 10,
        nebula: false,
        owner: None,
        pop: 0,
        pop_tenths: 0,
        factories: 0,
        waste: 0,
        bases: 0,
        shield: 0,
        stargate: false,
        pop_oper_fact: 2,
        sliders: [0, 0, 100, 0, 0],
        build: 1,
        rally: None,
        prod: 0,
        bc_to_ship: 0,
        bc_to_factory: 0,
        bc_to_base: 0,
        bc_to_shield: 0,
        bc_to_refit: 0,
        bc_to_ecoproj: 0,
        reserve: 0,
        rebels: 0,
        unrest: Unrest::Normal,
        unrest_reported: false,
        claim: None,
        rocks: 0,
        look: 0,
        frame: 0,
        battlebg: 1,
        prev_owner: None,
        pop_prev: 0,
        total_prod: 0,
        slider_lock: [false; NUM_SLIDERS],
        bc_upgrade_base: 0,
        trans_num: 0,
        trans_dest: 0,
        within_frange: vec![0; PLAYER_NUM],
        within_srange: vec![false; PLAYER_NUM],
        unrefuel: vec![false; PLAYER_NUM],
        finished: [false; FINISHED_NUM],
        inbound: vec![0; PLAYER_NUM],
        total_inbound: vec![0; PLAYER_NUM],
        artifact_looter: None,
    }
}

/// Straight-line distance between two planets (home placement only).
fn edist(st: &State, a: PlanetId, b: PlanetId) -> f64 {
    let (p, q) = (&st.planets[a], &st.planets[b]);
    (((p.x - q.x).pow(2) + (p.y - q.y).pow(2)) as f64).sqrt()
}

/// 1oom `tbl_nebula_data`: per nebula type, the x0, y0, x1 and y1 of four rectangles.
const NEBULA_DATA: [[[i32; 4]; 4]; 10] = [
    [
        [0x14, 0x07, 0x0f, 0x1f],
        [0x05, 0x12, 0x0c, 0x02],
        [0x28, 0x1c, 0x1f, 0x24],
        [0x14, 0x27, 0x1b, 0x04],
    ],
    [
        [0x0a, 0x14, 0x1b, 0x1e],
        [0x11, 0x0d, 0x0d, 0x05],
        [0x1a, 0x26, 0x29, 0x29],
        [0x25, 0x18, 0x10, 0x10],
    ],
    [
        [0x09, 0x03, 0x17, 0x1a],
        [0x02, 0x08, 0x12, 0x0f],
        [0x16, 0x19, 0x29, 0x23],
        [0x19, 0x15, 0x20, 0x24],
    ],
    [
        [0x19, 0x08, 0x0a, 0x10],
        [0x0c, 0x0f, 0x09, 0x07],
        [0x1e, 0x1c, 0x16, 0x17],
        [0x1c, 0x1d, 0x0f, 0x09],
    ],
    [
        [0x07, 0x0f, 0x15, 0x19],
        [0x02, 0x0f, 0x16, 0x23],
        [0x19, 0x25, 0x29, 0x29],
        [0x13, 0x1c, 0x23, 0x27],
    ],
    [
        [0x03, 0x08, 0x0e, 0x09],
        [0x0e, 0x15, 0x02, 0x0a],
        [0x23, 0x24, 0x1a, 0x22],
        [0x14, 0x23, 0x1f, 0x0f],
    ],
    [
        [0x0d, 0x09, 0x13, 0x05],
        [0x03, 0x0b, 0x21, 0x16],
        [0x1b, 0x22, 0x1b, 0x08],
        [0x21, 0x20, 0x25, 0x1e],
    ],
    [
        [0x08, 0x0f, 0x1c, 0x25],
        [0x08, 0x18, 0x02, 0x0a],
        [0x25, 0x22, 0x23, 0x27],
        [0x19, 0x1f, 0x07, 0x12],
    ],
    [
        [0x08, 0x05, 0x05, 0x1b],
        [0x06, 0x01, 0x0c, 0x03],
        [0x23, 0x0c, 0x09, 0x25],
        [0x1f, 0x08, 0x15, 0x0a],
    ],
    [
        [0x04, 0x08, 0x0d, 0x25],
        [0x02, 0x12, 0x0e, 0x11],
        [0x2a, 0x26, 0x26, 0x29],
        [0x12, 0x1e, 0x21, 0x1d],
    ],
];

/// Port of the nebula part of 1oom `game_generate_galaxy`.
fn generate_nebulae(rng: &mut Rng, size: GalaxySize, enabled: bool) -> Vec<Nebula> {
    let (w, h) = size.grid();
    let a = match size {
        GalaxySize::Small => rng.below(2),
        GalaxySize::Medium => rng.one_to(3),
        GalaxySize::Large => rng.one_to(2) + 1,
        GalaxySize::Huge => rng.one_to(3) + 1,
    } as usize;
    let mut num = if enabled { a } else { 0 };
    let gw = ((w as i32 - 1) * 0x1c + 0x14 - 70) as u16;
    let gh = ((h as i32 - 1) * 0x20 + 0x10 - 70) as u16;
    let mut out: Vec<(u8, i32, i32)> = Vec::new();
    let mut loops = 0;
    while out.len() < num {
        let kind = rng.below(10) as u8;
        let x = rng.one_to(gw) as i32 + 4;
        let y = rng.one_to(gh) as i32 + 4;
        let clash = out
            .iter()
            .any(|&(k, nx, ny)| k == kind || ((nx - x).abs() < 0x3c && (ny - y).abs() < 0x3c));
        if !clash {
            out.push((kind, x, y));
        }
        loops += 1;
        if loops > 200 {
            num = out.len();
        }
    }
    out.into_iter()
        .map(|(kind, x, y)| {
            let t = &NEBULA_DATA[kind as usize];
            Nebula {
                kind,
                x,
                y,
                rects: (0..4)
                    .map(|j| [t[0][j] + x, t[1][j] + y, t[2][j] + x, t[3][j] + y])
                    .collect(),
            }
        })
        .collect()
}

fn inside(nebulae: &[Nebula], x: i32, y: i32) -> bool {
    nebulae.iter().any(|n| {
        n.rects
            .iter()
            .any(|r| x >= r[0] && x <= r[2] && y >= r[1] && y <= r[3])
    })
}

fn planet_type_for(star: StarType, r: i32) -> PlanetType {
    use PlanetType::*;
    match star {
        StarType::Yellow => match r {
            -1 => Radiated,
            0 => Toxic,
            1 => Inferno,
            2 => Tundra,
            3 => Barren,
            4 => Minimal,
            5 => Desert,
            6 => Steppe,
            7..=8 => Arid,
            9..=10 => Ocean,
            11..=12 => Jungle,
            r if r > 12 => Terran,
            _ => NotHabitable,
        },
        StarType::Red => match r {
            2 => Radiated,
            3 => Toxic,
            4 => Inferno,
            5 => Dead,
            6 => Tundra,
            7 => Barren,
            8 => Minimal,
            9..=10 => Desert,
            11..=12 => Steppe,
            13..=15 => Arid,
            16..=17 => Ocean,
            18..=19 => Jungle,
            20 => Terran,
            _ => NotHabitable,
        },
        StarType::Green => match r {
            2 => Radiated,
            3 => Toxic,
            4 => Inferno,
            5 => Dead,
            6 => Tundra,
            7 => Barren,
            8 => Minimal,
            9 => Desert,
            10..=11 => Steppe,
            12..=13 => Arid,
            14..=15 => Ocean,
            16..=17 => Jungle,
            r if r > 17 => Terran,
            _ => NotHabitable,
        },
        StarType::White => match r {
            3 => Radiated,
            4..=5 => Toxic,
            6..=7 => Inferno,
            8..=9 => Dead,
            10..=11 => Tundra,
            12..=13 => Barren,
            14..=15 => Minimal,
            16..=17 => Desert,
            18 => Steppe,
            19 => Arid,
            20 => Ocean,
            _ => NotHabitable,
        },
        StarType::Blue => match r {
            4..=5 => Radiated,
            6..=7 => Toxic,
            8..=9 => Inferno,
            10..=11 => Dead,
            12..=13 => Tundra,
            14..=15 => Barren,
            16..=17 => Minimal,
            18 => Desert,
            19 => Steppe,
            20 => Arid,
            _ => NotHabitable,
        },
        StarType::Neutron => match r {
            5..=9 => Radiated,
            10..=12 => Toxic,
            13..=15 => Inferno,
            16..=17 => Dead,
            18 => Tundra,
            19 => Barren,
            20 => Minimal,
            _ => NotHabitable,
        },
    }
}

/// Port of 1oom `game_generate_planets` (positions, star colour, type, size, specials).
fn generate_planets(rng: &mut Rng, w: u16, h: u16, nebulae: &[Nebula]) -> Vec<Planet> {
    let mut pos: Vec<(i32, i32)> = Vec::new();
    for gy in 0..h as i32 {
        for gx in 0..w as i32 {
            'again: loop {
                let x = rng.one_to(0x2b) as i32 + gx * 0x1c + 9;
                let y = rng.one_to(0x33) as i32 + gy * 0x20 + 7;
                for &(px, py) in &pos {
                    let d = dist_fast(px, py, x, y);
                    if (((px < x) || (py < y)) && d < 20) || d < 10 {
                        continue 'again;
                    }
                }
                pos.push((x, y));
                break;
            }
        }
    }
    let mut planets = Vec::with_capacity(pos.len());
    for (x, y) in pos {
        let mut p = empty_planet(x + 4, y + 4);
        p.star = match rng.below(0x14) {
            3..=8 => StarType::Red,
            9..=13 => StarType::Green,
            14..=16 => StarType::Blue,
            17..=18 => StarType::White,
            19 => StarType::Neutron,
            _ => StarType::Yellow,
        };
        rng.below(2); // look
        rng.below(50); // animation frame
        rng.below(4); // keeps the original random sequence
        let in_nebula = inside(nebulae, p.x, p.y);
        p.nebula = in_nebula;
        let neb = if in_nebula { 4 } else { 0 };
        let r = rng.one_to(20) as i32 - neb;
        p.kind = planet_type_for(p.star, r);
        let t = p.kind.index() as i32;
        let mut a: i32 = match p.kind {
            PlanetType::NotHabitable => 10,
            PlanetType::Radiated | PlanetType::Toxic | PlanetType::Inferno => {
                rng.one_to(7) as i32 * 5 + 5
            }
            PlanetType::Dead | PlanetType::Tundra => rng.one_to(7) as i32 * 5 + 15,
            PlanetType::Barren | PlanetType::Minimal => rng.one_to(5) as i32 * 5 + 25,
            PlanetType::Jungle => rng.one_to(4) as i32 * 5 + 70,
            PlanetType::Terran => rng.one_to(4) as i32 * 5 + 80,
            _ => rng.one_to(4) as i32 * 5 + (t - PlanetType::Desert.index() as i32) * 10 + 30,
        };
        let mut di = rng.one_to(10);
        while di == 1 || di == 10 {
            a += if di == 1 { -20 } else { 20 };
            di = rng.one_to(10);
            a = a.max(10);
        }
        a = a.min(120);
        p.base_max_pop = a;
        p.max_pop = a;
        if !in_nebula {
            rng.one_to(4); // battle background
        }
        let star_mod = |s: StarType| match s {
            StarType::Red => -4,
            StarType::Green => -2,
            _ => 0,
        };
        let rich_mod = |s: StarType| match s {
            StarType::Blue => -2,
            StarType::Neutron => -5,
            _ => 0,
        };
        if p.kind >= PlanetType::Steppe {
            if rng.one_to(20) as i32 + star_mod(p.star) <= 2 {
                p.special = Special::Poor;
            }
            if rng.one_to(20) as i32 + star_mod(p.star) <= 5 && p.special == Special::Poor {
                p.special = Special::UltraPoor;
            }
        }
        if PlanetType::Steppe.index() as i32 - t
            > rng.one_to(20) as i32 - neb * 2 + rich_mod(p.star)
        {
            p.special = Special::Rich;
        }
        if (rng.one_to(20) as i32 - neb * 2 + rich_mod(p.star)) < 6 && p.special == Special::Rich {
            p.special = Special::UltraRich;
        }
        if p.kind >= PlanetType::Minimal
            && p.kind <= PlanetType::Ocean
            && p.special == Special::Normal
            && rng.one_to(10) == 1
        {
            p.special = Special::Artifacts;
        }
        {
            // Asteroids in battles (1oom game_generate_planets).
            let mut di = rng.one_to(100) as i32;
            if p.special == Special::UltraRich {
                di -= 20;
            } else if p.special == Special::Rich {
                di -= 10;
            }
            di -= 15 - p.kind as i32;
            p.rocks = if di < 15 {
                2
            } else if di < 40 {
                1
            } else {
                0
            };
        }
        p.growth = Growth::Normal;
        if p.kind < PlanetType::Minimal {
            p.growth = Growth::Hostile;
        } else if p.kind > PlanetType::Desert && rng.one_to(12) == 1 {
            p.growth = Growth::Fertile;
            p.base_max_pop += (p.base_max_pop / 20) * 5;
            p.max_pop = p.base_max_pop;
        }
        if p.kind == PlanetType::NotHabitable {
            p.growth = Growth::Normal;
            p.special = Special::Normal;
        }
        p.size = p.base_max_pop;
        p.image = PLANET_IMAGES[p.kind.index()][rng.below(6) as usize] - 1;
        planets.push(p);
    }
    planets
}

fn make_home(p: &mut Planet, rules: &Rules) {
    p.kind = PlanetType::Terran;
    p.star = StarType::Yellow;
    p.special = Special::Normal;
    p.growth = Growth::Normal;
    p.size = rules.home_max_pop;
    p.base_max_pop = rules.home_max_pop;
    p.max_pop = rules.home_max_pop;
    p.pop = rules.home_pop;
    p.factories = rules.home_factories;
    p.image = PLANET_IMAGES[PlanetType::Terran.index()][2] - 1;
    p.sliders = [0, 0, 50, 30, 20];
}

/// Picks home stars far from each other and from Orion.
fn pick_homes(st: &mut State, n: usize) -> Vec<PlanetId> {
    let stars = st.planets.len();
    let mut homes: Vec<PlanetId> = Vec::new();
    while homes.len() < n {
        let mut cand: Vec<(i64, PlanetId)> = (0..stars)
            .filter(|&i| i != st.orion && !homes.contains(&i))
            .map(|i| {
                let d = homes
                    .iter()
                    .chain(std::iter::once(&st.orion))
                    .map(|&h| {
                        let w = if h == st.orion { 0.6 } else { 1.0 };
                        (edist(st, i, h) * w) as i64
                    })
                    .min()
                    .unwrap_or(0);
                (d, i)
            })
            .collect();
        cand.sort_by(|a, b| b.cmp(a));
        // The first home is any star in the far half from Orion, the others are among the
        // stars farthest from every home picked so far.
        let top = if homes.is_empty() {
            cand.len().div_ceil(2)
        } else {
            cand.len().min(3)
        };
        let pick = cand[st.rng.below(top as u16) as usize].1;
        homes.push(pick);
    }
    homes
}

/// Every home gets two habitable neighbours, the nearest inside the starting fuel range, so
/// early expansion is possible. `reserved` holds stars already given to other homes.
fn ensure_neighbours(st: &mut State, home: PlanetId, reserved: &mut Vec<PlanetId>) {
    let mut near: Vec<PlanetId> = (0..st.planets.len())
        .filter(|&i| i != st.orion && st.planets[i].owner.is_none() && !reserved.contains(&i))
        .collect();
    near.sort_by_key(|&i| (edist(st, home, i) as i64, i));
    if let Some(&i) = near.first() {
        let (hx, hy) = (st.planets[home].x, st.planets[home].y);
        let p = &mut st.planets[i];
        let mut d = 29;
        while d > 0 && crate::umath::dist_steps(hx, hy, p.x, p.y) > 3 {
            let (mut x, mut y) = (hx, hy);
            crate::umath::go_line_dist(&mut x, &mut y, p.x, p.y, d);
            if crate::umath::dist_steps(hx, hy, x, y) <= 3 {
                p.x = x;
                p.y = y;
            }
            d -= 2;
        }
    }
    for &i in near.iter().take(2) {
        reserved.push(i);
        let p = &mut st.planets[i];
        if p.kind < PlanetType::Minimal {
            p.kind = if st.rng.below(2) == 0 {
                PlanetType::Steppe
            } else {
                PlanetType::Arid
            };
            p.growth = Growth::Normal;
            p.base_max_pop = 40 + 5 * st.rng.below(5) as i32;
            p.max_pop = p.base_max_pop;
            p.size = p.base_max_pop;
            p.image = PLANET_IMAGES[p.kind.index()][0] - 1;
        }
    }
}

pub fn new_game(opts: &NewGameOptions) -> Result<State, String> {
    let players = opts.players as usize;
    if !(2..=6).contains(&players) {
        return Err("players must be between 2 and 6".into());
    }
    let rules = opts.rules.clone().unwrap_or_default();
    if rules.start_designs.len() != rules.start_ships.len()
        || rules.start_designs.is_empty()
        || rules.start_designs.len() > MAX_DESIGNS
    {
        return Err(
            "rules: start_designs and start_ships must have the same length (1 to 6)".into(),
        );
    }
    let mut rng = Rng::new(opts.seed);
    let (w, h) = opts.galaxy_size.grid();
    let nebulae = generate_nebulae(&mut rng, opts.galaxy_size, rules.nebulae);
    let mut planets = generate_planets(&mut rng, w, h, &nebulae);
    let width = planets.iter().map(|p| p.x).max().unwrap_or(0) + 27;
    let height = planets.iter().map(|p| p.y).max().unwrap_or(0) + 25;

    let orion = rng.below(planets.len() as u16) as usize;
    {
        let p = &mut planets[orion];
        p.kind = PlanetType::Terran;
        p.base_max_pop = 120;
        p.max_pop = 120;
        p.special = Special::Orion;
        p.growth = Growth::Normal;
        p.image = PLANET_IMAGES[PlanetType::Terran.index()][rng.below(6) as usize] - 1;
    }

    let names: Vec<String> = if opts.star_names.len() >= planets.len() {
        opts.star_names.clone()
    } else {
        STAR_NAMES.iter().map(|s| s.to_string()).collect()
    };
    let mut used = vec![false; names.len()];
    for p in planets.iter_mut() {
        let mut j;
        loop {
            j = rng.below(names.len() as u16) as usize;
            if !used[j] {
                break;
            }
        }
        used[j] = true;
        p.name = names[j].clone();
    }
    planets[orion].name = opts.orion_name.clone().unwrap_or_else(|| "Orion".into());

    let mut st = State {
        version: SAVE_VERSION,
        rng,
        seed: opts.seed,
        turn: 0,
        year: rules.start_year,
        difficulty: opts.difficulty,
        galaxy_size: opts.galaxy_size,
        width,
        height,
        planets,
        nebulae,
        empires: Vec::new(),
        fleets: Vec::new(),
        transports: Vec::new(),
        orion,
        guardian: true,
        winner: None,
        next_id: 0,
        events: Vec::new(),
        rules,
        victory: None,
        council: Default::default(),
        evn: crate::events::EventState::new(),
        pending: Default::default(),
        battle: None,
        progress: None,
        ai_id: opts.ai_id,
        galaxy_seed: opts.seed,
        galaxy_w: 0,
        galaxy_h: 0,
        copyprot_status: 0,
        guardian_killer: None,
        star_dist: Vec::new(),
    };

    let mut races: Vec<Race> = Vec::new();
    if let (false, Some(r)) = (opts.all_ai, opts.race) {
        races.push(r);
    }
    while races.len() < players {
        let r = Race::ALL[st.rng.below(10) as usize];
        if !races.contains(&r) {
            races.push(r);
        }
    }
    let mut banners: Vec<u8> = Vec::new();
    if let (false, Some(b)) = (opts.all_ai, opts.banner) {
        banners.push(b.min(5));
    }
    while banners.len() < players {
        let b = st.rng.below(6) as u8;
        if !banners.contains(&b) {
            banners.push(b);
        }
    }

    let homes = pick_homes(&mut st, players);
    let stars = st.planets.len();
    let ndesigns = MAX_DESIGNS;
    for (i, (&race, &home)) in races.iter().zip(&homes).enumerate() {
        let mut orbit = vec![vec![0u32; ndesigns]; stars];
        for (d, &n) in st.rules.start_ships.iter().enumerate() {
            orbit[home][d] = n;
        }
        let mut designs = st.rules.start_designs.clone();
        for (d, name) in designs.iter_mut().zip(&opts.start_design_names) {
            if !name.trim().is_empty() {
                d.name = name.clone();
            }
        }
        for d in designs.iter_mut() {
            d.update();
            d.look += banners[i] * 24;
        }
        let ship_names = opts
            .ship_names
            .get(race.index())
            .filter(|n| n.len() >= 12)
            .cloned()
            .unwrap_or_else(|| {
                SHIP_NAMES[race.index()]
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            });
        let mut tech = Tech::new();
        tech.list = crate::tech::generate_list(&mut st.rng, race);
        let leader = match (i, &opts.leader) {
            (0, Some(l)) if !opts.all_ai => l.clone(),
            _ => opts
                .leader_names
                .get(race.index())
                .cloned()
                .unwrap_or_else(|| format!("{race:?}")),
        };
        st.empires.push(Empire {
            leader,
            race,
            banner: banners[i],
            ai: opts.all_ai || i > 0,
            alive: true,
            tech,
            designs,
            bombard: true,
            ship_names,
            reserve: 0,
            total_prod: 0,
            research: 0,
            maintenance: 0,
            orbit,
            explored: vec![false; stars],
            home,
            rel: Vec::new(),
            trait1: 0,
            trait2: 0,
            security: 0,
            tax: 0,
            total_trade: 0,
            spying_maint: 0,
            ab: Default::default(),
            base_shield: 1,
            base_comp: 1,
            base_weapon: 3,
            ship_maint: 0,
            bases_maint: 0,
            percent_prod_total_to_actual: 100,
            ai_p2_countdown: 0,
            ai_p3_countdown: 0,
            shipi_colony: -1,
            shipi_bomber: -1,
            design_year: vec![0; MAX_DESIGNS],
            shipcount: st
                .rules
                .start_ships
                .iter()
                .copied()
                .chain(std::iter::repeat(0))
                .take(MAX_DESIGNS)
                .collect(),
            have_reserve_fuel: vec![false; MAX_DESIGNS],
            seen: Vec::new(),
        });
        let rules = st.rules.clone();
        let p = &mut st.planets[home];
        make_home(p, &rules);
        p.owner = Some(i as u8);
        p.name = opts
            .home_names
            .get(race.index())
            .cloned()
            .unwrap_or_else(|| HOME_NAMES[race.index()].to_string());
    }
    crate::diplo::init(&mut st);
    st.council.voted = vec![None; st.empires.len()];
    st.council.refuse = vec![false; st.empires.len()];
    let mut reserved = Vec::new();
    for &home in &homes {
        ensure_neighbours(&mut st, home, &mut reserved);
    }
    // Sliders of the homes (1oom `game_generate_home_etc`, `game_ai_classic_new_game_init`).
    for (i, &home) in homes.iter().enumerate() {
        if st.empires[i].ai {
            crate::ai::new_game_init(&mut st, i, home);
        } else {
            st.planets[home].sliders = [0, 0, 100, 0, 0];
            st.empires[i].tech.alloc = [20, 10, 15, 15, 20, 20];
        }
    }
    // First research projects: computers pick at random (1oom
    // `game_ai_classic_new_game_tech`); humans get the first tech of each list and can
    // change it at no cost before any investment.
    crate::ai::new_game_tech(&mut st);
    for i in 0..st.empires.len() {
        if st.empires[i].ai {
            continue;
        }
        for f in 0..NUM_FIELDS {
            let tech = st.empires[i].tech.list[f][0][0];
            crate::tech::start_project(&mut st, i as u8, f, tech);
        }
    }
    for (i, &home) in homes.iter().enumerate() {
        st.empires[i].explored[home] = true;
    }
    st.init_star_dist();
    Ok(st)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_a_playable_galaxy() {
        let sizes = [
            GalaxySize::Small,
            GalaxySize::Medium,
            GalaxySize::Large,
            GalaxySize::Huge,
        ];
        for (size, seed) in sizes
            .iter()
            .flat_map(|&s| (1..=25).map(move |seed| (s, seed)))
        {
            let st = new_game(&NewGameOptions {
                galaxy_size: size,
                players: 6,
                seed,
                ..Default::default()
            })
            .unwrap();
            let (w, h) = size.grid();
            assert_eq!(st.planets.len(), (w * h) as usize);
            assert_eq!(st.empires.len(), 6);
            let homes: Vec<_> = st.empires.iter().map(|e| e.home).collect();
            for (i, &h) in homes.iter().enumerate() {
                assert_eq!(st.planets[h].owner, Some(i as u8));
                assert_eq!(st.planets[h].pop, 50);
                assert!(st.empires[i].explored[h]);
                assert_ne!(h, st.orion);
            }
            let mut names: Vec<_> = st.planets.iter().map(|p| p.name.clone()).collect();
            names.sort();
            names.dedup();
            assert_eq!(names.len(), st.planets.len());
            assert!(st.empires[0].ai && st.empires[1].ai || !st.empires[0].ai);
            for (i, e) in st.empires.iter().enumerate() {
                assert!(
                    (0..st.planets.len()).any(|p| st.planets[p].owner.is_none()
                        && p != st.orion
                        && st.planets[p].kind >= PlanetType::Minimal
                        && st.parsecs(e.home, p) <= 3),
                    "empire {i} has no planet to expand to"
                );
            }
        }
    }

    #[test]
    fn same_seed_same_galaxy() {
        let o = NewGameOptions {
            seed: 5,
            ..Default::default()
        };
        assert_eq!(new_game(&o).unwrap(), new_game(&o).unwrap());
        let o2 = NewGameOptions {
            seed: 6,
            ..Default::default()
        };
        assert_ne!(
            new_game(&o).unwrap().planets,
            new_game(&o2).unwrap().planets
        );
    }

    #[test]
    fn custom_names_are_used() {
        let names: Vec<String> = (0..24).map(|i| format!("City {i}")).collect();
        let st = new_game(&NewGameOptions {
            star_names: names,
            orion_name: Some("Atlantis".into()),
            race: Some(Race::Psilon),
            leader: Some("Me".into()),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(st.planets[st.orion].name, "Atlantis");
        assert_eq!(st.empires[0].race, Race::Psilon);
        assert_eq!(st.empires[0].leader, "Me");
        assert!(!st.empires[0].ai);
        assert!(st
            .planets
            .iter()
            .enumerate()
            .filter(|(i, p)| *i != st.orion && p.owner.is_none())
            .all(|(_, p)| p.name.starts_with("City ")));
    }
}
