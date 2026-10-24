"""Interface icons (`ui/*`, `icon/*`) of both themes, Classic's coats of arms (`arms/<race>`)
and the Age theme's map markers that used to be drawn by the game (camps, the Guardian's
standard, the free companies' banners)."""
from .kit import circle, ellipse, g, lin, line, path, poly, put, rad, rect

# The icons of the city window's rows (one per 5 people, 10 factories...), 24x24.
ICONS = {
    "age": {
        "pop": '<circle cx="12" cy="6" r="3.5" fill="#e3c9a0" stroke="#3b2a16"/><path d="M6 22l2-11h8l2 11z" fill="#8a5a2b" stroke="#3b2a16"/>',
        "factory": '<path d="M3 22V12l9-6 9 6v10z" fill="#c9a36b" stroke="#3b2a16"/><rect x="15" y="3" width="3" height="7" fill="#7a6a5a" stroke="#3b2a16"/><rect x="10" y="15" width="4" height="7" fill="#5a3b1a"/>',
        "waste": '<path d="M3 22c2-6 5-8 9-8s7 2 9 8z" fill="#6b5a3a" stroke="#2b2219"/><path d="M10 12c-2-3 1-5 0-8M14 12c2-3-1-5 1-8" stroke="#555" stroke-width="2" fill="none"/>',
        "base": '<path d="M7 22V8h10v14z" fill="#b9ad97" stroke="#3d342a"/><path d="M6 8V4h2v2h2V4h4v2h2V4h2v4z" fill="#b9ad97" stroke="#3d342a"/><rect x="11" y="15" width="3" height="7" fill="#3d342a"/>',
        "shield": '<path d="M2 22V10h3V7h3v3h3V7h3v3h3V7h3v3h2v12z" fill="#a59a88" stroke="#3d342a"/>',
        "gold": '<circle cx="12" cy="12" r="9" fill="#f2c14e" stroke="#7a5212" stroke-width="1.5"/><circle cx="12" cy="12" r="5.5" fill="none" stroke="#b8862b"/><path d="M12 8.5v7M10 10h4" stroke="#7a5212" stroke-width="1.4"/>',
        "research": '<path d="M3 5c3-1 6-1 9 1v14c-3-2-6-2-9-1z" fill="#efe0bb" stroke="#4a3220"/><path d="M21 5c-3-1-6-1-9 1v14c3-2 6-2 9-1z" fill="#e3d0a3" stroke="#4a3220"/><path d="M5 8h5M5 11h5M14 8h5M14 11h5" stroke="#8a6a44"/>',
        "reserve": '<path d="M3 11h18v10H3z" fill="#8a5a2b" stroke="#3b2414"/><path d="M3 11c0-4 3-6 9-6s9 2 9 6z" fill="#a8713a" stroke="#3b2414"/><path d="M3 11h18M3 16h18" stroke="#d9a440" stroke-width="1.5"/><rect x="10.5" y="12" width="3" height="4" fill="#f2c14e" stroke="#3b2414" stroke-width="0.8"/>',
    },
    "classic": {
        "pop": '<circle cx="12" cy="7" r="4.5" fill="#9fc4ff" stroke="#123"/><rect x="9" y="5.5" width="6" height="3" rx="1" fill="#123"/><path d="M6 22l1-9h10l1 9z" fill="#3a66c4" stroke="#123"/>',
        "factory": '<path d="M2 22V12l6 3v-3l6 3v-3l8 4v6z" fill="#8090a0" stroke="#123"/><rect x="17" y="4" width="3" height="10" fill="#6a7888" stroke="#123"/>',
        "waste": '<rect x="6" y="6" width="12" height="16" rx="2" fill="#4caf50" stroke="#123"/><path d="M6 11h12M6 17h12" stroke="#123"/><circle cx="12" cy="14" r="2" fill="#d8ff6a"/>',
        "base": '<path d="M4 22l3-6h10l3 6z" fill="#7a8796" stroke="#123"/><path d="M11 16V6l1-4 1 4v10z" fill="#e0e6ee" stroke="#123"/>',
        "shield": '<path d="M2 21a10 10 0 0 1 20 0z" fill="rgba(90,180,255,0.45)" stroke="#5ab4ff" stroke-width="1.5"/>',
        "gold": '<circle cx="12" cy="12" r="9" fill="#e9c349" stroke="#6b5310" stroke-width="1.5"/><text x="12" y="15.5" font-size="9" font-family="sans-serif" font-weight="bold" text-anchor="middle" fill="#6b5310">BC</text>',
        "research": '<path d="M3 5c3-1 6-1 9 1v14c-3-2-6-2-9-1z" fill="#cfe3ff" stroke="#1d3350"/><path d="M21 5c-3-1-6-1-9 1v14c3-2 6-2 9-1z" fill="#b5d2f5" stroke="#1d3350"/><path d="M5 8h5M5 11h5M14 8h5M14 11h5" stroke="#4d6d96"/>',
        "reserve": '<rect x="3" y="4" width="18" height="17" rx="2" fill="#7d8b99" stroke="#1c2630"/><rect x="5.5" y="6.5" width="13" height="12" rx="1" fill="#5d6a77" stroke="#1c2630"/><circle cx="12" cy="12.5" r="3" fill="#c7d0d8" stroke="#1c2630"/><path d="M12 9.5v6M9 12.5h6" stroke="#1c2630"/>',
    },
}

# Classic coats of arms: an emblem per race on a star-field shield (MOO1 order).
ARMS = [
    ("#2f5fd0", "human"),
    ("#d08a1f", "mrrshan"),
    ("#7a6f9a", "silicoid"),
    ("#3f8a3a", "sakkra"),
    ("#4a7ab8", "psilon"),
    ("#b07a2a", "alkari"),
    ("#a3372a", "klackon"),
    ("#6b4a32", "bulrathi"),
    ("#6f7b88", "meklar"),
    ("#3c3550", "darlok"),
]


def emblem(race):
    """A white emblem centred on (50, 52) in a 100x110 box."""
    w = dict(fill="#f4f1e8", stroke="#1a1a1a", stroke_width=2)
    if race == "human":
        return circle(50, 52, 18, fill="none", stroke="#f4f1e8", stroke_width=5) + path("M50 22 L54 46 L78 52 L54 58 L50 82 L46 58 L22 52 L46 46Z", **w)
    if race == "mrrshan":
        return path("M30 70 L34 34 L44 46 L56 46 L66 34 L70 70 Q50 84 30 70Z", **w) + path("M40 58 L46 60 M60 58 L54 60 M50 64 L50 70", stroke="#1a1a1a", stroke_width=3, fill="none")
    if race == "silicoid":
        return poly([(50, 24), (66, 44), (60, 80), (40, 80), (34, 44)], **w) + path("M50 24 L50 80 M34 44 L66 44", stroke="#1a1a1a", stroke_width=2)
    if race == "sakkra":
        return path("M28 60 Q36 34 60 38 L78 44 L62 50 L72 56 L56 58 Q46 72 28 60Z", **w) + circle(60, 44, 2.5, fill="#1a1a1a")
    if race == "psilon":
        return ellipse(50, 44, 20, 22, **w) + path("M36 58 Q50 84 64 58", **w) + path("M40 40 Q50 32 60 40", fill="none", stroke="#1a1a1a", stroke_width=2)
    if race == "alkari":
        return path("M50 34 L58 46 L82 40 L64 58 L56 56 L50 80 L44 56 L36 58 L18 40 L42 46Z", **w)
    if race == "klackon":
        return ellipse(50, 58, 14, 20, **w) + circle(50, 34, 9, **w) + path("M44 28 L34 16 M56 28 L66 16 M36 52 L22 46 M36 62 L20 64 M64 52 L78 46 M64 62 L80 64", stroke="#f4f1e8", stroke_width=3)
    if race == "bulrathi":
        return circle(50, 54, 20, **w) + circle(34, 36, 7, **w) + circle(66, 36, 7, **w) + ellipse(50, 62, 8, 6, fill="#1a1a1a")
    if race == "meklar":
        return circle(50, 52, 20, **w) + "".join(rect(48, 26, 4, 10, fill="#f4f1e8", stroke="#1a1a1a", transform=f"rotate({a} 50 52)") for a in range(0, 360, 45)) + circle(50, 52, 7, fill="#d23b2b", stroke="#1a1a1a", stroke_width=2)
    # darlok: a mask
    return path("M28 40 Q50 30 72 40 Q70 70 50 80 Q30 70 28 40Z", **w) + ellipse(41, 50, 6, 3, fill="#6be3ff") + ellipse(59, 50, 6, 3, fill="#6be3ff")


def build():
    for theme, icons in ICONS.items():
        for name, body in icons.items():
            folder = "ui" if name in ("gold", "research", "reserve") else "icon"
            put(theme, f"{folder}/{name}", 24, 24, body)

    shield = "M8 6 H92 V54 Q92 92 50 106 Q8 92 8 54Z"
    for i, (color, race) in enumerate(ARMS):
        defs = lin("f", 0, 0, 0, 1, (0, color), (1, "#0b0e18")) + rad("s", 0.3, 0.2, 0.9, (0, "#ffffff", 0.35), (1, "#ffffff", 0))
        body = path(shield, fill="url(#f)", stroke="#d8c48a", stroke_width=4)
        body += path(shield, fill="url(#s)")
        body += g(emblem(race))
        put("classic", f"arms/{i}", 100, 110, body, defs)

    # Age: what the world map drew by itself. Camps of tents on free land, a bare peak where
    # nobody can live (64x64, drawn at the site's size).
    tents = ""
    for dx in (-13, 13):
        tents += poly([(32 + dx - 11, 44), (32 + dx, 22), (32 + dx + 11, 44)], fill="#f2e6c8", stroke="#5b4630", stroke_width=2, stroke_linejoin="round")
        tents += line(32 + dx, 22, 32 + dx, 44, stroke="#5b4630", stroke_width=1.5)
    peak = poly([(6, 50), (32, 10), (58, 50)], fill="#6e665a") + poly([(24, 23), (32, 10), (40, 23), (35, 21), (32, 25), (28, 21)], fill="#efeee6")
    from .land import KINDS
    for kind in KINDS:
        put("age", f"site/{kind}", 64, 64, peak if kind == "not_habitable" else tents)

    # The Immortals' standard: a golden eagle with spread wings on a lance.
    eagle = line(32, 22, 32, 62, stroke="#3b2a16", stroke_width=3)
    eagle += poly([(4, 26), (24, 28), (32, 14), (40, 28), (60, 26), (42, 38), (22, 38)], fill="#e0b43a", stroke="#5a3d0c", stroke_width=1.5, stroke_linejoin="round")
    put("age", "guardian", 64, 64, eagle)

    # The free companies' banners: the Catalan Company (Aragon's bars), the Great Company.
    def banner(inner):
        pole = line(14, 6, 14, 62, stroke="#2b2219", stroke_width=3)
        flag = "M14 6 H54 L46 20 L54 34 H14Z"
        return pole + f'<clipPath id="c"><path d="{flag}"/></clipPath>' + g(inner, clip_path="url(#c)") + path(flag, fill="none", stroke="#2b2219", stroke_width=1.5)
    bars = rect(14, 6, 40, 28, fill="#e3b528") + "".join(rect(14, 6 + 28 / 9 * (2 * k + 1), 40, 28 / 9, fill="#b8231c") for k in range(4))
    checks = rect(14, 6, 40, 28, fill="#1c1c1c") + rect(14, 6, 20, 14, fill="#eeeeee") + rect(34, 20, 20, 14, fill="#eeeeee")
    put("age", "monster/crystal", 64, 64, banner(bars))
    put("age", "monster/amoeba", 64, 64, banner(checks))
