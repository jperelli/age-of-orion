"""Race portraits for Age of Orion, generated locally (Dreamshaper 8 + LCM LoRA, CPU)."""
import gc, os, sys, time, torch
from diffusers import StableDiffusionPipeline, LCMScheduler

torch.set_num_threads(int(os.environ.get("THREADS", "2")))
pipe = StableDiffusionPipeline.from_pretrained("Lykon/dreamshaper-8", torch_dtype=torch.float32, safety_checker=None)
pipe.scheduler = LCMScheduler.from_config(pipe.scheduler.config)
pipe.load_lora_weights("latent-consistency/lcm-lora-sdv1-5")
pipe.fuse_lora()
pipe.unload_lora_weights()  # keep only the fused weights: the sandbox has 4 GB
pipe.enable_attention_slicing()
pipe.vae.enable_slicing()

NEG = "blurry, lowres, text, watermark, signature, frame, deformed, extra fingers, cropped head, multiple people"
AGE = ", head and shoulders portrait, medieval fantasy, painted, oil painting, dramatic rim lighting, dark background, highly detailed, strategy game character portrait"
SCI = ", head and shoulders portrait, science fiction, spaceship bridge background, painted, dramatic lighting, highly detailed, 4x space strategy game character portrait"
RACES = ["human", "mrrshan", "silicoid", "sakkra", "psilon", "alkari", "klackon", "bulrathi", "meklar", "darlok"]
AGE_P = {
    "human": "regal merchant queen with a golden crown, red velvet and ermine cloak, auburn hair",
    "mrrshan": "fierce steppe khan warlord, fur hat, lamellar armor, braided beard, horse archer",
    "silicoid": "ancient stone golem elder, cracked granite skin, glowing crystal eyes, moss beard",
    "sakkra": "reptilian lizardman chieftain, crocodile head with green scales and yellow eyes, feather headdress, bone necklace, not human",
    "psilon": "wise archmage woman with silver hair, blue robes with glowing runes, crystal diadem",
    "alkari": "noble falconer lord with a falcon perched on his shoulder, feathered cloak, leather bracers",
    "klackon": "insectoid hive queen, mantis head with compound eyes and mandibles, chitin crown, amber armor, not human",
    "bulrathi": "bear-headed warrior jarl, brown bear face, bear pelt cloak, iron helmet, runes, not human",
    "meklar": "master engineer with brass goggles, clockwork mechanical arm, leather apron, gears, steampunk",
    "darlok": "mysterious veiled assassin in a dark hood, face hidden by a black veil, glowing eyes in the shadows",
}
SCI_P = {
    "human": "human space admiral in a dark blue uniform with gold insignia, grey temples",
    "mrrshan": "lion-like feline alien admiral, orange fur and mane, sci-fi uniform, not human",
    "silicoid": "crystalline rock alien made of glowing purple minerals and crystals, not human",
    "sakkra": "reptilian lizard alien warlord, green scales, frilled crest, sci-fi armor, not human",
    "psilon": "alien scientist with an enormous bald bulging cranium, huge black almond eyes, small mouth, pale blue-grey skin, purple robe, not human",
    "alkari": "bird-like avian alien with a hawk head and grey feathers, sci-fi flight suit, not human",
    "klackon": "giant ant alien, insect head with big mandibles, antennae and compound eyes, glossy red-brown chitin, not human, not a robot",
    "bulrathi": "bear-like alien warrior in sci-fi power armor, brown fur, not human",
    "meklar": "cyborg robot alien, metal face with glowing blue optics, cables, not human",
    "darlok": "shapeshifter alien spy with dark grey skin, black hood, glowing white eyes, not human",
}
jobs = []
for i, r in enumerate(RACES):
    jobs.append((f"age-{i}-{r}", AGE_P[r] + AGE))
    jobs.append((f"classic-{i}-{r}", SCI_P[r] + SCI))
only = sys.argv[1:]
seed = int(os.environ.get("SEED", "7"))
out = os.environ.get("OUT", "/tmp/art/portraits")
os.makedirs(out, exist_ok=True)
jobs = [(n, p) for n, p in jobs if (not only or any(n.startswith(o) for o in only)) and not os.path.exists(f"{out}/{n}-s{seed}.png")]
# Encode every prompt first, then drop the text encoder (about 500 MB) to fit in 4 GB.
with torch.no_grad():
    embeds = {n: pipe.encode_prompt(p, "cpu", 1, True, NEG) for n, p in jobs}
pipe.text_encoder = None
gc.collect()
for name, prompt in jobs:
    path = f"{out}/{name}-s{seed}.png"
    t = time.time()
    pe, ne = embeds[name]
    img = pipe(prompt_embeds=pe, negative_prompt_embeds=ne, num_inference_steps=4, guidance_scale=1.5, width=512,
               height=640, generator=torch.Generator().manual_seed(seed)).images[0]
    img.save(path)
    print(f"{name} {time.time() - t:.0f}s", flush=True)
