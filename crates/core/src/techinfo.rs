//! What each technology gives, for the research screen: the abilities it changes (with the
//! numbers before and after, from 1oom `game_update_tech_util`), the ship parts it unlocks
//! and the field level it brings (parts of a field get cheaper and smaller with its level).

use serde::Serialize;

use crate::data::{ARMOR, COMPS, ENGINES, JAMMERS, SHIELDS, SPECIALS, WEAPONS};
use crate::design::weapon_field;
use crate::rules::{Abilities, Rules};
use crate::state::Tech;
use crate::tech::field_level;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TechEffect {
    /// The tech level of the field.
    Level { from: u32, to: u32 },
    /// A number of the empire changes (`what` names it, as in `Abilities`, plus
    /// `transport_hp` and `base_hp`). `superseded`: a better tech already gives more; the
    /// numbers are then what this tech alone gives.
    Value {
        what: &'static str,
        from: i32,
        to: i32,
        superseded: bool,
    },
    /// Something becomes possible; `cost` is its price in BC when it has one.
    Gain { what: &'static str, cost: i32 },
    /// The worst planet type that can be colonized.
    Colonize { kind: PlanetType, superseded: bool },
    /// A ship part: `kind` is weapon, comp, jammer, shield, armor, engine or special, `index`
    /// its row in the part table.
    Part { kind: &'static str, index: u8 },
}

fn with(t: &Tech, f: usize, n: u8, add: bool) -> Tech {
    let mut t = t.clone();
    let c = &mut t.completed[f];
    match c.binary_search(&n) {
        Ok(i) if !add => {
            c.remove(i);
        }
        Err(i) if add => c.insert(i, n),
        _ => {}
    }
    t
}

fn values(a: &Abilities, rules: &Rules) -> [(&'static str, i32); 16] {
    let tier = a.armor_tier.min(6);
    [
        ("colonist_oper_factories", a.colonist_oper_factories),
        ("factory_cost", a.factory_cost),
        ("ind_waste_scale", a.ind_waste_scale),
        ("eco_restoration", a.eco_restoration),
        ("inc_pop_cost", a.inc_pop_cost),
        ("terraform", a.terraform),
        ("terraform_cost", a.terraform_cost),
        ("fuel_range", a.fuel_range),
        ("scanner_range", a.scanner_range),
        ("engine_warp", a.engine_warp),
        ("planet_shield", a.planet_shield),
        ("planet_shield_cost", a.planet_shield_cost),
        ("antidote", a.antidote),
        ("ground_bonus", a.ground_bonus),
        ("transport_hp", (tier as i32 + 1) * 15),
        ("base_hp", rules.base_hp[tier] as i32),
    ]
}

fn gains(a: &Abilities) -> [(&'static str, bool); 9] {
    [
        ("soil_enrich", a.soil_enrich),
        ("adv_soil_enrich", a.adv_soil_enrich),
        ("atmos_terraform", a.atmos_terraform),
        ("stargates", a.stargates),
        ("hyperspace_comm", a.hyperspace_comm),
        ("ia_scanner", a.ia_scanner),
        ("adv_scanner", a.adv_scanner),
        ("combat_transporters", a.combat_transporters),
        ("sub_space_interdictor", a.sub_space_interdictor),
    ]
}

/// The changes between two sets of abilities.
fn diff(a: &Abilities, b: &Abilities, rules: &Rules, superseded: bool) -> Vec<TechEffect> {
    let mut out = Vec::new();
    if a.colony_for != b.colony_for {
        out.push(TechEffect::Colonize {
            kind: b.colony_for,
            superseded,
        });
    }
    for ((what, from), (_, to)) in values(a, rules).into_iter().zip(values(b, rules)) {
        if from != to {
            out.push(TechEffect::Value {
                what,
                from,
                to,
                superseded,
            });
        }
    }
    if !superseded {
        for ((what, x), (_, y)) in gains(a).into_iter().zip(gains(b)) {
            if !x && y {
                let cost = match what {
                    "soil_enrich" => rules.soil_cost,
                    "adv_soil_enrich" => rules.adv_soil_cost,
                    "atmos_terraform" => rules.atmos_cost,
                    "stargates" => rules.stargate_cost,
                    _ => 0,
                };
                out.push(TechEffect::Gain { what, cost });
            }
        }
    }
    out
}

/// What tech `n` of field `f` gives an empire with techs `t`: compared with the same
/// empire without it, and, for what a better tech already gives, compared with an empire
/// that knows nothing else.
pub fn effects(t: &Tech, race: Race, rules: &Rules, f: usize, n: u8) -> Vec<TechEffect> {
    let mut out = Vec::new();
    if n == 0 {
        return out;
    }
    let (t0, t1) = (with(t, f, n, false), with(t, f, n, true));
    let (l0, l1) = (field_level(&t0, f), field_level(&t1, f));
    if l0 != l1 {
        out.push(TechEffect::Level { from: l0, to: l1 });
    }
    let main = diff(
        &Abilities::new(&t0, race),
        &Abilities::new(&t1, race),
        rules,
        false,
    );
    let base = Tech::new();
    let alone = diff(
        &Abilities::new(&with(&base, f, n, false), race),
        &Abilities::new(&with(&base, f, n, true), race),
        rules,
        true,
    );
    let same = |a: &TechEffect, b: &TechEffect| match (a, b) {
        (TechEffect::Value { what: x, .. }, TechEffect::Value { what: y, .. }) => x == y,
        (TechEffect::Colonize { .. }, TechEffect::Colonize { .. }) => true,
        _ => false,
    };
    let extra: Vec<TechEffect> = alone
        .into_iter()
        .filter(|e| !main.iter().any(|m| same(m, e)))
        .collect();
    out.extend(main);
    out.extend(extra);
    let mut part = |kind: &'static str, index: usize| {
        out.push(TechEffect::Part {
            kind,
            index: index as u8,
        })
    };
    for (i, w) in WEAPONS.iter().enumerate().skip(1) {
        if w.tech == n && weapon_field(i) == f {
            part("weapon", i);
        }
    }
    if f == FIELD_COMPUTER {
        for (i, c) in COMPS.iter().enumerate().skip(1) {
            if c.tech == n {
                part("comp", i);
            }
        }
        for (i, c) in JAMMERS.iter().enumerate().skip(1) {
            if c.tech == n {
                part("jammer", i);
            }
        }
    }
    if f == FIELD_FORCE_FIELD {
        for (i, s) in SHIELDS.iter().enumerate().skip(1) {
            if s.tech == n {
                part("shield", i);
            }
        }
    }
    if f == FIELD_CONSTRUCTION {
        for (i, a) in ARMOR.iter().enumerate() {
            if a.tech == n {
                part("armor", i);
            }
        }
    }
    if f == FIELD_PROPULSION {
        for (i, e) in ENGINES.iter().enumerate() {
            if e.tech == n {
                part("engine", i);
            }
        }
    }
    for (i, s) in SPECIALS.iter().enumerate().skip(1) {
        if s.tech == n && s.field as usize == f {
            part("special", i);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rules;

    fn value(e: &[TechEffect], w: &str) -> Option<(i32, i32, bool)> {
        e.iter().find_map(|x| match x {
            TechEffect::Value {
                what,
                from,
                to,
                superseded,
            } if *what == w => Some((*from, *to, *superseded)),
            _ => None,
        })
    }

    #[test]
    fn describes_abilities_parts_and_superseded_techs() {
        let rules = Rules::default();
        let mut t = Tech::new();
        // Improved Industrial Tech 9 (construction 3): factories cost 9 BC instead of 10.
        let e = effects(&t, Race::Human, &rules, FIELD_CONSTRUCTION, 3);
        assert_eq!(value(&e, "factory_cost"), Some((10, 9, false)));
        // Improved Robotic Controls III (computer 8): 3 factories per colonist.
        let e = effects(&t, Race::Human, &rules, FIELD_COMPUTER, 8);
        assert_eq!(value(&e, "colonist_oper_factories"), Some((2, 3, false)));
        // Lasers (weapon 1 is known; tech 1 gives the laser part) and Hand Lasers.
        let e = effects(&t, Race::Human, &rules, FIELD_WEAPON, 2);
        assert!(value(&e, "ground_bonus").is_some());
        // With Industrial Tech 8 known, Industrial Tech 9 is superseded.
        t.completed[FIELD_CONSTRUCTION] = vec![1, 8];
        let e = effects(&t, Race::Human, &rules, FIELD_CONSTRUCTION, 3);
        assert_eq!(value(&e, "factory_cost"), Some((10, 9, true)));
        // Controlled Barren Environment.
        let e = effects(&t, Race::Human, &rules, FIELD_PLANETOLOGY, 3);
        assert!(e.contains(&TechEffect::Colonize {
            kind: PlanetType::Barren,
            superseded: false
        }));
        // Nuclear engines: an engine part; transports only speed up with Sub-Light Drives
        // (1oom `have_engine`).
        let e = effects(&t, Race::Human, &rules, FIELD_PROPULSION, 6);
        assert!(e
            .iter()
            .any(|x| matches!(x, TechEffect::Part { kind: "engine", .. })));
        assert_eq!(value(&e, "engine_warp"), None);
        let e = effects(&t, Race::Human, &rules, FIELD_PROPULSION, 12);
        assert_eq!(value(&e, "engine_warp"), Some((1, 2, false)));
    }
}
