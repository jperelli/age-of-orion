//! Orders an empire can give. Humans (through the UI) and the AI use the same commands.

use serde::{Deserialize, Serialize};

use crate::battle::Action;
use crate::design::{self, ShipDesign, BUILD_STARGATE, MAX_DESIGNS};
use crate::diplo::{self, SpyMode, TechRef, Treaty};
use crate::spy::SabotageKind;
use crate::state::*;
use crate::tech;
use crate::types::*;
use crate::{audience, combat, council, spy};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    /// Spending of a colony in percent (normalized to a total of 100).
    SetSliders {
        planet: PlanetId,
        sliders: [i32; NUM_SLIDERS],
    },
    /// Design built with the ship slider, or `BUILD_STARGATE` (6).
    SetBuild {
        planet: PlanetId,
        design: usize,
    },
    /// Where a colony sends its new ships; `None` keeps them in orbit.
    SetRally {
        planet: PlanetId,
        to: Option<PlanetId>,
    },
    /// Sends ships from an orbit (counts per design) to another planet.
    SendFleet {
        from: PlanetId,
        to: PlanetId,
        ships: Vec<u32>,
    },
    /// Changes the destination of a fleet in flight.
    RedirectFleet {
        fleet: u32,
        to: PlanetId,
    },
    /// Scraps ships in orbit; a quarter of their cost goes to the reserve.
    ScrapShips {
        planet: PlanetId,
        ships: Vec<u32>,
    },
    /// Moves colonists to another planet (own: reinforce, enemy: invade).
    SendTransport {
        from: PlanetId,
        to: PlanetId,
        pop: i32,
    },
    /// Research share per field in percent (normalized to a total of 100).
    SetResearch {
        alloc: [u32; NUM_FIELDS],
    },
    /// Chooses the tech researched in a field (an ongoing project loses its investment).
    SetResearchProject {
        field: usize,
        tech: u8,
    },
    /// Adds a design, or replaces design `slot` (scrapping its ships).
    SaveDesign {
        slot: Option<usize>,
        design: ShipDesign,
    },
    /// Removes a design and scraps all its ships.
    ScrapDesign {
        design: usize,
    },
    /// Whether ships holding the orbit of enemy colonies bomb them.
    SetBombard {
        enabled: bool,
    },
    /// Spending on spies against `target`, tenths of a percent of production (0..=100).
    SetSpying {
        target: EmpireId,
        tenths: i32,
    },
    SetSpyMode {
        target: EmpireId,
        mode: SpyMode,
    },
    /// Internal security, tenths of a percent (0..=200).
    SetSecurity {
        tenths: i32,
    },
    /// Taxes for the reserve, tenths of a percent (0..=200); half the tax reaches the reserve.
    SetTax {
        tenths: i32,
    },
    /// Sends BC from the reserve to boost a colony's production.
    TransferReserve {
        planet: PlanetId,
        bc: i32,
    },
    /// Answers message `index` of the pending messages.
    AnswerMessage {
        index: usize,
        accept: bool,
        #[serde(default)]
        pick: Option<TechRef>,
    },
    /// Steals the tech of `field` offered by spy success `index`.
    StealTech {
        index: usize,
        field: usize,
    },
    /// Uses saboteur success `index` on a planet.
    Sabotage {
        index: usize,
        planet: PlanetId,
        kind: SabotageKind,
    },
    /// Votes in the Galactic Council (`None` abstains).
    Vote {
        candidate: Option<EmpireId>,
    },
    /// Accepts or refuses the High Master elected by the council.
    AcceptCouncil {
        accept: bool,
    },
    /// Begins the battle the turn is waiting for (1oom `ui_battle_init`).
    StartBattle {
        mode: BattleMode,
    },
    /// An order for the current stack in the battle.
    Battle {
        action: Action,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleMode {
    /// Command the ships on the battle grid.
    Fight,
    /// Let the computer fight for you.
    Auto,
    /// Every ship retreats.
    Retreat,
}

/// Why a command was rejected; the UI shows the text `error.<code>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandError {
    pub code: &'static str,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl std::error::Error for CommandError {}

fn err(code: &'static str) -> Result<(), CommandError> {
    Err(CommandError { code })
}

fn check_planet(st: &State, p: PlanetId) -> Result<(), CommandError> {
    if p < st.planets.len() {
        Ok(())
    } else {
        err("invalid_planet")
    }
}

fn own(st: &State, empire: EmpireId, p: PlanetId) -> Result<(), CommandError> {
    check_planet(st, p)?;
    if st.planets[p].owner == Some(empire) {
        Ok(())
    } else {
        err("not_owner")
    }
}

/// Normalizes percentages to a total of exactly 100.
fn normalize<const N: usize>(v: [i64; N]) -> Option<[i64; N]> {
    let v = v.map(|x| x.max(0));
    let sum: i64 = v.iter().sum();
    if sum == 0 {
        return None;
    }
    let mut out = v.map(|x| x * 100 / sum);
    let rest = 100 - out.iter().sum::<i64>();
    let imax = (0..N).max_by_key(|&i| v[i]).unwrap_or(0);
    out[imax] += rest;
    Some(out)
}

/// Fuel range of a group of ships: the empire range plus the smallest design bonus.
pub fn fleet_range(st: &State, empire: EmpireId, ships: &[u32]) -> i32 {
    let e = &st.empires[empire as usize];
    let extra = ships
        .iter()
        .enumerate()
        .filter(|(_, &c)| c > 0)
        .map(|(d, _)| e.designs.get(d).map_or(0, |d| d.extra_range()))
        .min()
        .unwrap_or(0);
    e.abilities().fuel_range + extra
}

/// Removes ships of `design` everywhere and refunds a quarter of their cost.
fn scrap_all(st: &mut State, empire: EmpireId, design: usize) {
    let cost = st.ship_cost(empire, design);
    let e = &mut st.empires[empire as usize];
    let mut count: u32 = 0;
    for o in e.orbit.iter_mut() {
        count += o[design];
        o[design] = 0;
    }
    for f in st.fleets.iter_mut().filter(|f| f.owner == empire) {
        count += f.ships[design];
        f.ships[design] = 0;
    }
    st.fleets.retain(|f| f.ships.iter().any(|&c| c > 0));
    st.empires[empire as usize].reserve += count as i32 * cost / 4;
}

pub fn apply(st: &mut State, empire: EmpireId, cmd: &Command) -> Result<(), CommandError> {
    let accepting = matches!(cmd, Command::AcceptCouncil { .. });
    if (st.winner.is_some() && !accepting)
        || !st.empires.get(empire as usize).is_some_and(|e| e.alive)
    {
        return err("game_over");
    }
    let battle_cmd = matches!(cmd, Command::StartBattle { .. } | Command::Battle { .. });
    if st.progress.is_some() && !battle_cmd {
        return err("battle_in_progress");
    }
    if battle_cmd {
        let Some(mut bt) = st.battle.take() else {
            return err("no_battle");
        };
        let mine = bt.s.iter().any(|s| s.party == empire as i32 && s.human);
        let r = if !mine || bt.winner.is_some() {
            Err("no_battle")
        } else {
            match *cmd {
                Command::StartBattle { mode } if !bt.started => {
                    bt.start(st, mode == BattleMode::Auto, mode == BattleMode::Retreat);
                    Ok(())
                }
                Command::Battle { action } if bt.started => bt.act(st, action),
                _ => Err("no_battle"),
            }
        };
        let over = bt.winner.is_some();
        st.battle = Some(bt);
        if over {
            combat::finish_battle(st);
        }
        return r.map_err(|code| CommandError { code });
    }
    match *cmd {
        Command::SetSliders { planet, sliders } => {
            own(st, empire, planet)?;
            let Some(s) = normalize(sliders.map(|x| x as i64)) else {
                return err("invalid_sliders");
            };
            st.planets[planet].sliders = s.map(|x| x as i32);
        }
        Command::SetBuild { planet, design } => {
            own(st, empire, planet)?;
            let e = &st.empires[empire as usize];
            if design == BUILD_STARGATE {
                if !e.abilities().stargates || st.planets[planet].stargate {
                    return err("invalid_design");
                }
            } else if design >= e.designs.len() {
                return err("invalid_design");
            }
            st.planets[planet].build = design;
        }
        Command::SetRally { planet, to } => {
            own(st, empire, planet)?;
            if let Some(t) = to {
                own(st, empire, t)?;
            }
            st.planets[planet].rally = to.filter(|&t| t != planet);
        }
        Command::SendFleet {
            from,
            to,
            ref ships,
        } => {
            check_planet(st, from)?;
            check_planet(st, to)?;
            let orbit = &st.empires[empire as usize].orbit[from];
            if from == to {
                return err("same_planet");
            }
            if ships.len() != orbit.len()
                || ships.iter().all(|&c| c == 0)
                || ships.iter().zip(orbit).any(|(&c, &have)| c > have)
            {
                return err("no_ships");
            }
            if st.range_to(empire, to) > fleet_range(st, empire, ships) {
                return err("out_of_range");
            }
            let pairs: Vec<(usize, u32)> = ships
                .iter()
                .copied()
                .enumerate()
                .filter(|&(_, c)| c > 0)
                .collect();
            crate::fleet::send_from_orbit(st, empire, from, to, &pairs);
        }
        Command::RedirectFleet { fleet, to } => {
            check_planet(st, to)?;
            let Some(i) = st
                .fleets
                .iter()
                .position(|f| f.id == fleet && f.owner == empire)
            else {
                return err("no_ships");
            };
            if st.fleets[i].speed == SPEED_STARGATE {
                return err("in_stargate");
            }
            let ships = st.fleets[i].ships.clone();
            if st.range_to(empire, to) > fleet_range(st, empire, &ships) {
                return err("out_of_range");
            }
            let (fx, fy) = (st.fleets[i].x, st.fleets[i].y);
            let from = st.planets.iter().position(|p| p.x == fx && p.y == fy);
            crate::fleet::redirect(st, i, from, to);
        }
        Command::ScrapShips { planet, ref ships } => {
            check_planet(st, planet)?;
            let orbit = &st.empires[empire as usize].orbit[planet];
            if ships.len() != orbit.len()
                || ships.iter().all(|&c| c == 0)
                || ships.iter().zip(orbit).any(|(&c, &have)| c > have)
            {
                return err("no_ships");
            }
            let refund: i32 = ships
                .iter()
                .enumerate()
                .map(|(d, &c)| c as i32 * st.ship_cost(empire, d) / 4)
                .sum();
            let e = &mut st.empires[empire as usize];
            for (have, &c) in e.orbit[planet].iter_mut().zip(ships) {
                *have -= c;
            }
            e.reserve += refund;
        }
        Command::SendTransport { from, pop: 0, .. } => {
            own(st, empire, from)?;
            st.planets[from].trans_num = 0;
        }
        Command::SendTransport { from, to, pop } => {
            own(st, empire, from)?;
            check_planet(st, to)?;
            if from == to {
                return err("same_planet");
            }
            let p = &st.planets[from];
            if pop < 1 || pop > p.pop / 2 {
                return err("invalid_pop");
            }
            if !st.planets[to].habitable() {
                return err("not_habitable");
            }
            if let Some(o) = st.planets[to].owner {
                if o != empire
                    && matches!(
                        diplo::treaty(st, empire, o),
                        Treaty::NonAggression | Treaty::Alliance
                    )
                {
                    return err("treaty");
                }
            }
            let a = st.empires[empire as usize].abilities();
            if st.range_to(empire, to) > a.fuel_range {
                return err("out_of_range");
            }
            let p = &mut st.planets[from];
            p.trans_num = pop;
            p.trans_dest = to;
        }
        Command::SetResearch { alloc } => {
            let Some(a) = normalize(alloc.map(|x| x as i64)) else {
                return err("invalid_sliders");
            };
            st.empires[empire as usize].tech.alloc = a.map(|x| x as u32);
        }
        Command::SetResearchProject { field, tech } => {
            if field >= NUM_FIELDS
                || !tech::next_techs(&st.empires[empire as usize].tech, field).contains(&tech)
            {
                return err("invalid_tech");
            }
            if st.empires[empire as usize].tech.project[field] != tech {
                tech::start_project(st, empire, field, tech);
            }
        }
        Command::SaveDesign { slot, ref design } => {
            let mut d = design.clone();
            d.name = d.name.trim().chars().take(24).collect();
            d.update();
            if let Err(code) = design::validate(&d, &st.empires[empire as usize].tech) {
                return Err(CommandError { code });
            }
            let len = st.empires[empire as usize].designs.len();
            match slot {
                Some(i) if i < len => {
                    scrap_all(st, empire, i);
                    st.empires[empire as usize].designs[i] = d;
                }
                Some(_) => return err("invalid_design"),
                None if len >= MAX_DESIGNS => return err("too_many_designs"),
                None => st.empires[empire as usize].designs.push(d),
            }
        }
        Command::ScrapDesign { design } => {
            let len = st.empires[empire as usize].designs.len();
            if design >= len {
                return err("invalid_design");
            }
            if len <= 1 {
                return err("last_design");
            }
            scrap_all(st, empire, design);
            let e = &mut st.empires[empire as usize];
            e.designs.remove(design);
            for o in e.orbit.iter_mut() {
                o.remove(design);
                o.push(0);
            }
            for f in st.fleets.iter_mut().filter(|f| f.owner == empire) {
                f.ships.remove(design);
                f.ships.push(0);
            }
            for p in st.planets.iter_mut().filter(|p| p.owner == Some(empire)) {
                if p.build == design {
                    p.build = 0;
                } else if p.build > design && p.build != BUILD_STARGATE {
                    p.build -= 1;
                }
            }
        }
        Command::SetBombard { enabled } => st.empires[empire as usize].bombard = enabled,
        Command::SetSpying { target, tenths } => {
            if target as usize >= st.empires.len()
                || target == empire
                || !(0..=100).contains(&tenths)
            {
                return err("invalid_empire");
            }
            diplo::rel_mut(st, empire, target).spying = tenths;
        }
        Command::SetSpyMode { target, mode } => {
            if target as usize >= st.empires.len() || target == empire {
                return err("invalid_empire");
            }
            let r = diplo::rel_mut(st, empire, target);
            r.spymode = mode;
            r.spymode_next = mode;
        }
        Command::SetSecurity { tenths } => {
            if !(0..=200).contains(&tenths) {
                return err("invalid_sliders");
            }
            st.empires[empire as usize].security = tenths;
        }
        Command::SetTax { tenths } => {
            if !(0..=200).contains(&tenths) {
                return err("invalid_sliders");
            }
            st.empires[empire as usize].tax = tenths;
        }
        Command::TransferReserve { planet, bc } => {
            own(st, empire, planet)?;
            if bc <= 0 || bc > st.empires[empire as usize].reserve {
                return err("no_money");
            }
            st.empires[empire as usize].reserve -= bc;
            st.planets[planet].reserve += bc;
        }
        Command::AnswerMessage {
            index,
            accept,
            pick,
        } => {
            let reply = audience::answer(st, empire, index, accept, pick)
                .map_err(|code| CommandError { code })?;
            if let Some(r) = reply {
                st.pending.messages.insert(0, r);
            }
        }
        Command::StealTech { index, field } => {
            let ok = st.pending.spy.get(index).is_some_and(|(o, _)| *o == empire)
                && spy::steal(st, empire, index, field);
            if !ok {
                return err("invalid_tech");
            }
        }
        Command::Sabotage {
            index,
            planet,
            kind,
        } => {
            let ok = st.pending.spy.get(index).is_some_and(|(o, _)| *o == empire);
            if !ok || spy::do_sabotage(st, empire, index, planet, kind).is_none() {
                return err("invalid_planet");
            }
        }
        Command::StartBattle { .. } | Command::Battle { .. } => unreachable!(),
        Command::Vote { candidate } => {
            council::vote(st, empire, candidate).map_err(|code| CommandError { code })?
        }
        Command::AcceptCouncil { accept } => {
            council::accept(st, empire, accept).map_err(|code| CommandError { code })?
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_to_100() {
        assert_eq!(normalize([1, 1, 1]), Some([33, 33, 34]));
        assert_eq!(normalize([0, 50, 0]), Some([0, 100, 0]));
        assert_eq!(normalize([0, 0]), None);
        assert_eq!(normalize([-5, 10]), Some([0, 100]));
    }
}
