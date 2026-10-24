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
mod ai_diplo;
pub mod audience;
pub mod battle;
mod battle_ai;
mod combat;
mod commands;
pub mod council;
pub mod data;
pub mod data_types;
pub mod design;
pub mod diplo;
pub mod events;
mod fleet;
mod galaxy;
mod ground;
mod misc;
mod rng;
mod rules;
pub mod spy;
mod state;
pub mod tech;
pub mod techinfo;
mod turn;
mod types;
mod umath;
mod view;

pub use audience::{Reply, Talk};
pub use commands::{BattleMode, Command, CommandError};
pub use design::{DesignStats, ShipDesign};
pub use galaxy::{NewGameOptions, HOME_NAMES, SHIP_NAMES, STAR_NAMES};
pub use rng::{trace_start, trace_take, Rng, RngCall};
pub use rules::{Abilities, Rules};
pub use state::*;
pub use types::*;
pub use view::*;

/// 1oom `game_start`: values derived from the state, recomputed after loading.
fn start(st: &mut State) {
    st.init_star_dist();
    // 1oom keeps `colonist_oper_factories` in its saves: derive it before production.
    tech::update_tech_util(st);
    misc::update_production(st);
    tech::update_tech_util(st);
    for i in 0..st.empires.len() {
        misc::update_eco_on_waste(st, i as u8, false);
        misc::update_seen_by_orbit(st, i);
    }
    misc::update_within_range(st);
    misc::update_visibility(st);
    misc::update_have_reserve_fuel(st);
}

/// A game in progress.
#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    st: State,
}

impl Game {
    pub fn new(opts: &NewGameOptions) -> Result<Self, String> {
        galaxy::new_game(opts).map(|mut st| {
            start(&mut st);
            Game { st }
        })
    }

    /// Wraps a state from elsewhere (the 1oom save importer), as 1oom does after loading.
    pub fn from_state(mut st: State) -> Self {
        start(&mut st);
        Game { st }
    }

    /// Restores a game from [`Game::save`] output.
    pub fn load(json: &str) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let version = value.get("version").and_then(|v| v.as_u64()).unwrap_or(0);
        if version != SAVE_VERSION as u64 {
            return Err(format!(
                "save format {version} is from another version of the game (this one reads format {SAVE_VERSION})"
            ));
        }
        let mut st: State = serde_json::from_value(value).map_err(|e| e.to_string())?;
        start(&mut st);
        Ok(Game { st })
    }

    pub fn save(&self) -> String {
        serde_json::to_string(&self.st).expect("state serializes")
    }

    pub fn state(&self) -> &State {
        &self.st
    }

    #[cfg(test)]
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.st
    }

    /// The human `empire` says something in an audience with the computer empire `with`.
    pub fn talk(
        &mut self,
        empire: EmpireId,
        with: EmpireId,
        talk: &Talk,
    ) -> Result<Reply, CommandError> {
        if self.st.winner.is_some() {
            return Err(CommandError { code: "game_over" });
        }
        if self.st.progress.is_some() {
            return Err(CommandError {
                code: "battle_in_progress",
            });
        }
        audience::talk(&mut self.st, empire, with, talk).map_err(|code| CommandError { code })
    }

    /// The human player, if any.
    pub fn human(&self) -> Option<EmpireId> {
        self.st.empires.iter().position(|e| !e.ai).map(|i| i as u8)
    }

    pub fn command(&mut self, empire: EmpireId, cmd: &Command) -> Result<(), CommandError> {
        commands::apply(&mut self.st, empire, cmd)
    }

    /// Lets the AI empires give their orders, then processes the turn.
    ///
    /// When a battle with a human interrupts the turn, `view().battle` shows it; fight it
    /// with `Command::StartBattle` and `Command::Battle`, then call `end_turn` again to
    /// finish the turn.
    pub fn end_turn(&mut self) {
        if self.st.winner.is_some() {
            return;
        }
        if self.st.battle.as_ref().is_some_and(|b| b.winner.is_none()) {
            return;
        }
        turn::process(&mut self.st);
    }

    /// Ends the turn and lets the computer fight the human players' battles (tools, tests).
    pub fn end_turn_auto(&mut self) {
        self.end_turn();
        for _ in 0..500 {
            if !self.in_battle() {
                break;
            }
            let human = self
                .st
                .battle
                .as_ref()
                .filter(|b| b.winner.is_none())
                .and_then(|b| b.s.iter().find(|s| s.human).map(|s| s.party as u8));
            if let Some(h) = human {
                let _ = self.command(
                    h,
                    &Command::StartBattle {
                        mode: BattleMode::Auto,
                    },
                );
            }
            self.end_turn();
        }
    }

    /// Whether the turn waits for a human battle.
    pub fn in_battle(&self) -> bool {
        self.st.progress.is_some()
    }

    pub fn view(&self, empire: EmpireId) -> View {
        view::view(&self.st, empire)
    }

    /// All ship part tables, for the design screen.
    pub fn parts_json() -> String {
        #[derive(serde::Serialize)]
        struct Parts {
            weapons: &'static [data_types::Weapon],
            comps: &'static [data_types::Comp],
            jammers: &'static [data_types::Comp],
            engines: &'static [data_types::Engine],
            armor: &'static [data_types::Armor],
            shields: &'static [data_types::Shield],
            specials: &'static [data_types::Special],
            hulls: &'static [data_types::Hull],
            /// Tech keys per field and number ("" = no such tech).
            techs: Vec<Vec<&'static str>>,
        }
        serde_json::to_string(&Parts {
            weapons: &data::WEAPONS,
            comps: &data::COMPS,
            jammers: &data::JAMMERS,
            engines: &data::ENGINES,
            armor: &data::ARMOR,
            shields: &data::SHIELDS,
            specials: &data::SPECIALS,
            hulls: &data::HULLS,
            techs: data::TECHS.iter().map(|f| f.to_vec()).collect(),
        })
        .expect("parts serialize")
    }

    /// Cost, space and combat figures of a design with an empire's current tech.
    pub fn design_stats(&self, empire: EmpireId, design: &ShipDesign) -> DesignStats {
        design::stats(design, &self.st.empires[empire as usize].tech)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle(st: &mut State, p: PlanetId, owner: EmpireId, pop: i32) {
        let pl = &mut st.planets[p];
        pl.owner = Some(owner);
        pl.pop = pop.min(pl.max_pop);
        st.empires[owner as usize].explored[p] = true;
    }

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
        // 30 factories + 50 pop * (1 * 3 + 50) / 100 = 56 BC before maintenance.
        assert_eq!(p.total_prod, 56);
        assert!(p.prod < 56);
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
            g.end_turn_auto();
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
        for _ in 0..40 {
            choose_research(&mut g, 0);
            g.end_turn_auto();
        }
        assert!(g.state().empires[0].tech.level.iter().any(|&l| l > 1));
        assert!(g.state().empires[0]
            .tech
            .completed
            .iter()
            .any(|c| c.len() > 1));
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
        let st = g.state();
        let target = (0..st.planets.len())
            .filter(|&p| turn::can_colonize(st, 0, p) && st.range_to(0, p) <= 3)
            .min_by_key(|&p| st.range_to(0, p))
            .expect("a colonizable neighbour");
        let mut ships = vec![0; 6];
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
            g.end_turn_auto();
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
                    ships: vec![0, 1, 0, 0, 0, 0],
                },
            )
            .unwrap_err();
        assert_eq!(err.code, "out_of_range");
    }

    #[test]
    fn save_and_load_round_trip() {
        let mut g = Game::new(&opts(6)).unwrap();
        for _ in 0..3 {
            g.end_turn_auto();
        }
        let json = g.save();
        let mut h = Game::load(&json).unwrap();
        assert_eq!(g, h);
        g.end_turn_auto();
        h.end_turn_auto();
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
                g.end_turn_auto();
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

    fn own_colony(g: &mut Game, empire: u8) -> usize {
        let home = g.st.empires[empire as usize].home;
        let p = (0..g.st.planets.len())
            .filter(|&p| g.st.planets[p].owner.is_none() && p != g.st.orion)
            .min_by_key(|&p| g.st.parsecs(home, p))
            .unwrap();
        let pl = &mut g.st.planets[p];
        pl.owner = Some(empire);
        pl.pop = 20;
        g.st.empires[empire as usize].explored[p] = true;
        p
    }

    #[test]
    fn transports_leave_at_the_end_of_the_turn() {
        let mut g = Game::new(&opts(12)).unwrap();
        let home = g.st.empires[0].home;
        let to = own_colony(&mut g, 0);
        g.command(
            0,
            &Command::SendTransport {
                from: home,
                to,
                pop: 10,
            },
        )
        .unwrap();
        let colony = |g: &Game| g.view(0).planets[home].colony.clone().unwrap();
        assert_eq!(colony(&g).transport, Some((10, to)));
        g.command(
            0,
            &Command::SendTransport {
                from: home,
                to: home,
                pop: 0,
            },
        )
        .unwrap();
        assert_eq!(colony(&g).transport, None);
        g.command(
            0,
            &Command::SendTransport {
                from: home,
                to,
                pop: 10,
            },
        )
        .unwrap();
        let before = g.st.planets[home].pop;
        g.end_turn_auto();
        assert!(g.st.planets[home].pop < before + 5);
        assert!(
            g.st.transports.iter().any(|t| t.owner == 0 && t.pop == 10)
                || g.st.planets[to].pop >= 20
        );
        assert_eq!(g.st.planets[home].trans_num, 0);
    }

    #[test]
    fn stargates_move_fleets_in_one_turn() {
        let mut g = Game::new(&opts(11)).unwrap();
        let home = g.st.empires[0].home;
        let far = own_colony(&mut g, 0);
        g.st.empires[0].tech.add(FIELD_PROPULSION, 27);
        tech::update_tech_util(&mut g.st);
        assert!(g
            .command(
                0,
                &Command::SetBuild {
                    planet: home,
                    design: design::BUILD_STARGATE
                }
            )
            .is_ok());
        g.st.planets[home].bc_to_ship = 5000;
        g.st.planets[home].sliders = [100, 0, 0, 0, 0];
        g.end_turn_auto();
        assert!(g.st.planets[home].stargate);
        g.st.planets[far].stargate = true;
        let mut ships = vec![0; 6];
        ships[0] = 1;
        g.command(
            0,
            &Command::SendFleet {
                from: home,
                to: far,
                ships,
            },
        )
        .unwrap();
        assert!(g
            .st
            .fleets
            .iter()
            .any(|f| f.owner == 0 && f.speed == SPEED_STARGATE));
        g.end_turn_auto();
        assert_eq!(g.st.empires[0].orbit[far][0], 1);
    }

    #[test]
    fn rally_points_send_new_ships() {
        let mut g = Game::new(&opts(12)).unwrap();
        let home = g.st.empires[0].home;
        let col = own_colony(&mut g, 0);
        g.command(
            0,
            &Command::SetRally {
                planet: home,
                to: Some(col),
            },
        )
        .unwrap();
        g.command(
            0,
            &Command::SetBuild {
                planet: home,
                design: 1,
            },
        )
        .unwrap();
        g.command(
            0,
            &Command::SetSliders {
                planet: home,
                sliders: [100, 0, 0, 0, 0],
            },
        )
        .unwrap();
        g.end_turn_auto();
        assert!(g
            .st
            .fleets
            .iter()
            .any(|f| f.owner == 0 && f.dest == col && f.ships[1] > 0));
    }

    #[test]
    fn designs_can_be_saved_and_scrapped() {
        let mut g = Game::new(&opts(13)).unwrap();
        let mut d = g.st.empires[0].designs[1].clone();
        d.name = "Lancer".into();
        d.hull = design::HULL_MEDIUM;
        d.weapon_counts[0] = 2;
        assert_eq!(g.design_stats(0, &d).error, None);
        g.command(
            0,
            &Command::SaveDesign {
                slot: None,
                design: d.clone(),
            },
        )
        .unwrap();
        assert_eq!(g.st.empires[0].designs.len(), 6);
        assert_eq!(
            g.command(
                0,
                &Command::SaveDesign {
                    slot: None,
                    design: d.clone()
                }
            )
            .unwrap_err()
            .code,
            "too_many_designs"
        );
        let home = g.st.empires[0].home;
        assert_eq!(g.st.empires[0].orbit[home][1], 2);
        g.command(0, &Command::ScrapDesign { design: 1 }).unwrap();
        assert_eq!(g.st.empires[0].designs.len(), 5);
        assert_eq!(g.st.empires[0].designs[4].name, "Lancer");
        assert_eq!(g.st.empires[0].orbit[home][1], 0, "fighters were scrapped");
        assert!(g.st.empires[0].reserve > 0);
        d.weapon_counts[0] = 60;
        assert_eq!(
            g.command(
                0,
                &Command::SaveDesign {
                    slot: Some(0),
                    design: d
                }
            )
            .unwrap_err()
            .code,
            "no_space"
        );
    }

    /// What the new-tech dialogs do: the first choice of every field without a project.
    fn choose_research(g: &mut Game, empire: EmpireId) {
        for (field, f) in g.view(empire).tech.iter().enumerate() {
            if f.project == 0 {
                if let Some(c) = f.choices.first() {
                    g.command(
                        empire,
                        &Command::SetResearchProject {
                            field,
                            tech: c.tech,
                        },
                    )
                    .unwrap();
                }
            }
        }
    }

    #[test]
    fn research_projects_follow_the_list() {
        let mut g = Game::new(&opts(14)).unwrap();
        let v = g.view(0);
        let f = &v.tech[FIELD_WEAPON];
        // As in MOO1, a human starts without projects and chooses them.
        assert!(f.project == 0 && f.choices.len() > 1);
        let first = f.choices[0].tech;
        let other = f.choices[1].tech;
        g.command(
            0,
            &Command::SetResearchProject {
                field: FIELD_WEAPON,
                tech: first,
            },
        )
        .unwrap();
        // A project in progress is locked until it is discovered.
        assert_eq!(
            g.command(
                0,
                &Command::SetResearchProject {
                    field: FIELD_WEAPON,
                    tech: other,
                },
            )
            .unwrap_err()
            .code,
            "project_locked"
        );
        tech::grant(&mut g.st, 0, FIELD_WEAPON, first, tech::Source::Research);
        assert_eq!(g.st.empires[0].tech.project[FIELD_WEAPON], 0);
        g.command(
            0,
            &Command::SetResearchProject {
                field: FIELD_WEAPON,
                tech: other,
            },
        )
        .unwrap();
        assert_eq!(g.st.empires[0].tech.project[FIELD_WEAPON], other);
        assert_eq!(
            g.command(
                0,
                &Command::SetResearchProject {
                    field: FIELD_WEAPON,
                    tech: 49
                }
            )
            .unwrap_err()
            .code,
            "invalid_tech"
        );
        tech::grant(&mut g.st, 0, FIELD_WEAPON, other, tech::Source::Research);
        assert_eq!(g.st.empires[0].tech.project[FIELD_WEAPON], 0);
        assert!(g.st.empires[0].tech.has(FIELD_WEAPON, other));
    }

    #[test]
    fn soil_enrichment_grows_planets() {
        let mut g = Game::new(&opts(15)).unwrap();
        let home = g.st.empires[0].home;
        g.st.empires[0].tech.add(FIELD_PLANETOLOGY, 16);
        let before = g.st.planets[home].base_max_pop;
        g.command(
            0,
            &Command::SetSliders {
                planet: home,
                sliders: [0, 0, 0, 100, 0],
            },
        )
        .unwrap();
        for _ in 0..6 {
            g.end_turn_auto();
        }
        let p = &g.st.planets[home];
        assert_eq!(p.growth, Growth::Fertile);
        assert!(p.base_max_pop > before);
    }

    #[test]
    fn nebulae_slow_ships_and_block_shields() {
        let g = Game::new(&NewGameOptions {
            seed: 3,
            galaxy_size: GalaxySize::Huge,
            ..Default::default()
        })
        .unwrap();
        assert!(!g.st.nebulae.is_empty());
        let n = &g.st.nebulae[0].rects[0];
        let (x, y) = ((n[0] + n[2]) / 2, (n[1] + n[3]) / 2);
        assert!(g.st.in_nebula(x, y));
        for p in g.st.planets.iter().filter(|p| p.nebula) {
            assert!(g.st.in_nebula(p.x, p.y));
        }
    }

    #[test]
    fn bombers_bomb_undefended_colonies() {
        let mut g = Game::new(&opts(16)).unwrap();
        let target = g.st.empires[1].home;
        g.st.empires[0].orbit[target][3] = 20; // bombers
        let pop = g.st.planets[target].pop;
        combat::bomb(&mut g.st);
        assert!(g.st.planets[target].pop < pop);
        assert!(g
            .st
            .events
            .iter()
            .any(|e| matches!(e.kind, EventKind::Bombed { .. })));
    }

    #[test]
    fn old_saves_are_rejected_clearly() {
        let err = Game::load(r#"{"version": 1, "planets": []}"#)
            .err()
            .unwrap();
        assert!(err.contains("format 1"), "{err}");
    }

    #[test]
    fn computers_talk_to_the_human() {
        let mut g = Game::new(&NewGameOptions {
            seed: 31,
            players: 5,
            galaxy_size: GalaxySize::Small,
            ..Default::default()
        })
        .unwrap();
        let mut kinds = std::collections::BTreeSet::new();
        for _ in 0..150 {
            g.end_turn_auto();
            for (_, m) in g.view(0).messages {
                kinds.insert(m.kind);
            }
            if g.state().winner.is_some() {
                break;
            }
        }
        assert!(
            kinds.iter().any(|&k| (14..=19).contains(&k)),
            "first contact greeting: {kinds:?}"
        );
    }

    #[test]
    fn treaties_stop_battles() {
        let mut g = Game::new(&opts(41)).unwrap();
        let target = g.st.empires[1].home;
        g.st.empires[0].orbit[target][1] = 10;
        diplo::set_treaty(&mut g.st, 0, 1, diplo::Treaty::Alliance);
        let bases = g.st.planets[target].bases;
        combat::battles(&mut g.st);
        assert_eq!(g.st.empires[0].orbit[target][1], 10, "allies do not fight");
        assert_eq!(g.st.planets[target].bases, bases);
        diplo::set_treaty(&mut g.st, 0, 1, diplo::Treaty::War);
        g.st.planets[target].bases = 3;
        assert!(combat::battles(&mut g.st), "the human fights on the grid");
        g.command(
            0,
            &Command::StartBattle {
                mode: BattleMode::Auto,
            },
        )
        .unwrap();
        assert!(g.st.battle.as_ref().unwrap().winner.is_some());
        assert!(g
            .st
            .events
            .iter()
            .any(|e| matches!(e.kind, EventKind::Battle { .. })));
    }

    #[test]
    fn council_elects_with_two_thirds() {
        let mut g = Game::new(&NewGameOptions {
            seed: 9,
            players: 4,
            all_ai: true,
            ..Default::default()
        })
        .unwrap();
        for p in 0..g.st.planets.len() {
            if g.st.planets[p].owner.is_none() && g.st.planets[p].habitable() {
                settle(&mut g.st, p, (p % 4) as u8, 10);
            }
        }
        council::convene(&mut g.st);
        assert!(g.st.council.held);
        assert!(
            g.st.council.pending.is_none(),
            "all-AI councils resolve at once"
        );
        assert!(g
            .st
            .events
            .iter()
            .any(|e| matches!(e.kind, EventKind::CouncilResult { .. })));
    }

    #[test]
    fn spies_steal_for_humans() {
        let mut g = Game::new(&opts(42)).unwrap();
        for n in [2u8, 3, 4, 5] {
            g.st.empires[1].tech.add(FIELD_WEAPON, n);
            g.st.empires[1].tech.add(FIELD_COMPUTER, n);
        }
        g.st.empires[1].tech.level = [1, 1, 1, 1, 1, 40];
        let r = diplo::rel_mut(&mut g.st, 0, 1);
        r.spies = 10;
        r.spymode = diplo::SpyMode::Espionage;
        let mut stolen = false;
        for _ in 0..60 {
            spy::turn(&mut g.st);
            if let Some(i) = g.st.pending.spy.iter().position(|(o, _)| *o == 0) {
                let spy::SpyChance::Steal { techs, .. } = g.st.pending.spy[i].1.clone() else {
                    continue;
                };
                let (f, t) = techs[0];
                g.command(
                    0,
                    &Command::StealTech {
                        index: i,
                        field: f as usize,
                    },
                )
                .unwrap();
                assert!(g.st.empires[0].tech.has(f as usize, t));
                stolen = true;
                break;
            }
            diplo::rel_mut(&mut g.st, 0, 1).spies = 10;
        }
        assert!(stolen);
    }

    #[test]
    fn council_waits_for_the_human_vote() {
        let mut g = Game::new(&NewGameOptions {
            seed: 9,
            players: 4,
            ..Default::default()
        })
        .unwrap();
        for p in 0..g.st.planets.len() {
            if g.st.planets[p].owner.is_none() && g.st.planets[p].habitable() {
                settle(&mut g.st, p, (p % 4) as u8, 30);
            }
        }
        assert!(council::due(&g.st));
        council::convene(&mut g.st);
        let el = g.st.council.pending.clone().expect("waiting for the human");
        assert!(!el.has_voted[0]);
        let c = el.candidates[0];
        g.command(0, &Command::Vote { candidate: Some(c) }).unwrap();
        let el = g.st.council.pending.clone();
        assert!(el.as_ref().is_none_or(|e| e.awaiting_acceptance));
        assert_eq!(g.st.council.voted[0], Some(c));
        if el.is_some() {
            g.command(0, &Command::AcceptCouncil { accept: false })
                .unwrap();
            assert!(g.st.council.final_war);
            assert!(g.st.empires[0]
                .rel
                .iter()
                .skip(1)
                .all(|r| r.treaty == diplo::Treaty::FinalWar));
        }
    }

    /// A human who answers everything at random and talks to everyone: no panics, and the
    /// state survives a save round trip.
    #[test]
    fn random_diplomacy_does_not_break() {
        let mut g = Game::new(&NewGameOptions {
            seed: 77,
            players: 6,
            galaxy_size: GalaxySize::Small,
            ..Default::default()
        })
        .unwrap();
        let mut r = rng::Rng::new(5);
        for turn in 0..160 {
            for n in 1..6u8 {
                diplo::rel_mut(&mut g.st, 0, n).contact |= turn > 20;
                let talks = [
                    Talk::ProposeNap,
                    Talk::ProposeAlliance,
                    Talk::ProposePeace,
                    Talk::Threaten,
                    Talk::TributeBc { bc: 25 },
                    Talk::TechOffers,
                    Talk::TributeTechs,
                    Talk::AcceptSweetener,
                    Talk::DeclareWarOn {
                        target: (n % 5) + 1,
                    },
                ];
                let t = &talks[r.below(talks.len() as u16) as usize];
                if let Ok(reply) = g.talk(0, n, t) {
                    if let Some(d) = reply.tech_deals.first() {
                        let _ = g.talk(
                            0,
                            n,
                            &Talk::ExchangeTech {
                                want: d.want,
                                give: d.give[0],
                            },
                        );
                    }
                }
            }
            g.command(
                0,
                &Command::SetSpying {
                    target: 1,
                    tenths: 50,
                },
            )
            .unwrap();
            g.command(
                0,
                &Command::SetSpyMode {
                    target: 1,
                    mode: diplo::SpyMode::Sabotage,
                },
            )
            .ok();
            while let Some((i, _)) = g.view(0).messages.first().cloned() {
                let accept = r.below(2) == 0;
                if g.command(
                    0,
                    &Command::AnswerMessage {
                        index: i,
                        accept,
                        pick: None,
                    },
                )
                .is_err()
                {
                    g.command(
                        0,
                        &Command::AnswerMessage {
                            index: i,
                            accept: false,
                            pick: None,
                        },
                    )
                    .unwrap();
                }
            }
            if g.view(0).election.is_some() {
                let _ = g.command(0, &Command::Vote { candidate: None });
                let _ = g.command(0, &Command::AcceptCouncil { accept: false });
            }
            g.end_turn_auto();
            if g.state().winner.is_some() || !g.state().empires[0].alive {
                break;
            }
        }
        assert!(g.state().turn > 50, "turn {}", g.state().turn);
        // Loading recomputes derived values as 1oom does; after that saves are stable.
        let again = Game::load(&g.save()).unwrap();
        let save = again.save();
        assert_eq!(Game::load(&save).unwrap().save(), save);
    }
    #[test]
    fn slider_locks_are_kept() {
        let mut g = Game::new(&NewGameOptions::default()).unwrap();
        let home = g
            .view(0)
            .planets
            .iter()
            .position(|p| p.colony.is_some())
            .unwrap();
        g.command(
            0,
            &Command::LockSlider {
                planet: home,
                slider: 2,
                locked: true,
            },
        )
        .unwrap();
        g.command(
            0,
            &Command::LockResearch {
                field: 5,
                locked: true,
            },
        )
        .unwrap();
        let v = g.view(0);
        assert!(v.planets[home].colony.as_ref().unwrap().locks[2]);
        assert!(v.tech[5].locked && !v.tech[0].locked);
        assert!(g
            .command(
                0,
                &Command::LockSlider {
                    planet: home,
                    slider: 9,
                    locked: true
                }
            )
            .is_err());
        // What a tech gives is part of the research view.
        assert!(v
            .tech
            .iter()
            .all(|f| f.choices.iter().all(|c| !c.effects.is_empty())));
    }
    #[test]
    fn redirecting_needs_hyperspace_communications() {
        let mut g = Game::new(&opts(4)).unwrap();
        let home = g.state().empires[0].home;
        let st = g.state();
        let mut scout = vec![0; 6];
        scout[0] = 1;
        let range = crate::commands::fleet_range(st, 0, &scout);
        let mut near: Vec<usize> = (0..st.planets.len())
            .filter(|&p| p != home && st.range_to(0, p) <= range)
            .collect();
        near.sort_by_key(|&p| (std::cmp::Reverse(st.parsecs(home, p)), p));
        let (a, b) = (near[0], near[1]);
        let mut ships = vec![0; 6];
        ships[0] = 1;
        g.command(
            0,
            &Command::SendFleet {
                from: home,
                to: a,
                ships,
            },
        )
        .unwrap();
        let id = g.state().fleets[0].id;
        // Still at its planet: it can change destination.
        assert!(g
            .view(0)
            .fleets
            .iter()
            .any(|f| f.id == Some(id) && f.redirect));
        g.command(0, &Command::RedirectFleet { fleet: id, to: b })
            .unwrap();
        g.end_turn_auto();
        let Some(f) = g.state().fleets.iter().find(|f| f.id == id) else {
            return;
        };
        if g.state().planets.iter().any(|p| p.x == f.x && p.y == f.y) {
            return;
        }
        // On its way: only with Hyperspace Communications (Computers 34).
        let err = g
            .command(0, &Command::RedirectFleet { fleet: id, to: a })
            .unwrap_err();
        assert_eq!(err.code, "no_hyperspace_comm");
        assert!(!g
            .view(0)
            .fleets
            .iter()
            .any(|f| f.id == Some(id) && f.redirect));
        g.state_mut().empires[0].ab.hyperspace_comm = true;
        g.command(0, &Command::RedirectFleet { fleet: id, to: a })
            .unwrap();
        assert_eq!(
            g.state().fleets.iter().find(|f| f.id == id).unwrap().dest,
            a
        );
    }
}
