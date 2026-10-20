//! Age of Orion game core: the rules of Master of Orion 1, ported to Rust from the
//! 1oom engine (GPLv2), without any user interface or I/O. The same crate runs natively
//! (CLI, tests) and as WebAssembly in the web and desktop builds.
//!
//! ```
//! use aoo_core::{Game, NewGameOptions};
//! let mut game = Game::new(&NewGameOptions::default()).unwrap();
//! game.end_turn();
//! assert_eq!(game.state().turn, 1);
//! ```

// Index loops over parallel tables (planets, empires, fields) read better than zips here.
#![allow(clippy::needless_range_loop)]

mod ai;
mod combat;
mod commands;
mod galaxy;
mod rng;
mod rules;
mod state;
mod turn;
mod types;
mod view;

pub use commands::{Command, CommandError};
pub use galaxy::{NewGameOptions, HOME_NAMES, STAR_NAMES};
pub use rng::Rng;
pub use rules::{Abilities, Design, Rules};
pub use state::*;
pub use types::*;
pub use view::*;

/// A game in progress.
#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    st: State,
}

impl Game {
    pub fn new(opts: &NewGameOptions) -> Result<Self, String> {
        galaxy::new_game(opts).map(|st| Game { st })
    }

    /// Restores a game from [`Game::save`] output.
    pub fn load(json: &str) -> Result<Self, String> {
        let st: State = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if st.version != SAVE_VERSION {
            return Err(format!("unsupported save version {}", st.version));
        }
        Ok(Game { st })
    }

    pub fn save(&self) -> String {
        serde_json::to_string(&self.st).expect("state serializes")
    }

    pub fn state(&self) -> &State {
        &self.st
    }

    /// The human player, if any.
    pub fn human(&self) -> Option<EmpireId> {
        self.st.empires.iter().position(|e| !e.ai).map(|i| i as u8)
    }

    pub fn command(&mut self, empire: EmpireId, cmd: &Command) -> Result<(), CommandError> {
        commands::apply(&mut self.st, empire, cmd)
    }

    /// Lets the AI empires give their orders, then processes the turn.
    pub fn end_turn(&mut self) {
        if self.st.winner.is_some() {
            return;
        }
        for i in 0..self.st.empires.len() {
            if self.st.empires[i].ai {
                ai::play(&mut self.st, i as u8);
            }
        }
        turn::process(&mut self.st);
    }

    pub fn view(&self, empire: EmpireId) -> View {
        view::view(&self.st, empire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(seed: u32) -> NewGameOptions {
        NewGameOptions {
            seed,
            ..Default::default()
        }
    }

    #[test]
    fn home_production_matches_moo1_start() {
        let g = Game::new(&opts(1)).unwrap();
        let home = g.state().empires[0].home;
        let p = &g.state().planets[home];
        // 30 factories + 50 pop * (1 * 3 + 50) / 100 = 56 BC, minus ship maintenance.
        let maint = g.state().empires[0].maintenance;
        assert_eq!(p.prod + maint, 56);
    }

    #[test]
    fn colony_grows_and_builds_factories() {
        let mut g = Game::new(&opts(2)).unwrap();
        let home = g.state().empires[0].home;
        g.command(
            0,
            &Command::SetSliders {
                planet: home,
                sliders: [0, 0, 70, 30, 0],
            },
        )
        .unwrap();
        let before = g.state().planets[home].clone();
        for _ in 0..5 {
            g.end_turn();
        }
        let after = &g.state().planets[home];
        assert!(after.factories > before.factories);
        assert!(after.pop > before.pop);
    }

    #[test]
    fn research_advances() {
        let mut g = Game::new(&opts(3)).unwrap();
        let home = g.state().empires[0].home;
        g.command(
            0,
            &Command::SetSliders {
                planet: home,
                sliders: [0, 0, 0, 0, 100],
            },
        )
        .unwrap();
        for _ in 0..10 {
            g.end_turn();
        }
        assert!(g.state().empires[0].tech.level.iter().any(|&l| l > 1));
        assert!(g
            .view(0)
            .events
            .iter()
            .all(|e| e.to.is_empty() || e.to.contains(&0)));
    }

    #[test]
    fn fleets_travel_and_colonize() {
        let mut g = Game::new(&opts(4)).unwrap();
        let home = g.state().empires[0].home;
        let target = g
            .view(0)
            .planets
            .iter()
            .filter(|p| p.colonizable && p.range <= 3)
            .min_by_key(|p| p.range)
            .map(|p| p.id)
            .expect("a colonizable neighbour");
        let mut ships = vec![0; 5];
        ships[4] = 1;
        g.command(
            0,
            &Command::SendFleet {
                from: home,
                to: target,
                ships,
            },
        )
        .unwrap();
        assert_eq!(g.state().fleets.len(), 1);
        for _ in 0..6 {
            g.end_turn();
        }
        assert_eq!(g.state().planets[target].owner, Some(0));
        assert!(g.state().planets[target].pop >= 2);
    }

    #[test]
    fn rejects_bad_commands() {
        let mut g = Game::new(&opts(5)).unwrap();
        let home1 = g.state().empires[1].home;
        let err = g
            .command(
                0,
                &Command::SetSliders {
                    planet: home1,
                    sliders: [20; 5],
                },
            )
            .unwrap_err();
        assert_eq!(err.code, "not_owner");
        let home = g.state().empires[0].home;
        let far = (0..g.state().planets.len())
            .max_by_key(|&p| g.state().parsecs(home, p))
            .unwrap();
        let err = g
            .command(
                0,
                &Command::SendFleet {
                    from: home,
                    to: far,
                    ships: vec![0, 1, 0, 0, 0],
                },
            )
            .unwrap_err();
        assert_eq!(err.code, "out_of_range");
    }

    #[test]
    fn save_and_load_round_trip() {
        let mut g = Game::new(&opts(6)).unwrap();
        for _ in 0..3 {
            g.end_turn();
        }
        let json = g.save();
        let mut h = Game::load(&json).unwrap();
        assert_eq!(g, h);
        g.end_turn();
        h.end_turn();
        assert_eq!(g, h, "deterministic after reload");
    }

    #[test]
    fn ai_games_finish() {
        let mut finished = 0;
        for seed in 1..=6 {
            let mut g = Game::new(&NewGameOptions {
                seed,
                players: 3,
                all_ai: true,
                ..Default::default()
            })
            .unwrap();
            for _ in 0..400 {
                g.end_turn();
                if g.state().winner.is_some() {
                    break;
                }
            }
            let st = g.state();
            assert!(st.planets.iter().filter(|p| p.owner.is_some()).count() > 3);
            if st.winner.is_some() {
                finished += 1;
            }
        }
        assert!(finished >= 1, "AI never wins a game in 400 turns");
    }
}
