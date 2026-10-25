# Notes for contributors and coding agents

## Commands

- Rust: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
- WebAssembly bindings: `./scripts/build-wasm.sh` (needs `wasm32-unknown-unknown` and `wasm-bindgen-cli` at the `Cargo.lock` version); output goes to `web/src/wasm/` (git-ignored)
- Web: `cd web && npm ci && npm run build` (runs `tsc --noEmit` then Vite); `npm run dev` for a dev server
- Desktop: build the web first, then `cd desktop && npm ci && npx tauri build` (Linux needs `libwebkit2gtk-4.1-dev`). `desktop/src-tauri` is its own Cargo workspace on purpose.
- Balance check: `cargo run --release -p aoo-cli -- sim --seed 1 --size medium --players 4`
- 1oom parity (needs your own MOO1 LBX files, see `docs/HARNESS.md`): `tools/harness/build-1oom.sh`, `tools/harness/run-1oom.sh LBXDIR OUT YEARS`, then `cargo run --release -p aoo-harness -- lockstep OUT --oom target/1oom/src/1oom_cmdline --data LBXDIR`. Run it after touching turn, economy, fleet or AI rules; a change that was identical before should stay identical.

## Conventions

- The core is pure: no I/O, deterministic for a given state (all randomness through `State.rng`).
- A battle with a human player interrupts the turn (`State.battle`, `State.progress`); tests that end turns with a human use `Game::end_turn_auto`, which lets the computer fight those battles.
- The stats screen's numbers (`crates/core/src/stats.rs`) are counted from events as they are pushed (`State::push_event`) and sampled after each turn; a new event kind that matters to the player gets its count there, and something done without an event uses `stats::add`. Counting never changes the game.
- Everything crossing to the UI is JSON (`View`, `Command`, `Event` with `type` tags in snake_case).
- Every user-visible string is a text key (`themes/classic/text/en.json`, the base of every theme); other themes override in `themes/<id>/text/en.json`. Add every new key to the Spanish files too (`themes/classic/text/es.json`, `themes/age/text/es.json`); missing keys fall back to English.
- Every picture and sound is a file of a theme folder named by what the game asks for (`themes/<id>/images/ship/3.svg` is `ship/3`); the code draws no substitute art, a missing picture shows a placeholder. Classic's folder has every file the game uses: when the UI asks for a new picture or sound, add Classic's (`python3 tools/art/gen_svg.py`, `tools/sound/gen_music.py`, `tools/sound/gen_sfx.py`). `python3 tools/themes_check.py` lists what other themes lack.
- When porting from 1oom, name the source function in a doc comment (for example "1oom `game_turn_build_eco`"). Keep 1oom's integer types and order of operations (random numbers drawn in the same order, 16/32-bit wrapping where MOO1 overflows, list order of fleets): the harness compares them. 1oom always applies MOO1 v1.3a's fixes (`game_num_fix_bugs_1_3a`) and none of its optional ones.
- After renaming anything in a theme's texts (races, techs, words), run `python3 tools/gen_moo1_doc.py`: it regenerates the tables of `docs/MOO1.md` and `docs/MOO1.es.md`, which the main menu also shows.
- Never commit original MOO1 files or anything converted from them (`*.LBX` is git-ignored).
- Update `CHANGELOG.md` under `[Unreleased]`; releases are cut by pushing a `vX.Y.Z` tag that matches the version in `Cargo.toml`, `web/package.json`, `desktop/package.json` and `desktop/src-tauri/tauri.conf.json`.
