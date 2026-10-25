"""Classic's pictures for MOO1's smaller screens: the planets seen by explorers (COLONIES.LBX),
the colony transport landing and the colonists walking out (LANDING.LBX), the bombs falling,
missile bases and factories blowing up, and the espionage panel (BACKGRND.LBX)."""
import math

from .kit import blob, circle, dark, ellipse, g, light, lin, line, num, path, poly, put, put_anim, rad, rect, rng
from .land import DOME, KINDS, colony, landscape
from .people import RACES, head
from .space import BANNERS


def transport(i, frames):
    """The colony transport seen from the side, 142x103: coming down on its thrusters, its
    legs out, then landed (the last frame)."""
    landed = i == frames - 1
    out = ""
    if not landed:
        k = 0.7 + 0.3 * math.sin(i * 2.1)
        out += ellipse(52, 86, 14, 16 * k, fill="url(#fl)") + ellipse(96, 86, 14, 16 * k, fill="url(#fl)")
    else:
        out += ellipse(74, 100, 60, 4, fill="#000000", opacity=0.35)
    legs = min(1.0, i / max(1, frames - 2))
    for x in (40, 108):
        out += line(x, 66, x + (x - 74) * 0.12, 66 + 30 * legs, stroke="#5a6478", stroke_width=3) + rect(x - 6 + (x - 74) * 0.12, 64 + 30 * legs, 12, 3, fill="#8a96a8")
    out += path("M8,52 Q14,30 46,26 L104,24 Q132,26 138,44 Q140,58 128,66 L22,68 Q8,64 8,52Z", fill="url(#hull)", stroke="#20242e", stroke_width=1.5)
    out += path("M58,24 Q74,6 92,24Z", fill="url(#hull)", stroke="#20242e", stroke_width=1.2) + ellipse(75, 18, 8, 4, fill="#bff0ff", opacity=0.8)
    out += "".join(rect(24 + k * 13, 40, 6, 4, fill="#ff5a4a" if k % 3 == 0 else "#ffd04a") for k in range(8))
    out += line(14, 54, 132, 54, stroke="#3a4252", stroke_width=1)
    return out


TRANSPORT_DEFS = (lin("hull", 0, 0, 0, 1, (0, "#d8dce4"), (0.5, "#8a94a4"), (1, "#3a4252")) +
                  rad("fl", 0.5, 0.2, 0.8, (0, "#ffffff"), (0.45, "#ffd04a"), (1, "#ff6a1a", 0)))


def walkers(color, i):
    """Colonists walking out of the transport with their banner, 62x200 (bottom part)."""
    out = ""
    for k, (x0, y0) in enumerate(((6, 186), (20, 190), (34, 184))):
        x = x0 + i * 2.2 - k * 3
        step = (i + k) % 2
        out += line(x, y0 - 8, x - 3 + step * 6, y0, stroke="#1a1a1a", stroke_width=1.6) + line(x, y0 - 8, x + 3 - step * 6, y0, stroke="#1a1a1a", stroke_width=1.6)
        out += rect(x - 3, y0 - 17, 6, 10, rx=1.5, fill="#e8ecf2", stroke="#1a1a1a", stroke_width=0.6) + circle(x, y0 - 20, 3, fill="#bff0ff", stroke="#1a1a1a", stroke_width=0.6)
        if k == 2:
            out += line(x + 4, y0 - 6, x + 4, y0 - 30, stroke="#3a3a3a", stroke_width=1) + path(f"M{num(x + 4)},{num(y0 - 30)} h10 l-3,4 l3,4 h-10Z", fill=color, stroke="#1a1a1a", stroke_width=0.5)
    return out


def bombard_frame(i, frames):
    """Bombs falling on a colony, 84x102: streaks coming down, flashes, smoke."""
    R = rng("bombard-sky")
    out = rect(0, 0, 84, 102, fill="url(#bsky)") + "".join(circle(R.random() * 84, R.random() * 50, R.uniform(0.3, 0.8), fill="#ffffff", opacity=0.7) for _ in range(20))
    out += path("M0,70 Q20,62 42,66 T84,64 V102 H0Z", fill="#6a3a24") + path("M0,80 Q30,74 60,80 T84,78 V102 H0Z", fill="#4a2414")
    out += colony(42, 74, 0.55, 3, rng("bombard-colony"))
    for k in range(3):
        t = (i - k * 4) / 6
        if 0 <= t < 1:
            x = 20 + k * 22
            y = 4 + t * 66
            out += line(x - 6, y - 16, x, y, stroke="#ffe9a8", stroke_width=2, opacity=0.8) + circle(x, y, 2, fill="#ffffff")
        b = (i - k * 4 - 6) / 6
        if 0 <= b < 1:
            x = 20 + k * 22
            out += circle(x, 72, 4 + 14 * b, fill="url(#boom)", opacity=1 - b * 0.7)
            out += path(blob(x, 60 - 10 * b, 6 + 8 * b, 4 + 5 * b, rng("smoke", k)), fill="#2a2420", opacity=0.6 * (1 - b))
    return out


BOMBARD_DEFS = (lin("bsky", 0, 0, 0, 1, (0, "#05070f"), (1, "#3a1a10")) + DOME +
                rad("boom", 0.5, 0.5, 0.5, (0, "#ffffff"), (0.3, "#ffe36b"), (0.7, "#ff7a1a", 0.9), (1, "#7a1a05", 0)))


def sabotage_frame(kind, i, frames, blow):
    """A hooded spy at night before a missile base or factories that blow up, 82x33."""
    R = rng("sab", kind)
    out = rect(0, 0, 82, 33, fill="url(#night)") + "".join(circle(R.random() * 82, R.random() * 16, 0.4, fill="#ffffff", opacity=0.7) for _ in range(12))
    out += path("M0,26 Q30,22 82,25 V33 H0Z", fill="#4a2a5a")
    out += path("M2,33 L6,14 Q10,8 14,14 L18,33Z", fill="#1a2a5a") + ellipse(10, 14, 3.5, 4, fill="#0a1020")
    if kind == "bases":
        out += rect(50, 16, 14, 10, fill="#5a6478", stroke="#1a1f28", stroke_width=0.5) + poly([(55, 16), (57, 4), (59, 16)], fill="#c8d0dc", stroke="#1a1f28", stroke_width=0.5)
    else:
        out += path("M40,26 V18 L48,22 V18 L56,22 V18 L64,22 V26Z", fill="#6a7480", stroke="#1a1f28", stroke_width=0.5) + rect(66, 12, 3, 14, fill="#5a6478")
    b = (i - blow) / max(1, frames - blow)
    if b >= 0:
        out += circle(57, 18, 3 + 13 * b, fill="url(#sboom)", opacity=1 - b * 0.6)
        out += "".join(line(57, 18, 57 + math.cos(a) * (6 + 14 * b), 18 + math.sin(a) * (4 + 9 * b), stroke="#ffd04a", stroke_width=0.8, opacity=1 - b) for a in [k * math.tau / 7 for k in range(7)])
    return out


SAB_DEFS = (lin("night", 0, 0, 0, 1, (0, "#05050f"), (1, "#2a1a40")) +
            rad("sboom", 0.5, 0.5, 0.5, (0, "#ffffff"), (0.35, "#ffe36b"), (0.75, "#ff6a1a", 0.9), (1, "#7a1a05", 0)))


def espionage_panel():
    """The espionage panel, 216x139: a window with two hooded agents, six field slots."""
    defs = lin("metal", 0, 0, 0, 1, (0, "#8a94a4"), (1, "#4a5262")) + lin("win", 0, 0, 0, 1, (0, "#1a1a40"), (1, "#0a0a20"))
    body = rect(0, 0, 216, 139, rx=4, fill="url(#metal)", stroke="#1a1f28", stroke_width=1.5)
    body += rect(10, 10, 196, 44, fill="url(#win)", stroke="#c8d0dc", stroke_width=1)
    R = rng("esp")
    body += "".join(circle(12 + R.random() * 192, 12 + R.random() * 40, R.uniform(0.3, 0.8), fill="#ffffff", opacity=0.6) for _ in range(30))
    for x, flip in ((30, 1), (186, -1)):
        body += path(f"M{num(x - 14 * flip)},54 L{num(x - 8 * flip)},28 Q{num(x)},16 {num(x + 8 * flip)},28 L{num(x + 14 * flip)},54Z", fill="#14141c")
        body += ellipse(x + 2 * flip, 30, 6, 7, fill="#0a0a10") + circle(x + 3 * flip, 30, 0.9, fill="#ff5a4a")
    for k in range(6):
        x = (k // 3) * 102 + 12
        y = (k % 3) * 22 + 70
        body += rect(x, y, 90, 16, fill="#2a2f38", stroke="#1a1f28") + rect(x + 2, y + 2, 86, 12, fill="#5a3a30")
        body += "".join(line(x + 4 + j * 8, y + 13, x + 10 + j * 8, y + 3, stroke="#7a4a3a", stroke_width=1.5) for j in range(10))
    return body, defs


def build():
    # Exploration: each kind of land, without and with a colony (84x102).
    for t, kind in enumerate(KINDS):
        defs, body = landscape(kind, 84, 102, 30, 0.42)
        put("classic", f"explore/{t}", 84, 102, body, defs)
        put("classic", f"explore/{t}/colony", 84, 102, body + colony(42, 74, 0.6, 3, rng("explore-col", kind)), defs + DOME)

    # The landing: the transport (142x103, 6 frames, the last landed) and the colonists of
    # each banner (62x200, 8 frames).
    n = 6
    put("classic", "landing/transport", 142, 103, transport(0, n), TRANSPORT_DEFS)
    put_anim("classic", "landing/transport", 142, 103, n, 2, lambda i: transport(i, n), TRANSPORT_DEFS)
    for b, color in enumerate(BANNERS):
        put("classic", f"landing/walk/{b}", 62, 200, walkers(color, 0))
        put_anim("classic", f"landing/walk/{b}", 62, 200, 8, 4, lambda i, c=color: walkers(c, i))

    # Bombs falling (84x102, 16 frames: MOO1 sounds on frames 3, 6 and 15).
    put("classic", "bombard", 84, 102, bombard_frame(0, 16), BOMBARD_DEFS)
    put_anim("classic", "bombard", 84, 102, 16, 0, lambda i: bombard_frame(i, 16), BOMBARD_DEFS)

    # Sabotage (82x33): missile bases (10 frames, blowing up from 5), factories (16, from 11).
    for kind, frames, blow in (("bases", 10, 5), ("factories", 16, 11)):
        put("classic", f"sabotage/{kind}", 82, 33, sabotage_frame(kind, 0, frames, blow), SAB_DEFS)
        put_anim("classic", f"sabotage/{kind}", 82, 33, frames, frames - 2, lambda i, k=kind, f=frames, bl=blow: sabotage_frame(k, i, f, bl), SAB_DEFS)

    body, defs = espionage_panel()
    put("classic", "spy/espionage", 216, 139, body, defs)
