# Modding

Everything the player reads, sees or hears comes from data files: the theme folders of
`themes/`, the player's imported MOO1 files and the active mod.

## Themes

Each theme is a folder of `themes/`; a new folder is a new theme in the menu (the build lists
them in `themes/index.json`). The folders have the same layout:

```
themes/<id>/
  theme.json        {"name": "...", "description": "...", "map": "space" or "world",
                     "use_lbx": false, "banners": ["#4f7dff", ... six colours]}
  theme.css         colours and fonts (the CSS variables of web/src/style.css)
  text/en.json      texts, and text/<lang>.json for other languages
  images/<name>.svg every picture, named as the game asks for it (images/ship/3.svg is `ship/3`)
  audio/music/<tune>.mid   music: MIDI for the game's synthesizer, or .ogg, .mp3, .wav
  audio/sfx/<effect>.wav   sound effects
```

A picture may be `.svg`, `.png`, `.jpg` or `.webp`; with several of one name the first of
that order wins. **Classic's folder has every picture, tune and sound the game uses** (drawn
as SVG by `tools/art/gen_svg.py`, composed by `tools/sound/gen_music.py`), so it is the
starting point of a new theme. Another theme may lack some: a missing picture shows a
placeholder (a dashed frame with a question mark) in its place, and a missing sound stays
silent. `python3 tools/themes_check.py` lists what each theme lacks of Classic's files.

- `map` picks the map and the screens drawn around it: `space` (stars, the GNN newscast,
  MOO1's embassy, ships in battle) or `world` (sites and cities on a land map, the Herald,
  throne rooms, troops). The pictures marked *world* or *space* below are used by one kind.
- `use_lbx`: the player's MOO1 files, once imported, take the place of the theme's
  pictures, music, sounds and texts (Classic).
- Texts: Classic's are the base of every theme, then the theme's, then the mod's. A missing
  key in a language falls back to English. A new language is a new `text/<lang>.json` in
  Classic's folder (and Age's), its code in `LANGS` in `web/src/theme.ts` and a
  `lang.<code>` name in every text file.

A picture comes from the active mod's `images/<name>.png` first, then from the player's MOO1
files (when the theme uses them), then from the theme's folder.

## Mod folder

```
my-mod/
  mod.json          {"id": "my-mod", "name": "My mod", "description": "...", "theme": "age"}
  text/en.json      any text keys to replace
  rules.json        game rules to replace (used by new games)
  images/army.png   pictures by alias
```

Install it from **Mods > Install mod folder**; it is stored in the browser (IndexedDB) and
becomes active. `theme` picks the base theme (a folder of `themes/`). See
[examples/mods/legions](../examples/mods/legions).

## Text keys

All keys are in `themes/classic/text/en.json`. The most useful groups:

| Keys | What |
|---|---|
| `app.title`, `app.subtitle` | Title screen |
| `planet.<type>` | Planet types: `not_habitable`, `radiated`, `toxic`, `inferno`, `dead`, `tundra`, `barren`, `minimal`, `desert`, `steppe`, `arid`, `ocean`, `jungle`, `terran` |
| `star.<colour>` | `yellow`, `red`, `green`, `white`, `blue`, `neutron` |
| `special.*`, `growth.*` | Planet specials and environments |
| `race.<race>`, `race.<race>.plural`, `race.<race>.trait`, `race.<race>.leader` | Races: `human`, `mrrshan`, `silicoid`, `sakkra`, `psilon`, `alkari`, `klackon`, `bulrathi`, `meklar`, `darlok` |
| `design.<key>` | Names of the five starting designs (`scout`, `fighter`, `destroyer`, `bomber`, `colony_ship`) |
| `names.ships.<race>` | 12 names per race offered to new designs, 3 per hull size (small to huge) |
| `tech.<field>.<key>` | Tech names (keys in `crates/core/src/data.rs`, `TECHS`), `tech.advanced` for techs above 50 |
| `part.<kind>.<key>` | Ship part names: `hull`, `weapon`, `comp`, `jammer`, `shield`, `armor`, `engine`, `special` (`.info` for the description of weapons and specials) |
| `designer.*`, `design.error.*` | Ship design screen |
| `research.*` | Research screen |
| `field.<field>` | Research fields: `computer`, `construction`, `force_field`, `planetology`, `propulsion`, `weapon` |
| `slider.<slider>` | Colony sliders: `ship`, `def`, `ind`, `eco`, `tech` |
| `event.<type>` | Turn report lines; placeholders like `{planet}`, `{count}`, `{design}`, `{empire}` |
| `news.<kind>.<stage>` | Random events (`plague`, `quake`, `nova`, `accident`, `assassin`, `virus`, `comet`, `pirates`, `derelict`, `rebellion`, `crystal`, `amoeba`, `enviro`, `rich`, `support`, `poor`, `stars`, `coup`). Stages follow MOO1's EVENTMSG.LBX: 0-3 about you, 4-6 about others. Placeholders `{planet}`, `{empire}`, `{other}`, `{n1}`, `{n2}`, `{field}`, `{personality}` |
| `diplo.<n>` | Messages of the computer rulers, numbered as in 1oom (`diplo_type`): 14-19 first contact per personality, 24-30 proposals, 31-41 refusals, 42-57 warnings and broken treaties, 62-68 acceptances... Placeholders `{leader}`, `{race}` (theirs), `{me}`, `{empire}` (a third empire), `{planet}`, `{field}`, `{bc}`, `{tech}`, `{gift}` |
| `treaty.*`, `attitude.*`, `trait1.<n>`, `trait2.<n>` | Treaties, attitudes, personalities and objectives |
| `audience.*`, `diplomacy.*`, `spy.*`, `council.*` | Audience, races, spy and council screens |
| `monster.crystal`, `monster.amoeba` | Names of the two space monsters |
| `battle.*` | Battle screen; `battle.special.<kind>` names the specials as they fire |
| `techinfo.<field>.<n>` | Description of tech `n` of field `0..5` (computer, construction, force field, planetology, propulsion, weapon), shown on the research screen; filled from RESEARCH.LBX in the Classic theme |
| `error.<code>` | Rejected orders |
| `names.sites` | List of star/city names (at least 108 for huge maps, otherwise MOO1 names are used) |
| `names.homes` | Home world name per race (10, in race order) |
| `names.leaders` | Leader name per race (10) |
| `names.orion` | Name of Orion |
| `help.lines` | How to play, one paragraph per entry |

## Pictures

The name of each picture (its path in `images/`), with MOO1's file when it has one:

| Name | Used for |
|---|---|
| `planet/<n>` | *space*: planet picture `n` (0 to 34) in the side panel and in battle (PLANETS.LBX 0..34) |
| `race/<n>` | Race portrait `n` (0 to 9, race order) in audiences and the empires screen (PLANETS.LBX 35..44) |
| `arms/<race>` | A coat of arms (0..9, race order): the owner in the side panel, the empires screen, the new game's empire choice |
| `ui/gold`, `ui/research`, `ui/reserve` | Icons of the side panel's header (24x24) |
| `icon/pop`, `icon/factory`, `icon/waste`, `icon/base`, `icon/shield` | Icons repeated in the planet window's rows (24x24) |
| `site/<type>` | *world*: map marker of a free site of that planet type |
| `army` | *world*: armies on the map |
| `caravan` | Transports on the map, with the owner's pennant |
| `guardian` | *world*: the Guardian's mark over Orion |
| `monster/crystal`, `monster/amoeba` | The two space monsters on the map |
| `ship/<look>` | A stack in battle (look = banner * 24 + hull * 6 + variant, 144..146 the crystal, amoeba and Guardian), facing right (SHIPS.LBX, SHIPS2.LBX) |
| `battle/bg/<n>` | Battle background, 0 in a nebula, 1..4 otherwise (SPACE.LBX 0x1a..) |
| `battle/rock/<n>` | Asteroids (rocks) on the battle grid, 0..3 |
| `battle/explosion/0` | Explosion |
| `missile/<kind>/<dir>` | Missiles in flight: kind 0..4 (missile, antimatter, hellfire, proton, plasma), direction 0 up, 1 up-right ... 7 up-left (MISSILE.LBX) |
| `nebula/<n>` | Nebula of type `n` (0 to 9), drawn at half its size in map units (STARMAP.LBX 0x0f..) |
| `stargate` | Stargate next to a planet (STARMAP.LBX 0x2d) |
| `map/background` | Optional: the whole map's background (stretched to the map size), in place of the one the map draws |
| `landing/invade/<type>` | Ground battle background for a planet type 1..13 (LANDING.LBX 0x24 + type) |
| `landing/colonize/<type>`, `landing/transport` | Colonization scenes and the landing transport (LANDING.LBX) |
| `ground/soldier/<race>` | A soldier of race 0..9 on the ground battle screen, facing right (LANDING.LBX 0x1b + race) |
| `ground/death` | A soldier falling (LANDING.LBX 0x14) |
| `news/tv`, `news/anchor`, `news/world`, `news/gnn` | *space*: the GNN newscast: the TV (its screen at 14,14 292x105), the newscaster, the globe, the logo (NEWSCAST.LBX 0..3) |
| `news/icon/<type>` | The newscast picture of news type 1..22 (1oom `GAME_NEWS_*`; NEWSCAST.LBX 3 + type) |
| `star/<k>` | A star on the Classic map: star type 0..5 (yellow, red, green, white, blue, neutron) + 6 for the bigger look (STARMAP.LBX 3..14) |
| `embassy/room`, `embassy/<race>`, `embassy/emperor/<banner>`, `embassy/border` | *space*: MOO1's audience: the room, the envoy of race 0..9 (175x130 at 96,0), the player's emperor seen from behind and the frame (EMBASSY.LBX 1, 2 + race, 0x0c + banner, 0) |
| `newtech/lab/<n>` | The new-tech screen's background: 0 the laboratory, 1 the spies' den, 2 the ruins (TECHNO.LBX 0..2); the tech's picture goes at 145,54 |
| `newtech/<n>` | Who brought the tech, over the background: 10 * source + race, source 0 the race's scientist, 1 its spy, 2 an explorer (SPIES.LBX) |
| `explore/<type>`, `explore/<type>/colony` | The exploration report and the sabotage screen: a planet of type 0..13 without and with a colony, 84x102 (COLONIES.LBX 2 * type, + 1) |
| `landing/colonize/<type>`, `landing/transport`, `landing/walk/<banner>` | A new colony's landing: the land, the transport coming down (142x103, its last frame landed) and the colonists walking out with their banner (62x200) (LANDING.LBX 6 + type, 0, 0x15 + banner) |
| `bombard` | Bombs falling on a colony, 84x102, on the bombing screens (BACKGRND.LBX 0x10) |
| `sabotage/bases`, `sabotage/factories` | Missile bases and factories blowing up, 82x33 (BACKGRND.LBX 7, 8) |
| `spy/espionage` | The espionage panel, 216x139: the agents, the target's portrait at 96,19 and six field buttons (BACKGRND.LBX 0xd) |
| `tech/<field>/<n>`, `tech/<field>/advanced` | The blueprint of tech `n` of field 0..5, and the one of a field's advanced techs, shown on the new-tech screen and the research cards (TECHNO.LBX, picture from RESEARCH.LBX entry 0) |

| `view/<type>`, `view/orion`, `view/city/<0..2>` | *world*: the wide view at the top of the side panel and in the planet window, for a site of that planet type, Orion, or a city by size (population under 25, under 60, more) |
| `battle/field/<kind>` | *world*: battlefield by land (`green`, `forest`, `dry`, `snow`, `rocky`, `ash`, `marsh`, `mists` in a nebula), in place of `battle/bg` |
| `unit/<look>` | *world*: a battle stack by its design's look 0..23 (hull * 6 + variant), drawn fitted in its square with a pennant in the empire's colour; `unit/crystal`, `unit/amoeba`, `unit/guardian` for the monsters and the Guardian |
| `mark/village`, `mark/town`, `mark/castle` | *world*: a city on the map and in battle, by size, with the owner's pennant over it |
| `news/herald` | *world*: the herald next to the Herald's masthead |
| `hall/<race>` | *world*: the throne room behind an empire's envoy (0..9, race order) |
| `council/hall` | The Great Council's hall |

### Animations

Names starting with `anim/` are animations: a horizontal strip of equal frames, played over
the still picture of the same name without `anim/` (shown when the theme has no animation).
The importer lists MOO1's in `manifest.json` under `frames` as `[frames, width, height,
loop]`; a theme's own come with a `.json` beside the picture (`images/anim/star/3.json`:
`{"frames": 4, "w": 13, "h": 11, "loop": 0}`). After the last frame the animation goes on
from frame `loop`.

| Name | Animation |
|---|---|
| `anim/star/<k>` | A star twinkling on the map (STARMAP.LBX 3..14) |
| `anim/ship/<look>` | A ship's engines in battle, 5 frames (SHIPS.LBX, SHIPS2.LBX) |
| `anim/embassy/<race>` | The envoy of race 0..9 in audiences and messages (EMBASSY.LBX 2 + race) |
| `anim/news/anchor`, `anim/news/world` | The newscaster talking and the turning globe (NEWSCAST.LBX 2, 3) |
| `anim/newtech/<n>` | Who brought a tech (SPIES.LBX) |
| `anim/landing/transport`, `anim/landing/walk/<banner>` | The transport coming down, the colonists walking (LANDING.LBX) |
| `anim/bombard`, `anim/sabotage/bases`, `anim/sabotage/factories` | The bombs, the explosions of a sabotage (BACKGRND.LBX) |
| `unit/<look>/march/<0..3>`, `unit/<look>/attack/<0..3>` | *world*: frames of a battle stack, played while it moves and when it shoots or strikes |

## Sounds

| Name | Used for |
|---|---|
| `music/<tune>` | `menu`, `battle`, `news`, `ground` (siege), `ground_win`, `ground_lose`, `council`, `ending_win`, `ending_lose`, `colonize` (a new colony's landing), `bomb`, `bomb_destroyed` (a colony bombed out), `espionage`, `sabotage`, `sabotage_done`, and `audience/<race>` (0..9) for audiences; the endings and the siege's outcome play once, the others loop |
| `sfx/<effect>` | `button` (every button), `click`, `beam`, `heavy_beam`, `missile`, `explosion`, `hit`, `bomb`, `special`, and on a world map `arrow` and `clash` |

With `use_lbx`, MOO1's tunes (MUSIC.LBX), the rulers' messages in English (DIPLOMAT.LBX, one
of MOO1's wordings for each), six leader names per race for new games (NAMES.LBX) and samples (SOUNDFX.LBX, weapons with their own,
1oom `tbl_shiptech_weap[].sound`) take their place, and `font/<n>` (n = 0..5) are MOO1's
fonts as TrueType, used when fonts 3 and 4 are present (`--font`, `--title-font`,
`--button-font`).

## Rules

`rules.json` overrides fields of the core's `Rules` (`crates/core/src/rules.rs`); missing
fields keep their defaults. The defaults as JSON are returned by `defaultRules()` from the
WebAssembly module. Main fields:

| Field | Default | Meaning |
|---|---|---|
| `start_year` | 2300 | First year |
| `start_designs` | 5 MOO1 designs | Ship designs every empire starts with (up to 6; same format as the designs in a save: `name`, `hull`, `weapons`, `weapon_counts`, `engine`, `specials`, `shield`, `jammer`, `comp`, `armor`, `man`, indices into the part tables) |
| `start_ships` | `[2, 2, 0, 0, 1]` | Ships per start design (same length as `start_designs`) |
| `home_pop`, `home_max_pop`, `home_factories` | 50, 100, 30 | Home world |
| `colony_ship_pop` | 2 | Colonists brought by a colony ship |
| `research_cost_percent` | 100 | Research cost relative to the 1oom formula |
| `race_tech_cost` | 1oom table | Research cost per race (10 rows) and field (6 columns), percent |
| `difficulty_tech_cost`, `ai_tech_cost` | 1oom tables | Research cost per difficulty for humans and computers |
| `base_hp` | 50 .. 200 | Missile base hit points by best armor |
| `stargate_cost`, `stargate_maint` | 3000, 100 | Stargates |
| `atmos_cost`, `soil_cost`, `adv_soil_cost` | 200, 150, 300 | Environment projects (ecology spending) |
| `pop_hp`, `fact_hp` | 200, 50 | Bombing damage per population unit and per factory |
| `combat_rounds` | 10 | Rounds of fire per battle |
| `nebulae` | true | Generate nebulae |
| `parsec` | 11 | Map units per parsec (distance unit of range and speed) |

The ship part tables (`crates/core/src/data.rs`) are generated from 1oom by
`tools/gen_1oom_tables.py` and are not part of `rules.json` yet.

The AI picks designs by role: a design with a colony base settles, one with reserve fuel
tanks and no weapons scouts, one with bombs bombards, the rest fight. It adds new designs of
its own as its techs improve.

## Imported asset tree

The LBX importer (`aoo import <dir> <out>` or the in-game import) writes:

```
manifest.json        format, aliases, animations (frames), looping tunes, every archive and entry, warnings
gfx/<lbx>/NNN.png    every graphics entry; animations as horizontal sheets (frame width in manifest)
gfx/<lbx>/NNN-0.png  first frame of animations
audio/music/NNN.mid  MUSIC.LBX tunes as standard MIDI files (alias music/<n>)
audio/<lbx>/NNN.wav  sound effects (SOUNDFX.LBX entries have the alias sfx/<n>)
fonts/moo1-N.ttf     FONTS.LBX fonts as TrueType (alias font/<n>; heights in manifest.fonts)
text/lbx/names.json  NAMES.LBX leader names
text/lbx/eventmsg.json  EVENTMSG.LBX texts, placeholders as {planet}, {race}, {n1}, {n2}, {leader}...
text/lbx/research.json  RESEARCH.LBX tech descriptions (field * 50 + tech - 1)
```
