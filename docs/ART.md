# Art

The pictures shipped with the themes are made for Age of Orion. None comes from Master of
Orion: the Classic theme shows MOO1's own pictures only when the player imports their LBX
files, and those take the place of the shipped ones.

## Race portraits

`web/public/themes/<age|classic>/images/race/<n>.webp` (n in race order: Human, Mrrshan,
Silicoid, Sakkra, Psilon, Alkari, Klackon, Bulrathi, Meklar, Darlok), listed under `images`
in each `theme.json`.

They were generated on a CPU with Stable Diffusion 1.5:

- model: [Dreamshaper 8](https://huggingface.co/Lykon/dreamshaper-8) (CreativeML Open RAIL-M)
- sampler: [LCM-LoRA SD 1.5](https://huggingface.co/latent-consistency/lcm-lora-sdv1-5), 4 steps,
  guidance 1.5, 512x640
- prompts and seeds: `tools/art/gen_portraits.py`
- finishing (crop, resize to 384x480, WebP, `theme.json`): `tools/art/finish_portraits.py`

Neither licence claims rights over generated pictures. The Open RAIL-M use restrictions
apply to the model, not to the pictures.

To make them again or try other seeds:

```sh
python3 -m venv /tmp/sd
/tmp/sd/bin/pip install --index-url https://download.pytorch.org/whl/cpu torch
/tmp/sd/bin/pip install diffusers transformers accelerate safetensors peft pillow
SEED=7 /tmp/sd/bin/python tools/art/gen_portraits.py            # all 20
SEED=11 /tmp/sd/bin/python tools/art/gen_portraits.py age-3      # one portrait
/tmp/sd/bin/python tools/art/finish_portraits.py /tmp/art/portraits
```

A portrait takes 2 to 4 minutes on two CPU cores, seconds on a GPU. `PICK` in
`finish_portraits.py` chooses the seed of each portrait when several were made.

## Mods

A mod overrides any picture with `images/<alias>.png` (`docs/MODDING.md`): `race/<n>` for
portraits.
