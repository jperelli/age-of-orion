# Notes for contributors and coding agents

## Commands

- Rust: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
- WebAssembly bindings: `./scripts/build-wasm.sh` (needs `wasm32-unknown-unknown` and `wasm-bindgen-cli` at the `Cargo.lock` version); output goes to `web/src/wasm/` (git-ignored)
- Web: `cd web && npm ci && npm run build` (runs `tsc --noEmit` then Vite); `npm run dev` for a dev server
- Desktop: build the web first, then `cd desktop && npm ci && npx tauri build` (Linux needs `libwebkit2gtk-4.1-dev`). `desktop/src-tauri` is its own Cargo workspace on purpose.
- Balance check: `cargo run --release -p aoo-cli -- sim --seed 1 --size medium --players 4`

## Conventions

- The core is pure: no I/O, deterministic for a given state (all randomness through `State.rng`).
- A battle with a human player interrupts the turn (`State.battle`, `State.progress`); tests that end turns with a human use `Game::end_turn_auto`, which lets the computer fight those battles.
- Everything crossing to the UI is JSON (`View`, `Command`, `Event` with `type` tags in snake_case).
- Every user-visible string is a text key (`web/public/text/en.json`); themes override in `web/public/themes/<id>/text/en.json`.
- When porting from 1oom, name the source function in a doc comment (for example "1oom `game_turn_build_eco`").
- Never commit original MOO1 files or anything converted from them (`*.LBX` is git-ignored).
- Update `CHANGELOG.md` under `[Unreleased]`; releases are cut by pushing a `vX.Y.Z` tag that matches the version in `Cargo.toml`, `web/package.json`, `desktop/package.json` and `desktop/src-tauri/tauri.conf.json`.
