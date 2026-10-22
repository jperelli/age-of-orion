"""Race portraits for Age of Orion, generated locally (Dreamshaper 8 + LCM LoRA, CPU).

    SEED=7 THREADS=10 python tools/art/gen_portraits.py [name prefix...]
"""
import gc, os, sys, time, torch
from diffusers import StableDiffusionPipeline, LCMScheduler

torch.set_num_threads(int(os.environ.get("THREADS", "2")))
pipe = StableDiffusionPipeline.from_pretrained("Lykon/dreamshaper-8", torch_dtype=torch.float32, safety_checker=None)
pipe.scheduler = LCMScheduler.from_config(pipe.scheduler.config)
pipe.load_lora_weights("latent-consistency/lcm-lora-sdv1-5")
pipe.fuse_lora()
pipe.unload_lora_weights()  # keep only the fused weights: saves memory
pipe.enable_attention_slicing()
pipe.vae.enable_slicing()

NEG = "blurry, lowres, text, watermark, signature, frame, deformed, extra fingers, cropped head, multiple people"
# The Age theme: real medieval empires, each with a hint of the MOO1 race it stands for.
AGE_NEG = NEG + ", fantasy, elf, pointed ears, monster, creature, animal head, horns, glowing eyes, magic, sci-fi, anime, cartoon"
AGE = ", head and shoulders portrait, medieval history, historically accurate costume, oil painting, realistic, dramatic chiaroscuro lighting, dark background, highly detailed"
SCI = ", head and shoulders portrait, science fiction, spaceship bridge background, painted, dramatic lighting, highly detailed, 4x space strategy game character portrait"
from subjects import PORTRAITS_AGE as AGE_P, PORTRAITS_CLASSIC as SCI_P, RACES  # noqa: E402

# Christian symbols kept off the rulers of the empires that were not Christian.
NOT_CHRISTIAN = {"mrrshan", "silicoid", "sakkra", "psilon", "klackon"}
jobs = []
for i, r in enumerate(RACES):
    jobs.append((f"age-{i}-{r}", AGE_P[r] + AGE, AGE_NEG + (", christian cross, crucifix" if r in NOT_CHRISTIAN else "")))
    jobs.append((f"classic-{i}-{r}", SCI_P[r] + SCI, NEG))
only = sys.argv[1:]
seed = int(os.environ.get("SEED", "7"))
out = os.environ.get("OUT", "/tmp/art/portraits")
os.makedirs(out, exist_ok=True)
steps = int(os.environ.get("STEPS", "4"))
jobs = [j for j in jobs if (not only or any(j[0].startswith(o) for o in only)) and not os.path.exists(f"{out}/{j[0]}-s{seed}.png")]
# Encode every prompt first, then drop the text encoder (about 500 MB of memory).
with torch.no_grad():
    embeds = {n: pipe.encode_prompt(p, "cpu", 1, True, neg) for n, p, neg in jobs}
pipe.text_encoder = None
gc.collect()
for name, _, _ in jobs:
    path = f"{out}/{name}-s{seed}.png"
    t = time.time()
    pe, ne = embeds[name]
    img = pipe(prompt_embeds=pe, negative_prompt_embeds=ne, num_inference_steps=steps, guidance_scale=1.5, width=512,
               height=640, generator=torch.Generator().manual_seed(seed)).images[0]
    img.save(path)
    print(f"{name} {time.time() - t:.0f}s", flush=True)
