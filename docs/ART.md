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

## Mods

A mod overrides any picture with `images/<alias>.png` (`docs/MODDING.md`): `race/<n>` for
portraits.
