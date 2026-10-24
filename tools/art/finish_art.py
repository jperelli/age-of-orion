"""Turns the chosen pictures of gen_art.py into Age theme pictures.

    [ONLY=prefix,...] python tools/art/finish_art.py SRC_DIR...

The SRC_DIRs hold `<name>-s<seed>.png` (gen_gemini.py numbers its tries 1, 2...; gen_art.py
uses its seeds); when several exist, PICK names the one to use, else the lowest wins. Landscapes are resized; sprites (units, rocks, markers, arms) are
cut out of their white background and cropped. Writes themes/age/images/<name>.webp (the
file's path is the picture's name).
"""

import os
import re
import sys

import numpy as np
from PIL import Image, ImageFilter

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")
THEME = os.path.join(ROOT, "themes/age")
RACES = ["human", "mrrshan", "silicoid", "sakkra", "psilon", "alkari", "klackon", "bulrathi", "meklar", "darlok"]
PLANET_TYPES = ["not_habitable", "radiated", "toxic", "inferno", "dead", "tundra", "barren", "minimal", "desert", "steppe", "arid", "ocean", "jungle", "terran"]
# 1oom GAME_NEWS_* numbers (web/src/news.ts).
NEWS_TYPES = ["", "plague", "quake", "nova", "accident", "assassin", "virus", "comet", "pirates", "derelict", "rebellion", "crystal", "amoeba", "enviro", "rich", "support", "poor", "orion", "coup", "stars", "stats", "genocide", "guardian"]
# Seed chosen for a picture when several were made (default: the lowest seed).
PICK: dict[str, int] = {"anim-01-attack": 2, "anim-06-march": 2, "anim-16-attack": 2, "anim-23-march": 3}


def alias_of(name: str) -> tuple[str, str]:
    """(picture name, kind) of a generated picture."""
    kind, _, key = name.partition("-")
    if kind == "view":
        if key.startswith("city"):
            return f"view/city/{key[4:]}", "view"
        return f"view/{key}", "view"
    if kind == "field":
        return f"battle/field/{key}", "field"
    if kind == "unit":
        return f"unit/{int(key) if key.isdigit() else key}", "unit"
    if kind == "rock":
        return f"battle/rock/{key}", "rock"
    if kind == "mark":
        alias = {"army": "army", "caravan": "caravan", "canal": "stargate"}.get(key, f"mark/{key}")
        return alias, "mark"
    if kind == "arms":
        return f"arms/{RACES.index(key)}", "arms"
    if kind == "siege":
        return f"landing/invade/{PLANET_TYPES.index(key)}", "siege"
    if kind == "soldier":
        return f"ground/soldier/{RACES.index(key)}", "soldier"
    if name == "fallen":
        return "ground/death", "soldier"
    if name == "herald":
        return "news/herald", "soldier"
    if kind == "news":
        return f"news/icon/{NEWS_TYPES.index(key)}", "vignette"
    if kind == "hall":
        return f"hall/{RACES.index(key)}", "hall"
    if name == "council":
        return "council/hall", "hall"
    raise ValueError(name)


def holes(mask: np.ndarray, min_area: int) -> np.ndarray:
    """The connected regions of `mask` of at least `min_area` pixels (background seen through
    gaps, such as between legs), labelled by spreading the smallest index through each region."""
    h, w = mask.shape
    big = h * w
    lab = np.where(mask, np.arange(big).reshape(h, w), big)
    while True:
        new = lab.copy()
        new[1:, :] = np.minimum(new[1:, :], lab[:-1, :])
        new[:-1, :] = np.minimum(new[:-1, :], lab[1:, :])
        new[:, 1:] = np.minimum(new[:, 1:], lab[:, :-1])
        new[:, :-1] = np.minimum(new[:, :-1], lab[:, 1:])
        new = np.where(mask, new, big)
        if (new == lab).all():
            break
        lab = new
    sizes = np.bincount(lab[mask], minlength=big + 1)
    return mask & (sizes[lab] >= min_area)


def cut_out(im: Image.Image, gaps: bool = True, crop: bool = True) -> Image.Image:
    """Removes the plain background around a sprite: grey pixels reached from the borders
    through smooth steps (so gradients and vignettes go too), and big grey holes seen through
    gaps (not for coats of arms: their white fields are inside an outline). The bottom strip is
    whitened first: the model sometimes signs there."""
    a = np.asarray(im.convert("RGB")).astype(np.int16).copy()
    a[-round(a.shape[0] * 0.06) :, :, :] = 255
    grey = (a.max(axis=2) - a.min(axis=2) < 30) & (a.min(axis=2) > 120)
    bg = np.zeros_like(grey)
    bg[0, :], bg[-1, :], bg[:, 0], bg[:, -1] = grey[0, :], grey[-1, :], grey[:, 0], grey[:, -1]

    def close(d1: np.ndarray, d2: np.ndarray) -> np.ndarray:
        return np.abs(d1 - d2).sum(axis=-1) < 18

    while True:
        grown = bg.copy()
        grown[1:, :] |= bg[:-1, :] & close(a[1:, :], a[:-1, :])
        grown[:-1, :] |= bg[1:, :] & close(a[:-1, :], a[1:, :])
        grown[:, 1:] |= bg[:, :-1] & close(a[:, 1:], a[:, :-1])
        grown[:, :-1] |= bg[:, 1:] & close(a[:, :-1], a[:, 1:])
        grown &= grey
        if (grown == bg).all():
            break
        bg = grown
    light = (a.min(axis=2) > 215) & (a.max(axis=2) - a.min(axis=2) < 28)
    if gaps:
        bg |= holes(light & ~bg, light.size // 400)
    alpha = Image.fromarray(np.where(bg, 0, 255).astype(np.uint8)).filter(ImageFilter.MinFilter(3)).filter(ImageFilter.GaussianBlur(0.8))
    out = im.convert("RGBA")
    out.putalpha(alpha)
    box = alpha.point(lambda v: 255 if v > 24 else 0).getbbox()
    return out.crop(box) if box and crop else out


def sheet_frames(im: Image.Image) -> list[Image.Image]:
    """The four frames of a 2x2 animation sheet, cut out and cropped to the box that holds
    all of them, so the troop keeps its place from frame to frame."""
    im = im.convert("RGB")
    w, h = im.width // 2, im.height // 2
    m = round(min(w, h) * 0.03)  # the model sometimes draws grid lines between the cells
    cells = [im.crop((x * w + m, y * h + m, (x + 1) * w - m, (y + 1) * h - m)) for y in (0, 1) for x in (0, 1)]
    for c in cells:
        c.thumbnail((480, 480), Image.LANCZOS)
    frames = [cut_out(c, crop=False) for c in cells]
    boxes = [f.getchannel("A").point(lambda v: 255 if v > 24 else 0).getbbox() for f in frames]
    boxes = [b for b in boxes if b]
    box = (min(b[0] for b in boxes), min(b[1] for b in boxes), max(b[2] for b in boxes), max(b[3] for b in boxes))
    return [f.crop(box) for f in frames]


def trim(im: Image.Image, k: float = 0.05) -> Image.Image:
    """Cuts a thin strip off every side: the model sometimes paints a frame or signs there."""
    dx, dy = round(im.width * k), round(im.height * k)
    return im.crop((dx, dy, im.width - dx, im.height - dy))


def cover(im: Image.Image, w: int, h: int) -> Image.Image:
    """Crops the middle of `im` to the w:h shape, then resizes it to w x h."""
    k = max(w / im.width, h / im.height)
    cw, ch = round(w / k), round(h / k)
    x, y = (im.width - cw) // 2, (im.height - ch) // 2
    return im.crop((x, y, x + cw, y + ch)).resize((w, h), Image.LANCZOS)


def fit(im: Image.Image, w: int, h: int) -> Image.Image:
    k = min(w / im.width, h / im.height, 1)
    return im.resize((max(1, round(im.width * k)), max(1, round(im.height * k))), Image.LANCZOS)


def main(srcs: list[str]) -> None:
    found: dict[str, dict[int, str]] = {}
    for src in srcs:
        for name in os.listdir(src):
            m = re.match(r"(.+)-s(\d+)\.png$", name)
            if m:
                found.setdefault(m[1], {})[int(m[2])] = os.path.join(src, name)
    done = 0
    only = [o for o in os.environ.get("ONLY", "").split(",") if o]  # name prefixes, to redo a few
    for name in sorted(found):
        if only and not any(name.startswith(o) for o in only):
            continue
        seeds = found[name]
        im = Image.open(seeds[PICK.get(name, min(seeds))])
        if name.startswith("anim-"):
            # anim-<troop>-<march|attack>: frames unit/<look>/<kind>/<n> of the troop's picture
            _, key, anim = name.split("-")
            unit = f"unit/{int(key) if key.isdigit() else key}"
            for n, frame in enumerate(sheet_frames(im)):
                path = os.path.join(THEME, f"images/{unit}/{anim}/{n}.webp")
                os.makedirs(os.path.dirname(path), exist_ok=True)
                fit(frame, 192, 144).save(path, "WEBP", quality=80, method=6)
                done += 1
            continue
        alias, kind = alias_of(name)
        if kind not in ("view", "field", "siege", "vignette", "hall"):
            im.thumbnail((640, 640), Image.LANCZOS)  # far above the sizes shown, and quick to cut out
        if kind == "view":
            im = cover(trim(im.convert("RGB")), 768, 384)
        elif kind == "field":
            im = cover(trim(im.convert("RGB")), 800, 480)
        elif kind == "siege":
            im = cover(trim(im.convert("RGB"), 0.02), 640, 400)
        elif kind == "vignette":
            im = cover(trim(im.convert("RGB"), 0.02), 400, 300)
        elif kind == "hall":
            im = cover(trim(im.convert("RGB"), 0.02), 960, 540)
        elif kind == "soldier":
            im = fit(cut_out(im), 240, 240)
        elif kind in ("unit", "rock"):
            im = fit(cut_out(im), 192, 144)
        elif kind == "mark":
            im = fit(cut_out(im), 160, 160)
        else:
            im = fit(cut_out(im, gaps=False), 128, 128)
        out = os.path.join(THEME, f"images/{alias}.webp")
        os.makedirs(os.path.dirname(out), exist_ok=True)
        im.save(out, "WEBP", quality=82, method=6)
        done += 1
    print(done, "pictures")


if __name__ == "__main__":
    main(sys.argv[1:])
