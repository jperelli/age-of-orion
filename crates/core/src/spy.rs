//! Espionage and sabotage (1oom `game_spy.c`).
//!
//! Spies are bought with a share of production per target empire. Each turn they try to
//! steal a tech (espionage) or destroy factories, missile bases or stir up rebels
//! (sabotage); the target's computer level and security catch them. A human spy owner
//! chooses what to steal or sabotage from the offers left in `State::pending`.

use serde::{Deserialize, Serialize};

use crate::data::TECH_GROUPS;
use crate::diplo::{self, msg, rel, rel_mut, SpyMode, TechRef};
use crate::state::*;
use crate::tech::{self, Source};
use crate::types::*;

pub const TECH_SPY_MAX: usize = 6;

/// Tech groups (1oom `tech_group_t`).
pub mod group {
    pub const SINGULAR: u8 = 0;
    pub const ROBOTIC_CONTROLS: u8 = 3;
    pub const SPACE_SCANNER: u8 = 4;
    pub const REDUCED_WASTE: u8 = 5;
    pub const INDUSTRIAL_TECH: u8 = 6;
    pub const PLANETARY_SHIELD: u8 = 10;
    pub const TERRAFORMING: u8 = 12;
    pub const CONTROLLED_ENV: u8 = 13;
    pub const ECO_RESTORATION: u8 = 14;
    pub const PERSONAL_ARMOR: u8 = 15;
    pub const PERSONAL_SHIELD: u8 = 16;
    pub const FUEL_CELLS: u8 = 18;
    pub const PERSONAL_WEAPONS: u8 = 21;
}

pub fn group_of(field: usize, tech: u8) -> u8 {
    if tech <= 50 {
        TECH_GROUPS[field][tech as usize][0]
    } else {
        group::SINGULAR
    }
}

pub fn tier_of(field: usize, tech: u8) -> u8 {
    if tech <= 50 {
        TECH_GROUPS[field][tech as usize][1]
    } else {
        1
    }
}

/// Groups where a better tech replaces the lesser ones.
fn superseding(g: u8) -> bool {
    use group::*;
    matches!(
        g,
        ROBOTIC_CONTROLS
            | SPACE_SCANNER
            | REDUCED_WASTE
            | INDUSTRIAL_TECH
            | PLANETARY_SHIELD
            | TERRAFORMING
            | CONTROLLED_ENV
            | ECO_RESTORATION
            | PERSONAL_ARMOR
            | PERSONAL_SHIELD
            | FUEL_CELLS
            | PERSONAL_WEAPONS
    )
}

fn useless_for_silicoids(g: u8) -> bool {
    matches!(
        g,
        group::REDUCED_WASTE | group::CONTROLLED_ENV | group::ECO_RESTORATION
    )
}

/// 1oom `game_spy_esp_sub2`: per field, the target's techs the spy could use, ignoring the
/// target's `skip` newest techs.
pub fn useful_techs(st: &State, spy: EmpireId, target: EmpireId, skip: usize) -> Vec<Vec<u8>> {
    let es = &st.empires[spy as usize].tech;
    let et = &st.empires[target as usize].tech;
    let silicoid = st.empires[spy as usize].race == Race::Silicoid;
    (0..NUM_FIELDS)
        .map(|f| {
            let theirs = &et.completed[f];
            let n = theirs.len().saturating_sub(skip);
            theirs[..n]
                .iter()
                .copied()
                .filter(|&t| {
                    let g = group_of(f, t);
                    let mut have = es.has(f, t);
                    if superseding(g) {
                        let best = es.completed[f]
                            .iter()
                            .filter(|&&m| group_of(f, m) == g)
                            .map(|&m| tier_of(f, m))
                            .max()
                            .unwrap_or(0);
                        if best >= tier_of(f, t) {
                            have = true;
                        }
                    }
                    if silicoid && useless_for_silicoids(g) {
                        have = true;
                    }
                    !have
                })
                .collect()
        })
        .collect()
}

/// 1oom `game_spy_esp_get_value`: how much `player` wants a tech.
pub fn value(st: &State, field: usize, tech: u8, player: EmpireId) -> i32 {
    let e = &st.empires[player as usize];
    let g = group_of(field, tech);
    let maxti = e.tech.completed[field]
        .iter()
        .copied()
        .filter(|&t| group_of(field, t) == g)
        .max_by_key(|&t| tier_of(field, t))
        .unwrap_or(0);
    let mut v = tech as i32 * tech as i32;
    match g {
        group::SINGULAR => v *= 10,
        group::ROBOTIC_CONTROLS => v *= 6,
        group::REDUCED_WASTE | group::ECO_RESTORATION => v *= 3,
        _ => {}
    }
    if maxti > tech {
        v /= 4;
    }
    if maxti == tech {
        v = 0;
    }
    if e.race == Race::Silicoid && useless_for_silicoids(g) {
        v = 0;
    }
    if field == FIELD_WEAPON && tech == 36 {
        v = 30000;
    }
    v
}

/// A tech one empire could take or get from another, with its value to the taker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    pub field: u8,
    pub tech: u8,
    pub value: i32,
}

/// 1oom `game_spy_esp_sub1`: up to six random useful techs worth at least `min_value`.
pub fn pick_techs(
    st: &mut State,
    spy: EmpireId,
    target: EmpireId,
    min_value: i32,
    skip: usize,
) -> Vec<Offer> {
    let lists = useful_techs(st, spy, target, skip);
    let mut out: Vec<Offer> = Vec::new();
    for _ in 0..500 {
        if out.len() >= TECH_SPY_MAX {
            break;
        }
        let f = st.rng.below(NUM_FIELDS as u16) as usize;
        if lists[f].is_empty() {
            continue;
        }
        let t = lists[f][st.rng.below(lists[f].len() as u16) as usize];
        let have = out.iter().any(|o| o.field as usize == f && o.tech == t);
        let v = value(st, f, t, spy);
        if !have && v != 0 && v >= min_value {
            out.push(Offer {
                field: f as u8,
                tech: t,
                value: v,
            });
        }
    }
    out
}

fn frame_random(st: &mut State, spy: EmpireId, target: EmpireId) -> Option<EmpireId> {
    let n = st.empires.len() as u8;
    let goats: Vec<EmpireId> = (0..n)
        .filter(|&i| i != spy && i != target && rel(st, target, i).contact)
        .collect();
    if goats.is_empty() {
        None
    } else {
        Some(goats[st.rng.below(goats.len() as u16) as usize])
    }
}

fn random_planet(st: &mut State, owner: EmpireId) -> Option<PlanetId> {
    let v: Vec<PlanetId> = st.colonies(owner).map(|(i, _)| i).collect();
    if v.is_empty() {
        None
    } else {
        Some(v[st.rng.below(v.len() as u16) as usize])
    }
}

/// What a human's spies achieved this turn, waiting for the player's choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpyChance {
    /// Choose a field to steal from: one tech per field.
    Steal {
        target: EmpireId,
        techs: Vec<TechRef>,
        framed: bool,
        /// Relation penalty if not framed.
        penalty: i32,
    },
    /// Choose a planet and what to sabotage.
    Sabotage {
        target: EmpireId,
        strength: i32,
        framed: bool,
        penalty: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SabotageKind {
    Factories,
    Bases,
    Revolt,
}

/// 1oom `game_spy_build`: spending buys spies, each costing twice the previous one.
pub fn build(st: &mut State) {
    for e in st.empires.iter_mut() {
        let mut base = e.tech.level[FIELD_COMPUTER] as i32 * 2 + 25;
        if e.race == Race::Darlok {
            base /= 2;
        }
        let total = e.total_prod;
        for r in e.rel.iter_mut() {
            if r.spying == 0 {
                continue;
            }
            let mut fund = total * r.spying / 1000 + r.spyfund;
            let mut cost = base;
            while fund >= cost {
                r.spies += 1;
                fund -= cost;
                cost *= 2;
            }
            r.spyfund = fund;
        }
    }
}

/// 1oom `game_spy_report`: spies tell the newest tech of each field.
pub fn report(st: &mut State) {
    let n = st.empires.len();
    let year = st.year;
    for i in 0..n {
        for j in 0..n {
            if i != j && st.empires[i].rel[j].spies > 0 {
                let rep: [u8; NUM_FIELDS] = std::array::from_fn(|f| {
                    st.empires[j].tech.completed[f]
                        .iter()
                        .copied()
                        .max()
                        .unwrap_or(1)
                });
                let r = &mut st.empires[i].rel[j];
                r.spy_report_year = year;
                r.spy_report = rep;
            }
        }
    }
}

/// 1oom `game_spy_turn`: spies are caught or succeed.
pub fn turn(st: &mut State) {
    let n = st.empires.len() as u8;
    let alive: Vec<bool> = (0..n).map(|i| st.colonies(i).next().is_some()).collect();
    let comp: Vec<(i32, i32)> = st
        .empires
        .iter()
        .map(|e| {
            let c = e.tech.level[FIELD_COMPUTER] as i32;
            if e.race == Race::Darlok {
                (c + 20, c + 30)
            } else {
                (c, c)
            }
        })
        .collect();
    for spy in 0..n {
        for target in 0..n {
            if spy == target || !alive[spy as usize] || !alive[target as usize] {
                continue;
            }
            let mode = rel(st, spy, target).spymode;
            let mut spies = rel(st, spy, target).spies;
            let mut dt1 = (comp[spy as usize].1 - comp[target as usize].0).max(0);
            let dt2 = (comp[target as usize].0 - comp[spy as usize].1).max(0)
                + st.empires[target as usize].security / 5;
            if mode == SpyMode::Hide {
                dt1 += 30;
            }
            let (mut caught, mut fail) = (0, 0);
            let mut any_caught = false;
            if mode == SpyMode::Hide {
                for _ in 0..spies {
                    if st.rng.one_to(100) as i32 + dt2 > 85 {
                        caught += 1;
                    }
                }
                spies = 0;
            } else {
                for _ in 0..spies {
                    let r = st.rng.one_to(100) as i32 + dt2;
                    if r > 99 {
                        caught = spies;
                        fail = spies;
                        any_caught = true;
                        break;
                    } else if r > 70 {
                        caught += 1;
                        fail += 1;
                        any_caught = true;
                    } else if r > 50 {
                        fail += 1;
                        any_caught = true;
                    }
                }
            }
            spies = (spies - fail).max(0);
            let caught = caught.min(rel(st, spy, target).spies);
            rel_mut(st, spy, target).spies -= caught;
            let r = st.rng.one_to(100) as i32 + dt1;
            let success = if r > 84 { spies } else { 0 };
            let framed = r > 100;
            if caught > 0 {
                st.push_event(
                    vec![spy, target],
                    EventKind::SpiesCaught {
                        spy,
                        target,
                        count: caught,
                    },
                );
            }
            match mode {
                SpyMode::Espionage => espionage(st, spy, target, framed, success, any_caught),
                SpyMode::Sabotage => sabotage(st, spy, target, framed, success, any_caught),
                SpyMode::Hide => {}
            }
        }
    }
}

/// 1oom `game_spy_espionage`.
fn espionage(
    st: &mut State,
    spy: EmpireId,
    target: EmpireId,
    framed: bool,
    spies: i32,
    any_caught: bool,
) {
    if spies <= 0 {
        return;
    }
    let mut rmax = 0;
    for _ in 0..spies {
        rmax = rmax.max(st.rng.one_to(100) as i32);
    }
    let tmax = st.empires[target as usize]
        .tech
        .level
        .iter()
        .copied()
        .max()
        .unwrap_or(1) as i32;
    let rmax = rmax * tmax / 100;
    let mut offers = pick_techs(st, spy, target, 0, 0);
    offers.retain(|o| o.tech as i32 <= rmax);
    if offers.is_empty() {
        return;
    }
    let human_spy = !st.empires[spy as usize].ai;
    let human_target = !st.empires[target as usize].ai;
    if human_spy {
        let mut v = 0;
        for _ in 0..5 {
            v += st.rng.one_to(12) as i32;
        }
        // 1oom game_spy_esp_human: five draws, one tech per field.
        let mut per_field: [u8; NUM_FIELDS] = [0; NUM_FIELDS];
        for _ in 0..5 {
            let mut o = pick_techs(st, spy, target, 0, 0);
            o.retain(|o| o.tech as i32 <= rmax);
            for x in o {
                if per_field[x.field as usize] == 0 {
                    per_field[x.field as usize] = x.tech;
                }
            }
        }
        let techs: Vec<TechRef> = (0..NUM_FIELDS)
            .filter(|&f| per_field[f] != 0)
            .map(|f| (f as u8, per_field[f]))
            .collect();
        if !techs.is_empty() {
            st.pending.spy.push((
                spy,
                SpyChance::Steal {
                    target,
                    techs,
                    framed,
                    penalty: if any_caught && !framed { v } else { 0 },
                },
            ));
        }
        return;
    }
    let o = offers[0];
    tech::grant_from(
        st,
        spy,
        o.field as usize,
        o.tech,
        Source::Espionage,
        Some(target),
    );
    if human_target {
        let blamed = if framed {
            frame_random(st, spy, target)
        } else if any_caught {
            Some(spy)
        } else {
            None
        };
        st.push_event(
            vec![target],
            EventKind::TechStolen {
                field: TechField::ALL[o.field as usize],
                tech: o.tech,
                by: blamed,
            },
        );
    } else if framed && st.rng.below(2) == 0 {
        let n = st.empires.len() as u8;
        let goats: Vec<EmpireId> = (0..n)
            .filter(|&i| {
                i != target
                    && !st.empires[i as usize].ai
                    && st.empires[i as usize].alive
                    && rel(st, target, i).contact
            })
            .collect();
        if !goats.is_empty() {
            let pi = goats[if goats.len() > 1 {
                st.rng.below(goats.len() as u16) as usize
            } else {
                0
            }];
            let v = -(st.rng.one_to(20) as i32 + 20);
            diplo::act(st, v, pi, target, msg::SPY_FRAMED, 0, o.field as i32);
        }
    }
}

/// 1oom `game_spy_sabotage`.
fn sabotage(
    st: &mut State,
    spy: EmpireId,
    target: EmpireId,
    framed: bool,
    spies: i32,
    any_caught: bool,
) {
    let rcaught = if any_caught {
        st.rng.one_to(20) as i32 + 20
    } else {
        0
    };
    let num = (st.empires[spy as usize].tech.level[FIELD_WEAPON] as i32 + 9) * spies / 10;
    let mut v8 = 0;
    for _ in 0..num {
        v8 += st.rng.one_to(5) as i32;
    }
    let bases = st.rng.below(2) != 0;
    let Some(pl) = random_planet(st, target) else {
        return;
    };
    if v8 <= 0 {
        return;
    }
    if !st.empires[spy as usize].ai {
        st.pending.spy.push((
            spy,
            SpyChance::Sabotage {
                target,
                strength: v8,
                framed,
                penalty: rcaught,
            },
        ));
        return;
    }
    if !st.empires[target as usize].ai {
        if st.rng.below(4) == 0 {
            let p = &mut st.planets[pl];
            let n = (p.pop * (v8 / 2) / 100).max(1).min(p.pop);
            add_rebels(p, n);
            st.push_event(
                vec![target],
                EventKind::Sabotaged {
                    planet: pl,
                    kind: SabotageKind::Revolt,
                    count: n,
                    by: (rcaught > 0 && !framed).then_some(spy),
                },
            );
        } else {
            let p = &st.planets[pl];
            let n = if bases {
                (v8 / 5).min(p.bases)
            } else {
                v8.min(p.factories)
            };
            if n > 0 {
                let by = if framed {
                    frame_random(st, spy, target)
                } else {
                    (rcaught > 0).then_some(spy)
                };
                let p = &mut st.planets[pl];
                if bases {
                    p.bases -= n;
                } else {
                    p.factories -= n;
                }
                st.push_event(
                    vec![target],
                    EventKind::Sabotaged {
                        planet: pl,
                        kind: if bases {
                            SabotageKind::Bases
                        } else {
                            SabotageKind::Factories
                        },
                        count: n,
                        by,
                    },
                );
            }
        }
        return;
    }
    let p = &mut st.planets[pl];
    if bases {
        p.bases = (p.bases - v8 / 5).max(0);
    } else {
        p.factories = (p.factories - v8).max(0);
    }
    if !framed {
        diplo::act(
            st,
            -rcaught,
            spy,
            target,
            msg::SABOTAGE,
            pl,
            i32::from(bases),
        );
    } else if let Some(p2) = frame_random(st, spy, target) {
        let r = -(st.rng.one_to(16) as i32 + st.rng.one_to(16) as i32);
        diplo::act(
            st,
            -r,
            p2,
            target,
            msg::SABOTAGE_FRAMED,
            pl,
            i32::from(bases),
        );
    }
}

/// Rebels on a planet; half the population in revolt is a rebellion.
pub fn add_rebels(p: &mut Planet, n: i32) {
    p.rebels = (p.rebels + n).min(p.pop);
    if p.rebels >= p.pop / 2 {
        p.unrest = Unrest::Rebellion;
        p.unrest_reported = false;
        p.rebels = p.pop / 2;
    }
}

/// A human steals the tech of `field` from a pending espionage success
/// (1oom `game_spy_esp_human`).
pub fn steal(st: &mut State, spy: EmpireId, idx: usize, field: usize) -> bool {
    let Some((
        owner,
        SpyChance::Steal {
            target,
            techs,
            framed,
            penalty,
        },
    )) = st.pending.spy.get(idx).cloned()
    else {
        return false;
    };
    let Some(&(_, t)) = techs.iter().find(|(f, _)| *f as usize == field) else {
        return false;
    };
    if owner != spy {
        return false;
    }
    st.pending.spy.remove(idx);
    tech::grant_from(st, spy, field, t, Source::Espionage, Some(target));
    if !framed {
        diplo::act(st, -penalty, spy, target, msg::SPY_CAUGHT, 0, field as i32);
    }
    true
}

/// A human's saboteurs strike a planet (1oom `game_spy_sab_human`).
pub fn do_sabotage(
    st: &mut State,
    spy: EmpireId,
    idx: usize,
    planet: PlanetId,
    kind: SabotageKind,
) -> Option<i32> {
    let Some((
        owner,
        SpyChance::Sabotage {
            target,
            strength,
            framed,
            penalty,
        },
    )) = st.pending.spy.get(idx).cloned()
    else {
        return None;
    };
    if owner != spy || planet >= st.planets.len() || st.planets[planet].owner != Some(target) {
        return None;
    }
    st.pending.spy.remove(idx);
    let mut snum = strength;
    match kind {
        SabotageKind::Factories => {
            let p = &mut st.planets[planet];
            snum = snum.min(p.factories);
            p.factories -= snum;
        }
        SabotageKind::Bases => {
            let mut n = 0;
            for _ in 0..snum {
                if st.rng.below(4) == 0 {
                    n += 1;
                }
            }
            let p = &mut st.planets[planet];
            snum = n.min(p.bases);
            p.bases -= snum;
        }
        SabotageKind::Revolt => {
            let p = &mut st.planets[planet];
            snum = p.pop * snum / 200;
            if p.pop < snum {
                snum = p.pop - p.rebels;
            }
            snum = snum.max(1);
            add_rebels(p, snum);
        }
    }
    st.empires[spy as usize].explored[planet] = true;
    if kind != SabotageKind::Revolt {
        if !framed {
            diplo::act(
                st,
                -penalty,
                spy,
                target,
                msg::SABOTAGE,
                planet,
                kind as i32,
            );
        } else if snum != 0 && kind == SabotageKind::Bases {
            if let Some(o) = frame_random(st, spy, target) {
                let v = -(st.rng.one_to(12) as i32 + st.rng.one_to(12) as i32);
                diplo::act(st, v, o, target, msg::SABOTAGE_FRAMED, planet, kind as i32);
            }
        }
    }
    Some(snum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Game, NewGameOptions};

    #[test]
    fn spies_are_bought_and_report() {
        let mut g = Game::new(&NewGameOptions {
            seed: 8,
            players: 2,
            ..Default::default()
        })
        .unwrap();
        let st = g.state_mut();
        st.empires[0].total_prod = 1000;
        rel_mut(st, 0, 1).spying = 100;
        build(st);
        assert!(rel(st, 0, 1).spies > 0);
        report(st);
        assert_eq!(rel(st, 0, 1).spy_report_year, st.year);
    }

    #[test]
    fn useful_techs_skip_lesser_series_techs() {
        let mut g = Game::new(&NewGameOptions::default()).unwrap();
        let st = g.state_mut();
        st.empires[1].tech.add(FIELD_COMPUTER, 8); // robotic controls III
        st.empires[0].tech.add(FIELD_COMPUTER, 18); // robotic controls IV
        let u = useful_techs(st, 0, 1, 0);
        assert!(!u[FIELD_COMPUTER].contains(&8));
        let u = useful_techs(st, 1, 0, 0);
        assert!(u[FIELD_COMPUTER].contains(&18));
        assert!(value(st, FIELD_COMPUTER, 18, 1) > 0);
        assert_eq!(value(st, FIELD_COMPUTER, 18, 0), 0);
    }
}
