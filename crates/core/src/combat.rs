//! Automatic resolution of space battles, bombing and ground combat.
//!
//! MOO1 plays battles tactically on a grid (1oom `game_battle*.c`). This first version
//! resolves them in rounds of simultaneous fire; the tactical battle is on the roadmap.

use crate::rng::Rng;
use crate::state::*;
use crate::turn::{can_colonize, found_colony};
use crate::types::*;

const GUARDIAN: i32 = -1;

#[derive(Debug, Clone)]
struct Unit {
    /// Design index, or `None` for missile bases / the guardian.
    design: Option<usize>,
    count: u32,
    hp: i32,
    attack: i32,
    /// The guardian: `count` is its remaining hit points and `attack` does not scale.
    pooled: bool,
}

#[derive(Debug, Clone)]
struct Side {
    id: i32,
    units: Vec<Unit>,
    damage_taken_pct: i32,
    start_ships: u32,
}

impl Side {
    fn attack(&self) -> i64 {
        self.units
            .iter()
            .map(|u| match (u.pooled, u.count) {
                (_, 0) => 0,
                (true, _) => u.attack as i64,
                _ => u.attack as i64 * u.count as i64,
            })
            .sum()
    }

    fn alive(&self) -> bool {
        self.units.iter().any(|u| u.count > 0)
    }

    fn ships(&self) -> u32 {
        self.units
            .iter()
            .filter(|u| u.design.is_some())
            .map(|u| u.count)
            .sum()
    }

    /// Spreads `dmg` over the units in proportion to their total hit points.
    fn take(&mut self, dmg: i64, rng: &mut Rng) {
        let dmg = dmg * self.damage_taken_pct as i64 / 100;
        let total: i64 = self
            .units
            .iter()
            .map(|u| u.hp as i64 * u.count as i64)
            .sum();
        if total == 0 {
            return;
        }
        for u in self.units.iter_mut().filter(|u| u.count > 0) {
            let share = dmg * u.hp as i64 * u.count as i64 / total;
            let hp = u.hp.max(1) as i64;
            let mut kills = share / hp;
            let rem = (share % hp) as u16;
            if rem > 0 && rng.below(hp.min(u16::MAX as i64) as u16) < rem {
                kills += 1;
            }
            u.count = u.count.saturating_sub(kills.min(u32::MAX as i64) as u32);
        }
    }
}

/// Whether another empire (or the guardian) has armed forces at `planet`.
pub fn hostile_armed_at(st: &State, planet: PlanetId, empire: EmpireId) -> bool {
    if planet == st.orion && st.guardian_alive() {
        return true;
    }
    st.empires.iter().enumerate().any(|(i, e)| {
        i as u8 != empire
            && e.orbit[planet]
                .iter()
                .zip(&st.rules.designs)
                .any(|(&c, d)| c > 0 && d.armed())
    }) || st.planets[planet]
        .owner
        .is_some_and(|o| o != empire && st.planets[planet].bases > 0)
}

fn armed_ships(st: &State, empire: usize, planet: PlanetId) -> bool {
    st.empires[empire].orbit[planet]
        .iter()
        .zip(&st.rules.designs)
        .any(|(&c, d)| c > 0 && d.armed())
}

fn build_sides(st: &State, planet: PlanetId) -> Vec<Side> {
    let mut sides = Vec::new();
    let p = &st.planets[planet];
    for (ei, e) in st.empires.iter().enumerate() {
        let has_bases = p.owner == Some(ei as u8) && p.bases > 0;
        if !armed_ships(st, ei, planet) && !has_bases {
            continue;
        }
        let a = e.abilities();
        let mut units: Vec<Unit> = e.orbit[planet]
            .iter()
            .enumerate()
            .filter(|(_, &c)| c > 0)
            .map(|(d, &c)| {
                let des = &st.rules.designs[d];
                Unit {
                    design: Some(d),
                    count: c,
                    hp: (des.hp * a.hp_pct / 100).max(1),
                    attack: des.attack * a.attack_pct / 100,
                    pooled: false,
                }
            })
            .collect();
        if has_bases {
            units.push(Unit {
                design: None,
                count: p.bases as u32,
                hp: (st.rules.base_hp * a.hp_pct / 100) * (100 + p.shield * 5) / 100,
                attack: st.rules.base_attack * a.attack_pct / 100,
                pooled: false,
            });
        }
        let start_ships = units
            .iter()
            .filter(|u| u.design.is_some())
            .map(|u| u.count)
            .sum();
        sides.push(Side {
            id: ei as i32,
            units,
            damage_taken_pct: a.damage_taken_pct,
            start_ships,
        });
    }
    if planet == st.orion && st.guardian_alive() {
        sides.push(Side {
            id: GUARDIAN,
            units: vec![Unit {
                design: None,
                count: st.guardian_hp.max(0) as u32,
                hp: 1,
                attack: st.rules.guardian_attack,
                pooled: true,
            }],
            damage_taken_pct: 100,
            start_ships: 0,
        });
    }
    sides
}

/// Resolves every planet where two or more hostile armed sides meet.
pub fn battles(st: &mut State) {
    for planet in 0..st.planets.len() {
        let mut sides = build_sides(st, planet);
        if sides.len() >= 2 {
            fight(st, planet, &mut sides);
        }
        // Unarmed ships cannot stay where a single hostile armed side holds the orbit.
        let armed: Vec<i32> = build_sides(st, planet).iter().map(|s| s.id).collect();
        if armed.len() == 1 {
            for ei in 0..st.empires.len() {
                if ei as i32 != armed[0] && st.empires[ei].orbit[planet].iter().any(|&c| c > 0) {
                    let lost: u32 = st.empires[ei].orbit[planet].iter().sum();
                    st.empires[ei].orbit[planet].iter_mut().for_each(|c| *c = 0);
                    st.push_event(
                        participants(st, planet, &[armed[0], ei as i32]),
                        EventKind::Battle {
                            planet,
                            sides: vec![armed[0], ei as i32],
                            winner: Some(armed[0]),
                            losses: vec![0, lost],
                            bases_lost: 0,
                        },
                    );
                }
            }
        }
    }
}

fn participants(st: &State, planet: PlanetId, ids: &[i32]) -> Vec<EmpireId> {
    let mut v: Vec<EmpireId> = ids.iter().filter(|&&i| i >= 0).map(|&i| i as u8).collect();
    if let Some(o) = st.planets[planet].owner {
        if !v.contains(&o) {
            v.push(o);
        }
    }
    v
}

fn fight(st: &mut State, planet: PlanetId, sides: &mut [Side]) {
    let bases_before = st.planets[planet].bases;
    for _ in 0..st.rules.combat_rounds {
        let active: Vec<usize> = (0..sides.len())
            .filter(|&i| sides[i].alive() && sides[i].attack() > 0)
            .collect();
        let targets_exist =
            |i: usize, sides: &[Side]| (0..sides.len()).any(|j| j != i && sides[j].alive());
        if active.iter().filter(|&&i| targets_exist(i, sides)).count() == 0
            || sides.iter().filter(|s| s.alive()).count() < 2
        {
            break;
        }
        let mut hits: Vec<(usize, i64)> = Vec::new();
        for &i in &active {
            let targets: Vec<usize> = (0..sides.len())
                .filter(|&j| j != i && sides[j].alive())
                .collect();
            if targets.is_empty() {
                continue;
            }
            let t = targets[st.rng.below(targets.len() as u16) as usize];
            let dmg = sides[i].attack() * (75 + st.rng.below(51) as i64) / 100;
            hits.push((t, dmg));
        }
        for (t, dmg) in hits {
            sides[t].take(dmg, &mut st.rng);
        }
    }
    let armed_left: Vec<i32> = sides
        .iter()
        .filter(|s| s.alive() && s.attack() > 0)
        .map(|s| s.id)
        .collect();
    let winner = (armed_left.len() == 1).then(|| armed_left[0]);
    let mut losses = Vec::new();
    for s in sides.iter() {
        losses.push(s.start_ships - s.ships());
        if s.id == GUARDIAN {
            st.guardian_hp = s.units[0].count as i32;
            if st.guardian_hp <= 0 {
                if let Some(w) = winner.filter(|&w| w >= 0) {
                    st.push_event(
                        vec![],
                        EventKind::GuardianDefeated {
                            planet,
                            by: w as u8,
                        },
                    );
                }
            }
            continue;
        }
        let e = s.id as usize;
        st.empires[e].orbit[planet].iter_mut().for_each(|c| *c = 0);
        for u in &s.units {
            match u.design {
                Some(d) => st.empires[e].orbit[planet][d] = u.count,
                None => st.planets[planet].bases = u.count as i32,
            }
        }
    }
    let ids: Vec<i32> = sides.iter().map(|s| s.id).collect();
    st.push_event(
        participants(st, planet, &ids),
        EventKind::Battle {
            planet,
            sides: ids,
            winner,
            losses,
            bases_lost: bases_before - st.planets[planet].bases,
        },
    );
}

/// Ships with bombs that hold the orbit of an undefended enemy colony bomb it.
pub fn bomb(st: &mut State) {
    for planet in 0..st.planets.len() {
        let Some(owner) = st.planets[planet].owner else {
            continue;
        };
        if st.planets[planet].bases > 0 || armed_ships(st, owner as usize, planet) {
            continue;
        }
        for ei in 0..st.empires.len() {
            if ei == owner as usize || st.planets[planet].owner != Some(owner) {
                continue;
            }
            let a = st.empires[ei].abilities();
            let bombs: i64 = st.empires[ei].orbit[planet]
                .iter()
                .zip(&st.rules.designs)
                .map(|(&c, d)| c as i64 * d.bombs as i64)
                .sum::<i64>()
                * a.attack_pct as i64
                / 100;
            if bombs == 0 {
                continue;
            }
            let p = &mut st.planets[planet];
            let bombs = (bombs * 100 / (100 + p.shield as i64 * 10)) as i32;
            let pop_killed = (bombs / 10).min(p.pop);
            let fact = (bombs / 4).min(p.factories);
            p.pop -= pop_killed;
            p.factories -= fact;
            st.push_event(
                vec![ei as u8, owner],
                EventKind::Bombed {
                    planet,
                    attacker: ei as u8,
                    pop_killed,
                    factories_destroyed: fact,
                },
            );
            if st.planets[planet].pop <= 0 {
                destroy_colony(st, planet);
            }
        }
    }
}

fn destroy_colony(st: &mut State, planet: PlanetId) {
    let p = &mut st.planets[planet];
    let Some(owner) = p.owner.take() else { return };
    p.pop = 0;
    p.factories = 0;
    p.bases = 0;
    p.shield = 0;
    p.waste = 0;
    st.push_event(vec![], EventKind::ColonyDestroyed { planet, owner });
}

/// Defence fire against incoming transports: bases and armed ships of the owner.
fn transport_losses(st: &mut State, planet: PlanetId, owner: EmpireId, pop: i32) -> i32 {
    let a = st.empires[owner as usize].abilities();
    let p = &st.planets[planet];
    let ships: i64 = st.empires[owner as usize].orbit[planet]
        .iter()
        .zip(&st.rules.designs)
        .map(|(&c, d)| c as i64 * d.attack as i64)
        .sum();
    let fire = (p.bases as i64 * st.rules.base_attack as i64 + ships) * a.attack_pct as i64 / 100;
    if fire == 0 {
        return 0;
    }
    let lost = fire * (50 + st.rng.below(51) as i64) / 1000;
    (lost as i32).clamp(0, pop)
}

/// 1oom `game_ground`: rounds of d100 + bonus, the loser of each round loses one unit.
fn ground_battle(
    rng: &mut Rng,
    mut att: i32,
    abonus: i32,
    mut def: i32,
    dbonus: i32,
) -> (i32, i32) {
    while att > 0 && def > 0 {
        let ra = rng.one_to(100) as i32 + abonus;
        let rd = rng.one_to(100) as i32 + dbonus;
        if ra > rd {
            def -= 1;
        } else {
            att -= 1;
        }
    }
    (att, def)
}

pub fn land_transports(st: &mut State, arrived: Vec<Transport>) {
    for t in arrived {
        let planet = t.dest;
        match st.planets[planet].owner {
            Some(o) if o == t.owner => {
                let p = &mut st.planets[planet];
                p.pop = (p.pop + t.pop).min(p.max_pop);
            }
            None => {
                if can_colonize(st, t.owner, planet) {
                    found_colony(st, planet, t.owner, t.pop);
                } else {
                    st.push_event(
                        vec![t.owner],
                        EventKind::TransportsLost {
                            planet,
                            owner: t.owner,
                            pop: t.pop,
                        },
                    );
                }
            }
            Some(def) => {
                let shot = transport_losses(st, planet, def, t.pop);
                let att = t.pop - shot;
                if shot > 0 {
                    st.push_event(
                        vec![t.owner, def],
                        EventKind::TransportsLost {
                            planet,
                            owner: t.owner,
                            pop: shot,
                        },
                    );
                }
                if att <= 0 {
                    continue;
                }
                let abonus = st.empires[t.owner as usize].abilities().ground_bonus;
                let dbonus = st.empires[def as usize].abilities().ground_bonus + 5;
                let defenders = st.planets[planet].pop;
                let (a, d) = ground_battle(&mut st.rng, att, abonus, defenders, dbonus);
                let won = d == 0;
                st.push_event(
                    vec![t.owner, def],
                    EventKind::GroundBattle {
                        planet,
                        attacker: t.owner,
                        defender: def,
                        won,
                        attackers_left: a,
                        defenders_left: d,
                    },
                );
                if won {
                    let cof = st.empires[t.owner as usize]
                        .abilities()
                        .colonist_oper_factories;
                    let p = &mut st.planets[planet];
                    p.owner = Some(t.owner);
                    p.pop = a.min(p.max_pop);
                    p.bases = 0;
                    p.shield = 0;
                    p.bc_to_shield = 0;
                    p.bc_to_base = 0;
                    p.bc_to_ship = 0;
                    p.bc_to_refit = 0;
                    p.pop_oper_fact = p.pop_oper_fact.min(cof);
                    p.sliders = crate::turn::new_colony_sliders();
                    p.build = 1;
                    st.empires[t.owner as usize].explored[planet] = true;
                    st.push_event(
                        vec![],
                        EventKind::PlanetCaptured {
                            planet,
                            from: def,
                            to: t.owner,
                        },
                    );
                } else {
                    st.planets[planet].pop = d;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ground_battle_bonus_matters() {
        let mut rng = Rng::new(3);
        let mut wins = 0;
        for _ in 0..200 {
            let (_, d) = ground_battle(&mut rng, 20, 30, 20, 0);
            if d == 0 {
                wins += 1;
            }
        }
        assert!(wins > 150, "{wins}");
    }

    #[test]
    fn damage_is_spread_by_hit_points() {
        let mut rng = Rng::new(9);
        let mut s = Side {
            id: 0,
            units: vec![
                Unit {
                    design: Some(0),
                    count: 10,
                    hp: 10,
                    attack: 1,
                    pooled: false,
                },
                Unit {
                    design: Some(1),
                    count: 10,
                    hp: 10,
                    attack: 1,
                    pooled: false,
                },
            ],
            damage_taken_pct: 100,
            start_ships: 20,
        };
        s.take(100, &mut rng);
        assert_eq!(s.ships(), 10);
        assert_eq!(s.units[0].count, 5);
    }
}
