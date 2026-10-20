//! Diplomacy of the computer players, ported from 1oom's classic AI (`game_ai_classic.c`:
//! `turn_diplo_p1`, `turn_diplo_p2`, `diplo_wage_war`, `vote`, and the spying part of
//! `turn_p3`). Where 1oom marks a MOO1 bug, the fixed (Classic+) behaviour is used.

use crate::diplo::{
    self, msg, rel, rel_mut, SpyMode, Treaty, ERRATIC, EXPANSIONIST, HONORABLE, MILITARIST, RELDIFF,
};
use crate::spy;
use crate::state::*;
use crate::tech::{self, Source};
use crate::types::*;

fn is_ai(st: &State, e: EmpireId) -> bool {
    st.empires[e as usize].ai
}

fn alive(st: &State, e: EmpireId) -> bool {
    st.empires[e as usize].alive
}

fn year_n(st: &State) -> i32 {
    (st.year as i32 - st.rules.start_year as i32 + 1).max(1)
}

fn ratio_roll(st: &mut State, a: i64, b: i64) -> i32 {
    let ratio = (((a - b) * 100) / b.max(1)).clamp(-300, 300) as i32;
    match ratio {
        0 => 0,
        r if r < 0 => -(st.rng.one_to((-r) as u16) as i32),
        r => st.rng.one_to(r as u16) as i32,
    }
}

/// 1oom `game_ai_classic_diplo_wage_war_fleet_w`.
fn fleet_w(st: &mut State, p1: EmpireId, p2: EmpireId) -> i32 {
    const HULL_W: [i64; 4] = [1, 5, 25, 125];
    let w = |st: &State, e: EmpireId| -> i64 {
        let t = crate::turn::ship_totals(st, e);
        st.empires[e as usize]
            .designs
            .iter()
            .enumerate()
            .map(|(d, sd)| t[d] as i64 * HULL_W[sd.hull.min(3) as usize])
            .sum::<i64>()
            .clamp(1, 3_250_000)
    };
    let (a, b) = (w(st, p1), w(st, p2));
    ratio_roll(st, a, b)
}

/// 1oom `game_ai_classic_diplo_wage_war_prod_w`.
fn prod_w(st: &mut State, p1: EmpireId, p2: EmpireId) -> i32 {
    let a = st.empires[p1 as usize].total_prod as i64;
    let b = (st.empires[p2 as usize].total_prod as i64).clamp(1, 3_250_000);
    ratio_roll(st, a, b)
}

/// 1oom `game_ai_classic_diplo_wage_war_do`: `p2` (a computer) turns on `p1`.
fn wage_war_do(st: &mut State, p1: EmpireId, p2: EmpireId) {
    let t = rel(st, p1, p2).treaty;
    if matches!(t, Treaty::Alliance | Treaty::NonAggression) {
        if (st.difficulty as u8) < Difficulty::Average as u8 {
            return;
        }
        let exp = st.empires[p2 as usize].trait2 == EXPANSIONIST;
        let go = if t == Treaty::Alliance {
            st.rng.below(4) == 0 || (st.rng.below(2) == 0 && exp)
        } else {
            st.rng.below(2) == 0 || exp
        };
        if go {
            diplo::act(st, -10000, p1, p2, msg::TREATY_BROKEN, 0, 0);
            diplo::break_treaty(st, p2, p1);
            if rel(st, p1, p2).relation > 30 {
                rel_mut(st, p1, p2).relation = 30;
                rel_mut(st, p2, p1).relation = 30;
            }
        }
    } else if rel(st, p1, p2).ceasefire <= 0 {
        let kind = if rel(st, p1, p2).relation < 0 {
            msg::WAR_DECLARED
        } else {
            msg::WAR_DECLARED_COLD
        };
        diplo::act(st, -10000, p1, p2, kind, 0, 0);
        diplo::start_war(st, p2, p1);
    }
}

/// 1oom `game_ai_classic_diplo_wage_war`: does the computer `p2` attack `p1`?
fn wage_war(st: &mut State, p1: EmpireId, p2: EmpireId) {
    if st.council.final_war {
        return;
    }
    if rel(st, p1, p2).treaty >= Treaty::War || !rel(st, p1, p2).contact || !alive(st, p1) {
        return;
    }
    let ok_human = |st: &State| is_ai(st, p1) || rel(st, p1, p2).ceasefire < 1;
    let t1 = st.empires[p2 as usize].trait1;
    let t2 = st.empires[p2 as usize].trait2;
    if t1 == ERRATIC && st.rng.one_to(300) <= st.difficulty as u16 && ok_human(st) {
        let r = rel_mut(st, p1, p2);
        r.diplo_type = msg::WAR_DECLARED_ERRATIC;
        r.diplo_val = 2000;
        diplo::start_war(st, p2, p1);
    } else {
        if st.rng.below(20) == 0
            && (t2 == MILITARIST || t2 == EXPANSIONIST)
            && t1 != HONORABLE
            && ok_human(st)
        {
            let w = fleet_w(st, p1, p2);
            let r = rel(st, p1, p2);
            let v = r.relation - w + RELDIFF[t1 as usize] + r.trust;
            if v < -150 {
                wage_war_do(st, p1, p2);
            }
        }
        if st.rng.below(20) == 0 && ok_human(st) {
            let w = prod_w(st, p1, p2);
            let r = rel(st, p1, p2);
            let v = r.relation - w + RELDIFF[t1 as usize] + r.trust;
            if v < -150 {
                wage_war_do(st, p1, p2);
            }
        }
    }
    let n = st.empires.len() as u8;
    for p3 in 0..n {
        if !is_ai(st, p3) {
            continue;
        }
        for p4 in 0..n {
            if is_ai(st, p4) {
                continue;
            }
            if rel(st, p4, p3).treaty == Treaty::War
                && rel(st, p2, p3).treaty == Treaty::Alliance
                && rel(st, p4, p2).ceasefire <= 0
                && st.rng.below(10) == 0
                && rel(st, p2, p1).treaty == Treaty::Alliance
            {
                wage_war_do(st, p1, p2);
            }
        }
    }
    if !is_ai(st, p1) && st.difficulty as u8 >= Difficulty::Average as u8 {
        let wars = (0..n)
            .filter(|&p3| rel(st, p1, p3).treaty >= Treaty::War)
            .count() as u8;
        if wars < st.difficulty as u8 {
            let v = rel(st, p1, p2).relation;
            if v < -30 && st.rng.one_to(10) as i32 <= (-v) / 10 {
                wage_war_do(st, p1, p2);
            }
        }
    }
}

/// 1oom `game_ai_classic_turn_diplo_p1_get_ai_trade_tech`: a random tech `p2` lacks from `p1`.
fn trade_tech(st: &mut State, p1: EmpireId, p2: EmpireId) -> Option<diplo::TechRef> {
    let r = st.rng.one_to(3) as usize;
    let lists = spy::useful_techs(st, p2, p1, r);
    let num: usize = lists.iter().map(|l| l.len()).sum();
    if num == 0 {
        return None;
    }
    let mut i = (year_n(st) as usize ^ r) % num;
    for (f, l) in lists.iter().enumerate() {
        if i < l.len() {
            return Some((f as u8, l[i]));
        }
        i -= l.len();
    }
    None
}

/// 1oom `game_ai_classic_turn_diplo_p1`: computer empires deal with each other.
pub fn turn_p1(st: &mut State) {
    let n = st.empires.len() as u8;
    for p1 in 0..n {
        if !is_ai(st, p1) {
            continue;
        }
        for p2 in 0..n {
            if p1 == p2 || !is_ai(st, p2) {
                continue;
            }
            let roll = st.rng.below((15 - st.difficulty as u16 * 2).max(1));
            if roll != 0 || !rel(st, p1, p2).contact {
                continue;
            }
            let r = rel(st, p1, p2).clone();
            let mut v = r.trust
                + r.relation
                + RELDIFF[st.empires[p2 as usize].trait1 as usize]
                + st.rng.one_to(100) as i32;
            v += match r.treaty {
                Treaty::NonAggression => 10,
                Treaty::Alliance => 20,
                _ => 0,
            };
            if r.trade_bc != 0 {
                v += 10;
            }
            if v + r.mood_treaty > 150 && r.treaty != Treaty::Alliance {
                diplo::set_treaty(st, p1, p2, Treaty::Alliance);
            } else if v + r.mood_treaty > 150 && r.treaty != Treaty::NonAggression {
                diplo::set_treaty(st, p1, p2, Treaty::NonAggression);
            } else if r.mood_tech + 80 < v {
                let a = trade_tech(st, p1, p2);
                let b = trade_tech(st, p2, p1);
                if let (Some(a), Some(b)) = (a, b) {
                    tech::grant_from(st, p2, a.0 as usize, a.1, Source::Trade, Some(p1));
                    tech::grant_from(st, p1, b.0 as usize, b.1, Source::Trade, Some(p2));
                }
            } else if v > 70 {
                let v1 = (st.empires[p1 as usize].total_prod / 4)
                    .min(st.empires[p2 as usize].total_prod / 4)
                    .min(32000);
                diplo::set_trade(st, p1, p2, v1);
            }
            if st.rng.below(20) == 0 {
                let d = st.rng.one_to(5) as i32;
                diplo::act(st, d, p1, p2, 1, 0, 0);
            }
            if rel(st, p1, p2).treaty == Treaty::War && v + rel(st, p1, p2).mood_peace > 100 {
                diplo::stop_war(st, p1, p2);
            }
            diplo::annoy(st, p1, p2, 3);
            break_double_alliances(st);
            wage_war(st, p1, p2);
        }
    }
    for p1 in 0..n {
        if !is_ai(st, p1) {
            continue;
        }
        for p2 in 0..n {
            if p1 == p2 || !is_ai(st, p2) || rel(st, p1, p2).treaty != Treaty::Alliance {
                continue;
            }
            for p3 in 0..n {
                if p3 != p1
                    && p3 != p2
                    && is_ai(st, p3)
                    && rel(st, p3, p1).treaty == Treaty::War
                    && rel(st, p3, p2).treaty != Treaty::War
                {
                    diplo::start_war(st, p2, p3);
                }
            }
        }
    }
}

/// 1oom `game_ai_classic_turn_diplo_p1_sub1` (with its loop fixed): an empire allied with
/// two empires at war with each other leaves both alliances.
fn break_double_alliances(st: &mut State) {
    let n = st.empires.len() as u8;
    for p1 in 0..n {
        if !is_ai(st, p1) {
            continue;
        }
        for p2 in p1 + 1..n {
            if !is_ai(st, p2) {
                continue;
            }
            for p3 in p2 + 1..n {
                if !is_ai(st, p3) {
                    continue;
                }
                if rel(st, p2, p3).treaty == Treaty::War
                    && rel(st, p1, p2).treaty == Treaty::Alliance
                    && rel(st, p1, p3).treaty == Treaty::Alliance
                {
                    diplo::break_treaty(st, p1, p2);
                    diplo::break_treaty(st, p1, p3);
                }
            }
        }
    }
}

fn any_fleet_to(st: &State, owner: EmpireId, target: EmpireId) -> bool {
    st.fleets
        .iter()
        .any(|f| f.owner == owner && st.planets[f.dest].owner == Some(target))
}

fn sweetener(st: &mut State, p1: EmpireId, p2: EmpireId) {
    if st.rng.below(4) != 0 {
        let v = ((st.empires[p2 as usize].total_prod / (st.rng.one_to(3) as i32 + 1) + year_n(st))
            / 25)
            * 25;
        rel_mut(st, p1, p2).offer_bc = v;
    } else if let Some(o) = spy::pick_techs(st, p1, p2, 0, 2).first().copied() {
        rel_mut(st, p1, p2).offer_tech = Some((o.field, o.tech));
    }
}

/// 1oom `game_ai_classic_turn_diplo_p2_sub1`: what the computer `p2` proposes to the human `p1`.
fn propose(st: &mut State, p1: EmpireId, p2: EmpireId) {
    if !rel(st, p1, p2).contact || rel(st, p1, p2).treaty >= Treaty::War || any_fleet_to(st, p2, p1)
    {
        rel_mut(st, p1, p2).diplo_type = 0;
        return;
    }
    if rel(st, p1, p2).diplo_type != 0 && st.rng.below(2) == 0 {
        return;
    }
    let r = rel(st, p1, p2).clone();
    let human = if st.empires[p1 as usize].race == Race::Human {
        50
    } else {
        0
    };
    let t1 = st.empires[p2 as usize].trait1 as usize;
    let v = r.trust + r.relation + human + RELDIFF[t1];
    let mut v4 = 0;
    if r.treaty == Treaty::None
        && r.relation > 15
        && v + st.rng.one_to(100) as i32 + r.mood_treaty > 50
    {
        v4 = 1;
        rel_mut(st, p1, p2).diplo_type = msg::PROPOSE_NAP;
    }
    if (r.treaty == Treaty::NonAggression || v4 == 1)
        && r.relation > 50
        && v + st.rng.one_to(100) as i32 + r.mood_treaty > 100
    {
        v4 = 2;
        rel_mut(st, p1, p2).diplo_type = msg::PROPOSE_ALLIANCE;
    }
    if v4 == 0 && v + st.rng.one_to(150) as i32 + r.mood_trade > 50 {
        let prod = (st.empires[p1 as usize]
            .total_prod
            .min(st.empires[p2 as usize].total_prod)
            / 4)
        .min(32000);
        let mut want = prod / 25;
        if want != 0 {
            want = st.rng.one_to(want as u16) as i32 * 25;
        }
        if r.trade_bc < want {
            v4 = 3;
            let rr = rel_mut(st, p1, p2);
            rr.diplo_type = msg::PROPOSE_TRADE;
            rr.want_trade = want;
        }
    }
    if v4 == 0 {
        let n = st.empires.len() as u8;
        let pa = (0..n).rev().find(|&p3| {
            p3 != p1 && p3 != p2 && rel(st, p2, p3).treaty >= Treaty::War && rel(st, p1, p3).contact
        });
        if let Some(pa) = pa {
            if st.rng.below(10) == 0 && r.attack_bounty.is_none() {
                let techs = spy::pick_techs(st, p1, p2, 0, 2);
                let bc = if techs.is_empty() {
                    ((st.rng.one_to(3) as i32 + 1) * year_n(st) / 50) * 50
                } else {
                    0
                };
                if bc != 0 || !techs.is_empty() {
                    v4 = 4;
                    let rr = rel_mut(st, p1, p2);
                    rr.diplo_type = msg::OFFER_BOUNTY;
                    rr.attack_bounty = Some(pa);
                    rr.attack_gift_tech = techs.first().map(|o| (o.field, o.tech));
                    rr.attack_gift_bc = bc;
                    rr.bounty_collect = None;
                }
            }
        }
    }
    if v4 == 0 && v + st.rng.one_to(100) as i32 + r.mood_trade > 25 {
        let mut v1c = r.mood_tech;
        if v1c > 0 {
            v1c /= 5;
        }
        v1c = v1c.min(30);
        if r.treaty == Treaty::Alliance {
            v1c += 25;
        }
        let human2 = if st.empires[p2 as usize].race == Race::Human {
            50
        } else {
            0
        };
        let v = r.trust + r.relation / 2 + human2 + RELDIFF[t1] + v1c - 125;
        let v8 = v + st.rng.one_to(100) as i32 + v1c - 125;
        let v14 = if v8 < 0 {
            v8.abs() + 100
        } else {
            20000 / (v8 + 200)
        }
        .min(50);
        let theirs = spy::pick_techs(st, p2, p1, 0, 0);
        if let Some(z) = theirs.first().copied() {
            let zvalue = z.value * 100 / v14.max(1);
            let mine = spy::pick_techs(st, p1, p2, zvalue, 1);
            let ok: Vec<diplo::TechRef> = mine
                .iter()
                .filter(|o| o.value <= zvalue)
                .map(|o| (o.field, o.tech))
                .collect();
            if !ok.is_empty() {
                v4 = 5;
                let rr = rel_mut(st, p1, p2);
                rr.diplo_type = msg::OFFER_TECH_TRADE;
                rr.want_tech = Some((z.field, z.tech));
                rr.tech_trade = ok;
                rr.mood_tech = 0;
                rel_mut(st, p2, p1).mood_tech = 0;
            }
        }
    }
    if v4 == 0
        && r.relation < 0
        && st.empires[p1 as usize].total_prod > st.empires[p2 as usize].total_prod
        && r.treaty == Treaty::None
    {
        v4 = 1;
        rel_mut(st, p1, p2).diplo_type = msg::PROPOSE_NAP;
    }
    let p2prod = st.empires[p2 as usize].total_prod;
    if p2prod != 0
        && (v4 == 1 || v4 == 2)
        && st.empires[p2 as usize].trait1 > diplo::RUTHLESS
        && st.rng.one_to(100) as i32 <= st.empires[p1 as usize].total_prod * 30 / p2prod
        && st.rng.below(8) == 0
    {
        sweetener(st, p1, p2);
    }
}

/// 1oom `game_ai_classic_turn_diplo_p2_sub2`: an ally asks for help in a war.
fn ally_request(st: &mut State, p1: EmpireId, p2: EmpireId) {
    if rel(st, p1, p2).treaty == Treaty::Alliance && st.rng.below(10) == 0 {
        let n = st.empires.len() as u8;
        let pa = (0..n).rev().find(|&p3| {
            p3 != p1
                && p3 != p2
                && rel(st, p3, p2).treaty == Treaty::War
                && rel(st, p1, p3).contact
                && rel(st, p1, p3).treaty != Treaty::War
        });
        let r = rel_mut(st, p1, p2);
        r.ally_attacker = pa;
        r.diplo_type = if pa.is_none() { 0 } else { msg::JOIN_WAR };
    } else {
        rel_mut(st, p1, p2).diplo_type = 0;
    }
}

/// 1oom `game_ai_classic_turn_diplo_p2_sub3`: grievances, threats, war or peace offers.
fn grievance(st: &mut State, p1: EmpireId, p2: EmpireId) {
    if !rel(st, p1, p2).contact {
        rel_mut(st, p1, p2).diplo_type = 0;
        return;
    }
    let r = rel(st, p1, p2).clone();
    let human = if st.empires[p1 as usize].race == Race::Human {
        50
    } else {
        0
    };
    let t1 = st.empires[p2 as usize].trait1 as usize;
    let v = r.trust + r.relation + human + RELDIFF[t1];
    if r.treaty < Treaty::War {
        if r.relation <= -95 {
            diplo::start_war(st, p2, p1);
            rel_mut(st, p1, p2).diplo_type = msg::WAR_DECLARED;
        } else if v - st.rng.one_to(100) as i32 + r.mood_treaty <= -100 {
            if r.hatred > 0 {
                rel_mut(st, p1, p2).hatred += 1;
                let h = rel(st, p1, p2).hatred;
                if h > 2 || r.treaty == Treaty::None {
                    if (st.rng.one_to(100) as i32) < -(RELDIFF[t1] / 2 + r.diplo_val) {
                        diplo::start_war(st, p2, p1);
                        let rr = rel_mut(st, p1, p2);
                        rr.diplo_type = msg::WAR_DECLARED;
                        rr.hatred = 1;
                    }
                } else {
                    rel_mut(st, p1, p2).diplo_type += msg::BREAK_GRIEVANCE;
                    diplo::break_treaty(st, p2, p1);
                }
            } else {
                rel_mut(st, p1, p2).hatred = 1;
                if matches!(r.treaty, Treaty::NonAggression | Treaty::Alliance) || r.trade_bc != 0 {
                    rel_mut(st, p1, p2).diplo_type += msg::WARN_GRIEVANCE;
                }
            }
        }
    } else if v + st.rng.one_to(100) as i32 + r.mood_peace >= 50 && st.rng.below(8) == 0 {
        rel_mut(st, p1, p2).diplo_type = msg::PROPOSE_PEACE;
        let p1prod = st.empires[p1 as usize].total_prod.max(1);
        if st.rng.one_to(100) as i32 <= st.empires[p2 as usize].total_prod * 30 / p1prod {
            sweetener(st, p1, p2);
        }
    }
}

/// 1oom `game_ai_classic_turn_diplo_p2`: what each computer empire tells each human.
pub fn turn_p2(st: &mut State) {
    diplo::limit_mood_treaty(st);
    let n = st.empires.len() as u8;
    for p1 in 0..n {
        if is_ai(st, p1) {
            continue;
        }
        if !alive(st, p1) {
            for r in st.empires[p1 as usize].rel.iter_mut() {
                r.diplo_type = 0;
            }
            continue;
        }
        for p2 in 0..n {
            if !is_ai(st, p2) {
                continue;
            }
            if !alive(st, p2) {
                rel_mut(st, p1, p2).diplo_type = 0;
                continue;
            }
            let r = rel(st, p1, p2).clone();
            if r.treaty == Treaty::FinalWar || r.diplo_type == msg::FINAL_WAR {
                if r.diplo_type != msg::FINAL_WAR {
                    rel_mut(st, p1, p2).diplo_type = 0;
                }
            } else if r.have_met == 1 {
                let t1 = st.empires[p2 as usize].trait1;
                let rr = rel_mut(st, p1, p2);
                rr.have_met = 2;
                rr.diplo_type = msg::FIRST_CONTACT + t1;
            } else if r.bounty_collect.is_some() {
                rel_mut(st, p1, p2).diplo_type = msg::PAYS_BOUNTY;
            } else if matches!(
                r.diplo_type,
                msg::WAR_DECLARED
                    | msg::TREATY_BROKEN
                    | msg::WAR_DECLARED_ERRATIC
                    | msg::WAR_DECLARED_COLD
            ) {
            } else {
                let mood = diplo::mood(st, p1, p2);
                let v2 = mood + r.trust + RELDIFF[st.empires[p2 as usize].trait1 as usize];
                let dv2 = r.diplo_val * 2;
                if v2 <= -100 || mood <= -100 {
                    rel_mut(st, p1, p2).diplo_type = 0;
                } else if r.diplo_val < 0 {
                    if r.hatred > 0 || (st.rng.one_to(100) as i32) < dv2.abs() {
                        grievance(st, p1, p2);
                    } else {
                        rel_mut(st, p1, p2).diplo_type = 0;
                    }
                } else if r.treaty == Treaty::War && (st.rng.one_to(100) as i32 + 30) < r.mood_peace
                {
                    grievance(st, p1, p2);
                } else if (st.rng.one_to(100) as i32) < dv2 + if r.hatred == 0 { 3 } else { 0 }
                    && st.rng.below(4) == 0
                {
                    if r.hatred > 0 {
                        rel_mut(st, p1, p2).hatred = 0;
                    } else {
                        propose(st, p1, p2);
                    }
                } else {
                    ally_request(st, p1, p2);
                }
            }
            if rel(st, p1, p2).diplo_type == 0 {
                wage_war(st, p1, p2);
            }
            if rel(st, p1, p2).diplo_type == 2 && st.rng.below(10) != 0 {
                rel_mut(st, p1, p2).diplo_type = 0;
            }
        }
    }
}

/// 1oom `game_ai_classic_vote`: 0 abstain, 1 or 2 for a candidate.
pub fn vote(st: &mut State, player: EmpireId, c1: EmpireId, c2: EmpireId) -> u8 {
    if player == c1 {
        return 1;
    }
    if player == c2 {
        return 2;
    }
    let t = |c| rel(st, player, c).treaty;
    if t(c1) == Treaty::Alliance {
        return 1;
    }
    if t(c2) == Treaty::Alliance {
        return 2;
    }
    if t(c1) == Treaty::War {
        return 2;
    }
    if t(c2) == Treaty::War {
        return 1;
    }
    let w = |st: &State, cand: EmpireId| {
        let mut v = 0;
        if st.planets[st.orion].owner == Some(cand) {
            v += 20;
        }
        if st.empires[cand as usize].race == Race::Human {
            v += 20;
        }
        if st.council.voted.get(player as usize).copied().flatten() == Some(cand) {
            v += 40;
        }
        v
    };
    let r1 = rel(st, player, c1).relation;
    let r2 = rel(st, player, c2).relation;
    if r1 >= 0 {
        let v = r1 / 4 + w(st, c1);
        if (st.rng.one_to(100) as i32) < v {
            return 1;
        }
    } else {
        let v = -r1 / 4 + w(st, c2);
        if (st.rng.one_to(100) as i32) < v {
            return 2;
        }
    }
    if r2 >= 0 {
        let v = r2 / 4 + w(st, c2);
        if (st.rng.one_to(100) as i32) < v {
            return 2;
        }
    } else {
        let v = -r2 / 5 + w(st, c1);
        if (st.rng.one_to(100) as i32) < v {
            return 1;
        }
    }
    0
}

/// Spying and security of a computer empire (1oom `game_ai_classic_turn_p3` and `_sub1`).
pub fn spying(st: &mut State, pi: EmpireId) {
    if st.turn <= 30 {
        return;
    }
    let n = st.empires.len() as u8;
    let total: i32 = (0..n)
        .filter(|&p2| rel(st, pi, p2).contact)
        .map(|p2| rel(st, p2, pi).spies)
        .sum();
    let mut sec = if total == 0 {
        0
    } else {
        st.rng.below(2) as i32 * total
    };
    sec = (sec + st.difficulty as i32 * 8).min(100);
    st.empires[pi as usize].security = sec;
    for p2 in 0..n {
        rel_mut(st, pi, p2).spymode_next = SpyMode::Hide;
        let human_cease = !is_ai(st, p2) && rel(st, p2, pi).ceasefire > 0;
        if human_cease || p2 == pi || !rel(st, pi, p2).contact {
            continue;
        }
        if rel(st, pi, p2).treaty >= Treaty::War {
            rel_mut(st, pi, p2).spymode_next = SpyMode::Sabotage;
        } else if st.empires[pi as usize].race == Race::Darlok || st.rng.below(2) == 1 {
            let r = rel(st, pi, p2).relation;
            if st.rng.one_to(200) as i32 > r * 2 + 200 {
                rel_mut(st, pi, p2).spymode_next = SpyMode::Espionage;
            }
        }
    }
    for p2 in 0..n {
        {
            let r = rel_mut(st, pi, p2);
            r.spying = 0;
            r.spymode = SpyMode::Hide;
        }
        let r = rel(st, pi, p2);
        let candidate = p2 != pi && r.contact && r.spymode_next != SpyMode::Hide;
        if candidate && (!is_ai(st, p2) || st.rng.below(3) == 0) {
            let next = rel(st, pi, p2).spymode_next;
            let s = st.rng.one_to(5) as i32 + 5;
            let r = rel_mut(st, pi, p2);
            r.spymode = next;
            r.spying = s;
        }
    }
}
