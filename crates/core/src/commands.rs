//! Orders an empire can give. Humans (through the UI) and the AI use the same commands.

use serde::{Deserialize, Serialize};

use crate::state::*;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    /// Spending of a colony in percent (normalized to a total of 100).
    SetSliders {
        planet: PlanetId,
        sliders: [i32; NUM_SLIDERS],
    },
    /// Design built with the ship slider.
    SetBuild { planet: PlanetId, design: usize },
    /// Sends ships from an orbit (counts per design) to another planet.
    SendFleet {
        from: PlanetId,
        to: PlanetId,
        ships: Vec<u32>,
    },
    /// Changes the destination of a fleet in flight.
    RedirectFleet { fleet: u32, to: PlanetId },
    /// Moves colonists to another planet (own: reinforce, enemy: invade, empty: settle).
    SendTransport {
        from: PlanetId,
        to: PlanetId,
        pop: i32,
    },
    /// Research share per field in percent (normalized to a total of 100).
    SetResearch { alloc: [u32; NUM_FIELDS] },
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
    let extra = ships
        .iter()
        .enumerate()
        .filter(|(_, &c)| c > 0)
        .map(|(d, _)| st.rules.designs[d].extra_range)
        .min()
        .unwrap_or(0);
    st.empires[empire as usize].abilities().fuel_range + extra
}

pub fn apply(st: &mut State, empire: EmpireId, cmd: &Command) -> Result<(), CommandError> {
    if st.winner.is_some() || !st.empires.get(empire as usize).is_some_and(|e| e.alive) {
        return err("game_over");
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
            if design >= st.rules.designs.len() {
                return err("invalid_design");
            }
            st.planets[planet].build = design;
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
            let speed = st.empires[empire as usize].abilities().speed;
            let o = &mut st.empires[empire as usize].orbit[from];
            for (have, &c) in o.iter_mut().zip(ships) {
                *have -= c;
            }
            let id = st.alloc_id();
            let (x, y) = (st.planets[from].x as f64, st.planets[from].y as f64);
            st.fleets.push(Fleet {
                id,
                owner: empire,
                x,
                y,
                dest: to,
                ships: ships.clone(),
                speed,
            });
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
            let ships = st.fleets[i].ships.clone();
            if st.range_to(empire, to) > fleet_range(st, empire, &ships) {
                return err("out_of_range");
            }
            st.fleets[i].dest = to;
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
            if st.range_to(empire, to) > st.empires[empire as usize].abilities().fuel_range {
                return err("out_of_range");
            }
            let speed = st.empires[empire as usize].abilities().speed;
            let (x, y) = (p.x as f64, p.y as f64);
            st.planets[from].pop -= pop;
            let id = st.alloc_id();
            st.transports.push(Transport {
                id,
                owner: empire,
                x,
                y,
                dest: to,
                pop,
                speed,
            });
        }
        Command::SetResearch { alloc } => {
            let Some(a) = normalize(alloc.map(|x| x as i64)) else {
                return err("invalid_sliders");
            };
            st.empires[empire as usize].tech.alloc = a.map(|x| x as u32);
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
