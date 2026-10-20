//! Diplomacy: relations, treaties and trade between empires (1oom `game_diplo.c` and the
//! diplomacy steps of `game_turn.c`).
//!
//! `empires[a].rel[b]` holds the per-pair values of 1oom's `eto[a].x[b]`. Relations are
//! the computer players' feelings: they are kept in sync on both sides, and `act` only
//! moves them when the one who feels (`pi2`) is a computer player.

use serde::{Deserialize, Serialize};

use crate::state::*;
use crate::types::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Treaty {
    #[default]
    None,
    NonAggression,
    Alliance,
    War,
    FinalWar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpyMode {
    #[default]
    Hide,
    Espionage,
    Sabotage,
}

/// A tech of a field: `(field, tech number)`.
pub type TechRef = (u8, u8);

/// Personality (1oom `trait1_t`).
pub const XENOPHOBIC: u8 = 0;
pub const RUTHLESS: u8 = 1;
pub const AGGRESSIVE: u8 = 2;
pub const ERRATIC: u8 = 3;
pub const HONORABLE: u8 = 4;
pub const PACIFISTIC: u8 = 5;
/// Objective (1oom `trait2_t`).
pub const DIPLOMAT: u8 = 0;
pub const MILITARIST: u8 = 1;
pub const EXPANSIONIST: u8 = 2;
pub const TECHNOLOGIST: u8 = 3;
pub const INDUSTRIALIST: u8 = 4;
pub const ECOLOGIST: u8 = 5;

/// 1oom `game_diplo_tbl_reldiff`: how easy each personality is to deal with.
pub const RELDIFF: [i32; 6] = [-50, -30, -20, 0, 10, 20];

/// 1oom `game_num_tbl_trait1` and `game_num_tbl_trait2`: personality and objective rolls per race.
pub const TRAIT1_TBL: [[u8; 10]; 10] = [
    [4, 4, 4, 4, 4, 4, 4, 5, 5, 3],
    [1, 1, 1, 1, 1, 1, 1, 2, 2, 0],
    [0, 0, 0, 0, 0, 0, 0, 2, 2, 3],
    [2, 2, 2, 2, 2, 2, 2, 3, 3, 1],
    [5, 5, 5, 5, 5, 5, 5, 4, 3, 3],
    [4, 4, 4, 4, 4, 4, 4, 5, 5, 3],
    [0, 0, 0, 0, 0, 0, 0, 2, 2, 1],
    [2, 2, 2, 2, 2, 2, 2, 3, 3, 1],
    [3, 3, 3, 3, 3, 3, 3, 2, 2, 0],
    [2, 2, 2, 2, 2, 2, 2, 1, 1, 0],
];
pub const TRAIT2_TBL: [[u8; 12]; 10] = [
    [0, 0, 0, 0, 1, 2, 3, 4, 4, 0, 3, 5],
    [0, 1, 1, 1, 1, 3, 2, 2, 3, 3, 4, 5],
    [0, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 5],
    [0, 3, 1, 5, 5, 3, 4, 2, 2, 2, 2, 2],
    [0, 0, 3, 1, 2, 3, 3, 3, 3, 4, 4, 5],
    [0, 1, 1, 1, 2, 2, 2, 3, 4, 2, 3, 5],
    [0, 1, 1, 2, 2, 3, 3, 4, 5, 5, 5, 5],
    [0, 5, 5, 5, 5, 2, 2, 3, 3, 3, 4, 5],
    [0, 1, 2, 2, 3, 3, 3, 4, 4, 4, 4, 5],
    [0, 0, 3, 3, 1, 2, 0, 3, 3, 4, 4, 5],
];

/// 1oom `tbl_orion_race_relation`: starting relations between races (x 12).
pub const RACE_RELATION: [[i32; 10]; 10] = [
    [0, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, -2, 0, -3, -1, -1, 0, -1],
    [1, 0, 0, 0, 0, 0, -1, 0, 1, -1],
    [1, -2, 0, 0, 0, -1, -1, 0, -1, -1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, -1],
    [1, -3, 0, -1, 0, 0, -1, 0, 0, -1],
    [1, -1, -1, -1, 0, -1, 0, 0, 0, -1],
    [1, -1, 0, 0, 0, 0, 0, 0, 0, -1],
    [1, 0, 1, -1, 0, 0, 0, 0, 0, -1],
    [1, -1, -1, -1, -1, -1, -1, -1, -1, 1],
];

/// Message types (1oom `diplo_type`; the UI shows `diplo.<type>`).
pub mod msg {
    pub const TRIBUTE_THANKS: u8 = 1;
    pub const SPY_CAUGHT: u8 = 4;
    pub const SPY_FRAMED: u8 = 5;
    pub const SABOTAGE: u8 = 6;
    pub const SABOTAGE_FRAMED: u8 = 7;
    pub const FLEET_THREAT: u8 = 8;
    pub const ATTACKED: u8 = 9;
    pub const KILLED_POP: u8 = 10;
    pub const BIO_WEAPONS: u8 = 11;
    pub const EXPANSION: u8 = 12;
    pub const WAR_DECLARED: u8 = 13;
    /// 14 + personality: first contact.
    pub const FIRST_CONTACT: u8 = 14;
    pub const REFUSE_AT_WAR: u8 = 20;
    pub const REFUSE_HOSTILE: u8 = 21;
    pub const GREETING: u8 = 22;
    pub const GREETING_COLD: u8 = 23;
    pub const PROPOSE_NAP: u8 = 24;
    pub const PROPOSE_ALLIANCE: u8 = 25;
    pub const PROPOSE_TRADE: u8 = 26;
    pub const ASK_BREAK_ALLIANCE: u8 = 27;
    pub const OFFER_BOUNTY: u8 = 28;
    pub const OFFER_TECH_TRADE: u8 = 29;
    pub const PROPOSE_PEACE: u8 = 30;
    pub const REFUSE: u8 = 31;
    pub const TREATY_BROKEN: u8 = 32;
    pub const REFUSE_BROKEN_TREATY: u8 = 33;
    /// 30 + grievance (4..=11): refusal citing a grievance.
    pub const REFUSE_GRIEVANCE: u8 = 30;
    /// 38 + grievance: warning.
    pub const WARN_GRIEVANCE: u8 = 38;
    /// 46 + grievance: they break the treaty over it.
    pub const BREAK_GRIEVANCE: u8 = 46;
    pub const PAYS_BOUNTY: u8 = 58;
    pub const FINAL_WAR: u8 = 59;
    pub const WAR_DECLARED_COLD: u8 = 60;
    pub const WAR_DECLARED_ERRATIC: u8 = 61;
    pub const ACCEPT_NAP: u8 = 62;
    pub const ACCEPT_ALLIANCE: u8 = 63;
    pub const ACCEPT_TRADE: u8 = 64;
    pub const ACCEPT_PEACE: u8 = 65;
    pub const ACCEPT_TRIBUTE_TECH: u8 = 66;
    pub const ACCEPT_WAR: u8 = 67;
    pub const ACCEPT_BREAK: u8 = 68;
    pub const THREAT_IGNORED: u8 = 69;
    pub const THREAT_HEEDED: u8 = 70;
    pub const THREAT_TRIBUTE: u8 = 71;
    pub const YOU_BROKE: u8 = 73;
    pub const LATER: u8 = 74;
    pub const NOTHING_TO_TRADE: u8 = 75;
    pub const JOIN_WAR: u8 = 76;
    pub const ALLIANCE_ENDED: u8 = 77;
    pub const THANKS_JOIN_WAR: u8 = 78;
    pub const VOTED_AGAINST: u8 = 79;
    pub const VOTED_FOR: u8 = 80;
}

/// What one empire has with another (1oom `empiretechorbit_t` arrays indexed by player).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Relation {
    pub contact: bool,
    pub contact_broken: bool,
    /// 0 never met, 1 just met (greeting pending), 2 met.
    pub have_met: u8,
    /// Current relation, -100..100 (1oom `relation1`).
    pub relation: i32,
    /// Long-term relation the current one drifts to (1oom `relation2`).
    pub base_relation: i32,
    pub trust: i32,
    pub treaty: Treaty,
    pub broken_treaty: Treaty,
    pub blunder: u8,
    pub mood_treaty: i32,
    pub mood_trade: i32,
    pub mood_tech: i32,
    pub mood_peace: i32,
    pub trade_bc: i32,
    pub trade_established: i32,
    pub trade_percent: i32,
    pub hatred: i32,
    /// Message from this other (computer) empire to a human (1oom `diplo_type` and params).
    pub diplo_type: u8,
    pub diplo_val: i32,
    pub diplo_p1: usize,
    pub diplo_p2: i32,
    pub tribute: Option<TechRef>,
    pub offer_bc: i32,
    pub offer_tech: Option<TechRef>,
    pub want_trade: i32,
    pub want_tech: Option<TechRef>,
    pub tech_trade: Vec<TechRef>,
    pub ask_break_treaty: Option<EmpireId>,
    pub ally_attacker: Option<EmpireId>,
    pub attack_bounty: Option<EmpireId>,
    pub bounty_collect: Option<EmpireId>,
    pub attack_gift_tech: Option<TechRef>,
    pub attack_gift_bc: i32,
    /// Spending on spies against this empire, tenths of a percent of production.
    pub spying: i32,
    pub spyfund: i32,
    pub spymode: SpyMode,
    pub spymode_next: SpyMode,
    pub spies: i32,
    pub spy_report_year: u32,
    pub spy_report: [u8; NUM_FIELDS],
    /// Turns this human empire's ships do not fight the other (1oom `ceasefire`).
    pub ceasefire: i32,
}

pub fn rel(st: &State, a: EmpireId, b: EmpireId) -> &Relation {
    &st.empires[a as usize].rel[b as usize]
}

pub fn rel_mut(st: &mut State, a: EmpireId, b: EmpireId) -> &mut Relation {
    &mut st.empires[a as usize].rel[b as usize]
}

pub fn treaty(st: &State, a: EmpireId, b: EmpireId) -> Treaty {
    if a == b {
        Treaty::Alliance
    } else {
        rel(st, a, b).treaty
    }
}

fn is_human(st: &State, e: EmpireId) -> bool {
    !st.empires[e as usize].ai
}

/// Whether ships of `a` and `b` fight at `planet` (1oom `game_battle_handle_all`): allies
/// never, NAP partners only at a colony of one of them, ceasefires hold.
pub fn hostile_at(st: &State, a: EmpireId, b: EmpireId, planet: PlanetId) -> bool {
    if a == b {
        return false;
    }
    let owner = st.planets[planet].owner;
    let t = treaty(st, a, b);
    if t == Treaty::Alliance {
        return false;
    }
    if t == Treaty::NonAggression && owner != Some(a) && owner != Some(b) {
        return false;
    }
    rel(st, a, b).ceasefire <= 0 && rel(st, b, a).ceasefire <= 0
}

/// 1oom `game_diplo_act`: `pi` did something worth `dv` to `pi2`.
pub fn act(st: &mut State, dv: i32, pi: EmpireId, pi2: EmpireId, dtype: u8, p1: usize, p2: i32) {
    let n = st.empires.len() as u8;
    if st.council.final_war {
        if !is_human(st, pi)
            && dv < 0
            && pi2 < n
            && is_human(st, pi2)
            && st
                .council
                .refuse
                .get(pi2 as usize)
                .copied()
                .unwrap_or(false)
            && st.rng.one_to(100) as i32 <= dv / -10
        {
            if let Some(w) = st.council.winner {
                let r = rel_mut(st, pi2, w);
                r.diplo_val = 100;
                r.diplo_type = msg::FINAL_WAR;
                r.diplo_p2 = p2;
                r.diplo_p1 = p1;
            }
        }
        return;
    }
    if pi >= n
        || pi2 >= n
        || pi == pi2
        || dv == 0
        || is_human(st, pi2)
        || treaty(st, pi, pi2) == Treaty::FinalWar
    {
        return;
    }
    if dv == -10000 {
        if is_human(st, pi) && dtype != 0 {
            let r = rel_mut(st, pi, pi2);
            r.diplo_val = 10000;
            r.diplo_type = dtype;
            r.diplo_p2 = p2;
            r.diplo_p1 = p1;
        }
        return;
    }
    let mut dv = dv;
    let r2 = rel(st, pi2, pi).relation;
    if dv > 0 {
        if r2 < 0 {
            dv *= 2;
            if r2 + dv > 10 {
                dv = 10 - r2;
            }
        } else {
            dv /= (r2 / 25 + 1).max(1);
        }
    } else if r2 > 0 {
        dv *= 2;
    } else {
        dv /= (r2 / -25 + 1).max(1);
    }
    let t1 = st.empires[pi2 as usize].trait1;
    if t1 == HONORABLE && dv < 0 {
        dv *= 2;
    }
    if t1 == XENOPHOBIC && dv < 0 {
        dv = dv * 3 / 2;
    }
    if t1 == XENOPHOBIC && dv > 0 {
        dv = dv * 3 / 4;
    }
    let v1 = dv;
    if st.empires[pi as usize].race == Race::Human && dv > 0 {
        dv *= 2;
    }
    if rel(st, pi2, pi).treaty == Treaty::War && (4..=12).contains(&dtype) && dv < 0 {
        if is_human(st, pi) {
            rel_mut(st, pi2, pi).mood_peace += dv / 4;
            rel_mut(st, pi, pi2).mood_peace -= dv / 4;
        }
        dv = 0;
    }
    if !is_human(st, pi) && !is_human(st, pi2) && st.turn > 100 {
        if dv > 0 {
            dv *= 2;
        }
        if dv < 0 {
            dv /= st.difficulty as i32 / 2 + 1;
        }
    }
    if dv != 0 {
        let v = (rel(st, pi2, pi).relation + dv).clamp(-100, 100);
        let human = is_human(st, pi);
        {
            let r = rel_mut(st, pi2, pi);
            r.relation = v;
            r.mood_treaty += dv;
            r.mood_trade += dv;
            r.mood_tech += dv;
        }
        let (mt, mtr, mte) = {
            let r = rel(st, pi2, pi);
            (r.mood_treaty, r.mood_trade, r.mood_tech)
        };
        {
            let r = rel_mut(st, pi, pi2);
            r.mood_treaty = mt;
            r.mood_trade = mtr;
            r.mood_tech = mte;
        }
        if v1 < 0 {
            rel_mut(st, pi, pi2).mood_peace -= v1 / 4;
            rel_mut(st, pi2, pi).mood_peace += v1 / 4;
        } else {
            rel_mut(st, pi2, pi).mood_peace += v1;
        }
        let r = rel_mut(st, pi, pi2);
        if human && dtype != 0 && dv.abs() > r.diplo_val.abs() {
            r.diplo_val = dv;
            r.diplo_type = dtype;
            r.diplo_p2 = p2;
            r.diplo_p1 = p1;
        }
    }
    let r2 = rel_mut(st, pi2, pi);
    if r2.treaty >= Treaty::War {
        r2.relation = r2.relation.min(-25);
    }
    if r2.treaty != Treaty::Alliance {
        r2.relation = r2.relation.min(65);
    }
    let v = r2.relation;
    rel_mut(st, pi, pi2).relation = v;
}

fn set_moods(r: &mut Relation, v: i32) {
    r.mood_treaty = v;
    r.mood_trade = v;
    r.mood_tech = v;
    r.mood_peace = v;
}

/// 1oom `game_diplo_break_treaty` (with the Classic+ fixes).
pub fn break_treaty(st: &mut State, breaker: EmpireId, victim: EmpireId) {
    if rel(st, victim, breaker).treaty >= Treaty::War {
        return;
    }
    let t = rel(st, breaker, victim).treaty;
    let mut w = match t {
        Treaty::NonAggression => 10,
        Treaty::Alliance => 20,
        _ => 0,
    };
    if st.empires[breaker as usize].trait1 == HONORABLE {
        w *= 2;
    }
    rel_mut(st, breaker, victim).trust -= w;
    if t == Treaty::Alliance {
        let v = (rel(st, breaker, victim).base_relation - w).max(-100);
        rel_mut(st, breaker, victim).base_relation = v;
        rel_mut(st, victim, breaker).base_relation = v;
    }
    if w != 0 {
        rel_mut(st, breaker, victim).broken_treaty = t;
        let v = (rel(st, breaker, victim).relation - st.rng.one_to(20) as i32).max(-100);
        rel_mut(st, breaker, victim).relation = v;
        rel_mut(st, victim, breaker).relation = v;
    }
    for (a, b) in [(breaker, victim), (victim, breaker)] {
        let r = rel_mut(st, a, b);
        r.treaty = Treaty::None;
        r.attack_bounty = None;
        set_moods(r, -200);
    }
}

/// 1oom `game_diplo_break_trade`.
pub fn break_trade(st: &mut State, pi: EmpireId, pi2: EmpireId) {
    for (a, b) in [(pi, pi2), (pi2, pi)] {
        let r = rel_mut(st, a, b);
        r.trade_bc = 0;
        r.trade_established = 0;
        r.trade_percent = 0;
        r.attack_bounty = None;
    }
}

/// 1oom `game_diplo_start_war`: `pi` and `pi2` are at war.
pub fn start_war(st: &mut State, pi: EmpireId, pi2: EmpireId) {
    if is_human(st, pi) && rel(st, pi, pi2).have_met == 0 {
        rel_mut(st, pi, pi2).have_met = 1;
    }
    if is_human(st, pi2) && rel(st, pi2, pi).have_met == 0 {
        rel_mut(st, pi2, pi).have_met = 1;
    }
    if rel(st, pi, pi2).treaty >= Treaty::War {
        return;
    }
    break_trade(st, pi, pi2);
    break_treaty(st, pi, pi2);
    let v = -75 - st.rng.one_to(25) as i32;
    for (a, b) in [(pi, pi2), (pi2, pi)] {
        let r = rel_mut(st, a, b);
        r.base_relation -= 5;
        r.relation = v;
        r.treaty = Treaty::War;
        set_moods(r, -200);
        r.mood_peace = -130;
    }
}

/// 1oom `game_diplo_annoy`.
pub fn annoy(st: &mut State, p1: EmpireId, p2: EmpireId, n: i32) {
    let r = rel_mut(st, p1, p2);
    let n = n * 10;
    r.mood_trade -= n;
    r.mood_treaty -= n;
    r.mood_tech -= n;
    r.mood_peace -= n;
}

pub fn set_treaty(st: &mut State, p1: EmpireId, p2: EmpireId, t: Treaty) {
    rel_mut(st, p1, p2).treaty = t;
    rel_mut(st, p2, p1).treaty = t;
}

/// 1oom `game_diplo_set_trade`.
pub fn set_trade(st: &mut State, p1: EmpireId, p2: EmpireId, bc: i32) {
    if bc == 0 {
        return;
    }
    let r = rel(st, p1, p2);
    let v = if r.trade_established < bc {
        (r.trade_percent * r.trade_established - bc * 30) / (r.trade_established + bc)
    } else {
        r.trade_percent
    }
    .min(100);
    for (a, b) in [(p1, p2), (p2, p1)] {
        let r = rel_mut(st, a, b);
        r.trade_bc = bc;
        r.trade_established = bc;
        r.trade_percent = v;
    }
}

/// 1oom `game_diplo_stop_war`.
pub fn stop_war(st: &mut State, p1: EmpireId, p2: EmpireId) {
    set_treaty(st, p1, p2, Treaty::None);
    let v = (rel(st, p1, p2).relation + 40).min(100);
    rel_mut(st, p1, p2).relation = v;
    rel_mut(st, p2, p1).relation = v;
    if is_human(st, p1) {
        let c = st.rng.below(6) as i32 + 8;
        rel_mut(st, p1, p2).ceasefire = c;
    }
    if is_human(st, p2) {
        let c = st.rng.below(6) as i32 + 8;
        rel_mut(st, p2, p1).ceasefire = c;
    }
}

/// 1oom `game_diplo_esp_frame`.
pub fn esp_frame(st: &mut State, framed: EmpireId, victim: EmpireId) {
    let v = -(st.rng.one_to(12) as i32 + st.rng.one_to(12) as i32);
    act(st, v, framed, victim, msg::SPY_FRAMED, 0, 0);
}

/// 1oom `game_diplo_limit_mood_treaty`.
pub fn limit_mood_treaty(st: &mut State) {
    for e in st.empires.iter_mut() {
        for r in e.rel.iter_mut() {
            r.mood_treaty = r.mood_treaty.clamp(-200, 120);
        }
    }
}

/// 1oom `game_diplo_mood_relax`.
pub fn mood_relax(st: &mut State) {
    for (a, e) in st.empires.iter_mut().enumerate() {
        for (b, r) in e.rel.iter_mut().enumerate() {
            if a == b {
                continue;
            }
            for m in [
                &mut r.mood_treaty,
                &mut r.mood_trade,
                &mut r.mood_tech,
                &mut r.mood_peace,
            ] {
                if *m < 100 {
                    *m += 10;
                }
            }
        }
    }
}

/// 1oom `game_diplo_get_mood`: the worst of the four moods.
pub fn mood(st: &State, p1: EmpireId, p2: EmpireId) -> i32 {
    if st.council.final_war {
        return 0;
    }
    let r = rel(st, p1, p2);
    r.mood_treaty
        .min(r.mood_trade)
        .min(r.mood_tech)
        .min(r.mood_peace)
}

/// 1oom `game_diplo_is_gone`: the computer refuses to talk.
pub fn is_gone(st: &State, ai: EmpireId, pi: EmpireId) -> bool {
    if is_human(st, pi) {
        return false;
    }
    let vr = mood(st, ai, pi);
    let v = rel(st, ai, pi).trust + vr + RELDIFF[st.empires[pi as usize].trait1 as usize];
    v <= -100 || vr <= -100
}

/// 1oom `game_diplo_battle_finish`: how a battle or bombing changes relations.
#[allow(clippy::too_many_arguments)]
pub fn battle_finish(
    st: &mut State,
    def: EmpireId,
    att: EmpireId,
    popdiff: i32,
    app_def: i32,
    biodamage: i32,
    app_att: i32,
    planet: PlanetId,
) {
    let claim = st.planets[planet].claim;
    let (side_z, side_ai, app_ai) = if claim.is_none() || (claim != Some(att) && claim != Some(def))
    {
        if !is_human(st, att) {
            (def, att, app_att)
        } else {
            (att, def, app_def)
        }
    } else if claim == Some(att) {
        (def, att, app_att)
    } else {
        (att, def, app_def)
    };
    let n = st.empires.len() as u8;
    if biodamage > 0 {
        for i in 0..n {
            if i != side_z && !is_human(st, i) {
                act(st, -10, side_z, i, msg::BIO_WEAPONS, planet, 0);
            }
        }
    }
    let popdiff = popdiff * 5;
    let app_ai = app_ai.min(50);
    let biodamage = biodamage * 10;
    let offense = popdiff.max(app_ai).max(biodamage);
    if !is_human(st, side_ai) {
        let extra = if biodamage > 0 {
            msg::BIO_WEAPONS
        } else if popdiff > 0 {
            msg::KILLED_POP
        } else {
            msg::ATTACKED
        };
        act(st, -offense, side_z, side_ai, extra, planet, 0);
    }
    if offense >= 30 && is_human(st, side_z) {
        for i in 0..n {
            if i != side_z
                && i != side_ai
                && !is_human(st, i)
                && (rel(st, side_ai, i).treaty >= Treaty::War
                    || rel(st, side_z, i).bounty_collect == Some(side_ai))
            {
                if rel(st, side_z, i).attack_bounty == Some(side_ai) {
                    rel_mut(st, side_z, i).bounty_collect = Some(i);
                } else if st.colonies(side_ai).next().is_none() {
                    act(st, offense, side_z, i, 3, planet, 0);
                }
            }
        }
    }
}

// --------------------------------------------------------------------- turn steps

/// 1oom `game_turn_countdown_ceasefire`.
pub fn countdown_ceasefire(st: &mut State) {
    for e in st.empires.iter_mut() {
        for r in e.rel.iter_mut() {
            if r.ceasefire > 0 {
                r.ceasefire -= 1;
            }
        }
    }
}

/// 1oom `game_turn_update_mood_blunder`.
pub fn update_mood_blunder(st: &mut State) {
    let n = st.empires.len();
    for i in 0..n {
        for j in 1..n {
            if i == j {
                continue;
            }
            let v = st.empires[i].rel[j].diplo_type;
            let r = &mut st.empires[i].rel[j];
            match v {
                4..=11 => r.blunder = v,
                42..=49 => r.blunder = v - 30,
                50..=57 => r.blunder = v - 46,
                _ => {}
            }
            if st.empires[i].rel[j].treaty != Treaty::FinalWar {
                for k in 0..4 {
                    let mut v = match k {
                        0 => st.empires[i].rel[j].mood_treaty,
                        1 => st.empires[i].rel[j].mood_trade,
                        2 => st.empires[i].rel[j].mood_tech,
                        _ => st.empires[i].rel[j].mood_peace,
                    };
                    if v < 0 {
                        v += st.rng.one_to(5) as i32;
                    }
                    if v < 50 {
                        v += st.rng.one_to(5) as i32;
                    }
                    let v = if k == 3 { v.clamp(-200, 200) } else { v };
                    let r = &mut st.empires[i].rel[j];
                    match k {
                        0 => r.mood_treaty = v,
                        1 => r.mood_trade = v,
                        2 => r.mood_tech = v,
                        _ => r.mood_peace = v,
                    }
                }
            }
        }
    }
    for e in st.empires.iter_mut() {
        for r in e.rel.iter_mut() {
            r.diplo_val = 0;
            r.diplo_type = 0;
            if r.bounty_collect.is_some() {
                r.attack_bounty = None;
                r.bounty_collect = None;
            }
        }
    }
}

/// 1oom `game_turn_update_trade`: trade routes grow more profitable over time.
pub fn update_trade(st: &mut State) {
    let n = st.empires.len();
    for i in 0..n {
        for j in i..n {
            let bc = st.empires[i].rel[j].trade_bc;
            if bc == 0 {
                continue;
            }
            if !st.empires[i].rel[j].contact {
                let r = &mut st.empires[i].rel[j];
                r.trade_bc = 0;
                r.trade_percent = 0;
                r.trade_established = 0;
                continue;
            }
            let v = ((st.rng.one_to(200) as i32 + st.empires[i].rel[j].relation + 25) / 60).max(0);
            let a = &mut st.empires[i].rel[j];
            a.trade_percent = (a.trade_percent + v).min(100);
            let b = &mut st.empires[j].rel[i];
            b.trade_percent = (b.trade_percent + v).min(100);
        }
    }
}

/// Fleet strength statistic, 0..100 (1oom `game_stat_fleet`).
pub fn stat_fleet(st: &State, e: EmpireId) -> i32 {
    const HULL_W: [i64; 4] = [1, 5, 25, 125];
    let totals = crate::turn::ship_totals(st, e);
    let mut sum: i64 = st.empires[e as usize]
        .designs
        .iter()
        .enumerate()
        .map(|(d, sd)| totals[d] as i64 * HULL_W[sd.hull.min(3) as usize])
        .sum();
    sum = sum.min(0x31_9750);
    let (mut v4, mut va, mut ve) = (0i64, 125i64, 0i64);
    while sum > va {
        sum -= va;
        v4 += 1;
        ve = va;
        va <<= 1;
    }
    ((sum * 10) / (va - ve) + v4 * 10).clamp(0, 100) as i32
}

/// 1oom `game_turn_diplo_adjust`: treaties and trade improve relations, strong fleets,
/// big empires and old grudges change them.
pub fn diplo_adjust(st: &mut State) {
    let n = st.empires.len() as u8;
    for i in 0..n {
        for j in i + 1..n {
            if st.rng.below(2) == 0 {
                match rel(st, i, j).treaty {
                    Treaty::NonAggression => {
                        let v = st.rng.one_to(3) as i32;
                        act(st, v, i, j, 0, 0, 0);
                    }
                    Treaty::Alliance => {
                        let v = st.rng.one_to(6) as i32;
                        act(st, v, i, j, 0, 0, 0);
                    }
                    _ => {}
                }
                if rel(st, i, j).trade_bc != 0 {
                    let v = st.rng.one_to(3) as i32;
                    act(st, v, i, j, 2, 0, 0);
                }
            }
        }
    }
    for i in 0..n {
        if is_human(st, i) {
            continue;
        }
        let s_ai = stat_fleet(st, i);
        for j in 0..n {
            if !is_human(st, j) {
                continue;
            }
            let s_h = stat_fleet(st, j);
            if s_ai < s_h
                && s_ai > 0
                && st.rng.one_to(100) as i32 >= s_h * 50 / s_ai
                && st.rng.below(20) == 0
            {
                act(st, -10, j, i, msg::FLEET_THREAT, 0, 0);
            }
        }
    }
    if st.year % 2 == 0 {
        let mut big = vec![0i32; n as usize];
        for p in &st.planets {
            if let Some(o) = p.owner {
                if p.prod >= 100 {
                    big[o as usize] += 1;
                }
            }
        }
        let gscale = (st.galaxy_size as i32 + 1) * 3;
        let gdiv = st.galaxy_size as i32 + 6 - st.difficulty as i32;
        for i in 0..n {
            if gscale < big[i as usize] && st.rng.below(4) == 0 {
                let mut v = st.rng.one_to(4) as i32 * ((-(big[i as usize] - gscale)) / gdiv.max(1));
                v = (v / 3) * 4;
                if is_human(st, i) {
                    v /= 2;
                }
                for j in 0..n {
                    if !is_human(st, j)
                        && rel(st, i, j).contact
                        && rel(st, i, j).treaty == Treaty::Alliance
                    {
                        act(st, v, i, j, msg::EXPANSION, 0, 0);
                    }
                }
            }
        }
    }
    for i in 0..n as usize {
        if !st.empires[i].ai {
            continue;
        }
        for j in 0..n as usize {
            if i == j || st.empires[i].rel[j].diplo_val != 0 {
                continue;
            }
            let (r1, r2) = (
                st.empires[i].rel[j].relation,
                st.empires[i].rel[j].base_relation,
            );
            let v = if r1.abs() > st.rng.one_to(105) as i32 {
                st.rng.below(2) as i32
            } else {
                0
            };
            let r1 = if r1 < r2 {
                (r1 + v).min(r2)
            } else {
                (r1 - v).max(r2)
            };
            st.empires[i].rel[j].relation = r1.clamp(-100, 100);
        }
    }
    for (i, e) in st.empires.iter_mut().enumerate() {
        e.rel[i].relation = 0;
    }
}

/// 1oom `game_update_empire_contact` and `game_turn_update_have_met`: empires are in
/// contact when their colonies are within fuel range of each other.
pub fn update_contact(st: &mut State) {
    let n = st.empires.len();
    for a in 0..n {
        for b in a + 1..n {
            let frange = st.empires[a]
                .abilities()
                .fuel_range
                .max(st.empires[b].abilities().fuel_range);
            let mut mind = i32::MAX;
            for (i, _) in st.colonies(a as u8) {
                for (j, _) in st.colonies(b as u8) {
                    mind = mind.min(st.parsecs(i, j));
                }
            }
            let c = mind <= frange;
            st.empires[a].rel[b].contact = c;
            st.empires[b].rel[a].contact = c;
        }
    }
    for i in 0..n {
        if st.empires[i].ai {
            continue;
        }
        for j in 0..n {
            if i != j && st.empires[i].rel[j].have_met == 0 && st.empires[i].rel[j].contact {
                st.empires[i].rel[j].have_met = 1;
                st.empires[i].rel[j].treaty = Treaty::None;
                st.empires[j].rel[i].treaty = Treaty::None;
            }
        }
    }
}

/// 1oom `game_turn_contact_broken`: losing contact ends trade and spying.
pub fn contact_broken(st: &mut State, before: &[Vec<bool>]) {
    let n = st.empires.len();
    for i in 0..n {
        if !st.empires[i].alive {
            continue;
        }
        for j in 0..n {
            if i != j && st.empires[j].alive && !st.empires[i].rel[j].contact && before[i][j] {
                for (a, b) in [(i, j), (j, i)] {
                    let r = &mut st.empires[a].rel[b];
                    r.mood_trade = 0;
                    r.trade_bc = 0;
                    r.trade_percent = 0;
                    r.spymode = SpyMode::Hide;
                    r.contact_broken = true;
                }
            }
        }
    }
}

/// 1oom `game_turn_update_final_war`: after a refused council, everyone against the rebels.
pub fn update_final_war(st: &mut State) {
    if !st.council.final_war {
        return;
    }
    crate::tech::final_war_share(st);
    let n = st.empires.len();
    for a in 0..n {
        for b in a + 1..n {
            let same = st.council.refuse[a] == st.council.refuse[b];
            let (t, r) = if same {
                (Treaty::Alliance, 100)
            } else {
                (Treaty::FinalWar, -100)
            };
            for (x, y) in [(a, b), (b, a)] {
                st.empires[x].rel[y].treaty = t;
                st.empires[x].rel[y].relation = r;
            }
        }
    }
}

/// Starting relations, personalities and objectives (1oom `game_generate_relation_etc`,
/// trait rolls of `game_generate_race_banner`).
pub fn init(st: &mut State) {
    let n = st.empires.len();
    for i in 0..n {
        if st.empires[i].ai {
            let r = st.empires[i].race.index();
            st.empires[i].trait2 = TRAIT2_TBL[r][st.rng.below(12) as usize];
        }
    }
    for i in 0..n {
        let r = st.empires[i].race.index();
        st.empires[i].trait1 = TRAIT1_TBL[r][st.rng.below(10) as usize];
    }
    for i in 0..n {
        st.empires[i].rel = vec![Relation::default(); n];
        for j in 0..n {
            let v = RACE_RELATION[st.empires[i].race.index()][st.empires[j].race.index()] * 12;
            let r = &mut st.empires[i].rel[j];
            r.relation = v;
            r.base_relation = v;
            r.spy_report = [1; NUM_FIELDS];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Game, NewGameOptions};

    fn game() -> Game {
        Game::new(&NewGameOptions {
            seed: 4,
            players: 3,
            ..Default::default()
        })
        .unwrap()
    }

    #[test]
    fn act_moves_ai_feelings_only() {
        let mut g = game();
        let st = g.state_mut();
        let before = rel(st, 1, 0).relation;
        act(st, 20, 0, 1, 0, 0, 0);
        assert!(rel(st, 1, 0).relation > before);
        assert_eq!(rel(st, 1, 0).relation, rel(st, 0, 1).relation);
        let h = rel(st, 0, 1).relation;
        act(st, -50, 1, 0, msg::ATTACKED, 0, 0);
        assert_eq!(
            rel(st, 0, 1).relation,
            h,
            "the human has no feelings to hurt"
        );
    }

    #[test]
    fn war_and_peace() {
        let mut g = game();
        let st = g.state_mut();
        set_treaty(st, 0, 1, Treaty::Alliance);
        start_war(st, 1, 0);
        assert_eq!(treaty(st, 0, 1), Treaty::War);
        assert!(rel(st, 0, 1).relation <= -75);
        stop_war(st, 0, 1);
        assert_eq!(treaty(st, 1, 0), Treaty::None);
        assert!(rel(st, 0, 1).ceasefire > 0, "a human gets a ceasefire");
    }

    #[test]
    fn breaking_a_treaty_costs_trust() {
        let mut g = game();
        let st = g.state_mut();
        set_treaty(st, 0, 2, Treaty::NonAggression);
        break_treaty(st, 0, 2);
        assert_eq!(treaty(st, 0, 2), Treaty::None);
        assert!(rel(st, 0, 2).trust < 0);
        assert_eq!(rel(st, 0, 2).broken_treaty, Treaty::NonAggression);
        assert_eq!(mood(st, 0, 2), -200);
    }

    #[test]
    fn trade_starts_at_a_loss() {
        let mut g = game();
        let st = g.state_mut();
        set_trade(st, 0, 1, 100);
        assert_eq!(rel(st, 1, 0).trade_bc, 100);
        assert!(rel(st, 0, 1).trade_percent < 0);
    }
}
