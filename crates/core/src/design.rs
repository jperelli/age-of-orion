//! Ship designs (1oom `game_shipdesign.h`, `game_design.c`): cost, space and engines of a
//! design depend on the empire's tech levels (miniaturization), so they are recomputed as
//! research advances.

use serde::{Deserialize, Serialize};

use crate::data::*;
use crate::data_types::{REDUCE_25, REDUCE_50};
use crate::state::Tech;
use crate::types::*;

pub const MAX_DESIGNS: usize = 6;
pub const WEAPON_SLOTS: usize = 4;
pub const SPECIAL_SLOTS: usize = 3;
/// Value of `Planet::build` that builds a stargate (1oom `BUILDSHIP_STARGATE`).
pub const BUILD_STARGATE: usize = MAX_DESIGNS;

pub const HULL_SMALL: u8 = 0;
pub const HULL_MEDIUM: u8 = 1;
pub const HULL_LARGE: u8 = 2;
pub const HULL_HUGE: u8 = 3;

/// Special kinds (1oom `shiptech_special_s.type`).
pub const KIND_FUEL: u8 = 1;
pub const KIND_COLONY: u8 = 2;
/// Bits of `Special::flags`.
pub const FLAG_SCANNER: u16 = 1 << 0;
pub const FLAG_CLOAK: u16 = 1 << 4;
pub const FLAG_ORACLE: u16 = 1 << 8;
pub const FLAG_DISPLACEMENT: u16 = 1 << 9;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShipDesign {
    pub name: String,
    pub hull: u8,
    /// Picture index (1oom `look`).
    pub look: u8,
    /// Weapon index per slot (`data::WEAPONS`).
    pub weapons: [u8; WEAPON_SLOTS],
    pub weapon_counts: [u8; WEAPON_SLOTS],
    pub engine: u8,
    /// Engines needed, in tenths.
    #[serde(default)]
    pub engines: u32,
    pub specials: [u8; SPECIAL_SLOTS],
    pub shield: u8,
    pub jammer: u8,
    pub comp: u8,
    pub armor: u8,
    /// Maneuverability, 0..=engine.
    pub man: u8,
    #[serde(default)]
    pub hp: u32,
    /// Cost in BC as of the last ship building (1oom `cost`, updated every turn).
    #[serde(default)]
    pub cost_bc: u32,
    /// Free space after the parts (1oom `space`).
    #[serde(default)]
    pub space: i32,
}

/// The five MOO1 starting designs (1oom `tbl_startship`).
pub fn start_designs() -> Vec<ShipDesign> {
    let d = |name: &str, hull, look, weapons: [u8; 4], counts: [u8; 4], special: u8| {
        let mut sd = ShipDesign {
            name: name.into(),
            hull,
            look,
            weapons,
            weapon_counts: counts,
            engine: 0,
            engines: 0,
            specials: [special, 0, 0],
            shield: 0,
            jammer: 0,
            comp: 0,
            armor: 0,
            man: 0,
            hp: 0,
            cost_bc: 0,
            space: 0,
        };
        sd.update();
        sd.cost_bc = sd.cost(&[1; NUM_FIELDS]);
        sd.space = sd.free_space(&[1; NUM_FIELDS]);
        sd
    };
    vec![
        d("Scout", HULL_SMALL, 0, [0; 4], [0; 4], 1),
        d("Fighter", HULL_SMALL, 1, [2, 0, 0, 0], [1, 0, 0, 0], 0),
        d("Destroyer", HULL_MEDIUM, 6, [3, 2, 0, 0], [1, 3, 0, 0], 0),
        d("Bomber", HULL_MEDIUM, 7, [1, 2, 0, 0], [2, 2, 0, 0], 0),
        d("Colony Ship", HULL_LARGE, 12, [0; 4], [0; 4], 2),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Weapon,
    Engine,
    Comp,
    Jammer,
    Shield,
    Armor,
    Special,
}

/// Tech levels above the part's tech, 0..=50 (1oom `tm`).
fn above(level: &[u32; NUM_FIELDS], field: usize, tech: u8) -> usize {
    (level[field] as i32 - tech as i32).clamp(0, 50) as usize
}

fn weapon_field(i: usize) -> usize {
    if WEAPONS[i].bio {
        FIELD_PLANETOLOGY
    } else {
        FIELD_WEAPON
    }
}

impl ShipDesign {
    fn h(&self) -> usize {
        self.hull.min(3) as usize
    }

    /// 1oom `game_design_calc_cost_item_do`: cost x 10 of one part.
    pub fn item_cost(&self, level: &[u32; NUM_FIELDS], slot: Slot, i: usize) -> u32 {
        let h = self.h();
        let (base, field, tech) = match slot {
            Slot::Weapon => (WEAPONS[i].cost, weapon_field(i), WEAPONS[i].tech),
            Slot::Engine => (ENGINES[i].cost, FIELD_PROPULSION, ENGINES[i].tech),
            Slot::Comp => (COMPS[i].cost[h], FIELD_COMPUTER, COMPS[i].tech),
            Slot::Jammer => (JAMMERS[i].cost[h], FIELD_COMPUTER, JAMMERS[i].tech),
            Slot::Shield => (SHIELDS[i].cost[h], FIELD_FORCE_FIELD, SHIELDS[i].tech),
            Slot::Armor => (ARMOR[i].cost[h], FIELD_CONSTRUCTION, ARMOR[i].tech),
            Slot::Special => (
                SPECIALS[i].cost[h],
                SPECIALS[i].field as usize,
                SPECIALS[i].tech,
            ),
        };
        base as u32 * REDUCE_50[above(level, field, tech)] as u32 / 100
    }

    /// 1oom `game_design_calc_space_item`.
    pub fn item_space(&self, level: &[u32; NUM_FIELDS], slot: Slot, i: usize) -> i32 {
        let h = self.h();
        let (base, field, tech, table) = match slot {
            Slot::Weapon => (
                WEAPONS[i].space,
                weapon_field(i),
                WEAPONS[i].tech,
                &REDUCE_50,
            ),
            Slot::Engine => (
                ENGINES[i].space,
                FIELD_PROPULSION,
                ENGINES[i].tech,
                &REDUCE_25,
            ),
            Slot::Comp => (COMPS[i].space[h], FIELD_COMPUTER, COMPS[i].tech, &REDUCE_25),
            Slot::Jammer => (
                JAMMERS[i].space[h],
                FIELD_COMPUTER,
                JAMMERS[i].tech,
                &REDUCE_25,
            ),
            Slot::Shield => (
                SHIELDS[i].space[h],
                FIELD_FORCE_FIELD,
                SHIELDS[i].tech,
                &REDUCE_25,
            ),
            Slot::Armor => (
                ARMOR[i].space[h],
                FIELD_CONSTRUCTION,
                ARMOR[i].tech,
                &REDUCE_25,
            ),
            Slot::Special => (
                SPECIALS[i].space[h],
                SPECIALS[i].field as usize,
                SPECIALS[i].tech,
                &REDUCE_25,
            ),
        };
        let mut space = (base as u32 * table[above(level, field, tech)] as u32 / 100) as i32;
        if i != 0 {
            space = space.max(1);
        }
        space
    }

    /// 1oom `game_design_get_hull_space`.
    pub fn hull_space(&self, level: &[u32; NUM_FIELDS]) -> i32 {
        let space = HULLS[self.h()].space as i32;
        space + space * level[FIELD_CONSTRUCTION] as i32 / 50
    }

    /// Free space left (1oom `game_design_calc_space`); negative means it does not fit.
    pub fn free_space(&self, level: &[u32; NUM_FIELDS]) -> i32 {
        let mut space = self.hull_space(level);
        for s in 0..WEAPON_SLOTS {
            let w = self.weapons[s] as usize;
            if w != 0 && self.weapon_counts[s] > 0 {
                space -= self.item_space(level, Slot::Weapon, w) * self.weapon_counts[s] as i32;
            }
        }
        if self.engines > 0 {
            space -= self.item_space(level, Slot::Engine, self.engine as usize)
                * self.engines as i32
                / 10;
        }
        for (slot, i) in [
            (Slot::Comp, self.comp),
            (Slot::Jammer, self.jammer),
            (Slot::Shield, self.shield),
            (Slot::Armor, self.armor),
        ] {
            if i != 0 {
                space -= self.item_space(level, slot, i as usize);
            }
        }
        for &s in &self.specials {
            if s != 0 {
                space -= self.item_space(level, Slot::Special, s as usize);
            }
        }
        space
    }

    /// Cost in BC (1oom `game_design_calc_cost`).
    pub fn cost(&self, level: &[u32; NUM_FIELDS]) -> u32 {
        let mut cost = HULLS[self.h()].cost as u32;
        for s in 0..WEAPON_SLOTS {
            let w = self.weapons[s] as usize;
            if w != 0 && self.weapon_counts[s] > 0 {
                cost +=
                    self.item_cost(level, Slot::Weapon, w).max(5) * self.weapon_counts[s] as u32;
            }
        }
        if self.engines > 0 {
            cost += self
                .item_cost(level, Slot::Engine, self.engine as usize)
                .max(5)
                * self.engines
                / 10;
        }
        for (slot, i, min) in [
            (Slot::Comp, self.comp, 1),
            (Slot::Jammer, self.jammer, 1),
            (Slot::Shield, self.shield, 5),
            (Slot::Armor, self.armor, 5),
        ] {
            if i != 0 {
                cost += self.item_cost(level, slot, i as usize).max(min);
            }
        }
        for &s in &self.specials {
            if s != 0 {
                cost += self.item_cost(level, Slot::Special, s as usize).max(5);
            }
        }
        ((cost + 5) / 10).max(1)
    }

    /// 1oom `game_design_update_engines`: engines needed to power everything else.
    pub fn update_engines(&mut self) {
        let h = self.h();
        let mut power: u32 = 0;
        for s in 0..WEAPON_SLOTS {
            if self.weapons[s] != 0 {
                power +=
                    self.weapon_counts[s] as u32 * WEAPONS[self.weapons[s] as usize].power as u32;
            }
        }
        power += HULLS[h].power as u32 * (self.man as u32 + 1);
        for &s in &self.specials {
            power += SPECIALS[s as usize].power[h] as u32;
        }
        power += COMPS[self.comp as usize].power[h] as u32;
        power += JAMMERS[self.jammer as usize].power[h] as u32;
        power += SHIELDS[self.shield as usize].power[h] as u32;
        self.engines = (power / (ENGINES[self.engine as usize].power as u32 / 10)).max(1);
    }

    /// 1oom `game_design_set_hp`.
    pub fn set_hp(&mut self) {
        self.hp = HULLS[self.h()].hits as u32 * ARMOR[self.armor as usize].armor as u32 / 100;
    }

    pub fn update(&mut self) {
        self.compact();
        self.update_engines();
        self.set_hp();
    }

    /// 1oom `game_design_compact_slots`.
    pub fn compact(&mut self) {
        let mut sp: Vec<u8> = self.specials.iter().copied().filter(|&s| s != 0).collect();
        sp.resize(SPECIAL_SLOTS, 0);
        self.specials.copy_from_slice(&sp);
        let mut w: Vec<(u8, u8)> = (0..WEAPON_SLOTS)
            .map(|i| (self.weapons[i], self.weapon_counts[i]))
            .filter(|&(t, n)| t != 0 && n != 0)
            .collect();
        w.resize(WEAPON_SLOTS, (0, 0));
        for (i, (t, n)) in w.into_iter().enumerate() {
            self.weapons[i] = t;
            self.weapon_counts[i] = n;
        }
    }

    pub fn speed(&self) -> i32 {
        ENGINES[self.engine as usize].warp as i32
    }

    pub fn has_flag(&self, flag: u16) -> bool {
        self.specials
            .iter()
            .any(|&s| SPECIALS[s as usize].flags & flag != 0)
    }

    pub fn has_kind(&self, kind: u8) -> bool {
        self.specials
            .iter()
            .any(|&s| s != 0 && SPECIALS[s as usize].kind == kind)
    }

    /// Worst planet type a colony base on this design can settle (1oom `game_turn_explore`).
    pub fn colony_type(&self) -> Option<PlanetType> {
        self.specials
            .iter()
            .filter(|&&s| (2..=8).contains(&s))
            .map(|&s| PlanetType::from_index(PlanetType::Minimal.index() - (s as usize - 2)))
            .min()
    }

    pub fn armed(&self) -> bool {
        (0..WEAPON_SLOTS).any(|i| self.weapons[i] != 0 && self.weapon_counts[i] > 0)
    }

    pub fn bombs(&self) -> bool {
        (0..WEAPON_SLOTS)
            .any(|i| self.weapon_counts[i] > 0 && WEAPONS[self.weapons[i] as usize].bomb)
    }

    pub fn extra_range(&self) -> i32 {
        if self.has_kind(KIND_FUEL) {
            3
        } else {
            0
        }
    }

    /// Rough combat value used by the AI: damage per turn times hit points.
    pub fn strength(&self) -> i64 {
        let mut dmg: i64 = 0;
        for i in 0..WEAPON_SLOTS {
            let w = &WEAPONS[self.weapons[i] as usize];
            if self.weapon_counts[i] == 0 || w.bomb {
                continue;
            }
            let avg = (w.damage_min as i64 + w.damage_max as i64) / 2
                * w.damage_mul as i64
                * (w.streams.max(1) as i64);
            dmg += avg * w.fire.max(1) as i64 * self.weapon_counts[i] as i64;
        }
        let comp = COMPS[self.comp as usize].level as i64;
        dmg * (10 + comp) / 10
            * (self.hp.max(1) as i64 + SHIELDS[self.shield as usize].absorb as i64 * 5)
    }
}

/// Why a design cannot be built; the UI shows `design.error.<code>`.
pub fn validate(d: &ShipDesign, tech: &Tech) -> Result<(), &'static str> {
    if d.name.trim().is_empty() {
        return Err("no_name");
    }
    if d.hull > HULL_HUGE
        || d.engine as usize >= ENGINES.len()
        || d.comp as usize >= COMPS.len()
        || d.jammer as usize >= JAMMERS.len()
        || d.shield as usize >= SHIELDS.len()
        || d.armor as usize >= ARMOR.len()
        || d.weapons.iter().any(|&w| w as usize >= WEAPONS.len())
        || d.specials.iter().any(|&s| s as usize >= SPECIALS.len())
        || d.weapon_counts.iter().any(|&n| n > 99)
    {
        return Err("invalid");
    }
    let known = |field: usize, n: u8| n == 0 || tech.has(field, n);
    let ok = known(FIELD_PROPULSION, ENGINES[d.engine as usize].tech)
        && (d.comp == 0 || known(FIELD_COMPUTER, COMPS[d.comp as usize].tech))
        && (d.jammer == 0 || known(FIELD_COMPUTER, JAMMERS[d.jammer as usize].tech))
        && (d.shield == 0 || known(FIELD_FORCE_FIELD, SHIELDS[d.shield as usize].tech))
        && known(FIELD_CONSTRUCTION, ARMOR[d.armor as usize].tech)
        && d.weapons
            .iter()
            .all(|&w| w == 0 || known(weapon_field(w as usize), WEAPONS[w as usize].tech))
        && d.specials.iter().all(|&s| {
            s == 0
                || known(
                    SPECIALS[s as usize].field as usize,
                    SPECIALS[s as usize].tech,
                )
        });
    if !ok {
        return Err("unknown_part");
    }
    if d.man > d.engine {
        return Err("maneuver");
    }
    let kinds: Vec<u8> = d
        .specials
        .iter()
        .filter(|&&s| s != 0)
        .map(|&s| SPECIALS[s as usize].kind)
        .collect();
    if (1..kinds.len()).any(|i| kinds[..i].contains(&kinds[i])) {
        return Err("duplicate_special");
    }
    let mut d = d.clone();
    d.update();
    if d.free_space(&tech.level) < 0 {
        return Err("no_space");
    }
    Ok(())
}

/// Parts an empire knows, for the design screen: indices into the `data` tables.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct KnownParts {
    pub weapons: Vec<u8>,
    pub engines: Vec<u8>,
    pub comps: Vec<u8>,
    pub jammers: Vec<u8>,
    pub shields: Vec<u8>,
    pub armor: Vec<u8>,
    pub specials: Vec<u8>,
}

pub fn known_parts(tech: &Tech) -> KnownParts {
    let idx = |n: usize, ok: &dyn Fn(usize) -> bool| -> Vec<u8> {
        (0..n).filter(|&i| ok(i)).map(|i| i as u8).collect()
    };
    KnownParts {
        weapons: idx(WEAPONS.len(), &|i| {
            i > 0 && tech.has(weapon_field(i), WEAPONS[i].tech)
        }),
        engines: idx(ENGINES.len(), &|i| {
            tech.has(FIELD_PROPULSION, ENGINES[i].tech)
        }),
        comps: idx(COMPS.len(), &|i| {
            i > 0 && tech.has(FIELD_COMPUTER, COMPS[i].tech)
        }),
        jammers: idx(JAMMERS.len(), &|i| {
            i > 0 && tech.has(FIELD_COMPUTER, JAMMERS[i].tech)
        }),
        shields: idx(SHIELDS.len(), &|i| {
            i > 0 && tech.has(FIELD_FORCE_FIELD, SHIELDS[i].tech)
        }),
        armor: idx(ARMOR.len(), &|i| {
            tech.has(FIELD_CONSTRUCTION, ARMOR[i].tech)
        }),
        specials: idx(SPECIALS.len(), &|i| {
            i > 0 && tech.has(SPECIALS[i].field as usize, SPECIALS[i].tech)
        }),
    }
}

/// Computed figures of a design for the current tech, shown in the design screen.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DesignStats {
    pub cost: u32,
    pub space: i32,
    pub free_space: i32,
    pub engines: u32,
    pub hp: u32,
    pub speed: i32,
    pub attack_level: u8,
    pub defense: i32,
    pub missile_defense: i32,
    pub absorb: u8,
    /// Most of each weapon slot that fits with the rest of the design.
    pub max_weapons: [u8; WEAPON_SLOTS],
    pub error: Option<&'static str>,
}

pub fn stats(d: &ShipDesign, tech: &Tech) -> DesignStats {
    let mut d = d.clone();
    d.update_engines();
    d.set_hp();
    let level = &tech.level;
    let mut max_weapons = [0u8; WEAPON_SLOTS];
    for slot in 0..WEAPON_SLOTS {
        if d.weapons[slot] == 0 {
            continue;
        }
        let mut t = d.clone();
        let mut n = 0u8;
        for k in 1..=99u8 {
            t.weapon_counts[slot] = k;
            t.update_engines();
            if t.free_space(level) < 0 {
                break;
            }
            n = k;
        }
        max_weapons[slot] = n;
    }
    let extra_man = d
        .specials
        .iter()
        .map(|&s| SPECIALS[s as usize].extra_man as i32)
        .max()
        .unwrap_or(0);
    let defense = d.man as i32 + HULLS[d.h()].defense as i32 + extra_man + 1;
    DesignStats {
        cost: d.cost(level),
        space: d.hull_space(level),
        free_space: d.free_space(level),
        engines: d.engines,
        hp: d.hp,
        speed: d.speed(),
        attack_level: COMPS[d.comp as usize].level + u8::from(d.has_flag(FLAG_SCANNER)),
        defense,
        missile_defense: defense + JAMMERS[d.jammer as usize].level as i32,
        absorb: SHIELDS[d.shield as usize].absorb,
        max_weapons,
        error: validate(&d, tech).err(),
    }
}

/// Builds the best design of a role with the parts an empire knows (used by the AI).
pub fn auto_design(tech: &Tech, role: Role, hull: u8, name: String) -> ShipDesign {
    let parts = known_parts(tech);
    let level = &tech.level;
    let best =
        |v: &[u8], score: &dyn Fn(u8) -> i64| v.iter().copied().max_by_key(|&i| (score(i), i));
    let mut d = ShipDesign {
        name,
        hull,
        look: hull * 6,
        weapons: [0; 4],
        weapon_counts: [0; 4],
        engine: best(&parts.engines, &|i| ENGINES[i as usize].warp as i64).unwrap_or(0),
        engines: 0,
        specials: [0; 3],
        shield: 0,
        jammer: 0,
        comp: 0,
        armor: 0,
        man: 0,
        hp: 0,
        cost_bc: 0,
        space: 0,
    };
    let fits = |d: &ShipDesign| {
        let mut t = d.clone();
        t.update_engines();
        t.free_space(level) >= 0
    };
    match role {
        Role::Scout => {
            if let Some(&s) = parts
                .specials
                .iter()
                .find(|&&s| SPECIALS[s as usize].kind == KIND_FUEL)
            {
                d.specials[0] = s;
            }
        }
        Role::Colony => {
            let base = parts
                .specials
                .iter()
                .copied()
                .filter(|&s| (2..=8).contains(&s))
                .max();
            d.specials[0] = base.unwrap_or(2);
        }
        Role::Warship | Role::Bomber => {
            d.man = d.engine / 2;
            for (slot, value) in [
                (0, best(&parts.armor, &|i| ARMOR[i as usize].armor as i64)),
                (
                    1,
                    best(&parts.shields, &|i| SHIELDS[i as usize].absorb as i64),
                ),
                (2, best(&parts.comps, &|i| COMPS[i as usize].level as i64)),
            ] {
                let Some(v) = value else { continue };
                let mut t = d.clone();
                match slot {
                    0 => t.armor = v,
                    1 => t.shield = v,
                    _ => t.comp = v,
                }
                if fits(&t) {
                    d = t;
                }
            }
            let beam = best(&parts.weapons, &|i| {
                let w = &WEAPONS[i as usize];
                if w.bomb || w.bio || w.damage_min == w.damage_max {
                    return -1;
                }
                (w.damage_min as i64 + w.damage_max as i64) * w.fire as i64 * 100
                    / w.space.max(1) as i64
                    + (w.damage_min as i64 + w.damage_max as i64) * 4
            });
            let missile = best(&parts.weapons, &|i| {
                let w = &WEAPONS[i as usize];
                if !w.is_missile() || w.shots != 5 {
                    return -1;
                }
                w.damage_max as i64 * w.streams.max(1) as i64
            });
            let bomb = best(&parts.weapons, &|i| {
                let w = &WEAPONS[i as usize];
                if !w.bomb || w.bio {
                    return -1;
                }
                w.damage_max as i64
            });
            let mut slots: Vec<u8> = Vec::new();
            if role == Role::Bomber {
                slots.extend(bomb);
            }
            slots.extend(beam);
            if role == Role::Warship {
                slots.extend(missile.filter(|&m| WEAPONS[m as usize].damage_max > 0));
            }
            for (i, &w) in slots.iter().enumerate().take(WEAPON_SLOTS) {
                d.weapons[i] = w;
            }
            // Share the space: each weapon slot takes as many as fit, the first slot half.
            for i in 0..slots.len().min(WEAPON_SLOTS) {
                let share = if i + 1 < slots.len() { 2 } else { 1 };
                let mut n = 0u8;
                for k in 1..=99u8 {
                    let mut t = d.clone();
                    t.weapon_counts[i] = k;
                    if !fits(&t) {
                        break;
                    }
                    n = k;
                }
                d.weapon_counts[i] = (n / share).max(u8::from(n > 0));
            }
            if let Some(j) = best(&parts.jammers, &|i| JAMMERS[i as usize].level as i64) {
                let mut t = d.clone();
                t.jammer = j;
                if fits(&t) {
                    d = t;
                }
            }
        }
    }
    d.update();
    while d.free_space(level) < 0 && d.man > 0 {
        d.man -= 1;
        d.update();
    }
    d
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Scout,
    Colony,
    Warship,
    Bomber,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_designs_cost_what_moo1_says() {
        // 1oom game_num_fix_starting_ships: costs and engines with the starting techs.
        let tech = Tech::new();
        let costs: Vec<u32> = start_designs()
            .iter()
            .map(|d| d.cost(&tech.level))
            .collect();
        assert_eq!(costs, vec![8, 14, 74, 65, 570]);
        let engines: Vec<u32> = start_designs().iter().map(|d| d.engines).collect();
        assert_eq!(engines, vec![2, 27, 110, 85, 100]);
        for d in start_designs() {
            assert_eq!(validate(&d, &tech), Ok(()), "{}", d.name);
            assert!(d.free_space(&tech.level) >= 0);
        }
        assert_eq!(start_designs()[4].colony_type(), Some(PlanetType::Minimal));
        assert_eq!(start_designs()[0].extra_range(), 3);
    }

    #[test]
    fn miniaturization_lowers_cost() {
        let mut tech = Tech::new();
        let d = &start_designs()[2];
        let c1 = d.cost(&tech.level);
        tech.level = [20; NUM_FIELDS];
        assert!(d.cost(&tech.level) < c1);
        assert!(d.free_space(&tech.level) > d.free_space(&Tech::new().level));
    }

    #[test]
    fn validation_catches_errors() {
        let tech = Tech::new();
        let mut d = start_designs()[1].clone();
        d.weapon_counts[0] = 50;
        assert_eq!(validate(&d, &tech), Err("no_space"));
        let mut d = start_designs()[1].clone();
        d.weapons[0] = 62; // death ray
        assert_eq!(validate(&d, &tech), Err("unknown_part"));
        let mut d = start_designs()[1].clone();
        d.name = " ".into();
        assert_eq!(validate(&d, &tech), Err("no_name"));
    }

    #[test]
    fn auto_designs_fit() {
        let mut tech = Tech::new();
        for role in [Role::Scout, Role::Colony, Role::Warship, Role::Bomber] {
            for hull in 0..4 {
                if role == Role::Colony && hull < HULL_LARGE {
                    continue; // a colony base needs 700 space
                }
                let d = auto_design(&tech, role, hull, "X".into());
                assert_eq!(validate(&d, &tech), Ok(()), "{role:?} {hull}");
            }
        }
        for n in 1..=30 {
            for f in 0..NUM_FIELDS {
                if crate::tech::exists(f, n) {
                    tech.add(f, n);
                }
            }
        }
        tech.level = [30; NUM_FIELDS];
        let d = auto_design(&tech, Role::Warship, HULL_LARGE, "Y".into());
        assert_eq!(validate(&d, &tech), Ok(()));
        assert!(d.armed() && d.armor > 0 && d.shield > 0 && d.comp > 0);
    }
}
