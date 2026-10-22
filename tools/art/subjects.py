"""What the Age theme's map, city panel and battle pictures show, shared by the local
generator (gen_art.py) and the Nano Banana one (gen_gemini.py)."""

VIEWS = {
    "terran": "a fertile green valley with wheat fields, a river and orchards",
    "ocean": "a rocky sea coast with a sandy bay and fishing boats",
    "jungle": "a dense tropical jungle with a river and huge trees",
    "arid": "a golden savanna with acacia trees and tall dry grass",
    "steppe": "a vast grassy steppe under a wide sky, distant yurts",
    "desert": "sand dunes in a hot desert with a small oasis of palm trees",
    "minimal": "dry scrubland with thorny bushes and sparse grass",
    "barren": "bare grey stony hills with boulders, no grass, no trees, dry and lifeless",
    "tundra": "a frozen snowy tundra with a few dark pine trees",
    "dead": "eroded red badlands with canyons and mesas",
    "inferno": "volcanic fields with glowing lava flows and smoke",
    "toxic": "a foggy poisonous swamp with dead grey trees, sickly green water and bubbles",
    "radiated": "a desolate blighted wasteland of grey ash, burnt black tree stumps, no grass, smoke under an overcast sky",
    "not_habitable": "towering impassable snowy mountain peaks and glaciers",
    "orion": "the ruins of Persepolis, ancient Persian palace columns and stairways on a terrace",
    "city0": "a small medieval village with thatched cottages, a wooden palisade and fields",
    "city1": "a medieval walled town with a stone church tower, market square and red roofs",
    "city2": "a great medieval walled city with a castle, many towers and domes, seen from a hill",
}
FIELDS = {
    "green": "a green meadow with short grass, dirt patches and a few bushes",
    "forest": "a grassy clearing with small groups of trees and bushes",
    "dry": "dry yellow grass and sandy earth with pebbles",
    "snow": "a snowy field with patches of frozen grass",
    "rocky": "grey rocky ground with stones and gravel",
    "ash": "dark volcanic ash ground with cracks and black rocks",
    "marsh": "muddy marsh ground with puddles and reeds",
    "mists": "a green meadow half hidden by white fog",
}
UNITS = [
    # Band (small)
    "three medieval spearmen with round shields",
    "three medieval archers with longbows",
    "two light horsemen scouts",
    "three swordsmen with kite shields and mail",
    "three slingers in tunics",
    "three axemen in chain mail",
    # Company (medium)
    "five medieval crossbowmen",
    "four light cavalry riders with lances",
    "six pikemen with long pikes",
    "four men-at-arms in mail with swords and shields",
    "four mounted horse archers",
    "three camel riders with spears",
    # Battalion (large)
    "four heavy armored knights on barded horses with lances",
    "a dense formation of medieval infantry soldiers holding tall halberds, with a banner",
    "a war elephant carrying soldiers",
    "four cataphracts, fully armored horsemen",
    "a row of medieval archers drawing longbows",
    "a medieval wooden war wagon with crossbowmen, pulled by two horses",
    # Host (huge)
    "a large wooden trebuchet siege catapult with a long throwing arm",
    "a tall wooden siege tower on wheels",
    "a host of knights on horseback with many banners",
    "two war elephants with howdahs",
    "a medieval wooden battering ram with a pointed wooden roof on wooden wheels, pushed by soldiers",
    "a bronze bombard cannon on a wooden carriage with gunners",
]
SPECIAL = {
    "crystal": "a band of almogavar mercenaries with javelins under a red and gold striped banner",
    "amoeba": "a company of mercenary soldiers in black and white with halberds",
    "guardian": "a phalanx of ancient Persian Immortals, infantry with long spears and wicker shields, a golden eagle standard",
}
ROCKS = ["a large grey granite boulder", "a ruined stone wall fragment", "a cluster of three oak trees", "a pile of rocks and stones"]
MARKS = {
    "village": "a small medieval village of thatched cottages",
    "town": "a small medieval town with stone walls and a church tower",
    "castle": "a medieval castle with towers and walls",
    "army": "a mounted medieval knight carrying a banner",
    "caravan": "a covered wooden wagon pulled by two oxen",
    "canal": "a small stone canal lock with wooden gates and a moored boat",
}
ARMS_P = {
    "human": "a black eagle on a gold field",
    "mrrshan": "a white horse on a sky blue field",
    "silicoid": "a golden mountain on a green field",
    "sakkra": "a golden many-headed cobra on a dark green field",
    "psilon": "a plain black field with a golden border",
    "alkari": "a black raven in profile standing with folded wings, on a white field",
    "klackon": "a golden Chinese dragon on a black field",
    "bulrathi": "a golden lion on a dark red field",
    "meklar": "a golden double-headed eagle on a purple field",
    "darlok": "a golden winged lion on a red field",
}

RACES = ["human", "mrrshan", "silicoid", "sakkra", "psilon", "alkari", "klackon", "bulrathi", "meklar", "darlok"]
# Race portraits (gen_portraits.py, gen_gemini.py): the Age theme's rulers, each with a hint of
# the MOO1 race its empire stands for, and the Classic theme's aliens.
PORTRAITS_AGE = {
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
    "klackon": "Song dynasty Chinese emperor Shenzong wearing the black futou official's hat with two long, thin, straight horizontal flaps sticking out to the sides like antennae, glossy black lacquered lamellar armour like a beetle shell over a yellow silk robe, porcelain kilns in the background",
    # First Bulgarian Empire / Bulrathi: bear warriors.
    "bulrathi": "Bulgar khan Krum, huge broad shouldered warrior, thick brown bear fur cloak, iron conical helmet, heavy dark beard, wooden palisade fortress background",
    # Byzantine Empire / Meklar: machines.
    "meklar": "Byzantine emperor Theophilos, a slim man with olive skin and a short dark brown beard, a Byzantine stemma crown with strings of pearls hanging at the sides, purple and gold robes with jewelled loros, beside a golden mechanical automaton lion with gears, Constantinople palace background",
    # Republic of Venice / Darloks: spies and shapeshifters.
    "darlok": "Doge of Venice wearing the corno ducale cap, holding a white Venetian carnival mask, black cloak, secretive smile, candlelight, Venetian canal at night background",
}
PORTRAITS_CLASSIC = {
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
