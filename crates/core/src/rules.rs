//! Tunable numbers and tables. Everything here is data: a mod can replace any of it by
//! passing a `rules` object when starting a game (missing fields keep their defaults).

use serde::{Deserialize, Serialize};

use crate::types::{Difficulty, PlanetType, Race, TechField, NUM_FIELDS};

/// A ship design. The five default designs are the MOO1 starting designs
/// (1oom `tbl_startship`): scout, fighter, destroyer, bomber, colony ship.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Design {
    /// Text key for the UI (`design.<key>`), so themes can rename it.
    pub key: String,
    /// Cost in BC with construction level 1.
    pub cost: i32,
    pub hp: i32,
    /// Damage per combat round against ships and bases.
    pub attack: i32,
    /// Damage against population and factories when bombing a planet.
    pub bombs: i32,
    /// Can found a colony (consumed on use).
    pub colony: bool,
    /// Extra fuel range in parsecs (reserve tanks of the scout).
    pub extra_range: i32,
}

impl Design {
    pub fn armed(&self) -> bool {
        self.attack > 0 || self.bombs > 0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Rules {
    pub start_year: u32,
    pub designs: Vec<Design>,
    /// Ships each empire starts with, per design.
    pub start_ships: Vec<u32>,
    pub home_pop: i32,
    pub home_max_pop: i32,
    pub home_factories: i32,
    pub max_pop: i32,
    pub max_factories: i32,
    pub colony_ship_pop: i32,
    /// Research cost of level `n` in percent of the 1oom formula. Every level counts in
    /// this port (MOO1 skips unused tech numbers), so the default is 40%.
    pub research_cost_percent: u32,
    /// 1oom `game_num_tech_costmulr`: cost multiplier per race and field.
    pub race_tech_cost: [[u32; NUM_FIELDS]; 10],
    /// 1oom `game_num_tech_costmuld`, per difficulty, for human players.
    pub difficulty_tech_cost: [u32; 5],
    /// 1oom `game_num_pshield_cost`.
    pub shield_cost: [i32; 5],
    pub base_cost: i32,
    pub base_attack: i32,
    pub base_hp: i32,
    /// Parsec length in map units (1oom moves 5 + 6 units per parsec).
    pub parsec: f64,
    pub combat_rounds: u32,
    pub guardian_attack: i32,
    pub guardian_hp: i32,
    /// Tech levels granted in every field to whoever colonizes Orion.
    pub orion_tech_bonus: u32,
    pub research_weights_ai: [u32; NUM_FIELDS],
}

impl Default for Rules {
    fn default() -> Self {
        let d = |key: &str, cost, hp, attack, bombs, colony, extra_range| Design {
            key: key.into(),
            cost,
            hp,
            attack,
            bombs,
            colony,
            extra_range,
        };
        Rules {
            start_year: 2300,
            designs: vec![
                d("scout", 8, 2, 0, 0, false, 3),
                d("fighter", 14, 3, 3, 0, false, 0),
                d("destroyer", 74, 18, 12, 0, false, 0),
                d("bomber", 65, 18, 3, 12, false, 0),
                d("colony_ship", 570, 40, 0, 0, true, 0),
            ],
            start_ships: vec![2, 2, 0, 0, 1],
            home_pop: 50,
            home_max_pop: 100,
            home_factories: 30,
            max_pop: 300,
            max_factories: 2500,
            colony_ship_pop: 2,
            research_cost_percent: 40,
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
            shield_cost: [0, 500, 1000, 1500, 2000],
            base_cost: 100,
            base_attack: 10,
            base_hp: 40,
            parsec: 11.0,
            combat_rounds: 8,
            guardian_attack: 150,
            guardian_hp: 2500,
            orion_tech_bonus: 5,
            research_weights_ai: [15, 20, 10, 25, 15, 15],
        }
    }
}

/// Values derived from an empire's tech levels: the thresholds of 1oom
/// `game_update_tech_util`, where an empire at level `n` knows every tech numbered `<= n`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Abilities {
    pub colonist_oper_factories: i32,
    pub factory_cost: i32,
    pub ind_waste_scale: i32,
    pub fuel_range: i32,
    pub speed: i32,
    pub scanner_range: i32,
    pub colony_for: PlanetType,
    pub inc_pop_cost: i32,
    pub eco_restoration: i32,
    pub terraform: i32,
    pub terraform_cost: i32,
    pub planet_shield: i32,
    /// Percent multipliers for ships and bases.
    pub attack_pct: i32,
    pub hp_pct: i32,
    /// Damage taken in percent.
    pub damage_taken_pct: i32,
    /// Ship and base cost in percent (miniaturization).
    pub cost_pct: i32,
    pub ground_bonus: i32,
}

/// Highest threshold `<= level` in `table` (`(threshold, value)` ascending) or `default`.
fn step(level: u32, default: i32, table: &[(u32, i32)]) -> i32 {
    table
        .iter()
        .rev()
        .find(|(t, _)| level >= *t)
        .map_or(default, |(_, v)| *v)
}

/// 1oom `find_best_tech_type`: best tech number `base + k * step <= level`, 0 if none.
fn best(level: u32, base: u32, stride: u32) -> u32 {
    if level < base {
        0
    } else {
        base + (level - base) / stride * stride
    }
}

impl Abilities {
    pub fn new(level: &[u32; NUM_FIELDS], race: Race) -> Self {
        let lv = |f: TechField| level[f as usize];
        let comp = lv(TechField::Computer);
        let cons = lv(TechField::Construction);
        let ff = lv(TechField::ForceField);
        let plan = lv(TechField::Planetology);
        let prop = lv(TechField::Propulsion);
        let weap = lv(TechField::Weapon);

        let t = best(comp.min(48), 8, 10) as i32;
        let mut cof = if t > 0 { (t + 12) / 10 + 1 } else { 2 };
        if race == Race::Meklar {
            cof += 2;
        }
        let factory_cost = step(
            cons,
            10,
            &[
                (3, 9),
                (8, 8),
                (13, 7),
                (18, 6),
                (23, 5),
                (28, 4),
                (33, 3),
                (38, 2),
            ],
        );
        let colony_for = if race == Race::Silicoid {
            PlanetType::Radiated
        } else {
            let mut c = PlanetType::Minimal;
            for i in 0..6 {
                if plan >= i * 3 + 3 {
                    c = PlanetType::from_index(PlanetType::Barren.index() - i as usize);
                }
            }
            c
        };
        let mut inc_pop_cost = step(plan, 20, &[(21, 10), (42, 5)]);
        if race == Race::Sakkra {
            inc_pop_cost = inc_pop_cost * 2 / 3;
        }
        let tf = best(plan.min(50), 2, 6) as i32;
        let terraform = match if tf > 0 { ((tf - 2) / 6 + 1) * 10 } else { 0 } {
            70 => 80,
            80 => 100,
            90 => 120,
            b => b,
        };
        let terraform_cost = if tf > 0 {
            (5 - (tf - 2) / 12).max(2)
        } else {
            5
        };
        let fs = best(ff.min(50), 2, 10) as i32;
        let planet_shield = if fs >= 10 { 5 * ((fs - 2) / 10) } else { 0 };
        let mut scanner_range = step(comp, 3, &[(4, 5), (13, 7), (23, 9)]);
        if race == Race::Darlok {
            scanner_range += 2;
        }
        Abilities {
            colonist_oper_factories: cof,
            factory_cost,
            ind_waste_scale: step(cons, 10, &[(5, 8), (15, 6), (25, 4), (35, 2), (45, 0)]),
            fuel_range: step(
                prop,
                3,
                &[
                    (3, 4),
                    (5, 5),
                    (9, 6),
                    (14, 7),
                    (19, 8),
                    (23, 9),
                    (29, 10),
                    (41, 30),
                ],
            ),
            speed: (1 + prop as i32 / 6).min(9),
            scanner_range,
            colony_for,
            inc_pop_cost,
            eco_restoration: step(plan, 2, &[(5, 3), (13, 5), (24, 10), (34, 20)]),
            terraform,
            terraform_cost,
            planet_shield,
            attack_pct: 100 + 10 * weap as i32 + if race == Race::Mrrshan { 25 } else { 0 },
            hp_pct: 100 + 7 * cons as i32,
            damage_taken_pct: (10000 / (100 + 4 * ff as i32))
                * if race == Race::Alkari { 80 } else { 100 }
                / 100,
            cost_pct: (100 - (cons as i32 - 1) * 3 / 2).max(40),
            ground_bonus: 5 * (weap as i32 / 10)
                + 5 * (cons as i32 / 10)
                + if race == Race::Bulrathi { 25 } else { 0 },
        }
    }
}

impl Rules {
    /// 1oom `game_tech_get_next_rp` scaled by `research_cost_percent`.
    pub fn research_cost(
        &self,
        race: Race,
        field: usize,
        level: u32,
        ai: bool,
        diff: Difficulty,
    ) -> u32 {
        let mut cost = level * level * self.race_tech_cost[race.index()][field];
        if ai {
            cost = cost * 20 / 100;
        } else {
            cost = cost * self.difficulty_tech_cost[diff as usize] / 1000 * 10;
        }
        (cost * self.research_cost_percent / 100).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_abilities_match_moo1() {
        let a = Abilities::new(&[1; NUM_FIELDS], Race::Human);
        assert_eq!(a.colonist_oper_factories, 2);
        assert_eq!(a.factory_cost, 10);
        assert_eq!(a.fuel_range, 3);
        assert_eq!(a.scanner_range, 3);
        assert_eq!(a.colony_for, PlanetType::Minimal);
        assert_eq!(a.inc_pop_cost, 20);
        assert_eq!(a.speed, 1);
        let m = Abilities::new(&[1; NUM_FIELDS], Race::Meklar);
        assert_eq!(m.colonist_oper_factories, 4);
    }

    #[test]
    fn thresholds_advance() {
        let a = Abilities::new(&[18, 38, 22, 18, 9, 1], Race::Human);
        assert_eq!(a.colonist_oper_factories, 4);
        assert_eq!(a.factory_cost, 2);
        assert_eq!(a.colony_for, PlanetType::Radiated);
        assert_eq!(a.fuel_range, 6);
        assert_eq!(a.planet_shield, 10);
        assert_eq!(a.terraform, 30);
    }

    #[test]
    fn research_cost_grows_quadratically() {
        let r = Rules::default();
        let c2 = r.research_cost(Race::Human, 0, 2, false, Difficulty::Average);
        let c4 = r.research_cost(Race::Human, 0, 4, false, Difficulty::Average);
        assert_eq!(c2, 48);
        assert_eq!(c4, 192);
    }
}
