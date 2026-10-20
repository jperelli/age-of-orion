# Modding

Everything the player reads or sees comes from data files, layered in this order (later
wins):

1. `web/public/text/en.json`: base texts (Master of Orion wording).
2. `web/public/themes/<theme>/text/en.json`: the theme's texts (`age` renames planets to
   cities, ships to units, and so on).
3. The active mod's `text/*.json`.

Images are looked up by **alias**: the active mod's `images/<alias>.png` first, then images
converted from the player's LBX files (Classic theme), then procedural art drawn by the map.

## Mod folder

```
my-mod/
  mod.json          {"id": "my-mod", "name": "My mod", "description": "...", "theme": "age"}
  text/en.json      any text keys to replace
  rules.json        game rules to replace (used by new games)
  images/army.png   pictures by alias
```

Install it from **Mods > Install mod folder**; it is stored in the browser (IndexedDB) and
becomes active. `theme` picks the base theme (`age` or `classic`). See
[examples/mods/legions](../examples/mods/legions).

## Text keys

All keys are in `web/public/text/en.json`. The most useful groups:

| Keys | What |
|---|---|
| `app.title`, `app.subtitle` | Title screen |
| `planet.<type>` | Planet types: `not_habitable`, `radiated`, `toxic`, `inferno`, `dead`, `tundra`, `barren`, `minimal`, `desert`, `steppe`, `arid`, `ocean`, `jungle`, `terran` |
| `star.<colour>` | `yellow`, `red`, `green`, `white`, `blue`, `neutron` |
| `special.*`, `growth.*` | Planet specials and environments |
| `race.<race>`, `race.<race>.plural`, `race.<race>.trait`, `race.<race>.leader` | Races: `human`, `mrrshan`, `silicoid`, `sakkra`, `psilon`, `alkari`, `klackon`, `bulrathi`, `meklar`, `darlok` |
| `design.<key>` | Ship designs (`scout`, `fighter`, `destroyer`, `bomber`, `colony_ship`, or your own keys) |
| `field.<field>` | Research fields: `computer`, `construction`, `force_field`, `planetology`, `propulsion`, `weapon` |
| `slider.<slider>` | Colony sliders: `ship`, `def`, `ind`, `eco`, `tech` |
| `event.<type>` | Turn report lines; placeholders like `{planet}`, `{count}`, `{design}`, `{empire}` |
| `error.<code>` | Rejected orders |
| `names.sites` | List of star/city names (at least 108 for huge maps, otherwise MOO1 names are used) |
| `names.homes` | Home world name per race (10, in race order) |
| `names.leaders` | Leader name per race (10) |
| `names.orion` | Name of Orion |
| `help.lines` | How to play, one paragraph per entry |

## Image aliases

| Alias | Used for |
|---|---|
| `planet/<n>` | Planet picture `n` (0 to 34) in the side panel (Classic theme; PLANETS.LBX 0..34) |
| `race/<n>` | Race portrait `n` (0 to 9, race order) in the Races dialog (PLANETS.LBX 35..44) |
| `site/<type>` | Map icon of an unclaimed site of that planet type (Age map) |
| `site/city` | Map icon of a colony |
| `army` | Fleets on the map |
| `caravan` | Transports on the map |
| `guardian` | The Guardian of Orion |
| `map/background` | Whole map background (stretched to the map size) |

## Rules

`rules.json` overrides fields of the core's `Rules` (`crates/core/src/rules.rs`); missing
fields keep their defaults. The defaults as JSON are returned by `defaultRules()` from the
WebAssembly module. Main fields:

| Field | Default | Meaning |
|---|---|---|
| `start_year` | 2300 | First year |
| `designs` | 5 MOO1 designs | `{key, cost, hp, attack, bombs, colony, extra_range}` each |
| `start_ships` | `[2, 2, 0, 0, 1]` | Ships per design at the start (same length as `designs`) |
| `home_pop`, `home_max_pop`, `home_factories` | 50, 100, 30 | Home world |
| `colony_ship_pop` | 2 | Colonists brought by a colony ship |
| `research_cost_percent` | 40 | Research cost relative to the 1oom formula |
| `race_tech_cost` | 1oom table | Research cost per race (10 rows) and field (6 columns), percent |
| `base_cost`, `base_attack`, `base_hp` | 100, 10, 40 | Missile bases |
| `guardian_attack`, `guardian_hp` | 150, 2500 | The Guardian of Orion |
| `orion_tech_bonus` | 5 | Levels gained in every field when colonizing Orion |
| `parsec` | 11 | Map units per parsec (distance unit of range and speed) |

The AI picks designs by role: a design with `colony: true` settles, one with `extra_range`
and no weapons scouts, `bombs` bombards, the rest fight.

## Imported asset tree

The LBX importer (`aoo import <dir> <out>` or the in-game import) writes:

```
manifest.json        format, aliases, every archive and entry, warnings
gfx/<lbx>/NNN.png    every graphics entry; animations as horizontal sheets (frame width in manifest)
text/lbx/names.json  NAMES.LBX leader names
text/lbx/eventmsg.json  EVENTMSG.LBX texts, placeholders as {planet}, {race}, {n1}, {n2}
```
