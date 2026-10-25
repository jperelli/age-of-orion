# Age of Orion

A 4X strategy game on the rules of **Master of Orion** (SimTex, 1993), with an Age of
Empires look: the medieval empires of history (the Mongols, Byzantium, the Song, Venice…)
with their cities and armies on a hand-drawn world map. The rules are a
Rust port of [1oom](https://github.com/1oom-fork/1oom), the open-source engine of MOO1, compiled
to **WebAssembly** so the same game runs in the browser and as a desktop app.

- **Play in the browser:** https://jperelli.com/age-of-orion/
- **Download** (Windows, macOS, Linux, plus the web build and the CLI):
  [latest release](https://github.com/jperelli/age-of-orion/releases/latest)

No original game files are needed to play. If you own Master of Orion, you can import your
own `.LBX` files: they are converted to PNG/JSON inside your browser and never leave your
device, and the **Classic** theme then shows the original planet pictures and race portraits.

**How close is it to Master of Orion?** [docs/MOO1.md](docs/MOO1.md) lists what is exactly as
in MOO1 and what differs, with tables of the Age theme's empires, words and technologies
([en español](docs/MOO1.es.md)). The game shows the same text from its main menu.

## Features

- Galaxy generation, planets, specials and the random number generator ported from 1oom
  (`game_new.c`, `rnd.c`); home planets are placed in a simpler way, so the same seed does
  not yet give 1oom's map.
- MOO1 economy: the five colony sliders (ship, defense, industry, ecology, technology),
  factories and refits, pollution and cleanup, terraforming, population growth, ship and
  base maintenance (`game_turn.c`, `game_misc.c`).
- MOO1 research (`game_tech.c`): every empire gets its own list of the 240 MOO1 techs,
  you choose one project per field, investment earns interest and breakthroughs are
  random once it exceeds the cost. Completed techs unlock everything else (robotic
  controls, factory cost, fuel range, colonizable planet types, terraforming, soil
  enrichment, planetary shields, stargates...).
- Ship design (`game_shiptech.c`, `game_design.c`): 4 hulls, 64 weapons, computers,
  shields, ECM, armor, engines and 30 specials with the MOO1 cost, space and power
  formulas, including miniaturization as your tech levels rise. Up to six designs.
- Ten races with their bonuses and research costs, five difficulty levels, 2 to 6 empires,
  four galaxy sizes, nebulae.
- Fleets with fuel range and speed, colonization with colony bases for hostile planets,
  rally points, stargates, scrapping, missile bases, orbital bombing, transports and ground
  invasions, the Guardian of Orion.
- Tactical battles on the MOO1 battle grid (`game_battle_human.c`): move your stacks, fire
  beams and missiles, use stasis fields, pulsars, black holes, teleporters and the other
  specials, retreat, or let the computer fight; computer opponents fight with the classic
  battle AI.
- MOO1 diplomacy (`game_diplo.c`, `game_audience.c`): contact, relations and moods,
  non-aggression pacts, alliances, wars, trade agreements, tribute, technology exchange,
  threats, and computer rulers who make their own proposals, demands and declarations.
- Espionage and sabotage (`game_spy.c`), security and taxes.
- Random events (`game_event.c`): plagues, novas, comets, pirates, rebellions, coups, and
  the Space Crystal and Space Amoeba roaming the map.
- The Galactic Council (`game_election.c`): win by conquest or by being elected High Master.
- The computer players of MOO1, ported line by line from 1oom's classic AI
  (`game_ai_classic.c`): scouting, colony ships, attack and defence fleets, invasions,
  production sliders, ship design and scrapping, research focus, taxes, spying, diplomacy and
  votes, with a choice between MOO1's behaviour ("Classic") and 1oom's fixes ("Classic+").
- Checked against the C original: a comparison harness plays the same turns in 1oom and in
  Age of Orion and diffs the game state and every random number
  ([docs/HARNESS.md](docs/HARNESS.md)).
- Two themes: **Age** (medieval world map, the default) and **Classic** (starfield, with
  your original MOO1 art when imported). Each theme is a folder of `themes/` with all its
  pictures, music, sounds and texts; Classic's has every one the game uses (drawn as SVG,
  with its own music), so a new theme can start as a copy of it. Age has its own medieval
  music and sounds of steel, bows and stone. Every text and picture can
  also be replaced by a mod.
- The MOO1 soundtrack from your own MUSIC.LBX, converted to MIDI and played by a built-in
  synthesizer; MOO1's sound effects (SOUNDFX.LBX) and fonts (FONTS.LBX, converted to
  TrueType); tech descriptions from RESEARCH.LBX.
- The ground battle screen and the GNN newscast, with MOO1's landing and newscast art when
  imported.
- English and Spanish.
- Autosave every turn, save/load to a file, keyboard shortcuts (Enter ends the turn).

See [docs/PLAN.md](docs/PLAN.md) for what is ported, what is simplified and what comes next,
and [docs/MOO1_QUIRKS.md](docs/MOO1_QUIRKS.md) for the MOO1 bugs kept on purpose.

## Modding

Texts, names, pictures and game rules are plain files: a mod is a folder with
`text/en.json`, `rules.json` and `images/*.png`, installed from the **Mods** menu.
[examples/mods/legions](examples/mods/legions) is a small working example and
[docs/MODDING.md](docs/MODDING.md) lists every option.

## Building

Requirements: Rust (stable) with the `wasm32-unknown-unknown` target, Node.js 22, and
`wasm-bindgen-cli` at the version locked in `Cargo.lock`:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
```

| Task | Command |
|---|---|
| Rust tests | `cargo test --workspace` |
| Lint | `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings` |
| Build WebAssembly | `./scripts/build-wasm.sh` (writes `web/src/wasm/`) |
| Web dev server | `cd web && npm ci && npm run dev` |
| Web production build | `cd web && npm run build` (writes `web/dist/`) |
| Desktop app | build the web first, then `cd desktop && npm ci && npx tauri build` |
| CLI | `cargo run --release -p aoo-cli -- --help` |

The desktop app needs the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
(WebKitGTK on Linux).

The CLI converts LBX files and runs AI-only games:

```sh
aoo import ~/games/MOO1 assets/     # LBX -> manifest.json, gfx/**.png, text/**.json
aoo sim --seed 7 --size medium --players 4 --turns 300 --save end.json
```

## Project layout

| Path | What it is |
|---|---|
| `crates/core` | Game rules: galaxy generation, economy, research, ship design, fleets, combat, AI, saves (no I/O) |
| `crates/lbx` | LBX reader (container, graphics, palettes, string tables) and LBX to PNG/JSON converter |
| `crates/wasm` | WebAssembly bindings for the UI (game API with JSON in/out, LBX importer) |
| `crates/cli` | `aoo` command line tool |
| `crates/harness` | `aoo-harness`: comparison against C 1oom (save decoder, state diff, random number diff) |
| `tools/harness/` | Patch and scripts to build and run 1oom for the harness |
| `tools/gen_1oom_tables.py` | Generates the ship part and tech tables (`crates/core/src/data.rs`) from a 1oom checkout |
| `web/` | HTML/CSS/canvas UI (TypeScript, Vite) |
| `themes/` | One folder per theme: pictures, music, sounds, texts and colours (`docs/MODDING.md`) |
| `desktop/` | Tauri shell that packages `web/dist` as a native app |
| `docs/` | Plan, MOO1 comparison, modding guide, comparison harness, MOO1 quirks, art |
| `tools/art/`, `tools/sound/` | Scripts that make the theme pictures, music and sound effects |

## Releases

Pushing a tag `vX.Y.Z` runs `.github/workflows/release.yml`, which builds the web zip, the
desktop installers (Linux `.deb`/`.rpm`/`.AppImage`, Windows `.msi`/`.exe`, macOS `.dmg`) and
the CLI for three platforms, and publishes them as a GitHub release with the notes of that
version from [CHANGELOG.md](CHANGELOG.md). Every push to `main` deploys the website to
GitHub Pages.

## License

GPL-2.0-only, like 1oom, from which the rules are derived. See [LICENSE](LICENSE).

Master of Orion is a trademark of its respective owners. This project contains no original
game assets; the LBX importer only reads files the player provides, locally.
