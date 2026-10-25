//! The statistics of a game, for the stats screen: what each empire had at the start of each
//! year (population, colonies, factories, production, reserve, ships...) and what it did, as
//! running totals with their history (colonies founded, worlds conquered and lost, battles
//! won and lost against each empire, ships built by design, sieges, techs stolen, councils...).
//! Deeds are counted from the events as they are pushed, so nothing here changes the game;
//! the UI adds the player's own `ui_*` totals (clicks, time played) with `Command::Tally`.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::state::*;
use crate::tech::Source;
use crate::types::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Stats {
    /// The year of each sample.
    pub years: Vec<u32>,
    pub empires: Vec<EmpireStats>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EmpireStats {
    /// What the empire had, one value per sample (`Stats::years`).
    pub series: BTreeMap<String, Vec<i64>>,
    /// Running totals of what it did.
    pub totals: BTreeMap<String, i64>,
    /// Each total's history: (sample, value) at each sample where it changed.
    pub history: BTreeMap<String, Vec<(u32, i64)>>,
    /// Ships built, by the design's name when built.
    pub ships_built: BTreeMap<String, i64>,
    /// Against each other empire (by id): battles won and lost, ships destroyed and lost,
    /// worlds taken and lost.
    pub versus: BTreeMap<String, Vec<i64>>,
}

/// The empire's stats, created as needed (old saves have none).
fn of(st: &mut State, e: EmpireId) -> &mut EmpireStats {
    let n = st.empires.len().max(e as usize + 1);
    if st.stats.empires.len() < n {
        st.stats.empires.resize_with(n, Default::default);
    }
    &mut st.stats.empires[e as usize]
}

/// Adds `n` to total `key` of empire `e`.
pub fn add(st: &mut State, e: EmpireId, key: &str, n: i64) {
    if n != 0 && (e as usize) < st.empires.len() {
        *of(st, e).totals.entry(key.to_string()).or_insert(0) += n;
    }
}

fn versus(st: &mut State, e: EmpireId, other: EmpireId, key: &str, n: i64) {
    let count = st.empires.len();
    if n == 0 || e as usize >= count || other as usize >= count {
        return;
    }
    let v = of(st, e)
        .versus
        .entry(key.to_string())
        .or_insert_with(|| vec![0; count]);
    if v.len() < count {
        v.resize(count, 0);
    }
    v[other as usize] += n;
}

/// Counts an event about to be pushed (`State::push_event`).
pub fn count(st: &mut State, to: &[EmpireId], kind: &EventKind) {
    let owner = |st: &State, p: PlanetId| st.planets.get(p).and_then(|p| p.owner);
    match kind {
        EventKind::ColonyFounded { planet } => {
            if let Some(o) = owner(st, *planet).or(to.first().copied()) {
                add(st, o, "colonies_founded", 1);
            }
        }
        EventKind::ShipsBuilt {
            planet,
            design,
            count,
        } => {
            if let Some(o) = owner(st, *planet).or(to.first().copied()) {
                let name = st.empires[o as usize]
                    .designs
                    .get(*design)
                    .map(|d| d.name.clone())
                    .unwrap_or_default();
                add(st, o, "ships_built", *count as i64);
                *of(st, o).ships_built.entry(name).or_insert(0) += *count as i64;
            }
        }
        EventKind::StargateBuilt { planet } => {
            if let Some(o) = owner(st, *planet) {
                add(st, o, "stargates_built", 1);
            }
        }
        EventKind::TechDiscovered { source, from, .. } => {
            let Some(&e) = to.first() else { return };
            let key = match source {
                Source::Research => "techs_researched",
                Source::Espionage => "techs_stolen",
                Source::Trade => "techs_traded",
                _ => "techs_found",
            };
            add(st, e, key, 1);
            if let (Source::Espionage, Some(v)) = (source, from) {
                add(st, *v, "techs_lost_to_spies", 1);
            }
        }
        EventKind::SpiesCaught { spy, target, count } => {
            add(st, *target, "spies_caught", *count as i64);
            add(st, *spy, "spies_lost", *count as i64);
        }
        EventKind::Council { .. } => {
            for e in 0..st.empires.len() {
                if st.empires[e].alive {
                    add(st, e as EmpireId, "councils", 1);
                }
            }
        }
        EventKind::CouncilResult {
            candidates,
            votes,
            elected,
            ..
        } => {
            for (c, v) in candidates.iter().zip(votes) {
                add(st, *c, "council_votes", *v as i64);
            }
            if let Some(e) = elected {
                add(st, *e, "council_elected", 1);
            }
        }
        EventKind::Battle {
            sides,
            winner,
            losses,
            ..
        } => {
            let count = st.empires.len() as i32;
            let empire = move |s: i32| (0..count).contains(&s).then_some(s as EmpireId);
            for (i, &s) in sides.iter().enumerate() {
                let Some(e) = empire(s) else { continue };
                let won = *winner == Some(s);
                add(st, e, "battles", 1);
                add(st, e, if won { "battles_won" } else { "battles_lost" }, 1);
                add(
                    st,
                    e,
                    "ships_lost",
                    losses.get(i).copied().unwrap_or(0) as i64,
                );
                for (j, &o) in sides.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let theirs = losses.get(j).copied().unwrap_or(0) as i64;
                    add(st, e, "ships_destroyed", theirs);
                    match empire(o) {
                        Some(oe) => {
                            versus(
                                st,
                                e,
                                oe,
                                if won { "battles_won" } else { "battles_lost" },
                                1,
                            );
                            versus(st, e, oe, "ships_destroyed", theirs);
                            versus(
                                st,
                                e,
                                oe,
                                "ships_lost",
                                losses.get(i).copied().unwrap_or(0) as i64,
                            );
                        }
                        None => add(
                            st,
                            e,
                            if won {
                                "monsters_beaten"
                            } else {
                                "monsters_lost_to"
                            },
                            1,
                        ),
                    }
                }
            }
        }
        EventKind::Bombed {
            planet,
            attacker,
            pop_killed,
            factories_destroyed,
        } => {
            add(st, *attacker, "bombings", 1);
            add(st, *attacker, "pop_killed_by_bombs", *pop_killed as i64);
            add(
                st,
                *attacker,
                "factories_bombed",
                *factories_destroyed as i64,
            );
            if let Some(o) = owner(st, *planet) {
                add(st, o, "bombed", 1);
            }
        }
        EventKind::TransportsLost { owner, pop, .. } => {
            add(st, *owner, "troops_lost_in_transit", *pop as i64)
        }
        EventKind::GroundBattle {
            attacker,
            defender,
            won,
            rebels,
            captured,
            ..
        } => {
            if *rebels {
                add(
                    st,
                    *attacker,
                    if *won {
                        "rebellions_put_down"
                    } else {
                        "rebellions_lost"
                    },
                    1,
                );
                return;
            }
            add(st, *attacker, "invasions", 1);
            add(
                st,
                *attacker,
                if *won {
                    "invasions_won"
                } else {
                    "invasions_lost"
                },
                1,
            );
            add(st, *defender, "invaded", 1);
            add(
                st,
                *defender,
                if *won {
                    "invasions_fallen"
                } else {
                    "invasions_repelled"
                },
                1,
            );
            add(st, *attacker, "techs_captured", captured.len() as i64);
        }
        EventKind::PlanetCaptured { from, by, .. } => {
            add(st, *by, "worlds_conquered", 1);
            add(st, *from, "worlds_lost", 1);
            versus(st, *by, *from, "worlds_taken", 1);
            versus(st, *from, *by, "worlds_lost", 1);
        }
        EventKind::ColonyDestroyed { planet, owner } => {
            add(st, *owner, "worlds_lost", 1);
            add(st, *owner, "colonies_destroyed", 1);
            let by = st.events.iter().rev().find_map(|e| match e.kind {
                EventKind::Bombed {
                    planet: p,
                    attacker,
                    ..
                } if p == *planet => Some(attacker),
                _ => None,
            });
            if let Some(b) = by {
                add(st, b, "colonies_razed", 1);
                versus(st, b, *owner, "worlds_taken", 1);
                versus(st, *owner, b, "worlds_lost", 1);
            }
        }
        EventKind::GuardianDefeated { by, .. } => add(st, *by, "guardian_defeated", 1),
        EventKind::OrionColonized { by, .. } => add(st, *by, "orion_colonized", 1),
        _ => {}
    }
}

/// Samples what each empire has, at the start of the year now shown (after a turn, and at
/// the start of the game), and the history of its totals.
pub fn sample(st: &mut State) {
    let n = st.empires.len();
    if n == 0 {
        return;
    }
    let idx = st.stats.years.len() as u32;
    st.stats.years.push(st.year);
    let mut values: Vec<Vec<(&'static str, i64)>> = Vec::with_capacity(n);
    for (i, e) in st.empires.iter().enumerate() {
        let mine = || st.planets.iter().filter(move |p| p.owner == Some(i as u8));
        let ships: i64 = st
            .fleets
            .iter()
            .filter(|f| f.owner as usize == i)
            .flat_map(|f| f.ships.iter())
            .map(|&s| s as i64)
            .sum::<i64>()
            + e.orbit
                .iter()
                .flat_map(|o| o.iter())
                .map(|&s| s as i64)
                .sum::<i64>();
        values.push(vec![
            ("population", mine().map(|p| p.pop as i64).sum()),
            ("colonies", mine().count() as i64),
            ("factories", mine().map(|p| p.factories as i64).sum()),
            ("production", e.total_prod as i64),
            ("reserve", e.reserve as i64),
            ("ships", ships),
            ("bases", mine().map(|p| p.bases as i64).sum()),
            (
                "techs",
                e.tech.completed.iter().map(|f| f.len() as i64).sum(),
            ),
            ("explored", e.explored.iter().filter(|&&x| x).count() as i64),
            ("trade", e.total_trade as i64),
        ]);
    }
    for (i, vals) in values.into_iter().enumerate() {
        let es = of(st, i as EmpireId);
        for (k, v) in vals {
            let s = es.series.entry(k.to_string()).or_default();
            // An empire's series start where the stats did (old saves): pad the front.
            while s.len() < idx as usize {
                s.insert(0, 0);
            }
            s.push(v);
        }
        let totals: Vec<(String, i64)> = es.totals.iter().map(|(k, v)| (k.clone(), *v)).collect();
        for (k, v) in totals {
            let h = es.history.entry(k).or_default();
            if h.last().map(|x| x.1) != Some(v) {
                h.push((idx, v));
            }
        }
    }
}

/// Adds the player's own counts from the UI (`ui_` names only: clicks, keys, time played...).
pub fn tally(st: &mut State, e: EmpireId, counts: &BTreeMap<String, i64>) {
    for (k, &v) in counts {
        let ok = k.starts_with("ui_")
            && k.len() <= 32
            && k.bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_' || b.is_ascii_digit());
        if ok {
            add(st, e, k, v.clamp(0, 1_000_000));
        }
    }
}

/// What the stats screen shows of one empire.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EmpireStatsView {
    pub empire: EmpireId,
    pub race: Race,
    pub banner: u8,
    pub leader: String,
    pub alive: bool,
    #[serde(flatten)]
    pub stats: EmpireStats,
}

/// The stats screen's data: the player's empire during the game, every empire once the game
/// is over for the player (won, lost or eliminated).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatsView {
    pub years: Vec<u32>,
    pub player: EmpireId,
    pub all: bool,
    pub empires: Vec<EmpireStatsView>,
}

pub fn view(st: &State, me: EmpireId) -> StatsView {
    let all = st.winner.is_some() || !st.empires.get(me as usize).is_some_and(|e| e.alive);
    let empires = (0..st.empires.len())
        .filter(|&i| all || i == me as usize)
        .map(|i| {
            let e = &st.empires[i];
            let mut stats = st.stats.empires.get(i).cloned().unwrap_or_default();
            if i != me as usize {
                stats.totals.retain(|k, _| !k.starts_with("ui_"));
                stats.history.retain(|k, _| !k.starts_with("ui_"));
            }
            EmpireStatsView {
                empire: i as EmpireId,
                race: e.race,
                banner: e.banner,
                leader: e.leader.clone(),
                alive: e.alive,
                stats,
            }
        })
        .collect();
    StatsView {
        years: st.stats.years.clone(),
        player: me,
        all,
        empires,
    }
}
