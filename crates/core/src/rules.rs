//! Tunable numbers and tables. Everything here is data: a mod can replace any of it by
//! passing a `rules` object when starting a game (missing fields keep their defaults).
//! Defaults are the values of 1oom `game_num.c`.

use serde::{Deserialize, Serialize};

use crate::design::{start_designs, ShipDesign};
use crate::state::Tech;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Rules {
    pub start_year: u32,
    /// Designs every empire starts with (1oom `tbl_startship`).
    pub start_designs: Vec<ShipDesign>,
    /// Ships each empire starts with, per start design.
    pub start_ships: Vec<u32>,
    pub home_pop: i32,
    pub home_max_pop: i32,
    pub home_factories: i32,
    pub max_pop: i32,
    pub max_factories: i32,
    pub colony_ship_pop: i32,
    /// Research cost in percent of the 1oom formula.
    pub research_cost_percent: u32,
    /// 1oom `game_num_tech_costmulr`: cost multiplier per race and field.
    pub race_tech_cost: [[u32; NUM_FIELDS]; 10],
    /// 1oom `game_num_tech_costmuld`, per difficulty, for human players.
    pub difficulty_tech_cost: [u32; 5],
    /// 1oom `game_num_tech_costmula`, per difficulty, for computer players.
    pub ai_tech_cost: [u32; 5],
    /// 1oom `game_num_pshield_cost`.
    pub shield_cost: [i32; 5],
    /// 1oom `game_num_base_hp`, by best armour tier.
    pub base_hp: [u32; 7],
    /// Parsec length in map units (1oom moves 5 + 6 units per parsec).
    pub parsec: f64,
    pub combat_rounds: u32,
    pub nebulae: bool,
    pub stargate_cost: i32,
    pub stargate_maint: i32,
    pub atmos_cost: i32,
    pub soil_cost: i32,
    pub adv_soil_cost: i32,
    /// Bombing damage per population unit and per factory.
    pub pop_hp: i32,
    pub fact_hp: i32,
    pub max_bomb_dmg: u32,
    pub max_bio_dmg: i32,
    pub max_ship_maint: i32,
}

impl Default for Rules {
    fn default() -> Self {
        Rules {
            start_year: 2300,
            start_designs: start_designs(),
            start_ships: vec![2, 2, 0, 0, 1],
            home_pop: 50,
            home_max_pop: 100,
            home_factories: 30,
            max_pop: 300,
            max_factories: 2700,
            colony_ship_pop: 2,
            research_cost_percent: 100,
            race_tech_cost: [
                [100, 100, 60, 80, 80, 100],
                [100, 125, 100, 100, 100, 60],
                [80, 125, 125, 125, 125, 125],
                [100, 100, 100, 60, 100, 100],
                [80, 80, 80, 80, 80, 80],
                [100, 100, 125, 100, 60, 100],
                [100, 60, 100, 100, 125, 100],
                [125, 80, 100, 100, 100, 80],
                [60, 100, 100, 125, 100, 100],
                [80, 100, 100, 100, 100, 100],
            ],
            difficulty_tech_cost: [20, 25, 30, 35, 40],
            ai_tech_cost: [20, 20, 20, 20, 20],
            shield_cost: [0, 500, 1000, 1500, 2000],
            base_hp: [50, 75, 100, 125, 150, 175, 200],
            parsec: 11.0,
            combat_rounds: 10,
            nebulae: true,
            stargate_cost: 3000,
            stargate_maint: 100,
            atmos_cost: 200,
            soil_cost: 150,
            adv_soil_cost: 300,
            pop_hp: 200,
            fact_hp: 50,
            max_bomb_dmg: 100_000,
            max_bio_dmg: 10_000,
            max_ship_maint: 32_000,
        }
    }
}

/// Values derived from the techs an empire has completed (1oom `game_update_tech_util`).
/// They are stored per empire and refreshed when 1oom refreshes them (after research, at
/// the end of the turn), not the moment a tech arrives.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Abilities {
    pub colonist_oper_factories: i32,
    pub factory_cost: i32,
    /// 1oom `factory_adj_cost`: cost of a factory including the colonists to run it.
    pub factory_adj_cost: i32,
    pub ind_waste_scale: i32,
    pub fuel_range: i32,
    pub scanner_range: i32,
    pub colony_for: PlanetType,
    pub inc_pop_cost: i32,
    pub eco_restoration: i32,
    pub terraform: i32,
    pub terraform_cost: i32,
    pub planet_shield: i32,
    pub planet_shield_cost: i32,
    pub soil_enrich: bool,
    pub adv_soil_enrich: bool,
    pub atmos_terraform: bool,
    pub stargates: bool,
    pub hyperspace_comm: bool,
    pub ia_scanner: bool,
    pub adv_scanner: bool,
    pub combat_transporters: bool,
    pub sub_space_interdictor: bool,
    pub antidote: i32,
    /// 1oom `have_engine`: the best engine's warp, the speed of transports.
    pub engine_warp: i32,
    /// Ground combat bonus (1oom `game_ground_resolve_init`).
    pub ground_bonus: i32,
    /// Best armour tier (0 titanium .. 6 neutronium): bases and transports.
    pub armor_tier: usize,
}

impl Default for Abilities {
    fn default() -> Self {
        Abilities::new(&Tech::new(), Race::Human)
    }
}

/// 1oom `find_best_tech_type`: best completed tech `base + k * step <= last`, 0 if none.
fn best(t: &Tech, field: usize, base: u8, step: u8, last: u8) -> i32 {
    let mut b = 0;
    let mut i = base;
    while i <= last {
        if t.has(field, i) {
            b = i as i32;
        }
        i += step;
    }
    b
}

/// The value of the first completed tech of `table` (`(tech, value)`, in 1oom's test
/// order), or `default`.
fn first(t: &Tech, field: usize, default: i32, table: &[(u8, i32)]) -> i32 {
    table
        .iter()
        .find(|(n, _)| t.has(field, *n))
        .map_or(default, |(_, v)| *v)
}

/// Tier of the last completed tech of a research group, in completion order (1oom loops
/// over `researchcompleted` keeping the last group match; groups and tiers are
/// RESEARCH.LBX data, `TECH_GROUPS` here).
pub fn group_tier(t: &Tech, field: usize, group: u8) -> (i32, u8) {
    let mut b = (0, 0);
    for &n in &t.completed[field] {
        if n <= 50 {
            let [g, tier] = crate::data::TECH_GROUPS[field][n as usize];
            if g == group && tier > 0 {
                b = (tier as i32, n);
            }
        }
    }
    b
}

/// 1oom `TECH_GROUP_*` numbers used by the rules.
pub const GROUP_ARMOR: u8 = 7;
pub const GROUP_PERSONAL_ARMOR: u8 = 15;
pub const GROUP_PERSONAL_SHIELD: u8 = 16;
pub const GROUP_PERSONAL_WEAPONS: u8 = 21;

impl Abilities {
    /// 1oom `game_update_tech_util` for one empire.
    pub fn new(t: &Tech, race: Race) -> Self {
        let (comp, cons, ff, plan, prop, weap) = (
            FIELD_COMPUTER,
            FIELD_CONSTRUCTION,
            FIELD_FORCE_FIELD,
            FIELD_PLANETOLOGY,
            FIELD_PROPULSION,
            FIELD_WEAPON,
        );
        let silicoid = race == Race::Silicoid;
        let mut colony_for = PlanetType::Minimal;
        for i in 0..6u8 {
            if t.has(plan, i * 3 + 3) {
                colony_for = PlanetType::from_index(PlanetType::Barren.index() - i as usize);
            }
        }
        if silicoid {
            colony_for = PlanetType::Radiated;
        }
        let mut inc_pop_cost = first(t, plan, 20, &[(42, 5), (21, 10)]);
        if race == Race::Sakkra {
            inc_pop_cost = inc_pop_cost * 2 / 3;
        }
        let tf = best(t, plan, 2, 6, 50);
        let terraform = match if tf > 0 { ((tf - 2) / 6 + 1) * 10 } else { 0 } {
            90 => 120,
            80 => 100,
            70 => 80,
            b => b,
        };
        let terraform_cost = if tf > 0 {
            (5 - (tf - 2) / 12).max(2)
        } else {
            5
        };
        let mut eco_restoration = 2;
        for (n, v) in [(5, 3), (13, 5), (24, 10), (34, 20)] {
            if t.has(plan, n) {
                eco_restoration = v;
            }
        }
        let scanner_range = first(t, comp, 3, &[(23, 9), (13, 7), (4, 5)]);
        let rc = best(t, comp, 8, 10, 50);
        let mut cof = if rc > 0 { (rc + 12) / 10 + 1 } else { 2 };
        if race == Race::Meklar {
            cof += 2;
        }
        let factory_cost = first(
            t,
            cons,
            10,
            &[
                (38, 2),
                (33, 3),
                (28, 4),
                (23, 5),
                (18, 6),
                (13, 7),
                (8, 8),
                (3, 9),
            ],
        );
        let factory_adj_cost = if race != Race::Meklar {
            factory_cost * cof / 2
        } else {
            factory_cost
        };
        let fs = best(t, ff, 2, 10, 50);
        let fs = if fs < 10 { 0 } else { fs };
        let planet_shield = if fs > 0 { 5 * ((fs - 2) / 10) } else { 0 };
        let mut engine = 1;
        let mut i = 6;
        while i < 50 {
            if t.has(prop, i) {
                engine = (i as i32 - 6) / 6 + 1;
            }
            i += 6;
        }
        let armor_tier = group_tier(t, cons, GROUP_ARMOR).0 as usize;
        let suit = group_tier(t, cons, GROUP_PERSONAL_ARMOR).0;
        let pshield = group_tier(t, ff, GROUP_PERSONAL_SHIELD).0;
        let mut pweap = group_tier(t, weap, GROUP_PERSONAL_WEAPONS).0;
        if pweap > 2 {
            pweap += 1;
        }
        let ground_bonus = armor_tier as i32 * 5
            + suit * 10
            + pshield * 10
            + pweap * 5
            + if race == Race::Bulrathi { 25 } else { 0 };
        Abilities {
            colonist_oper_factories: cof,
            factory_cost,
            factory_adj_cost,
            ind_waste_scale: first(t, cons, 10, &[(45, 0), (35, 2), (25, 4), (15, 6), (5, 8)]),
            fuel_range: first(
                t,
                prop,
                3,
                &[
                    (41, 30),
                    (29, 10),
                    (23, 9),
                    (19, 8),
                    (14, 7),
                    (9, 6),
                    (5, 5),
                    (3, 4),
                ],
            ),
            scanner_range,
            colony_for,
            inc_pop_cost,
            eco_restoration,
            terraform,
            terraform_cost,
            planet_shield,
            planet_shield_cost: [0, 500, 1000, 1500, 2000]
                [(planet_shield / 5).clamp(0, 4) as usize],
            soil_enrich: !silicoid && t.has(plan, 16),
            adv_soil_enrich: !silicoid && t.has(plan, 30),
            atmos_terraform: !silicoid && t.has(plan, 22),
            stargates: t.has(prop, 27),
            hyperspace_comm: t.has(comp, 34),
            ia_scanner: t.has(comp, 13) || t.has(comp, 23),
            adv_scanner: t.has(comp, 23),
            combat_transporters: t.has(prop, 45),
            sub_space_interdictor: t.has(prop, 43),
            antidote: first(t, plan, 0, &[(36, 2), (17, 1)]),
            engine_warp: engine,
            ground_bonus,
            armor_tier,
        }
    }
}

impl Rules {
    /// 1oom `game_tech_get_next_rp`, scaled by `research_cost_percent`.
    pub fn research_cost(
        &self,
        race: Race,
        field: usize,
        tech: u8,
        ai: bool,
        diff: Difficulty,
    ) -> u32 {
        let t = tech as u32;
        let mut cost = t * t * self.race_tech_cost[race.index()][field];
        if ai {
            cost = cost * self.ai_tech_cost[diff as usize] / 100;
        } else {
            cost = cost * self.difficulty_tech_cost[diff as usize] / 1000 * 10;
        }
        cost * self.research_cost_percent / 100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tech_with(known: &[(usize, &[u8])]) -> Tech {
        let mut t = Tech::new();
        for (f, ns) in known {
            for &n in ns.iter() {
                t.add(*f, n);
            }
        }
        t
    }

    #[test]
    fn start_abilities_match_moo1() {
        let a = Abilities::new(&Tech::new(), Race::Human);
        assert_eq!(a.colonist_oper_factories, 2);
        assert_eq!(a.factory_cost, 10);
        assert_eq!(a.fuel_range, 3);
        assert_eq!(a.scanner_range, 3);
        assert_eq!(a.colony_for, PlanetType::Minimal);
        assert_eq!(a.inc_pop_cost, 20);
        assert_eq!(a.engine_warp, 1);
        assert_eq!(a.ground_bonus, 0);
        let m = Abilities::new(&Tech::new(), Race::Meklar);
        assert_eq!(m.colonist_oper_factories, 4);
    }

    #[test]
    fn completed_techs_unlock_abilities() {
        let t = tech_with(&[
            (FIELD_COMPUTER, &[8, 18]),
            (FIELD_CONSTRUCTION, &[3, 38, 10, 11]),
            (FIELD_PLANETOLOGY, &[3, 6, 9, 12, 15, 18, 2, 8, 14]),
            (FIELD_PROPULSION, &[9, 6]),
            (FIELD_FORCE_FIELD, &[22]),
            (FIELD_WEAPON, &[2]),
        ]);
        let a = Abilities::new(&t, Race::Human);
        assert_eq!(a.colonist_oper_factories, 4);
        assert_eq!(a.factory_cost, 2);
        assert_eq!(a.colony_for, PlanetType::Radiated);
        assert_eq!(a.fuel_range, 6);
        assert_eq!(a.planet_shield, 10);
        assert_eq!(a.terraform, 30);
        // 1oom `have_engine`: nuclear engines (tech 6) still give 1.
        assert_eq!(a.engine_warp, 1);
        assert_eq!(a.armor_tier, 1);
        assert_eq!(a.ground_bonus, 5 + 10 + 5);
    }

    #[test]
    fn research_cost_matches_1oom() {
        let r = Rules::default();
        assert_eq!(
            r.research_cost(Race::Human, 0, 2, false, Difficulty::Average),
            120
        );
        assert_eq!(
            r.research_cost(Race::Human, 0, 10, false, Difficulty::Average),
            3000
        );
        assert_eq!(
            r.research_cost(Race::Human, 0, 10, true, Difficulty::Average),
            2000
        );
    }
}
