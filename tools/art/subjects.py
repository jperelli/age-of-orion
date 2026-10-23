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

# Siege screen (ground battles): the land in front of a besieged city, by kind of land.
SIEGES = {
    "not_habitable": "a mountain pass below snowy peaks",
    "radiated": "a blighted plain of grey ash and burnt tree stumps",
    "toxic": "a foggy swamp with dead trees",
    "inferno": "a volcanic plain with smoking cracks and black rocks",
    "dead": "red eroded badlands",
    "tundra": "a frozen snowy plain",
    "barren": "bare stony hills",
    "minimal": "dry scrubland",
    "desert": "a sandy desert with dunes",
    "steppe": "an open grassy steppe",
    "arid": "a golden savanna with acacia trees",
    "ocean": "a sea shore with the sea beside the walls",
    "jungle": "a jungle clearing among huge trees",
    "terran": "green farmland with fields and hedges",
}
# A foot soldier of each empire for the siege screen, by race order.
SOLDIERS = {
    "human": "a German imperial foot soldier in mail and a kettle hat with a spear and a heater shield bearing a black eagle on gold",
    "mrrshan": "a Mongol warrior in lamellar armour and a fur-trimmed helmet with a sabre and a small round shield",
    "silicoid": "a Ghurid infantryman in mail with a turban wrapped around his helmet, a spear and a round shield",
    "sakkra": "a Khmer soldier with a bare chest, a gold collar, a spear and a round shield",
    "psilon": "an Abbasid soldier in black robes and mail with a lance and a round shield",
    "alkari": "a Danish huscarl with a long Dane axe, a nasal helmet and a round shield painted with a raven",
    "klackon": "a Song dynasty Chinese infantryman in black lacquered lamellar armour with a halberd",
    "bulrathi": "a Bulgar warrior with a bear fur cloak, a conical helmet and a spear",
    "meklar": "a Byzantine infantryman with a long oval shield, a scale coat and a spear",
    "darlok": "a Venetian crossbowman in a red tunic and a sallet helmet with a crossbow",
}
FALLEN = "a fallen medieval soldier lying on the ground on his side, his shield and spear dropped beside him"
HERALD = "a medieval herald in a colourful quartered tabard reading a proclamation from an unrolled scroll"
# The Herald's pictures of the news (1oom GAME_NEWS_* types), as illuminated manuscript miniatures.
NEWS = {
    "plague": "the plague in a medieval town, the sick in their beds and a doctor",
    "quake": "an earthquake bringing down the houses and towers of a medieval town",
    "nova": "a volcano erupting above a medieval town",
    "accident": "a great fire in the workshops of a medieval town, black smoke over the fields",
    "assassin": "an envoy stabbed by an assassin in a palace hall",
    "virus": "a great library on fire, scholars carrying books out",
    "comet": "a great flood sweeping over fields and villages, people building dikes",
    "pirates": "bandits ambushing a merchant caravan on a road through a forest",
    "derelict": "soldiers discovering an abandoned armoury full of weapons",
    "rebellion": "peasants in revolt storming the gate of a town with pitchforks and torches",
    "crystal": "mercenaries under a red and gold striped banner sacking a town",
    "amoeba": "a mercenary company in black and white plundering a village",
    "enviro": "a rich harvest, peasants reaping golden wheat under a kind sun",
    "rich": "miners finding a rich vein of silver ore in a mountain",
    "support": "merchants presenting chests of gold coins to a king",
    "poor": "miners leaving an exhausted mine with empty carts",
    "orion": "settlers entering the ruined columned palace of Persepolis",
    "coup": "nobles seizing the throne in a palace coup, the old king led away",
    "stars": "a great empire's banners raised over many cities on a map",
    "stats": "a herald announcing a ranking of kingdoms to a crowd in a town square",
    "genocide": "a burning capital and its fallen banner, the end of an empire",
    "guardian": "knights defeating the Persian Immortals among ancient columns",
}
# Throne rooms behind the envoys of each empire, by race order.
HALLS = {
    "human": "the throne hall of the Holy Roman Emperor in a Romanesque imperial palace, round arches, the black eagle on gold banners",
    "mrrshan": "the great felt tent of a Mongol khan with a golden throne, carpets, furs and horsehair standards",
    "silicoid": "the stone throne hall of a Ghurid sultan in a mountain fortress, carved brick arches and lapis tiles",
    "sakkra": "the throne hall of a Khmer king at Angkor, carved sandstone pillars, naga balustrades and parasols",
    "psilon": "the audience hall of the Abbasid caliph in Baghdad, a domed hall with arches, books and black banners",
    "alkari": "the timber great hall of a Danish king, carved pillars, a long hearth, shields and raven banners",
    "klackon": "the throne hall of a Song emperor, red lacquered columns, a dragon throne and painted screens",
    "bulrathi": "the stone throne hall of a Bulgar khan at Pliska, heavy columns, furs and torches",
    "meklar": "the golden throne hall of the Byzantine emperor in Constantinople, mosaics, purple curtains and golden automaton lions",
    "darlok": "the Hall of the Great Council in the Doge's Palace in Venice, gilded ceiling, paintings and candles",
}
COUNCIL = "a great medieval council hall where envoys of many nations sit in a half circle under their banners before an empty throne"
