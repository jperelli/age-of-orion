#!/usr/bin/env python3
"""Fills the generated tables of docs/MOO1.md and docs/MOO1.es.md from the game's texts.

    python3 tools/gen_moo1_doc.py

Each table sits between `<!-- generated:NAME -->` and `<!-- end generated -->`. The names
come from web/public/text (MOO1, the Classic theme) and web/public/themes/age/text (the Age
theme), the tech levels from crates/core/src/data.rs (1oom's tech tables). Run it after
changing a name of either theme.
"""

import json
import os
import re

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
RACES = ["human", "mrrshan", "silicoid", "sakkra", "psilon", "alkari", "klackon", "bulrathi", "meklar", "darlok"]
FIELDS = ["computer", "construction", "force_field", "planetology", "propulsion", "weapon"]

# Why each empire stands for its race, and what of the race its portrait shows.
WHY = {
    "en": {
        "human": ("The best diplomats and traders: an empire of princes, electors and free trading cities.", "A diplomat holding a sealed charter."),
        "mrrshan": ("Feline warriors with +25% damage: the horse archers who conquered most of Eurasia.", "Amber cat's eyes and snow-leopard fur."),
        "silicoid": ("Rock beings who settle anywhere, never pollute and grow slowly: an empire of the harsh Afghan mountains.", "Stone-grey skin and a turban set with uncut stones."),
        "sakkra": ("Reptiles whose population grows twice as fast: the rice fields and reservoirs of Angkor fed a huge people.", "A crown of naga serpents and scale-patterned gold."),
        "psilon": ("Scientists with +50% research: the caliphs of the House of Wisdom in Baghdad.", "An astrolabe in the House of Wisdom."),
        "alkari": ("Bird people whose ships are fast and hard to hit: Cnut's longships ruled the North Sea.", "A raven on his shoulder and a cloak of raven feathers."),
        "klackon": ("An insect hive whose workers produce double: the Song's mass workshops of iron and porcelain.", "Lacquered armour like a beetle's shell, a hat with long antenna-like flaps."),
        "bulrathi": ("Bears, +25 in ground combat: the Bulgar warriors who beat Byzantium on land.", "Bear fur and an iron helmet."),
        "meklar": ("Cyborgs whose colonists run extra factories: Byzantium, famous for its automata and Greek fire.", "A golden mechanical lion."),
        "darlok": ("Shapeshifting spies: Venice, whose Council of Ten ran the best spy network of its age.", "A carnival mask by candlelight."),
    },
    "es": {
        "human": ("Los mejores diplomáticos y comerciantes: un imperio de príncipes, electores y ciudades libres de comerciantes.", "Un diplomático con una carta sellada."),
        "mrrshan": ("Guerreros felinos con +25% de daño: los arqueros a caballo que conquistaron casi toda Eurasia.", "Ojos ambarinos de gato y piel de leopardo de las nieves."),
        "silicoid": ("Seres de roca que se asientan en cualquier lugar, no contaminan y crecen despacio: un imperio de las duras montañas afganas.", "Piel gris como la piedra y un turbante con piedras sin tallar."),
        "sakkra": ("Reptiles cuya población crece el doble de rápido: los arrozales y embalses de Angkor alimentaron a un pueblo enorme.", "Una corona de serpientes naga y oro con dibujo de escamas."),
        "psilon": ("Científicos con +50% de investigación: los califas de la Casa de la Sabiduría de Bagdad.", "Un astrolabio en la Casa de la Sabiduría."),
        "alkari": ("Pájaros cuyas naves son rápidas y difíciles de alcanzar: los drakares de Canuto dominaron el mar del Norte.", "Un cuervo en el hombro y una capa de plumas de cuervo."),
        "klackon": ("Una colmena de insectos cuyos obreros producen el doble: los grandes talleres de hierro y porcelana de los Song.", "Una armadura laqueada como el caparazón de un escarabajo y un sombrero con largas alas como antenas."),
        "bulrathi": ("Osos, +25 en el combate terrestre: los guerreros búlgaros que vencieron a Bizancio en tierra.", "Piel de oso y un casco de hierro."),
        "meklar": ("Cíborgs cuyos colonos atienden más fábricas: Bizancio, famoso por sus autómatas y el fuego griego.", "Un león mecánico de oro."),
        "darlok": ("Espías que cambian de forma: Venecia, cuyo Consejo de los Diez tuvo la mejor red de espías de su época.", "Una máscara de carnaval a la luz de las velas."),
    },
}
HEAD = {
    "en": {
        "races": ["MOO1 race", "Its ability", "Age empire", "Ruler", "Why it fits", "The race in the portrait"],
        "terms": ["Classic (MOO1)", "Age"],
        "techs": ["Level", "Classic (MOO1)", "Age"],
    },
    "es": {
        "races": ["Raza de MOO1", "Su habilidad", "Imperio de Era", "Gobernante", "Por qué encaja", "La raza en el retrato"],
        "terms": ["Clásico (MOO1)", "Era"],
        "techs": ["Nivel", "Clásico (MOO1)", "Era"],
    },
}
# Words shown in both themes, by text key.
TERMS = [
    "planet.name", "planet.pop", "planet.factories", "planet.bases", "planet.shield", "unit.bc", "unit.rp",
    "slider.eco", "slider.tech", "fleet.title", "orbit.title",
    "transport.panel_title", "part.special.standard_colony_base", "part.hull.small", "part.hull.medium",
    "part.hull.large", "part.hull.huge", "planet.nebula", "planet.stargate", "game.btn.rivals",
    "newgame.size", "news.title", "council.title", "names.orion", "guardian.name", "monster.crystal",
    "monster.amoeba",
]


def load(path):
    with open(os.path.join(ROOT, path), encoding="utf-8") as f:
        return json.load(f)


def texts(lang):
    base = load("web/public/text/en.json")
    if lang != "en":
        base.update(load(f"web/public/text/{lang}.json"))
    classic = dict(base)
    classic.update(load("web/public/themes/classic/text/en.json"))
    age = dict(base)
    age.update(load("web/public/themes/age/text/en.json"))
    if lang != "en":
        for theme, d in (("classic", classic), ("age", age)):
            p = f"web/public/themes/{theme}/text/{lang}.json"
            if os.path.exists(os.path.join(ROOT, p)):
                d.update(load(p))
    classic.setdefault("names.orion", "Orion")
    return classic, age


def tech_levels():
    src = open(os.path.join(ROOT, "crates/core/src/data.rs"), encoding="utf-8").read()
    block = src[src.index("pub const TECHS"):]
    block = block[block.index("= [") + 3 : block.index("\n];")]
    fields = re.findall(r"\[\s*((?:\"[^\"]*\",\s*)+)\]", block)
    assert len(fields) == 6, len(fields)
    return [re.findall(r"\"([^\"]*)\"", f) for f in fields]


def table(head, rows):
    esc = lambda s: str(s).replace("|", "\\|").replace("\n", " ")
    out = ["| " + " | ".join(head) + " |", "|" + "|".join(["---"] * len(head)) + "|"]
    out += ["| " + " | ".join(esc(c) for c in r) + " |" for r in rows]
    return "\n".join(out)


def generate(lang):
    classic, age = texts(lang)
    h = HEAD[lang]
    races = []
    for i, r in enumerate(RACES):
        empire = age[f"race.{r}.trait"].split(". ")[0]
        why, hint = WHY[lang][r]
        races.append([f"**{classic[f'race.{r}.plural']}**", classic[f"race.{r}.trait"], f"**{empire}**", age["names.leaders"][i], why, hint])
    terms = [[classic[k].replace("{n} ", ""), age[k].replace("{n} ", "")] for k in TERMS]
    techs = []
    for f, names in zip(FIELDS, tech_levels()):
        techs.append(f"#### {classic[f'field.{f}']} / {age[f'field.{f}']}\n")
        rows = [[n, classic[f"tech.{f}.{k}"], age[f"tech.{f}.{k}"]] for n, k in enumerate(names) if k]
        techs.append(table(h["techs"], rows) + "\n")
    return {"races": table(h["races"], races), "terms": table(h["terms"], terms), "techs": "\n".join(techs).rstrip()}


def main():
    for lang, doc in (("en", "docs/MOO1.md"), ("es", "docs/MOO1.es.md")):
        path = os.path.join(ROOT, doc)
        s = open(path, encoding="utf-8").read()
        for name, body in generate(lang).items():
            s, n = re.subn(
                rf"(<!-- generated:{name} -->\n).*?(<!-- end generated -->)",
                lambda m: m[1] + body + "\n" + m[2],
                s,
                flags=re.S,
            )
            assert n == 1, (doc, name)
        open(path, "w", encoding="utf-8").write(s)
        print("wrote", doc)


if __name__ == "__main__":
    main()
