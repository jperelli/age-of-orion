# Age of Orion and Master of Orion

Age of Orion plays by the rules of **Master of Orion** (SimTex, 1993). The rules are ported
formula by formula from [1oom](https://github.com/1oom-fork/1oom), the open-source engine
that reproduces MOO1 v1.3, and checked turn by turn against it. The screens, the texts, the
pictures and the setting of the Age theme are new.

There are two themes with the same rules:

- **Classic**: MOO1's own setting of stars, planets, ships and alien races. It shows MOO1's
  pictures, music, sounds and fonts if you import the LBX files of your own copy of the game.
- **Age**: the same game as a medieval world map. Planets are cities, fleets are armies, and
  the ten races are real medieval empires (tables below).

[Español](MOO1.es.md)

## What is exactly as in Master of Orion

"Exactly" means the same formula and the same order of operations as 1oom, drawing the same
random numbers.

- **Random numbers**: 1oom's generator, so the same rules draw the same rolls.
- **Economy**: the five spending sliders, including how they push each other and how a locked
  slider stays put. Also production, factories and their cost, pollution and its cleanup,
  population growth, terraforming and soil enrichment, taxes and the reserve, and ship and
  base upkeep.
- **Research**: each field's tech tree and the random list of techs each empire can research.
  Also research costs, interest and breakthroughs, and field levels, which make ship parts
  cheaper and smaller. A field's project is chosen when it has none (at the start and after
  each discovery) and cannot be changed until it is discovered.
- **Ship design**: every hull and part, with MOO1's cost, space and combat values. The five
  starting designs cost 8, 14, 74, 65 and 570 BC, as in MOO1.
- **Movement**: map distances, speeds, fuel range and reserve tanks, nebulae, stargates and
  transports. A fleet in flight can only change destination with Hyperspace Communications.
- **Combat**:
  - the tactical grid, initiative, beams and missiles, every combat special and retreating
  - battles between computer empires
  - orbital bombing, transports shot down and ground combat
- **Computer players**: the original ("Classic") AI for production, research, ship design,
  fleets, transports, diplomacy, spying and voting. 1oom's "Classic+" variant, with fewer AI
  bugs, is also offered.
- **Diplomacy and events**:
  - contact, relations, treaties, trade, audiences, spies and sabotage
  - all 16 random events, the two space monsters and the Guardian of Orion
  - rebellions and coups
  - the Galactic Council and the final war against an empire that refuses its verdict
- **Bugs**: 1oom's rules are MOO1 v1.3 with the official v1.3a fixes. The MOO1 bugs that are
  part of how the game plays are kept; [MOO1_QUIRKS.md](MOO1_QUIRKS.md) lists them.

**How this is checked.** The comparison harness ([HARNESS.md](HARNESS.md)) takes the yearly
saves of computer-only 1oom games. It plays each year in Age of Orion and compares every
planet, empire, fleet and event with 1oom's save of the next year. In five games of 100 to 250
years, 613 of 710 turns (86%) are identical, including the first 39 turns of the small game.

## What differs

### Rules

- **Galaxy generation**: the same seed does not give 1oom's galaxy yet, because placing the
  home planets is simplified.
- **Turns that still differ**: 14% of the harness turns. The differences are in some relation
  changes, rare battles between computer empires, some bombing results and some events.
- **Missiles in tactical battles**: a missile reaches a ship when it gets to the centre of the
  ship's square. MOO1 uses a point on each ship picture, stored in FIRING.LBX.
- **Fleets in a stargate**: they cannot be redirected. In MOO1 such a fleet keeps the stargate
  speed and can reach any star in one turn.
- **Tech groups**: spying and tech trading use groups of techs derived from the tech names.
  MOO1 keeps these groups in RESEARCH.LBX.
- **Players**: one human player per game, as in MOO1. 1oom's hot-seat games for several
  players are not supported.
- **Saves**: games are saved as JSON files. MOO1 and 1oom saves cannot be loaded; the harness
  can convert 1oom saves.

### Screens and presentation

- **Interface**: new, not MOO1's 320x200 screens, but laid out like them:
  - the map with a side panel, and MOO1's buttons at the bottom (Game, Design, Fleet, Map,
    Races, Planets, Tech, Next turn)
  - fleets are given orders as in MOO1: click the ships next to a star, choose how many of
    each, click the destination and Accept
  - the Tech screen has the known techs of each field on the left and the research sliders on
    the right, and a new tech's project is chosen in its own dialog
  - MOO1's slider controls: − and + on each side, and clicking a slider's name locks it
  - as in MOO1, an unexplored star shows only its distance (no name or details), stars are
    small points that twinkle (MOO1's own star pictures when imported), the Races screen
    lists only the races met (those out of contact greyed out), and ship pictures are chosen
    with arrows in the design screen
  - unlike MOO1, the map zooms, there are keyboard shortcuts, and dialogs fit the window
- **Additions**: the Tech screen shows exactly what each tech gives, with its numbers, and a
  technology tree of what has been discovered. Clicking a planet's picture opens a window
  with its land, the ships in orbit, and its population, factories, waste and bases drawn as
  rows of icons. A planet's type, star, special, environment and owner can be clicked to see
  what they mean in the rules (population, production, growth, race abilities). You can also
  change the language and the theme, and load mods.
- **Texts**: written for this game, because MOO1's are copyrighted. In English, the Classic
  theme uses MOO1's own news and tech descriptions if you import EVENTMSG.LBX and
  RESEARCH.LBX.
- **Pictures**:
  - Classic shows your imported MOO1 art: planets, races, ships, battle backgrounds, and the
    landing and newscast screens. Without it, Classic uses the game's own art.
  - Age has its own art and painted portraits.
- **Music and sound**: MOO1's tunes and sound effects come from your MUSIC.LBX and
  SOUNDFX.LBX. A built-in synthesizer plays the tunes, so they sound different from an AdLib
  or Roland card. Without those files, the game generates its own sound effects.
- **Fonts**: the Classic theme uses MOO1's own fonts, from FONTS.LBX.
- **Platforms and languages**: it runs in the browser and on the desktop, in English and Spanish.

## The Age theme

The Age theme changes names, pictures and texts, never rules. Each MOO1 race becomes a
medieval empire whose history fits the race's abilities, and its painted ruler carries a
hint of the race.

### Empires

<!-- generated:races -->
| MOO1 race | Its ability | Age empire | Ruler | Why it fits | The race in the portrait |
|---|---|---|---|---|---|
| **Humans** | Cheap Force Fields, Planetology and Propulsion research. | **Holy Roman Empire** | Emperor Frederick Barbarossa | The best diplomats and traders: an empire of princes, electors and free trading cities. | A diplomat holding a sealed charter. |
| **Mrrshans** | +25% weapon damage, cheap Weapons research. | **Mongol Empire** | Genghis Khan | Feline warriors with +25% damage: the horse archers who conquered most of Eurasia. | Amber cat's eyes and snow-leopard fur. |
| **Silicoids** | Can colonize any planet and ignore pollution, but grow slowly. | **Ghurid Empire** | Sultan Muhammad of Ghor | Rock beings who settle anywhere, never pollute and grow slowly: an empire of the harsh Afghan mountains. | Stone-grey skin and a turban set with uncut stones. |
| **Sakkras** | Double population growth, cheap Planetology. | **Khmer Empire** | King Suryavarman II | Reptiles whose population grows twice as fast: the rice fields and reservoirs of Angkor fed a huge people. | A crown of naga serpents and scale-patterned gold. |
| **Psilons** | +50% research and every field is cheaper. | **Abbasid Caliphate** | Caliph al-Ma'mun | Scientists with +50% research: the caliphs of the House of Wisdom in Baghdad. | An astrolabe in the House of Wisdom. |
| **Alkaris** | Ships take 20% less damage, cheap Propulsion. | **North Sea Empire** | King Cnut the Great | Bird people whose ships are fast and hard to hit: Cnut's longships ruled the North Sea. | A raven on his shoulder and a cloak of raven feathers. |
| **Klackons** | Workers produce double, cheap Construction. | **Song Empire** | Emperor Shenzong | An insect hive whose workers produce double: the Song's mass workshops of iron and porcelain. | Lacquered armour like a beetle's shell, a hat with long antenna-like flaps. |
| **Bulrathis** | +25 in ground combat. | **First Bulgarian Empire** | Khan Krum | Bears, +25 in ground combat: the Bulgar warriors who beat Byzantium on land. | Bear fur and an iron helmet. |
| **Meklars** | Each colonist runs 2 extra factories, cheap Computers. | **Byzantine Empire** | Emperor Theophilos | Cyborgs whose colonists run extra factories: Byzantium, famous for its automata and Greek fire. | A golden mechanical lion. |
| **Darloks** | +2 scanner range. | **Republic of Venice** | Doge Enrico Dandolo | Shapeshifting spies: Venice, whose Council of Ten ran the best spy network of its age. | A carnival mask by candlelight. |
<!-- end generated -->

### Words

<!-- generated:terms -->
| Classic (MOO1) | Age |
|---|---|
| Planet | City |
| Population | People |
| Factories | Workshops |
| Missile bases | Towers |
| Planetary shield | Walls |
| BC | gold |
| RP | learning |
| Ecology | Land |
| Technology | Learning |
| Fleet | Army on the march |
| In orbit | Garrison |
| Transports | Migrants and levies |
| Standard Colony Base | Settler Camp |
| Small | Band |
| Medium | Company |
| Large | Battalion |
| Huge | Host |
| Nebula | Mists |
| Stargate | Canal port |
| Races | Empires |
| Galaxy size | World size |
| Galactic News Network | The Herald |
| The Galactic Council | The Great Council |
| Orion | Persepolis |
| Guardian | Immortals |
| Space Crystal | Catalan Company |
| Space Amoeba | Great Company |
<!-- end generated -->

### Technologies

The level is the tech's place in its field (1 to 50); research costs grow with it.

<!-- generated:techs -->
#### Computers / Scholarship

| Level | Classic (MOO1) | Age |
|---|---|---|
| 1 | Battle Scanner | Scouts' Reports |
| 2 | ECM Jammer Mark I | Camouflage I |
| 4 | Deep Space Scanner | Watchtowers |
| 5 | Battle Computer Mark II | Veterans II |
| 7 | ECM Jammer Mark II | Camouflage II |
| 8 | Improved Robotic Controls III | Guild Workshops III |
| 10 | Battle Computer Mark III | Veterans III |
| 12 | ECM Jammer Mark III | Camouflage III |
| 13 | Improved Space Scanner | Beacon Fires |
| 15 | Battle Computer Mark IV | Veterans IV |
| 17 | ECM Jammer Mark IV | Camouflage IV |
| 18 | Improved Robotic Controls IV | Guild Workshops IV |
| 20 | Battle Computer Mark V | Veterans V |
| 22 | ECM Jammer Mark V | Camouflage V |
| 23 | Advanced Space Scanner | Spy Network |
| 25 | Battle Computer Mark VI | Veterans VI |
| 27 | ECM Jammer Mark VI | Camouflage VI |
| 28 | Improved Robotic Controls V | Guild Workshops V |
| 30 | Battle Computer Mark VII | Veterans VII |
| 32 | ECM Jammer Mark VII | Camouflage VII |
| 34 | Hyperspace Communications | Carrier Pigeons |
| 35 | Battle Computer Mark VIII | Veterans VIII |
| 37 | ECM Jammer Mark VIII | Camouflage VIII |
| 38 | Improved Robotic Controls VI | Guild Workshops VI |
| 40 | Battle Computer Mark IX | Veterans IX |
| 42 | ECM Jammer Mark IX | Camouflage IX |
| 45 | Battle Computer Mark X | Veterans X |
| 46 | Oracle Interface | Signal Drums |
| 47 | ECM Jammer Mark X | Camouflage X |
| 48 | Improved Robotic Controls VII | Guild Workshops VII |
| 49 | Technology Nullifier | Bribed Captains |
| 50 | Battle Computer Mark XI | Veterans XI |

#### Construction / Masonry

| Level | Classic (MOO1) | Age |
|---|---|---|
| 1 | Reserve Fuel Tanks | Supply Wagons |
| 3 | Improved Industrial Tech 9 | Master Builders 9 |
| 5 | Reduced Industrial Waste 80% | Sanitation 80% |
| 8 | Improved Industrial Tech 8 | Master Builders 8 |
| 10 | Duralloy Armor | Chainmail Armor |
| 11 | Battle Suits | Gambesons |
| 13 | Improved Industrial Tech 7 | Master Builders 7 |
| 14 | Automated Repair System | Field Surgeons |
| 15 | Reduced Industrial Waste 60% | Sanitation 60% |
| 17 | Zortium Armor | Scale Armor |
| 18 | Improved Industrial Tech 6 | Master Builders 6 |
| 23 | Improved Industrial Tech 5 | Master Builders 5 |
| 24 | Armored Exoskeleton | Brigandine |
| 25 | Reduced Industrial Waste 40% | Sanitation 40% |
| 26 | Andrium Armor | Plate Armor |
| 28 | Improved Industrial Tech 4 | Master Builders 4 |
| 33 | Improved Industrial Tech 3 | Master Builders 3 |
| 34 | Tritanium Armor | Steel Plate Armor |
| 35 | Reduced Industrial Waste 20% | Sanitation 20% |
| 36 | Advanced Damage Control | Royal Physicians |
| 38 | Improved Industrial Tech 2 | Master Builders 2 |
| 40 | Powered Armor | Full Plate |
| 42 | Adamantium Armor | Milanese Armor |
| 45 | Industrial Waste Elimination | Clean Workshops |
| 50 | Neutronium Armor | Gothic Armor |

#### Force Fields / Fortification

| Level | Classic (MOO1) | Age |
|---|---|---|
| 1 | Class I Deflector Shields | Shield Wall I |
| 4 | Class II Deflector Shields | Shield Wall II |
| 8 | Personal Deflector Shield | Bucklers |
| 10 | Class III Deflector Shields | Shield Wall III |
| 12 | Class V Planetary Shield | City Walls V |
| 14 | Class IV Deflector Shields | Shield Wall IV |
| 16 | Repulsor Beam | Pike Walls |
| 20 | Class V Deflector Shields | Shield Wall V |
| 21 | Personal Absorption Shield | Tower Shields |
| 22 | Class X Planetary Shield | City Walls X |
| 24 | Class VI Deflector Shields | Shield Wall VI |
| 27 | Cloaking Device | Camouflage Cloaks |
| 30 | Class VII Deflector Shields | Shield Wall VII |
| 31 | Zyro Shield | Mantlets |
| 32 | Class XV Planetary Shield | City Walls XV |
| 34 | Class IX Deflector Shields | Shield Wall IX |
| 37 | Stasis Field | Smoke Pots |
| 38 | Personal Barrier Shield | Great Shields |
| 40 | Class XI Deflector Shields | Shield Wall XI |
| 42 | Class XX Planetary Shield | City Walls XX |
| 43 | Black Hole Generator | Sappers' Mines |
| 44 | Class XIII Deflector Shields | Shield Wall XIII |
| 46 | Lightning Shield | Testudo |
| 50 | Class XV Deflector Shields | Shield Wall XV |

#### Planetology / Agriculture

| Level | Classic (MOO1) | Age |
|---|---|---|
| 1 | Ecological Restoration | Forestry |
| 2 | Improved Terraforming +10 | Irrigation +10 |
| 3 | Controlled Barren Environment | Rocky Hills Settlement |
| 5 | Improved Eco Restoration | Improved Forestry |
| 6 | Controlled Tundra Environment | Tundra Settlement |
| 8 | Improved Terraforming +20 | Irrigation +20 |
| 9 | Controlled Dead Environment | Badlands Settlement |
| 10 | Death Spores | Plague Rats |
| 12 | Controlled Inferno Environment | Volcano Settlement |
| 13 | Enhanced Eco Restoration | Woodland Care |
| 14 | Improved Terraforming +30 | Irrigation +30 |
| 15 | Controlled Toxic Environment | Marsh Settlement |
| 16 | Soil Enrichment | Crop Rotation |
| 17 | Bio Toxin Antidote | Herbal Remedies |
| 18 | Controlled Radiated Environment | Wasteland Settlement |
| 20 | Improved Terraforming +40 | Irrigation +40 |
| 21 | Cloning | Midwifery |
| 22 | Atmospheric Terraforming | Land Reclamation |
| 24 | Advanced Eco Restoration | Royal Forests |
| 26 | Improved Terraforming +50 | Irrigation +50 |
| 27 | Doom Virus | Black Death |
| 30 | Advanced Soil Enrichment | Three-Field System |
| 32 | Improved Terraforming +60 | Irrigation +60 |
| 34 | Complete Eco Restoration | Sacred Groves |
| 36 | Universal Antidote | Medicine |
| 38 | Improved Terraforming +80 | Irrigation +80 |
| 40 | Bio Terminator | Pestilence |
| 42 | Advanced Cloning | Hospitals |
| 44 | Improved Terraforming +100 | Irrigation +100 |
| 50 | Complete Terraforming | Great Aqueducts |

#### Propulsion / Roads

| Level | Classic (MOO1) | Age |
|---|---|---|
| 1 | Retro Engines | Marching |
| 3 | Hydrogen Fuel Cells | Supply Carts |
| 5 | Deuterium Fuel Cells | Supply Trains |
| 6 | Nuclear Engines | Pony Breeding |
| 9 | Irridium Fuel Cells | Granaries |
| 10 | Inertial Stabilizer | Light Cavalry Drill |
| 12 | Sub Light Drives | Horse Breeding |
| 14 | Dotomite Crystals | Road Network |
| 16 | Energy Pulsar | Fire Ring |
| 18 | Fusion Drives | Destrier Breeding |
| 19 | Uridium Fuel Cells | Paved Roads |
| 20 | Warp Dissipator | Caltrops |
| 23 | Reajax II Fuel Cells | Royal Roads |
| 24 | Impulse Drives | Chariots |
| 27 | Intergalactic Star Gates | Grand Canals |
| 29 | Trilithium Crystals | Waystations |
| 30 | Ion Drives | Arabian Horse Breeding |
| 34 | High Energy Focus | Master Archers |
| 36 | Anti Matter Drives | Turkmen Horse Breeding |
| 38 | Sub Space Teleporter | Secret Paths |
| 40 | Ionic Pulsar | Firestorm |
| 41 | Thorium Cells | Imperial Post |
| 42 | Inter Phased Drives | River Galleys |
| 43 | Sub Space Interdictor | Sentry Posts |
| 45 | Combat Transporters | Siege Ladders |
| 46 | Inertial Nullifier | Cavalry Drill |
| 48 | Hyper Drives | Relay Stations |
| 50 | Displacement Device | Decoy Banners |

#### Weapons / Smithing

| Level | Classic (MOO1) | Age |
|---|---|---|
| 1 | Lasers | Archery |
| 2 | Hand Lasers | Hand Crossbows |
| 4 | Hyper V Rockets | Throwing Axes |
| 5 | Gatling Laser | Repeating Crossbows |
| 6 | Anti Missile Rockets | Pavises |
| 7 | Neutron Pellet Gun | Slings |
| 8 | Hyper X Rockets | Darts |
| 9 | Fusion Bomb | Greek Fire |
| 10 | Ion Cannon | Crossbows |
| 11 | Scatter Pack V Rockets | Hail of Arrows |
| 12 | Ion Rifle | Pikes |
| 13 | Mass Driver | Ballistae |
| 14 | Merculite Missiles | Fire Arrows |
| 15 | Neutron Blaster | Hand Cannons |
| 16 | Anti Matter Bomb | Siege Mortars |
| 17 | Graviton Beam | Chain Shot |
| 18 | Stinger Missiles | Barbed Javelins |
| 19 | Hard Beam | Arquebuses |
| 20 | Fusion Beam | Muskets |
| 21 | Ion Stream Projector | Quicklime |
| 22 | Omega V Bomb | Bombards |
| 23 | Anti Matter Torpedoes | Onagers |
| 24 | Fusion Rifle | Halberds |
| 25 | Megabolt Cannon | Scorpions |
| 26 | Phasor | Rifles |
| 27 | Scatter Pack VII Missiles | Arrow Storm |
| 28 | Auto Blaster | Volley Guns |
| 29 | Pulson Missiles | Mangonels |
| 30 | Tachyon Beam | Grapeshot |
| 31 | Hand Phasor | Pistols |
| 32 | Gauss Autocannon | Organ Guns |
| 33 | Particle Beam | Field Guns |
| 34 | Hercular Missiles | Trebuchets |
| 35 | Plasma Cannon | Cannons |
| 36 | Death Ray | Burning Mirrors |
| 37 | Disruptor | Great Cannons |
| 38 | Pulse Phasor | Rapid Rifles |
| 39 | Neutronium Bomb | Great Bombards |
| 40 | Hellfire Torpedoes | Fire Trebuchets |
| 41 | Zeon Missiles | Rockets |
| 42 | Plasma Rifle | Carbines |
| 43 | Proton Torpedoes | Siege Rockets |
| 44 | Scatter Pack X Missiles | Rocket Volleys |
| 45 | Tri Focus Plasma Cannon | Triple Cannons |
| 46 | Stellar Converter | Siege Train |
| 47 | Neutron Stream Projector | Fire Siphons |
| 48 | Mauler Device | Mauler Catapults |
| 50 | Plasma Torpedoes | Fire Ships |
<!-- end generated -->

The tables are generated from the game's text files by `tools/gen_moo1_doc.py`.
