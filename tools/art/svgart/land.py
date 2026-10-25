"""Classic's planet surfaces: the colonization and invasion scenes (MOO1's LANDING.LBX), the
views of places and the battlefields of a world map, and the planets as map markers."""
import math

from .kit import blob, blur, circle, dark, ellipse, g, light, lin, mix, num, path, poly, put, rad, rect, ridge, rng, starfield
from .space import globe

KINDS = ["not_habitable", "radiated", "toxic", "inferno", "dead", "tundra", "barren", "minimal", "desert", "steppe", "arid", "ocean", "jungle", "terran"]

# Sky top, sky at the horizon, far hills, ground, ground detail.
LAND = {
    "not_habitable": ("#05040a", "#2a1f3a", "#4a3a4a", "#5a4a50", "#3a2f38"),
    "radiated": ("#0a1405", "#6a7a1a", "#3a4a14", "#5a6a24", "#c8e050"),
    "toxic": ("#1a2a0a", "#8aa03a", "#4a6a24", "#5a7a2a", "#b8d040"),
    "inferno": ("#1a0505", "#b0401a", "#4a1a10", "#5a2414", "#ff8a2a"),
    "dead": ("#020204", "#1a1a24", "#4a4a4a", "#6a6a6a", "#3a3a3a"),
    "tundra": ("#3a5a8a", "#c8dcf0", "#8aa0b8", "#e8f0f8", "#a8b8c8"),
    "barren": ("#0a0a14", "#3a3040", "#6a5040", "#7a6250", "#4a3a2e"),
    "minimal": ("#1a2440", "#a8a0b8", "#8a7a5a", "#b09a70", "#7a6a4a"),
    "desert": ("#3a5aa0", "#f0d0a0", "#c0844a", "#e0a860", "#b07838"),
    "steppe": ("#3a6ab0", "#d8e0b0", "#7a8a4a", "#a8a058", "#6a7a30"),
    "arid": ("#4a5a90", "#f0c8a0", "#9a6a44", "#c49060", "#8a5a34"),
    "ocean": ("#2a5ab8", "#b8e0ff", "#3a7a5a", "#2f6ad0", "#e0f4ff"),
    "jungle": ("#2a6a8a", "#c8f0d0", "#1f5a2a", "#2f7a34", "#14401a"),
    "terran": ("#2a6ad0", "#c8e8ff", "#4a8a4a", "#5aa04a", "#3a7030"),
}


def landscape(kind, w, h, seed, horizon=0.45):
    """A planet's surface seen from the ground: (defs, body)."""
    sky0, sky1, far, ground, detail = LAND[kind]
    R = rng("land", kind, seed)
    hy = h * horizon
    defs = lin("sky", 0, 0, 0, 1, (0, sky0), (1, sky1)) + lin("gr", 0, 0, 0, 1, (0, light(ground, 0.1)), (1, dark(ground, 0.45))) + blur("soft", w / 200)
    body = rect(0, 0, w, h, fill="url(#sky)")
    if kind in ("dead", "barren", "not_habitable", "radiated", "minimal"):
        body += starfield(R, w, hy, int(w * hy / 900), "#ffffff", 0.05, w / 400)
    # A sun or a ringed giant in the sky, and a moon.
    sx, sy = R.uniform(0.1, 0.9) * w, R.uniform(0.08, 0.3) * h
    if kind == "not_habitable":
        d, b = globe("not_habitable", sx, sy + h * 0.05, h * 0.18, seed, "gs")
        defs += d
        body += b
    else:
        sun = {"inferno": "#ffcf6a", "radiated": "#f4ff9a", "toxic": "#f0ff9a"}.get(kind, "#fff8e0")
        defs += rad("sun", 0.5, 0.5, 0.5, (0, "#ffffff"), (0.25, sun), (1, sun, 0))
        body += circle(sx, sy, h * 0.12, fill="url(#sun)")
        if R.random() < 0.6:
            mx, my, mr = R.uniform(0.1, 0.9) * w, R.uniform(0.1, 0.25) * h, h * R.uniform(0.025, 0.05)
            body += circle(mx, my, mr, fill=light(far, 0.5), opacity=0.8) + circle(mx + mr * 0.35, my - mr * 0.2, mr * 0.9, fill=sky1, opacity=0.5)
    if kind in ("terran", "ocean", "jungle", "steppe", "tundra", "toxic"):
        for _ in range(4):
            cx, cy = R.uniform(0, w), R.uniform(0.08, 0.35) * h
            body += path(blob(cx, cy, w * R.uniform(0.06, 0.14), h * R.uniform(0.02, 0.04), R), fill="#ffffff" if kind != "toxic" else "#d8e890", opacity=0.55, filter="url(#soft)")
    # Far mountains, then nearer hills.
    body += path(ridge(0, w, hy, h * 0.12, R, 18, hy + h * 0.05), fill=mix(far, sky1, 0.45))
    body += path(ridge(0, w, hy + h * 0.04, h * 0.08, R, 12, hy + h * 0.12), fill=far)
    if kind == "ocean":
        body += rect(0, hy + h * 0.06, w, h, fill="url(#gr)")
        for _ in range(int(w / 25)):
            y = R.uniform(hy + h * 0.08, h)
            x = R.uniform(0, w)
            body += path(f"M{num(x)},{num(y)} q{num(w * 0.01)},{num(-h * 0.008)} {num(w * 0.02)},0", stroke=detail, stroke_width=w / 500, fill="none", opacity=0.6)
        body += path(blob(w * 0.25, h * 0.92, w * 0.3, h * 0.12, R), fill="#c8b070") + path(blob(w * 0.25, h * 0.95, w * 0.26, h * 0.1, R), fill="#4a9a3a")
        return defs, body
    body += path(ridge(0, w, hy + h * 0.1, h * 0.04, R, 8, h), fill="url(#gr)")
    # Ground details per kind.
    for _ in range(int(w / 18)):
        x, y = R.uniform(0, w), R.uniform(hy + h * 0.12, h)
        k = (y - hy) / (h - hy)
        if kind in ("dead", "barren", "minimal", "not_habitable"):
            body += ellipse(x, y, w * 0.02 * k + 1, h * 0.006 * k + 0.5, fill=detail, opacity=0.7)
        elif kind == "inferno":
            body += path(f"M{num(x)},{num(y)} l{num(w * 0.02 * k)},{num(h * 0.004)} l{num(w * 0.015 * k)},{num(-h * 0.006)}", stroke=detail, stroke_width=w / 400 * k + 0.4, fill="none", opacity=0.9)
        elif kind in ("radiated", "toxic"):
            body += ellipse(x, y, w * 0.025 * k + 1, h * 0.007 * k + 0.5, fill=detail, opacity=0.6)
        elif kind == "tundra":
            body += ellipse(x, y, w * 0.03 * k + 1, h * 0.005 * k + 0.5, fill="#ffffff", opacity=0.8)
        elif kind in ("jungle", "terran", "steppe"):
            th = h * 0.05 * k + 1
            body += path(f"M{num(x)},{num(y)} v{num(-th)}", stroke=dark(detail, 0.3), stroke_width=w / 600 * k + 0.3) + circle(x, y - th, th * 0.6, fill=detail, opacity=0.95)
        else:
            body += path(f"M{num(x - w * 0.02 * k)},{num(y)} q{num(w * 0.02 * k)},{num(-h * 0.01 * k)} {num(w * 0.04 * k)},0", fill=detail, opacity=0.5)
    return defs, body


def colony(cx, cy, s, n, R):
    """Domes of a colony on the ground around (cx, cy), `s` a scale."""
    out = ellipse(cx, cy + 2 * s, 30 * s * (1 + n * 0.3), 5 * s, fill="#2a2f3a", opacity=0.6)
    spots = [(0, 0, 12)] + [(-20, 2, 8), (20, 2, 8), (-34, 4, 6), (34, 4, 6), (-8, -10, 7)][: n]
    for dx, dy, r in sorted(spots, key=lambda p: p[1]):
        x, y, r = cx + dx * s, cy + dy * s, r * s
        out += path(f"M{num(x - r)},{num(y)} A{num(r)},{num(r)} 0 0 1 {num(x + r)},{num(y)}Z", fill="url(#dome)", stroke="#1c2630", stroke_width=0.6 * s)
        if R.random() < 0.5:
            out += path(f"M{num(x)},{num(y - r)} v{num(-5 * s)}", stroke="#c0c8d8", stroke_width=0.8 * s) + circle(x, y - r - 5 * s, 0.9 * s, fill="#ff5a4a")
    return out


DOME = lin("dome", 0, 0, 0, 1, (0, "#e8f4ff", 0.95), (1, "#6a8ab8", 0.85))
ORION = lin("orion", 0, 0, 0, 1, (0, "#fff4c0"), (1, "#a08a40"))


def build():
    for t, kind in enumerate(KINDS):
        # MOO1's colonization scene: the land where the transport comes down (the landing
        # screen draws the transport, `landing/transport`).
        defs, body = landscape(kind, 320, 200, 1)
        put("classic", f"landing/colonize/{t}", 320, 200, body, defs)
        # The invasion: the defenders' colony in the distance, room for the troops in front.
        defs, body = landscape(kind, 320, 200, 2, 0.38)
        body += colony(160, 88, 0.9, 3, rng("inv", kind))
        put("classic", f"landing/invade/{t}", 320, 200, body, defs + DOME)
        # A world map's view of the place (wide), and its marker on the map (a small planet).
        defs, body = landscape(kind, 768, 384, 3)
        put("classic", f"view/{kind}", 768, 384, body, defs)
        d, b = globe(kind, 32, 32, 24, t + 100)
        put("classic", f"site/{kind}", 64, 64, b, d)
    for level in range(3):
        defs, body = landscape("terran", 768, 384, 10 + level)
        body += colony(384, 250, 3 + level, 1 + level * 2, rng("city", level))
        put("classic", f"view/city/{level}", 768, 384, body, defs + DOME)
    defs, body = landscape("barren", 768, 384, 20)
    R = rng("orion")
    for k in range(7):
        x = 160 + k * 75
        hgt = R.uniform(60, 140)
        body += poly([(x - 18, 260), (x - 10, 260 - hgt), (x + 10, 260 - hgt), (x + 18, 260)], fill="url(#orion)", stroke="#5a4a20", stroke_width=2)
    body += poly([(300, 260), (384, 120), (468, 260)], fill="url(#orion)", stroke="#5a4a20", stroke_width=2) + circle(384, 175, 12, fill="#9fe8ff", opacity=0.9)
    put("classic", "view/orion", 768, 384, body, defs + ORION)

    # A world map's battlefields seen from above: alien ground of each kind.
    fields = {"green": "terran", "dry": "desert", "forest": "jungle", "marsh": "toxic", "rocky": "barren", "snow": "tundra", "ash": "inferno", "mists": "minimal"}
    for name, kind in fields.items():
        _, _, far, ground, detail = LAND[kind]
        R = rng("field", name)
        body = rect(0, 0, 800, 480, fill=ground)
        for _ in range(60):
            body += path(blob(R.uniform(0, 800), R.uniform(0, 480), R.uniform(20, 70), R.uniform(15, 50), R), fill=R.choice([far, detail, dark(ground, 0.15), light(ground, 0.12)]), opacity=0.45)
        for _ in range(40):
            x, y, r = R.uniform(0, 800), R.uniform(0, 480), R.uniform(3, 12)
            body += circle(x, y, r, fill=dark(ground, 0.3), opacity=0.5) + circle(x - r * 0.3, y - r * 0.3, r * 0.6, fill=light(ground, 0.2), opacity=0.4)
        if name == "mists":
            body += rect(0, 0, 800, 480, fill="#e8eef4", opacity=0.3)
        put("classic", f"battle/field/{name}", 800, 480, g(body, filter="url(#s)"), blur("s", 1.2))
