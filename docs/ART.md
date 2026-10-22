# Art

The pictures shipped with the themes are made for Age of Orion. None comes from Master of
Orion: the Classic theme shows MOO1's own pictures only when the player imports their LBX
files, and those take the place of the shipped ones.

## Race portraits

`web/public/themes/<age|classic>/images/race/<n>.webp` (n in race order: Human, Mrrshan,
Silicoid, Sakkra, Psilon, Alkari, Klackon, Bulrathi, Meklar, Darlok), listed under `images`
in each `theme.json`.

In the Age theme each player is a real medieval empire, painted with a hint of the MOO1
race it stands for:

| Race | Empire | Ruler | Hint of the race asked of the model |
| --- | --- | --- | --- |
| Human | Holy Roman Empire | Frederick Barbarossa | diplomat with a sealed charter |
| Mrrshan | Mongol Empire | Genghis Khan | amber cat's eyes, snow leopard fur |
| Silicoid | Ghurid Empire | Muhammad of Ghor | stone-grey skin, uncut crystals |
| Sakkra | Khmer Empire | Suryavarman II | naga serpent crown, scale-patterned gold |
| Psilon | Abbasid Caliphate | al-Ma'mun | very tall turban, astrolabe, House of Wisdom |
| Alkari | North Sea Empire | Cnut the Great | raven-feather cloak, raven on the shoulder |
| Klackon | Song Empire | Shenzong | beetle-shell lacquer armour, antenna-like hat wings |
| Bulrathi | First Bulgarian Empire | Krum | bear fur, huge build |
| Meklar | Byzantine Empire | Theophilos | golden automaton lion |
| Darlok | Republic of Venice | Enrico Dandolo | carnival mask |

The Classic theme keeps MOO1's aliens. What each empire takes from its race:
[MOO1.md](MOO1.md).

They were generated on a CPU with Stable Diffusion 1.5:

- model: [Dreamshaper 8](https://huggingface.co/Lykon/dreamshaper-8) (CreativeML Open RAIL-M)
- sampler: [LCM-LoRA SD 1.5](https://huggingface.co/latent-consistency/lcm-lora-sdv1-5), guidance
  1.5, 512x640; 6 steps for the Age theme, 4 for Classic
- prompts and seeds: `tools/art/gen_portraits.py`
- finishing (crop, resize to 384x480, WebP, `theme.json`): `tools/art/finish_portraits.py`

Neither licence claims rights over generated pictures. The Open RAIL-M use restrictions
apply to the model, not to the pictures.

To make them again or try other seeds:

```sh
python3 -m venv /tmp/sd
/tmp/sd/bin/pip install --index-url https://download.pytorch.org/whl/cpu torch
/tmp/sd/bin/pip install diffusers transformers accelerate safetensors peft pillow
SEED=7 THREADS=10 /tmp/sd/bin/python tools/art/gen_portraits.py              # all 20
SEED=11 STEPS=6 /tmp/sd/bin/python tools/art/gen_portraits.py age-3      # one portrait
/tmp/sd/bin/python tools/art/finish_portraits.py /tmp/art/portraits
```

A portrait takes about 2 minutes on ten CPU cores (6 steps), seconds on a GPU. `PICK` in
`finish_portraits.py` chooses the seed of each portrait when several were made.

## Age theme: map, city panel and battles

Made with Google's Nano Banana 2.1 (`gemini-nano-banana-2.1`, Gemini API) by
`tools/art/gen_gemini.py`, from the subjects in `tools/art/subjects.py`. The troops, map
markers and coats of arms are drawn with one approved picture of their kind as a style
reference (the halberdiers, the castle, the Holy Roman arms), so each set looks alike.
`tools/art/finish_art.py` crops them, cuts the sprites out of their white background, writes
WebP files to `web/public/themes/age/images/` and lists them in `theme.json`.
`tools/art/gen_art.py` makes the same set with the local Stable Diffusion model, without an
API key, at lower quality.

| Alias | Picture | Where |
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

A mod overrides any picture with `images/<alias>.png` (`docs/MODDING.md`), with the aliases
above: `race/<n>` for portraits, `unit/<look>` for battle troops, and so on.
