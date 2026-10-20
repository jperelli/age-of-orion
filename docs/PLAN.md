# Plan

## Goal

A web and desktop remake of Master of Orion 1 whose rules come from the 1oom engine,
rewritten in Rust and compiled to WebAssembly, with every text, picture and number in open
files so the game can be reskinned (the default theme turns planets into cities, fleets into
armies and empires into kingdoms) and modded without touching code.

## Decisions

1. **Rust core, no UI inside.** `crates/core` holds the whole game state and rules and talks
   JSON: `new game options -> state`, `command`, `end turn`, `view for one player`. It runs
   natively (tests, CLI) and as WebAssembly (web, desktop).
2. **One UI for web and desktop.** HTML/CSS for panels, menus and dialogs, canvas for the
   map. The desktop app is the same build inside Tauri's webview, so there is one code path.
3. **LBX only as an import source.** The engine never reads LBX. `crates/lbx` converts the
   player's own files into PNG and JSON, in the browser (IndexedDB) or with the CLI.
   No original asset is ever hosted or committed.
4. **Data over code.** Texts are JSON keyed by ID (`text/en.json`, theme and mod overrides),
   rules are a `Rules` object with defaults that a mod can override, images are looked up by
   alias with procedural fallbacks.
5. **Rewrite the UI, port the rules.** The 1993 UI (24.7k lines of 320x200 screens with
   blocking loops) is not ported; the event-driven DOM/canvas UI replaces it. The game logic
   is ported formula by formula.

## Architecture

```mermaid
flowchart LR
  LBX[Player's LBX files] -->|aoo-lbx, in WASM or CLI| A[(PNG + JSON assets)]
  MOD[Mod folder: texts, images, rules] --> T
  A --> T[Theme: texts + images]
  subgraph Browser or Tauri webview
    CORE[aoo-core in WASM: rules, AI, turns, saves]
    UI[web/: DOM panels + canvas map]
    UI <-->|commands / view JSON| CORE
    T --> UI
  end
```

## Port status (1oom `src/game` -> `crates/core`)

| 1oom | Rust | Status |
|---|---|---|
| `rnd.c` | `rng.rs` | Ported exactly |
| `game_new.c` (nebulae, planets, names, homes, research lists) | `galaxy.rs`, `tech.rs` | Nebulae, planets and research lists ported exactly; home placement simplified |
| `game_misc.c` production, growth, tech output | `turn.rs` | Ported |
| `game_turn.c` eco, def, ship, ind, reserve, movement, explore | `turn.rs` | Ported with 1oom's fix options, including soil enrichment, atmospheric terraforming, stargates, nebulae and relocation |
| `game_tech.c` | `tech.rs`, `rules.rs` | Ported: research lists, choices, interest, breakthroughs, field levels, tech effects, Orion and artifact loot |
| `game_battle*.c` | `combat.rs` | Shot formula (`game_battle_attack`), ship and base stats (`game_parsed.c`), Guardian ported; battles resolve in rounds of simultaneous fire without the tactical grid, movement and most specials |
| `game_bomb.c`, `game_turn_transport_shoot` | `combat.rs` | Ported |
| `game_ground.c` | `combat.rs` | Ported (d100 + bonus rounds) |
| `game_shiptech.c`, `game_shipdesign.c`, `game_design.c` | `data.rs` (generated), `design.rs` | Ported: part tables, cost, space, engines, hit points, starting designs |
| `game_ai_classic.c` | `ai.rs` | Research choice and focus ported; the rest is a compact strategy with an automatic ship designer |
| `game_diplo.c`, `game_audience.c`, `game_spy.c`, `game_election.c`, `game_event.c`, `game_news.c` | - | Not yet: every empire is at war with every other |
| `game_save.c` | `serde` JSON | New format; MOO1 save import not yet |

## Milestones

### 0.1.0 (done)
Playable game from start to conquest in the browser and on the desktop, two themes, LBX
import, mods, CI/CD, releases, website.

### 0.2.0: closer to MOO1 (done)
- Ship design screen with every MOO1 part (`game_shiptech.c`, `game_design.c`).
- Research choices among the available techs per tier (`game_tech.c`), tech names.
- Soil enrichment, atmospheric terraforming, stargates, nebulae.
- Fleet orders: split (send part of an orbit), automatic merge in orbit, rally points for
  new ships, scrapping. Orbital bombing and transport losses as in MOO1.
- Still open from this list: tech descriptions (MOO1 keeps them in RESEARCH.LBX, to be
  imported), the effects of combat specials that need the tactical grid.

### 0.3.0: diplomacy and events
- Contact, treaties, trade and audiences (`game_diplo.c`, `game_audience.c`).
- Espionage and sabotage (`game_spy.c`), random events with EVENTMSG texts (`game_event.c`),
  the Galactic Council and diplomatic victory (`game_election.c`).
- Port of the classic AI.

### 0.4.0: battles and presentation
- Tactical combat on a grid (`game_battle_human.c`) with sprites from imported SHIPS.LBX or
  theme art.
- Music and sound: XMIDI to MIDI in the importer (`fmt_mus.c`) and a SoundFont synth.
- Combat specials (repulsor, stasis, black hole, teleporter, ...), retreat.
- LBX import in a Web Worker, more aliases (ships, screens), tech descriptions from RESEARCH.LBX, fonts.

### Later
- Comparison harness against C 1oom (same seed, turn-by-turn state diff) to pin the rules.
- Translations, accessibility, touch controls, multiplayer by save exchange.

## Legal constraints

- The rules are derived from 1oom, so the project is GPL-2.0-only and its source is public.
- Original MOO1 assets are copyrighted: only the player's own files are imported, locally.
