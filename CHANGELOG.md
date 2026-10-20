# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- The side panel scrolls back to the destination prompt when choosing where to send a fleet.
- Race names in the Races dialog are readable with light banner colours.
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

[Unreleased]: https://github.com/jperelli/age-of-orion/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jperelli/age-of-orion/releases/tag/v0.1.0
