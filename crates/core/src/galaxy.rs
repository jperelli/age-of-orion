//! New game generation (port of 1oom `game_new.c`, without nebulae).

use serde::{Deserialize, Serialize};

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
    pub star_names: Vec<String>,
    /// Home world name per race (10 entries).
    pub home_names: Vec<String>,
    /// Leader name per race (10 entries).
    pub leader_names: Vec<String>,
    pub leader: Option<String>,
    pub orion_name: Option<String>,
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
            star_names: Vec::new(),
            home_names: Vec::new(),
            leader_names: Vec::new(),
            leader: None,
            orion_name: None,
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
        base_max_pop: 10,
        max_pop: 10,
        owner: None,
        pop: 0,
        pop_tenths: 0,
        factories: 0,
        waste: 0,
        bases: 0,
        shield: 0,
        pop_oper_fact: 2,
        sliders: [0, 0, 100, 0, 0],
        build: 1,
        prod: 0,
        bc_to_ship: 0,
        bc_to_factory: 0,
        bc_to_base: 0,
        bc_to_shield: 0,
        bc_to_refit: 0,
    }
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
fn generate_planets(rng: &mut Rng, w: u16, h: u16) -> Vec<Planet> {
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
        let r = rng.one_to(20) as i32;
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
        rng.one_to(4); // battle background
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
        if PlanetType::Steppe.index() as i32 - t > rng.one_to(20) as i32 + rich_mod(p.star) {
            p.special = Special::Rich;
        }
        if (rng.one_to(20) as i32 + rich_mod(p.star)) < 6 && p.special == Special::Rich {
            p.special = Special::UltraRich;
        }
        if p.kind >= PlanetType::Minimal
            && p.kind <= PlanetType::Ocean
            && p.special == Special::Normal
            && rng.one_to(10) == 1
        {
            p.special = Special::Artifacts;
        }
        rng.one_to(100); // asteroid rocks
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
                        (st.dist(i, h) * w) as i64
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
    near.sort_by_key(|&i| (st.dist(home, i) as i64, i));
    if let Some(&i) = near.first() {
        let d = st.dist(home, i);
        let max = st.rules.parsec * 3.0 - 3.0;
        if d > max {
            let (hx, hy) = (st.planets[home].x as f64, st.planets[home].y as f64);
            let p = &mut st.planets[i];
            p.x = (hx + (p.x as f64 - hx) * max / d).round() as i32;
            p.y = (hy + (p.y as f64 - hy) * max / d).round() as i32;
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
    if rules.designs.len() != rules.start_ships.len() || rules.designs.is_empty() {
        return Err("rules: designs and start_ships must have the same length".into());
    }
    let mut rng = Rng::new(opts.seed);
    let (w, h) = opts.galaxy_size.grid();
    let mut planets = generate_planets(&mut rng, w, h);
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
        empires: Vec::new(),
        fleets: Vec::new(),
        transports: Vec::new(),
        orion,
        guardian_hp: rules.guardian_hp,
        winner: None,
        next_id: 0,
        events: Vec::new(),
        rules,
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
    let ndesigns = st.rules.designs.len();
    for (i, (&race, &home)) in races.iter().zip(&homes).enumerate() {
        let mut orbit = vec![vec![0u32; ndesigns]; stars];
        orbit[home] = st.rules.start_ships.clone();
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
            tech: Tech {
                level: [1; NUM_FIELDS],
                invested: [0; NUM_FIELDS],
                alloc: st.rules.research_weights_ai,
            },
            reserve: 0,
            total_prod: 0,
            research: 0,
            maintenance: 0,
            orbit,
            explored: vec![false; stars],
            home,
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
    let mut reserved = Vec::new();
    for &home in &homes {
        ensure_neighbours(&mut st, home, &mut reserved);
    }
    crate::turn::update_exploration(&mut st);
    crate::turn::update_production(&mut st);
    for (i, &home) in homes.iter().enumerate() {
        let eco = crate::turn::clean_eco_percent(&st.planets[home], &st.empires[i]);
        st.planets[home].sliders = [0, 0, 75 - eco, eco, 25];
    }
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
