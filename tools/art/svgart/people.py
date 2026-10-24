"""Classic's people and rooms: the ten races' envoys, scientists, spies, explorers and soldiers,
the embassy (MOO1's EMBASSY.LBX), the throne rooms and the council hall of a world map, the
GNN newscast (NEWSCAST.LBX) and its 22 news pictures, and the laboratories of a discovery
(TECHNO.LBX)."""
import math

from .kit import blob, blur, circle, dark, ellipse, g, light, lin, line, mix, num, path, poly, put, rad, rect, rng, starfield, text
from .space import BANNERS, globe

RACES = ["human", "mrrshan", "silicoid", "sakkra", "psilon", "alkari", "klackon", "bulrathi", "meklar", "darlok"]
SKIN = {
    "human": "#e3b48c", "mrrshan": "#d99a3a", "silicoid": "#8a80a8", "sakkra": "#5f9a44", "psilon": "#9fb8e0",
    "alkari": "#b8864a", "klackon": "#a8452a", "bulrathi": "#6b4a32", "meklar": "#8f9aa8", "darlok": "#4a4060",
}
# Each race's colour (robes, rooms).
ACCENT = {
    "human": "#2f5fd0", "mrrshan": "#c8502a", "silicoid": "#6a5a9a", "sakkra": "#2f7a3a", "psilon": "#3a6ab0",
    "alkari": "#b07a2a", "klackon": "#8a2a1a", "bulrathi": "#5a3a24", "meklar": "#5a6470", "darlok": "#2a2440",
}


def eye(x, y, s, iris="#2a2a2a", white="#ffffff", slit=False):
    out = ellipse(x, y, 3.4 * s, 2.2 * s, fill=white, stroke="#1a1a1a", stroke_width=0.5 * s)
    out += ellipse(x, y, 0.8 * s, 2 * s, fill=iris) if slit else circle(x, y, 1.5 * s, fill=iris)
    return out


def head(race, cx, cy, s):
    """A race's head facing us, centred on (cx, cy), about 20 * s across."""
    c = SKIN[race]
    o = dict(stroke="#1a1a1a", stroke_width=0.8 * s)
    if race == "human":
        return (ellipse(cx, cy, 10 * s, 12.5 * s, fill=c, **o) + path(f"M{num(cx - 10.5 * s)},{num(cy - 2 * s)} Q{num(cx - 11 * s)},{num(cy - 15 * s)} {num(cx)},{num(cy - 14 * s)} Q{num(cx + 11 * s)},{num(cy - 15 * s)} {num(cx + 10.5 * s)},{num(cy - 2 * s)} Q{num(cx + 6 * s)},{num(cy - 9 * s)} {num(cx)},{num(cy - 9 * s)} Q{num(cx - 6 * s)},{num(cy - 9 * s)} {num(cx - 10.5 * s)},{num(cy - 2 * s)}Z", fill="#5a3a20", **o)
                + eye(cx - 4 * s, cy - 1 * s, s * 0.8, "#3a5a8a") + eye(cx + 4 * s, cy - 1 * s, s * 0.8, "#3a5a8a")
                + path(f"M{num(cx)},{num(cy)} l{num(-1.2 * s)},{num(4 * s)} h{num(2 * s)}", fill="none", **o) + path(f"M{num(cx - 3 * s)},{num(cy + 7 * s)} q{num(3 * s)},{num(1.5 * s)} {num(6 * s)},0", fill="none", **o))
    if race == "mrrshan":
        return (poly([(cx - 10 * s, cy - 4 * s), (cx - 9 * s, cy - 17 * s), (cx - 3 * s, cy - 10 * s)], fill=c, **o) + poly([(cx + 10 * s, cy - 4 * s), (cx + 9 * s, cy - 17 * s), (cx + 3 * s, cy - 10 * s)], fill=c, **o)
                + ellipse(cx, cy, 11 * s, 11.5 * s, fill=c, **o)
                + "".join(path(f"M{num(cx + dx * s)},{num(cy - 11 * s)} l{num(dx * 0.2 * s)},{num(4 * s)}", stroke=dark(c, 0.45), stroke_width=1.4 * s) for dx in (-3, 0, 3))
                + ellipse(cx, cy + 5 * s, 6 * s, 4.5 * s, fill="#f4e4c8", **o) + poly([(cx - 1.6 * s, cy + 2 * s), (cx + 1.6 * s, cy + 2 * s), (cx, cy + 3.6 * s)], fill="#d86a7a")
                + eye(cx - 4.5 * s, cy - 2 * s, s * 0.8, "#4aa02a", "#e8f070", True) + eye(cx + 4.5 * s, cy - 2 * s, s * 0.8, "#4aa02a", "#e8f070", True)
                + "".join(line(cx + sx * 4 * s, cy + 5 * s + dy * s, cx + sx * 13 * s, cy + 4 * s + dy * 2 * s, stroke="#f4f4f4", stroke_width=0.4 * s) for sx in (-1, 1) for dy in (-1, 0, 1)))
    if race == "silicoid":
        p = [(cx - 9 * s, cy + 10 * s), (cx - 11 * s, cy - 3 * s), (cx - 6 * s, cy - 12 * s), (cx + 5 * s, cy - 13 * s), (cx + 11 * s, cy - 4 * s), (cx + 9 * s, cy + 10 * s), (cx, cy + 13 * s)]
        spikes = "".join(poly([(cx + dx * s, cy - 11 * s), (cx + (dx + 2) * s, cy - (19 + abs(dx) % 3) * s), (cx + (dx + 4) * s, cy - 11 * s)], fill=light(c, 0.35), **o) for dx in (-8, -3, 2))
        return (spikes + poly(p, fill=c, **o) + path(f"M{num(cx - 11 * s)},{num(cy - 3 * s)} L{num(cx + 11 * s)},{num(cy - 4 * s)} M{num(cx - 6 * s)},{num(cy - 12 * s)} L{num(cx)},{num(cy + 13 * s)}", stroke=dark(c, 0.3), stroke_width=0.5 * s)
                + ellipse(cx - 4.5 * s, cy, 2.6 * s, 1.6 * s, fill="#ffa030") + ellipse(cx + 4.5 * s, cy, 2.6 * s, 1.6 * s, fill="#ffa030"))
    if race == "sakkra":
        crest = "".join(poly([(cx + dx * s, cy - 10 * s), (cx + (dx + 1.5) * s, cy - 17 * s), (cx + (dx + 3) * s, cy - 10 * s)], fill="#c8402a", **o) for dx in (-6, -1.5, 3))
        return (crest + ellipse(cx, cy, 12 * s, 11 * s, fill=c, **o) + ellipse(cx, cy + 6 * s, 9 * s, 5 * s, fill=light(c, 0.2), **o)
                + "".join(circle(cx + dx * s, cy + dy * s, 0.9 * s, fill=dark(c, 0.25)) for dx, dy in ((-7, -6), (-4, -8), (0, -9), (4, -8), (7, -6), (-8, 2), (8, 2)))
                + eye(cx - 5 * s, cy - 2 * s, s * 0.85, "#1a1a1a", "#f0d040", True) + eye(cx + 5 * s, cy - 2 * s, s * 0.85, "#1a1a1a", "#f0d040", True)
                + path(f"M{num(cx - 6 * s)},{num(cy + 7 * s)} Q{num(cx)},{num(cy + 9 * s)} {num(cx + 6 * s)},{num(cy + 7 * s)}", fill="none", **o)
                + circle(cx - 1.5 * s, cy + 3 * s, 0.6 * s, fill="#1a1a1a") + circle(cx + 1.5 * s, cy + 3 * s, 0.6 * s, fill="#1a1a1a"))
    if race == "psilon":
        return (ellipse(cx, cy - 5 * s, 13 * s, 15 * s, fill=c, **o) + ellipse(cx, cy + 5 * s, 7.5 * s, 8 * s, fill=c, **o)
                + path(f"M{num(cx - 6 * s)},{num(cy - 14 * s)} q{num(2 * s)},{num(4 * s)} {num(0)},{num(7 * s)} M{num(cx + 5 * s)},{num(cy - 15 * s)} q{num(-2 * s)},{num(4 * s)} {num(1 * s)},{num(8 * s)}", stroke=dark(c, 0.25), stroke_width=0.5 * s, fill="none")
                + ellipse(cx - 3.8 * s, cy + 2 * s, 2.8 * s, 3.6 * s, fill="#101018") + ellipse(cx + 3.8 * s, cy + 2 * s, 2.8 * s, 3.6 * s, fill="#101018")
                + circle(cx - 4.5 * s, cy + 1 * s, 0.7 * s, fill="#ffffff") + circle(cx + 3.1 * s, cy + 1 * s, 0.7 * s, fill="#ffffff")
                + path(f"M{num(cx - 1.5 * s)},{num(cy + 9 * s)} h{num(3 * s)}", **o))
    if race == "alkari":
        crest = "".join(path(f"M{num(cx + dx * s)},{num(cy - 9 * s)} q{num(dx * 0.4 * s)},{num(-8 * s)} {num(dx * 1.3 * s)},{num(-10 * s)}", stroke="#e8c040", stroke_width=2 * s, fill="none") for dx in (-3, 0, 3))
        return (crest + ellipse(cx, cy, 10.5 * s, 12 * s, fill=c, **o) + path(f"M{num(cx - 8 * s)},{num(cy - 3 * s)} q{num(8 * s)},{num(-4 * s)} {num(16 * s)},0", fill=light(c, 0.3), stroke="none")
                + eye(cx - 4.5 * s, cy - 2 * s, s * 0.75, "#1a1a1a", "#f0a020") + eye(cx + 4.5 * s, cy - 2 * s, s * 0.75, "#1a1a1a", "#f0a020")
                + poly([(cx - 3 * s, cy + 1 * s), (cx + 3 * s, cy + 1 * s), (cx, cy + 10 * s)], fill="#f0c030", **o))
    if race == "klackon":
        return (path(f"M{num(cx - 3 * s)},{num(cy - 10 * s)} q{num(-4 * s)},{num(-8 * s)} {num(-9 * s)},{num(-9 * s)} M{num(cx + 3 * s)},{num(cy - 10 * s)} q{num(4 * s)},{num(-8 * s)} {num(9 * s)},{num(-9 * s)}", stroke="#1a1a1a", stroke_width=1 * s, fill="none")
                + ellipse(cx, cy, 9 * s, 12 * s, fill=c, **o) + ellipse(cx - 7 * s, cy - 2 * s, 4 * s, 6 * s, fill="#1a1010", **o) + ellipse(cx + 7 * s, cy - 2 * s, 4 * s, 6 * s, fill="#1a1010", **o)
                + circle(cx - 8 * s, cy - 4 * s, 1 * s, fill="#ffffff", opacity=0.6) + circle(cx + 6 * s, cy - 4 * s, 1 * s, fill="#ffffff", opacity=0.6)
                + path(f"M{num(cx - 4 * s)},{num(cy + 9 * s)} q{num(-1 * s)},{num(5 * s)} {num(3 * s)},{num(5 * s)} M{num(cx + 4 * s)},{num(cy + 9 * s)} q{num(1 * s)},{num(5 * s)} {num(-3 * s)},{num(5 * s)}", stroke="#1a1a1a", stroke_width=1.2 * s, fill="none"))
    if race == "bulrathi":
        return (circle(cx - 9 * s, cy - 10 * s, 4 * s, fill=c, **o) + circle(cx + 9 * s, cy - 10 * s, 4 * s, fill=c, **o) + ellipse(cx, cy, 12.5 * s, 12 * s, fill=c, **o)
                + ellipse(cx, cy + 5 * s, 6.5 * s, 5 * s, fill=light(c, 0.35), **o) + ellipse(cx, cy + 2.6 * s, 2.6 * s, 1.8 * s, fill="#101010")
                + circle(cx - 4.5 * s, cy - 3 * s, 1.5 * s, fill="#101010") + circle(cx + 4.5 * s, cy - 3 * s, 1.5 * s, fill="#101010")
                + path(f"M{num(cx - 2.5 * s)},{num(cy + 7 * s)} q{num(2.5 * s)},{num(1.5 * s)} {num(5 * s)},0", fill="none", **o))
    if race == "meklar":
        return (line(cx, cy - 13 * s, cx, cy - 19 * s, stroke="#1a1a1a", stroke_width=0.8 * s) + circle(cx, cy - 19.5 * s, 1.4 * s, fill="#ff4a3a")
                + rect(cx - 10 * s, cy - 13 * s, 20 * s, 25 * s, rx=6 * s, fill=c, **o) + rect(cx - 10 * s, cy - 4 * s, 20 * s, 6 * s, fill="#2a2f38", **o)
                + ellipse(cx + 2 * s, cy - 1 * s, 3.5 * s, 2 * s, fill="#ff3a2a") + rect(cx - 5 * s, cy + 5 * s, 10 * s, 3 * s, fill="#3a3f48", **o)
                + "".join(circle(cx + dx * s, cy + dy * s, 0.7 * s, fill=light(c, 0.4)) for dx, dy in ((-8, -11), (8, -11), (-8, 10), (8, 10))))
    # darlok
    return (path(f"M{num(cx - 13 * s)},{num(cy + 12 * s)} Q{num(cx - 14 * s)},{num(cy - 18 * s)} {num(cx)},{num(cy - 18 * s)} Q{num(cx + 14 * s)},{num(cy - 18 * s)} {num(cx + 13 * s)},{num(cy + 12 * s)}Z", fill="#1a1626", **o)
            + ellipse(cx, cy, 8 * s, 13 * s, fill=c, **o) + ellipse(cx - 3.5 * s, cy - 2 * s, 2.6 * s, 1.2 * s, fill="#6be3ff") + ellipse(cx + 3.5 * s, cy - 2 * s, 2.6 * s, 1.2 * s, fill="#6be3ff")
            + path(f"M{num(cx - 2 * s)},{num(cy + 7 * s)} h{num(4 * s)}", stroke=dark(c, 0.4), stroke_width=0.6 * s))


def bust(race, cx, cy, s, outfit, color=None):
    """Head and shoulders: `outfit` robe (envoys), coat (scientists), cloak (spies), suit (explorers)."""
    color = color or ACCENT[race]
    o = dict(stroke="#1a1a1a", stroke_width=0.8 * s)
    body = ""
    sh = f"M{num(cx - 26 * s)},{num(cy + 40 * s)} Q{num(cx - 25 * s)},{num(cy + 14 * s)} {num(cx - 8 * s)},{num(cy + 11 * s)} L{num(cx + 8 * s)},{num(cy + 11 * s)} Q{num(cx + 25 * s)},{num(cy + 14 * s)} {num(cx + 26 * s)},{num(cy + 40 * s)}Z"
    if outfit == "robe":
        body += path(sh, fill=color, **o) + path(f"M{num(cx - 8 * s)},{num(cy + 11 * s)} L{num(cx)},{num(cy + 26 * s)} L{num(cx + 8 * s)},{num(cy + 11 * s)}", fill=light(color, 0.5), **o)
        body += path(f"M{num(cx - 20 * s)},{num(cy + 40 * s)} Q{num(cx - 18 * s)},{num(cy + 22 * s)} {num(cx - 9 * s)},{num(cy + 14 * s)}", stroke="#e8c84a", stroke_width=1.6 * s, fill="none")
        body += path(f"M{num(cx + 20 * s)},{num(cy + 40 * s)} Q{num(cx + 18 * s)},{num(cy + 22 * s)} {num(cx + 9 * s)},{num(cy + 14 * s)}", stroke="#e8c84a", stroke_width=1.6 * s, fill="none")
    elif outfit == "coat":
        body += path(sh, fill="#eef2f6", **o) + path(f"M{num(cx - 6 * s)},{num(cy + 11 * s)} L{num(cx)},{num(cy + 22 * s)} L{num(cx + 6 * s)},{num(cy + 11 * s)}", fill=color, **o)
        body += line(cx, cy + 22 * s, cx, cy + 40 * s, stroke="#9aa4b0", stroke_width=0.8 * s) + rect(cx + 9 * s, cy + 22 * s, 6 * s, 1.5 * s, fill="#3a6ab0")
    elif outfit == "cloak":
        body += path(sh, fill="#1c1c24", **o) + path(f"M{num(cx - 15 * s)},{num(cy + 14 * s)} Q{num(cx - 17 * s)},{num(cy - 20 * s)} {num(cx)},{num(cy - 20 * s)} Q{num(cx + 17 * s)},{num(cy - 20 * s)} {num(cx + 15 * s)},{num(cy + 14 * s)}", fill="#24242e", **o)
    else:
        body += path(sh, fill="#8a7a52", **o) + rect(cx - 22 * s, cy + 24 * s, 44 * s, 3 * s, fill="#4a3a24") + circle(cx + 12 * s, cy + 20 * s, 2.5 * s, fill="#c0a040", **o)
    body += rect(cx - 4 * s, cy + 6 * s, 8 * s, 7 * s, fill=dark(SKIN[race], 0.15), **o)
    body += head(race, cx, cy, s)
    if outfit == "cloak":
        body += path(f"M{num(cx - 15 * s)},{num(cy + 14 * s)} Q{num(cx - 17 * s)},{num(cy - 20 * s)} {num(cx)},{num(cy - 20 * s)} Q{num(cx + 17 * s)},{num(cy - 20 * s)} {num(cx + 15 * s)},{num(cy + 14 * s)} Q{num(cx + 12 * s)},{num(cy - 12 * s)} {num(cx)},{num(cy - 13 * s)} Q{num(cx - 12 * s)},{num(cy - 12 * s)} {num(cx - 15 * s)},{num(cy + 14 * s)}Z", fill="#24242e", **o)
    if outfit == "suit":
        body += path(f"M{num(cx - 12 * s)},{num(cy - 8 * s)} Q{num(cx)},{num(cy - 22 * s)} {num(cx + 12 * s)},{num(cy - 8 * s)}Z", fill="#c8b070", **o) + rect(cx - 13 * s, cy - 9 * s, 26 * s, 2.5 * s, fill="#8a7a52", **o)
    return body


# ------------------------------------------------------------------ news pictures

def news_icon(t, R):
    """The picture of news type t (1oom GAME_NEWS_*: 1 plague .. 22 Guardian), 41x37."""
    c = lambda x, y, r, f, **k: circle(x, y, r, fill=f, **k)
    if t == 1:  # plague
        return c(20, 19, 11, "#7a9a3a") + "".join(c(20 + 10 * math.cos(a), 19 + 10 * math.sin(a), 2.5, "#b8d040") for a in [i * math.tau / 8 for i in range(8)]) + c(17, 16, 3, "#4a6a1a")
    if t == 2:  # quake
        return rect(4, 22, 33, 12, fill="#8a6a4a") + path("M4,22 L12,20 L16,26 L22,18 L27,25 L37,21", stroke="#2a1a10", stroke_width=2, fill="none") + path("M20,22 l-3,6 l4,2 l-2,4", stroke="#2a1a10", stroke_width=1.5, fill="none")
    if t == 3:  # nova
        return "".join(line(20, 18, 20 + 17 * math.cos(a), 18 + 16 * math.sin(a), stroke="#ffd04a", stroke_width=1.5) for a in [i * math.tau / 12 for i in range(12)]) + c(20, 18, 7, "#ffffff")
    if t == 4:  # accident
        return poly([(20, 4), (37, 33), (3, 33)], fill="#ffd04a", stroke="#3a2a10", stroke_width=1.5) + rect(19, 13, 2.5, 11, fill="#3a2a10") + c(20.2, 28, 1.6, "#3a2a10")
    if t == 5:  # assassin
        return path("M8,30 L28,10 L31,13 L11,33Z", fill="#c8d0dc", stroke="#1a1a1a") + rect(6, 27, 8, 3, fill="#5a3a20", transform="rotate(-45 10 28)") + c(30, 28, 3, "#b8231c")
    if t == 6:  # virus
        return rect(6, 8, 29, 20, rx=2, fill="#1a2a3a", stroke="#5ab4ff") + path("M10,14 h8 M10,18 h14 M10,22 h6", stroke="#3aff6a") + c(28, 18, 4, "#ff3a3a")
    if t == 7:  # comet
        return path("M6,6 Q20,14 30,28", stroke="#9fe8ff", stroke_width=5, opacity=0.5, fill="none") + c(30, 28, 5, "#e8f8ff")
    if t == 8:  # pirates
        return c(20, 16, 9, "#e8e8e8") + c(16, 15, 2.5, "#1a1a1a") + c(24, 15, 2.5, "#1a1a1a") + path("M8,28 L32,34 M8,34 L32,28", stroke="#e8e8e8", stroke_width=3)
    if t == 9:  # derelict ship
        return poly([(4, 20), (16, 12), (34, 14), (37, 22), (20, 27)], fill="#6a7480", stroke="#1a1a1a") + path("M18,14 l3,6 l-2,5", stroke="#1a1a1a", fill="none") + c(30, 18, 1.5, "#ff7a1a")
    if t == 10:  # rebellion
        return line(12, 34, 12, 6, stroke="#3a2a1a", stroke_width=2) + path("M12,6 L32,10 L12,16Z", fill="#d8302a") + path("M24,34 l4,-12 l4,12", fill="#e3b48c", stroke="#1a1a1a")
    if t == 11:  # crystal
        return poly([(20, 3), (27, 18), (20, 34), (13, 18)], fill="#9fe0ff", stroke="#e8f8ff")
    if t == 12:  # amoeba
        return path(blob(20, 19, 14, 11, R), fill="#5ad25a", stroke="#1f6a1f") + c(23, 17, 4, "#2f8a2f")
    if t == 13:  # environment
        return path("M20,32 v-14 M20,22 q-8,-2 -9,-10 q8,0 9,10 M20,20 q8,-2 9,-10 q-8,0 -9,10", stroke="#2f8a2f", stroke_width=2, fill="#6ad26a")
    if t == 14:  # rich minerals
        return poly([(8, 30), (14, 16), (22, 22), (30, 12), (34, 30)], fill="#8a7a6a") + poly([(16, 26), (20, 18), (24, 26)], fill="#ffd04a", stroke="#8a6a10")
    if t == 15:  # support
        return path("M8,22 l8,-6 h8 l8,6 l-8,8 h-8Z", fill="#e3b48c", stroke="#1a1a1a") + path("M16,16 l8,14", stroke="#1a1a1a")
    if t == 16:  # poor
        return poly([(8, 30), (14, 18), (22, 24), (30, 16), (34, 30)], fill="#6a5a4a") + path("M10,10 l20,16 M30,10 l-20,16", stroke="#d8302a", stroke_width=2)
    if t == 17:  # Orion
        return poly([(20, 4), (34, 32), (6, 32)], fill="#e8d07a", stroke="#5a4a20", stroke_width=1.5) + c(20, 22, 4, "#9fe8ff")
    if t == 18:  # coup
        return path("M8,30 h24 l-3,-10 l-6,5 l-3,-9 l-3,9 l-6,-5Z", fill="#ffd04a", stroke="#5a3a10") + path("M6,6 L34,32", stroke="#d8302a", stroke_width=2.5)
    if t == 19:  # stars (the year's ranking)
        return "".join(poly([(x + 5 * math.cos(a) * (1 if k % 2 == 0 else 0.45), y + 5 * math.sin(a) * (1 if k % 2 == 0 else 0.45)) for k, a in enumerate([-math.pi / 2 + i * math.pi / 5 for i in range(10)])], fill="#ffd04a", stroke="#5a3a10", stroke_width=0.6) for x, y in ((10, 12), (30, 12), (20, 26)))
    if t == 20:  # statistics
        return rect(5, 5, 31, 28, fill="#1a2a3a") + "".join(rect(8 + i * 7, 30 - hh, 5, hh, fill="#5ab4ff") for i, hh in enumerate((8, 14, 11, 20)))
    if t == 21:  # genocide
        return c(20, 17, 11, "#d8d8d8") + c(15, 15, 3, "#1a1a1a") + c(25, 15, 3, "#1a1a1a") + rect(15, 24, 10, 5, fill="#d8d8d8") + path("M17,24 v5 M20,24 v5 M23,24 v5", stroke="#1a1a1a")
    # 22: the Guardian
    return poly([(4, 18), (14, 14), (18, 6), (22, 14), (36, 18), (22, 24), (18, 32), (14, 24)], fill="#8f1d14", stroke="#1a0505") + c(18, 18, 3, "#ffd04a")


# ------------------------------------------------------------------ build

def build():
    # The embassy (1oom `ui_audience`): the room behind the envoy, the envoy (175x130 at 96,0),
    # the player's emperor seen from behind (at 0,-2) and the frame with the words' panel.
    R = rng("embassy")
    defs = lin("wall", 0, 0, 0, 1, (0, "#2a3446"), (1, "#141a26")) + lin("floor", 0, 0, 0, 1, (0, "#3a3020"), (1, "#1a140c")) + rad("win", 0.5, 0.5, 0.7, (0, "#1a2a5a"), (1, "#05070f"))
    body = rect(0, 0, 320, 130, fill="url(#wall)") + rect(110, 10, 150, 80, rx=6, fill="url(#win)", stroke="#8a96a8", stroke_width=3)
    for _ in range(40):
        body += circle(112 + R.random() * 146, 12 + R.random() * 76, R.uniform(0.3, 1), fill="#ffffff", opacity=R.uniform(0.3, 1))
    d2, b2 = globe("terran", 225, 70, 22, 5, "eg")
    defs += d2
    body += g(b2, clip_path="url(#wc)") + line(185, 10, 185, 90, stroke="#8a96a8", stroke_width=2)
    defs += '<clipPath id="wc"><rect x="110" y="10" width="150" height="80" rx="6"/></clipPath>'
    body += rect(0, 96, 320, 34, fill="url(#floor)") + rect(0, 94, 320, 3, fill="#8a7a52")
    for x in (40, 290):
        body += rect(x - 8, 14, 16, 82, fill="#3a4458", stroke="#8a96a8") + rect(x - 10, 10, 20, 6, fill="#8a96a8")
    put("classic", "embassy/room", 320, 130, body, defs)
    frame = "M0,0 H320 V200 H0Z M8,8 V128 H312 V8Z"
    defs = lin("fr", 0, 0, 0, 1, (0, "#7a869a"), (1, "#3a4252"))
    body = path(frame, fill="url(#fr)", fill_rule="evenodd", stroke="#1a1f28", stroke_width=1) + rect(8, 8, 304, 120, fill="none", stroke="#c8d0dc", stroke_width=1.5)
    body += rect(30, 134, 260, 62, rx=3, fill="#7a1a14", stroke="#1a0505", stroke_width=2) + rect(34, 137, 252, 56, fill="#4a0e0a")
    body += "".join(circle(x, 165, 4, fill="#c8d0dc", stroke="#1a1f28") for x in (16, 304))
    put("classic", "embassy/border", 320, 200, body, defs)
    for b, color in enumerate(BANNERS):
        o = dict(stroke="#0a0a0a", stroke_width=1)
        body = path("M-10,202 Q-8,112 42,106 Q92,112 96,202Z", fill=color, **o) + path("M8,202 Q14,140 42,132 Q70,140 76,202", fill="none", stroke=light(color, 0.5), stroke_width=3)
        body += ellipse(42, 92, 17, 20, fill="#3a2a1a", **o) + path("M28,84 Q42,62 56,84 L52,74 L46,80 L42,70 L38,80 L32,74Z", fill="#e8c84a", **o)
        put("classic", f"embassy/emperor/{b}", 320, 200, body)
    for r, race in enumerate(RACES):
        put("classic", f"embassy/{r}", 175, 130, bust(race, 88, 52, 2.1, "robe"))

    # A world map's audiences: the envoy's throne room, and the council hall (960x540).
    for r, race in enumerate(RACES):
        a = ACCENT[race]
        R = rng("hall", race)
        defs = lin("w", 0, 0, 0, 1, (0, dark(a, 0.55)), (1, dark(a, 0.85))) + lin("f", 0, 0, 0, 1, (0, "#2a2a34"), (1, "#0a0a10")) + rad("glow", 0.5, 0.4, 0.6, (0, light(a, 0.4), 0.6), (1, a, 0))
        body = rect(0, 0, 480, 270, fill="url(#w)") + rect(0, 190, 480, 80, fill="url(#f)")
        body += path("M140,190 L180,20 H300 L340,190Z", fill="#05070f", stroke=light(a, 0.3), stroke_width=4)
        for _ in range(60):
            body += circle(150 + R.random() * 180, 30 + R.random() * 150, R.uniform(0.4, 1.2), fill="#ffffff", opacity=R.uniform(0.3, 1))
        for x in (40, 100, 380, 440):
            body += rect(x - 12, 10, 24, 180, fill=dark(a, 0.3), stroke=light(a, 0.2), stroke_width=2) + rect(x - 16, 8, 32, 8, fill=light(a, 0.25))
        body += rect(0, 0, 480, 270, fill="url(#glow)") + path("M200,270 L220,190 H260 L280,270Z", fill=dark(a, 0.2), opacity=0.8)
        put("classic", f"hall/{r}", 960, 540, body, defs, vb="0 0 480 270")
    R = rng("council")
    defs = rad("dome", 0.5, 0.0, 1.0, (0, "#1a2a5a"), (1, "#05070f")) + lin("tier", 0, 0, 0, 1, (0, "#4a5468"), (1, "#1a1f28"))
    body = rect(0, 0, 480, 270, fill="url(#dome)") + starfield(R, 480, 150, 120, "#ffffff", 0.04, 1)
    for k in range(4):
        y = 150 + k * 28
        body += path(f"M{num(20 - k * 10)},{num(y + 30)} Q240,{num(y - 30)} {num(460 + k * 10)},{num(y + 30)} L{num(470 + k * 10)},{num(y + 40)} Q240,{num(y - 20)} {num(10 - k * 10)},{num(y + 40)}Z", fill="url(#tier)", stroke="#8a96a8", stroke_width=1)
        for i in range(9):
            x = 50 + i * 47 + (k % 2) * 20
            body += circle(x, y + 22 - 30 * math.sin(math.pi * (x / 480)) * 0.9, 4, fill=ACCENT[RACES[(i + k) % 10]], stroke="#0a0a0a", stroke_width=0.6)
    body += rect(220, 150, 40, 60, fill="#8a96a8", stroke="#1a1f28") + circle(240, 140, 14, fill="#ffd04a", opacity=0.8)
    put("classic", "council/hall", 960, 540, body, defs, vb="0 0 480 270")

    # The GNN newscast (1oom `ui_news`): the TV (its screen at 14,14 292x105, the text below),
    # the studio with the newscaster, the turning globe, the logo, and a picture per news.
    defs = lin("tv", 0, 0, 0, 1, (0, "#5a6478"), (1, "#20242e"))
    body = rect(0, 0, 320, 200, rx=10, fill="url(#tv)", stroke="#0a0a0a", stroke_width=2) + rect(10, 10, 300, 113, rx=6, fill="#0a0a0a", stroke="#9aa4b4", stroke_width=2)
    body += rect(28, 138, 263, 56, rx=3, fill="#0a0a0a", stroke="#9aa4b4") + "".join(circle(x, 166, 6, fill="#3a3f48", stroke="#9aa4b4") for x in (14, 306))
    body += text(160, 133, "GALACTIC NEWS NETWORK", font_family="sans-serif", font_size=7, font_weight="bold", fill="#ffd04a", text_anchor="middle")
    put("classic", "news/tv", 320, 200, body, defs)
    defs = lin("st", 0, 0, 0, 1, (0, "#1a2a5a"), (1, "#0a1020")) + lin("desk", 0, 0, 0, 1, (0, "#8a96a8"), (1, "#3a4252"))
    studio = rect(0, 0, 292, 105, fill="url(#st)") + "".join(line(x, 0, x, 105, stroke="#2a3a6a", stroke_width=1) for x in range(0, 292, 24))
    anchor = studio + bust("human", 146, 40, 1.35, "robe", "#2a3a5a") + rect(80, 78, 132, 27, fill="url(#desk)", stroke="#1a1f28") + text(146, 96, "GNN", font_family="sans-serif", font_size=12, font_weight="bold", fill="#ffd04a", text_anchor="middle")
    put("classic", "news/anchor", 292, 105, anchor, defs)
    put("classic", "news/gnn", 292, 105, studio + text(146, 64, "GNN", font_family="sans-serif", font_size=44, font_weight="bold", fill="#ffd04a", text_anchor="middle", stroke="#5a3a10", stroke_width=1.5), defs)
    d, b = globe("terran", 24.5, 21, 19, 11, "w")
    put("classic", "news/world", 49, 42, b, d)
    for t in range(1, 23):
        body = rect(0, 0, 41, 37, fill="#0e1830", stroke="#5a6478") + news_icon(t, rng("news", t))
        put("classic", f"news/icon/{t}", 41, 37, body)
    # A world map's newscast: the herald (a messenger droid with its scroll).
    body = ellipse(58, 232, 40, 6, fill="#000000", opacity=0.3) + path("M30,230 L36,120 Q58,104 80,120 L86,230Z", fill="#5a6478", stroke="#0a0a0a", stroke_width=2)
    body += rect(40, 60, 36, 50, rx=10, fill="#8f9aa8", stroke="#0a0a0a", stroke_width=2) + rect(44, 76, 28, 10, fill="#20242e") + ellipse(64, 81, 5, 3, fill="#5ab4ff")
    body += line(58, 60, 58, 44, stroke="#0a0a0a", stroke_width=2) + circle(58, 42, 4, fill="#ffd04a")
    body += rect(76, 130, 34, 46, rx=3, fill="#f0e6c8", stroke="#5a4a20", stroke_width=2) + "".join(line(80, y, 106, y, stroke="#8a7a52") for y in range(138, 172, 6))
    put("classic", "news/herald", 117, 240, body)

    # A discovery (1oom `ui_newtech`): the laboratory, the spies' den and the ruins, with the
    # display where the tech's picture goes (145,54 119x78), and who brought it, by race:
    # scientists (0..9), spies (10..19) and explorers (20..29), standing on the left.
    for i, name in enumerate(["lab", "den", "ruins"]):
        R = rng("lab", name)
        if name == "lab":
            defs = lin("w", 0, 0, 0, 1, (0, "#2a3a4a"), (1, "#141c26"))
            body = rect(0, 0, 320, 200, fill="url(#w)") + rect(0, 150, 320, 50, fill="#1a222c")
            for k in range(6):
                x = 8 + k * 22
                body += rect(x, 20, 16, 40, fill="#0e141c", stroke="#5a6a7a") + "".join(circle(x + 8, 28 + j * 9, 2, fill=R.choice(["#3aff6a", "#ff5a4a", "#ffd04a", "#5ab4ff"])) for j in range(4))
            for x, c in ((290, "#3aff6a"), (305, "#5ab4ff")):
                body += path(f"M{x - 5},{150} l3,-18 v-10 h4 v10 l3,18Z", fill=c, opacity=0.7, stroke="#c8d0dc")
        elif name == "den":
            defs = lin("w", 0, 0, 0, 1, (0, "#1a1410"), (1, "#0a0806"))
            body = rect(0, 0, 320, 200, fill="url(#w)") + rect(0, 150, 320, 50, fill="#120e0a")
            body += path("M200,0 L150,40 H250Z", fill="#3a3020") + ellipse(200, 44, 60, 10, fill="#ffd88a", opacity=0.15)
            for k in range(4):
                body += rect(275, 60 + k * 22, 40, 20, fill="#3a3a3a", stroke="#1a1a1a") + rect(290, 68 + k * 22, 10, 3, fill="#8a8a8a")
        else:
            defs = lin("w", 0, 0, 0, 1, (0, "#3a2a4a"), (1, "#8a6a5a"))
            body = rect(0, 0, 320, 200, fill="url(#w)") + starfield(R, 320, 80, 40, "#ffffff", 0.05, 0.8) + rect(0, 150, 320, 50, fill="#6a5a44")
            for x in (20, 60, 285):
                hh = R.uniform(90, 130)
                body += rect(x, 150 - hh, 22, hh, fill="#b8a888", stroke="#5a4a30") + rect(x - 4, 150 - hh - 6, 30, 7, fill="#c8b898", stroke="#5a4a30")
        # The display (or altar) holding the tech's picture.
        body += rect(139, 48, 131, 90, rx=4, fill="#0a0e18", stroke="#9aa4b4" if name != "ruins" else "#c8b070", stroke_width=3)
        body += rect(160, 138, 89, 12, fill="#3a4252" if name != "ruins" else "#8a7a52", stroke="#0a0a0a")
        put("classic", f"newtech/lab/{i}", 320, 200, body, defs)
    for src, outfit in enumerate(["coat", "cloak", "suit"]):
        for r, race in enumerate(RACES):
            put("classic", f"newtech/{src * 10 + r}", 320, 200, bust(race, 72, 112, 2.0, outfit))

    # The siege screen's soldiers (8x16, one per race, facing right) and a fallen one (17x16).
    for r, race in enumerate(RACES):
        c = ACCENT[race]
        body = rect(4, 26, 3, 6, fill="#2a2a2a") + rect(9, 26, 3, 6, fill="#2a2a2a") + rect(3, 13, 10, 14, rx=2, fill=c, stroke="#0a0a0a", stroke_width=0.6)
        body += f'<g transform="translate(8 8) scale(0.36) translate(-50 -50)">{head(race, 50, 50, 1.6)}</g>' + rect(10, 15, 6, 2, fill="#c8d0dc", stroke="#0a0a0a", stroke_width=0.3)
        put("classic", f"ground/soldier/{r}", 8, 16, body, vb="0 0 16 32")
    put("classic", "ground/death", 17, 16, ellipse(8.5, 13, 8, 2.5, fill="#000000", opacity=0.3) + rect(1, 9, 14, 4, rx=2, fill="#5a6478", stroke="#0a0a0a", stroke_width=0.5) + circle(15, 11, 2.2, fill="#c8b8a0", stroke="#0a0a0a", stroke_width=0.4))
