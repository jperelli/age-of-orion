"""Classic's space pictures: stars, nebulae, planets, ships and monsters, the battle screen's
pieces and the map markers; the Age theme's mists."""
import math

from .kit import (blob, blur, circle, dark, ellipse, g, light, lin, line, mix, num, path, pline, poly, put, rad, rect, rng, rot,
                  starfield)

BANNERS = ["#4f7dff", "#3fd25a", "#b46cff", "#ff4a3d", "#f4f4f4", "#ffd23f"]

# ------------------------------------------------------------------ planets

# Colours per kind of planet: base, light, dark, feature, clouds.
PALETTE = {
    "not_habitable": ("#a07a5a", "#e8c79a", "#3a2a2a", "#7a4a6a", None),
    "radiated": ("#9aa83a", "#e6f27a", "#2f3a10", "#5a6a1a", "#d8ff6a"),
    "toxic": ("#6f9a3a", "#c8e070", "#1f3a14", "#a8b83a", "#c8d860"),
    "inferno": ("#c0461f", "#ffb04a", "#3a0a05", "#ff7a1a", None),
    "dead": ("#8a8a8a", "#d0d0d0", "#2a2a2a", "#5f5f5f", None),
    "tundra": ("#b8c8d8", "#ffffff", "#3a4a5a", "#8aa0b0", "#ffffff"),
    "barren": ("#8a6a52", "#c8a888", "#2a1f18", "#6a5040", None),
    "minimal": ("#b89a6a", "#ecd8a8", "#3a2a1a", "#8a7a52", "#ffffff"),
    "desert": ("#d89a4a", "#ffd88a", "#4a2a10", "#b8743a", None),
    "steppe": ("#a8a24a", "#e8e08a", "#3a3410", "#7a8a3a", "#ffffff"),
    "arid": ("#c08a5a", "#f0c898", "#3a2414", "#9a6a3a", "#ffffff"),
    "ocean": ("#2f6ad0", "#8ac8ff", "#0a1a40", "#3a8a4a", "#ffffff"),
    "jungle": ("#2f8a3a", "#8ad86a", "#0a2a10", "#1f5a2a", "#e8fff0"),
    "terran": ("#2f6ad0", "#8ac8ff", "#0a1a40", "#4a9a3a", "#ffffff"),
}

# PLANETS.LBX picture -> the kind that uses it most (1oom `tbl_planet_gfx` via galaxy.rs PLANET_IMAGES).
IMAGE_KIND = {0: 'not_habitable', 1: 'barren', 2: 'jungle', 3: 'toxic', 4: 'radiated', 5: 'toxic', 6: 'dead', 7: 'dead', 8: 'minimal', 9: 'terran', 10: 'inferno',
              11: 'jungle', 12: 'inferno', 13: 'ocean', 14: 'steppe', 15: 'jungle', 16: 'desert', 17: 'steppe', 18: 'terran', 19: 'radiated', 20: 'arid',
              21: 'jungle', 22: 'toxic', 23: 'inferno', 24: 'minimal', 25: 'barren', 26: 'ocean', 27: 'arid', 28: 'desert', 29: 'tundra', 30: 'steppe',
              31: 'tundra', 32: 'terran', 33: 'radiated', 34: 'dead'}


def globe(kind, cx, cy, r, seed, uid="p"):
    """A planet seen from space, lit from the upper left: (defs, body)."""
    base, hi, lo, feat, cloud = PALETTE[kind]
    R = rng("globe", kind, seed)
    defs = rad(f"{uid}b", 0.38, 0.32, 0.75, (0, hi), (0.55, base), (1, lo))
    defs += rad(f"{uid}s", 0.3, 0.25, 0.95, (0, "#000000", 0), (0.62, "#000000", 0), (1, "#000000", 0.75))
    defs += f'<clipPath id="{uid}c"><circle cx="{num(cx)}" cy="{num(cy)}" r="{num(r)}"/></clipPath>'
    inner = circle(cx, cy, r, fill=f"url(#{uid}b)")
    if kind == "not_habitable":
        # A gas giant: bands.
        for k in range(7):
            y = cy - r + (k + 0.5) * 2 * r / 7 + R.uniform(-0.6, 0.6) * r / 7
            inner += ellipse(cx, y, r * 1.2, r * R.uniform(0.06, 0.14), fill=R.choice([feat, light(base, 0.3), dark(base, 0.25)]), opacity=0.7)
        inner += ellipse(cx + r * 0.3, cy + r * 0.25, r * 0.22, r * 0.12, fill=dark(feat, 0.2), opacity=0.8)
    elif kind in ("dead", "barren", "minimal", "radiated"):
        for _ in range(7):
            x, y, cr = cx + R.uniform(-0.8, 0.8) * r, cy + R.uniform(-0.8, 0.8) * r, r * R.uniform(0.08, 0.2)
            inner += circle(x, y, cr, fill=dark(base, 0.25), opacity=0.8) + circle(x - cr * 0.25, y - cr * 0.25, cr * 0.7, fill=light(base, 0.15), opacity=0.6)
        for _ in range(3):
            inner += path(blob(cx + R.uniform(-0.6, 0.6) * r, cy + R.uniform(-0.6, 0.6) * r, r * 0.35, r * 0.22, R), fill=feat, opacity=0.5)
    elif kind in ("desert", "arid", "steppe"):
        for k in range(5):
            y = cy - r + (k + 0.5) * 2 * r / 5
            inner += ellipse(cx, y, r * 1.1, r * R.uniform(0.05, 0.1), fill=R.choice([feat, light(base, 0.2)]), opacity=0.55)
        for _ in range(4):
            inner += path(blob(cx + R.uniform(-0.7, 0.7) * r, cy + R.uniform(-0.7, 0.7) * r, r * 0.3, r * 0.2, R), fill=feat, opacity=0.6)
    elif kind == "inferno":
        for _ in range(6):
            x, y = cx + R.uniform(-0.8, 0.8) * r, cy + R.uniform(-0.8, 0.8) * r
            inner += path(f"M{num(x)},{num(y)} l{num(R.uniform(-1, 1) * r * 0.5)},{num(R.uniform(-1, 1) * r * 0.3)} l{num(R.uniform(-1, 1) * r * 0.4)},{num(R.uniform(-1, 1) * r * 0.4)}",
                          stroke="#ffd04a", stroke_width=r * 0.07, fill="none", opacity=0.9)
        for _ in range(4):
            inner += path(blob(cx + R.uniform(-0.7, 0.7) * r, cy + R.uniform(-0.7, 0.7) * r, r * 0.3, r * 0.2, R), fill=dark(base, 0.35), opacity=0.7)
    else:
        # Continents (ocean, terran, jungle, tundra, toxic).
        n = {"ocean": 3, "terran": 5, "jungle": 6, "tundra": 4, "toxic": 5}[kind]
        for _ in range(n):
            land = feat if kind != "tundra" else light(base, 0.5)
            inner += path(blob(cx + R.uniform(-0.75, 0.75) * r, cy + R.uniform(-0.7, 0.7) * r, r * R.uniform(0.2, 0.45), r * R.uniform(0.15, 0.35), R, 10, 0.35),
                          fill=land, opacity=0.95)
        if kind == "terran":
            for _ in range(3):
                inner += path(blob(cx + R.uniform(-0.6, 0.6) * r, cy + R.uniform(-0.6, 0.6) * r, r * 0.15, r * 0.1, R), fill="#a8864a", opacity=0.9)
        if kind in ("terran", "ocean", "tundra"):
            inner += ellipse(cx, cy - r * 0.93, r * 0.55, r * 0.16, fill="#ffffff", opacity=0.9) + ellipse(cx, cy + r * 0.95, r * 0.45, r * 0.12, fill="#ffffff", opacity=0.85)
    if cloud:
        for _ in range(4):
            x, y = cx + R.uniform(-0.8, 0.8) * r, cy + R.uniform(-0.8, 0.8) * r
            inner += ellipse(x, y, r * R.uniform(0.25, 0.5), r * R.uniform(0.05, 0.1), fill=cloud, opacity=0.55, transform=f"rotate({num(R.uniform(-15, 15))} {num(x)} {num(y)})")
    inner += circle(cx, cy, r, fill=f"url(#{uid}s)")
    body = g(inner, clip_path=f"url(#{uid}c)") + circle(cx, cy, r, fill="none", stroke=light(base, 0.4), stroke_width=r * 0.06, opacity=0.6)
    return defs, body


# ------------------------------------------------------------------ ships

# Six ship silhouettes (nose to the right) per hull size, around (0, 0).
SHAPES = [
    [14, 0, -8, -8, -4, -2, -13, -4, -13, 4, -4, 2, -8, 8],
    [14, 0, -12, -9, -8, 0, -12, 9],
    [12, 0, 6, -6, -6, -7, -12, -3, -12, 3, -6, 7, 6, 6],
    [14, -2, 2, -9, -12, -9, -12, -4, -4, 0, -12, 4, -12, 9, 2, 9, 14, 2],
    [15, 0, 0, -3, -6, -10, -10, -10, -6, -2, -13, -2, -13, 2, -6, 2, -10, 10, -6, 10, 0, 3],
    [12, -3, 8, -7, -10, -7, -13, -3, -13, 3, -10, 7, 8, 7, 12, 3],
]
HULL_SCALE = [0.5, 0.66, 0.84, 1.04]


def ship(color, hull, variant, uid="s"):
    """A ship in a 32x24 cell facing right: (defs, body)."""
    s = HULL_SCALE[hull]
    sh = SHAPES[variant]
    p = [(16 + sh[i] * s, 12 + sh[i + 1] * s) for i in range(0, len(sh), 2)]
    defs = lin(f"{uid}h", 0, 0, 0, 1, (0, light(color, 0.45)), (0.5, color), (1, dark(color, 0.55)))
    defs += rad(f"{uid}e", 0.5, 0.5, 0.5, (0, "#fff6c0"), (0.5, "#ffb347"), (1, "#ff6a1a", 0))
    back = min(x for x, _ in p)
    body = ellipse(back - 1.2 * s, 12, 3.2 * s, 2.2 * s, fill=f"url(#{uid}e)")
    if hull >= 2:
        body += ellipse(back - 0.5 * s, 12 - 5 * s, 2 * s, 1.4 * s, fill=f"url(#{uid}e)") + ellipse(back - 0.5 * s, 12 + 5 * s, 2 * s, 1.4 * s, fill=f"url(#{uid}e)")
    body += poly(p, fill=f"url(#{uid}h)", stroke="#0b0b0b", stroke_width=0.7 * max(s, 0.7), stroke_linejoin="round")
    body += line(back + 2 * s, 12, 16 + 8 * s, 12, stroke=dark(color, 0.45), stroke_width=0.5, opacity=0.8)
    if hull >= 1:
        body += line(16 - 4 * s, 12 - 3 * s, 16 + 3 * s, 12 - 3 * s, stroke=light(color, 0.6), stroke_width=0.4, opacity=0.8)
    body += ellipse(16 + 5 * s, 12, 2.6 * s, 1.3 * s, fill="#bff0ff", stroke="#123", stroke_width=0.3)
    return defs, body


def crystal(cx, cy, k):
    out = ""
    for a, l, w in ((-0.4, 11, 3.5), (0.5, 9, 3), (0, 13, 4), (-1.1, 7, 2.5), (1.2, 7, 2.5)):
        tip = rot(cx, cy - l * k, a, cx, cy)
        left = rot(cx - w * k, cy, a, cx, cy)
        right = rot(cx + w * k, cy, a, cx, cy)
        out += poly([left, tip, right, rot(cx, cy + 2 * k, a, cx, cy)], fill="url(#cr)", stroke="#e8f8ff", stroke_width=0.5 * k)
    return out


def amoeba(cx, cy, k, R):
    return path(blob(cx, cy, 11 * k, 8 * k, R, 11, 0.22), fill="url(#am)", stroke="#1f6a1f", stroke_width=0.6 * k) + \
        circle(cx + 2 * k, cy - 1 * k, 3 * k, fill="#2f8a2f", opacity=0.8) + circle(cx - 4 * k, cy + 2 * k, 1.4 * k, fill="#d8ffb0", opacity=0.7)


def guardian(cx, cy, k):
    p = [(14, 0), (6, -4), (2, -10), (-2, -5), (-10, -8), (-8, -2), (-13, 0), (-8, 2), (-10, 8), (-2, 5), (2, 10), (6, 4)]
    return poly([(cx + x * k, cy + y * k) for x, y in p], fill="url(#gd)", stroke="#1a0505", stroke_width=0.7 * k, stroke_linejoin="round") + \
        circle(cx + 3 * k, cy, 2.2 * k, fill="#ffd04a", stroke="#5a1a05", stroke_width=0.4 * k)


MONSTER_DEFS = (rad("cr", 0.4, 0.3, 0.8, (0, "#ffffff"), (0.4, "#9fe0ff"), (1, "#2f6ab8")) +
                rad("am", 0.4, 0.35, 0.8, (0, "#c8ff9a"), (0.6, "#5ad25a"), (1, "#1f6a1f", 0.9)) +
                lin("gd", 0, 0, 0, 1, (0, "#d8504a"), (0.5, "#8f1d14"), (1, "#3a0805")))


# ------------------------------------------------------------------ build

def build():
    # Stars: 1oom's star types (yellow, red, green, white, blue, neutron), small (7x7) and big (13x11).
    colors = ["#ffe36b", "#ff6a4d", "#7dff8a", "#f4f4ff", "#7fb6ff", "#c58cff"]
    for k in range(12):
        c = colors[k % 6]
        w, h = (13, 11) if k >= 6 else (7, 7)
        cx, cy = w / 2, h / 2
        r = min(w, h) / 2
        defs = rad("glow", 0.5, 0.5, 0.5, (0, "#ffffff"), (0.22, c), (0.6, c, 0.35), (1, c, 0))
        defs += lin("gx", 0, 0, 1, 0, (0, c, 0), (0.5, "#ffffff", 0.85), (1, c, 0)) + lin("gy", 0, 0, 0, 1, (0, c, 0), (0.5, "#ffffff", 0.85), (1, c, 0))
        body = circle(cx, cy, r, fill="url(#glow)")
        body += rect(0, cy - 0.3, w, 0.6, fill="url(#gx)") + rect(cx - 0.3, 0, 0.6, h, fill="url(#gy)")
        body += circle(cx, cy, r * 0.28, fill="#ffffff")
        put("classic", f"star/{k}", w, h, body, defs)

    # Nebulae: clouds over 1oom's four rectangles of each kind (`tbl_nebula_data`), 97x85
    # pixels drawn at half size on the map. Classic: purple gas; Age: drifting mists.
    data = [
        [[0x14, 0x07, 0x0f, 0x1f], [0x05, 0x12, 0x0c, 0x02], [0x28, 0x1c, 0x1f, 0x24], [0x14, 0x27, 0x1b, 0x04]],
        [[0x0a, 0x14, 0x1b, 0x1e], [0x11, 0x0d, 0x0d, 0x05], [0x1a, 0x26, 0x29, 0x29], [0x25, 0x18, 0x10, 0x10]],
        [[0x09, 0x03, 0x17, 0x1a], [0x02, 0x08, 0x12, 0x0f], [0x16, 0x19, 0x29, 0x23], [0x19, 0x15, 0x20, 0x24]],
        [[0x19, 0x08, 0x0a, 0x10], [0x0c, 0x0f, 0x09, 0x07], [0x1e, 0x1c, 0x16, 0x17], [0x1c, 0x1d, 0x0f, 0x09]],
        [[0x07, 0x0f, 0x15, 0x19], [0x02, 0x0f, 0x16, 0x23], [0x19, 0x25, 0x29, 0x29], [0x13, 0x1c, 0x23, 0x27]],
        [[0x03, 0x08, 0x0e, 0x09], [0x0e, 0x15, 0x02, 0x0a], [0x23, 0x24, 0x1a, 0x22], [0x14, 0x23, 0x1f, 0x0f]],
        [[0x0d, 0x09, 0x13, 0x05], [0x03, 0x0b, 0x21, 0x16], [0x1b, 0x22, 0x1b, 0x08], [0x21, 0x20, 0x25, 0x1e]],
        [[0x08, 0x0f, 0x1c, 0x25], [0x08, 0x18, 0x02, 0x0a], [0x25, 0x22, 0x23, 0x27], [0x19, 0x1f, 0x07, 0x12]],
        [[0x08, 0x05, 0x05, 0x1b], [0x06, 0x01, 0x0c, 0x03], [0x23, 0x0c, 0x09, 0x25], [0x1f, 0x08, 0x15, 0x0a]],
        [[0x04, 0x08, 0x0d, 0x25], [0x02, 0x12, 0x0e, 0x11], [0x2a, 0x26, 0x26, 0x29], [0x12, 0x1e, 0x21, 0x1d]],
    ]
    for kind, t in enumerate(data):
        rects = [(t[0][j] * 2, t[1][j] * 2, t[2][j] * 2, t[3][j] * 2) for j in range(4)]
        for theme, cols in (("classic", ["#7a2fb8", "#b04ad8", "#d86ae8", "#5a2fa0"]), ("age", ["#d6d6e8", "#eeeef6", "#c0c4d0", "#ffffff"])):
            R = rng("nebula", theme, kind)
            body = ""
            for x0, y0, x1, y1 in rects:
                x0, x1 = min(x0, x1), max(x0, x1)
                y0, y1 = min(y0, y1), max(y0, y1)
                cx, cy, rx, ry = (x0 + x1) / 2, (y0 + y1) / 2, max(5, (x1 - x0) / 2 + 5), max(5, (y1 - y0) / 2 + 5)
                for _ in range(4):
                    body += path(blob(cx + R.uniform(-0.3, 0.3) * rx, cy + R.uniform(-0.3, 0.3) * ry, rx * R.uniform(0.6, 1), ry * R.uniform(0.6, 1), R, 9, 0.3),
                                 fill=R.choice(cols), opacity=0.32 if theme == "classic" else 0.4)
            stars = starfield(R, 97, 85, 14, "#ffffff", 0.1, 0.6) if theme == "classic" else ""
            put(theme, f"nebula/{kind}", 97, 85, g(body, filter="url(#b)") + stars, blur("b", 3.5))

    # Planets: MOO1's 35 pictures (32x24), each in the colours of the kind that uses it.
    for i in range(35):
        defs, body = globe(IMAGE_KIND[i], 16, 12, 10.5, i)
        put("classic", f"planet/{i}", 32, 24, body, defs)

    # Ships: 4 hull sizes x 6 looks in each empire's colour (`ship/<banner * 24 + look>`),
    # then the monsters (0x90 crystal, 0x91 amoeba, 0x92 Guardian).
    for b, color in enumerate(BANNERS):
        for hull in range(4):
            for v in range(6):
                defs, body = ship(color, hull, v)
                put("classic", f"ship/{b * 24 + hull * 6 + v}", 32, 24, body, defs)
    put("classic", "ship/144", 32, 24, crystal(16, 14, 0.85), MONSTER_DEFS)
    put("classic", "ship/145", 32, 24, amoeba(16, 12, 1, rng("amoeba")), MONSTER_DEFS)
    put("classic", "ship/146", 32, 24, guardian(16, 12, 1.05), MONSTER_DEFS)
    # The same as troops of a world map (`unit/<look>`, no empire colour: the game adds a pennant).
    for hull in range(4):
        for v in range(6):
            defs, body = ship("#9aa6b4", hull, v)
            put("classic", f"unit/{hull * 6 + v}", 32, 24, body, defs)
    put("classic", "unit/crystal", 32, 24, crystal(16, 14, 0.85), MONSTER_DEFS)
    put("classic", "unit/amoeba", 32, 24, amoeba(16, 12, 1, rng("amoeba")), MONSTER_DEFS)
    put("classic", "unit/guardian", 32, 24, guardian(16, 12, 1.05), MONSTER_DEFS)

    # Map markers: monsters, the Guardian, the stargate, colonies, fleets and transports.
    put("classic", "monster/crystal", 64, 64, crystal(32, 38, 2.2), MONSTER_DEFS)
    put("classic", "monster/amoeba", 64, 64, amoeba(32, 32, 2.3, rng("amoeba")), MONSTER_DEFS)
    put("classic", "guardian", 64, 64, guardian(32, 32, 2.2), MONSTER_DEFS)
    gate = rad("gi", 0.5, 0.5, 0.5, (0, "#ffffff"), (0.4, "#9fe8ff"), (1, "#2f6ab8", 0.2))
    gate_body = ellipse(23, 13, 20, 10, fill="url(#gi)") + ellipse(23, 13, 20, 10, fill="none", stroke="#c0c8d8", stroke_width=4) + \
        ellipse(23, 13, 20, 10, fill="none", stroke="#5a6478", stroke_width=1.2) + \
        "".join(circle(23 + 20 * math.cos(a), 13 + 10 * math.sin(a), 1.3, fill="#ffd04a") for a in [i * math.tau / 8 for i in range(8)])
    put("classic", "stargate", 46, 26, gate_body, gate)
    dome = lin("d", 0, 0, 0, 1, (0, "#e8f4ff", 0.95), (1, "#6a8ab8", 0.85))
    for level, name in enumerate(["village", "town", "castle"]):
        R = rng("colony", name)
        body = ellipse(32, 50, 26, 7, fill="#3a3f4a")
        domes = [(32, 46, 12)] + [(18, 48, 7), (46, 48, 7)][: level + 1] + ([(32, 36, 7), (12, 50, 5), (52, 50, 5)] if level == 2 else [])
        for x, y, r in sorted(domes, key=lambda d: d[1]):
            body += path(f"M{num(x - r)},{num(y)} A{num(r)},{num(r)} 0 0 1 {num(x + r)},{num(y)}Z", fill="url(#d)", stroke="#1c2630", stroke_width=1)
            body += rect(x - 1, y - r - 4, 2, 4, fill="#c0c8d8") if R.random() < 0.6 else ""
        put("classic", f"mark/{name}", 64, 64, body, dome)
    fleet, fleet_defs = "", ""
    for x, y, k in ((22, 22, 1.1), (40, 34, 1.3), (20, 44, 0.9)):
        d, b = ship("#c8d0dc", 1, 1, f"f{x}")
        fleet += f'<g transform="translate({num(x - 16 * k)} {num(y - 12 * k)}) scale({num(k)})">{b}</g>'
        fleet_defs += d
    put("classic", "army", 64, 64, fleet, fleet_defs)
    d, b = ship("#d8c8a0", 3, 5, "t")
    put("classic", "caravan", 64, 64, f'<g transform="translate(0 8) scale(2)">{b}</g>', d)

    # Battle: backgrounds (0 inside a nebula), asteroids, explosion frames and missiles.
    for i in range(5):
        R = rng("battle-bg", i)
        defs = rad("neb", 0.6, 0.4, 0.7, (0, "#a040d0", 0.55), (1, "#2a0f40", 0)) + blur("b", 8)
        body = rect(0, 0, 320, 200, fill="#1b0f2a" if i == 0 else "#04060e")
        if i == 0:
            for _ in range(9):
                body += path(blob(R.uniform(0, 320), R.uniform(0, 200), R.uniform(40, 90), R.uniform(25, 60), R), fill=R.choice(["#7a2fb8", "#b04ad8", "#4a1f80"]), opacity=0.35, filter="url(#b)")
        body += starfield(R, 320, 200, 160, "#ffffff", 0.04, 0.9)
        if i == 1:
            for k in range(160):
                a = k * 0.25
                rr = 2 + k * 0.35
                body += circle(240 + math.cos(a) * rr * 1.6, 60 + math.sin(a) * rr * 0.6, 0.8, fill="#d8d0ff", opacity=0.6 - k / 320)
            body += ellipse(240, 60, 10, 4, fill="#fff4d0", opacity=0.6, filter="url(#b)")
        if i == 2:
            d2, b2 = globe("terran", 300, 210, 70, 7, "bp")
            defs += d2
            body += b2
        if i == 3:
            body += path("M0,150 Q160,90 320,40 L320,70 Q160,120 0,180Z", fill="#6a5a8a", opacity=0.18, filter="url(#b)")
        if i == 4:
            d2, b2 = globe("not_habitable", 50, 40, 26, 3, "bp")
            defs += d2
            body += ellipse(50, 40, 46, 9, fill="none", stroke="#d8c8a0", stroke_width=3, opacity=0.6) + b2 + \
                path("M10,42 A46,9 0 0 0 90,42", fill="none", stroke="#d8c8a0", stroke_width=3, opacity=0.6)
        put("classic", f"battle/bg/{i}", 320, 200, body, defs)
    for i in range(4):
        R = rng("rock", i)
        defs = rad("r", 0.35, 0.3, 0.8, (0, "#b8a890"), (0.6, "#7a6a58"), (1, "#3a3028"))
        body = path(blob(16, 12, 11 - i, 8.5 - i * 0.5, R, 9, 0.28), fill="url(#r)", stroke="#2a2420", stroke_width=0.6)
        for _ in range(3):
            x, y, r = 16 + R.uniform(-5, 5), 12 + R.uniform(-3, 3), R.uniform(1, 2.2)
            body += circle(x, y, r, fill="#5a4c40", opacity=0.8) + circle(x - r * 0.3, y - r * 0.3, r * 0.6, fill="#9a8a74", opacity=0.5)
        put("classic", f"battle/rock/{i}", 32, 24, body, defs)
    for f in range(10):
        k = (f + 1) / 10
        defs = rad("x", 0.5, 0.5, 0.5, (0, "#ffffff", 1 - k * 0.6), (0.3, "#ffe36b", 1 - k * 0.5), (0.65, "#ff7a1a", 0.9 - k * 0.5), (1, "#7a1a05", 0))
        R = rng("boom", f)
        body = circle(16, 15, 3 + 12 * math.sqrt(k), fill="url(#x)")
        for _ in range(6):
            a = R.uniform(0, math.tau)
            d = 4 + 11 * k
            body += circle(16 + math.cos(a) * d, 15 + math.sin(a) * d, 1.6 * (1 - k * 0.6), fill="#ffd04a", opacity=1 - k * 0.8)
        put("classic", f"battle/explosion/{f}", 32, 30, body, defs)
    missile_cols = ["#d0d0d0", "#ffe36b", "#7fe0ff", "#ff7ad8"]
    for kind in range(5):
        for d in range(8):
            a = (d - 2) * 45
            if kind < 4:
                body = g(poly([(6.5, 3.5), (4, 2.6), (1.5, 2.8), (1.5, 4.2), (4, 4.4)], fill=missile_cols[kind], stroke="#202020", stroke_width=0.3) +
                         ellipse(1.1, 3.5, 1, 0.6, fill="#ffb347"), transform=f"rotate({a} 3.5 3.5)")
                put("classic", f"missile/{kind}/{d}", 7, 7, body)
            else:
                defs = rad("t", 0.5, 0.5, 0.5, (0, "#ffffff"), (0.35, "#ff9aff"), (1, "#b040d0", 0))
                body = g(ellipse(9.5, 9.5, 8, 5, fill="url(#t)") + circle(12, 9.5, 2.5, fill="#ffffff"), transform=f"rotate({a} 9.5 9.5)")
                put("classic", f"missile/{kind}/{d}", 19, 19, body, defs)
