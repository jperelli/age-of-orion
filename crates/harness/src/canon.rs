//! The rule-relevant state of both games as (key, value) pairs with the same keys, so
//! that a 1oom save and an Age of Orion state can be compared field by field.

use aoo_core::diplo::{SpyMode, Treaty};
use aoo_core::*;

use crate::oom;

pub type Canon = Vec<(String, String)>;

struct C(Canon);

impl C {
    fn put(&mut self, k: impl Into<String>, v: impl ToString) {
        self.0.push((k.into(), v.to_string()));
    }
}

fn opt(v: Option<u8>) -> i32 {
    v.map_or(-1, |x| x as i32)
}

fn oo(v: u8) -> i32 {
    if v >= 6 {
        -1
    } else {
        v as i32
    }
}

fn treaty_n(t: Treaty) -> i32 {
    match t {
        Treaty::None => 0,
        Treaty::NonAggression => 1,
        Treaty::Alliance => 2,
        Treaty::War => 3,
        Treaty::FinalWar => 4,
    }
}

fn spy_n(m: SpyMode) -> i32 {
    match m {
        SpyMode::Hide => 0,
        SpyMode::Espionage => 1,
        SpyMode::Sabotage => 2,
    }
}

fn list<T: ToString>(v: impl IntoIterator<Item = T>) -> String {
    v.into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[allow(clippy::too_many_arguments)]
fn design_key(
    c: &mut C,
    k: &str,
    name: &str,
    hull: u8,
    look: u8,
    w: &[u8],
    wn: &[u8],
    engine: u8,
    engines: u32,
    sp: &[u8],
    shield: u8,
    jammer: u8,
    comp: u8,
    armor: u8,
    man: u8,
    hp: u32,
    cost: u32,
) {
    c.put(
        k,
        format!(
            "{}|h{hull} l{look} w{} n{} e{engine}x{engines} s{} sh{shield} j{jammer} c{comp} a{armor} m{man} hp{hp} ${cost}",
            name.to_uppercase(),
            list(w.iter().copied()),
            list(wn.iter().copied()),
            list(sp.iter().copied())
        ),
    );
}

pub fn from_oom(g: &oom::Game) -> Canon {
    let mut c = C(Vec::new());
    let n = g.players as usize;
    c.put("seed", g.seed);
    c.put("year", g.year);
    for (i, p) in g.planet.iter().enumerate() {
        let k = format!("p{i}.");
        c.put(k.clone() + "owner", oo(p.owner));
        c.put(k.clone() + "pop", p.pop);
        c.put(k.clone() + "pop_tenths", p.pop_tenths);
        c.put(k.clone() + "factories", p.factories);
        c.put(k.clone() + "waste", p.waste);
        c.put(k.clone() + "bases", p.missile_bases);
        c.put(k.clone() + "shield", p.shield);
        c.put(
            k.clone() + "max_pop",
            format!("{}/{}/{}", p.max_pop1, p.max_pop2, p.max_pop3),
        );
        c.put(k.clone() + "kind", format!("{}/{}", p.kind, p.growth));
        c.put(k.clone() + "sliders", list(p.slider.iter().copied()));
        c.put(k.clone() + "build", p.buildship);
        c.put(k.clone() + "reloc", p.reloc);
        c.put(
            k.clone() + "bc",
            format!(
                "ship{} fact{} base{} up{} sh{} refit{} eco{}",
                p.bc_to_ship,
                p.bc_to_factory,
                p.bc_to_base,
                p.bc_upgrade_base,
                p.bc_to_shield,
                p.bc_to_refit,
                p.bc_to_ecoproj
            ),
        );
        c.put(k.clone() + "reserve", p.reserve);
        c.put(k.clone() + "unrest", format!("{}/{}", p.unrest, p.rebels));
        c.put(
            k.clone() + "trans",
            format!(
                "{}->{}",
                p.trans_num,
                if p.trans_num > 0 {
                    p.trans_dest as i32
                } else {
                    -1
                }
            ),
        );
        c.put(k.clone() + "pop_oper_fact", p.pop_oper_fact);
        c.put(k.clone() + "stargate", p.have_stargate);
        c.put(
            k.clone() + "explored",
            list((0..n).map(|j| p.explored[j] as u8)),
        );
        c.put(k.clone() + "claim", oo(p.claim));
        c.put(k + "pop_prev", p.pop_prev);
    }
    for i in 0..n {
        let e = &g.eto[i];
        let s = &g.srd[i];
        let k = format!("e{i}.");
        c.put(
            k.clone() + "home",
            if g.evn.home[i] == oom::PLANET_NONE {
                -1
            } else {
                g.evn.home[i] as i32
            },
        );
        c.put(k.clone() + "reserve", e.reserve_bc);
        c.put(k.clone() + "tax", e.tax);
        c.put(k.clone() + "security", e.security);
        c.put(
            k.clone() + "countdown",
            format!("{}/{}", e.ai_p2_countdown, e.ai_p3_countdown),
        );
        c.put(
            k.clone() + "shipi",
            format!("{}/{}", e.shipi_colony, e.shipi_bomber),
        );
        c.put(
            k.clone() + "base_parts",
            format!("{}/{}/{}", e.base_shield, e.base_comp, e.base_weapon),
        );
        for f in 0..6 {
            let kf = format!("{k}t{f}.");
            c.put(
                kf.clone() + "completed",
                list(
                    s.researchcompleted[f][..e.tech_completed[f] as usize]
                        .iter()
                        .copied(),
                ),
            );
            c.put(
                kf.clone() + "project",
                format!("{} ${}", e.tech_project[f], e.tech_cost[f]),
            );
            c.put(kf.clone() + "invest", e.tech_investment[f]);
            c.put(kf.clone() + "slider", e.tech_slider[f]);
            c.put(kf + "percent", e.tech_percent[f]);
        }
        for (d, sd) in s.design.iter().enumerate() {
            design_key(
                &mut c,
                &format!("{k}d{d}"),
                &sd.name,
                sd.hull,
                sd.look,
                &sd.wpnt,
                &sd.wpnn,
                sd.engine,
                sd.engines,
                &sd.special,
                sd.shield,
                sd.jammer,
                sd.comp,
                sd.armor,
                sd.man,
                sd.hp as u32,
                sd.cost as u32,
            );
            c.put(format!("{k}d{d}.year"), s.year[d]);
        }
        c.put(k.clone() + "designs", s.design.len());
        let orbits: Vec<String> = e
            .orbit
            .iter()
            .enumerate()
            .filter(|(_, o)| o.iter().any(|&x| x != 0))
            .map(|(p, o)| format!("{p}:{}", list(o.iter().copied())))
            .collect();
        c.put(k.clone() + "orbit", orbits.join(" "));
        for j in 0..n {
            let kr = format!("{k}r{j}.");
            c.put(
                kr.clone() + "rel",
                format!(
                    "{}/{} trust{} hatred{} blunder{}",
                    e.relation1[j], e.relation2[j], e.trust[j], e.hatred[j], e.blunder[j]
                ),
            );
            c.put(
                kr.clone() + "treaty",
                format!("{} broken{}", e.treaty[j], e.broken_treaty[j]),
            );
            c.put(
                kr.clone() + "mood",
                format!(
                    "{}/{}/{}/{}",
                    e.mood_treaty[j], e.mood_trade[j], e.mood_tech[j], e.mood_peace[j]
                ),
            );
            c.put(
                kr.clone() + "trade",
                format!(
                    "{} {}% est{}",
                    e.trade_bc[j], e.trade_percent[j], e.trade_established_bc[j]
                ),
            );
            c.put(
                kr.clone() + "spy",
                format!(
                    "{} mode{} next{} spies{} fund{}",
                    e.spying[j], e.spymode[j], e.spymode_next[j], e.spies[j], e.spyfund[j]
                ),
            );
            c.put(
                kr.clone() + "contact",
                format!("{} met{}", e.contact[j] as u8, e.have_met[j]),
            );
            c.put(kr + "ceasefire", g.evn.ceasefire[i][j]);
        }
    }
    for (k, f) in g.enroute.iter().enumerate() {
        c.put(
            format!("fleet{k}"),
            format!(
                "o{} ({},{})->{} v{} [{}]",
                f.owner,
                f.x,
                f.y,
                f.dest,
                f.speed,
                list(f.ships.iter().copied())
            ),
        );
    }
    c.put("fleets", g.enroute.len());
    for (k, t) in g.transport.iter().enumerate() {
        c.put(
            format!("trans{k}"),
            format!(
                "o{} ({},{})->{} v{} pop{}",
                t.owner, t.x, t.y, t.dest, t.speed, t.pop
            ),
        );
    }
    c.put("transports", g.transport.len());
    let ev = &g.evn;
    c.put("evn.year", ev.year);
    c.put("evn.done", list(ev.done.iter().map(|&b| b as u8)));
    c.put(
        "evn.plague",
        if ev.have_plague != 0 {
            format!(
                "{}@{} {}",
                ev.have_plague, ev.plague_planet_i, ev.plague_val
            )
        } else {
            "-".into()
        },
    );
    c.put(
        "evn.nova",
        if ev.have_nova != 0 {
            format!(
                "{}@{} {} {}",
                ev.have_nova, ev.nova_planet_i, ev.nova_years, ev.nova_val
            )
        } else {
            "-".into()
        },
    );
    c.put(
        "evn.comet",
        if ev.have_comet != 0 {
            format!(
                "{}@{} {} {} {}",
                ev.have_comet, ev.comet_planet_i, ev.comet_years, ev.comet_hp, ev.comet_dmg
            )
        } else {
            "-".into()
        },
    );
    c.put(
        "evn.pirates",
        if ev.have_pirates != 0 {
            format!(
                "{}@{} {}",
                ev.have_pirates, ev.pirates_planet_i, ev.pirates_hp
            )
        } else {
            "-".into()
        },
    );
    for (name, m) in [("crystal", &ev.crystal), ("amoeba", &ev.amoeba)] {
        c.put(
            format!("evn.{name}"),
            if m.exists != 0 {
                format!(
                    "({},{})->{} c{} n{} k{}",
                    m.x,
                    m.y,
                    m.dest,
                    m.counter,
                    m.nuked,
                    oo(m.killer)
                )
            } else {
                "-".into()
            },
        );
    }
    c.put(
        "council",
        format!(
            "held{} refuse{}",
            g.election_held,
            list(g.refuse[..n].iter().map(|&b| b as u8))
        ),
    );
    c.0
}

pub fn from_state(st: &State) -> Canon {
    let mut c = C(Vec::new());
    let n = st.empires.len();
    c.put("seed", st.rng.0);
    c.put("year", st.year - 2299);
    for (i, p) in st.planets.iter().enumerate() {
        let k = format!("p{i}.");
        c.put(k.clone() + "owner", opt(p.owner));
        c.put(k.clone() + "pop", p.pop);
        c.put(k.clone() + "pop_tenths", p.pop_tenths);
        c.put(k.clone() + "factories", p.factories);
        c.put(k.clone() + "waste", p.waste);
        c.put(k.clone() + "bases", p.bases);
        c.put(k.clone() + "shield", p.shield);
        c.put(
            k.clone() + "max_pop",
            format!("{}/{}/{}", p.size, p.base_max_pop, p.max_pop),
        );
        c.put(
            k.clone() + "kind",
            format!("{}/{}", p.kind as i32, p.growth as i32),
        );
        c.put(k.clone() + "sliders", list(p.sliders.iter().copied()));
        c.put(k.clone() + "build", p.build);
        c.put(k.clone() + "reloc", p.rally.unwrap_or(i));
        c.put(
            k.clone() + "bc",
            format!(
                "ship{} fact{} base{} up{} sh{} refit{} eco{}",
                p.bc_to_ship,
                p.bc_to_factory,
                p.bc_to_base,
                p.bc_upgrade_base,
                p.bc_to_shield,
                p.bc_to_refit,
                p.bc_to_ecoproj
            ),
        );
        c.put(k.clone() + "reserve", p.reserve);
        let unrest = match p.unrest {
            Unrest::Normal => 0,
            Unrest::Rebellion => 3,
            Unrest::Resolved => 4,
        };
        c.put(k.clone() + "unrest", format!("{}/{}", unrest, p.rebels));
        c.put(
            k.clone() + "trans",
            format!(
                "{}->{}",
                p.trans_num,
                if p.trans_num > 0 {
                    p.trans_dest as i32
                } else {
                    -1
                }
            ),
        );
        c.put(k.clone() + "pop_oper_fact", p.pop_oper_fact);
        c.put(k.clone() + "stargate", p.stargate as u8);
        c.put(
            k.clone() + "explored",
            list((0..n).map(|j| st.empires[j].explored[i] as u8)),
        );
        c.put(k.clone() + "claim", opt(p.claim));
        c.put(k + "pop_prev", p.pop_prev);
    }
    for (i, e) in st.empires.iter().enumerate() {
        let k = format!("e{i}.");
        c.put(k.clone() + "home", if e.alive { e.home as i32 } else { -1 });
        c.put(k.clone() + "reserve", e.reserve);
        c.put(k.clone() + "tax", e.tax);
        c.put(k.clone() + "security", e.security);
        c.put(
            k.clone() + "countdown",
            format!("{}/{}", e.ai_p2_countdown, e.ai_p3_countdown),
        );
        c.put(
            k.clone() + "shipi",
            format!("{}/{}", e.shipi_colony, e.shipi_bomber),
        );
        c.put(
            k.clone() + "base_parts",
            format!("{}/{}/{}", e.base_shield, e.base_comp, e.base_weapon),
        );
        for f in 0..6 {
            let kf = format!("{k}t{f}.");
            c.put(
                kf.clone() + "completed",
                list(e.tech.completed[f].iter().copied()),
            );
            c.put(
                kf.clone() + "project",
                format!("{} ${}", e.tech.project[f], e.tech.cost[f]),
            );
            c.put(kf.clone() + "invest", e.tech.invested[f]);
            c.put(kf.clone() + "slider", e.tech.alloc[f]);
            c.put(kf + "percent", e.tech.level[f]);
        }
        for (d, sd) in e.designs.iter().enumerate() {
            design_key(
                &mut c,
                &format!("{k}d{d}"),
                &sd.name,
                sd.hull,
                sd.look,
                &sd.weapons,
                &sd.weapon_counts,
                sd.engine,
                sd.engines,
                &sd.specials,
                sd.shield,
                sd.jammer,
                sd.comp,
                sd.armor,
                sd.man,
                sd.hp,
                sd.cost_bc,
            );
            c.put(
                format!("{k}d{d}.year"),
                e.design_year.get(d).copied().unwrap_or(0),
            );
        }
        c.put(k.clone() + "designs", e.designs.len());
        let orbits: Vec<String> = e
            .orbit
            .iter()
            .enumerate()
            .filter(|(_, o)| o.iter().any(|&x| x != 0))
            .map(|(p, o)| format!("{p}:{}", list(o.iter().copied())))
            .collect();
        c.put(k.clone() + "orbit", orbits.join(" "));
        for (j, r) in e.rel.iter().enumerate() {
            let kr = format!("{k}r{j}.");
            c.put(
                kr.clone() + "rel",
                format!(
                    "{}/{} trust{} hatred{} blunder{}",
                    r.relation, r.base_relation, r.trust, r.hatred, r.blunder
                ),
            );
            c.put(
                kr.clone() + "treaty",
                format!("{} broken{}", treaty_n(r.treaty), treaty_n(r.broken_treaty)),
            );
            c.put(
                kr.clone() + "mood",
                format!(
                    "{}/{}/{}/{}",
                    r.mood_treaty, r.mood_trade, r.mood_tech, r.mood_peace
                ),
            );
            c.put(
                kr.clone() + "trade",
                format!(
                    "{} {}% est{}",
                    r.trade_bc, r.trade_percent, r.trade_established
                ),
            );
            c.put(
                kr.clone() + "spy",
                format!(
                    "{} mode{} next{} spies{} fund{}",
                    r.spying,
                    spy_n(r.spymode),
                    spy_n(r.spymode_next),
                    r.spies,
                    r.spyfund
                ),
            );
            c.put(
                kr.clone() + "contact",
                format!("{} met{}", r.contact as u8, r.have_met),
            );
            c.put(kr + "ceasefire", r.ceasefire);
        }
    }
    for (k, f) in st.fleets.iter().enumerate() {
        c.put(
            format!("fleet{k}"),
            format!(
                "o{} ({},{})->{} v{} [{}]",
                f.owner,
                f.x,
                f.y,
                f.dest,
                f.speed,
                list(f.ships.iter().copied())
            ),
        );
    }
    c.put("fleets", st.fleets.len());
    for (k, t) in st.transports.iter().enumerate() {
        c.put(
            format!("trans{k}"),
            format!(
                "o{} ({},{})->{} v{} pop{}",
                t.owner, t.x, t.y, t.dest, t.speed, t.pop
            ),
        );
    }
    c.put("transports", st.transports.len());
    let ev = &st.evn;
    c.put("evn.year", ev.last_year);
    c.put("evn.done", list(ev.done.iter().map(|&b| b as u8)));
    let l = |x: &Option<aoo_core::events::Lasting>,
             f: &dyn Fn(&aoo_core::events::Lasting) -> String| {
        x.as_ref().map_or("-".to_string(), f)
    };
    c.put(
        "evn.plague",
        l(&ev.plague, &|x| {
            format!("{}@{} {}", x.stage, x.planet, x.val)
        }),
    );
    c.put(
        "evn.nova",
        l(&ev.nova, &|x| {
            format!("{}@{} {} {}", x.stage, x.planet, x.years, x.val)
        }),
    );
    c.put(
        "evn.comet",
        l(&ev.comet, &|x| {
            format!(
                "{}@{} {} {} {}",
                x.stage, x.planet, x.years, x.val, x.damage
            )
        }),
    );
    c.put(
        "evn.pirates",
        l(&ev.pirates, &|x| {
            format!("{}@{} {}", x.stage, x.planet, x.val)
        }),
    );
    for (name, kind) in [
        ("crystal", aoo_core::events::MonsterKind::Crystal),
        ("amoeba", aoo_core::events::MonsterKind::Amoeba),
    ] {
        let m = ev.monsters.iter().find(|m| m.kind == kind);
        c.put(
            format!("evn.{name}"),
            m.map_or("-".to_string(), |m| {
                format!(
                    "({},{})->{} c{} n{} k{}",
                    m.x,
                    m.y,
                    m.dest,
                    m.counter,
                    m.nuked,
                    opt(m.killer)
                )
            }),
        );
    }
    c.put(
        "council",
        format!(
            "held{} refuse{}",
            st.council.held as u8,
            list((0..n).map(|i| st.council.refuse.get(i).copied().unwrap_or(false) as u8))
        ),
    );
    c.0
}

/// Fields that differ: (key, 1oom value, Age of Orion value).
pub fn diff(a: &Canon, b: &Canon) -> Vec<(String, String, String)> {
    let mb: std::collections::HashMap<&str, &str> =
        b.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let ma: std::collections::HashMap<&str, &str> =
        a.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let mut out = Vec::new();
    for (k, va) in a {
        match mb.get(k.as_str()) {
            Some(vb) if vb == va => {}
            Some(vb) => out.push((k.clone(), va.clone(), vb.to_string())),
            None => out.push((k.clone(), va.clone(), "(missing)".into())),
        }
    }
    for (k, vb) in b {
        if !ma.contains_key(k.as_str()) {
            out.push((k.clone(), "(missing)".into(), vb.clone()));
        }
    }
    out
}
