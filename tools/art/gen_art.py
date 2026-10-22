"""Age theme pictures for the map, the city panel and the battle screen, generated locally
(Dreamshaper 8 + LCM LoRA, CPU), like gen_portraits.py.

    SEED=7 THREADS=10 python tools/art/gen_art.py [name prefix...]

Writes OUT/<name>-s<seed>.png; finish_art.py turns the chosen ones into theme pictures.
Sprites (units, rocks, map markers, coats of arms) are painted on white for finish_art.py to
cut out.
"""
import gc, os, sys, time, torch
from diffusers import StableDiffusionPipeline, LCMScheduler

torch.set_num_threads(int(os.environ.get("THREADS", "10")))
pipe = StableDiffusionPipeline.from_pretrained("Lykon/dreamshaper-8", torch_dtype=torch.float32, safety_checker=None)
pipe.scheduler = LCMScheduler.from_config(pipe.scheduler.config)
pipe.load_lora_weights("latent-consistency/lcm-lora-sdv1-5")
pipe.fuse_lora()
pipe.unload_lora_weights()
pipe.enable_attention_slicing()
pipe.vae.enable_slicing()

NEG = "blurry, lowres, text, watermark, signature, frame, border, deformed, fantasy, dragon, magic, sci-fi, modern, anime, cartoon"
VIEW = ", medieval landscape oil painting, wide panorama, golden hour light, highly detailed"
FIELD = ", hand-painted top-down battle map for a medieval strategy game, painterly texture, muted colors, flat even lighting, no perspective, no horizon, no people, no buildings"
SPRITE = ", full body, side view, medieval concept art, painted illustration, isolated on a plain white background, centered, no ground, no scenery"
SPRITE_NEG = NEG + ", miniature, diorama, base, platform, tabletop, isometric, landscape, scenery, ground, grass, sky, horizon, shadow, cropped, multiple scenes"
# Map markers look like the tokens of a board game: a small diorama seen from above.
MARK = ", painted miniature for a strategy game map, seen from above at an angle, isometric view, isolated on a plain white background, centered"
MARK_NEG = NEG + ", landscape, scenery, sky, horizon, cropped, multiple scenes"
ARMS = ", medieval heraldic coat of arms on a shield, flat heraldry illustration, bold simple shapes, isolated on a plain white background, centered"

from subjects import ARMS_P, FIELDS, MARKS, ROCKS, SPECIAL, UNITS, VIEWS  # noqa: E402

jobs = []  # name, prompt, negative, width, height
for k, p in VIEWS.items():
    jobs.append((f"view-{k}", p + VIEW, NEG + ", people, crowd", 768, 384))
for k, p in FIELDS.items():
    jobs.append((f"field-{k}", p + FIELD, NEG + ", people, soldiers, horizon, sky, buildings", 800, 480))
for i, p in enumerate(UNITS):
    jobs.append((f"unit-{i:02}", p + SPRITE, SPRITE_NEG, 512, 384))
for k, p in SPECIAL.items():
    jobs.append((f"unit-{k}", p + SPRITE, SPRITE_NEG, 512, 384))
for i, p in enumerate(ROCKS):
    jobs.append((f"rock-{i}", p + SPRITE, SPRITE_NEG + ", people", 512, 384))
for k, p in MARKS.items():
    jobs.append((f"mark-{k}", p + MARK, MARK_NEG, 512, 512))
for k, p in ARMS_P.items():
    jobs.append((f"arms-{k}", p + ARMS, NEG + ", text, letters, photo, 3d", 512, 512))

only = sys.argv[1:]
seed = int(os.environ.get("SEED", "7"))
steps = int(os.environ.get("STEPS", "6"))
out = os.environ.get("OUT", "/tmp/art/scenes")
os.makedirs(out, exist_ok=True)
jobs = [j for j in jobs if (not only or any(j[0].startswith(o) for o in only)) and not os.path.exists(f"{out}/{j[0]}-s{seed}.png")]
with torch.no_grad():
    embeds = {j[0]: pipe.encode_prompt(j[1], "cpu", 1, True, j[2]) for j in jobs}
pipe.text_encoder = None
gc.collect()
for name, _, _, w, h in jobs:
    t = time.time()
    pe, ne = embeds[name]
    img = pipe(prompt_embeds=pe, negative_prompt_embeds=ne, num_inference_steps=steps, guidance_scale=1.5, width=w,
               height=h, generator=torch.Generator().manual_seed(seed)).images[0]
    img.save(f"{out}/{name}-s{seed}.png")
    print(f"{name} {time.time() - t:.0f}s", flush=True)
