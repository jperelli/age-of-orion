# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.8.0] - 2026-10-21

### Added

- Spanish: every text of the game in Spanish, for both themes (`text/es.json`,
  `themes/age/text/es.json`), and a language selector on the main menu. The language
  follows the browser's on the first start.
- The converted MOO1 fonts gain the Spanish letters (á é í ó ú ü ñ ¡ ¿), drawn from the
  original glyphs; import the LBX files again to get them.

### Changed

- MOO1's own news texts and tech descriptions (EVENTMSG.LBX, RESEARCH.LBX), which are in
  English, are only used when the game is in English.

## [0.7.0] - 2026-10-21

Exact rules: the comparison harness now agrees with 1oom on 86% of the turns of five
computer-only test games (613 of 710; the first 40 years of the small game, all of them).

### Fixed

- Battles: a planet's missile bases fought the same attacker again and again in one turn
  (up to 50 times); now, as in 1oom, the loser of a battle is out of that planet's fights.
- Orbital bombing as in 1oom: the best computer of every design counts, only the first three
  weapon slots bomb, biological weapons hurt even when the planet is spared, bombing kills
  rebels too, and a destroyed colony keeps its factories.
- Comets and pirates: damage rolled as in MOO1; the computer players react to plagues, novas,
  comets and pirates with 1oom's slider changes instead of ones of our own.
- Diplomacy: big empires worry their allies on 1oom's years (the year counter starts at 1
  in 2300); breaking a treaty follows MOO1 in Classic (and 1oom's fixes in Classic+); the
  "leave both alliances" rule that never fires in MOO1 no longer fires; computers judge
  fleet strength from the last maintenance count; MOO1's use of the last audience message
  when deciding on peace.
- The computer players read the "expansionist" objective correctly (it was another one).
- Random events: the galaxy statistics newscast no longer follows turns with news.
 - 2026-10-21

Presentation: MOO1's own fonts, the ground battle screen, the GNN newscast and MOO1's sound
effects.

### Added

- MOO1 fonts: the importer turns the six bitmap fonts of FONTS.LBX (1oom `lbxfont.c`) into
  TrueType fonts made of square pixels, and the Classic theme uses them for all text
  (font 3 for text, 4 for titles, 5 for buttons).
- The ground battle screen (1oom `uiground.c`): when your transports land on an enemy
  planet or the enemy lands on yours, the troops face each other and fall one by one in the
  order the fight was resolved, then the screen tells who holds the planet, how many
  factories were captured and which technologies were found. In the Classic theme it uses
  the invasion scenes and soldiers of LANDING.LBX; the Age theme draws a siege before the
  city walls.
- The GNN newscast (1oom `uinews.c`): the turn's news (random events, the Orion colony, the
  fall of the Guardian, empires destroyed, the largest empire) is shown one item at a time on
  MOO1's TV with the GNN newscaster and the icon of each news type from NEWSCAST.LBX, or as
  the herald's proclamations in the Age theme. The galaxy statistics newscast ranks the
  empires by production, population, technology or fleet strength.
- Sound effects: the importer converts the Creative Voice samples of SOUNDFX.LBX (and other
  sound LBX files) to WAV (1oom `fmt_sfx.c`); in the Classic theme buttons, weapons, hits,
  explosions and ground combat play MOO1's samples instead of the synthesized effects.
- New imported pictures: LANDING.LBX (invasion and colonization scenes, transports,
  soldiers) and NEWSCAST.LBX, drawn with the right palette.

### Changed

- The galaxy statistics news is a real event of the turn (1oom `game_event_run`), with its
  ranking.
 - 2026-10-21

The computer players of Master of Orion, and a harness that checks the rules against C 1oom.

### Added

- The classic AI's turn, ported line by line from 1oom `game_ai_classic.c`: scouts, colony
  ships (including MOO1's way of conjuring one now and then), fronts against neighbours,
  attacks on weak enemy planets (breaking pacts when it suits them), defence of threatened
  colonies, idle fleets moved to the front, invasions with transports, colonists moved to
  under-populated planets, reserve spent on threatened and young colonies, taxes
  (`turn_p1`); ship design with MOO1's part choices, scrapping old designs and choosing what
  each planet builds (`turn_p2`); production sliders by personality and race, research
  focus, security and spy missions (`turn_p3`); the opening sliders and research.
- A choice of computer players on the new game screen: Classic (MOO1's behaviour, bugs
  included) or Classic+ (1oom's fixes).
- Comparison harness against C 1oom (`crates/harness`, `tools/harness`, docs/HARNESS.md):
  decodes 1oom saves, plays each year of a computer-only 1oom game in Age of Orion and
  compares the state and every random number. The first 40 years of a three-opponent game
  are identical turn by turn; about half the turns of longer games are.
- Colonists to send leave at the end of the turn, as in MOO1; the colony panel shows them
  and can cancel.

### Changed

- Turn processing follows 1oom `game_turn_process` step by step, with exact ports of
  production and maintenance, waste and population growth, missile bases and shields, ship
  building, factories and refits, research and breakthroughs, contact, fuel and scanner
  ranges, visibility, fleet and transport movement, colonization, transports and ground
  combat, and the end of turn bookkeeping.
- Map positions, distances and movement use 1oom's integer geometry (`util_math.c`): fleets
  move in steps of 5 and 6 map units, parsecs come from the same lookup tables.
- 1oom's default rules: MOO1 v1.3a's fixes (the "4 colony ships" AI curse, factories up to
  2700, the Oracle interface, orbital weapons, ship scanners, the Guardian's repairs) and
  none of 1oom's optional fixes. Some numbers change accordingly, for example the starting
  sliders (industry and ecology only) and transports, which move at the speed of your best
  engine minus one level, as in MOO1.
- Saves from 0.4 no longer load (positions are now integers).

### Fixed

- Captured planets broke the save file (an event field clashed with the event's recipients).
- Ground combat, the computer-against-computer battle resolution, design costs and ship
  maintenance overflow and round exactly as in MOO1.
- Cities and planets on the battle grid no longer cover the neighbouring squares.

## [0.4.0] - 2026-10-21

Battles and presentation: tactical battles on the MOO1 grid, the soundtrack, and more of
your original art.

### Added

- Tactical battles (1oom `game_battle_human.c`): when your ships or planets fight, the turn
  pauses on the 10x8 battle grid. Stacks act in order of initiative; move them, fire beams
  at range, launch missiles that fly over several turns (and can be dodged or hit
  asteroids), hold missiles back, wait, retreat, or hand over to the computer at any time.
  You can also let the computer fight the whole battle or order a retreat before it starts.
- Every combat special works: stasis field, energy and ionic pulsars, ion and neutron stream
  projectors, warp dissipator, black hole generator, technology nullifier, repulsor beam,
  sub-space teleporter and interdictor, cloaking device, displacement device, damage control,
  Oracle interface; asteroids in battle; retreat to the nearest colony.
- The classic battle AI (1oom `game_ai_classic.c`): target choice, best range, dodging
  missiles, retreating when the battle is lost; battles between computer empires are
  resolved from the strength of both sides, as in 1oom.
- Monsters and the Guardian fight on the battle grid too.
- Music: the importer converts MUSIC.LBX tunes to MIDI (1oom `fmt_mus.c`) and a built-in
  synthesizer plays them: on the menu, in battles, at the council and in audiences.
  Sound effects for battles. Volume sliders on the main menu.
- The importer runs in a Web Worker, so the page no longer freezes, and exports battle
  pictures (SHIPS.LBX, SHIPS2.LBX, SPACE.LBX, MISSILE.LBX), music and the tech descriptions
  of RESEARCH.LBX, shown on the research screen in the Classic theme.
- Battle art for both themes when no original files are imported (formations and banners
  in the Age theme, spaceships in the Classic theme).

### Changed

- Battles follow 1oom: two parties at a time per planet, so treaties, ceasefires and
  retreats work as in MOO1. A loaded game that was saved during a battle resumes it.

### Fixed

- Messages from other rulers are titled "Envoy from" instead of "Envoy to".

## [0.3.0] - 2026-10-20

Diplomacy and events: the other empires talk, trade, spy, plot and vote.

### Added

- Diplomacy (1oom `game_diplo.c`): contact when colonies come within fuel range, relations
  from the MOO1 race table, personalities and objectives, moods, trust, non-aggression
  pacts, alliances, wars, ceasefires and trade agreements whose income grows over the years.
  Allies share fuel range; treaty partners do not fight, bomb or invade each other; losers
  of a battle retreat to their nearest colony.
- Audiences (1oom `game_audience.c` and the classic AI's answers): propose a pact, an
  alliance, peace or trade, ask an empire to declare war or to leave an alliance, break
  treaties, threaten, offer tribute in BC or technology, exchange technology. The computer
  may ask for a gift before agreeing.
- Computer rulers greet you, propose pacts, alliances, trade, technology exchanges and
  peace, offer bounties for attacking their enemies, ask allies to join their wars, warn you
  about spies or attacks and declare war (1oom `turn_diplo_p2`). Computer empires make
  treaties, trade technology and go to war among themselves (`turn_diplo_p1`).
- Espionage and sabotage (1oom `game_spy.c`): a spy budget and mission per empire,
  security, spies caught or framing others, stealing a technology or destroying factories,
  missile bases or stirring up revolt, with the choice left to the player.
- Random events (1oom `game_event.c`): plague, earthquake, nova, industrial accident,
  assassination, computer virus, comet, pirates, derelict ship, rebellion, the Space Crystal
  and the Space Amoeba roaming the map, climate shift, rich and poor minerals, donations.
  Rebellions stop a colony's production until troops land; half the planets in rebellion
  topple a government (1oom `game_turn_coup`).
- The Galactic Council (1oom `game_election.c`): every 25 years once two thirds of the planets
  are settled, the two largest empires stand for High Master. Two thirds of the votes win the
  game; refusing the verdict starts the final war against everyone else.
- Taxes that fill the reserve and sending reserve BC to a colony (1oom `game_turn_reserve`).
- Screens for all of the above: races with relations, spies and security; audiences;
  incoming messages and proposals; spy successes; the council vote. Every message has its
  own text in both themes (kingdoms, envoys, frost giants and wyrms in the Age theme).
- The Classic theme shows the original MOO1 news texts when EVENTMSG.LBX was imported; the
  importer now converts all its placeholders.
- `aoo sim --save FILE` writes the final position as a save file.

### Changed

- Defeating the Guardian, not colonizing Orion, reveals its technologies (as in 1oom).
- Saves have version 3; saves of 0.2 cannot be loaded.

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

[Unreleased]: https://github.com/jperelli/age-of-orion/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/jperelli/age-of-orion/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/jperelli/age-of-orion/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/jperelli/age-of-orion/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jperelli/age-of-orion/releases/tag/v0.1.0
