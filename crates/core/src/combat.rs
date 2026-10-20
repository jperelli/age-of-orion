//! Space battles, orbital bombing, transports and ground combat.
//!
//! MOO1 fights battles tactically on a grid (1oom `game_battle_human.c`, milestone 0.4).
//! Until then battles are resolved in rounds where every stack fires its weapons with the
//! 1oom to-hit and damage formula (`game_battle_attack`): computer level against defense,
//! shields absorbing damage, missiles against jammers and anti-missile specials.
//! Bombing (`game_bomb.c`), shooting at transports (`game_turn_transport_shoot`) and
//! ground combat (`game_ground.c`) are direct ports.

use crate::data::*;
use crate::data_types::Weapon;
use crate::design::*;
use crate::rng::Rng;
use crate::state::*;
use crate::types::*;

const GUARDIAN: i32 = -1;

/// 1oom `tbl_monster[MONSTER_GUARDIAN]`, per difficulty:
/// hit points, defense (= missile defense), shield absorb, weapon counts.
const GUARDIAN_STATS: [(u32, i32, i32, [u32; 4]); 5] = [
    (2000, 1, 5, [5, 1, 5, 6]),
    (4000, 3, 6, [25, 1, 15, 9]),
    (6000, 5, 7, [45, 1, 25, 12]),
    (8000, 7, 8, [65, 1, 35, 15]),
    (10000, 9, 9, [85, 1, 45, 18]),
];
/// Scatter Pack X (5), Death Ray, Stellar Converter, Plasma Torpedo.
const GUARDIAN_WEAPONS: [usize; 4] = [56, 62, 58, 60];

#[derive(Debug, Clone)]
struct Arm {
    kind: usize,
    count: u32,
    /// Shots left in the battle, -1 = unlimited.
    shots: i32,
}

#[derive(Debug, Clone)]
struct Stack {
    /// Design index; `None` for missile bases and the guardian.
    design: Option<usize>,
    count: u32,
    hp: u32,
    hp_loss: u32,
    comp_level: i32,
    defense: i32,
    missile_defense: i32,
    absorb: i32,
    missile_shield: i32,
    flags: u16,
    arms: Vec<Arm>,
    /// Missile bases: beams and torpedoes do half damage (the planet absorbs it).
    planet: bool,
}

impl Stack {
    fn armed(&self) -> bool {
        self.count > 0
            && self
                .arms
                .iter()
                .any(|a| a.count > 0 && !WEAPONS[a.kind].bomb)
    }
}

#[derive(Debug, Clone)]
struct Side {
    id: i32,
    stacks: Vec<Stack>,
    start_ships: u32,
}

impl Side {
    fn alive(&self) -> bool {
        self.stacks.iter().any(|s| s.count > 0)
    }

    fn armed(&self) -> bool {
        self.stacks.iter().any(Stack::armed)
    }

    fn ships(&self) -> u32 {
        self.stacks
            .iter()
            .filter(|s| s.design.is_some())
            .map(|s| s.count)
            .sum()
    }
}

/// Whether another empire (or the guardian) has armed forces at `planet`.
pub fn hostile_armed_at(st: &State, planet: PlanetId, empire: EmpireId) -> bool {
    if planet == st.orion && st.guardian {
        return true;
    }
    st.empires
        .iter()
        .enumerate()
        .any(|(i, e)| i as u8 != empire && armed_ships(e, planet))
        || st.planets[planet]
            .owner
            .is_some_and(|o| o != empire && st.planets[planet].bases > 0)
}

fn armed_ships(e: &Empire, planet: PlanetId) -> bool {
    e.orbit[planet]
        .iter()
        .zip(&e.designs)
        .any(|(&c, d)| c > 0 && d.armed())
}

fn ship_stack(e: &Empire, d: usize, count: u32, nebula: bool) -> Stack {
    let sd = &e.designs[d];
    let extra_man = sd
        .specials
        .iter()
        .map(|&s| SPECIALS[s as usize].extra_man as i32)
        .max()
        .unwrap_or(0);
    let mut comp_level =
        COMPS[sd.comp as usize].level as i32 + i32::from(sd.has_flag(FLAG_SCANNER));
    let mut defense = sd.man as i32 + HULLS[sd.hull as usize].defense as i32 + extra_man + 1;
    let mut missile_defense = defense + JAMMERS[sd.jammer as usize].level as i32;
    if e.race == Race::Mrrshan {
        comp_level += 4;
    }
    if e.race == Race::Alkari {
        defense += 3;
        missile_defense += 3;
    }
    Stack {
        design: Some(d),
        count,
        hp: sd.hp.max(1),
        hp_loss: 0,
        comp_level,
        defense,
        missile_defense,
        absorb: if nebula {
            0
        } else {
            SHIELDS[sd.shield as usize].absorb as i32
        },
        missile_shield: sd
            .specials
            .iter()
            .map(|&s| SPECIALS[s as usize].missile_shield as i32)
            .max()
            .unwrap_or(0),
        flags: sd
            .specials
            .iter()
            .fold(0, |m, &s| m | SPECIALS[s as usize].flags),
        arms: (0..WEAPON_SLOTS)
            .filter(|&i| sd.weapons[i] != 0 && sd.weapon_counts[i] > 0)
            .map(|i| Arm {
                kind: sd.weapons[i] as usize,
                count: sd.weapon_counts[i] as u32,
                shots: WEAPONS[sd.weapons[i] as usize].shots as i32,
            })
            .collect(),
        planet: false,
    }
}

/// 1oom `game_parsed_from_planet`: the missile bases of a colony.
fn base_stack(st: &State, owner: EmpireId, planet: PlanetId) -> Stack {
    let e = &st.empires[owner as usize];
    let p = &st.planets[planet];
    let (shield, comp, jammer) = st.base_parts(owner);
    let mut comp_level = comp as i32 + 1;
    if e.race == Race::Mrrshan {
        comp_level += 4;
    }
    let a = e.abilities();
    let absorb = if p.nebula {
        0
    } else {
        SHIELDS[shield].absorb as i32 + p.shield
    };
    Stack {
        design: None,
        count: p.bases as u32,
        hp: st.rules.base_hp[a.armor_tier.min(6)],
        hp_loss: 0,
        comp_level,
        defense: 1,
        missile_defense: jammer as i32 + 1,
        absorb,
        missile_shield: 0,
        flags: FLAG_SCANNER,
        arms: vec![
            Arm {
                kind: st.base_weapon(owner),
                count: 3,
                shots: -1,
            },
            Arm {
                kind: st.base_weapon_2(owner),
                count: 3,
                shots: -1,
            },
        ],
        planet: true,
    }
}

fn guardian_stack(st: &State) -> Stack {
    let (hp, def, absorb, counts) = GUARDIAN_STATS[st.difficulty as usize];
    Stack {
        design: None,
        count: 1,
        hp,
        hp_loss: 0,
        comp_level: COMPS[10].level as i32,
        defense: def,
        missile_defense: def,
        absorb,
        missile_shield: 100,
        flags: 0,
        arms: GUARDIAN_WEAPONS
            .iter()
            .zip(counts)
            .map(|(&kind, count)| Arm {
                kind,
                count,
                shots: WEAPONS[kind].shots as i32,
            })
            .collect(),
        planet: false,
    }
}

fn build_sides(st: &State, planet: PlanetId) -> Vec<Side> {
    let p = &st.planets[planet];
    let mut sides = Vec::new();
    for (ei, e) in st.empires.iter().enumerate() {
        let bases = p.owner == Some(ei as u8) && p.bases > 0;
        let ships = e.orbit[planet].iter().any(|&c| c > 0);
        if !bases && !ships {
            continue;
        }
        let mut stacks: Vec<Stack> = e.orbit[planet]
            .iter()
            .enumerate()
            .filter(|(d, &c)| c > 0 && *d < e.designs.len())
            .map(|(d, &c)| ship_stack(e, d, c, p.nebula))
            .collect();
        if bases {
            stacks.push(base_stack(st, ei as u8, planet));
        }
        let start_ships = stacks
            .iter()
            .filter(|s| s.design.is_some())
            .map(|s| s.count)
            .sum();
        sides.push(Side {
            id: ei as i32,
            stacks,
            start_ships,
        });
    }
    if planet == st.orion && st.guardian {
        sides.push(Side {
            id: GUARDIAN,
            stacks: vec![guardian_stack(st)],
            start_ships: 0,
        });
    }
    sides
}

/// One weapon of a stack firing at a target stack (1oom `game_battle_attack`).
fn fire(rng: &mut Rng, w: &Weapon, volleys: u32, attacker: &Stack, target: &mut Stack) {
    let missile = w.is_missile();
    let defense = if missile {
        target.missile_defense
    } else {
        target.defense
    };
    let mut miss = 50 - (attacker.comp_level - defense) * 10 - w.extra_acc as i32 * 10;
    if target.flags & FLAG_CLOAK != 0 {
        miss += 50;
    }
    let miss = miss.min(95);
    let range = (w.damage_max - w.damage_min) as i32;
    let damage_div = if target.planet && (w.miss_type > 0 || range != 0) {
        2
    } else {
        1
    };
    let mut absorb_div =
        1 + i32::from(w.halve_shield) + i32::from(attacker.flags & FLAG_ORACLE != 0);
    if absorb_div == 3 {
        absorb_div = 4;
    }
    let streaming = !missile && w.streams > 0;
    let warheads = if missile { w.streams.max(1) as u32 } else { 1 };
    let mut mul1 = volleys * w.fire.max(1) as u32 * warheads;
    let mut mul2: u32 = 1;
    while mul1 > 1000 {
        mul1 /= 2;
        mul2 *= 2;
    }
    let hp = target.hp.max(1);
    for _ in 0..mul1 {
        if target.count == 0 {
            break;
        }
        let r = rng.one_to(100) as i32;
        if r < miss {
            continue;
        }
        if missile && target.missile_shield > 0 && rng.one_to(100) as i32 <= target.missile_shield {
            continue;
        }
        let mut dmg = ((r - miss) * 2 * range) / (100 - miss).max(1);
        dmg = (dmg + 1) / 2 + w.damage_min as i32;
        dmg /= damage_div;
        dmg -= target.absorb / absorb_div;
        dmg *= w.damage_mul as i32 * mul2 as i32;
        if target.flags & FLAG_DISPLACEMENT != 0 && rng.one_to(100) < 35 {
            dmg = 0;
        }
        if dmg <= 0 {
            continue;
        }
        target.hp_loss += dmg as u32;
        let mut k = 0;
        while (streaming || k < mul2) && target.hp_loss >= hp && target.count > 0 {
            target.count -= 1;
            target.hp_loss -= hp;
            k += 1;
        }
        target.hp_loss %= hp;
    }
}

fn round(rng: &mut Rng, sides: &mut [Side]) {
    let mut shots: Vec<(usize, usize, usize, u32)> = Vec::new();
    for (si, side) in sides.iter_mut().enumerate() {
        for (ki, stack) in side.stacks.iter_mut().enumerate() {
            if stack.count == 0 {
                continue;
            }
            for (ai, arm) in stack.arms.iter_mut().enumerate() {
                let w = &WEAPONS[arm.kind];
                if arm.count == 0 || w.bomb || arm.shots == 0 {
                    continue;
                }
                if arm.shots > 0 {
                    arm.shots -= 1;
                }
                shots.push((si, ki, ai, stack.count * arm.count));
            }
        }
    }
    for (si, ki, ai, volleys) in shots {
        let targets: Vec<(usize, usize)> = (0..sides.len())
            .filter(|&t| t != si)
            .flat_map(|t| {
                sides[t]
                    .stacks
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.count > 0)
                    .map(move |(k, _)| (t, k))
            })
            .collect();
        if targets.is_empty() || sides[si].stacks[ki].count == 0 {
            continue;
        }
        let (ts, tk) = targets[rng.below(targets.len() as u16) as usize];
        let attacker = sides[si].stacks[ki].clone();
        let w = WEAPONS[attacker.arms[ai].kind];
        // Ships destroyed earlier this round still fire: rounds are simultaneous.
        fire(rng, &w, volleys, &attacker, &mut sides[ts].stacks[tk]);
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
        if sides.iter().filter(|s| s.armed()).count() == 0
            || sides.iter().filter(|s| s.alive()).count() < 2
            || !sides
                .iter()
                .any(|s| s.armed() && sides.iter().any(|o| o.id != s.id && o.alive()))
        {
            break;
        }
        round(&mut st.rng, sides);
    }
    let armed_left: Vec<i32> = sides.iter().filter(|s| s.armed()).map(|s| s.id).collect();
    let winner = (armed_left.len() == 1).then(|| armed_left[0]);
    let mut losses = Vec::new();
    for s in sides.iter() {
        losses.push(s.start_ships - s.ships());
        if s.id == GUARDIAN {
            if s.stacks[0].count == 0 {
                st.guardian = false;
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
        for stack in &s.stacks {
            match stack.design {
                Some(d) => st.empires[e].orbit[planet][d] = stack.count,
                None => st.planets[planet].bases = stack.count as i32,
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

/// Resolves every planet where two or more sides meet and one of them is armed.
pub fn battles(st: &mut State) {
    for planet in 0..st.planets.len() {
        let mut sides = build_sides(st, planet);
        if sides.len() >= 2 && sides.iter().any(Side::armed) {
            fight(st, planet, &mut sides);
        }
        // Unarmed ships cannot stay where a single hostile armed side holds the orbit.
        let armed: Vec<i32> = build_sides(st, planet)
            .iter()
            .filter(|s| s.armed())
            .map(|s| s.id)
            .collect();
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

/// 1oom `game_turn_bomb_damage`: (population killed, factories destroyed, bio damage).
fn bomb_damage(st: &mut State, planet: PlanetId, attacker: EmpireId) -> (i32, i32, i32) {
    let p = &st.planets[planet];
    let owner = p.owner.expect("owned planet");
    let pshield = p.shield;
    let antidote = st.empires[owner as usize].abilities().antidote;
    let e = &st.empires[attacker as usize];
    let mut tbl = vec![0u32; WEAPONS.len()];
    let mut maxcomp = 0;
    for (d, sd) in e.designs.iter().enumerate() {
        let n = e.orbit[planet][d];
        if n == 0 {
            continue;
        }
        maxcomp = maxcomp.max(sd.comp as i32);
        for s in 0..WEAPON_SLOTS {
            let v = n * sd.weapon_counts[s] as u32;
            if v != 0 {
                let ns = WEAPONS[sd.weapons[s] as usize].shots;
                tbl[sd.weapons[s] as usize] += v * if ns == -1 { 30 } else { ns as u32 };
            }
        }
    }
    let complevel = ((maxcomp - 1) * 2 + 6).clamp(1, 20) as u16;
    let (mut total, mut bio) = (0i64, 0i32);
    let rng = &mut st.rng;
    for (i, &count) in tbl.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let w = &WEAPONS[i];
        let mut vcur = count.min(st.rules.max_bomb_dmg);
        let v = if vcur < 10 {
            1
        } else {
            let v = vcur / 10;
            vcur = 10;
            v
        };
        let (dmin, mut dmax) = (w.damage_min as i32, w.damage_max as i32);
        if dmin == dmax {
            if w.bio {
                for _ in 0..vcur {
                    if rng.one_to(20) <= complevel {
                        let dmg = (rng.below(dmin as u16 + 1) as i32 - antidote).max(0);
                        bio = (bio + v as i32 * dmg).min(st.rules.max_bio_dmg);
                    }
                }
            } else {
                if w.miss_type != 0 {
                    dmax /= 2;
                }
                dmax *= w.streams as i32;
                if dmax > pshield {
                    for _ in 0..vcur {
                        if rng.one_to(20) <= complevel {
                            total += v as i64 * (dmax - pshield) as i64;
                        }
                    }
                }
            }
        } else {
            let (mut dmin, mut dmax) = (dmin, dmax);
            if !w.bomb {
                dmin /= 2;
                dmax /= 2;
            }
            if dmax > pshield {
                let range = (dmax - dmin + 1) as u16;
                let base = dmin - 1 - pshield;
                let shots = complevel as u32 * vcur * v / 20;
                for _ in 0..shots {
                    let dmg = rng.one_to(range) as i32 + base;
                    if dmg > 0 {
                        total += dmg as i64;
                    }
                }
            }
        }
    }
    let p = &st.planets[planet];
    let roll = |rng: &mut Rng, hp: i32| {
        let h = hp / 5;
        (rng.one_to((h * 2).max(1) as u16) as i32 + hp - h).max(1)
    };
    let fact_hp = st.rules.fact_hp;
    let pop_hp = st.rules.pop_hp;
    let fact = ((total / roll(&mut st.rng, fact_hp) as i64) as i32).min(p.factories);
    let pop = ((total / roll(&mut st.rng, pop_hp) as i64) as i32 + bio).min(p.pop);
    (pop, fact, bio)
}

/// 1oom `game_turn_bomb`: ships in orbit of an enemy colony may bomb it.
pub fn bomb(st: &mut State) {
    for planet in 0..st.planets.len() {
        for ei in 0..st.empires.len() {
            let Some(owner) = st.planets[planet].owner else {
                break;
            };
            if ei == owner as usize || st.empires[ei].orbit[planet].iter().all(|&c| c == 0) {
                continue;
            }
            let (pop_dmg, fact_dmg, bio) = bomb_damage(st, planet, ei as u8);
            if pop_dmg == 0 && fact_dmg == 0 {
                continue;
            }
            let e = &st.empires[ei];
            let inbound: i32 = st
                .transports
                .iter()
                .filter(|t| t.owner as usize == ei && t.dest == planet)
                .map(|t| t.pop)
                .sum();
            let go = if e.ai {
                let inbound = if e.race == Race::Bulrathi {
                    inbound + inbound / 5
                } else {
                    inbound
                };
                st.planets[planet].pop > inbound
            } else {
                e.bombard
            };
            if !go {
                continue;
            }
            let p = &mut st.planets[planet];
            if bio > 0 {
                p.max_pop = (p.max_pop - bio).max(10);
            }
            p.pop -= pop_dmg;
            p.factories -= fact_dmg;
            st.push_event(
                vec![ei as u8, owner],
                EventKind::Bombed {
                    planet,
                    attacker: ei as u8,
                    pop_killed: pop_dmg,
                    factories_destroyed: fact_dmg,
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
    p.stargate = false;
    st.push_event(vec![], EventKind::ColonyDestroyed { planet, owner });
}

/// 1oom `game_turn_transport_shoot`: transports killed by `shooter` at `planet`.
fn transport_shoot(
    st: &mut State,
    planet: PlanetId,
    owner: EmpireId,
    num: i32,
    speed: i32,
    shooter: EmpireId,
    bases: i32,
) -> i32 {
    let transport_hp = (st.empires[owner as usize].abilities().armor_tier as i32 + 1) * 15;
    let total_hp = transport_hp as i64 * num as i64;
    let mut tbl = vec![0u32; WEAPONS.len()];
    let speed = speed.max(1);
    if bases > 0 {
        tbl[st.base_weapon(shooter)] += bases as u32 * 24;
    }
    let mut bestcomp = 0;
    let e = &st.empires[shooter as usize];
    for (d, sd) in e.designs.iter().enumerate() {
        if sd.comp as i32 > speed {
            bestcomp = sd.comp as i32;
        }
        let n = e.orbit[planet][d];
        for s in 0..WEAPON_SLOTS {
            let w = &WEAPONS[sd.weapons[s] as usize];
            let mut v = n * sd.weapon_counts[s] as u32;
            if v == 0 {
                continue;
            }
            v *= if w.shots == -1 { 4 } else { w.shots as u32 };
            if w.streams != 0 {
                v *= w.streams as u32;
            }
            v = v.min(32_000);
            if w.bomb || w.bio {
                v = 0;
            }
            if w.miss_type != 0 {
                v /= 2;
            }
            tbl[sd.weapons[s] as usize] += v * 2 / speed as u32;
        }
    }
    if bases > 0 {
        bestcomp = st.base_parts(owner).1 as i32;
    }
    let complevel = ((bestcomp - speed) * 2 + 12).clamp(1, 20) as u16;
    let mut total: i64 = 0;
    for (i, &vcur) in tbl.iter().enumerate() {
        if vcur == 0 {
            continue;
        }
        let w = &WEAPONS[i];
        let (dmin, dmax) = (w.damage_min as i64, w.damage_max as i64);
        if dmin == dmax {
            for _ in 0..vcur {
                if st.rng.one_to(20) <= complevel {
                    total += dmax;
                }
            }
        } else {
            let range = (dmax - dmin + 1) as u16;
            for _ in 0..(complevel as u32 * vcur / 20) {
                let dmg = st.rng.one_to(range) as i64 + dmin - 1;
                total += dmg.max(0);
            }
        }
        if total >= total_hp {
            break;
        }
    }
    ((total / transport_hp as i64).min(1000)) as i32
}

/// 1oom `game_ground_kill`: rounds of d100 + bonus, the loser of each round loses one unit.
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

/// 1oom `game_turn_transport` and ground combat.
pub fn land_transports(st: &mut State, arrived: Vec<Transport>) {
    for t in arrived {
        let planet = t.dest;
        let mut pop = t.pop;
        for j in 0..st.empires.len() as u8 {
            if j == t.owner {
                continue;
            }
            if st.planets[planet].owner == Some(j) {
                let bases = st.planets[planet].bases;
                pop -= transport_shoot(st, planet, t.owner, pop, t.speed, j, bases);
            } else if st.empires[j as usize].orbit[planet].iter().any(|&c| c > 0) {
                pop -= transport_shoot(st, planet, t.owner, pop, t.speed, j, 0);
            }
        }
        if planet == st.orion && st.guardian {
            pop = 0;
        }
        pop = pop.max(0);
        let a = st.empires[t.owner as usize].abilities();
        if a.combat_transporters && pop < t.pop {
            let chance = match st.planets[planet].owner {
                Some(o) if st.empires[o as usize].abilities().sub_space_interdictor => 4,
                _ => 2,
            };
            for _ in 0..(t.pop - pop) {
                if st.rng.below(chance) == 0 {
                    pop += 1;
                }
            }
        }
        if pop < t.pop {
            let mut to = vec![t.owner];
            to.extend(st.planets[planet].owner);
            st.push_event(
                to,
                EventKind::TransportsLost {
                    planet,
                    owner: t.owner,
                    pop: t.pop - pop,
                },
            );
        }
        if pop <= 0 {
            continue;
        }
        match st.planets[planet].owner {
            Some(o) if o == t.owner => {
                let p = &mut st.planets[planet];
                p.pop = (p.pop + pop).min(p.max_pop);
            }
            None => {
                // MOO1: transports cannot found colonies; they are lost.
                st.push_event(
                    vec![t.owner],
                    EventKind::TransportsLost {
                        planet,
                        owner: t.owner,
                        pop,
                    },
                );
            }
            Some(def) => invade(st, planet, t.owner, def, pop),
        }
    }
}

fn invade(st: &mut State, planet: PlanetId, att_e: EmpireId, def: EmpireId, att: i32) {
    let abonus = st.empires[att_e as usize].abilities().ground_bonus;
    let dbonus = st.empires[def as usize].abilities().ground_bonus + 5;
    let defenders = st.planets[planet].pop;
    let (a, d) = ground_battle(&mut st.rng, att, abonus, defenders, dbonus);
    let won = d == 0;
    st.push_event(
        vec![att_e, def],
        EventKind::GroundBattle {
            planet,
            attacker: att_e,
            defender: def,
            won,
            attackers_left: a,
            defenders_left: d,
        },
    );
    if !won {
        st.planets[planet].pop = d;
        return;
    }
    let p = &mut st.planets[planet];
    p.owner = Some(att_e);
    p.pop = a.min(p.max_pop);
    p.bases = 0;
    p.shield = 0;
    p.bc_to_shield = 0;
    p.bc_to_base = 0;
    p.bc_to_ship = 0;
    p.bc_to_refit = 0;
    p.pop_oper_fact = 1;
    p.sliders = crate::turn::new_colony_sliders();
    p.build = 0;
    p.rally = None;
    st.empires[att_e as usize].explored[planet] = true;
    st.push_event(
        vec![],
        EventKind::PlanetCaptured {
            planet,
            from: def,
            to: att_e,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(count: u32, hp: u32, comp: i32, defense: i32, absorb: i32) -> Stack {
        Stack {
            design: Some(0),
            count,
            hp,
            hp_loss: 0,
            comp_level: comp,
            defense,
            missile_defense: defense,
            absorb,
            missile_shield: 0,
            flags: 0,
            arms: vec![],
            planet: false,
        }
    }

    #[test]
    fn ground_battle_bonus_matters() {
        let mut rng = Rng::new(3);
        let wins = (0..200)
            .filter(|_| ground_battle(&mut rng, 20, 30, 20, 0).1 == 0)
            .count();
        assert!(wins > 150, "{wins}");
    }

    #[test]
    fn shields_absorb_lasers() {
        let laser = WEAPONS[2];
        let mut rng = Rng::new(5);
        let attacker = stack(100, 3, 1, 1, 0);
        let mut unshielded = stack(1000, 3, 1, 1, 0);
        fire(&mut rng, &laser, 100, &attacker, &mut unshielded);
        assert!(unshielded.count < 1000);
        let mut shielded = stack(1000, 3, 1, 1, 5);
        fire(&mut rng, &laser, 100, &attacker, &mut shielded);
        assert_eq!(shielded.count, 1000, "laser 1-4 cannot pierce absorb 5");
    }

    #[test]
    fn better_computers_hit_more() {
        let laser = WEAPONS[2];
        let (mut a, mut b) = (stack(200, 100, 1, 1, 0), stack(200, 100, 1, 1, 0));
        let mut rng = Rng::new(9);
        fire(&mut rng, &laser, 500, &stack(1, 1, 1, 1, 0), &mut a);
        fire(&mut rng, &laser, 500, &stack(1, 1, 6, 1, 0), &mut b);
        assert!(b.hp_loss + (200 - b.count) * 100 > a.hp_loss + (200 - a.count) * 100);
    }
}
