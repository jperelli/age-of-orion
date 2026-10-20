//! Research (1oom `game_tech.c`, research lists from `game_new.c`).
//!
//! Each empire has, per field, a list of 10 tiers of up to 3 techs it can ever research
//! (generated at the start, Psilons get more). It researches one project per field;
//! investment earns 15% interest, and once it exceeds the cost each turn has a growing
//! chance to complete the tech. Completing techs raises the field level, which makes
//! ship parts cheaper and smaller.

use serde::Serialize;

use crate::data::{RESEARCH_FLAGS, TECHS};
use crate::rng::Rng;
use crate::state::*;
use crate::types::*;

pub const TIERS: usize = 10;
pub const NEXT_MAX: usize = 12;
const NEVER: u8 = 1;
const ALWAYS: u8 = 2;

/// Whether tech `n` exists in `field`. Numbers 55, 60 .. 100 are the repeatable
/// advanced techs available once a field's list is exhausted.
pub fn exists(field: usize, n: u8) -> bool {
    match n {
        1..=50 => !TECHS[field][n as usize].is_empty(),
        55..=100 => n % 5 == 0,
        _ => false,
    }
}

/// Text key of a tech (`tech.<field>.<key>`).
pub fn key(field: usize, n: u8) -> String {
    if n <= 50 {
        TECHS[field][n as usize].to_string()
    } else {
        "advanced".to_string()
    }
}

/// Port of 1oom `game_generate_research` for one empire.
pub fn generate_list(rng: &mut Rng, race: Race) -> Vec<[[u8; 3]; TIERS]> {
    let rmax = if race == Race::Psilon { 3 } else { 2 };
    loop {
        let mut all = Vec::with_capacity(NUM_FIELDS);
        let mut ok = true;
        for field in 0..NUM_FIELDS {
            let rr = &RESEARCH_FLAGS[race.index()][field];
            let mut rl = [[0u8; 3]; TIERS];
            for (tier, row) in rl.iter_mut().enumerate() {
                let mut taken = 0;
                while taken == 0 {
                    for ti in (tier * 5..=tier * 5 + 4).rev() {
                        if taken < 3 && rr[ti + 1] & ALWAYS != 0 {
                            row[taken] = ti as u8 + 1;
                            taken += 1;
                        }
                    }
                    for ti in (tier * 5..=tier * 5 + 4).rev() {
                        // Tech 1 of every field is known from the start.
                        let skip = rr[ti + 1] & (NEVER | ALWAYS) != 0
                            || ti == 0
                            || !exists(field, ti as u8 + 1);
                        if !skip && rng.one_to(4) <= rmax && taken < 3 {
                            row[taken] = ti as u8 + 1;
                            taken += 1;
                        }
                    }
                    for _ in 0..3 {
                        for l in 0..2 {
                            if row[l + 1] != 0 && row[l] > row[l + 1] {
                                row.swap(l, l + 1);
                            }
                        }
                    }
                }
            }
            // Every essential tech group must be represented (1oom `..._check_field`).
            let rmask = rr[1..].iter().fold(0, |m, &f| m | f) & !(NEVER | ALWAYS);
            let got = rl.iter().flatten().fold(0, |m, &t| m | rr[t as usize]);
            if rmask != 0 && (rmask & got) != rmask {
                ok = false;
            }
            all.push(rl);
        }
        if ok {
            return all;
        }
    }
}

/// 1oom `game_tech_get_field_percent`: the field's tech level.
pub fn field_level(t: &Tech, field: usize) -> u32 {
    let c = &t.completed[field];
    let tmax = c.iter().copied().max().unwrap_or(0) as i32;
    let len = c.len() as i32;
    let v = if len == 1 { 1 } else { (tmax - 1) / 5 + 2 };
    (tmax + len - v).clamp(1, 99) as u32
}

/// 1oom `game_tech_get_next_techs`: what can be researched next in a field.
pub fn next_techs(t: &Tech, field: usize) -> Vec<u8> {
    let c = &t.completed[field];
    let tmax = c.iter().copied().max().unwrap_or(0);
    let maxtier = if c.len() == 1 {
        1
    } else {
        ((tmax as usize).saturating_sub(1) / 5 + 2).min(TIERS)
    };
    let mut out: Vec<u8> = t.list[field][..maxtier]
        .iter()
        .flatten()
        .copied()
        .filter(|&n| n != 0 && !c.contains(&n))
        .take(NEXT_MAX)
        .collect();
    if out.is_empty() {
        let top = if tmax <= 50 { 55 } else { (tmax + 5).min(100) };
        out = (55..=top)
            .step_by(5)
            .filter(|n| !c.contains(n))
            .take(NEXT_MAX)
            .collect();
    }
    out
}

pub fn cost_of(st: &State, empire: EmpireId, field: usize, tech: u8) -> u32 {
    let e = &st.empires[empire as usize];
    st.rules
        .research_cost(e.race, field, tech, e.ai, st.difficulty)
}

/// 1oom `game_tech_start_next`: changing an ongoing project loses its investment.
pub fn start_project(st: &mut State, empire: EmpireId, field: usize, tech: u8) {
    let cost = cost_of(st, empire, field, tech);
    let t = &mut st.empires[empire as usize].tech;
    if t.project[field] != 0 {
        t.invested[field] = 0;
    }
    t.project[field] = tech;
    t.cost[field] = cost;
}

/// 1oom `game_ai_classic_tech_next`.
pub fn ai_choose(st: &mut State, empire: EmpireId, field: usize) {
    let next = next_techs(&st.empires[empire as usize].tech, field);
    if next.is_empty() {
        return;
    }
    let tech = if st.rng.one_to(6) < st.difficulty as u16 {
        next[next.len() - 1]
    } else {
        next[st.rng.below(next.len() as u16) as usize]
    };
    start_project(st, empire, field, tech);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Research,
    Orion,
    Artifacts,
    Trade,
    Espionage,
}

/// 1oom `game_tech_get_new`.
pub fn grant(st: &mut State, empire: EmpireId, field: usize, tech: u8, source: Source) {
    grant_from(st, empire, field, tech, source, None);
}

/// `grant` with the empire the tech came from (trade, espionage).
pub fn grant_from(
    st: &mut State,
    empire: EmpireId,
    field: usize,
    tech: u8,
    source: Source,
    from: Option<EmpireId>,
) {
    let ai = st.empires[empire as usize].ai;
    let t = &mut st.empires[empire as usize].tech;
    if tech == 0 || t.has(field, tech) {
        if t.project[field] == tech {
            if ai {
                ai_choose(st, empire, field);
            } else {
                t.project[field] = 0;
            }
        }
        return;
    }
    t.add(field, tech);
    t.level[field] = field_level(t, field);
    if t.project[field] == tech {
        t.project[field] = 0;
        t.invested[field] = 1;
    }
    let idle = t.project[field] == 0;
    st.push_event(
        vec![empire],
        EventKind::TechDiscovered {
            field: TechField::ALL[field],
            tech,
            source,
            from,
        },
    );
    if idle && ai {
        ai_choose(st, empire, field);
    }
}

/// 1oom `game_tech_research`: interest, investment and the chance of a breakthrough.
pub fn research(st: &mut State) {
    for ei in 0..st.empires.len() {
        let total = st.empires[ei].research.max(0) as u32;
        for f in 0..NUM_FIELDS {
            let t = &mut st.empires[ei].tech;
            let slider = t.alloc[f];
            let mut invest = t.invested[f];
            let t3 = slider * total / 100;
            let t1 = (invest * 3 / 20).min(t3 * 2);
            invest += t3 + t1;
            t.invested[f] = invest;
            t.level[f] = field_level(t, f);
            let cost = t.cost[f];
            if cost != 0 && slider != 0 && total != 0 {
                if cost < invest {
                    let v = ((invest - cost) as u64 * 250 / cost as u64).min(500) as u16;
                    if st.rng.one_to(500) <= v {
                        let project = st.empires[ei].tech.project[f];
                        grant(st, ei as u8, f, project, Source::Research);
                    }
                }
            } else {
                t.invested[f] = invest * 9 / 10;
            }
        }
    }
}

/// 1oom `game_tech_final_war_share`: the sides of the final war pool their techs.
pub fn final_war_share(st: &mut State) {
    let n = st.empires.len();
    let refusers = st.council.refuse.iter().filter(|&&r| r).count();
    for f in 0..NUM_FIELDS {
        for side in [false, true] {
            if side && refusers < 2 {
                continue;
            }
            let members: Vec<usize> = (0..n)
                .filter(|&i| st.council.refuse.get(i).copied().unwrap_or(false) == side)
                .collect();
            let mut pool: Vec<(u8, usize)> = Vec::new();
            for &i in &members {
                for &t in &st.empires[i].tech.completed[f] {
                    pool.push((t, i));
                }
            }
            let alive: Vec<usize> = members
                .iter()
                .copied()
                .filter(|&i| st.empires[i].alive)
                .collect();
            for i in alive {
                for &(t, src) in &pool {
                    if !st.empires[i].tech.has(f, t) {
                        grant_from(st, i as u8, f, t, Source::Trade, Some(src as u8));
                    }
                }
            }
        }
    }
}

/// Picks a random tech the empire lacks among `range`, up to `levels` above its level.
fn random_missing(
    st: &mut State,
    empire: EmpireId,
    range: (u8, u8),
    levels: u32,
) -> Option<(usize, u8)> {
    for _ in 0..200 {
        let field = st.rng.below(NUM_FIELDS as u16) as usize;
        let tech = range.0 + st.rng.below((range.1 - range.0 + 1) as u16) as u8;
        let t = &st.empires[empire as usize].tech;
        let limit = (t.level[field] + levels).min(50);
        if limit >= tech as u32 && exists(field, tech) && !t.has(field, tech) {
            return Some((field, tech));
        }
    }
    None
}

/// 1oom `game_tech_get_orion_loot`: the Death Ray and three advanced techs.
pub fn orion_loot(st: &mut State, empire: EmpireId) {
    grant(st, empire, FIELD_WEAPON, 36, Source::Orion);
    for _ in 0..3 {
        if let Some((f, n)) = random_missing(st, empire, (20, 50), 25) {
            grant(st, empire, f, n, Source::Orion);
        }
    }
}

/// 1oom `game_tech_get_artifact_loot`: one tech for the first explorer of ancient ruins.
pub fn artifact_loot(st: &mut State, empire: EmpireId) {
    if let Some((f, n)) = random_missing(st, empire, (1, 30), 10) {
        grant(st, empire, f, n, Source::Artifacts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_contain_only_real_techs_and_essentials() {
        let mut rng = Rng::new(7);
        for race in Race::ALL {
            let list = generate_list(&mut rng, race);
            for (f, field) in list.iter().enumerate() {
                for (tier, row) in field.iter().enumerate() {
                    assert!(
                        row.iter().any(|&t| t != 0),
                        "{race:?} field {f} tier {tier}"
                    );
                    for &t in row.iter().filter(|&&t| t != 0) {
                        assert!(exists(f, t) && t > 1);
                        assert!((tier * 5 + 1..=tier * 5 + 5).contains(&(t as usize)));
                    }
                }
            }
            // Robotic controls (computers 8, 18, ...) are essential for everyone.
            assert!(list[FIELD_COMPUTER].iter().flatten().any(|&t| t % 10 == 8));
        }
    }

    #[test]
    fn next_techs_open_tiers_as_you_advance() {
        let mut rng = Rng::new(3);
        let mut t = Tech::new();
        t.list = generate_list(&mut rng, Race::Human);
        let first = next_techs(&t, FIELD_WEAPON);
        assert!(first.iter().all(|&n| n <= 5));
        t.add(FIELD_WEAPON, first[0]);
        let more = next_techs(&t, FIELD_WEAPON);
        assert!(more.iter().any(|&n| n > 5));
        assert_eq!(field_level(&Tech::new(), 0), 1);
    }
}
