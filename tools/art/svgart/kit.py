"""Small helpers to write SVG pictures: colours, shapes, gradients and the output registry."""
import hashlib
import math
import random

OUT = {}  # (theme, picture name) -> SVG text


def num(x):
    """A number for SVG: two decimals at most, no trailing zeros."""
    s = f"{x:.2f}".rstrip("0").rstrip(".")
    return "0" if s in ("-0", "") else s


def at(**kw):
    """Attributes: fill_opacity="0.5" -> fill-opacity="0.5"; None values are left out."""
    out = []
    for k, v in kw.items():
        if v is None:
            continue
        k = {"cls": "class", "href": "href"}.get(k, k.replace("_", "-"))
        out.append(f'{k}="{num(v) if isinstance(v, float) else v}"')
    return (" " + " ".join(out)) if out else ""


def put(theme, name, w, h, body, defs="", vb=None):
    """Registers picture `name` of `theme`: `w` x `h` pixels (its size in the game), drawn in
    the `vb` coordinates (the same by default)."""
    vb = vb or f"0 0 {num(w)} {num(h)}"
    d = f"<defs>{defs}</defs>" if defs else ""
    OUT[(theme, name)] = f'<svg xmlns="http://www.w3.org/2000/svg" width="{num(w)}" height="{num(h)}" viewBox="{vb}">{d}{body}</svg>\n'


def rng(*key):
    """A random generator of its own for each picture, so every run draws the same."""
    return random.Random(hashlib.md5("/".join(map(str, key)).encode()).hexdigest())


# ------------------------------------------------------------------ colours

def rgb(c):
    c = c.lstrip("#")
    if len(c) == 3:
        c = "".join(x * 2 for x in c)
    return tuple(int(c[i:i + 2], 16) for i in (0, 2, 4))


def hexc(r, g, b):
    return "#%02x%02x%02x" % tuple(max(0, min(255, round(v))) for v in (r, g, b))


def mix(a, b, t):
    ra, rb = rgb(a), rgb(b)
    return hexc(*(x + (y - x) * t for x, y in zip(ra, rb)))


def light(c, t):
    return mix(c, "#ffffff", t)


def dark(c, t):
    return mix(c, "#000000", t)


# ------------------------------------------------------------------ shapes

def pts(points):
    return " ".join(f"{num(x)},{num(y)}" for x, y in points)


def poly(points, **kw):
    return f'<polygon points="{pts(points)}"{at(**kw)}/>'


def pline(points, **kw):
    return f'<polyline points="{pts(points)}"{at(fill="none", **kw)}/>'


def path(d, **kw):
    return f'<path d="{d}"{at(**kw)}/>'


def circle(cx, cy, r, **kw):
    return f'<circle cx="{num(cx)}" cy="{num(cy)}" r="{num(r)}"{at(**kw)}/>'


def ellipse(cx, cy, rx, ry, **kw):
    return f'<ellipse cx="{num(cx)}" cy="{num(cy)}" rx="{num(rx)}" ry="{num(ry)}"{at(**kw)}/>'


def rect(x, y, w, h, **kw):
    return f'<rect x="{num(x)}" y="{num(y)}" width="{num(w)}" height="{num(h)}"{at(**kw)}/>'


def line(x1, y1, x2, y2, **kw):
    return f'<line x1="{num(x1)}" y1="{num(y1)}" x2="{num(x2)}" y2="{num(y2)}"{at(**kw)}/>'


def text(x, y, s, **kw):
    return f'<text x="{num(x)}" y="{num(y)}"{at(**kw)}>{s}</text>'


def g(body, **kw):
    return f"<g{at(**kw)}>{body}</g>"


def stops(*cs):
    """Gradient stops from (offset, colour[, opacity]) tuples."""
    out = []
    for s in cs:
        o, c = s[0], s[1]
        op = f' stop-opacity="{num(s[2])}"' if len(s) > 2 else ""
        out.append(f'<stop offset="{num(o)}" stop-color="{c}"{op}/>')
    return "".join(out)


def lin(id, x1, y1, x2, y2, *cs):
    return f'<linearGradient id="{id}" x1="{num(x1)}" y1="{num(y1)}" x2="{num(x2)}" y2="{num(y2)}">{stops(*cs)}</linearGradient>'


def rad(id, cx, cy, r, *cs, fx=None, fy=None):
    f = f' fx="{num(fx)}" fy="{num(fy)}"' if fx is not None else ""
    return f'<radialGradient id="{id}" cx="{num(cx)}" cy="{num(cy)}" r="{num(r)}"{f}>{stops(*cs)}</radialGradient>'


def blur(id, sd):
    return f'<filter id="{id}" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="{num(sd)}"/></filter>'


def starfield(r, w, h, n, color="#ffffff", big=0.05, maxr=0.9):
    out = []
    for _ in range(n):
        b = r.random()
        rr = maxr * (1.6 if b < big else 0.35 + 0.65 * r.random())
        out.append(circle(r.random() * w, r.random() * h, rr, fill=color, opacity=0.25 + 0.75 * r.random()))
    return "".join(out)


def blob(cx, cy, rx, ry, r, n=9, wobble=0.25):
    """A closed irregular shape (smooth), for clouds, continents and rocks."""
    p = []
    for i in range(n):
        a = i / n * math.tau
        k = 1 + (r.random() * 2 - 1) * wobble
        p.append((cx + math.cos(a) * rx * k, cy + math.sin(a) * ry * k))
    d = []
    for i in range(n):
        x0, y0 = p[i - 1]
        x1, y1 = p[i]
        mx, my = (x0 + x1) / 2, (y0 + y1) / 2
        if i == 0:
            d.append(f"M{num(mx)},{num(my)}")
        else:
            d.append(f"Q{num(x0)},{num(y0)} {num(mx)},{num(my)}")
    x0, y0 = p[-1]
    x1, y1 = p[0]
    d.append(f"Q{num(x0)},{num(y0)} {num((x0 + x1) / 2)},{num((y0 + y1) / 2)}")
    return "".join(d) + "Z"


def ridge(x0, x1, base, amp, r, steps=12, close_to=None):
    """A mountain or hill skyline from x0 to x1 around height `base`, closed down to `close_to`."""
    xs = [x0 + (x1 - x0) * i / steps for i in range(steps + 1)]
    ys = [base - amp * (0.3 + 0.7 * r.random()) for _ in xs]
    d = f"M{num(x0)},{num(close_to if close_to is not None else base + amp)}"
    for x, y in zip(xs, ys):
        d += f"L{num(x)},{num(y)}"
    d += f"L{num(x1)},{num(close_to if close_to is not None else base + amp)}Z"
    return d


def rot(x, y, a, cx=0, cy=0):
    c, s = math.cos(a), math.sin(a)
    x, y = x - cx, y - cy
    return cx + x * c - y * s, cy + x * s + y * c
