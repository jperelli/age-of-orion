"""Turns the chosen pictures of gen_art.py into Age theme pictures.

    python tools/art/finish_art.py SRC_DIR...

The SRC_DIRs hold `<name>-s<seed>.png` (gen_gemini.py numbers its tries 1, 2...; gen_art.py
uses its seeds); when several exist, PICK names the one to use, else the lowest wins. Landscapes are resized; sprites (units, rocks, markers, arms) are
cut out of their white background and cropped. Writes web/public/themes/age/images/... and
lists every picture under `images` in web/public/themes/age/theme.json.
"""

import json
import os
import re
import sys

import numpy as np
from PIL import Image, ImageFilter

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")
THEME = os.path.join(ROOT, "web/public/themes/age")
RACES = ["human", "mrrshan", "silicoid", "sakkra", "psilon", "alkari", "klackon", "bulrathi", "meklar", "darlok"]
# Seed chosen for a picture when several were made (default: the lowest seed).
PICK: dict[str, int] = {}


def alias_of(name: str) -> tuple[str, str, str]:
    """(alias, file, kind) of a generated picture."""
    kind, _, key = name.partition("-")
    if kind == "view":
        if key.startswith("city"):
            return f"view/city/{key[4:]}", f"images/view/city{key[4:]}.webp", "view"
        return f"view/{key}", f"images/view/{key}.webp", "view"
    if kind == "field":
        return f"battle/field/{key}", f"images/battle/field-{key}.webp", "field"
    if kind == "unit":
        return f"unit/{int(key) if key.isdigit() else key}", f"images/unit/{key}.webp", "unit"
    if kind == "rock":
        return f"battle/rock/{key}", f"images/battle/rock{key}.webp", "rock"
    if kind == "mark":
        alias = {"army": "army", "caravan": "caravan", "canal": "stargate"}.get(key, f"mark/{key}")
        return alias, f"images/map/{key}.webp", "mark"
    if kind == "arms":
        return f"arms/{RACES.index(key)}", f"images/arms/{key}.webp", "arms"
    raise ValueError(name)


def holes(mask: np.ndarray, min_area: int) -> np.ndarray:
    """The connected regions of `mask` of at least `min_area` pixels (background seen through
    gaps, such as between legs)."""
    h, w = mask.shape
    seen = np.zeros_like(mask)
    out = np.zeros_like(mask)
    for y0, x0 in zip(*np.nonzero(mask)):
        if seen[y0, x0]:
            continue
        region, todo = [], [(y0, x0)]
        seen[y0, x0] = True
        while todo:
            y, x = todo.pop()
            region.append((y, x))
            for ny, nx in ((y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1)):
                if 0 <= ny < h and 0 <= nx < w and mask[ny, nx] and not seen[ny, nx]:
                    seen[ny, nx] = True
                    todo.append((ny, nx))
        if len(region) >= min_area:
            ys, xs = zip(*region)
            out[list(ys), list(xs)] = True
    return out


def cut_out(im: Image.Image, gaps: bool = True) -> Image.Image:
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
    return out.crop(box) if box else out


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
    images = {}
    for name in sorted(found):
        seeds = found[name]
        im = Image.open(seeds[PICK.get(name, min(seeds))])
        alias, path, kind = alias_of(name)
        if kind not in ("view", "field"):
            im.thumbnail((640, 640), Image.LANCZOS)  # far above the sizes shown, and quick to cut out
        if kind == "view":
            im = cover(trim(im.convert("RGB")), 768, 384)
        elif kind == "field":
            im = cover(trim(im.convert("RGB")), 800, 480)
        elif kind in ("unit", "rock"):
            im = fit(cut_out(im), 192, 144)
        elif kind == "mark":
            im = fit(cut_out(im), 160, 160)
        else:
            im = fit(cut_out(im, gaps=False), 128, 128)
        out = os.path.join(THEME, path)
        os.makedirs(os.path.dirname(out), exist_ok=True)
        im.save(out, "WEBP", quality=82, method=6)
        images[alias] = path
    tj = os.path.join(THEME, "theme.json")
    d = json.load(open(tj))
    d.setdefault("images", {}).update(images)
    with open(tj, "w") as f:
        json.dump(d, f, indent=1, ensure_ascii=False)
        f.write("\n")
    print(len(images), "pictures")


if __name__ == "__main__":
    main(sys.argv[1:])
