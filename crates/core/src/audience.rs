//! Audiences between a human and a computer empire (1oom `game_audience.c` and the
//! `aud_*` answers of `game_ai_classic.c`).
//!
//! MOO1 runs an audience as a blocking dialogue. Here it is split in steps: computer
//! messages and proposals wait in `State::pending.messages` until the player answers, and
//! the player's own proposals (`Talk`) get an immediate `Reply`, possibly asking for a gift
//! ("sweetener") that the player accepts or declines with a second `Talk`.

use serde::{Deserialize, Serialize};

use crate::diplo::{self, msg, rel, rel_mut, TechRef, Treaty, RELDIFF};
use crate::spy;
use crate::state::*;
use crate::tech::{self, Source};
use crate::types::*;

/// Values a message refers to (1oom audience placeholders).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MsgParams {
    pub planet: Option<PlanetId>,
    pub field: Option<u8>,
    /// Another empire the message is about (war target, bounty, ally to drop).
    pub empire: Option<EmpireId>,
    pub bc: i32,
    pub tech: Option<TechRef>,
    /// Gift offered with a proposal.
    pub offer_bc: i32,
    pub offer_tech: Option<TechRef>,
    /// Techs to choose from (tech trade offers).
    pub techs: Vec<TechRef>,
}

/// A message or proposal from a computer empire to a human.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Incoming {
    pub to: EmpireId,
    pub from: EmpireId,
    /// Message type (`diplo::msg`).
    pub kind: u8,
    pub params: MsgParams,
    /// The proposal to make if this one (dropping an alliance) is accepted.
    #[serde(default)]
    pub then: Option<u8>,
    pub needs_answer: bool,
}

/// What a human proposes or does in an audience.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Talk {
    ProposeNap,
    ProposeAlliance,
    ProposePeace,
    /// Ask them to declare war on `target`.
    DeclareWarOn {
        target: EmpireId,
    },
    /// Ask them to break their alliance with `target`.
    BreakAllianceWith {
        target: EmpireId,
    },
    ProposeTrade {
        bc: i32,
    },
    BreakTreaty,
    BreakTrade,
    Threaten,
    TributeBc {
        bc: i32,
    },
    /// Lists the techs the player could give as tribute.
    TributeTechs,
    TributeTech {
        field: u8,
        tech: u8,
    },
    /// Lists what they would trade.
    TechOffers,
    ExchangeTech {
        want: TechRef,
        give: TechRef,
    },
    /// Give what they asked for to seal the last proposal.
    AcceptSweetener,
    DeclineSweetener,
}

/// One of their techs, and the techs of ours they would take for it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechDeal {
    pub want: TechRef,
    pub give: Vec<TechRef>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Reply {
    /// Message type of the answer (`diplo::msg`), 0 = no message.
    pub kind: u8,
    pub params: MsgParams,
    pub accepted: bool,
    /// They want this gift before agreeing: answer with `AcceptSweetener` or `DeclineSweetener`.
    pub sweetener: Option<Sweetener>,
    pub tech_deals: Vec<TechDeal>,
    pub techs: Vec<TechRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sweetener {
    pub bc: i32,
    pub tech: Option<TechRef>,
}

/// A proposal waiting for the player's sweetener, or the tech lists of the last request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deal {
    pub human: EmpireId,
    pub ai: EmpireId,
    pub talk: Talk,
    pub sweetener: Option<Sweetener>,
    pub kind: u8,
    pub tech_deals: Vec<TechDeal>,
    pub techs: Vec<TechRef>,
}

// ------------------------------------------------------------- AI answers (classic)

/// 1oom `game_ai_classic_aud_check_mood`: 0 no, 1-2 maybe (with a gift), 3 yes.
fn check_mood(st: &mut State, ph: EmpireId, pa: EmpireId, a0: i32, which: u8) -> i32 {
    let r = rel(st, ph, pa).clone();
    let mood = match which {
        1 => r.mood_trade,
        2 => r.mood_peace,
        3 => r.mood_tech,
        _ => r.mood_treaty,
    };
    let human_bonus = if st.empires[ph as usize].race == Race::Human {
        50
    } else {
        0
    };
    let mut v = mood
        + r.trust
        + r.relation
        + human_bonus
        + RELDIFF[st.empires[pa as usize].trait1 as usize];
    v += st.rng.one_to(100) as i32;
    v -= a0;
    if r.treaty == Treaty::Alliance {
        v += 40;
    }
    diplo::annoy(st, ph, pa, 1);
    let d = match which {
        2 => st.rng.one_to(50) as i32 + 50,
        3 => st.rng.one_to(50) as i32 + 20,
        _ => st.rng.one_to(30) as i32 + 20,
    };
    let r = rel_mut(st, ph, pa);
    match which {
        1 => r.mood_trade -= d,
        2 => r.mood_peace -= d,
        3 => r.mood_tech -= d,
        _ => r.mood_treaty -= d,
    }
    if v < -75 {
        0
    } else if v < -50 {
        1
    } else if v < 0 {
        2
    } else {
        3
    }
}

/// 1oom `game_ai_classic_aud_start_human`: greeting, or refusal to talk.
pub fn greeting(st: &State, ph: EmpireId, pa: EmpireId) -> (u8, bool) {
    let r = rel(st, ph, pa);
    let v = r.trust + diplo::mood(st, ph, pa) + RELDIFF[st.empires[pa as usize].trait1 as usize];
    if v < -100 {
        (
            if r.treaty >= Treaty::War {
                msg::REFUSE_AT_WAR
            } else {
                msg::REFUSE_HOSTILE
            },
            false,
        )
    } else if v + r.relation > -50 {
        (msg::GREETING, true)
    } else {
        (msg::GREETING_COLD, true)
    }
}

/// 1oom `game_ai_classic_aud_later`.
pub fn later(st: &State, ph: EmpireId, pa: EmpireId) -> bool {
    diplo::mood(st, ph, pa) < -100
}

/// 1oom `game_ai_classic_aud_sweeten`: what they ask for to agree.
fn sweeten(st: &mut State, ph: EmpireId, pa: EmpireId, si: i32) -> Option<Sweetener> {
    let mut bc = ((st.rng.one_to(8) as i32 + st.rng.one_to(8) as i32) * year_n(st) / 25) * 25;
    if bc > st.empires[ph as usize].reserve {
        bc = 0;
    }
    let year = year_n(st);
    let tech = spy::pick_techs(st, pa, ph, year, 1)
        .first()
        .map(|o| (o.field, o.tech));
    if bc == 0 && tech.is_none() {
        return None;
    }
    if (tech.is_some() && si == 1) || bc == 0 {
        Some(Sweetener { bc: 0, tech })
    } else {
        Some(Sweetener { bc, tech: None })
    }
}

/// Game year counted from 1 (1oom `g->year`).
fn year_n(st: &State) -> i32 {
    (st.year as i32 - st.rules.start_year as i32 + 1).max(1)
}

/// 1oom `game_ai_classic_aud_get_dtype`: the answer to a proposal.
fn answer_kind(
    st: &mut State,
    ph: EmpireId,
    pa: EmpireId,
    kind: u8,
    si: i32,
) -> (u8, Option<TechRef>) {
    if !(kind == 31 || (33..=41).contains(&kind) || (62..=68).contains(&kind)) {
        return (kind, None);
    }
    match si {
        0..=2 => {
            let r = rel(st, ph, pa).clone();
            if r.blunder != 0 && st.rng.below(2) == 0 {
                rel_mut(st, ph, pa).blunder = 0;
                (r.blunder + 30, None)
            } else if r.broken_treaty != Treaty::None && st.rng.below(4) == 0 {
                rel_mut(st, ph, pa).broken_treaty = Treaty::None;
                (msg::REFUSE_BROKEN_TREATY, None)
            } else {
                (msg::REFUSE, None)
            }
        }
        _ => {
            if st.rng.below(4) == 0 {
                if let Some(t) = rel(st, ph, pa).tribute {
                    rel_mut(st, ph, pa).tribute = None;
                    return (msg::ACCEPT_TRIBUTE_TECH, Some(t));
                }
            }
            (kind, None)
        }
    }
}

/// 1oom `game_ai_classic_aud_tech_scale`.
fn tech_scale(st: &mut State, ph: EmpireId, pa: EmpireId) -> i32 {
    let r = rel(st, ph, pa).clone();
    let mut di = r.mood_tech;
    if di > 0 {
        di /= 5;
    }
    di = di.min(30);
    if r.treaty == Treaty::Alliance {
        di += 25;
    }
    let human = if st.empires[ph as usize].race == Race::Human {
        50
    } else {
        0
    };
    let mut v = r.trust
        + r.relation / 2
        + human
        + RELDIFF[st.empires[pa as usize].trait1 as usize] * 2
        + di
        + st.rng.one_to(100) as i32
        - 125;
    v = if v < 0 {
        v.abs() + 100
    } else {
        20000 / (v + 200)
    };
    (v / 4).max(75)
}

// ------------------------------------------------------------- human proposals

fn treaty_answer(st: &mut State, ph: EmpireId, pa: EmpireId, talk: &Talk) -> i32 {
    let r = rel(st, ph, pa).clone();
    match *talk {
        Talk::ProposeNap => {
            if r.relation > 10 {
                check_mood(st, ph, pa, 50, 0)
            } else {
                0
            }
        }
        Talk::ProposeAlliance => {
            if r.relation > 50 {
                check_mood(st, ph, pa, 125, 0)
            } else {
                0
            }
        }
        Talk::ProposePeace => check_mood(st, ph, pa, 60, 2),
        Talk::DeclareWarOn { target } => {
            if r.treaty == Treaty::Alliance && rel(st, ph, target).treaty == Treaty::War {
                if st.rng.below(8) == 0 {
                    2
                } else {
                    3
                }
            } else {
                let t = rel(st, pa, target);
                let v = t.relation + 150 + t.treaty as i32 * 30;
                check_mood(st, ph, pa, v, 0)
            }
        }
        Talk::BreakAllianceWith { target } => {
            if r.relation > 24 {
                let v = rel(st, pa, target).relation + 175;
                check_mood(st, ph, pa, v, 0)
            } else {
                0
            }
        }
        Talk::ProposeTrade { .. } => check_mood(st, ph, pa, 50, 1),
        _ => 0,
    }
}

fn treaty_kind(talk: &Talk) -> u8 {
    match talk {
        Talk::ProposeNap => msg::ACCEPT_NAP,
        Talk::ProposeAlliance => msg::ACCEPT_ALLIANCE,
        Talk::ProposePeace => msg::ACCEPT_PEACE,
        Talk::DeclareWarOn { .. } => msg::ACCEPT_WAR,
        Talk::BreakAllianceWith { .. } => msg::ACCEPT_BREAK,
        Talk::ProposeTrade { .. } => msg::ACCEPT_TRADE,
        _ => 0,
    }
}

fn execute(st: &mut State, ph: EmpireId, pa: EmpireId, talk: &Talk) {
    match *talk {
        Talk::ProposeNap => diplo::set_treaty(st, ph, pa, Treaty::NonAggression),
        Talk::ProposeAlliance => diplo::set_treaty(st, ph, pa, Treaty::Alliance),
        Talk::ProposePeace => diplo::stop_war(st, ph, pa),
        Talk::DeclareWarOn { target } => diplo::start_war(st, pa, target),
        Talk::BreakAllianceWith { target } => diplo::break_treaty(st, pa, target),
        Talk::ProposeTrade { bc } => diplo::set_trade(st, ph, pa, bc),
        _ => {}
    }
}

fn params_for(talk: &Talk) -> MsgParams {
    match *talk {
        Talk::DeclareWarOn { target } | Talk::BreakAllianceWith { target } => MsgParams {
            empire: Some(target),
            ..Default::default()
        },
        Talk::ProposeTrade { bc } => MsgParams {
            bc,
            ..Default::default()
        },
        _ => MsgParams::default(),
    }
}

/// Trade amounts the player can propose (1oom `audience_menu_main`).
pub fn trade_options(st: &State, ph: EmpireId, pa: EmpireId) -> Vec<i32> {
    let cur = rel(st, ph, pa).trade_bc;
    let prod = (st.empires[ph as usize]
        .total_prod
        .min(st.empires[pa as usize].total_prod)
        / 4)
    .min(32000);
    let want = (prod / 25) * 25 - cur;
    if want <= 0 {
        Vec::new()
    } else if want < 125 {
        (1..=want / 25).map(|k| cur + 25 * k).collect()
    } else {
        vec![
            want / 5 + cur,
            want * 2 / 5 / 25 * 25 + cur,
            want * 3 / 5 / 25 * 25 + cur,
            want * 4 / 5 / 25 * 25 + cur,
            want + cur,
        ]
    }
}

/// Tribute amounts (1oom `audience_menu_tribute`).
pub fn tribute_options(st: &State, ph: EmpireId) -> Vec<i32> {
    let reserve = (st.empires[ph as usize].reserve.min(32000) / 25) * 25;
    if reserve < 100 {
        (1..=reserve / 25).map(|k| k * 25).collect()
    } else {
        vec![
            reserve / 4 / 25 * 25,
            reserve / 2 / 25 * 25,
            3 * reserve / 4 / 25 * 25,
            reserve,
        ]
    }
}

/// Which audience menus and proposals are possible now (deterministic part of 1oom's menus).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AudienceOptions {
    pub greeting: u8,
    pub talks: bool,
    pub later: bool,
    pub nap: bool,
    pub alliance: bool,
    pub peace: bool,
    pub war_on: Vec<EmpireId>,
    pub break_alliance_with: Vec<EmpireId>,
    pub trade: Vec<i32>,
    pub tribute: Vec<i32>,
    pub break_treaty: bool,
    pub break_trade: bool,
    pub threaten: bool,
    pub exchange: bool,
}

pub fn options(st: &State, ph: EmpireId, pa: EmpireId) -> AudienceOptions {
    let (greeting, talks) = greeting(st, ph, pa);
    let t = rel(st, ph, pa).treaty;
    let n = st.empires.len() as u8;
    let war = t >= Treaty::War;
    AudienceOptions {
        greeting,
        talks,
        later: later(st, ph, pa),
        nap: t == Treaty::None,
        alliance: t < Treaty::Alliance,
        peace: war,
        war_on: if war {
            Vec::new()
        } else {
            (0..n)
                .filter(|&i| {
                    i != ph
                        && i != pa
                        && st.empires[i as usize].alive
                        && (rel(st, ph, i).have_met > 0 || rel(st, ph, i).contact)
                        && rel(st, pa, i).treaty < Treaty::War
                })
                .collect()
        },
        break_alliance_with: (0..n)
            .filter(|&i| {
                i != ph
                    && i != pa
                    && rel(st, pa, i).contact
                    && rel(st, pa, i).treaty == Treaty::Alliance
            })
            .collect(),
        trade: if war {
            Vec::new()
        } else {
            trade_options(st, ph, pa)
        },
        tribute: tribute_options(st, ph),
        break_treaty: matches!(t, Treaty::NonAggression | Treaty::Alliance),
        break_trade: rel(st, ph, pa).trade_bc != 0,
        threaten: !war,
        exchange: !war,
    }
}

fn reply(kind: u8, params: MsgParams, accepted: bool) -> Reply {
    Reply {
        kind,
        params,
        accepted,
        ..Default::default()
    }
}

/// The player's move in an audience with `pa`.
pub fn talk(st: &mut State, ph: EmpireId, pa: EmpireId, t: &Talk) -> Result<Reply, &'static str> {
    if ph == pa
        || pa as usize >= st.empires.len()
        || !st.empires[pa as usize].ai
        || !st.empires[pa as usize].alive
    {
        return Err("invalid_empire");
    }
    if !rel(st, ph, pa).contact {
        return Err("no_contact");
    }
    let opts = options(st, ph, pa);
    if !opts.talks {
        return Ok(reply(opts.greeting, MsgParams::default(), false));
    }
    let deal = st
        .pending
        .deals
        .iter()
        .position(|d| d.human == ph && d.ai == pa)
        .map(|i| st.pending.deals.remove(i));
    match *t {
        Talk::AcceptSweetener | Talk::DeclineSweetener => {
            let Some(deal) = deal else {
                return Err("no_deal");
            };
            let Some(sw) = deal.sweetener else {
                return Err("no_deal");
            };
            if *t == Talk::DeclineSweetener {
                return Ok(reply(0, MsgParams::default(), false));
            }
            if let Some((f, n)) = sw.tech {
                if !st.empires[ph as usize].tech.has(f as usize, n) {
                    return Err("invalid_tech");
                }
                tech::grant_from(st, pa, f as usize, n, Source::Trade, Some(ph));
            } else {
                if st.empires[ph as usize].reserve < sw.bc {
                    return Err("no_money");
                }
                st.empires[ph as usize].reserve -= sw.bc;
                st.empires[pa as usize].reserve += sw.bc;
            }
            execute(st, ph, pa, &deal.talk);
            if deal.talk == Talk::ProposePeace {
                diplo::annoy(st, ph, pa, 2);
            }
            let (k, tech) = answer_kind(st, ph, pa, deal.kind, 3);
            let mut p = params_for(&deal.talk);
            p.tech = tech;
            return Ok(reply(k, p, true));
        }
        Talk::ExchangeTech { want, give } => {
            let offered = deal.as_ref().is_some_and(|d| {
                d.tech_deals
                    .iter()
                    .any(|td| td.want == want && td.give.contains(&give))
            });
            if !offered {
                return Err("invalid_tech");
            }
            tech::grant_from(st, ph, want.0 as usize, want.1, Source::Trade, Some(pa));
            tech::grant_from(st, pa, give.0 as usize, give.1, Source::Trade, Some(ph));
            diplo::annoy(st, ph, pa, 3);
            let r = rel_mut(st, ph, pa);
            r.mood_tech = r.mood_tech.min(50);
            return Ok(reply(0, MsgParams::default(), true));
        }
        Talk::TributeTech { field, tech: n } => {
            let offered = deal.as_ref().is_some_and(|d| d.techs.contains(&(field, n)));
            if !offered {
                return Err("invalid_tech");
            }
            tech::grant_from(st, pa, field as usize, n, Source::Trade, Some(ph));
            tribute_tech(st, ph, pa, n);
            rel_mut(st, ph, pa).tribute = Some((field, n));
            return Ok(reply(
                msg::TRIBUTE_THANKS,
                MsgParams {
                    tech: Some((field, n)),
                    ..Default::default()
                },
                true,
            ));
        }
        _ => {}
    }
    if opts.later {
        return Ok(reply(msg::LATER, MsgParams::default(), false));
    }
    match t {
        Talk::ProposeNap
        | Talk::ProposeAlliance
        | Talk::ProposePeace
        | Talk::DeclareWarOn { .. }
        | Talk::BreakAllianceWith { .. }
        | Talk::ProposeTrade { .. } => {
            let allowed = match *t {
                Talk::ProposeNap => opts.nap,
                Talk::ProposeAlliance => opts.alliance,
                Talk::ProposePeace => opts.peace,
                Talk::DeclareWarOn { target } => opts.war_on.contains(&target),
                Talk::BreakAllianceWith { target } => opts.break_alliance_with.contains(&target),
                Talk::ProposeTrade { bc } => opts.trade.contains(&bc),
                _ => false,
            };
            if !allowed {
                return Err("not_possible");
            }
            if let Talk::ProposeTrade { .. } = t {
                diplo::annoy(st, ph, pa, 1);
                let d = st.rng.one_to(30) as i32;
                rel_mut(st, ph, pa).mood_trade -= d;
            }
            let kind = treaty_kind(t);
            let mut si = treaty_answer(st, ph, pa, t);
            if matches!(t, Talk::ProposeTrade { .. }) && si < 3 {
                si = 0;
            }
            if si == 1 || si == 2 {
                if let Some(sw) = sweeten(st, ph, pa, si) {
                    st.pending.deals.push(Deal {
                        human: ph,
                        ai: pa,
                        talk: t.clone(),
                        sweetener: Some(sw.clone()),
                        kind,
                        tech_deals: Vec::new(),
                        techs: Vec::new(),
                    });
                    if *t == Talk::ProposePeace {
                        diplo::annoy(st, ph, pa, 2);
                    }
                    return Ok(Reply {
                        sweetener: Some(sw),
                        params: params_for(t),
                        ..Default::default()
                    });
                }
            }
            if si == 3 {
                execute(st, ph, pa, t);
            }
            if *t == Talk::ProposePeace {
                diplo::annoy(st, ph, pa, 2);
            }
            let (k, tech) = answer_kind(st, ph, pa, kind, si);
            let mut p = params_for(t);
            p.tech = tech;
            Ok(reply(k, p, si == 3))
        }
        Talk::BreakTreaty | Talk::BreakTrade | Talk::Threaten => {
            let kind = match t {
                Talk::BreakTreaty => {
                    if !opts.break_treaty {
                        return Err("not_possible");
                    }
                    diplo::break_treaty(st, ph, pa);
                    msg::YOU_BROKE
                }
                Talk::BreakTrade => {
                    if !opts.break_trade {
                        return Err("not_possible");
                    }
                    diplo::break_trade(st, ph, pa);
                    msg::YOU_BROKE
                }
                _ => return Ok(threaten(st, ph, pa)),
            };
            diplo::annoy(st, ph, pa, 10);
            Ok(reply(kind, MsgParams::default(), true))
        }
        Talk::TributeBc { bc } => {
            if !opts.tribute.contains(bc) {
                return Err("not_possible");
            }
            let sel = opts.tribute.iter().position(|b| b == bc).unwrap_or(0) as i32;
            st.empires[ph as usize].reserve -= bc;
            st.empires[pa as usize].reserve += bc;
            tribute_bc(st, ph, pa, sel, *bc);
            Ok(reply(
                msg::TRIBUTE_THANKS,
                MsgParams {
                    bc: *bc,
                    ..Default::default()
                },
                true,
            ))
        }
        Talk::TributeTechs => {
            let techs: Vec<TechRef> = spy::pick_techs(st, pa, ph, 0, 0)
                .into_iter()
                .take(4)
                .map(|o| (o.field, o.tech))
                .collect();
            if techs.is_empty() {
                return Ok(reply(msg::NOTHING_TO_TRADE, MsgParams::default(), false));
            }
            st.pending.deals.push(Deal {
                human: ph,
                ai: pa,
                talk: t.clone(),
                sweetener: None,
                kind: 0,
                tech_deals: Vec::new(),
                techs: techs.clone(),
            });
            Ok(Reply {
                techs,
                ..Default::default()
            })
        }
        Talk::TechOffers => {
            if !opts.exchange {
                return Err("not_possible");
            }
            let deals = tech_offers(st, ph, pa);
            if deals.is_empty() {
                diplo::annoy(st, ph, pa, 3);
                let r = rel_mut(st, ph, pa);
                r.mood_tech = r.mood_tech.min(50);
                return Ok(reply(msg::NOTHING_TO_TRADE, MsgParams::default(), false));
            }
            st.pending.deals.push(Deal {
                human: ph,
                ai: pa,
                talk: t.clone(),
                sweetener: None,
                kind: 0,
                tech_deals: deals.clone(),
                techs: Vec::new(),
            });
            Ok(Reply {
                tech_deals: deals,
                ..Default::default()
            })
        }
        Talk::AcceptSweetener
        | Talk::DeclineSweetener
        | Talk::ExchangeTech { .. }
        | Talk::TributeTech { .. } => unreachable!(),
    }
}

/// 1oom `audience_menu_tech`: their techs the player could get, each with ours they take.
fn tech_offers(st: &mut State, ph: EmpireId, pa: EmpireId) -> Vec<TechDeal> {
    let v = tech_scale(st, ph, pa);
    let theirs = spy::pick_techs(st, ph, pa, 0, 1);
    let mut out = Vec::new();
    for o in theirs {
        let min = o.value * v / 100;
        let give: Vec<TechRef> = spy::pick_techs(st, pa, ph, min, 0)
            .into_iter()
            .take(4)
            .map(|x| (x.field, x.tech))
            .collect();
        if !give.is_empty() {
            out.push(TechDeal {
                want: (o.field, o.tech),
                give,
            });
        }
        if out.len() >= 5 {
            break;
        }
    }
    if !out.is_empty() {
        diplo::annoy(st, ph, pa, 1);
        let d = st.rng.one_to(50) as i32 + 20;
        rel_mut(st, ph, pa).mood_tech -= d;
    }
    out
}

/// 1oom `game_ai_classic_aud_threaten` (Classic+ fixes).
fn threaten(st: &mut State, ph: EmpireId, pa: EmpireId) -> Reply {
    let r = rel(st, ph, pa).clone();
    let t1 = st.empires[pa as usize].trait1 as usize;
    let mut v = st.rng.one_to(200) as i32 + r.mood_treaty / 2 + RELDIFF[t1] * 2;
    let ap = st.empires[pa as usize].total_prod;
    v += if ap > 0 {
        st.empires[ph as usize].total_prod * 100 / ap
    } else {
        100
    };
    let nr = (r.relation - st.rng.one_to(15) as i32).max(-100);
    rel_mut(st, ph, pa).relation = nr;
    rel_mut(st, pa, ph).relation = nr;
    rel_mut(st, ph, pa).mood_treaty = -120;
    let mut p = MsgParams::default();
    let kind = if v < 170 {
        if st.rng.one_to(15) as i32 - RELDIFF[t1] > st.rng.one_to(100) as i32 {
            diplo::start_war(st, ph, pa);
            msg::WAR_DECLARED
        } else {
            msg::THREAT_IGNORED
        }
    } else {
        rel_mut(st, pa, ph).spymode_next = diplo::SpyMode::Hide;
        rel_mut(st, pa, ph).spymode = diplo::SpyMode::Hide;
        let c = st.rng.one_to(15) as i32 + 5;
        rel_mut(st, ph, pa).ceasefire = c;
        let mut kind = msg::THREAT_HEEDED;
        if v >= 275 {
            if let Some(o) = spy::pick_techs(st, ph, pa, 0, 1).first().copied() {
                tech::grant_from(st, ph, o.field as usize, o.tech, Source::Trade, Some(pa));
                p.tech = Some((o.field, o.tech));
            }
        } else if v >= 200 {
            let bc = ((st.rng.one_to(8) as i32 + 2) * year_n(st) / 25) * 25;
            if bc != 0 {
                st.empires[ph as usize].reserve += bc;
                p.bc = bc;
                kind = msg::THREAT_TRIBUTE;
            }
        }
        kind
    };
    diplo::annoy(st, ph, pa, 10);
    reply(
        kind,
        p,
        kind != msg::WAR_DECLARED && kind != msg::THREAT_IGNORED,
    )
}

/// 1oom `game_ai_classic_aud_tribute_bc` (Classic+).
fn tribute_bc(st: &mut State, ph: EmpireId, pa: EmpireId, selected: i32, bc: i32) {
    let ap = st.empires[pa as usize].total_prod;
    let mut v = if ap != 0 { bc * 12 / ap } else { selected + 1 };
    if st.empires[ph as usize].race == Race::Human {
        v *= 2;
    }
    let mut nv = (rel(st, ph, pa).relation + v).min(100);
    let t = rel(st, ph, pa).treaty;
    if t >= Treaty::War {
        nv = nv.min(-25);
    }
    if t != Treaty::Alliance {
        nv = nv.min(65);
    }
    rel_mut(st, ph, pa).relation = nv;
    rel_mut(st, pa, ph).relation = nv;
}

/// 1oom `game_ai_classic_aud_tribute_tech` (Classic+).
fn tribute_tech(st: &mut State, ph: EmpireId, pa: EmpireId, tech: u8) {
    let r = rel(st, ph, pa).clone();
    let mut v = if r.relation < 0 {
        20
    } else {
        (100 - r.relation) / 10
    };
    v = ((st.rng.one_to(8) as i32 + st.rng.one_to(8) as i32 + tech as i32 / 4) * v) / 10;
    if st.empires[ph as usize].race == Race::Human {
        v *= 2;
    }
    let trust = st.rng.one_to(8) as i32 + 2;
    let mut nv = (r.relation + v).min(100);
    {
        let rr = rel_mut(st, ph, pa);
        rr.mood_peace = (rr.mood_peace + v).min(200);
        rr.trust = (rr.trust + trust).min(30);
    }
    if r.treaty >= Treaty::War {
        nv = nv.min(-25);
    }
    if r.treaty != Treaty::Alliance {
        nv = nv.min(70);
    }
    rel_mut(st, ph, pa).relation = nv;
    rel_mut(st, pa, ph).relation = nv;
}

// ------------------------------------------------------------- computer messages

/// 1oom `game_turn_audiences` and `game_audience`: turns the messages computer empires
/// want to deliver into `pending.messages`.
pub fn collect(st: &mut State) {
    diplo::limit_mood_treaty(st);
    let n = st.empires.len() as u8;
    for ph in 0..n {
        if st.empires[ph as usize].ai {
            continue;
        }
        for pa in 0..n {
            if !st.empires[pa as usize].ai {
                continue;
            }
            let kind = rel(st, ph, pa).diplo_type;
            if kind == 0 {
                continue;
            }
            rel_mut(st, ph, pa).diplo_type = 0;
            if kind == msg::ACCEPT_TRIBUTE_TECH || kind == msg::TRIBUTE_THANKS {
                continue;
            }
            deliver(st, ph, pa, kind);
            if kind == msg::TREATY_BROKEN {
                diplo::break_treaty(st, pa, ph);
            }
        }
    }
}

fn params_of(st: &State, ph: EmpireId, pa: EmpireId, kind: u8) -> MsgParams {
    let r = rel(st, ph, pa);
    let mut p = MsgParams {
        planet: (r.diplo_p1 < st.planets.len() && r.diplo_p1 != 0).then_some(r.diplo_p1),
        field: (0..NUM_FIELDS as i32)
            .contains(&r.diplo_p2)
            .then_some(r.diplo_p2 as u8),
        offer_bc: r.offer_bc,
        offer_tech: r.offer_tech,
        ..Default::default()
    };
    match kind {
        msg::PROPOSE_TRADE => p.bc = r.want_trade,
        msg::OFFER_BOUNTY | msg::PAYS_BOUNTY => {
            p.empire = r.attack_bounty;
            p.bc = r.attack_gift_bc;
            p.tech = r.attack_gift_tech;
        }
        msg::OFFER_TECH_TRADE => {
            p.tech = r.want_tech;
            p.techs = r.tech_trade.iter().copied().take(4).collect();
        }
        msg::JOIN_WAR | msg::ALLIANCE_ENDED | msg::THANKS_JOIN_WAR => p.empire = r.ally_attacker,
        msg::ASK_BREAK_ALLIANCE => p.empire = r.ask_break_treaty,
        msg::VOTED_FOR | msg::VOTED_AGAINST => p.empire = Some(r.diplo_p2.clamp(0, 5) as u8),
        _ => {}
    }
    p
}

fn is_proposal(kind: u8) -> bool {
    (24..=30).contains(&kind) || kind == msg::JOIN_WAR
}

/// 1oom `game_audience` for a computer-initiated audience.
fn deliver(st: &mut State, ph: EmpireId, pa: EmpireId, kind: u8) {
    let mut params = params_of(st, ph, pa, kind);
    if is_proposal(kind) || kind == msg::PAYS_BOUNTY || kind == msg::OFFER_BOUNTY {
        // 1oom game_audience_check_alliances: first ask to drop an alliance with an enemy.
        let n = st.empires.len() as u8;
        let enemy = (0..n).rev().find(|&i| {
            i != ph
                && i != pa
                && rel(st, pa, i).treaty >= Treaty::War
                && rel(st, ph, i).treaty == Treaty::Alliance
        });
        let then = Some(kind);
        if let Some(pf) = enemy {
            rel_mut(st, ph, pa).ask_break_treaty = Some(pf);
            st.pending.messages.push(Incoming {
                to: ph,
                from: pa,
                kind: msg::ASK_BREAK_ALLIANCE,
                params: MsgParams {
                    empire: Some(pf),
                    ..Default::default()
                },
                then,
                needs_answer: true,
            });
        } else {
            match kind {
                msg::PAYS_BOUNTY => {
                    let r = rel(st, ph, pa).clone();
                    st.empires[ph as usize].reserve += r.attack_gift_bc;
                    if let Some((f, t)) = r.attack_gift_tech {
                        tech::grant_from(st, ph, f as usize, t, Source::Trade, Some(pa));
                    }
                    rel_mut(st, ph, pa).bounty_collect = None;
                    st.pending.messages.push(Incoming {
                        to: ph,
                        from: pa,
                        kind,
                        params,
                        then: None,
                        needs_answer: false,
                    });
                }
                msg::OFFER_BOUNTY => st.pending.messages.push(Incoming {
                    to: ph,
                    from: pa,
                    kind,
                    params,
                    then: None,
                    needs_answer: false,
                }),
                _ => st.pending.messages.push(Incoming {
                    to: ph,
                    from: pa,
                    kind,
                    params: params.clone(),
                    then: None,
                    needs_answer: true,
                }),
            }
        }
        // After an offer the computer is less eager for a while, whatever the answer.
        diplo::annoy(st, ph, pa, 1);
        let r = match kind {
            msg::PROPOSE_NAP | msg::PROPOSE_ALLIANCE | msg::PROPOSE_TRADE => {
                st.rng.one_to(30) as i32 + 20
            }
            msg::PROPOSE_PEACE => st.rng.one_to(50) as i32 + 50,
            msg::OFFER_TECH_TRADE => st.rng.one_to(30) as i32 + 20,
            _ => 0,
        };
        let rr = rel_mut(st, ph, pa);
        match kind {
            msg::PROPOSE_NAP | msg::PROPOSE_ALLIANCE => rr.mood_treaty -= r,
            msg::PROPOSE_TRADE => rr.mood_trade -= r,
            msg::PROPOSE_PEACE => rr.mood_peace -= r,
            msg::OFFER_TECH_TRADE => rr.mood_tech -= r,
            _ => {}
        }
        return;
    }
    if kind == msg::FIRST_CONTACT {
        params = MsgParams::default();
    }
    st.pending.messages.push(Incoming {
        to: ph,
        from: pa,
        kind,
        params,
        then: None,
        needs_answer: false,
    });
}

/// The player answers message `idx` of `pending.messages` (1oom `game_audience_ai_offers_*`).
/// `pick` chooses a tech for tech trade offers. Returns the computer's reply message.
pub fn answer(
    st: &mut State,
    ph: EmpireId,
    idx: usize,
    accept: bool,
    pick: Option<TechRef>,
) -> Result<Option<Incoming>, &'static str> {
    let Some(m) = st.pending.messages.get(idx).cloned() else {
        return Err("no_message");
    };
    if m.to != ph {
        return Err("no_message");
    }
    st.pending.messages.remove(idx);
    if !m.needs_answer {
        return Ok(None);
    }
    resolve(st, &m, accept, pick)
}

fn note(m: &Incoming, kind: u8, params: MsgParams) -> Incoming {
    Incoming {
        to: m.to,
        from: m.from,
        kind,
        params,
        then: None,
        needs_answer: false,
    }
}

fn resolve(
    st: &mut State,
    m: &Incoming,
    accept: bool,
    pick: Option<TechRef>,
) -> Result<Option<Incoming>, &'static str> {
    let (ph, pa) = (m.to, m.from);
    let offer = |st: &mut State| {
        let r = rel(st, ph, pa).clone();
        st.empires[ph as usize].reserve += r.offer_bc;
        if let Some((f, t)) = r.offer_tech {
            tech::grant_from(st, ph, f as usize, t, Source::Trade, Some(pa));
        }
        let rr = rel_mut(st, ph, pa);
        rr.offer_bc = 0;
        rr.offer_tech = None;
    };
    match m.kind {
        msg::ASK_BREAK_ALLIANCE => {
            let pf = m.params.empire;
            if accept {
                if let Some(pf) = pf {
                    diplo::break_treaty(st, ph, pf);
                }
                if let Some(k) = m.then {
                    deliver_after_break(st, ph, pa, k);
                }
                Ok(None)
            } else {
                rel_mut(st, pa, ph).relation = -100;
                rel_mut(st, ph, pa).relation = -100;
                if m.then == Some(msg::JOIN_WAR) {
                    diplo::break_treaty(st, pa, ph);
                    return Ok(Some(note(m, msg::ALLIANCE_ENDED, MsgParams::default())));
                }
                Ok(None)
            }
        }
        msg::OFFER_TECH_TRADE => {
            if !accept {
                return Ok(None);
            }
            let r = rel(st, ph, pa).clone();
            let Some(choice) = pick.filter(|p| r.tech_trade.contains(p)) else {
                return Err("invalid_tech");
            };
            tech::grant_from(st, ph, choice.0 as usize, choice.1, Source::Trade, Some(pa));
            if let Some((f, t)) = r.want_tech {
                tech::grant_from(st, pa, f as usize, t, Source::Trade, Some(ph));
            }
            Ok(None)
        }
        msg::JOIN_WAR => {
            let war = m.params.empire;
            if accept {
                if let Some(w) = war {
                    diplo::start_war(st, ph, w);
                }
                Ok(Some(note(m, msg::THANKS_JOIN_WAR, m.params.clone())))
            } else {
                diplo::break_treaty(st, pa, ph);
                Ok(Some(note(m, msg::ALLIANCE_ENDED, m.params.clone())))
            }
        }
        k => {
            if !accept {
                return Ok(None);
            }
            match k {
                msg::PROPOSE_NAP => diplo::set_treaty(st, ph, pa, Treaty::NonAggression),
                msg::PROPOSE_ALLIANCE => diplo::set_treaty(st, ph, pa, Treaty::Alliance),
                msg::PROPOSE_TRADE => {
                    let bc = rel(st, ph, pa).want_trade;
                    diplo::set_trade(st, ph, pa, bc);
                }
                msg::PROPOSE_PEACE => {
                    diplo::stop_war(st, ph, pa);
                    if rel(st, ph, pa).relation < 80 {
                        let v = rel(st, ph, pa).relation + 20;
                        rel_mut(st, ph, pa).relation = v;
                        rel_mut(st, pa, ph).relation = v;
                    }
                    diplo::annoy(st, ph, pa, 2);
                }
                _ => return Ok(None),
            }
            offer(st);
            Ok(None)
        }
    }
}

fn deliver_after_break(st: &mut State, ph: EmpireId, pa: EmpireId, kind: u8) {
    let params = params_of(st, ph, pa, kind);
    if kind == msg::PAYS_BOUNTY {
        let r = rel(st, ph, pa).clone();
        st.empires[ph as usize].reserve += r.attack_gift_bc;
        if let Some((f, t)) = r.attack_gift_tech {
            tech::grant_from(st, ph, f as usize, t, Source::Trade, Some(pa));
        }
        rel_mut(st, ph, pa).bounty_collect = None;
    }
    st.pending.messages.insert(
        0,
        Incoming {
            to: ph,
            from: pa,
            kind,
            params,
            then: None,
            needs_answer: is_proposal(kind),
        },
    );
}

/// Unanswered proposals count as refused at the end of the next turn.
pub fn expire(st: &mut State) {
    let old = std::mem::take(&mut st.pending.messages);
    for m in old {
        if m.needs_answer {
            let _ = resolve(st, &m, false, None);
        }
    }
    st.pending.deals.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Game, NewGameOptions};

    fn met() -> Game {
        let mut g = Game::new(&NewGameOptions {
            seed: 21,
            players: 3,
            ..Default::default()
        })
        .unwrap();
        let st = g.state_mut();
        for b in 1..3u8 {
            rel_mut(st, 0, b).contact = true;
            rel_mut(st, b, 0).contact = true;
            rel_mut(st, 0, b).have_met = 2;
        }
        g
    }

    #[test]
    fn friendly_empires_accept_a_pact() {
        let mut g = met();
        let st = g.state_mut();
        rel_mut(st, 0, 1).relation = 60;
        rel_mut(st, 1, 0).relation = 60;
        rel_mut(st, 0, 1).mood_treaty = 100;
        rel_mut(st, 0, 1).trust = 50;
        let r = talk(st, 0, 1, &Talk::ProposeNap).unwrap();
        assert!(r.accepted || r.sweetener.is_some(), "{r:?}");
        if r.accepted {
            assert_eq!(rel(st, 0, 1).treaty, Treaty::NonAggression);
            assert_eq!(r.kind, msg::ACCEPT_NAP);
        }
    }

    #[test]
    fn hostile_empires_refuse() {
        let mut g = met();
        let st = g.state_mut();
        rel_mut(st, 0, 2).relation = -50;
        rel_mut(st, 2, 0).relation = -50;
        let r = talk(st, 0, 2, &Talk::ProposeAlliance);
        let r = r.unwrap();
        assert!(!r.accepted);
        assert_ne!(rel(st, 0, 2).treaty, Treaty::Alliance);
    }

    #[test]
    fn computer_proposals_wait_for_an_answer() {
        let mut g = met();
        let st = g.state_mut();
        rel_mut(st, 0, 1).diplo_type = msg::PROPOSE_NAP;
        collect(st);
        let i = st
            .pending
            .messages
            .iter()
            .position(|m| m.kind == msg::PROPOSE_NAP)
            .unwrap();
        answer(st, 0, i, true, None).unwrap();
        assert_eq!(rel(st, 0, 1).treaty, Treaty::NonAggression);
    }

    #[test]
    fn tribute_improves_relations() {
        let mut g = met();
        let st = g.state_mut();
        st.empires[0].reserve = 200;
        st.empires[1].total_prod = 100;
        let before = rel(st, 1, 0).relation;
        let r = talk(st, 0, 1, &Talk::TributeBc { bc: 100 }).unwrap();
        assert!(r.accepted);
        assert!(rel(st, 1, 0).relation > before);
        assert_eq!(st.empires[0].reserve, 100);
    }
}
