//! Reader for 1oom save files (1oom `game_save.c`, `game_save_decode`, format version 0).
//!
//! The structures mirror 1oom's `struct game_s` for the fields the save keeps; derived
//! values (abilities, ranges) are not in the file and are recomputed by both games.

use serde::Serialize;

pub const PLAYER_NUM: usize = 6;
pub const NUM_SHIPDESIGNS: usize = 6;
pub const PLANET_NONE: u8 = 255;
/// 1oom `PLAYER_NONE` (= `PLAYER_NUM`).
pub const PLAYER_NONE: u8 = 6;

struct Rd<'a> {
    b: &'a [u8],
    pos: usize,
}

impl Rd<'_> {
    fn u8(&mut self) -> u8 {
        let v = self.b[self.pos];
        self.pos += 1;
        v
    }
    fn i8(&mut self) -> i8 {
        self.u8() as i8
    }
    fn u16(&mut self) -> u16 {
        let v = u16::from_le_bytes([self.b[self.pos], self.b[self.pos + 1]]);
        self.pos += 2;
        v
    }
    fn i16(&mut self) -> i16 {
        self.u16() as i16
    }
    fn u32(&mut self) -> u32 {
        let v = u32::from_le_bytes([
            self.b[self.pos],
            self.b[self.pos + 1],
            self.b[self.pos + 2],
            self.b[self.pos + 3],
        ]);
        self.pos += 4;
        v
    }
    fn skip(&mut self, n: usize) {
        self.pos += n;
    }
    fn bv(&mut self, n: usize) -> Vec<bool> {
        let l = n.div_ceil(8);
        let bytes = &self.b[self.pos..self.pos + l];
        self.pos += l;
        (0..n).map(|i| bytes[i / 8] & (1 << (i % 8)) != 0).collect()
    }
    fn str(&mut self, n: usize) -> String {
        let bytes = &self.b[self.pos..self.pos + n];
        self.pos += n;
        let end = bytes.iter().position(|&c| c == 0).unwrap_or(n);
        bytes[..end].iter().map(|&c| c as char).collect()
    }
    fn tu8(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| self.u8()).collect()
    }
    fn tu16(&mut self, n: usize) -> Vec<u16> {
        (0..n).map(|_| self.u16()).collect()
    }
    fn ti16(&mut self, n: usize) -> Vec<i16> {
        (0..n).map(|_| self.i16()).collect()
    }
    fn tu32(&mut self, n: usize) -> Vec<u32> {
        (0..n).map(|_| self.u32()).collect()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Planet {
    pub name: String,
    pub x: u16,
    pub y: u16,
    pub star_type: u8,
    pub look: u8,
    pub frame: u8,
    pub rocks: u8,
    pub max_pop1: i16,
    pub max_pop2: i16,
    pub max_pop3: i16,
    pub kind: u8,
    pub battlebg: u8,
    pub infogfx: u8,
    pub growth: u8,
    pub special: u8,
    pub bc_to_ecoproj: i16,
    pub bc_to_ship: u16,
    pub bc_to_factory: u16,
    pub reserve: u32,
    pub waste: i16,
    pub owner: u8,
    pub prev_owner: u8,
    pub claim: u8,
    pub pop: i16,
    pub pop_prev: i16,
    pub factories: i16,
    pub slider: Vec<i16>,
    pub slider_lock: Vec<u8>,
    pub buildship: u8,
    pub reloc: u8,
    pub missile_bases: u16,
    pub bc_to_base: u16,
    pub bc_upgrade_base: u16,
    pub have_stargate: u8,
    pub shield: u8,
    pub bc_to_shield: u16,
    pub trans_num: u16,
    pub trans_dest: u8,
    pub pop_tenths: i8,
    pub explored: Vec<bool>,
    pub unrefuel: Vec<bool>,
    pub pop_oper_fact: u8,
    pub bc_to_refit: u16,
    pub rebels: i16,
    pub unrest: u8,
    pub unrest_reported: u8,
    pub finished: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Seen {
    pub owner: u8,
    pub pop: u16,
    pub bases: u16,
    pub factories: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct Fleet {
    pub owner: u8,
    pub x: u16,
    pub y: u16,
    pub dest: u8,
    pub speed: u8,
    pub retreat: bool,
    pub ships: Vec<u16>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Transport {
    pub owner: u8,
    pub x: u16,
    pub y: u16,
    pub dest: u8,
    pub speed: u8,
    pub pop: u16,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Eto {
    pub race: u8,
    pub banner: u8,
    pub trait1: u8,
    pub trait2: u8,
    pub ai_p3_countdown: i8,
    pub ai_p2_countdown: i8,
    pub contact: Vec<bool>,
    pub contact_broken: Vec<bool>,
    pub relation1: Vec<i16>,
    pub relation2: Vec<i16>,
    pub diplo_type: Vec<u8>,
    pub diplo_val: Vec<i16>,
    pub diplo_p1: Vec<u16>,
    pub diplo_p2: Vec<i16>,
    pub trust: Vec<i16>,
    pub broken_treaty: Vec<u8>,
    pub blunder: Vec<i16>,
    pub tribute_field: Vec<u8>,
    pub tribute_tech: Vec<u8>,
    pub mood_treaty: Vec<i16>,
    pub mood_trade: Vec<i16>,
    pub mood_tech: Vec<i16>,
    pub mood_peace: Vec<i16>,
    pub treaty: Vec<u8>,
    pub trade_bc: Vec<u16>,
    pub trade_percent: Vec<i16>,
    pub spymode_next: Vec<u8>,
    pub offer_field: Vec<u8>,
    pub offer_tech: Vec<u8>,
    pub offer_bc: Vec<u16>,
    pub attack_bounty: Vec<u8>,
    pub bounty_collect: Vec<u8>,
    pub attack_gift_field: Vec<u8>,
    pub attack_gift_tech: Vec<u8>,
    pub attack_gift_bc: Vec<i16>,
    pub hatred: Vec<i16>,
    pub have_met: Vec<u16>,
    pub trade_established_bc: Vec<u16>,
    pub spying: Vec<i16>,
    pub spyfund: Vec<u16>,
    pub spymode: Vec<u8>,
    pub security: i16,
    pub spies: Vec<u16>,
    pub reserve_bc: u32,
    pub tax: i16,
    pub base_shield: u8,
    pub base_comp: u8,
    pub base_weapon: u8,
    pub colonist_oper_factories: u8,
    pub tech_percent: Vec<u8>,
    pub tech_slider: Vec<i16>,
    pub tech_slider_lock: Vec<u8>,
    pub tech_investment: Vec<u32>,
    pub tech_project: Vec<u8>,
    pub tech_cost: Vec<u32>,
    pub tech_completed: Vec<u16>,
    pub shipdesigns_num: u8,
    /// Ships in orbit per planet.
    pub orbit: Vec<Vec<u16>>,
    pub spyreportfield: Vec<Vec<u8>>,
    pub spyreportyear: Vec<u16>,
    pub shipi_colony: i8,
    pub shipi_bomber: i8,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Design {
    pub name: String,
    pub cost: u16,
    pub space: u16,
    pub hull: u8,
    pub look: u8,
    pub wpnt: Vec<u8>,
    pub wpnn: Vec<u8>,
    pub engine: u8,
    pub engines: u32,
    pub special: Vec<u8>,
    pub shield: u8,
    pub jammer: u8,
    pub comp: u8,
    pub armor: u8,
    pub man: u8,
    pub hp: u16,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Srd {
    pub design: Vec<Design>,
    /// [field][tier][3]
    pub researchlist: Vec<Vec<Vec<u8>>>,
    /// [field][60]
    pub researchcompleted: Vec<Vec<u8>>,
    pub year: Vec<u16>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Monster {
    pub exists: u8,
    pub x: i16,
    pub y: i16,
    pub killer: u8,
    pub dest: u8,
    pub counter: i8,
    pub nuked: i8,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Evn {
    pub year: u16,
    pub done: Vec<bool>,
    pub diplo_msg_subtype: i8,
    pub have_plague: u8,
    pub plague_player: u8,
    pub plague_planet_i: u8,
    pub plague_val: u32,
    pub have_nova: u8,
    pub nova_player: u8,
    pub nova_planet_i: u8,
    pub nova_years: i8,
    pub nova_val: u32,
    pub have_accident: u8,
    pub accident_planet_i: u8,
    pub have_comet: u8,
    pub comet_player: u8,
    pub comet_planet_i: u8,
    pub comet_years: u8,
    pub comet_hp: u16,
    pub comet_dmg: u16,
    pub have_pirates: u8,
    pub pirates_planet_i: u8,
    pub pirates_hp: u16,
    pub crystal: Monster,
    pub amoeba: Monster,
    pub planet_orion_i: u8,
    pub have_guardian: u8,
    pub home: Vec<u8>,
    pub report_stars: u8,
    pub new_ships: Vec<Vec<u32>>,
    pub spies_caught: Vec<Vec<u16>>,
    pub ceasefire: Vec<Vec<i16>>,
    pub build_finished_num: Vec<u16>,
    pub voted: Vec<u8>,
    pub best_ecorestore: Vec<u8>,
    pub best_wastereduce: Vec<u8>,
    pub best_roboctrl: Vec<u8>,
    pub best_terraform: Vec<u8>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Game {
    pub save_name: String,
    pub players: u8,
    pub is_ai: Vec<bool>,
    pub refuse: Vec<bool>,
    pub ai_id: u8,
    pub active_player: u8,
    pub difficulty: u8,
    pub galaxy_size: u8,
    pub galaxy_w: u8,
    pub galaxy_h: u8,
    pub galaxy_stars: u8,
    pub galaxy_maxx: u16,
    pub galaxy_maxy: u16,
    pub galaxy_seed: u32,
    pub seed: u32,
    pub year: u16,
    pub end: u8,
    pub winner: u8,
    pub election_held: u8,
    pub nebula_x: Vec<u16>,
    pub nebula_y: Vec<u16>,
    pub nebula_x0: Vec<Vec<u16>>,
    pub nebula_x1: Vec<Vec<u16>>,
    pub nebula_y0: Vec<Vec<u16>>,
    pub nebula_y1: Vec<Vec<u16>>,
    pub emperor_names: Vec<String>,
    pub planet_focus_i: Vec<u8>,
    pub planet: Vec<Planet>,
    /// [player][planet]
    pub seen: Vec<Vec<Seen>>,
    pub enroute: Vec<Fleet>,
    pub transport: Vec<Transport>,
    pub eto: Vec<Eto>,
    pub srd: Vec<Srd>,
    pub current_design: Vec<Design>,
    pub evn: Evn,
}

fn planet(r: &mut Rd) -> Planet {
    Planet {
        name: r.str(12),
        x: r.u16(),
        y: r.u16(),
        star_type: r.u8(),
        look: r.u8(),
        frame: r.u8(),
        rocks: r.u8(),
        max_pop1: r.i16(),
        max_pop2: r.i16(),
        max_pop3: r.i16(),
        kind: r.u8(),
        battlebg: r.u8(),
        infogfx: r.u8(),
        growth: r.u8(),
        special: r.u8(),
        bc_to_ecoproj: r.i16(),
        bc_to_ship: r.u16(),
        bc_to_factory: r.u16(),
        reserve: r.u32(),
        waste: r.i16(),
        owner: r.u8(),
        prev_owner: r.u8(),
        claim: r.u8(),
        pop: r.i16(),
        pop_prev: r.i16(),
        factories: r.i16(),
        slider: r.ti16(5),
        slider_lock: r.tu8(5),
        buildship: r.u8(),
        reloc: r.u8(),
        missile_bases: r.u16(),
        bc_to_base: r.u16(),
        bc_upgrade_base: r.u16(),
        have_stargate: r.u8(),
        shield: r.u8(),
        bc_to_shield: r.u16(),
        trans_num: r.u16(),
        trans_dest: r.u8(),
        pop_tenths: r.i8(),
        explored: r.bv(PLAYER_NUM),
        unrefuel: {
            let v = r.bv(PLAYER_NUM);
            r.skip(4);
            v
        },
        pop_oper_fact: r.u8(),
        bc_to_refit: r.u16(),
        rebels: r.i16(),
        unrest: r.u8(),
        unrest_reported: r.u8(),
        finished: {
            let v = r.bv(6);
            r.skip(5);
            v
        },
    }
}

fn design(r: &mut Rd) -> Design {
    Design {
        name: r.str(12),
        cost: r.u16(),
        space: r.u16(),
        hull: r.u8(),
        look: r.u8(),
        wpnt: r.tu8(4),
        wpnn: r.tu8(4),
        engine: r.u8(),
        engines: r.u32(),
        special: r.tu8(3),
        shield: r.u8(),
        jammer: r.u8(),
        comp: r.u8(),
        armor: r.u8(),
        man: r.u8(),
        hp: r.u16(),
    }
}

fn eto(r: &mut Rd, pnum: usize, planets: usize) -> Result<Eto, String> {
    let mut e = Eto {
        race: r.u8(),
        banner: r.u8(),
        trait1: r.u8(),
        trait2: r.u8(),
        ai_p3_countdown: r.i8(),
        ai_p2_countdown: r.i8(),
        contact: r.bv(PLAYER_NUM),
        contact_broken: r.bv(PLAYER_NUM),
        ..Default::default()
    };
    r.skip(4);
    e.relation1 = r.ti16(pnum);
    e.relation2 = r.ti16(pnum);
    e.diplo_type = r.tu8(pnum);
    e.diplo_val = r.ti16(pnum);
    e.diplo_p1 = r.tu16(pnum);
    e.diplo_p2 = r.ti16(pnum);
    e.trust = r.ti16(pnum);
    e.broken_treaty = r.tu8(pnum);
    e.blunder = r.ti16(pnum);
    e.tribute_field = r.tu8(pnum);
    e.tribute_tech = r.tu8(pnum);
    e.mood_treaty = r.ti16(pnum);
    e.mood_trade = r.ti16(pnum);
    e.mood_tech = r.ti16(pnum);
    e.mood_peace = r.ti16(pnum);
    e.treaty = r.tu8(pnum);
    e.trade_bc = r.tu16(pnum);
    e.trade_percent = r.ti16(pnum);
    e.spymode_next = r.tu8(pnum);
    e.offer_field = r.tu8(pnum);
    e.offer_tech = r.tu8(pnum);
    e.offer_bc = r.tu16(pnum);
    let ab = r.tu16(pnum);
    let bc = r.tu16(pnum);
    e.attack_bounty = vec![PLAYER_NONE; pnum];
    e.bounty_collect = vec![PLAYER_NONE; pnum];
    e.attack_gift_field = vec![0; pnum];
    e.attack_gift_tech = vec![0; pnum];
    e.attack_gift_bc = vec![0; pnum];
    if bc[0] != PLAYER_NONE as u16 {
        for i in 0..pnum {
            e.attack_bounty[i] = (bc[i] >> 8) as u8 & 0xf;
            e.bounty_collect[i] = bc[i] as u8 & 0xf;
            let v = ab[i];
            if v & 1 != 0 {
                e.attack_gift_tech[i] = (v >> 8) as u8;
                e.attack_gift_field[i] = ((v >> 1) & 0xf) as u8;
            } else {
                e.attack_gift_bc[i] = v as i16;
            }
        }
    } else {
        for i in 0..pnum {
            e.attack_bounty[i] = ab[i] as u8;
            e.bounty_collect[i] = bc[i] as u8;
        }
    }
    e.hatred = r.ti16(pnum);
    e.have_met = r.tu16(pnum);
    e.trade_established_bc = r.tu16(pnum);
    e.spying = r.ti16(pnum);
    e.spyfund = r.tu16(pnum);
    e.spymode = r.tu8(pnum);
    e.security = r.i16();
    e.spies = r.tu16(pnum);
    e.reserve_bc = r.u32();
    e.tax = r.i16();
    e.base_shield = r.u8();
    e.base_comp = r.u8();
    e.base_weapon = r.u8();
    e.colonist_oper_factories = r.u8();
    e.tech_percent = r.tu8(6);
    e.tech_slider = r.ti16(6);
    e.tech_slider_lock = r.tu8(6);
    e.tech_investment = r.tu32(6);
    e.tech_project = r.tu8(6);
    e.tech_cost = r.tu32(6);
    e.tech_completed = r.tu16(6);
    e.shipdesigns_num = r.u8();
    if e.shipdesigns_num as usize > NUM_SHIPDESIGNS {
        return Err("invalid number of ship designs".into());
    }
    e.orbit = vec![vec![0; NUM_SHIPDESIGNS]; planets];
    loop {
        let i = r.u8();
        if i == PLANET_NONE {
            break;
        }
        if i as usize >= planets {
            return Err(format!("orbit planet {i} >= {planets}"));
        }
        e.orbit[i as usize] = r.tu16(NUM_SHIPDESIGNS);
    }
    e.spyreportfield = (0..pnum).map(|_| r.tu8(6)).collect();
    e.spyreportyear = r.tu16(pnum);
    e.shipi_colony = r.i8();
    e.shipi_bomber = r.i8();
    Ok(e)
}

fn monster(r: &mut Rd) -> Monster {
    Monster {
        exists: r.u8(),
        x: r.i16(),
        y: r.i16(),
        killer: r.u8(),
        dest: r.u8(),
        counter: r.i8(),
        nuked: r.i8(),
    }
}

/// Reads a 1oom save file (with its 64-byte header).
pub fn load(data: &[u8]) -> Result<Game, String> {
    if data.len() < 64 + 512 || &data[..8] != b"1oomSAVE" {
        return Err("not a 1oom save".into());
    }
    let version = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    if version != 0 {
        return Err(format!("unsupported 1oom save version {version}"));
    }
    let save_name = Rd { b: data, pos: 16 }.str(32);
    let mut r = Rd {
        b: &data[64..],
        pos: 0,
    };
    let mut g = Game {
        save_name,
        players: r.u8(),
        ..Default::default()
    };
    let pnum = g.players as usize;
    if !(2..=6).contains(&pnum) {
        return Err(format!("invalid number of players {pnum}"));
    }
    g.is_ai = r.bv(PLAYER_NUM);
    g.refuse = r.bv(PLAYER_NUM);
    g.ai_id = r.u8();
    r.skip(3);
    g.active_player = r.u8();
    g.difficulty = r.u8();
    g.galaxy_size = r.u8();
    g.galaxy_w = r.u8();
    g.galaxy_h = r.u8();
    g.galaxy_stars = r.u8();
    g.galaxy_maxx = r.u16();
    g.galaxy_maxy = r.u16();
    g.galaxy_seed = r.u32();
    g.seed = r.u32();
    g.year = r.u16();
    let enroute_num = r.u16() as usize;
    let transport_num = r.u16() as usize;
    g.end = r.u8();
    g.winner = r.u8();
    g.election_held = r.u8();
    let nebula_num = r.u8() as usize;
    g.nebula_x = r.tu16(nebula_num);
    g.nebula_y = r.tu16(nebula_num);
    g.nebula_x0 = (0..nebula_num).map(|_| r.tu16(4)).collect();
    g.nebula_x1 = (0..nebula_num).map(|_| r.tu16(4)).collect();
    g.nebula_y0 = (0..nebula_num).map(|_| r.tu16(4)).collect();
    g.nebula_y1 = (0..nebula_num).map(|_| r.tu16(4)).collect();
    g.emperor_names = (0..pnum).map(|_| r.str(15)).collect();
    g.planet_focus_i = r.tu8(pnum);
    let stars = g.galaxy_stars as usize;
    g.planet = (0..stars).map(|_| planet(&mut r)).collect();
    g.seen = (0..pnum)
        .map(|_| {
            (0..stars)
                .map(|_| Seen {
                    owner: r.u8(),
                    pop: r.u16(),
                    bases: r.u16(),
                    factories: r.u16(),
                })
                .collect()
        })
        .collect();
    g.enroute = (0..enroute_num)
        .map(|_| {
            let owner = r.u8();
            let x = r.u16();
            let y = r.u16();
            let dest = r.u8();
            let v = r.u8();
            Fleet {
                owner,
                x,
                y,
                dest,
                speed: v & 0x7f,
                retreat: v & 0x80 != 0,
                ships: r.tu16(NUM_SHIPDESIGNS),
            }
        })
        .collect();
    g.transport = (0..transport_num)
        .map(|_| Transport {
            owner: r.u8(),
            x: r.u16(),
            y: r.u16(),
            dest: r.u8(),
            speed: r.u8(),
            pop: r.u16(),
        })
        .collect();
    for _ in 0..pnum {
        let e = eto(&mut r, pnum, stars)?;
        g.eto.push(e);
    }
    for i in 0..pnum {
        let sdnum = g.eto[i].shipdesigns_num as usize;
        let mut s = Srd {
            design: (0..sdnum).map(|_| design(&mut r)).collect(),
            ..Default::default()
        };
        s.researchlist = (0..6)
            .map(|_| (0..10).map(|_| r.tu8(3)).collect())
            .collect();
        s.researchcompleted = (0..6).map(|_| r.tu8(60)).collect();
        r.skip(NUM_SHIPDESIGNS);
        s.year = r.tu16(NUM_SHIPDESIGNS);
        r.skip(NUM_SHIPDESIGNS * 4);
        g.srd.push(s);
    }
    g.current_design = (0..pnum).map(|_| design(&mut r)).collect();
    let ev = &mut g.evn;
    ev.year = r.u16();
    ev.done = r.bv(20);
    ev.diplo_msg_subtype = r.i8();
    r.skip(16);
    ev.have_plague = r.u8();
    ev.plague_player = r.u8();
    ev.plague_planet_i = r.u8();
    ev.plague_val = r.u32();
    ev.have_nova = r.u8();
    ev.nova_player = r.u8();
    ev.nova_planet_i = r.u8();
    ev.nova_years = r.i8();
    ev.nova_val = r.u32();
    ev.have_accident = r.u8();
    ev.accident_planet_i = r.u8();
    ev.have_comet = r.u8();
    ev.comet_player = r.u8();
    ev.comet_planet_i = r.u8();
    ev.comet_years = r.u8();
    ev.comet_hp = r.u16();
    ev.comet_dmg = r.u16();
    ev.have_pirates = r.u8();
    ev.pirates_planet_i = r.u8();
    ev.pirates_hp = r.u16();
    ev.crystal = monster(&mut r);
    ev.amoeba = monster(&mut r);
    ev.planet_orion_i = r.u8();
    ev.have_guardian = r.u8();
    ev.home = r.tu8(pnum);
    ev.report_stars = r.u8();
    ev.new_ships = (0..pnum).map(|_| r.tu32(NUM_SHIPDESIGNS)).collect();
    ev.spies_caught = (0..pnum).map(|_| r.tu16(pnum)).collect();
    ev.ceasefire = (0..pnum).map(|_| r.ti16(pnum)).collect();
    for _ in 0..pnum {
        r.bv(16);
        r.skip(14);
    }
    ev.build_finished_num = r.tu16(pnum);
    ev.voted = r.tu8(pnum);
    ev.best_ecorestore = r.tu8(pnum);
    ev.best_wastereduce = r.tu8(pnum);
    ev.best_roboctrl = r.tu8(pnum);
    ev.best_terraform = r.tu8(pnum);
    if r.u32() != 0x646e_450a {
        return Err(format!("invalid end mark at {}", r.pos));
    }
    Ok(g)
}
