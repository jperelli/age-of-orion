# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- The Age theme says Disband instead of Scrap in the army roster.

## [0.2.0] - 2026-10-20

Closer to Master of Orion: research choices, ship design and the missing planet and fleet
systems.

### Added

- Research as in MOO1 (1oom `game_tech.c`): per-empire research lists generated from the
  1oom tables (Psilons get more choices), one project per field chosen by the player,
  15% interest on investment and random breakthroughs, field levels, the "advanced"
  techs once a list is exhausted. A research screen with the choices, their costs, progress
  and the known techs, opened after each turn when a field is idle, with a button that
  picks the cheapest techs.
- Tech effects now come from the techs actually completed (1oom `game_update_tech_util`):
  robotic controls, factory cost, waste, fuel range, scanners, colony types, terraforming,
  population cost, planetary shields, antidotes, ground combat bonuses.
- Orion grants the Death Ray and three advanced techs; ancient artifacts grant a tech to
  the first explorer.
- Ship design (1oom `game_shiptech.c`, `game_design.c`): the complete part tables (hulls,
  64 weapons, computers, ECM, shields, armor, engines, 30 specials), cost, space, power and
  engine formulas with miniaturization, up to six designs per empire, a design screen with
  live cost, space and combat figures, replacing and scrapping designs. The five MOO1
  starting designs cost exactly what they cost in MOO1.
- Colony ships settle according to their colony base (standard, barren, tundra ... radiated).
- Space battles use the designs: computer level against defense and ECM, shields, weapon
  damage ranges, missile shots and anti-missile specials, race bonuses, the Guardian's
  stats per difficulty (1oom `game_battle_attack`, `tbl_monster`).
- Orbital bombing (1oom `game_bomb.c`), with an option to stop bombing enemy planets; the
  computer bombs unless its own transports are on the way.
- Transports are shot down by bases and ships (1oom `game_turn_transport_shoot`); combat
  transporters save some; transports sent to empty planets are lost as in MOO1.
- Soil enrichment, advanced soil enrichment and atmospheric terraforming
  (1oom `game_turn_build_eco`).
- Stargates: built from the shipyard, 100 BC upkeep, fleets between two of your stargates
  arrive in one turn.
- Nebulae (1oom `game_generate_galaxy`): drawn on the map, half speed inside, no shields for
  planets and ships fighting there.
- Rally points: new ships of a colony go to another colony. Scrapping ships in orbit (a
  quarter of the cost back).
- Computer players choose research like the classic AI and design new ships as techs
  arrive.
- `tools/gen_1oom_tables.py` regenerates the data tables and English names from a 1oom
  checkout. The Age theme renames every tech and ship part (bows, javelins, siege rams,
  waygates...) and gives every race its own unit names.

### Changed

- Rules: `start_designs` (full ship designs) replaces `designs`; `guardian_*`,
  `base_cost`, `base_attack`, `orion_tech_bonus` and `research_weights_ai` are gone (the
  1oom formulas are used); `research_cost_percent` defaults to 100.
- Save format 2. Saves from 0.1 cannot be loaded and say so.
- End turn ignores repeated key presses while a turn is being processed.

### Fixed

- The side panel scrolls back to the destination prompt when choosing where to send a fleet.
- Race names in the Races dialog are readable with light banner colours.
- The first home world is placed away from Orion and its Guardian.
- The release workflow uploads the CLI archives and can be re-run for an existing tag.

## [0.1.0] - 2026-10-20

First playable version.

### Added

- Rust game core (`aoo-core`) ported from 1oom: MOO1 random number generator, galaxy
  generation, planet types and specials, colony economy with the five sliders, pollution,
  terraforming, population growth, factory refits, maintenance, six research fields with the
  MOO1 cost formula, ten races, five difficulty levels.
- Fleets with fuel range and speed, colonization, missile bases and planetary shields,
  automatic space battles, bombing, transports and ground invasions, the Guardian of Orion,
  victory by conquest.
- Computer players that expand, scout, research, build fleets and invade.
- JSON save format, deterministic turns (same save + same orders = same result).
- LBX reader and converter (`aoo-lbx`): container, graphics with RLE and delta frames,
  embedded palettes, FONTS.LBX palettes, string tables (NAMES, EVENTMSG) to PNG and JSON.
- WebAssembly bindings (`aoo-wasm`) and a web UI: world map renderer with terrain and fog of
  war, colony panel with sliders, shipyard, fleet orders with range preview, transports,
  research, planets and races dialogs, turn reports.
- Themes: Age (cities, armies and kingdoms) and Classic (stars and planets, with the
  player's imported MOO1 art). Mods with texts, images and rules.
- In-browser LBX import stored in IndexedDB; nothing is uploaded.
- Desktop app (Tauri) for Linux, Windows and macOS running the same WebAssembly build.
- `aoo` CLI: `import` (LBX to assets) and `sim` (AI-only games).
- CI (fmt, clippy, tests, web and desktop builds), tag-driven releases and GitHub Pages
  deployment.

[Unreleased]: https://github.com/jperelli/age-of-orion/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/jperelli/age-of-orion/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jperelli/age-of-orion/releases/tag/v0.1.0
