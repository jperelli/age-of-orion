# Art

The pictures, music and sounds shipped with the themes (`themes/<theme>/`) are made for Age
of Orion. None comes from Master of Orion: the Classic theme shows MOO1's own pictures only
when the player imports their LBX files, and those take the place of the shipped ones.

## Classic theme: drawn pictures, composed music

Every picture of the Classic theme but the race portraits is an SVG drawn by
`tools/art/gen_svg.py` (the drawing code is in `tools/art/svgart/`): planets, ships in the
six empire colours, stars, nebulae, the battle screen's pieces, the ten races' envoys,
scientists, spies and soldiers, the embassy, the newscast and its 22 news pictures, the
laboratories, a blueprint for each of the 193 techs, the colonization and invasion scenes,
and the pictures only a world map uses (sites, colonies, views, battlefields, throne rooms).
It also writes the few pictures of the Age theme the game used to draw itself (interface
icons, camps, the Guardian's standard, the free companies' banners, mists).

The music is composed by `tools/sound/gen_music.py`: MIDI files for the game's synthesizer, a
tune per screen and one per race for audiences. `tools/sound/gen_sfx.py` renders the sound
effects (tones and filtered noise) to WAV files for both themes.

```sh
python3 tools/art/gen_svg.py
python3 tools/sound/gen_music.py
python3 tools/sound/gen_sfx.py
python3 tools/themes_check.py   # what each theme lacks of Classic's files
```

## Race portraits

`themes/<age|classic>/images/race/<n>.webp` (n in race order: Human, Mrrshan, Silicoid,
Sakkra, Psilon, Alkari, Klackon, Bulrathi, Meklar, Darlok).

In the Age theme each player is a real medieval empire, painted with a hint of the MOO1
race it stands for:

| Race | Empire | Ruler | The race in the portrait |
| --- | --- | --- | --- |
| Human | Holy Roman Empire | Frederick Barbarossa | diplomat with a sealed charter |
| Mrrshan | Mongol Empire | Genghis Khan | amber cat's eyes, snow leopard fur |
| Silicoid | Ghurid Empire | Muhammad of Ghor | stone-grey skin, a turban set with uncut stones |
| Sakkra | Khmer Empire | Suryavarman II | a crown of naga serpents, scale-patterned gold |
| Psilon | Abbasid Caliphate | al-Ma'mun | a tall turban, an astrolabe, the House of Wisdom |
| Alkari | North Sea Empire | Cnut the Great | raven-feather cloak, raven on the shoulder |
| Klackon | Song Empire | Shenzong | beetle-shell lacquer armour, the official's hat with long antenna-like flaps |
| Bulrathi | First Bulgarian Empire | Krum | bear fur, huge build |
| Meklar | Byzantine Empire | Theophilos | golden automaton lion |
| Darlok | Republic of Venice | Enrico Dandolo | a carnival mask by candlelight |

The Classic theme keeps MOO1's aliens. What each empire takes from its race:
[MOO1.md](MOO1.md).

They are made with Google's Nano Banana 2.1 (`gemini-nano-banana-2.1`) by
`tools/art/gen_gemini.py portraits/` from the descriptions in `tools/art/subjects.py`; the
first portrait of each theme is the style reference for the other nine.
`tools/art/finish_portraits.py` crops them to 384x480 WebP (`PICK` chooses among tries). Google's terms leave the generated pictures to the user; every
picture carries Google's invisible SynthID watermark.

```sh
GEMINI_API_KEY=... python3 tools/art/gen_gemini.py portraits/       # all 20, about $0.70
GEMINI_API_KEY=... TRY=2 python3 tools/art/gen_gemini.py portraits/age-8
/tmp/sd/bin/python tools/art/finish_portraits.py /tmp/art/gemini/portraits
```

`tools/art/gen_portraits.py` makes them with a local Stable Diffusion 1.5 model instead, with
no API key, at lower quality (the portraits of 0.10.0 were made that way):

- model: [Dreamshaper 8](https://huggingface.co/Lykon/dreamshaper-8) (CreativeML Open RAIL-M)
- sampler: [LCM-LoRA SD 1.5](https://huggingface.co/latent-consistency/lcm-lora-sdv1-5), guidance
  1.5, 512x640, 4 to 6 steps; about 2 minutes a portrait on ten CPU cores

Neither licence claims rights over generated pictures; the Open RAIL-M use restrictions
apply to the model, not to the pictures.

```sh
python3 -m venv /tmp/sd
/tmp/sd/bin/pip install --index-url https://download.pytorch.org/whl/cpu torch
/tmp/sd/bin/pip install diffusers transformers accelerate safetensors peft pillow
SEED=7 THREADS=10 /tmp/sd/bin/python tools/art/gen_portraits.py
```

## Age theme: map, city panel and battles

Made with Google's Nano Banana 2.1 (`gemini-nano-banana-2.1`, Gemini API) by
`tools/art/gen_gemini.py`, from the subjects in `tools/art/subjects.py`. The troops, map
markers and coats of arms are drawn with one approved picture of their kind as a style
reference (the halberdiers, the castle, the Holy Roman arms), so each set looks alike.
`tools/art/finish_art.py` crops them, cuts the sprites out of their white background and
writes WebP files to `themes/age/images/`, each named as its picture.
`tools/art/gen_art.py` makes the same set with the local Stable Diffusion model, without an
API key, at lower quality.

| Name | Picture | Where |
| --- | --- | --- |
| `view/<land>`, `view/orion` | 14 landscapes (one per kind of land) and the ruins of Persepolis, 768x384 | top of the side panel when a site is selected |
| `view/city/0..2` | a village, a walled town, a great city | top of the panel of a city, by population (under 25, under 60, more) |
| `battle/field/<kind>` | 8 battlefields seen from above: green, forest, dry, snow, rocky, ash, marsh, mists | behind the battle grid, by the land of the battle's site |
| `unit/0..23` | 24 troop types (6 per unit size: band, company, battalion, host) | battle stacks, by the design's look; a pennant in the empire's colour is drawn over them |
| `unit/crystal`, `unit/amoeba`, `unit/guardian` | the Catalan Company, the Great Company, the Immortals | battles against them |
| `battle/rock/0..3` | a boulder, a ruined wall, trees, a pile of stones | obstacles on the battle grid |
| `mark/village`, `mark/town`, `mark/castle` | cities by size | map markers and the defended city in battles, with the owner's pennant |
| `army`, `caravan`, `stargate` | a knight with a banner, an ox wagon, a canal lock | armies, migrants and levies, canal ports on the map |
| `arms/<race>` | the ten empires' coats of arms | city panel (owner) and the empires screen |
| `unit/<look>/march/0..3`, `unit/<look>/attack/0..3` | four frames of each troop marching and attacking, drawn from its still picture as the reference | battle stacks while they move and when they shoot or strike |
| `landing/invade/<land>` | 14 sieges: a besieged city's walls behind an empty battlefield, one per kind of land | the siege screen (ground battles) |
| `ground/soldier/<race>`, `ground/death` | a foot soldier of each empire, a fallen soldier | the siege screen |
| `news/herald`, `news/icon/<type>` | the herald, and an illuminated miniature for each of the 22 kinds of news | the Herald |
| `hall/<race>` | the throne room of each empire | behind the envoy in audiences and messages |
| `council/hall` | the hall of the Great Council | the council's vote and verdict |

Missiles (javelins, axes, darts, fire arrows, stones, rockets, fire ships) and the dust of a
hit are drawn by the battle screen itself (`web/src/battle.ts`).

```sh
GEMINI_API_KEY=... python3 tools/art/gen_gemini.py              # everything, about $2.50
GEMINI_API_KEY=... TRY=2 python3 tools/art/gen_gemini.py unit-04  # another try of one picture
/tmp/sd/bin/python tools/art/finish_art.py /tmp/art/gemini      # PICK chooses among tries
```

Google's terms leave the generated pictures to the user; every picture carries Google's
invisible SynthID watermark.

## Mods

A mod overrides any picture with `images/<name>.png` (`docs/MODDING.md`), with the names
above: `race/<n>` for portraits, `unit/<look>` for battle troops, and so on.
