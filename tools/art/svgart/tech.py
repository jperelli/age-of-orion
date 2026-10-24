"""Classic's tech pictures (`tech/<field>/<n>`, 119x78): a blueprint per tech, as MOO1's
TECHNO.LBX, its motif chosen from the tech's name and a mark for its level."""
import json
import math
import os
import re

from .kit import circle, ellipse, g, line, path, poly, put, rect, rng, text

FIELDS = ["computer", "construction", "force_field", "planetology", "propulsion", "weapon"]
INK = "#d8ecff"
L = dict(stroke=INK, stroke_width=1.4, fill="none", stroke_linejoin="round", stroke_linecap="round")
F = dict(stroke=INK, stroke_width=1.4, fill="#d8ecff", fill_opacity=0.18, stroke_linejoin="round")


def techs():
    """The tech names by field and number, from the game's tables (crates/core/src/data.rs)."""
    root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
    s = open(os.path.join(root, "crates/core/src/data.rs")).read()
    block = s[s.index("pub const TECHS"):]
    block = block[: block.index("\n];")]
    fields = re.split(r"\n    \[\n", block)[1:]
    return [re.findall(r'"([^"]*)"', f) for f in fields]


# ------------------------------------------------------------------ motifs (centred on 60, 40)

def ship_outline(x=60, y=40, k=1.0, **kw):
    p = [(30, 0), (8, -12), (-22, -12), (-30, -4), (-30, 4), (-22, 12), (8, 12)]
    return poly([(x + a * k, y + b * k) for a, b in p], **(kw or F))


def scanner(n):
    out = path("M40,58 L60,40 L80,58Z", **F) + path("M44,30 Q60,10 82,26 Q66,44 44,30Z", **F) + line(62, 28, 74, 18, **L)
    return out + "".join(path(f"M{74 + 6 * i},{14 - 2 * i} q6,6 0,{12 + 4 * i}", **L) for i in range(1 + n % 3))


def jammer(n):
    return line(40, 64, 40, 18, **L) + poly([(32, 64), (48, 64), (40, 50)], **F) + "".join(path(f"M46,{24 + 8 * i} l8,-3 l6,6 l8,-3 l6,6 l8,-3", **L) for i in range(3))


def computer(n):
    return rect(36, 14, 48, 34, rx=3, **F) + circle(60, 31, 9, **L) + line(60, 18, 60, 44, **L) + line(47, 31, 73, 31, **L) + rect(32, 52, 56, 10, rx=2, **F) + \
        "".join(line(36 + i * 6, 57, 39 + i * 6, 57, **L) for i in range(9))


def robot_arm(n):
    return rect(36, 56, 30, 8, **F) + line(50, 56, 50, 40, stroke_width=4, stroke=INK) + line(50, 40, 72, 26, stroke_width=4, stroke=INK) + \
        circle(50, 40, 4, **F) + circle(72, 26, 3, **F) + path("M72,26 l8,-4 M72,26 l8,4", **L)


def tower(n):
    return poly([(52, 66), (60, 16), (68, 66)], **F) + line(54, 50, 66, 50, **L) + line(56, 34, 64, 34, **L) + \
        "".join(circle(60, 16, 6 + 6 * i, **L, opacity=1 - i * 0.25) for i in range(3))


def eye(n):
    return poly([(30, 62), (60, 12), (90, 62)], **F) + path("M42,46 Q60,30 78,46 Q60,62 42,46Z", **L) + circle(60, 46, 5, fill=INK)


def chip_x(n):
    return rect(42, 22, 36, 36, **F) + "".join(line(42 - 6, 26 + i * 7, 42, 26 + i * 7, **L) + line(78, 26 + i * 7, 84, 26 + i * 7, **L) for i in range(5)) + \
        path("M48,28 L72,52 M72,28 L48,52", stroke="#ff8a7a", stroke_width=2.5)


def tanks(n):
    return "".join(rect(34 + i * 18, 18, 14, 44, rx=7, **F) + line(41 + i * 18, 24, 41 + i * 18, 56, **L) for i in range(3)) + line(30, 64, 90, 64, **L)


def factory(n):
    return path("M24,64 V40 L38,48 V40 L52,48 V40 L66,48 V30 H74 V64Z", **F) + rect(78, 20, 8, 44, **F) + circle(42, 26, 9, **L) + \
        "".join(line(42 + 9 * math.cos(a), 26 + 9 * math.sin(a), 42 + 12 * math.cos(a), 26 + 12 * math.sin(a), **L) for a in [i * math.pi / 4 for i in range(8)])


def recycle(n):
    return path("M48,60 h24 l6,-30 h-36Z", **F) + path("M52,44 a8,8 0 1 1 16,0", **L) + poly([(66, 40), (70, 44), (72, 38)], fill=INK)


def armor(n):
    layers = 2 + n // 10
    return "".join(path(f"M{30 + i * 6},{64 - i * 2} V{20 + i * 4} Q{60},{10 + i * 4} {90 - i * 6},{20 + i * 4} V{64 - i * 2}", **(F if i == layers - 1 else L)) for i in range(min(layers, 6)))


def suit(n):
    return circle(60, 18, 7, **F) + path("M48,28 H72 L76,48 H68 L66,66 H54 L52,48 H44Z", **F) + line(48, 28, 40, 46, **L) + line(72, 28, 80, 46, **L) + rect(55, 14, 10, 5, **L)


def wrench(n):
    return path("M34,58 L64,28", stroke=INK, stroke_width=5) + path("M60,22 a10,10 0 1 0 12,14 l-6,-2 l-2,-6Z", **F) + circle(78, 58, 8, **L) + \
        "".join(line(78 + 8 * math.cos(a), 58 + 8 * math.sin(a), 78 + 11 * math.cos(a), 58 + 11 * math.sin(a), **L) for a in [i * math.pi / 3 for i in range(6)])


def deflector(n):
    return ship_outline(56, 40, 0.8) + "".join(ellipse(56, 40, 32 + 6 * i, 20 + 5 * i, **L, opacity=1 - i * 0.2) for i in range(1 + min(3, n // 12)))


def planet_shield(n):
    return path("M24,64 Q60,54 96,64", **L) + path("M30,62 a30,30 0 0 1 60,0", **F) + circle(60, 64, 22, fill="#d8ecff", fill_opacity=0.25, stroke=INK, stroke_width=1) + \
        "".join(path(f"M{30 - i * 4},62 a{30 + i * 4},{30 + i * 4} 0 0 1 {60 + i * 8},0", **L) for i in range(1, 1 + n // 15))


def person_shield(n):
    return circle(60, 24, 6, **F) + path("M52,32 H68 L70,52 H50Z", **F) + line(54, 52, 52, 66, **L) + line(66, 52, 68, 66, **L) + ellipse(60, 42, 22, 28, **L, stroke_dasharray="4 3")


def repulsor(n):
    return rect(30, 30, 20, 20, **F) + "".join(path(f"M54,{40 + d} L84,{40 + d * 2.2}", **L) + poly([(84, 40 + d * 2.2 - 3), (90, 40 + d * 2.2), (84, 40 + d * 2.2 + 3)], fill=INK) for d in (-8, 0, 8))


def cloak(n):
    return ship_outline(60, 40, 1, stroke=INK, stroke_width=1.4, fill="none", stroke_dasharray="4 4")


def lightning(n):
    return ship_outline(60, 40, 0.8) + "".join(path(f"M{30 + i * 20},{12} l-5,10 l6,0 l-6,12", **L) for i in range(4))


def stasis(n):
    return rect(32, 16, 56, 48, **L, stroke_dasharray="2 2") + ship_outline(60, 40, 0.6) + path("M32,16 L88,64 M88,16 L32,64", **L, opacity=0.4)


def black_hole(n):
    return "".join(path(f"M60,40 m{-r},0 a{r},{r * 0.7} 0 1 1 {r * 2},0", **L, transform=f"rotate({i * 40} 60 40)") for i, r in enumerate((8, 14, 20, 26))) + circle(60, 40, 5, fill=INK)


def leaf(n):
    return path("M60,66 V36", **L) + path("M60,40 Q36,36 38,14 Q60,18 60,40Z", **F) + path("M60,50 Q84,46 82,24 Q60,28 60,50Z", **F) + line(30, 66, 90, 66, **L)


def terraform(n):
    return circle(60, 42, 22, **F) + path("M40,46 q10,-8 20,0 t20,0", **L) + "".join(poly([(x, 16), (x - 4, 22), (x + 4, 22)], fill=INK) + line(x, 22, x, 30, **L) for x in (44, 60, 76)) + \
        text(98, 66, f"+{10 * (1 + n // 7)}", font_size=8, fill=INK, text_anchor="end", font_family="monospace")


def dome(n):
    return line(24, 62, 96, 62, **L) + path("M34,62 a26,26 0 0 1 52,0", **F) + path("M40,62 L48,52 L54,58 L62,46 L70,56 L80,62", **L) + line(60, 36, 60, 30, **L)


def biohazard(n):
    return circle(60, 40, 6, **L) + "".join(circle(60 + 10 * math.cos(a), 40 + 10 * math.sin(a), 9, **L) for a in (-math.pi / 2, math.pi / 6, 5 * math.pi / 6)) + circle(60, 40, 22, **L, stroke_dasharray="3 3")


def sprout(n):
    return rect(26, 50, 68, 16, **F) + line(26, 56, 94, 56, **L) + path("M60,50 V30 M60,38 q-12,-4 -14,-14 q12,2 14,14 M60,34 q12,-4 14,-14 q-12,2 -14,14", **L)


def vial(n):
    return path("M54,14 h12 v8 l10,34 a6,6 0 0 1 -6,8 h-20 a6,6 0 0 1 -6,-8 l10,-34Z", **F) + path("M48,50 h24", **L) + line(52, 14, 68, 14, **L)


def dna(n):
    out = ""
    for i in range(9):
        y = 12 + i * 7
        dx = 14 * math.sin(i * 0.8)
        out += line(60 - dx, y, 60 + dx, y, **L) + circle(60 - dx, y, 2, fill=INK) + circle(60 + dx, y, 2, fill=INK)
    return out


def clouds(n):
    return circle(60, 42, 22, **F) + "".join(path(f"M{40 + i * 6},{30 + i * 8} q10,-6 20,0 q10,-6 20,0", **L) for i in range(3))


def engine(n):
    flames = 1 + n // 10
    out = path("M28,30 H66 L80,24 V56 L66,50 H28Z", **F) + line(40, 30, 40, 50, **L) + line(52, 30, 52, 50, **L)
    return out + "".join(path(f"M80,{32 + i * 16 / max(1, flames - 1) if flames > 1 else 40} l{10 + 3 * flames},0", stroke=INK, stroke_width=2, stroke_dasharray="3 2") for i in range(min(flames, 5)))


def cells(n):
    return "".join(rect(28 + i * 16, 26, 12, 30, rx=2, **F) + rect(31 + i * 16, 22, 6, 4, **F) + text(34 + i * 16, 46, "+", font_size=10, fill=INK, text_anchor="middle") for i in range(4))


def crystals(n):
    return "".join(poly([(x, 64), (x - w, 40), (x, 14 + d), (x + w, 40)], **F) for x, w, d in ((48, 8, 10), (62, 10, 0), (76, 7, 14)))


def gyro(n):
    return circle(60, 40, 22, **L) + ellipse(60, 40, 22, 8, **L) + ellipse(60, 40, 8, 22, **L) + circle(60, 40, 4, fill=INK)


def pulsar(n):
    return circle(60, 40, 8, **F) + "".join(line(60 + 12 * math.cos(a), 40 + 12 * math.sin(a), 60 + 28 * math.cos(a), 40 + 28 * math.sin(a), **L) for a in [i * math.tau / 10 for i in range(10)])


def warp(n):
    return "".join(path(f"M24,{24 + i * 10} q9,-8 18,0 t18,0 t18,0 t18,0", **L) for i in range(4))


def gate(n):
    return ellipse(60, 40, 26, 30, **F) + ellipse(60, 40, 18, 22, **L) + "".join(circle(60 + 26 * math.cos(a), 40 + 30 * math.sin(a), 2, fill=INK) for a in [i * math.tau / 8 for i in range(8)])


def lens(n):
    return path("M56,12 Q66,40 56,68 Q46,40 56,12Z", **F) + "".join(line(20, 40 + d, 54, 40 + d * 0.4, **L) for d in (-16, -8, 0, 8, 16)) + line(60, 40, 98, 40, stroke=INK, stroke_width=3)


def teleport(n):
    return circle(60, 22, 6, **F) + path("M52,30 H68 L70,50 H50Z", **F) + ellipse(60, 64, 20, 5, **L) + ellipse(60, 12, 20, 5, **L) + \
        "".join(line(44 + i * 8, 14, 44 + i * 8, 62, **L, stroke_dasharray="2 4", opacity=0.6) for i in range(5))


def interdictor(n):
    return ship_outline(40, 40, 0.5) + line(70, 14, 70, 66, stroke=INK, stroke_width=3) + "".join(path(f"M74,{20 + i * 10} l12,0", **L) for i in range(5))


def displacement(n):
    return ship_outline(48, 44, 0.6) + ship_outline(74, 34, 0.6, stroke=INK, stroke_width=1.2, fill="none", stroke_dasharray="3 3") + path("M58,40 Q64,30 70,34", **L)


def beam(n):
    return rect(22, 32, 36, 16, rx=3, **F) + rect(58, 36, 12, 8, **F) + line(70, 40, 100, 40, stroke=INK, stroke_width=1 + n // 12) + circle(100, 40, 3, fill=INK) + \
        "".join(line(30 + i * 7, 32, 30 + i * 7, 48, **L) for i in range(4))


def missile(n):
    return path("M24,40 L34,32 H80 L94,40 L80,48 H34Z", **F) + poly([(34, 32), (26, 22), (40, 32)], **F) + poly([(34, 48), (26, 58), (40, 48)], **F) + line(80, 32, 80, 48, **L) + \
        path("M24,40 l-8,-3 v6Z", fill=INK)


def bomb(n):
    return circle(56, 44, 18, **F) + rect(66, 22, 8, 10, **F, transform="rotate(35 70 27)") + path("M74,20 q6,-6 10,-2", **L) + text(56, 48, "!" * max(1, min(3, n // 12)), font_size=11, fill=INK, text_anchor="middle", font_family="monospace")


def rifle(n):
    return path("M20,44 H70 V38 H96 V44 H86 V48 H54 L48,58 H40 L44,48 H20Z", **F) + line(70, 41, 96, 41, **L) + rect(56, 32, 10, 6, **L)


def gatling(n):
    return "".join(rect(46, 30 + i * 6, 46, 4, **F) for i in range(4)) + rect(26, 28, 22, 26, rx=3, **F) + circle(37, 41, 5, **L)


def converter(n):
    return circle(70, 40, 20, **F) + path("M58,26 L66,38 L58,48 L70,56", **L) + rect(14, 34, 22, 12, **F) + line(36, 40, 50, 40, stroke=INK, stroke_width=3, stroke_dasharray="4 2")


def mauler(n):
    return path("M24,26 H64 L86,40 L64,54 H24Z", **F) + "".join(circle(40 + i * 10, 40, 4, **L) for i in range(3)) + line(86, 40, 100, 40, stroke=INK, stroke_width=4)


def question(n):
    return circle(60, 40, 24, **L, stroke_dasharray="4 3") + text(60, 50, "?", font_size=30, fill=INK, text_anchor="middle", font_family="serif", font_weight="bold")


RULES = [
    ("scanner", scanner), ("ecm_jammer", jammer), ("battle_computer", computer), ("robotic", robot_arm), ("hyperspace_comm", tower), ("oracle", eye),
    ("technology_nullifier", chip_x), ("fuel_tanks", tanks), ("industrial_tech", factory), ("waste", recycle), ("exoskeleton", suit), ("powered_armor", suit), ("armor", armor), ("suits", suit),
    ("repair", wrench), ("damage_control", wrench), ("planetary_shield", planet_shield), ("personal", person_shield), ("deflector", deflector), ("repulsor", repulsor),
    ("cloaking", cloak), ("zyro", lightning), ("lightning", lightning), ("stasis", stasis), ("black_hole", black_hole), ("eco", leaf), ("restoration", leaf),
    ("atmospheric", clouds), ("terraforming", terraform), ("environment", dome), ("spores", biohazard), ("virus", biohazard), ("terminator", biohazard),
    ("soil", sprout), ("antidote", vial), ("cloning", dna), ("fuel_cells", cells), ("thorium", cells), ("crystals", crystals), ("inertial", gyro), ("pulsar", pulsar),
    ("warp_dissipator", warp), ("star_gates", gate), ("high_energy", lens), ("teleporter", teleport), ("transporters", teleport), ("interdictor", interdictor),
    ("displacement", displacement), ("engines", engine), ("drives", engine), ("bomb", bomb), ("rockets", missile), ("missiles", missile), ("torpedoes", missile),
    ("hand_", rifle), ("rifle", rifle), ("gatling", gatling), ("autocannon", gatling), ("auto_blaster", gatling), ("mass_driver", gatling), ("pellet", gatling),
    ("stellar_converter", converter), ("mauler", mauler), ("", beam),
]


def blueprint(motif, n, field, mark):
    body = rect(0, 0, 119, 78, fill="#0c2a5a") + "".join(line(x, 0, x, 78, stroke="#1d4a8a", stroke_width=0.5) for x in range(0, 120, 8)) + \
        "".join(line(0, y, 119, y, stroke="#1d4a8a", stroke_width=0.5) for y in range(0, 79, 8))
    body += rect(2, 2, 115, 74, fill="none", stroke="#6a9ad8", stroke_width=1)
    body += motif(n)
    body += text(114, 10, mark, font_size=7, fill="#8ab8f0", text_anchor="end", font_family="monospace")
    return body


def build():
    names = techs()
    for f, field in enumerate(names):
        for n, key in enumerate(field):
            if not key:
                continue
            motif = next(m for k, m in RULES if k in key)
            put("classic", f"tech/{f}/{n}", 119, 78, blueprint(motif, n, f, f"{FIELDS[f][:4].upper()}-{n:02}"))
        put("classic", f"tech/{f}/advanced", 119, 78, blueprint(question, 0, f, f"{FIELDS[f][:4].upper()}-ADV"))
