//! WebAssembly API used by the web and desktop UI. Data crosses the boundary as JSON
//! strings (game state, views, commands) and byte arrays (LBX files, PNG assets).

use aoo_core::{Command, Game, NewGameOptions, Rules, ShipDesign, Talk};
use wasm_bindgen::prelude::*;

fn js_err(e: impl std::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Ship part tables (weapons, computers, shields, ... hulls) as JSON.
#[wasm_bindgen]
pub fn parts() -> String {
    Game::parts_json()
}

/// Default rules as JSON: the starting point for rule mods.
#[wasm_bindgen(js_name = defaultRules)]
pub fn default_rules() -> String {
    serde_json::to_string_pretty(&Rules::default()).expect("rules serialize")
}

#[wasm_bindgen]
pub struct WasmGame {
    game: Game,
    player: u8,
}

#[wasm_bindgen]
impl WasmGame {
    /// Starts a new game from `NewGameOptions` JSON (all fields optional).
    #[wasm_bindgen(constructor)]
    pub fn new(options_json: &str) -> Result<WasmGame, JsError> {
        let opts: NewGameOptions = serde_json::from_str(options_json).map_err(js_err)?;
        let game = Game::new(&opts).map_err(js_err)?;
        let player = game.human().unwrap_or(0);
        Ok(WasmGame { game, player })
    }

    pub fn load(save_json: &str) -> Result<WasmGame, JsError> {
        let game = Game::load(save_json).map_err(js_err)?;
        let player = game.human().unwrap_or(0);
        Ok(WasmGame { game, player })
    }

    pub fn save(&self) -> String {
        self.game.save()
    }

    #[wasm_bindgen(getter)]
    pub fn player(&self) -> u8 {
        self.player
    }

    /// What the player knows, as JSON (see `aoo_core::View`).
    pub fn view(&self) -> String {
        serde_json::to_string(&self.game.view(self.player)).expect("view serializes")
    }

    /// Applies a `Command` given as JSON. Throws with the error code on rejection.
    pub fn command(&mut self, command_json: &str) -> Result<(), JsError> {
        let cmd: Command = serde_json::from_str(command_json).map_err(js_err)?;
        self.game.command(self.player, &cmd).map_err(js_err)
    }

    /// Says something (`Talk`, JSON) in an audience with empire `with`; returns the `Reply`.
    pub fn talk(&mut self, with: u8, talk_json: &str) -> Result<String, JsError> {
        let t: Talk = serde_json::from_str(talk_json).map_err(js_err)?;
        let r = self.game.talk(self.player, with, &t).map_err(js_err)?;
        Ok(serde_json::to_string(&r).expect("reply serializes"))
    }

    /// Cost, space and combat figures of a `ShipDesign` (JSON) with the player's tech.
    #[wasm_bindgen(js_name = designStats)]
    pub fn design_stats(&self, design_json: &str) -> Result<String, JsError> {
        let d: ShipDesign = serde_json::from_str(design_json).map_err(js_err)?;
        Ok(
            serde_json::to_string(&self.game.design_stats(self.player, &d))
                .expect("stats serialize"),
        )
    }

    #[wasm_bindgen(js_name = endTurn)]
    pub fn end_turn(&mut self) {
        self.game.end_turn();
    }

    /// The stats screen's data (JSON): the player's empire, every empire once the game is over.
    pub fn stats(&self) -> String {
        serde_json::to_string(&self.game.stats(self.player)).expect("stats serialize")
    }

    /// Turns from (x, y) to planet `dest` at `speed` (1oom `game_calc_eta_ship`, or
    /// `game_calc_eta_trans` for transports): slower inside nebulae.
    pub fn eta(&self, x: i32, y: i32, dest: usize, speed: i32, transport: bool) -> u32 {
        let st = self.game.state();
        if dest >= st.planets.len() {
            return 0;
        }
        let p = &st.planets[dest];
        if transport {
            st.eta_trans(speed, x, y, p.x, p.y).max(0) as u32
        } else {
            st.eta(x, y, dest, speed)
        }
    }
}

/// Converts LBX files into the open asset tree: `add` every file, then `run`.
#[wasm_bindgen]
#[derive(Default)]
pub struct LbxImporter {
    files: Vec<(String, Vec<u8>)>,
    out: Vec<aoo_lbx::AssetFile>,
    manifest: String,
}

#[wasm_bindgen]
impl LbxImporter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> LbxImporter {
        LbxImporter::default()
    }

    pub fn add(&mut self, name: String, bytes: Vec<u8>) {
        self.files.push((name, bytes));
    }

    /// Converts the added files; returns the number of generated assets.
    pub fn run(&mut self) -> usize {
        let r = aoo_lbx::import(&self.files);
        self.files.clear();
        self.manifest = serde_json::to_string(&r.manifest).expect("manifest serializes");
        self.out = r.files;
        self.out.len()
    }

    pub fn manifest(&self) -> String {
        self.manifest.clone()
    }

    pub fn path(&self, i: usize) -> String {
        self.out.get(i).map(|f| f.path.clone()).unwrap_or_default()
    }

    pub fn bytes(&self, i: usize) -> Vec<u8> {
        self.out.get(i).map(|f| f.bytes.clone()).unwrap_or_default()
    }
}
