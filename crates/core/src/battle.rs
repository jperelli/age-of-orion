//! Tactical space battles on a 10x8 grid (1oom `game_battle_human.c`, `game_battle.c`).
//!
//! MOO1 runs a battle as a blocking loop that asks the player for input. Here `Battle` is a
//! state machine stored in `State::battle`: `run` advances computer-controlled turns until a
//! human-controlled stack has to act (or the battle ends), `act` applies the player's order.
//! Everything that happens is appended to `log` for the UI to animate.
//!
//! Item 0 is the planet (its missile bases), items `1..=s[L].items` belong to the left side
//! and the rest to the right side, as in 1oom. Destroyed items are removed and the indices
//! shift; animations refer to stable `uid`s instead.

use serde::{Deserialize, Serialize};

use crate::battle_ai;
use crate::data::*;
use crate::design::*;
use crate::rng::Rng;
use crate::state::*;
use crate::types::*;

pub const AREA_W: i32 = 10;
pub const AREA_H: i32 = 8;
/// 1oom `game_num_bt_turn_max`.
pub const TURN_MAX: u16 = 50;
const POP_HP: u32 = 200;
const FACT_HP: u32 = 50;
const MISSILE_MAX: usize = 30;
const ITEM_MAX: usize = MAX_DESIGNS * 2 + 1;
const ROUTE_LEN: usize = 20;
/// Hit point of a ship picture within its 32x24 cell (FIRING.LBX in MOO1).
const TARGET_X: i32 = 16;
const TARGET_Y: i32 = 12;

pub const FLAG_REPULSOR: u16 = 1 << 1;
pub const FLAG_WARPDIS: u16 = 1 << 2;
pub const FLAG_STASIS: u16 = 1 << 3;
pub const FLAG_BLACKHOLE: u16 = 1 << 5;
pub const FLAG_SUBSPACE: u16 = 1 << 6;
pub const FLAG_TECHNULL: u16 = 1 << 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    L,
    R,
    None,
}

impl Side {
    fn idx(self) -> usize {
        match self {
            Side::L => 0,
            Side::R => 1,
            Side::None => 2,
        }
    }
}

/// 1oom `(side_a + side_b) == 1`: the two are on opposite sides.
pub fn opposed(a: Side, b: Side) -> bool {
    matches!((a, b), (Side::L, Side::R) | (Side::R, Side::L))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Wpn {
    pub t: usize,
    pub n: i32,
    pub numfire: i32,
    pub numshots: i32,
}

/// A stack of identical ships, the planet's missile bases or a monster (1oom `battle_item_s`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub uid: u32,
    pub name: String,
    /// Design index of the owner, `None` for the planet and monsters.
    pub design: Option<usize>,
    /// Picture: banner * 24 + hull * 6 + variant for ships, 0x90.. monsters, planet image.
    pub look: u16,
    pub complevel: i32,
    pub specials: [u8; 3],
    pub hp1: i32,
    pub hp2: i32,
    pub hull: u8,
    pub man: i32,
    pub defense: i32,
    pub misdefense: i32,
    pub absorb: i32,
    pub repair: i32,
    pub misshield: i32,
    pub pshield: i32,
    pub extrarange: i32,
    pub num: i32,
    pub pulsar: i32,
    pub stream: i32,
    pub stasis: i32,
    pub cloak: i32,
    pub subspace: i32,
    pub warpdis: i32,
    pub blackhole: i32,
    pub technull: i32,
    pub repulsor: i32,
    pub sbmask: u16,
    pub retreat: u8,
    pub sx: i32,
    pub sy: i32,
    pub selected: u8,
    pub stasisby: usize,
    pub unman: i32,
    pub can_retaliate: bool,
    pub side: Side,
    pub actman: i32,
    pub hploss: i32,
    pub maxrange: i32,
    /// -1 no missiles, 0 missiles held back, 1 missiles enabled.
    pub missile: i8,
    pub wpn: [Wpn; 4],
}

impl Item {
    fn blank(uid: u32) -> Self {
        Item {
            uid,
            name: String::new(),
            design: None,
            look: 0,
            complevel: 0,
            specials: [0; 3],
            hp1: 0,
            hp2: 0,
            hull: 0,
            man: 0,
            defense: 0,
            misdefense: 0,
            absorb: 0,
            repair: 0,
            misshield: 0,
            pshield: 0,
            extrarange: 0,
            num: 0,
            pulsar: 0,
            stream: 0,
            stasis: 0,
            cloak: 0,
            subspace: 0,
            warpdis: 0,
            blackhole: 0,
            technull: 0,
            repulsor: 0,
            sbmask: 0,
            retreat: 0,
            sx: -1,
            sy: -1,
            selected: 0,
            stasisby: 0,
            unman: 0,
            can_retaliate: false,
            side: Side::None,
            actman: 0,
            hploss: 0,
            maxrange: 0,
            missile: -1,
            wpn: [Wpn::default(); 4],
        }
    }

    fn set_bools(&mut self) {
        let f = |b| i32::from(self.sbmask & b != 0);
        self.stasis = f(FLAG_STASIS);
        self.subspace = f(FLAG_SUBSPACE);
        self.blackhole = f(FLAG_BLACKHOLE);
        self.warpdis = f(FLAG_WARPDIS);
        self.technull = f(FLAG_TECHNULL);
        self.repulsor = f(FLAG_REPULSOR);
        self.cloak = f(FLAG_CLOAK);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Missile {
    pub id: u32,
    pub wpnt: usize,
    pub nummissiles: i32,
    pub damagemul2: i32,
    pub x: i32,
    pub y: i32,
    pub source: usize,
    /// Item index, -1 = none (removed at the next cleanup).
    pub target: i32,
    pub fuel: i32,
    pub speed: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BSide {
    /// Empire, or -1 guardian, -2 crystal, -3 amoeba.
    pub party: i32,
    /// `None` for monsters.
    pub race: Option<Race>,
    /// Ships per design of the owner at the end of the battle.
    pub ships: Vec<u32>,
    pub items: usize,
    pub apparent_force: i32,
    pub have_scan: bool,
    pub base_missile: bool,
    pub human: bool,
    pub auto: bool,
}

/// What the UI animates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Anim {
    Turn {
        uid: u32,
    },
    Move {
        uid: u32,
        x: i32,
        y: i32,
    },
    Teleport {
        uid: u32,
        x: i32,
        y: i32,
    },
    Beam {
        from: u32,
        to: u32,
        weapon: usize,
    },
    Bomb {
        from: u32,
        to: u32,
        weapon: usize,
    },
    Damage {
        uid: u32,
        amount: i32,
        killed: i32,
    },
    MissileLaunch {
        id: u32,
        from: u32,
        to: u32,
        weapon: usize,
        x: i32,
        y: i32,
    },
    MissileStep {
        id: u32,
        x: i32,
        y: i32,
    },
    MissileGone {
        id: u32,
    },
    MissileShield {
        uid: u32,
    },
    Explosion {
        x: i32,
        y: i32,
    },
    Destroyed {
        uid: u32,
    },
    Retreat {
        uid: u32,
    },
    Special {
        kind: String,
        from: u32,
        to: Option<u32>,
    },
    Repulse {
        uid: u32,
        x: i32,
        y: i32,
    },
    Cloak {
        uid: u32,
        on: bool,
    },
}

/// The player's order for the current stack.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Click on a grid cell: move there or attack what is there.
    Cell {
        x: i32,
        y: i32,
    },
    Done,
    Wait,
    /// Let the computer fight the rest of the battle.
    Auto,
    /// Toggle missiles (or, for the planet, switch base weapons).
    Missile,
    /// Toggle the special device (stasis, pulsar).
    Special,
    Retreat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Stage {
    Start,
    Round,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Iter {
    Continue,
    NeedHuman,
    TurnDone,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Battle {
    pub planet: PlanetId,
    pub planet_side: Side,
    pub pop: i32,
    pub fact: i32,
    pub bases: i32,
    pub biodamage: i32,
    pub s: [BSide; 2],
    pub autoresolve: bool,
    pub autoretreat: bool,
    pub has_attacked: bool,
    pub bases_using_mirv: bool,
    pub turn_done: bool,
    pub num_repulsed: u8,
    pub flag_cur_item_destroyed: bool,
    pub items_num: usize,
    pub items_num2: usize,
    pub cur_item: usize,
    pub special_button: i8,
    pub popdamage: u32,
    pub factdamage: u32,
    pub num_turn: u16,
    pub have_subspace_int: bool,
    pub antidote: i32,
    pub item: Vec<Item>,
    pub prio_i: usize,
    pub priority: [i32; ITEM_MAX],
    pub rocks: Vec<(i32, i32, u8)>,
    pub missiles: Vec<Missile>,
    /// 1oom `area`: -100 rock, -30 enemy planet out of range, -i enemy out of range, 0 empty
    /// or unreachable, 1 can move there, 10 + i friendly, 30 + i enemy in range.
    pub area: [[i8; AREA_W as usize]; AREA_H as usize],
    pub difficulty: u8,
    /// Battle ended: the winning side.
    pub winner: Option<Side>,
    /// A human must give an order to `cur_item`.
    pub waiting: bool,
    pub started: bool,
    pub log: Vec<Anim>,
    stage: Stage,
    item_started: bool,
    vc: usize,
    flag_round_done: bool,
    /// The result has been put into the game.
    #[serde(default)]
    pub applied: bool,
    next_uid: u32,
    next_missile: u32,
    #[serde(skip, default = "default_rng")]
    rng: Rng,
}

fn default_rng() -> Rng {
    Rng::new(1)
}

/// 1oom `util_math_dist_maxabs`.
pub fn dist(x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
    (x1 - x0).abs().max((y1 - y0).abs())
}

/// 1oom `util_math_line_plot`: the cells from (x0, y0) (excluded) to (x1, y1).
pub fn line_plot(mut x0: i32, mut y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let (dx, dirx) = if x1 - x0 < 0 {
        (x0 - x1, -1)
    } else {
        (x1 - x0, 1)
    };
    let (dy, diry) = if y1 - y0 < 0 {
        (y0 - y1, -1)
    } else {
        (y1 - y0, 1)
    };
    let mut zerr: i32 = 0x8000;
    if dx < dy {
        let slope = (dx << 16) / dy;
        while (out.len() as i32) < dy {
            y0 += diry;
            zerr += slope;
            if zerr >= 0x10000 {
                x0 += dirx;
            }
            zerr &= 0xffff;
            out.push((x0, y0));
        }
    } else if dy < dx {
        let slope = (dy << 16) / dx;
        while (out.len() as i32) < dx {
            x0 += dirx;
            zerr += slope;
            if zerr >= 0x10000 {
                y0 += diry;
            }
            zerr &= 0xffff;
            out.push((x0, y0));
        }
    } else {
        while (out.len() as i32) < dy {
            x0 += dirx;
            y0 += diry;
            out.push((x0, y0));
        }
    }
    out
}

fn route_step_len(adx: i32, ady: i32) -> i32 {
    let v = adx + ady;
    if v != 0 {
        v + 1
    } else {
        3
    }
}

/// 1oom `util_math_get_route_len`.
fn route_len(x0: i32, y0: i32, t: &[(i32, i32)]) -> i32 {
    let mut l = route_step_len((x0 - t[0].0).abs(), (y0 - t[0].1).abs());
    for i in 1..t.len() {
        l += route_step_len((t[i - 1].0 - t[i].0).abs(), (t[i - 1].1 - t[i].1).abs());
    }
    l
}

fn dist_fast(x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    if dx > dy {
        dx + dy / 2
    } else {
        dy + dx / 2
    }
}

/// 1oom `game_battle_get_absorbdiv` (with MOO1 v1.3a's Oracle fix, always on in 1oom).
pub fn absorbdiv(b: &Item, w: usize, _force_oracle_check: bool) -> i32 {
    let mut v = 1;
    if b.sbmask & FLAG_ORACLE != 0 {
        v = 2;
    }
    v += i32::from(WEAPONS[w].halve_shield);
    if v == 3 {
        4
    } else {
        v
    }
}

fn is_missile_weapon(w: usize) -> bool {
    let w = &WEAPONS[w];
    w.damage_min == w.damage_max && !w.bomb && w.miss_type == 0
}

/// Ship stats in battle (1oom `game_parsed_from_design` and `game_battle_prepare_add_ships`).
fn item_from_design(e: &Empire, d: usize, num: u32, nebula: bool, uid: u32) -> Item {
    let sd = &e.designs[d];
    let mut b = Item::blank(uid);
    let max = |f: fn(&crate::data_types::Special) -> u8| -> i32 {
        sd.specials
            .iter()
            .map(|&s| f(&SPECIALS[s as usize]) as i32)
            .max()
            .unwrap_or(0)
    };
    let extraman = max(|s| s.extra_man);
    b.name = sd.name.clone();
    b.design = Some(d);
    // Designs keep their banner in `look` (1oom `SHIP_LOOK_PER_BANNER`); designs saved
    // without it still get their empire's colour.
    b.look = e.banner as u16 * 24 + (sd.look % 24) as u16;
    b.hull = sd.hull;
    b.specials = sd.specials;
    b.hp1 = sd.hp as i32;
    b.hp2 = sd.hp as i32;
    b.repair = max(|s| s.repair);
    b.man = (sd.man as i32 + 3) / 2 + extraman / 2;
    b.complevel = COMPS[sd.comp as usize].level as i32;
    b.defense = sd.man as i32 + HULLS[sd.hull as usize].defense as i32 + extraman + 1;
    b.misdefense = b.defense + JAMMERS[sd.jammer as usize].level as i32;
    b.absorb = SHIELDS[sd.shield as usize].absorb as i32;
    b.misshield = max(|s| s.missile_shield);
    b.extrarange = max(|s| s.extra_range);
    b.num = num as i32;
    b.pulsar = max(|s| s.pulsar);
    b.stream = max(|s| s.stream);
    b.sbmask = sd
        .specials
        .iter()
        .fold(0, |m, &s| m | SPECIALS[s as usize].flags);
    if b.sbmask & FLAG_SCANNER != 0 {
        b.complevel += 1;
    }
    for i in 0..4 {
        b.wpn[i].t = sd.weapons[i] as usize;
        b.wpn[i].n = sd.weapon_counts[i] as i32;
    }
    if e.race == Race::Mrrshan {
        b.complevel += 4;
    }
    if e.race == Race::Alkari {
        b.defense += 3;
        b.misdefense += 3;
    }
    if nebula {
        b.pshield = 0;
        b.absorb = 0;
    }
    b.set_bools();
    b
}

/// 1oom `tbl_monster`: the Guardian, the crystal and the amoeba per difficulty.
fn monster_item(party: i32, diff: Difficulty, hp: u32, uid: u32) -> Item {
    let d = diff as usize;
    let mut b = Item::blank(uid);
    b.hull = HULL_HUGE;
    b.num = 1;
    b.man = 2;
    b.complevel = 10;
    match party {
        -1 => {
            const HP: [i32; 5] = [2000, 4000, 6000, 8000, 10000];
            const DEF: [i32; 5] = [1, 3, 5, 7, 9];
            const ABS: [i32; 5] = [5, 6, 7, 8, 9];
            const N: [[i32; 4]; 5] = [
                [5, 1, 5, 6],
                [25, 1, 15, 9],
                [45, 1, 25, 12],
                [65, 1, 35, 15],
                [85, 1, 45, 18],
            ];
            b.name = "guardian".into();
            b.look = 0x92;
            b.hp1 = HP[d];
            b.defense = DEF[d];
            b.misdefense = DEF[d];
            b.absorb = ABS[d];
            b.repair = match d {
                3 => 15,
                4 => 30,
                _ => 0,
            };
            b.misshield = 100;
            b.extrarange = 3;
            for (i, &t) in [56usize, 62, 58, 60].iter().enumerate() {
                b.wpn[i] = Wpn {
                    t,
                    n: N[d][i],
                    numfire: 0,
                    numshots: 0,
                };
            }
        }
        -2 => {
            b.name = "crystal".into();
            b.look = 0x90;
            b.hp1 = 3000 + 1000 * d as i32;
            b.defense = d as i32 + 1;
            b.misdefense = d as i32 + 1;
            b.absorb = 5;
            b.misshield = 100;
            b.sbmask = FLAG_BLACKHOLE;
            b.wpn[0] = Wpn {
                t: 61,
                n: 10,
                numfire: 0,
                numshots: 0,
            };
        }
        _ => {
            b.name = "amoeba".into();
            b.look = 0x91;
            b.hp1 = 1000 + 1000 * d as i32;
            b.defense = d as i32 + 1;
            b.misdefense = d as i32 + 1;
            b.repair = 50;
            b.wpn[0] = Wpn {
                t: 63,
                n: 1,
                numfire: 0,
                numshots: 0,
            };
        }
    }
    if party != -1 {
        b.hp1 = hp as i32;
    }
    b.hp2 = b.hp1;
    b.set_bools();
    b
}

impl Battle {
    /// 1oom `game_battle_prepare` and `game_battle_with_human_init`: `party_l` and
    /// `party_r` are empires (>= 0) or monsters (< 0, right side only).
    pub fn new(st: &mut State, party_r: i32, party_l: i32, planet: PlanetId) -> Battle {
        let p = st.planets[planet].clone();
        let side = |st: &State, party: i32| -> BSide {
            let (race, human, force) = if party >= 0 {
                let e = &st.empires[party as usize];
                let force = e
                    .designs
                    .iter()
                    .enumerate()
                    .map(|(d, sd)| (sd.hull as i32 + 1) * e.orbit[planet][d] as i32)
                    .sum();
                (Some(e.race), !e.ai, force)
            } else {
                (None, false, 1)
            };
            BSide {
                party,
                race,
                ships: vec![0; MAX_DESIGNS],
                items: 0,
                apparent_force: force,
                have_scan: false,
                base_missile: false,
                human,
                auto: !human,
            }
        };
        let mut bt = Battle {
            planet,
            planet_side: Side::None,
            pop: p.pop,
            fact: p.factories,
            bases: 0,
            biodamage: 0,
            s: [side(st, party_l), side(st, party_r)],
            autoresolve: false,
            autoretreat: false,
            has_attacked: false,
            bases_using_mirv: false,
            turn_done: false,
            num_repulsed: 0,
            flag_cur_item_destroyed: false,
            items_num: 0,
            items_num2: 0,
            cur_item: 0,
            special_button: -1,
            popdamage: 0,
            factdamage: 0,
            num_turn: 0,
            have_subspace_int: false,
            antidote: 0,
            item: vec![Item::blank(0)],
            prio_i: 0,
            priority: [0; ITEM_MAX],
            rocks: Vec::new(),
            missiles: Vec::new(),
            area: [[0; AREA_W as usize]; AREA_H as usize],
            difficulty: st.difficulty as u8,
            winner: None,
            waiting: false,
            started: false,
            log: Vec::new(),
            stage: Stage::Start,
            item_started: false,
            vc: 0,
            flag_round_done: false,
            applied: false,
            next_uid: 1,
            next_missile: 1,
            rng: st.rng,
        };
        // The planet.
        if let Some(owner) = p
            .owner
            .filter(|&o| o as i32 == party_l || o as i32 == party_r)
        {
            bt.planet_side = if owner as i32 == party_l {
                Side::L
            } else {
                Side::R
            };
            bt.bases = p.bases;
            bt.item[0] = bt.planet_item(st, owner);
        }
        bt.add_ships(st, Side::L, planet);
        if party_r < 0 {
            let hp = match party_r {
                -2 | -3 => st
                    .evn
                    .monsters
                    .iter()
                    .find(|m| m.dest == planet && m.killer.is_none())
                    .map_or(1, |m| m.hp),
                _ => 0,
            };
            let uid = bt.uid();
            let mut m = monster_item(party_r, st.difficulty, hp, uid);
            m.side = Side::R;
            bt.s[1].items = 1;
            bt.s[1].ships = vec![1];
            bt.item.push(m);
        } else {
            bt.add_ships(st, Side::R, planet);
        }
        bt
    }

    /// 1oom `game_battle_prepare` and `game_battle_with_human_init`: a battle on the grid
    /// (places the asteroids and the stacks, drawing random numbers).
    pub fn new_human(st: &mut State, party_r: i32, party_l: i32, planet: PlanetId) -> Battle {
        let mut bt = Battle::new(st, party_r, party_l, planet);
        bt.rng = st.rng;
        bt.init(st);
        st.rng = bt.rng;
        bt
    }

    fn uid(&mut self) -> u32 {
        let u = self.next_uid;
        self.next_uid += 1;
        u
    }

    /// 1oom `game_parsed_from_planet`.
    fn planet_item(&mut self, st: &State, owner: EmpireId) -> Item {
        let e = &st.empires[owner as usize];
        let p = &st.planets[self.planet];
        let (shield, comp, jammer) = st.base_parts(owner);
        let uid = self.uid();
        let mut b = Item::blank(uid);
        b.name = p.name.clone();
        b.complevel = comp as i32 + 1;
        if e.race == Race::Mrrshan {
            b.complevel += 4;
        }
        b.absorb = SHIELDS[shield].absorb as i32;
        b.hp1 = st.rules.base_hp[e.abilities().armor_tier.min(6)] as i32;
        b.hp2 = b.hp1;
        b.defense = 1;
        b.misdefense = jammer as i32 + 1;
        if p.bases > 0 {
            b.num = p.bases;
            b.sbmask = FLAG_SCANNER;
            if e.abilities().sub_space_interdictor {
                b.sbmask |= FLAG_SUBSPACE;
            }
        }
        b.pulsar = e.abilities().antidote;
        b.look = p.image as u16;
        b.pshield = p.shield;
        b.wpn[0] = Wpn {
            t: st.base_weapon(owner),
            n: 3,
            numfire: 0,
            numshots: -1,
        };
        b.wpn[1] = Wpn {
            t: st.base_weapon_2(owner),
            n: 3,
            numfire: 0,
            numshots: -1,
        };
        if p.nebula {
            b.pshield = 0;
            b.absorb = 0;
        }
        b.absorb += b.pshield;
        if p.bases == 0 {
            b.wpn[0].t = 0;
            b.wpn[0].n = 0;
        }
        b.set_bools();
        b.side = Side::None;
        b
    }

    /// 1oom `game_battle_prepare_add_ships`.
    fn add_ships(&mut self, st: &State, side: Side, planet: PlanetId) {
        let party = self.s[side.idx()].party;
        if party < 0 {
            return;
        }
        let e = &st.empires[party as usize];
        let nebula = st.planets[planet].nebula;
        for d in 0..e.designs.len() {
            let n = e.orbit[planet][d];
            if n == 0 {
                continue;
            }
            let uid = self.uid();
            let mut b = item_from_design(e, d, n, nebula, uid);
            b.side = side;
            if b.sbmask & FLAG_SCANNER != 0 {
                self.s[side.idx()].have_scan = true;
            }
            let pos = if side == Side::L {
                self.s[0].items + 1
            } else {
                self.item.len()
            };
            self.item.insert(pos, b);
            self.s[side.idx()].items += 1;
        }
    }

    /// 1oom `game_battle_with_human_init`, `_init_sub1` and `game_battle_place_items`.
    fn init(&mut self, st: &State) {
        let rocks = match st.planets[self.planet].rocks {
            1 => self.rng.one_to(4) as usize,
            2 => self.rng.below(3) as usize + 5,
            _ => 0,
        };
        if self.planet_side != Side::None && self.bases > 0 {
            self.s[self.planet_side.idx()].have_scan = true;
        }
        self.item[0].side = self.planet_side;
        self.items_num = self.s[0].items + self.s[1].items;
        for b in self.item.iter_mut() {
            let any = b
                .wpn
                .iter()
                .any(|w| WEAPONS[w.t].shots > 0 && !WEAPONS[w.t].bomb);
            b.missile = if any { 1 } else { -1 };
        }
        self.antidote = self.item[0].pulsar;
        self.item[0].pulsar = 0;
        if self.item[0].subspace > 0 {
            self.have_subspace_int = true;
            let (att, def) = if self.item[0].side == Side::L {
                (Side::R, Side::L)
            } else {
                (Side::L, Side::R)
            };
            let mut att_tele = false;
            for b in self.item.iter_mut().skip(1).filter(|b| b.side == att) {
                if b.subspace > 0 {
                    b.subspace = -1;
                    att_tele = true;
                }
            }
            if att_tele {
                for b in self.item.iter_mut().skip(1).filter(|b| b.side == def) {
                    if b.subspace > 0 {
                        b.subspace = -1;
                    }
                }
            }
        }
        for b in self.item.iter_mut() {
            b.hp1 = b.hp2;
        }
        // Placement.
        const STARTY: [i32; 6] = [3, 4, 2, 5, 1, 6];
        let (x, y) = match self.item[0].side {
            Side::None => (-1, -1),
            Side::L => (1, 3),
            Side::R => (8, 3),
        };
        self.item[0].sx = x;
        self.item[0].sy = y;
        let nl = self.s[0].items;
        for i in 1..=self.items_num {
            let (sx, k) = if i <= nl { (0, i - 1) } else { (9, i - nl - 1) };
            self.item[i].sx = sx;
            self.item[i].sy = STARTY[k.min(5)];
        }
        for i in 0..rocks {
            let look = self.rng.below(4) as u8;
            loop {
                let x = self.rng.below(7) as i32 + 2;
                let y = self.rng.below(8) as i32;
                let p = &self.item[0];
                let mut ok =
                    !(p.side != Side::None && (p.sx - x).abs() <= 1 && (p.sy - y).abs() <= 1);
                for &(rx, ry, _) in &self.rocks[..i] {
                    if (ry == y && (rx == x - 1 || rx == x + 1))
                        || (rx == x && (ry == y - 1 || ry == y + 1))
                    {
                        ok = false;
                    }
                }
                if ok {
                    self.rocks.push((x, y, look));
                    break;
                }
            }
        }
        for b in self.item.iter_mut().skip(1) {
            for w in b.wpn.iter_mut() {
                if w.n > 0 {
                    w.numshots = WEAPONS[w.t].shots as i32;
                }
            }
        }
    }

    // --------------------------------------------------------------- helpers

    fn side_race(&self, side: Side) -> Option<Race> {
        if side == Side::None {
            return None;
        }
        self.s[side.idx()].race
    }

    fn monster_right(&self) -> bool {
        self.s[1].race.is_none()
    }

    /// Whether specials affect `bd` (monsters on the right are immune).
    fn affects(&self, bd: &Item) -> bool {
        bd.side == Side::L || !self.monster_right()
    }

    fn quick(&self) -> bool {
        self.s[0].auto && self.s[1].auto
    }

    /// 1oom `game_battle_damage_planet`.
    fn damage_planet(&mut self, damage: u32) {
        let mut damage = damage;
        if self.item[0].num > 0 {
            damage /= 2;
        }
        self.popdamage += damage;
        let v = (self.popdamage / POP_HP) as i32;
        self.popdamage %= POP_HP;
        self.pop = (self.pop - v).max(0);
        self.factdamage += damage;
        let v = (self.factdamage / FACT_HP) as i32;
        self.factdamage %= FACT_HP;
        self.fact = (self.fact - v).max(0);
    }

    fn missile_remove_unused(&mut self) {
        let gone: Vec<u32> = self
            .missiles
            .iter()
            .filter(|m| m.target < 0)
            .map(|m| m.id)
            .collect();
        for id in gone {
            self.log.push(Anim::MissileGone { id });
        }
        self.missiles.retain(|m| m.target >= 0);
    }

    /// 1oom `game_battle_item_destroy`.
    fn item_destroy(&mut self, item_i: usize) {
        let side = self.item[item_i].side;
        self.item[item_i].selected = 0;
        if item_i != 0 {
            self.log.push(Anim::Destroyed {
                uid: self.item[item_i].uid,
            });
            if self.cur_item > item_i {
                self.cur_item -= 1;
            }
            for i in 0..=self.items_num2.min(ITEM_MAX - 1) {
                if self.priority[i] == item_i as i32 {
                    self.priority[i] = 50;
                }
                if self.priority[i] > item_i as i32 && self.priority[i] < 50 {
                    self.priority[i] -= 1;
                }
            }
            for b in self.item.iter_mut() {
                if b.stasisby == item_i {
                    b.stasisby = 0;
                }
                if b.stasisby > item_i {
                    b.stasisby -= 1;
                }
            }
            for m in self.missiles.iter_mut() {
                if m.source == item_i || m.target == item_i as i32 {
                    m.target = -1;
                }
                if m.source > item_i {
                    m.source -= 1;
                }
                if m.target > item_i as i32 {
                    m.target -= 1;
                }
            }
            if side != Side::None {
                self.s[side.idx()].items -= 1;
            }
            self.item.remove(item_i);
            self.items_num -= 1;
        } else {
            self.item[0].wpn[0].t = 0;
            for m in self.missiles.iter_mut() {
                if m.source == 0 {
                    m.target = -1;
                }
            }
            if self.have_subspace_int {
                self.have_subspace_int = false;
                for b in self.item.iter_mut() {
                    if b.subspace == -1 {
                        b.subspace = 1;
                    }
                }
            }
        }
        self.missile_remove_unused();
    }

    /// 1oom `game_battle_missile_spawn`.
    fn missile_spawn(
        &mut self,
        attacker_i: usize,
        target_i: usize,
        nummissiles: i32,
        wpnt: usize,
        damagemul2: i32,
    ) {
        if self.missiles.len() == MISSILE_MAX {
            self.missiles[0].target = -1;
            self.missile_remove_unused();
        }
        let w = &WEAPONS[wpnt];
        let b = &self.item[attacker_i];
        let side_n = match b.side {
            Side::R => 1,
            _ => 0,
        };
        let x = b.sx * 32 + TARGET_X + (32 - TARGET_X * 2) * side_n;
        let y = b.sy * 24 + TARGET_Y;
        let mut fuel = w.fuel as i32;
        if attacker_i == 0 {
            fuel += 12;
        }
        let id = self.next_missile;
        self.next_missile += 1;
        self.log.push(Anim::MissileLaunch {
            id,
            from: b.uid,
            to: self.item[target_i].uid,
            weapon: wpnt,
            x,
            y,
        });
        self.missiles.push(Missile {
            id,
            wpnt,
            nummissiles: nummissiles * w.streams as i32,
            damagemul2,
            x,
            y,
            source: attacker_i,
            target: target_i as i32,
            fuel,
            speed: w.dtbl[0] as i32,
        });
    }

    /// 1oom `game_battle_missile_hit`.
    fn missile_hit(&mut self, mi: usize) {
        let target = self.missiles[mi].target;
        if target < 0 {
            return;
        }
        let target_i = target as usize;
        let m = self.missiles[mi].clone();
        let w = &WEAPONS[m.wpnt];
        let damage = {
            let damagediv =
                if target_i == 0 && !w.bomb && (w.miss_type > 0 || w.damage_min != w.damage_max) {
                    2
                } else {
                    1
                };
            let mut d = if w.damage_fade {
                w.damage_max as i32
                    - ((w.fuel as i32 - m.fuel) * w.dtbl[0] as i32 + (w.dtbl[0] as i32 - m.speed))
                        / 2
            } else {
                w.damage_max as i32
            };
            d /= damagediv;
            d -= self.item[target_i].absorb / if w.halve_shield { 2 } else { 1 };
            d *= w.damage_mul as i32;
            if self.item[target_i].sbmask & FLAG_DISPLACEMENT != 0 && self.rng.one_to(100) < 35 {
                d = 0;
            }
            d
        };
        if damage < 0 {
            self.missiles[mi].target = -1;
            return;
        }
        let bs_complevel = self.item[m.source].complevel;
        let b = &self.item[target_i];
        let v1c = m.damagemul2;
        let mut hploss = b.hploss;
        let mut totaldamage: i32 = 0;
        let mut num_destroyed = 0;
        let mut hit_shield = false;
        let totalhp = b.hp1 * b.num - hploss;
        let mut misschance = 50 - (bs_complevel - b.misdefense) * 10;
        if b.cloak == 1 {
            misschance += 50;
        }
        misschance = misschance.min(95);
        let (hp1, misshield, bnum) = (b.hp1.max(1), b.misshield, b.num);
        for _ in 0..m.nummissiles {
            let chance = misschance - w.extra_acc as i32 * 10;
            if chance <= self.rng.one_to(100) as i32 {
                if misshield - (w.tech as i32) < self.rng.one_to(100) as i32 {
                    let v = damage.min(hp1) * v1c;
                    totaldamage += v;
                    hploss += v;
                } else {
                    hit_shield = true;
                }
            }
            let mut j = 0;
            while j < v1c && hp1 <= hploss {
                if bnum >= num_destroyed {
                    num_destroyed += 1;
                }
                hploss -= hp1;
                j += 1;
            }
            hploss %= hp1;
        }
        let tuid = self.item[target_i].uid;
        self.item[target_i].hploss = (totaldamage + self.item[target_i].hploss) % hp1;
        if hit_shield {
            self.log.push(Anim::MissileShield { uid: tuid });
        }
        self.missiles[mi].target = -1;
        if target_i == 0 {
            self.damage_planet(totaldamage.max(0) as u32);
        }
        totaldamage = totaldamage.min(totalhp);
        if self.item[target_i].hploss >= hp1 {
            num_destroyed += 1;
            self.item[target_i].hploss = 0;
        }
        let num = (self.item[target_i].num - num_destroyed).max(0);
        let killed = self.item[target_i].num - num;
        self.item[target_i].num = num;
        if totaldamage > 0 {
            self.log.push(Anim::Damage {
                uid: tuid,
                amount: totaldamage,
                killed,
            });
        }
        if num == 0 {
            if target_i == self.cur_item {
                self.flag_cur_item_destroyed = true;
            }
            self.item_destroy(target_i);
        }
    }

    /// 1oom `game_battle_missile_rock_collide`.
    fn missile_rock_collide(&mut self, mi: usize) -> i32 {
        const CHANCE: [u16; 10] = [10, 9, 8, 7, 7, 6, 5, 4, 3, 3];
        let m = &self.missiles[mi];
        let c = CHANCE[(WEAPONS[m.wpnt].extra_acc as usize).min(9)];
        let n = m.nummissiles;
        (0..n).filter(|_| self.rng.one_to(100) <= c).count() as i32
    }

    /// 1oom `game_battle_missile_move`: towards the target at pixel (`tx`, `ty`).
    fn missile_move(&mut self, mi: usize, tx: i32, ty: i32, step: i32) {
        let m = self.missiles[mi].clone();
        if m.target < 0 {
            return;
        }
        let b = &self.item[m.target as usize];
        let thx = if b.side == Side::R {
            tx + 32 - TARGET_X
        } else {
            tx + TARGET_X
        };
        let thy = ty + TARGET_Y;
        let (mut mx, mut my) = (m.x, m.y);
        let mut step = step;
        let (mut dx, mut dy) = (thx - mx, thy - my);
        if dx.abs() > dy.abs() {
            step = step.min(dx.abs());
            if dx == 0 {
                dx = 1;
            }
            mx += (dx / dx.abs()) * step;
            my += (dy * step * 3 + 3) / (dx.abs() * 4);
        } else {
            step = step.min(dy.abs());
            if dy == 0 {
                dy = 1;
            }
            mx += (dx * step) / dy.abs();
            my += ((dy / dy.abs()) * step * 3 + 3) / 4;
        }
        if !self.rocks.is_empty() {
            let mut collisions = 0;
            let mut d = 1;
            while d <= step {
                // 1oom util_math_go_line_dist, sampled every 4 pixels.
                let len = dist_fast(m.x, m.y, mx, my).max(1);
                let (x0, y0) = (
                    m.x + (mx - m.x) * d.min(len) / len,
                    m.y + (my - m.y) * d.min(len) / len,
                );
                for k in 0..self.rocks.len() {
                    let (rx, ry, _) = self.rocks[k];
                    if rx * 32 + 2 <= x0
                        && rx * 32 + 30 >= x0
                        && ry * 24 + 2 <= y0
                        && ry * 24 + 22 >= y0
                    {
                        collisions += self.missile_rock_collide(mi);
                    }
                }
                d += 4;
            }
            if collisions > 0 {
                let n = (self.missiles[mi].nummissiles - collisions).max(0);
                self.missiles[mi].nummissiles = n;
                if n == 0 {
                    self.missiles[mi].target = -1;
                }
                self.log.push(Anim::Explosion {
                    x: mx - 4,
                    y: my - 4,
                });
            }
        }
        if self.missiles[mi].nummissiles > 0 {
            self.log.push(Anim::MissileStep {
                id: m.id,
                x: mx,
                y: my,
            });
            let w = &WEAPONS[m.wpnt];
            if w.miss_type == 4 {
                let v = w.damage_max as i32
                    - ((w.fuel as i32 - m.fuel) * w.dtbl[0] as i32 + (w.dtbl[0] as i32 - m.speed))
                        / 2;
                if v < 0 {
                    self.missiles[mi].target = -1;
                }
            }
        }
        if thx - 10 <= mx && thx + 10 >= mx && thy - 10 <= my && thy + 10 >= my {
            self.missile_hit(mi);
        } else {
            let mm = &mut self.missiles[mi];
            mm.x = mx;
            mm.y = my;
            mm.speed = (mm.speed - step).max(0);
        }
    }

    /// 1oom `game_battle_item_finish`: missiles aimed at the current stack fly on.
    fn item_finish(&mut self, quick: bool) {
        let itemi = self.cur_item;
        if itemi >= self.item.len() || self.item[itemi].side == Side::None {
            return;
        }
        let delay = if quick { 5 } else { 10 };
        let mut done = false;
        while !done {
            done = true;
            let mut i = 0;
            while i < self.missiles.len() {
                if itemi >= self.item.len() {
                    break;
                }
                let m = &self.missiles[i];
                if m.target == itemi as i32 && m.speed > 0 {
                    let step = (WEAPONS[m.wpnt].dtbl[0] as i32 / delay).min(m.speed).max(1);
                    let (tx, ty) = (self.item[itemi].sx * 32, self.item[itemi].sy * 24);
                    self.missile_move(i, tx, ty, step);
                    done = false;
                }
                i += 1;
            }
        }
        self.missile_remove_unused();
    }

    fn get_priority(b: &Item, race: Option<Race>) -> i32 {
        let mut prio = b.complevel + b.man - b.unman;
        if b.sbmask & FLAG_SCANNER != 0 {
            prio += 3;
        }
        if b.subspace > 0 {
            prio += b.subspace * 1000;
        }
        if race == Some(Race::Alkari) {
            prio += 3;
        }
        prio
    }

    /// 1oom `game_battle_build_priority`.
    fn build_priority(&mut self) {
        let n = self.items_num;
        for i in 0..=n {
            self.priority[i] = (n - i) as i32;
        }
        let prio = |bt: &Battle, i: usize| {
            let b = &bt.item[bt.priority[i] as usize];
            Battle::get_priority(b, bt.side_race(b.side))
        };
        for i in 1..=n {
            let itemi = self.priority[i];
            let prio_i = {
                let b = &self.item[itemi as usize];
                Battle::get_priority(b, self.side_race(b.side))
            };
            let mut j = i as i32 - 1;
            while j >= 0 && prio(self, j as usize) < prio_i {
                self.priority[(j + 1) as usize] = self.priority[j as usize];
                j -= 1;
            }
            self.priority[(j + 1) as usize] = itemi;
        }
    }

    /// 1oom `game_battle_item_done`.
    fn item_done(&mut self) {
        if self.cur_item < self.item.len() {
            self.item[self.cur_item].selected = 0;
        }
        let m = self.items_num2 + 1;
        let mut guard = 0;
        loop {
            self.priority[self.prio_i] = -1;
            self.prio_i = (self.prio_i + 1) % m;
            guard += 1;
            if self.priority[self.prio_i] != 50 || guard > ITEM_MAX {
                break;
            }
        }
    }

    /// 1oom `game_battle_missile_turn_done`.
    fn missile_turn_done(&mut self) {
        for m in self.missiles.iter_mut() {
            if m.speed <= 0 {
                m.speed = WEAPONS[m.wpnt].dtbl[0] as i32;
                m.fuel -= 1;
                if m.fuel == 0 {
                    m.target = -1;
                }
            }
        }
        self.missile_remove_unused();
    }

    /// 1oom `game_battle_reset_specials`.
    fn reset_specials(&mut self) {
        let itemi = self.cur_item;
        for b in self.item.iter_mut() {
            if b.repulsor > 0 {
                b.repulsor = 1;
            }
        }
        let b = &self.item[itemi];
        self.special_button = if b.stasis > 0 || b.pulsar > 0 { 1 } else { -1 };
        if itemi == 0 {
            self.special_button = -1;
        }
        let no_missiles = !b.wpn.iter().any(|w| w.numshots > 0 && !WEAPONS[w.t].bomb);
        let b = &mut self.item[itemi];
        if no_missiles {
            b.missile = -1;
        }
        if itemi == 0 {
            b.missile = 1;
        }
        b.can_retaliate = true;
        for b2 in self.item.iter_mut() {
            if b2.stasisby == itemi {
                b2.stasisby = 0;
            }
        }
        let b = &mut self.item[itemi];
        if b.warpdis == 2 {
            b.warpdis = 1;
        }
        if b.stasis == 3 {
            b.stasis = 1;
        }
        if b.pulsar > 2 {
            b.pulsar -= 2;
        }
        if b.stream > 2 {
            b.stream -= 2;
        }
        if b.subspace == 2 {
            b.subspace = 1;
        }
        if b.blackhole == 2 {
            b.blackhole = 1;
        }
        if b.technull == 2 {
            b.technull = 1;
        }
        if b.repair > 0 {
            let repair = b.hp2 * b.repair / 100;
            if b.hp1 < b.hp2 {
                b.hp1 = (b.hp1 + repair).min(b.hp2);
            } else {
                b.hploss = (b.hploss - repair).max(0);
            }
        }
        if b.cloak > 1 {
            b.cloak = 2;
        }
    }

    /// 1oom `game_battle_get_weap_maxrange`.
    pub(crate) fn weap_maxrange(&mut self) -> i32 {
        let ci = self.cur_item;
        let is_planet = ci == 0;
        let b = &self.item[ci];
        let num_weap = if is_planet { 1 } else { 4 };
        if is_planet && b.num > 0 && b.wpn[0].numfire > 0 {
            return 12;
        }
        let mut maxrange = 0;
        if b.blackhole == 1 {
            maxrange = 1;
        }
        if self.special_button == 1 && (b.stasis == 1 || b.pulsar == 1 || b.pulsar == 2) {
            maxrange = 1;
        }
        if b.stream == 1 || b.stream == 2 {
            maxrange = 2;
        }
        if b.warpdis == 1 {
            maxrange = 3;
        }
        if b.technull == 1 {
            maxrange = 4;
        }
        for i in 0..num_weap {
            let wp = b.wpn[i];
            let w = &WEAPONS[wp.t];
            if wp.n > 0 && wp.numfire > 0 && wp.numshots != 0 && !w.bomb {
                let range = if wp.numshots == -1 && w.miss_type == 0 {
                    w.range as i32 + b.extrarange
                } else {
                    let mut r = w.range as i32;
                    if b.missile == 0 && r > 1 && w.miss_type == 0 {
                        r = 0;
                    }
                    r
                };
                maxrange = maxrange.max(range);
            }
        }
        if maxrange == 0 && opposed(b.side, self.item[0].side) {
            for i in 0..num_weap {
                let wp = b.wpn[i];
                if wp.n > 0 && wp.numfire > 0 && wp.numshots != 0 && WEAPONS[wp.t].bomb {
                    self.has_attacked = false;
                }
            }
        }
        maxrange
    }

    /// 1oom `game_battle_area_check_line_ok`: 1 clear, 0 through a rock, -1 blocked.
    pub(crate) fn line_ok(&self, t: &[(i32, i32)]) -> i32 {
        let mut r = 1;
        for &(x, y) in t {
            if !(0..AREA_W).contains(&x) || !(0..AREA_H).contains(&y) {
                continue;
            }
            let v = self.area[y as usize][x as usize];
            if v == -100 {
                r = 0;
            }
            if v == 10 || v == 30 || v == -30 {
                r = -1;
            }
        }
        r
    }

    /// 1oom `game_battle_item_move_find_route`.
    fn find_route(&self, itemi: usize, sx: i32, sy: i32) -> Vec<(i32, i32)> {
        let b = &self.item[itemi];
        let direct = line_plot(b.sx, b.sy, sx, sy);
        if self.line_ok(&direct) == 1 {
            return direct;
        }
        let mut best: Vec<(i32, i32)> = Vec::new();
        let (mut minrlen, mut minlen) = (999, 999);
        for sx2 in 0..AREA_W {
            for sy2 in 0..AREA_H {
                if b.sx == sx2 && b.sy == sy2 {
                    continue;
                }
                let t1 = line_plot(b.sx, b.sy, sx2, sy2);
                let len = t1.len() as i32;
                if self.line_ok(&t1) != 1 || b.man < len {
                    continue;
                }
                let rlen1 = route_len(b.sx, b.sy, &t1);
                if rlen1 > minrlen {
                    continue;
                }
                let t2 = line_plot(sx2, sy2, sx, sy);
                let len2 = t2.len() as i32;
                if self.line_ok(&t2) == 1 && b.man >= len + len2 {
                    let last = t1[t1.len() - 1];
                    let rlen2 = route_len(last.0, last.1, &t2);
                    if len2 + len <= minlen && rlen2 + rlen1 < minrlen {
                        minlen = len2 + len;
                        minrlen = rlen2 + rlen1;
                        best = t1.iter().chain(t2.iter()).copied().collect();
                    }
                } else {
                    'outer: for sx3 in 0..AREA_W {
                        for sy3 in 0..AREA_H {
                            if sx3 == sx2 && sy3 == sy2 {
                                break 'outer;
                            }
                            let t2 = line_plot(sx2, sy2, sx3, sy3);
                            let len2 = t2.len() as i32;
                            if self.line_ok(&t2) != 1 || b.man < len + len2 {
                                continue;
                            }
                            let last1 = t1[t1.len() - 1];
                            let rlen2 = route_len(last1.0, last1.1, &t2);
                            if rlen2 + rlen1 >= minrlen {
                                continue;
                            }
                            let t3 = line_plot(sx3, sy3, sx, sy);
                            let len3 = t3.len() as i32;
                            if self.line_ok(&t3) == 1 && b.man >= len + len2 + len3 {
                                let last2 = t2[t2.len() - 1];
                                let rlen3 = route_len(last2.0, last2.1, &t3);
                                if len3 + len + len2 <= minlen && rlen3 + rlen1 + rlen2 < minrlen {
                                    minlen = len3 + len + len2;
                                    minrlen = rlen3 + rlen1 + rlen2;
                                    best = t1
                                        .iter()
                                        .chain(t2.iter())
                                        .chain(t3.iter())
                                        .copied()
                                        .collect();
                                }
                            }
                        }
                    }
                }
            }
        }
        best.truncate(ROUTE_LEN);
        best
    }

    /// 1oom `game_battle_pulsar_get_dmg`.
    fn pulsar_dmg(&mut self, target_i: usize, v: i32) -> i32 {
        let b = &mut self.item[target_i];
        let totalhp = b.num * b.hp1;
        let mut v = v;
        if target_i == 0 {
            v /= 2;
        }
        v -= b.absorb;
        let mut dmg = 0;
        if v > 0 {
            if b.hp1 <= v {
                b.num = 0;
                dmg = totalhp;
            } else {
                b.hp1 -= v;
                dmg = b.num * v;
            }
            if b.hploss > b.hp1 && b.num > 0 {
                b.num -= 1;
            }
        }
        dmg
    }

    /// 1oom `game_battle_pulsar`.
    fn pulsar(&mut self, attacker_i: usize, ptype: i32) {
        let (ndiv, rbase) = if ptype == 0 { (2, 5) } else { (1, 10) };
        let (ax, ay, anum, auid) = {
            let b = &self.item[attacker_i];
            (b.sx, b.sy, b.num, b.uid)
        };
        self.log.push(Anim::Special {
            kind: if ptype == 0 {
                "pulsar".into()
            } else {
                "ion_pulsar".into()
            },
            from: auid,
            to: None,
        });
        for i in 0..self.item.len() {
            let bd = &self.item[i];
            if bd.stasisby == 0 && dist(ax, ay, bd.sx, bd.sy) == 1 {
                let v = self
                    .rng
                    .one_to((anum / ndiv + rbase).clamp(1, 32000) as u16)
                    as i32;
                let d = self.pulsar_dmg(i, v);
                if d > 0 {
                    self.log.push(Anim::Damage {
                        uid: self.item[i].uid,
                        amount: d,
                        killed: 0,
                    });
                }
            }
        }
    }

    /// 1oom `game_battle_special`: special devices on attack. Returns true if the attack
    /// is over (target destroyed or held in stasis).
    fn special(
        &mut self,
        attacker_i: usize,
        target_i: usize,
        dist_: i32,
        killedbelow: &mut usize,
    ) -> bool {
        let mut dist_ = dist_;
        let auid = self.item[attacker_i].uid;
        if self.item[attacker_i].cloak != 0 {
            if self.item[attacker_i].cloak == 1 {
                self.log.push(Anim::Cloak {
                    uid: auid,
                    on: false,
                });
            }
            self.item[attacker_i].cloak = 3;
        }
        let mut use_stasis = true;
        let aside = self.item[attacker_i].side;
        if !self.s[aside.idx().min(1)].human {
            let num = self.s[0].items;
            let n = (1..=num).filter(|&i| self.item[i].stasisby > 0).count();
            use_stasis = num as i32 - 2 >= n as i32;
        }
        let tuid = self.item[target_i].uid;
        {
            let b = &self.item[attacker_i];
            let bd = &self.item[target_i];
            if b.stasis == 1
                && dist_ == 1
                && target_i != 0
                && use_stasis
                && bd.cloak != 1
                && self.special_button == 1
            {
                self.item[attacker_i].stasis = 3;
                if self.affects(&self.item[target_i]) {
                    self.special_button = 0;
                    self.log.push(Anim::Special {
                        kind: "stasis".into(),
                        from: auid,
                        to: Some(tuid),
                    });
                    for m in self.missiles.iter_mut() {
                        if m.target == target_i as i32 {
                            m.target = -1;
                        }
                    }
                    self.item[target_i].stasisby = attacker_i;
                    return true;
                }
                return false;
            }
        }
        let stream = self.item[attacker_i].stream;
        if (stream == 1 || stream == 2) && dist_ <= 2 {
            if self.affects(&self.item[target_i]) {
                let anum = self.item[attacker_i].num;
                let bd = &mut self.item[target_i];
                let pct = if stream == 1 {
                    (anum / 2 + 20).min(50)
                } else {
                    (anum / 2 + 40).min(75)
                };
                let v = (bd.hp1 * pct + 99) / 100;
                bd.hp1 = (bd.hp1 - v).max(0);
                let damage = bd.num * v;
                self.item[attacker_i].stream = if stream == 1 { 3 } else { 4 };
                self.log.push(Anim::Special {
                    kind: if stream == 1 {
                        "stream".into()
                    } else {
                        "neutron_stream".into()
                    },
                    from: auid,
                    to: Some(tuid),
                });
                if damage > 0 {
                    self.log.push(Anim::Damage {
                        uid: tuid,
                        amount: damage,
                        killed: 0,
                    });
                }
                let bd = &mut self.item[target_i];
                if bd.hp1 <= bd.hploss {
                    if bd.num > 0 {
                        bd.num -= 1;
                    }
                    bd.hploss = 0;
                }
                if bd.num <= 0 || bd.hp1 <= 0 {
                    bd.num = 0;
                    bd.hp1 = 50;
                    self.item_destroy(target_i);
                    if target_i != 0 {
                        return true;
                    }
                }
            } else {
                self.item[attacker_i].stream += 2;
            }
        }
        let pulsar = self.item[attacker_i].pulsar;
        if pulsar >= 1 && dist_ <= 1 && self.special_button != 0 {
            self.special_button = 0;
            if pulsar == 1 || pulsar == 2 {
                self.pulsar(attacker_i, pulsar - 1);
                self.item[attacker_i].pulsar += 2;
            }
            let mut n = self.items_num as i32;
            let mut below = 0;
            while n > -1 {
                let nu = n as usize;
                if nu < self.item.len() && self.item[nu].num <= 0 && nu != 0 {
                    self.item_destroy(nu);
                    if target_i > nu {
                        below += 1;
                    }
                    if target_i == nu && target_i != 0 {
                        dist_ = 100;
                    }
                }
                n -= 1;
            }
            *killedbelow = below;
            if dist_ == 100 && target_i != 0 {
                return true;
            }
        }
        if attacker_i >= self.item.len() || target_i >= self.item.len() {
            return false;
        }
        if self.item[attacker_i].warpdis == 1 {
            let b = self.item[attacker_i].clone();
            let bd = &self.item[target_i];
            let t = if bd.unman >= bd.man || dist_ > 3 || target_i == 0 {
                let mut t = -1;
                for i in 1..=self.items_num {
                    let bi = &self.item[i];
                    if opposed(b.side, bi.side)
                        && dist(b.sx, b.sy, bi.sx, bi.sy) <= 3
                        && bi.unman < bi.man
                    {
                        t = i as i32;
                    }
                }
                t
            } else {
                target_i as i32
            };
            self.item[attacker_i].warpdis = 2;
            if t > 0 && self.affects(&self.item[t as usize]) {
                let tu = t as usize;
                self.log.push(Anim::Special {
                    kind: "warp_dissipator".into(),
                    from: auid,
                    to: Some(self.item[tu].uid),
                });
                let v = self.rng.below(2) as i32;
                let bi = &mut self.item[tu];
                bi.unman += v;
                if v > 0 && bi.man > bi.unman {
                    bi.defense = (bi.defense - 2).max(0);
                    bi.misdefense = (bi.misdefense - 2).max(0);
                }
                bi.unman = bi.unman.min(bi.man);
                bi.actman = (bi.actman - bi.unman).max(0);
            }
        }
        if self.item[attacker_i].blackhole == 1 && dist_ <= 1 {
            if self.affects(&self.item[target_i]) {
                let tuid = self.item[target_i].uid;
                self.log.push(Anim::Special {
                    kind: "black_hole".into(),
                    from: auid,
                    to: Some(tuid),
                });
                let mut v = self.rng.one_to(75) as i32 + 25;
                v -= self.item[target_i].absorb * 2;
                if v > 0 {
                    let n = self.item[target_i].num;
                    if n > 1 {
                        self.item[target_i].num = (100 - v) * n / 100;
                    } else if self.rng.one_to(100) as i32 > v {
                        self.item[target_i].num = 0;
                    }
                    let killed = n - self.item[target_i].num;
                    if killed > 0 {
                        self.log.push(Anim::Damage {
                            uid: tuid,
                            amount: killed * self.item[target_i].hp1,
                            killed,
                        });
                    }
                }
                if self.item[target_i].num <= 0 {
                    self.item_destroy(target_i);
                    if target_i != 0 {
                        return true;
                    }
                }
            }
            self.item[attacker_i].blackhole = 2;
        }
        if target_i < self.item.len() && self.item[attacker_i].technull == 1 && dist_ <= 4 {
            if self.affects(&self.item[target_i]) {
                let a = self.rng.one_to(3) as i32 + self.rng.one_to(3) as i32;
                let c = self.rng.one_to(3) as i32 + self.rng.one_to(3) as i32;
                let bd = &mut self.item[target_i];
                bd.complevel -= a;
                bd.misdefense = (bd.misdefense - c).max(bd.defense);
                let tuid = bd.uid;
                self.log.push(Anim::Special {
                    kind: "tech_nullifier".into(),
                    from: auid,
                    to: Some(tuid),
                });
            }
            self.item[attacker_i].technull = 2;
        }
        false
    }

    fn repulse_do(&mut self, target_i: usize, sx: i32, sy: i32, attacker_i: usize) {
        self.log.push(Anim::Repulse {
            uid: self.item[target_i].uid,
            x: sx,
            y: sy,
        });
        self.item[attacker_i].repulsor = 2;
        self.item[target_i].sx = sx;
        self.item[target_i].sy = sy;
        self.num_repulsed += 1;
    }

    /// 1oom `game_battle_repulse`.
    fn repulse(&mut self, attacker_i: usize, target_i: usize) {
        let (bx, by) = (self.item[attacker_i].sx, self.item[attacker_i].sy);
        let sx = self.item[target_i].sx * 2 - bx;
        let sy = self.item[target_i].sy * 2 - by;
        let free = |a: i8| a == 0 || a == 1 || a == (target_i as i8 + 10);
        if (0..AREA_W).contains(&sx)
            && (0..AREA_H).contains(&sy)
            && free(self.area[sy as usize][sx as usize])
        {
            self.repulse_do(target_i, sx, sy, attacker_i);
            return;
        }
        for xo in -1..2 {
            let x = sx + xo;
            if !(0..AREA_W).contains(&x) {
                continue;
            }
            for yo in -1..2 {
                let y = sy + yo;
                if !(0..AREA_H).contains(&y) {
                    continue;
                }
                if free(self.area[y as usize][x as usize]) && dist(x, y, bx, by) == 2 {
                    self.repulse_do(target_i, x, y, attacker_i);
                    return;
                }
            }
        }
    }

    /// 1oom `game_battle_move_retaliate`: enemies fire at a stack moving past them.
    fn move_retaliate(&mut self, itemi: usize) {
        let mut num_repulsed = 0;
        for b2 in self.item.iter_mut().skip(1) {
            if b2.repulsor > 1 {
                b2.repulsor = 1;
            }
        }
        let mut i = 1;
        while i <= self.items_num {
            if itemi >= self.item.len() {
                break;
            }
            let mut destroyed = false;
            let (bs, bx, by) = (
                self.item[itemi].side,
                self.item[itemi].sx,
                self.item[itemi].sy,
            );
            let b2 = &self.item[i];
            if opposed(bs, b2.side) && b2.stasisby == 0 && b2.cloak != 1 {
                if b2.can_retaliate {
                    destroyed = self.attack(i, itemi, true);
                } else if b2.repulsor == 1 && dist(bx, by, b2.sx, b2.sy) == 1 {
                    self.repulse(i, itemi);
                    if self.num_repulsed > num_repulsed {
                        i = 0;
                    }
                }
            }
            num_repulsed = self.num_repulsed;
            if destroyed {
                break;
            }
            i += 1;
        }
    }

    /// 1oom `game_battle_with_human_do_turn_ai`.
    fn do_turn_ai(&mut self) {
        let itemi = self.cur_item;
        self.item[itemi].maxrange = self.weap_maxrange();
        if self.item[itemi].retreat >= 2 {
            let b = &self.item[itemi];
            if let Some(d) = b.design {
                self.s[b.side.idx()].ships[d] = b.num as u32;
            }
            self.log.push(Anim::Retreat { uid: b.uid });
            self.item_destroy(itemi);
        } else {
            self.flag_cur_item_destroyed = false;
            self.num_repulsed = 0;
            if self.item[itemi].stasisby == 0 && self.item[itemi].num > 0 {
                battle_ai::turn(self);
            } else if itemi < self.item.len() {
                self.item[itemi].retreat = 0;
            }
            if itemi < self.item.len() && self.item[itemi].retreat == 1 {
                self.item[itemi].retreat = 2;
            }
        }
        if itemi < self.item.len() {
            let b = &mut self.item[itemi];
            if b.cloak == 2 && b.stasisby == 0 {
                b.cloak = 1;
                let uid = b.uid;
                self.log.push(Anim::Cloak { uid, on: true });
            }
        }
    }

    /// 1oom `game_battle_attack`: `attacker_i` fires every weapon it can at `target_i`.
    /// Returns true if the target was destroyed.
    pub(crate) fn attack(&mut self, attacker_i: usize, target_i: usize, retaliate: bool) -> bool {
        let mut target_i = target_i;
        let aside = self.item[attacker_i].side;
        let num_weap = if attacker_i == 0 {
            let b = &self.item[0];
            if !self.s[aside.idx().min(1)].auto
                && b.wpn[1].t != 0
                && self.s[aside.idx().min(1)].base_missile == (WEAPONS[b.wpn[0].t].streams > 1)
            {
                self.item[0].wpn.swap(0, 1);
            }
            1
        } else {
            4
        };
        let mut hploss = self.item[target_i].hploss;
        let totalhp: i64 = if target_i != 0 {
            (self.item[target_i].hp1 * self.item[target_i].num - hploss) as i64
        } else {
            2_000_000_000
        };
        let (ax, ay) = (self.item[attacker_i].sx, self.item[attacker_i].sy);
        let mut d = dist(ax, ay, self.item[target_i].sx, self.item[target_i].sy);
        if !retaliate {
            self.has_attacked = true;
        }
        let acl = self.item[attacker_i].complevel;
        let mut miss_beam = 50 - (acl - self.item[target_i].defense) * 10;
        let mut miss_missile = 50 - (acl - self.item[target_i].misdefense) * 10;
        let line = line_plot(ax, ay, self.item[target_i].sx, self.item[target_i].sy);
        if self.line_ok(&line) == 0 {
            miss_beam += 30;
        }
        if self.item[target_i].cloak == 1 {
            miss_beam += 50;
            miss_missile += 50;
        }
        miss_beam = miss_beam.min(95);
        miss_missile = miss_missile.min(95);
        {
            let mut below = 0;
            let mut done1 = false;
            if !(retaliate && self.item[target_i].cloak != 1) {
                done1 = self.special(attacker_i, target_i, d, &mut below);
            }
            if below > 0 {
                self.flag_cur_item_destroyed = false;
                target_i -= below;
            }
            if done1 || target_i >= self.item.len() || attacker_i >= self.item.len() {
                return false;
            }
        }
        d = dist(
            self.item[attacker_i].sx,
            self.item[attacker_i].sy,
            self.item[target_i].sx,
            self.item[target_i].sy,
        );
        let mut destroyed = false;
        let mut planetdamage: u32 = 0;
        let mut numkill = 0;
        for i in 0..num_weap {
            let mut damage2: i64 = 0;
            let wp = self.item[attacker_i].wpn[i];
            if !(wp.n > 0 && wp.numfire > 0 && wp.numshots != 0) {
                continue;
            }
            let w = &WEAPONS[wp.t];
            let damagerange = w.damage_max as i32 - w.damage_min as i32;
            let mut range = if attacker_i == 0 { 40 } else { 0 };
            if !w.bomb && damagerange != 0 {
                range += self.item[attacker_i].extrarange;
            } else if self.cur_item < self.item.len() && self.item[self.cur_item].maxrange < 0 {
                range += self.item[self.cur_item].maxrange;
            }
            range += w.range as i32;
            if range >= d && (!w.bomb || target_i == 0) {
                miss_beam -= w.extra_acc as i32 * 10;
                miss_missile -= w.extra_acc as i32 * 10;
                if damagerange != 0 || w.bio {
                    let adiv = if w.bomb {
                        if w.halve_shield {
                            2
                        } else {
                            1
                        }
                    } else {
                        absorbdiv(&self.item[attacker_i], wp.t, true)
                    };
                    let (auid, tuid) = (self.item[attacker_i].uid, self.item[target_i].uid);
                    if w.bomb {
                        if w.bio {
                            let mut dmgsum = 0;
                            for _ in 0..(wp.n * self.item[attacker_i].num) {
                                let dmg = (self.rng.below(w.damage_max + 1) as i32 - self.antidote)
                                    .max(0);
                                dmgsum += dmg;
                            }
                            self.pop = (self.pop - dmgsum).max(0);
                            self.biodamage += dmgsum;
                        }
                        self.log.push(Anim::Bomb {
                            from: auid,
                            to: tuid,
                            weapon: wp.t,
                        });
                    }
                    let damagediv = if target_i == 0
                        && !w.bomb
                        && (w.miss_type > 0 || w.damage_min != w.damage_max)
                    {
                        2
                    } else {
                        1
                    };
                    let mut damagemul2: i64 = 1;
                    let mut damagemul1: i64 = (self.item[attacker_i].num * wp.n) as i64;
                    while damagemul1 > 1000 {
                        damagemul1 /= 2;
                        damagemul2 *= 2;
                    }
                    let mut killed_total = 0;
                    while self.item[attacker_i].wpn[i].numfire > 0
                        && (self.item[target_i].num > 0 || target_i == 0)
                    {
                        if !w.bomb {
                            self.log.push(Anim::Beam {
                                from: auid,
                                to: tuid,
                                weapon: wp.t,
                            });
                        }
                        let aw = &mut self.item[attacker_i].wpn[i];
                        if w.miss_type >= 1 {
                            aw.numfire -= 1;
                        }
                        aw.numfire -= 1;
                        if aw.numshots > 0 {
                            aw.numshots -= 1;
                        }
                        let miss_chance = if w.bomb { miss_missile } else { miss_beam };
                        let (hp1, absorb, sbmask) = {
                            let bd = &self.item[target_i];
                            (bd.hp1.max(1), bd.absorb, bd.sbmask)
                        };
                        for _ in 0..damagemul1 {
                            let r = self.rng.one_to(100) as i32;
                            let mut dmg: i64 = 0;
                            if r >= miss_chance {
                                let mut x = ((r - miss_chance) * 2 * damagerange)
                                    / (100 - miss_chance).max(1);
                                x = (x + 1) / 2;
                                x += w.damage_min as i32;
                                x /= damagediv;
                                x -= absorb / adiv;
                                dmg = x as i64 * w.damage_mul as i64 * damagemul2;
                                if sbmask & FLAG_DISPLACEMENT != 0 && self.rng.one_to(100) < 35 {
                                    dmg = 0;
                                }
                            }
                            if dmg > 0 {
                                if (hp1 as i64) < dmg && w.streams == 0 {
                                    planetdamage = planetdamage.saturating_add(hp1 as u32);
                                    damage2 += hp1 as i64;
                                } else {
                                    planetdamage = planetdamage
                                        .saturating_add(dmg.min(u32::MAX as i64) as u32);
                                    damage2 += dmg;
                                }
                                hploss = (hploss as i64 + dmg).min(i32::MAX as i64) as i32;
                            }
                            let bnum = self.item[target_i].num;
                            let mut k: i64 = 0;
                            while (w.streams == 1 || k < damagemul2) && hp1 <= hploss {
                                if numkill < bnum {
                                    numkill += 1;
                                }
                                hploss -= hp1;
                                k += 1;
                            }
                            hploss %= hp1;
                        }
                        self.item[target_i].num -= numkill;
                        killed_total += numkill;
                        numkill = 0;
                        damage2 = damage2.min(totalhp);
                    }
                    if damage2 > 0 {
                        self.log.push(Anim::Damage {
                            uid: tuid,
                            amount: damage2.min(i32::MAX as i64) as i32,
                            killed: killed_total,
                        });
                    }
                } else if !retaliate
                    && self.cur_item < self.item.len()
                    && self.item[self.cur_item].missile != 0
                {
                    let aw = &mut self.item[attacker_i].wpn[i];
                    if w.miss_type >= 1 {
                        aw.numfire -= 1;
                    }
                    aw.numfire -= 1;
                    aw.numshots -= 1;
                    if attacker_i == 0 {
                        self.item[0].wpn[1].numfire -= 1;
                    }
                    let mut damagemul2 = 1;
                    let mut damagemul1 = self.item[attacker_i].num * wp.n;
                    while damagemul1 > 1000 {
                        damagemul1 /= 2;
                        damagemul2 *= 2;
                    }
                    if damagemul1 > 0 && !w.bomb {
                        self.missile_spawn(attacker_i, target_i, damagemul1, wp.t, damagemul2);
                    }
                }
                numkill = 0;
            }
            if self.item[target_i].num <= 0 {
                if target_i == self.cur_item {
                    self.flag_cur_item_destroyed = true;
                }
                self.item_destroy(target_i);
                destroyed = true;
                if target_i != 0 {
                    break;
                }
            }
        }
        if !destroyed && target_i < self.item.len() {
            self.item[target_i].hploss = hploss;
        }
        if target_i == 0 {
            self.damage_planet(planetdamage);
        }
        if self.cur_item < self.item.len() {
            let cs = self.item[self.cur_item].side;
            if cs != Side::None && !self.s[cs.idx()].auto {
                let r = self.weap_maxrange();
                self.item[self.cur_item].maxrange = r;
            }
        }
        if !destroyed
            && d == 1
            && target_i != 0
            && attacker_i < self.item.len()
            && target_i < self.item.len()
            && self.item[attacker_i].repulsor == 1
        {
            self.repulse(attacker_i, target_i);
        }
        destroyed
    }

    /// 1oom `game_battle_item_move`.
    pub(crate) fn item_move(&mut self, itemi: usize, sx: i32, sy: i32) {
        self.num_repulsed = 0;
        let uid = self.item[itemi].uid;
        if self.item[itemi].subspace == 1 {
            let b = &mut self.item[itemi];
            b.sx = sx;
            b.sy = sy;
            b.actman = 0;
            b.subspace = 2;
            self.log.push(Anim::Teleport { uid, x: sx, y: sy });
            return;
        }
        let stepdiv = if self.quick() { 2 } else { 8 };
        let mut x = self.item[itemi].sx * 32;
        let mut y = self.item[itemi].sy * 24;
        let route = self.find_route(itemi, sx, sy);
        for (sx, sy) in route {
            if self.flag_cur_item_destroyed || self.num_repulsed > 0 || itemi >= self.item.len() {
                break;
            }
            let b = &mut self.item[itemi];
            b.actman -= 1;
            let dz = sx - b.sx;
            let vx = if dz != 0 {
                (dz * (32 / stepdiv)) / dz.abs()
            } else {
                0
            };
            let dz = sy - b.sy;
            let vy = if dz != 0 {
                (dz * (24 / stepdiv)) / dz.abs()
            } else {
                0
            };
            b.sx = sx;
            b.sy = sy;
            self.log.push(Anim::Move { uid, x: sx, y: sy });
            for _ in 0..stepdiv {
                x += vx;
                y += vy;
                let mut j = 0;
                while j < self.missiles.len() {
                    if self.missiles[j].target == itemi as i32 {
                        let b = &self.item[itemi];
                        let mut v = WEAPONS[self.missiles[j].wpnt].dtbl[0] as i32;
                        if b.man - b.unman != 0 {
                            v /= b.man - b.unman;
                        }
                        v = ((v + stepdiv - 1) / stepdiv).min(self.missiles[j].speed);
                        self.missile_move(j, x, y, v);
                        if self.flag_cur_item_destroyed || self.num_repulsed > 0 {
                            break;
                        }
                    }
                    j += 1;
                }
                if self.flag_cur_item_destroyed || self.num_repulsed > 0 {
                    break;
                }
            }
            self.missile_remove_unused();
            if !self.flag_cur_item_destroyed
                && self.num_repulsed == 0
                && itemi < self.item.len()
                && self.item[itemi].cloak != 1
            {
                self.move_retaliate(itemi);
            }
            if self.flag_cur_item_destroyed || self.num_repulsed > 0 {
                break;
            }
        }
    }

    /// 1oom `game_battle_area_setup`: what the current stack can reach and hit.
    pub(crate) fn area_setup(&mut self) {
        let ci = self.cur_item;
        if ci >= self.item.len() {
            return;
        }
        let num_weap = if ci == 0 { 1 } else { 4 };
        let side = self.item[ci].side;
        if side != Side::None && !self.s[side.idx()].auto {
            let r = self.weap_maxrange();
            self.item[ci].maxrange = r;
        }
        self.turn_done = true;
        self.area = [[0; AREA_W as usize]; AREA_H as usize];
        let b = self.item[ci].clone();
        if b.actman > 0 {
            self.turn_done = false;
            for sy in 0..AREA_H {
                for sx in 0..AREA_W {
                    if b.subspace == 1 || b.actman >= dist(b.sx, b.sy, sx, sy) {
                        self.area[sy as usize][sx as usize] = 1;
                    }
                }
            }
        }
        for &(rx, ry, _) in &self.rocks {
            self.area[ry as usize][rx as usize] = -100;
        }
        for i in 0..self.item.len() {
            let b2 = &self.item[i];
            if b2.side == Side::None || b2.sx < 0 {
                continue;
            }
            let v: i8 = if b.side == b2.side {
                10 + i as i8
            } else {
                let d = dist(b.sx, b.sy, b2.sx, b2.sy);
                if i == 0 {
                    let mut v = -30;
                    for j in 0..num_weap {
                        let wp = b.wpn[j];
                        let w = &WEAPONS[wp.t];
                        let missile = is_missile_weapon(wp.t);
                        if b.actman > 0
                            || (w.range as i32 >= d && wp.numfire > 0 && wp.numshots != 0)
                        {
                            self.turn_done = false;
                        }
                        let range = if wp.numshots == -1 && w.miss_type == 0 {
                            w.range as i32 + b.extrarange
                        } else {
                            w.range as i32
                        };
                        if range >= d
                            && (b.missile == 1 || !missile)
                            && wp.numfire > 0
                            && wp.numshots != 0
                        {
                            v = 30;
                        }
                    }
                    if (b.blackhole == 1 && d == 1)
                        || ((b.pulsar == 1 || b.pulsar == 2) && self.special_button == 1 && d == 1)
                        || ((b.stream == 1 || b.stream == 2) && d <= 2)
                        || (b.technull == 1 && d <= 4)
                    {
                        self.turn_done = false;
                        v = 30;
                    }
                    v
                } else if b.maxrange >= d && b2.stasisby == 0 {
                    self.turn_done = false;
                    30 + i as i8
                } else {
                    -(i as i8)
                }
            };
            self.area[b2.sy as usize][b2.sx as usize] = v;
        }
        // Corners blocked by a rock or ship on the way are unreachable.
        let am = b.actman;
        for (dxs, dys) in [(-1, -1), (1, -1), (1, 1), (-1, 1)] {
            let mut zx = b.sx + dxs * am;
            let mut zy = b.sy + dys * am;
            let mut guard = 0;
            while (!(0..AREA_W).contains(&zx) || !(0..AREA_H).contains(&zy)) && guard < 40 {
                zx -= dxs;
                zy -= dys;
                guard += 1;
            }
            if !(0..AREA_W).contains(&zx) || !(0..AREA_H).contains(&zy) {
                continue;
            }
            let t = line_plot(b.sx, b.sy, zx, zy);
            if self.line_ok(&t) < 1 && self.area[zy as usize][zx as usize] == 1 {
                self.area[zy as usize][zx as usize] = 0;
            }
        }
        if b.missile == 0 {
            self.turn_done = false;
        }
    }

    // --------------------------------------------------------------- the loop

    /// 1oom `game_battle_with_human_do_sub2`.
    fn check_end(&mut self) -> Option<Side> {
        let mut items = [self.s[0].items, self.s[1].items];
        if self.item[0].num > 0 && self.item[0].side != Side::None {
            items[self.item[0].side.idx()] += 1;
        }
        if items[0] == 0 {
            return Some(Side::R);
        }
        if items[1] == 0 {
            return Some(Side::L);
        }
        if self.num_turn > TURN_MAX {
            return Some(if self.item[0].side == Side::None {
                Side::R
            } else {
                self.item[0].side
            });
        }
        if self.monster_right() || !self.s[1].auto {
            return None;
        }
        if battle_ai::retreat(self) {
            for b in self.item.iter_mut().skip(self.s[0].items + 1) {
                b.retreat += 1;
            }
        }
        None
    }

    /// Start of a round (1oom `game_battle_with_human_do_sub3`, before its loop).
    fn round_start(&mut self) {
        for b in self.item.iter_mut() {
            b.actman = b.man - b.unman;
            b.can_retaliate = false;
            for w in b.wpn.iter_mut() {
                let n = WEAPONS[w.t].fire as i32;
                w.numfire = (w.numfire + n).min(n);
            }
        }
        self.num_turn += 1;
        self.build_priority();
        self.items_num2 = self.items_num;
        self.flag_round_done = false;
        self.prio_i = 0;
        self.vc = 0;
        self.item_started = false;
    }

    fn check_round_done(&mut self) {
        let p = &self.item[0];
        if (p.side != Side::R || p.num <= 0) && self.s[1].items == 0 {
            self.flag_round_done = true;
        }
        if (p.side != Side::L || p.num <= 0) && self.s[0].items == 0 {
            self.flag_round_done = true;
        }
    }

    fn end_item(&mut self) {
        if !self.flag_cur_item_destroyed {
            let q = self.quick();
            self.item_finish(q);
        }
        if self.flag_cur_item_destroyed {
            self.item_done();
        }
        self.check_round_done();
        self.item_started = false;
    }

    /// One pass of the inner loop of `game_battle_with_human_do_sub3` for the current stack.
    fn iteration(&mut self, action: Option<Action>) -> Iter {
        let itemi = self.cur_item;
        if itemi >= self.item.len() {
            return Iter::TurnDone;
        }
        {
            let b = &mut self.item[itemi];
            if b.missile == 1 || b.missile == 0 {
                let any = b.wpn.iter().any(|w| w.numshots > 0 && !WEAPONS[w.t].bomb);
                if !any {
                    b.missile = -1;
                }
            } else {
                b.missile = -1;
            }
            if itemi == 0 {
                b.missile = 1;
            }
            b.selected = 1;
        }
        self.flag_cur_item_destroyed = false;
        self.num_repulsed = 0;
        let side = self.item[itemi].side;
        let sidx = side.idx().min(1);
        if self.s[sidx].auto || self.item[itemi].retreat > 0 || side == Side::None {
            if self.s[sidx].human
                && self.autoretreat
                && self.item[itemi].retreat == 0
                && side != Side::None
            {
                self.item[itemi].retreat = 1;
                return Iter::Continue;
            }
            self.do_turn_ai();
            self.item_done();
            return Iter::TurnDone;
        }
        // 1oom `ui_battle_turn` polls: with nothing left to do the turn ends by itself.
        let b = &self.item[itemi];
        let nothing_left =
            self.turn_done || (b.missile != 0 && b.maxrange == 0 && self.has_attacked);
        let act = if b.stasisby > 0 || (itemi == 0 && b.num <= 0) {
            Action::Done
        } else {
            match action {
                Some(a) => a,
                None if nothing_left => Action::Done,
                None => return Iter::NeedHuman,
            }
        };
        let mut turn_done = false;
        let b = &self.item[itemi];
        if act == Action::Done
            || self.turn_done
            || (b.missile != 0 && b.maxrange == 0 && self.has_attacked)
        {
            turn_done = true;
            let b = &mut self.item[itemi];
            if b.cloak == 2 && b.stasisby == 0 {
                b.cloak = 1;
                let uid = b.uid;
                self.log.push(Anim::Cloak { uid, on: true });
            }
            self.item_done();
        }
        match act {
            Action::Wait => {
                turn_done = true;
                self.item[itemi].selected = 0;
                self.item[itemi].can_retaliate = true;
                let m = self.items_num2 + 1;
                self.priority[self.vc] = itemi as i32;
                if self.vc != self.prio_i {
                    self.priority[self.prio_i] = -1;
                }
                self.vc = (self.vc + 1) % m;
                let mut guard = 0;
                loop {
                    self.prio_i = (self.prio_i + 1) % m;
                    guard += 1;
                    if self.priority[self.prio_i] != 50 || guard > ITEM_MAX {
                        break;
                    }
                    self.priority[self.prio_i] = -1;
                }
            }
            Action::Auto => {
                self.s[sidx].auto = true;
                if self.item[itemi].missile == 0 {
                    self.item[itemi].missile = 1;
                }
            }
            Action::Missile => {
                if itemi == 0 {
                    self.bases_using_mirv = !self.bases_using_mirv;
                    self.s[sidx].base_missile = !self.s[sidx].base_missile;
                    self.item[0].wpn.swap(0, 1);
                } else {
                    let b = &mut self.item[itemi];
                    b.missile = if b.missile == 0 { 1 } else { 0 };
                }
            }
            Action::Special => {
                self.special_button = if self.special_button == 0 { 1 } else { 0 };
            }
            Action::Retreat => self.item[itemi].retreat = 1,
            Action::Cell { x, y } => {
                if (0..AREA_W).contains(&x) && (0..AREA_H).contains(&y) {
                    let sa = self.area[y as usize][x as usize];
                    if sa == 1 {
                        self.item_move(itemi, x, y);
                    } else if (30..=43).contains(&sa) {
                        let t = (sa - 30) as usize;
                        if t < self.item.len() {
                            self.attack(itemi, t, false);
                        }
                    }
                }
            }
            Action::Done => {}
        }
        if !self.flag_cur_item_destroyed {
            self.area_setup();
            let ci = self.cur_item;
            if ci < self.item.len()
                && (self.turn_done
                    || (self.has_attacked
                        && self.item[ci].maxrange == 0
                        && self.item[ci].missile != 0))
            {
                self.item[ci].selected = 0;
            }
        } else {
            turn_done = true;
        }
        if turn_done {
            Iter::TurnDone
        } else {
            Iter::Continue
        }
    }

    /// Advances the battle until a human order is needed or the battle is over.
    fn run(&mut self, mut action: Option<Action>) {
        self.waiting = false;
        let mut guard = 0u32;
        loop {
            guard += 1;
            if guard > 200_000 {
                // Never expected: end the battle rather than hang.
                self.winner = Some(Side::R);
            }
            if self.winner.is_some() {
                return;
            }
            match self.stage {
                Stage::Start => {
                    if let Some(w) = self.check_end() {
                        self.winner = Some(w);
                        for b in self.item.iter().skip(1) {
                            if let (Some(d), true) = (b.design, b.side != Side::None) {
                                self.s[b.side.idx()].ships[d] = b.num.max(0) as u32;
                            }
                        }
                        return;
                    }
                    self.round_start();
                    self.stage = Stage::Round;
                }
                Stage::Round => {
                    if self.priority[self.prio_i] < 0 || self.flag_round_done {
                        self.missile_turn_done();
                        self.stage = Stage::Start;
                        continue;
                    }
                    if !self.item_started {
                        let itemi = self.priority[self.prio_i] as usize;
                        if itemi >= self.item.len() {
                            self.item_done();
                            continue;
                        }
                        self.cur_item = itemi;
                        if itemi == 0 {
                            self.item[0].retreat = 0;
                            let ps = self.item[0].side;
                            if ps != Side::None {
                                self.bases_using_mirv = self.s[ps.idx()].base_missile;
                            }
                        }
                        if itemi == 0 && self.item[0].num <= 0 {
                            self.special_button = -1;
                            let q = self.quick();
                            self.item_finish(q);
                            self.item_done();
                            self.check_round_done();
                            continue;
                        }
                        self.has_attacked = false;
                        self.reset_specials();
                        self.area_setup();
                        self.item_started = true;
                        self.log.push(Anim::Turn {
                            uid: self.item[itemi].uid,
                        });
                    }
                    match self.iteration(action.take()) {
                        Iter::Continue => {}
                        Iter::NeedHuman => {
                            self.waiting = true;
                            return;
                        }
                        Iter::TurnDone => self.end_item(),
                    }
                }
            }
        }
    }

    /// Starts the battle: fight it, let the computer resolve it, or retreat
    /// (1oom `ui_battle_init` choices).
    pub fn start(&mut self, st: &mut State, auto: bool, retreat: bool) {
        self.started = true;
        if auto || retreat {
            self.autoresolve = true;
            self.autoretreat = retreat;
            self.s[0].auto = true;
            self.s[1].auto = true;
        }
        self.with_rng(st, |bt| bt.run(None));
    }

    /// The player's order for the current stack.
    pub fn act(&mut self, st: &mut State, action: Action) -> Result<(), &'static str> {
        if self.winner.is_some() {
            return Err("battle_over");
        }
        if !self.waiting {
            return Err("not_your_turn");
        }
        self.with_rng(st, |bt| bt.run(Some(action)));
        Ok(())
    }

    pub(crate) fn rng_mut(&mut self) -> &mut Rng {
        &mut self.rng
    }

    fn with_rng(&mut self, st: &mut State, f: impl FnOnce(&mut Battle)) {
        self.rng = st.rng;
        self.log.clear();
        f(self);
        st.rng = self.rng;
    }

    /// Human side, if any.
    pub fn human_side(&self) -> Option<Side> {
        if self.s[0].human {
            Some(Side::L)
        } else if self.s[1].human {
            Some(Side::R)
        } else {
            None
        }
    }

    /// Resolves a battle without any human input (computer against computer and monsters).
    pub fn resolve_auto(st: &mut State, party_r: i32, party_l: i32, planet: PlanetId) -> Battle {
        let mut bt = Battle::new_human(st, party_r, party_l, planet);
        bt.start(st, true, false);
        bt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BattleMode, Command, Game, NewGameOptions};

    fn random_design(r: &mut Rng, name: &str) -> ShipDesign {
        let mut d = ShipDesign {
            name: name.into(),
            hull: r.below(4) as u8,
            look: 0,
            weapons: [0; 4],
            weapon_counts: [0; 4],
            engine: r.below(ENGINES.len() as u16) as u8,
            engines: 0,
            specials: [0; 3],
            shield: r.below(SHIELDS.len() as u16) as u8,
            jammer: r.below(JAMMERS.len() as u16) as u8,
            comp: r.below(COMPS.len() as u16) as u8,
            armor: r.below(ARMOR.len() as u16) as u8,
            man: 0,
            hp: 0,
            cost_bc: 0,
            space: 0,
        };
        d.look = d.hull * 6;
        d.man = r.below(d.engine as u16 + 1) as u8;
        for i in 0..4 {
            if r.below(3) > 0 {
                d.weapons[i] = r.below(WEAPONS.len() as u16) as u8;
                d.weapon_counts[i] = r.one_to(6) as u8;
            }
        }
        for i in 0..3 {
            if r.below(2) == 0 {
                d.specials[i] = r.below(SPECIALS.len() as u16) as u8;
            }
        }
        d.update();
        d
    }

    /// Two random fleets meet at empire 1's home; the human (empire 0) fights at random.
    fn random_battle(seed: u32, mode: BattleMode) -> Battle {
        let mut g = Game::new(&NewGameOptions {
            seed,
            players: 3,
            ..Default::default()
        })
        .unwrap();
        let mut r = Rng::new(seed);
        let st = g.state_mut();
        let target = st.empires[1].home;
        st.planets[target].bases = r.below(8) as i32;
        st.planets[target].rocks = (seed % 3) as u8;
        st.planets[target].nebula = seed % 7 == 0;
        for e in 0..2 {
            let n = r.one_to(6) as usize;
            st.empires[e].designs = (0..n)
                .map(|i| random_design(&mut r, &format!("d{i}")))
                .collect();
            for d in 0..n {
                st.empires[e].orbit[target][d] =
                    r.one_to(if seed % 5 == 0 { 3000 } else { 30 }) as u32;
            }
        }
        crate::diplo::set_treaty(st, 0, 1, crate::diplo::Treaty::War);
        assert!(crate::combat::battles(st));
        g.command(0, &Command::StartBattle { mode }).unwrap();
        let mut moves = 0;
        while g.state().battle.as_ref().unwrap().winner.is_none() {
            moves += 1;
            assert!(moves < 5000, "battle {seed} does not end");
            let bt = g.state().battle.as_ref().unwrap();
            assert!(bt.waiting);
            let action = match r.below(10) {
                0 => Action::Wait,
                1 => Action::Missile,
                2 => Action::Special,
                3 if r.below(10) == 0 => Action::Retreat,
                4 => Action::Done,
                _ => {
                    // Prefer cells with a target or a free square.
                    let cells: Vec<(i32, i32)> = (0..AREA_H)
                        .flat_map(|y| (0..AREA_W).map(move |x| (x, y)))
                        .filter(|&(x, y)| {
                            let a = bt.area[y as usize][x as usize];
                            a == 1 || a >= 30
                        })
                        .collect();
                    match cells.get(r.below(cells.len().max(1) as u16) as usize) {
                        Some(&(x, y)) => Action::Cell { x, y },
                        None => Action::Done,
                    }
                }
            };
            g.command(0, &Command::Battle { action }).unwrap();
        }
        let bt = (**g.state().battle.as_ref().unwrap()).clone();
        g.end_turn();
        bt
    }

    #[test]
    fn random_battles_end() {
        let mut wins = [0; 2];
        for seed in 1..60 {
            let mode = match seed % 3 {
                0 => BattleMode::Auto,
                1 => BattleMode::Fight,
                _ => BattleMode::Retreat,
            };
            let bt = random_battle(seed, mode);
            wins[(bt.winner == Some(Side::R)) as usize] += 1;
            assert!(bt.applied);
        }
        assert!(wins[0] > 0 && wins[1] > 0, "{wins:?}");
    }

    #[test]
    fn manual_battle_waits_for_orders() {
        let mut g = Game::new(&NewGameOptions::default()).unwrap();
        let st = g.state_mut();
        let target = st.empires[1].home;
        st.empires[0].orbit[target][1] = 5;
        st.planets[target].bases = 1;
        crate::diplo::set_treaty(st, 0, 1, crate::diplo::Treaty::War);
        assert!(crate::combat::battles(st));
        let bt = g.state().battle.as_ref().unwrap();
        assert!(!bt.started && bt.winner.is_none());
        g.command(
            0,
            &Command::StartBattle {
                mode: BattleMode::Fight,
            },
        )
        .unwrap();
        let bt = g.state().battle.as_ref().unwrap();
        assert!(
            bt.waiting,
            "the human's fighters act first or after the base"
        );
        let me = &bt.item[bt.cur_item];
        assert_eq!(me.side, Side::L);
        let reach = bt.area.iter().flatten().filter(|&&a| a == 1).count();
        assert!(reach > 1, "can move");
        assert!(g
            .command(
                1,
                &Command::Battle {
                    action: Action::Done
                }
            )
            .is_err());
        g.command(
            0,
            &Command::Battle {
                action: Action::Done,
            },
        )
        .unwrap();
    }

    #[test]
    fn line_plot_matches_1oom() {
        assert_eq!(line_plot(0, 0, 3, 1), vec![(1, 0), (2, 1), (3, 1)]);
        assert_eq!(line_plot(2, 2, 2, 5), vec![(2, 3), (2, 4), (2, 5)]);
        assert_eq!(line_plot(0, 0, 2, 2), vec![(1, 1), (2, 2)]);
        assert!(line_plot(4, 4, 4, 4).is_empty());
    }
}
