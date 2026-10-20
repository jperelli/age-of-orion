//! Orders an empire can give. Humans (through the UI) and the AI use the same commands.

use serde::{Deserialize, Serialize};

use crate::design::{self, ShipDesign, BUILD_STARGATE, MAX_DESIGNS};
use crate::state::*;
use crate::tech;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    /// Spending of a colony in percent (normalized to a total of 100).
    SetSliders {
        planet: PlanetId,
        sliders: [i32; NUM_SLIDERS],
    },
    /// Design built with the ship slider, or `BUILD_STARGATE` (6).
    SetBuild { planet: PlanetId, design: usize },
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
    RedirectFleet { fleet: u32, to: PlanetId },
    /// Scraps ships in orbit; a quarter of their cost goes to the reserve.
    ScrapShips { planet: PlanetId, ships: Vec<u32> },
    /// Moves colonists to another planet (own: reinforce, enemy: invade).
    SendTransport {
        from: PlanetId,
        to: PlanetId,
        pop: i32,
    },
    /// Research share per field in percent (normalized to a total of 100).
    SetResearch { alloc: [u32; NUM_FIELDS] },
    /// Chooses the tech researched in a field (an ongoing project loses its investment).
    SetResearchProject { field: usize, tech: u8 },
    /// Adds a design, or replaces design `slot` (scrapping its ships).
    SaveDesign {
        slot: Option<usize>,
        design: ShipDesign,
    },
    /// Removes a design and scraps all its ships.
    ScrapDesign { design: usize },
    /// Whether ships holding the orbit of enemy colonies bomb them.
    SetBombard { enabled: bool },
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

fn gate(st: &State, empire: EmpireId, from: PlanetId, to: PlanetId) -> bool {
    let (a, b) = (&st.planets[from], &st.planets[to]);
    a.stargate && b.stargate && a.owner == Some(empire) && b.owner == Some(empire)
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
            let speed = st.fleet_speed(empire, ships);
            let gate = gate(st, empire, from, to);
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
                gate,
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
            if st.fleets[i].gate {
                return err("in_stargate");
            }
            let ships = st.fleets[i].ships.clone();
            if st.range_to(empire, to) > fleet_range(st, empire, &ships) {
                return err("out_of_range");
            }
            st.fleets[i].dest = to;
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
            let a = st.empires[empire as usize].abilities();
            if st.range_to(empire, to) > a.fuel_range {
                return err("out_of_range");
            }
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
                speed: a.engine_warp,
            });
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
