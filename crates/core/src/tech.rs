//! Research (1oom `game_tech.c`, research lists from `game_new.c`).
//!
//! Each empire has, per field, a list of 10 tiers of up to 3 techs it can ever research
//! (generated at the start, Psilons get more). It researches one project per field;
//! investment earns 15% interest, and once it exceeds the cost each turn has a growing
//! chance to complete the tech. Completing techs raises the field level, which makes
//! ship parts cheaper and smaller.

use serde::Serialize;

use crate::data::{RESEARCH_FLAGS, TECHS, TECH_GROUPS};
use crate::rng::Rng;
use crate::rules::Abilities;
use crate::state::*;
use crate::types::*;

pub const TIERS: usize = 10;
/// 1oom `TECH_NEXT_MAX`.
pub const NEXT_MAX: usize = 12;
const NEVER: u8 = 1;
const ALWAYS: u8 = 2;

/// Whether tech `n` exists in `field` (1oom: its RESEARCH.LBX group is not `0xff`).
/// Numbers 55, 60 .. 100 are the repeatable advanced techs available once a field's list
/// is exhausted.
pub fn exists(field: usize, n: u8) -> bool {
    match n {
        1..=50 => !TECHS[field][n as usize].is_empty() && TECH_GROUPS[field][n as usize][1] > 0,
        55..=100 => n % 5 == 0,
        _ => false,
    }
}

/// 1oom `game_tech_get_group` (`None` = `TECH_GROUP_UNUSED`).
pub fn group(field: usize, n: u8) -> Option<u8> {
    (n <= 50 && exists(field, n)).then(|| TECH_GROUPS[field][n as usize][0])
}

/// 1oom `game_tech_get_tier`.
pub fn tier(field: usize, n: u8) -> u8 {
    if group(field, n).is_some() {
        TECH_GROUPS[field][n as usize][1]
    } else {
        0
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
    let mut out: Vec<u8> = Vec::new();
    for row in &t.list[field][..maxtier] {
        for &n in row {
            if n != 0 && !c.contains(&n) && out.len() < NEXT_MAX {
                out.push(n);
            }
        }
    }
    if out.is_empty() {
        let top = if tmax <= 50 { 55 } else { (tmax + 5).min(100) };
        let mut n = 55;
        while n <= top {
            if !c.contains(&n) && out.len() < NEXT_MAX {
                out.push(n);
            }
            n += 5;
        }
    }
    out
}

/// 1oom `game_tech_get_next_rp` (with the mod multiplier `research_cost_percent`).
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

/// 1oom `game_tech_ai_tech_next`.
pub fn ai_choose(st: &mut State, empire: EmpireId, field: usize) {
    let next = next_techs(&st.empires[empire as usize].tech, field);
    if next.is_empty() {
        return;
    }
    let tech = crate::ai::tech_next(st, &next);
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
            let t3 = slider.wrapping_mul(total) / 100;
            let t1 = (invest.wrapping_mul(3) / 20).min(t3.wrapping_mul(2));
            invest = invest.wrapping_add(t3).wrapping_add(t1);
            t.invested[f] = invest;
            t.level[f] = field_level(t, f);
            let cost = t.cost[f];
            if cost != 0 && slider != 0 && total != 0 {
                if cost < invest {
                    let v = (invest - cost).wrapping_mul(250) / cost;
                    if st.rng.one_to(500) as u32 <= v {
                        let project = st.empires[ei].tech.project[f];
                        crate::misc::planet_random(st, ei as EmpireId);
                        grant(st, ei as u8, f, project, Source::Research);
                    }
                }
            } else {
                t.invested[f] = invest * 9 / 10;
            }
        }
    }
    update_tech_util(st);
}

/// 1oom `game_update_tech_util`.
pub fn update_tech_util(st: &mut State) {
    for e in st.empires.iter_mut() {
        e.ab = Abilities::new(&e.tech, e.race);
    }
}

/// 1oom `game_tech_share`: the empires on one side of the final war pool their techs.
fn share(st: &mut State, f: usize, accepted: bool, from_dead: bool) {
    let n = st.empires.len();
    let refuse = |st: &State, i: usize| st.council.refuse.get(i).copied().unwrap_or(false);
    let mut have = [false; 101];
    let mut source = [0u8; 101];
    let mut maxtech = 0u8;
    for pi in 0..n {
        if (!refuse(st, pi)) == accepted && (from_dead || st.empires[pi].alive) {
            for &t in &st.empires[pi].tech.completed[f] {
                have[t as usize] = true;
                maxtech = maxtech.max(t);
                source[t as usize] = pi as u8;
            }
        }
    }
    for pi in 0..n {
        if refuse(st, pi) == accepted || !st.empires[pi].alive {
            continue;
        }
        if st.empires[pi].ai {
            st.empires[pi].tech.completed[f] =
                (0..=maxtech).filter(|&t| have[t as usize]).collect();
        } else {
            for t in 0..=maxtech {
                if have[t as usize] && !st.empires[pi].tech.has(f, t) {
                    grant_from(st, pi as u8, f, t, Source::Trade, Some(source[t as usize]));
                }
            }
        }
    }
}

/// 1oom `game_tech_final_war_share`.
pub fn final_war_share(st: &mut State) {
    for f in 0..NUM_FIELDS {
        share(st, f, true, true);
        share(st, f, false, false);
    }
}

/// 1oom `game_tech_get_orion_loot`: the Death Ray and three advanced techs.
pub fn orion_loot(st: &mut State, empire: EmpireId) {
    grant(st, empire, FIELD_WEAPON, 36, Source::Orion);
    let level = st.empires[empire as usize].tech.level;
    let percent: Vec<u32> = level.iter().map(|&l| (l + 25).min(50)).collect();
    for _ in 0..3 {
        for _ in 0..200 {
            let field = st.rng.below(NUM_FIELDS as u16) as usize;
            let tech = (st.rng.one_to(31) + 19) as u8;
            if percent[field] >= tech as u32
                && group(field, tech).is_some()
                && !st.empires[empire as usize].tech.has(field, tech)
            {
                grant(st, empire, field, tech, Source::Orion);
                break;
            }
        }
    }
}

/// 1oom `game_tech_get_artifact_loot`: one tech for the first human explorer of ruins.
pub fn artifact_loot(st: &mut State, empire: EmpireId) {
    if st.empires[empire as usize].ai {
        return;
    }
    let level = st.empires[empire as usize].tech.level;
    let percent: Vec<u32> = level.iter().map(|&l| (l + 10).min(50)).collect();
    for _ in 0..200 {
        let field = st.rng.below(NUM_FIELDS as u16) as usize;
        let tech = st.rng.one_to(30) as u8;
        if percent[field] >= tech as u32
            && group(field, tech).is_some()
            && !st.empires[empire as usize].tech.has(field, tech)
        {
            grant(st, empire, field, tech, Source::Artifacts);
            break;
        }
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
