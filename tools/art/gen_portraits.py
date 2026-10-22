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
RACES = ["human", "mrrshan", "silicoid", "sakkra", "psilon", "alkari", "klackon", "bulrathi", "meklar", "darlok"]
AGE_P = {
    # Holy Roman Empire / Humans: diplomats and traders.
    "human": "Holy Roman Emperor Frederick Barbarossa, red beard, jewelled imperial crown, ermine and gold brocade mantle, holding a sealed charter, gothic cathedral background",
    # Mongol Empire / Mrrshan: feline warriors.
    "mrrshan": "Mongol khan Genghis Khan, fierce amber eyes like a big cat, snow leopard fur collar, lamellar armor, fur-trimmed helmet, steppe background",
    # Ghurid Empire / Silicoids: stone people who live anywhere.
    "silicoid": "Ghurid sultan of the Afghan mountains, weathered grey skin like stone, stern unmoving face, white turban set with uncut crystals and lapis lazuli, chain mail, mountain fortress background",
    # Khmer Empire / Sakkra: reptiles who multiply fast.
    "sakkra": "Khmer king of Angkor, tall golden crown shaped like a seven-headed naga serpent, bronze skin, gold collar patterned like snake scales, jade jewelry, Angkor Wat temple background",
    # Abbasid Caliphate / Psilons: scientists.
    "psilon": "Abbasid caliph al-Mamun, very tall white turban, wise piercing eyes, holding a brass astrolabe, House of Wisdom library with manuscripts background",
    # North Sea Empire / Alkari: bird people, swift and hard to hit.
    "alkari": "Danish Viking king Cnut the Great, cloak of black raven feathers, a raven perched on his shoulder, gold crown, braided blond beard, longship sails background",
    # Song Empire / Klackons: tireless insect workers.
    "klackon": "Song dynasty Chinese emperor, black futou hat with two long straight horizontal flaps like antennae, glossy black lacquered lamellar armor like a beetle shell, yellow silk robe, porcelain kilns background",
    # First Bulgarian Empire / Bulrathi: bear warriors.
    "bulrathi": "Bulgar khan Krum, huge broad shouldered warrior, thick brown bear fur cloak, iron conical helmet, heavy dark beard, wooden palisade fortress background",
    # Byzantine Empire / Meklar: machines.
    "meklar": "Byzantine emperor Theophilos, purple and gold robes with jewelled loros, pearl crown, beside a golden mechanical automaton lion with gears, Constantinople palace background",
    # Republic of Venice / Darloks: spies and shapeshifters.
    "darlok": "Doge of Venice wearing the corno ducale cap, holding a white Venetian carnival mask, black cloak, secretive smile, candlelight, Venetian canal at night background",
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
