//! A 1oom save as an Age of Orion state: the same game, field by field.

use aoo_core::council::Council;
use aoo_core::design::ShipDesign;
use aoo_core::diplo::{Relation, SpyMode, Treaty};
use aoo_core::events::{EventState, Lasting, Monster, MonsterKind};
use aoo_core::*;

use crate::oom::{self, PLANET_NONE, PLAYER_NONE};

fn owner(v: u8) -> Option<EmpireId> {
    (v != PLAYER_NONE && v < 6).then_some(v)
}

fn treaty(v: u8) -> Treaty {
    match v {
        1 => Treaty::NonAggression,
        2 => Treaty::Alliance,
        3 => Treaty::War,
        4 => Treaty::FinalWar,
        _ => Treaty::None,
    }
}

fn spymode(v: u8) -> SpyMode {
    match v {
        1 => SpyMode::Espionage,
        2 => SpyMode::Sabotage,
        _ => SpyMode::Hide,
    }
}

fn tech_ref(field: u8, tech: u8) -> Option<(u8, u8)> {
    (tech != 0).then_some((field, tech))
}

pub fn design(d: &oom::Design) -> ShipDesign {
    let mut weapons = [0u8; 4];
    let mut counts = [0u8; 4];
    weapons.copy_from_slice(&d.wpnt);
    counts.copy_from_slice(&d.wpnn);
    let mut specials = [0u8; 3];
    specials.copy_from_slice(&d.special);
    ShipDesign {
        name: d.name.clone(),
        hull: d.hull,
        look: d.look,
        weapons,
        weapon_counts: counts,
        engine: d.engine,
        engines: d.engines,
        specials,
        shield: d.shield,
        jammer: d.jammer,
        comp: d.comp,
        armor: d.armor,
        man: d.man,
        hp: d.hp as u32,
        cost_bc: d.cost as u32,
        space: d.space as i16 as i32,
    }
}

fn lasting(planet: u8, player: u8, val: i32, years: i32, damage: i32, stage: u8) -> Lasting {
    Lasting {
        planet: planet as usize,
        player: player.min(5),
        val,
        years,
        damage,
        stage,
    }
}

/// Converts a decoded 1oom save. Values that 1oom derives on loading (production,
/// ranges, abilities) are left for `Game::from_state` to compute, as 1oom does.
pub fn state(g: &oom::Game) -> State {
    let n = g.players as usize;
    let np = g.galaxy_stars as usize;
    let mut base = Game::new(&NewGameOptions {
        seed: 1,
        players: n as u8,
        all_ai: true,
        ..Default::default()
    })
    .expect("template game")
    .state()
    .clone();
    let planets: Vec<Planet> = g
        .planet
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let mut q = base.planets[0].clone();
            q.name = p.name.clone();
            q.x = p.x as i32;
            q.y = p.y as i32;
            q.star = match p.star_type {
                1 => StarType::Red,
                2 => StarType::Green,
                3 => StarType::White,
                4 => StarType::Blue,
                5 => StarType::Neutron,
                _ => StarType::Yellow,
            };
            q.kind = PlanetType::from_index(p.kind as usize);
            q.special = match p.special {
                0 => Special::UltraPoor,
                1 => Special::Poor,
                3 => Special::Artifacts,
                4 => Special::Rich,
                5 => Special::UltraRich,
                6 => Special::Orion,
                _ => Special::Normal,
            };
            q.growth = match p.growth {
                0 => Growth::Hostile,
                2 => Growth::Fertile,
                3 => Growth::Gaia,
                _ => Growth::Normal,
            };
            q.image = p.infogfx;
            q.size = p.max_pop1 as i32;
            q.base_max_pop = p.max_pop2 as i32;
            q.max_pop = p.max_pop3 as i32;
            q.nebula = p.battlebg == 0;
            q.owner = owner(p.owner);
            q.pop = p.pop as i32;
            q.pop_tenths = p.pop_tenths as i32;
            q.factories = p.factories as i32;
            q.waste = p.waste as i32;
            q.bases = p.missile_bases as i32;
            q.shield = p.shield as i32;
            q.stargate = p.have_stargate != 0;
            q.pop_oper_fact = p.pop_oper_fact as i32;
            for k in 0..5 {
                q.sliders[k] = p.slider[k] as i32;
                q.slider_lock[k] = p.slider_lock[k] != 0;
            }
            q.build = p.buildship as usize;
            q.rally = (p.reloc as usize != i).then_some(p.reloc as usize);
            q.prod = 0;
            q.bc_to_ship = p.bc_to_ship as i32;
            q.bc_to_factory = p.bc_to_factory as i32;
            q.bc_to_base = p.bc_to_base as i32;
            q.bc_to_shield = p.bc_to_shield as i32;
            q.bc_to_refit = p.bc_to_refit as i32;
            q.bc_to_ecoproj = p.bc_to_ecoproj as i32;
            q.reserve = p.reserve as i32;
            q.rebels = p.rebels as i32;
            q.unrest = match p.unrest {
                3 => Unrest::Rebellion,
                4 => Unrest::Resolved,
                _ => Unrest::Normal,
            };
            q.unrest_reported = p.unrest_reported != 0;
            q.claim = owner(p.claim);
            q.rocks = p.rocks;
            q.look = p.look;
            q.frame = p.frame;
            q.battlebg = p.battlebg;
            q.prev_owner = owner(p.prev_owner);
            q.pop_prev = p.pop_prev as i32;
            q.bc_upgrade_base = p.bc_upgrade_base as i32;
            q.trans_num = p.trans_num as i32;
            q.trans_dest = p.trans_dest as usize;
            q.unrefuel = p.unrefuel.clone();
            for k in 0..6 {
                q.finished[k] = p.finished[k];
            }
            q
        })
        .collect();
    let mut empires = Vec::with_capacity(n);
    for i in 0..n {
        let e = &g.eto[i];
        let s = &g.srd[i];
        let mut emp = base.empires[i.min(base.empires.len() - 1)].clone();
        emp.leader = g.emperor_names[i].clone();
        emp.race = Race::ALL[e.race as usize];
        emp.banner = e.banner;
        emp.ship_names = aoo_core::SHIP_NAMES[e.race as usize]
            .iter()
            .map(|s| s.to_uppercase())
            .collect();
        emp.ai = g.is_ai[i];
        emp.alive = g.evn.home[i] != PLANET_NONE;
        emp.home = if emp.alive { g.evn.home[i] as usize } else { 0 };
        let mut tech = Tech::new();
        for f in 0..6 {
            tech.completed[f] = s.researchcompleted[f][..e.tech_completed[f] as usize].to_vec();
            tech.project[f] = e.tech_project[f];
            tech.cost[f] = e.tech_cost[f];
            tech.invested[f] = e.tech_investment[f];
            tech.alloc[f] = e.tech_slider[f] as u32;
            tech.level[f] = e.tech_percent[f] as u32;
            for t in 0..10 {
                for k in 0..3 {
                    tech.list[f][t][k] = s.researchlist[f][t][k];
                }
            }
        }
        emp.tech = tech;
        emp.designs = s.design.iter().map(design).collect();
        emp.design_year = s.year.iter().map(|&y| y as i32).collect();
        emp.reserve = e.reserve_bc as i32;
        emp.orbit = e
            .orbit
            .iter()
            .map(|o| o.iter().map(|&v| v as u32).collect())
            .collect();
        emp.explored = (0..np).map(|p| g.planet[p].explored[i]).collect();
        emp.trait1 = e.trait1;
        emp.trait2 = e.trait2;
        emp.security = e.security as i32;
        emp.tax = e.tax as i32;
        emp.base_shield = e.base_shield as usize;
        emp.base_comp = e.base_comp as usize;
        emp.base_weapon = e.base_weapon as usize;
        emp.ai_p2_countdown = e.ai_p2_countdown as i32;
        emp.ai_p3_countdown = e.ai_p3_countdown as i32;
        emp.shipi_colony = e.shipi_colony as i32;
        emp.shipi_bomber = e.shipi_bomber as i32;
        emp.seen = g.seen[i]
            .iter()
            .map(|s| Seen {
                owner: owner(s.owner),
                pop: s.pop as i32,
                bases: s.bases as i32,
                factories: s.factories as i32,
            })
            .collect();
        emp.rel = (0..n)
            .map(|j| Relation {
                contact: e.contact[j],
                contact_broken: e.contact_broken[j],
                have_met: e.have_met[j] as u8,
                relation: e.relation1[j] as i32,
                base_relation: e.relation2[j] as i32,
                trust: e.trust[j] as i32,
                treaty: treaty(e.treaty[j]),
                broken_treaty: treaty(e.broken_treaty[j]),
                blunder: e.blunder[j] as u8,
                mood_treaty: e.mood_treaty[j] as i32,
                mood_trade: e.mood_trade[j] as i32,
                mood_tech: e.mood_tech[j] as i32,
                mood_peace: e.mood_peace[j] as i32,
                trade_bc: e.trade_bc[j] as i32,
                trade_established: e.trade_established_bc[j] as i32,
                trade_percent: e.trade_percent[j] as i32,
                hatred: e.hatred[j] as i32,
                diplo_type: e.diplo_type[j],
                diplo_val: e.diplo_val[j] as i32,
                diplo_p1: e.diplo_p1[j] as usize,
                diplo_p2: e.diplo_p2[j] as i32,
                tribute: tech_ref(e.tribute_field[j], e.tribute_tech[j]),
                offer_bc: e.offer_bc[j] as i32,
                offer_tech: tech_ref(e.offer_field[j], e.offer_tech[j]),
                attack_bounty: owner(e.attack_bounty[j]),
                bounty_collect: owner(e.bounty_collect[j]),
                attack_gift_tech: tech_ref(e.attack_gift_field[j], e.attack_gift_tech[j]),
                attack_gift_bc: e.attack_gift_bc[j] as i32,
                spying: e.spying[j] as i32,
                spyfund: e.spyfund[j] as i32,
                spymode: spymode(e.spymode[j]),
                spymode_next: spymode(e.spymode_next[j]),
                spies: e.spies[j] as i32,
                spy_report_year: e.spyreportyear[j] as u32,
                spy_report: {
                    let mut r = [0u8; 6];
                    r.copy_from_slice(&e.spyreportfield[j]);
                    r
                },
                ceasefire: g.evn.ceasefire[i][j] as i32,
                ..Default::default()
            })
            .collect();
        empires.push(emp);
    }
    let fleets: Vec<Fleet> = g
        .enroute
        .iter()
        .enumerate()
        .map(|(k, f)| Fleet {
            id: k as u32 + 1,
            owner: f.owner,
            x: f.x as i32,
            y: f.y as i32,
            dest: f.dest as usize,
            ships: f.ships.iter().map(|&v| v as u32).collect(),
            speed: f.speed as i32,
            retreat: f.retreat,
            visible: Vec::new(),
        })
        .collect();
    let transports: Vec<Transport> = g
        .transport
        .iter()
        .enumerate()
        .map(|(k, t)| Transport {
            id: (g.enroute.len() + k) as u32 + 1,
            owner: t.owner,
            x: t.x as i32,
            y: t.y as i32,
            dest: t.dest as usize,
            pop: t.pop as i32,
            speed: t.speed as i32,
            visible: Vec::new(),
        })
        .collect();
    let ev = &g.evn;
    let mut evn = EventState::new();
    evn.last_year = ev.year as i32;
    evn.done = ev.done.clone();
    evn.report_stars = ev.report_stars;
    if ev.have_plague != 0 {
        evn.plague = Some(lasting(
            ev.plague_planet_i,
            ev.plague_player,
            ev.plague_val as i32,
            0,
            0,
            ev.have_plague,
        ));
    }
    if ev.have_nova != 0 {
        evn.nova = Some(lasting(
            ev.nova_planet_i,
            ev.nova_player,
            ev.nova_val as i32,
            ev.nova_years as i32,
            0,
            ev.have_nova,
        ));
    }
    if ev.have_accident != 0 {
        evn.accident = Some(lasting(ev.accident_planet_i, 0, 0, 0, 0, ev.have_accident));
    }
    if ev.have_comet != 0 {
        evn.comet = Some(lasting(
            ev.comet_planet_i,
            ev.comet_player,
            ev.comet_hp as i32,
            ev.comet_years as i32,
            ev.comet_dmg as i32,
            ev.have_comet,
        ));
    }
    if ev.have_pirates != 0 {
        evn.pirates = Some(lasting(
            ev.pirates_planet_i,
            0,
            ev.pirates_hp as i32,
            0,
            0,
            ev.have_pirates,
        ));
    }
    for (kind, m) in [
        (MonsterKind::Crystal, &ev.crystal),
        (MonsterKind::Amoeba, &ev.amoeba),
    ] {
        if m.exists != 0 {
            evn.monsters.push(Monster {
                kind,
                x: m.x as i32,
                y: m.y as i32,
                dest: m.dest as usize,
                counter: m.counter as i32,
                nuked: m.nuked as i32,
                killer: owner(m.killer),
                hp: aoo_core::events::monster_stats(kind, Difficulty::ALL[g.difficulty as usize]).0,
            });
        }
    }
    base.version = SAVE_VERSION;
    base.rng = Rng(g.seed);
    base.seed = g.galaxy_seed;
    base.year = 2299 + g.year as u32;
    base.turn = g.year as u32 - 1;
    base.difficulty = Difficulty::ALL[g.difficulty as usize];
    base.galaxy_size = GalaxySize::ALL[g.galaxy_size as usize];
    base.width = g.galaxy_maxx as i32;
    base.height = g.galaxy_maxy as i32;
    base.nebulae = (0..g.nebula_x.len())
        .map(|k| Nebula {
            kind: 0,
            x: g.nebula_x[k] as i32,
            y: g.nebula_y[k] as i32,
            rects: (0..4)
                .map(|r| {
                    [
                        g.nebula_x0[k][r] as i32,
                        g.nebula_y0[k][r] as i32,
                        g.nebula_x1[k][r] as i32,
                        g.nebula_y1[k][r] as i32,
                    ]
                })
                .collect(),
        })
        .collect();
    base.planets = planets;
    base.empires = empires;
    base.fleets = fleets;
    base.transports = transports;
    base.orion = ev.planet_orion_i as usize;
    base.guardian = ev.have_guardian != 0;
    base.winner = None;
    base.next_id = (g.enroute.len() + g.transport.len()) as u32 + 1;
    base.events = Vec::new();
    base.council = Council {
        held: g.election_held != 0,
        final_war: g.end == 3,
        refuse: g.refuse[..n].to_vec(),
        voted: ev.voted.iter().map(|&v| owner(v)).collect(),
        winner: None,
        pending: None,
    };
    base.evn = evn;
    base.ai_id = g.ai_id;
    base.galaxy_seed = g.galaxy_seed;
    base.galaxy_w = g.galaxy_w as i32;
    base.galaxy_h = g.galaxy_h as i32;
    base.battle = None;
    base.progress = None;
    base.pending = Default::default();
    base
}
