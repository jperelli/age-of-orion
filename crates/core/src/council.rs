//! The Galactic Council (1oom `game_election.c`): every 25 years once two thirds of the
//! planets are settled, the two biggest empires are candidates and every empire votes with
//! one vote per 100 population. Two thirds of the votes elect the High Master: if it is a
//! human, the game is won; if it is a computer, a human may accept (and lose) or refuse and
//! fight the final war against everyone else.

use serde::{Deserialize, Serialize};

use crate::ai_diplo;
use crate::diplo::{self, Treaty};
use crate::state::*;
use crate::types::*;

pub const COUNCIL_YEARS: i32 = 25;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Council {
    pub held: bool,
    /// The council chose a High Master who was refused: everyone fights the refusers.
    pub final_war: bool,
    pub refuse: Vec<bool>,
    /// Who each empire voted for last time.
    pub voted: Vec<Option<EmpireId>>,
    /// The elected High Master.
    pub winner: Option<EmpireId>,
    /// An election waiting for the human votes (or, after the vote, for acceptance).
    pub pending: Option<Election>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Election {
    pub candidates: [EmpireId; 2],
    /// Votes of each empire (population / 100, rounded up).
    pub votes: Vec<i32>,
    pub total: i32,
    /// Whether each empire has voted, and for whom (`None` = abstained).
    pub has_voted: Vec<bool>,
    pub ballots: Vec<Option<EmpireId>>,
    pub got: [i32; 2],
    pub elected: Option<EmpireId>,
    /// The vote is over and a computer was elected: humans must accept or refuse.
    pub awaiting_acceptance: bool,
}

fn year_n(st: &State) -> i32 {
    (st.year as i32 - st.rules.start_year as i32 + 1).max(1)
}

/// Whether the council meets this turn (1oom `game_turn_process`).
pub fn due(st: &State) -> bool {
    let alive = st.empires.iter().filter(|e| e.alive).count();
    let colonies = st.planets.iter().filter(|p| p.owner.is_some()).count();
    (year_n(st) % COUNCIL_YEARS == 0 || !st.council.held)
        && alive > 2
        && st.planets.len() * 2 / 3 <= colonies
        && st.winner.is_none()
        && !st.council.final_war
        && st.council.pending.is_none()
}

/// 1oom `game_election_prepare`.
pub fn convene(st: &mut State) {
    let n = st.empires.len();
    let mut pop = vec![0i32; n];
    for p in &st.planets {
        if let Some(o) = p.owner {
            if p.unrest != Unrest::Rebellion {
                pop[o as usize] += p.pop;
            }
        }
    }
    let votes: Vec<i32> = pop
        .iter()
        .map(|&v| v / 100 + i32::from(v % 100 != 0))
        .collect();
    let total = votes.iter().sum();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| (-votes[i], i));
    let (a, b) = (order[0] as u8, order[1] as u8);
    let candidates = if st.empires[a as usize].ai && !st.empires[b as usize].ai {
        [b, a]
    } else {
        [a, b]
    };
    st.council.held = true;
    st.council.voted.resize(n, None);
    st.council.refuse.resize(n, false);
    st.council.pending = Some(Election {
        candidates,
        votes,
        total,
        has_voted: vec![false; n],
        ballots: vec![None; n],
        got: [0, 0],
        elected: None,
        awaiting_acceptance: false,
    });
    st.push_event(
        vec![],
        EventKind::Council {
            candidates: candidates.to_vec(),
        },
    );
    if st.empires.iter().all(|e| e.ai || !e.alive) {
        finish(st);
    }
}

fn cast(st: &mut State, voter: EmpireId, choice: Option<EmpireId>) {
    let Some(el) = st.council.pending.clone() else {
        return;
    };
    let n = el.votes[voter as usize];
    let choice = if n == 0 { None } else { choice };
    match choice {
        None => {
            st.council.voted[voter as usize] = None;
            for c in el.candidates {
                diplo::act(st, -6, voter, c, 0, 0, 0);
            }
        }
        Some(c) => {
            let k = usize::from(el.candidates[1] == c);
            let other = el.candidates[k ^ 1];
            st.council.voted[voter as usize] = Some(c);
            if !st.empires[voter as usize].ai && el.candidates[0] != voter {
                diplo::act(st, 24, voter, c, diplo::msg::VOTED_FOR, 0, c as i32);
                diplo::act(
                    st,
                    -12,
                    voter,
                    other,
                    diplo::msg::VOTED_AGAINST,
                    0,
                    c as i32,
                );
            } else {
                diplo::act(st, 24, voter, c, 0, 0, 0);
                diplo::act(st, -12, voter, other, 0, 0, 0);
            }
            if let Some(e) = st.council.pending.as_mut() {
                e.got[k] += n;
            }
        }
    }
    if let Some(e) = st.council.pending.as_mut() {
        e.has_voted[voter as usize] = true;
        e.ballots[voter as usize] = choice;
    }
}

/// A human votes: `None` abstains. Computers vote when the last human has voted.
pub fn vote(st: &mut State, voter: EmpireId, choice: Option<EmpireId>) -> Result<(), &'static str> {
    let Some(el) = st.council.pending.as_ref() else {
        return Err("no_election");
    };
    if el.awaiting_acceptance || el.has_voted[voter as usize] {
        return Err("already_voted");
    }
    if let Some(c) = choice {
        if !el.candidates.contains(&c) {
            return Err("invalid_empire");
        }
    }
    cast(st, voter, choice);
    let waiting = st.empires.iter().enumerate().any(|(i, e)| {
        !e.ai
            && e.alive
            && st
                .council
                .pending
                .as_ref()
                .is_some_and(|el| !el.has_voted[i])
    });
    if !waiting {
        finish(st);
    }
    Ok(())
}

/// Computers vote and the result is counted (1oom `game_election`).
pub fn finish(st: &mut State) {
    let Some(el) = st.council.pending.clone() else {
        return;
    };
    let n = st.empires.len() as u8;
    for i in 0..n {
        if st.empires[i as usize].alive && !el.has_voted[i as usize] {
            let choice = if st.empires[i as usize].ai {
                match ai_diplo::vote(st, i, el.candidates[0], el.candidates[1]) {
                    1 => Some(el.candidates[0]),
                    2 => Some(el.candidates[1]),
                    _ => None,
                }
            } else {
                None
            };
            cast(st, i, choice);
        }
    }
    let el = st.council.pending.clone().expect("election");
    let need = (el.total + 1) * 2 / 3;
    let elected = (0..2)
        .find(|&k| need <= el.got[k])
        .map(|k| el.candidates[k]);
    st.push_event(
        vec![],
        EventKind::CouncilResult {
            candidates: el.candidates.to_vec(),
            votes: el.got.to_vec(),
            total: el.total,
            elected,
        },
    );
    match elected {
        None => st.council.pending = None,
        Some(w) => {
            st.council.winner = Some(w);
            let humans_lose = st
                .empires
                .iter()
                .enumerate()
                .any(|(i, e)| !e.ai && e.alive && i as u8 != w);
            if humans_lose {
                if let Some(e) = st.council.pending.as_mut() {
                    e.elected = Some(w);
                    e.awaiting_acceptance = true;
                }
            } else {
                st.council.pending = None;
                st.winner = Some(w);
                st.victory = Some(Victory::Council);
                st.push_event(vec![], EventKind::Victory { empire: w });
            }
        }
    }
}

/// A human accepts or refuses the High Master (1oom `game_election_accept`).
pub fn accept(st: &mut State, human: EmpireId, yes: bool) -> Result<(), &'static str> {
    let Some(el) = st.council.pending.clone().filter(|e| e.awaiting_acceptance) else {
        return Err("no_election");
    };
    let w = el.elected.expect("elected");
    st.council.pending = None;
    if yes {
        st.winner = Some(w);
        st.victory = Some(Victory::Council);
        st.push_event(vec![], EventKind::Victory { empire: w });
        return Ok(());
    }
    st.council.refuse = vec![false; st.empires.len()];
    st.council.refuse[human as usize] = true;
    let n = st.empires.len() as u8;
    for p1 in 0..n {
        if st.council.refuse[p1 as usize] {
            continue;
        }
        diplo::break_trade(st, human, p1);
        diplo::set_treaty(st, human, p1, Treaty::FinalWar);
        for p2 in p1 + 1..n {
            if !st.council.refuse[p2 as usize] {
                diplo::set_treaty(st, p1, p2, Treaty::Alliance);
            }
        }
    }
    st.council.final_war = true;
    crate::tech::final_war_share(st);
    st.push_event(vec![], EventKind::FinalWar { against: human });
    Ok(())
}

/// Unanswered elections are settled when the next turn ends: humans abstain and accept.
pub fn expire(st: &mut State) {
    let Some(el) = st.council.pending.clone() else {
        return;
    };
    if el.awaiting_acceptance {
        let h = (0..st.empires.len())
            .find(|&i| !st.empires[i].ai)
            .unwrap_or(0) as u8;
        let _ = accept(st, h, true);
    } else {
        finish(st);
        if st
            .council
            .pending
            .as_ref()
            .is_some_and(|e| e.awaiting_acceptance)
        {
            let h = (0..st.empires.len())
                .find(|&i| !st.empires[i].ai)
                .unwrap_or(0) as u8;
            let _ = accept(st, h, true);
        }
    }
}
