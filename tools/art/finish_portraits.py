"""Turns the chosen generated portraits into the theme pictures.

    python tools/art/finish_portraits.py SRC_DIR

SRC_DIR holds `<theme>-<race index>-<race>-s<seed>.png` (from gen_portraits.py); when
several seeds exist for one portrait, the one named in PICK below wins, else the lowest
seed. Writes web/public/themes/<age|classic>/images/race/<n>.webp (384x480) and lists them
in each theme.json. The bottom strip is cropped away: the model sometimes signs there.
"""

import json
import os
import re
import sys

from PIL import Image

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")
THEMES = {"age": "age", "classic": "classic"}
# Seed chosen for a portrait when several were made (default: the lowest seed).
PICK: dict[str, int] = {"classic-4": 11, "age-5": 11, "classic-6": 11, "age-7": 11}


def main(src: str) -> None:
    found: dict[str, dict[int, str]] = {}
    for name in os.listdir(src):
        m = re.match(r"(age|classic)-(\d)-\w+-s(\d+)\.png$", name)
        if m:
            found.setdefault(f"{m[1]}-{m[2]}", {})[int(m[3])] = os.path.join(src, name)
    for theme in THEMES:
        out_dir = os.path.join(ROOT, "web/public/themes", theme, "images/race")
        os.makedirs(out_dir, exist_ok=True)
        images = {}
        for n in range(10):
            seeds = found.get(f"{theme}-{n}")
            if not seeds:
                continue
            seed = PICK.get(f"{theme}-{n}", min(seeds))
            im = Image.open(seeds[seed]).convert("RGB")
            w, h = im.size
            cut = h // 20
            im = im.crop((cut * w // h // 2, 0, w - cut * w // h // 2, h - cut))
            im = im.resize((384, 480), Image.LANCZOS)
            path = f"images/race/{n}.webp"
            im.save(os.path.join(ROOT, "web/public/themes", theme, path), "WEBP", quality=82, method=6)
            images[f"race/{n}"] = path
        tj = os.path.join(ROOT, "web/public/themes", theme, "theme.json")
        d = json.load(open(tj))
        d.setdefault("images", {}).update(images)
        with open(tj, "w") as f:
            json.dump(d, f, indent=1, ensure_ascii=False)
            f.write("\n")
        print(theme, len(images), "portraits")


if __name__ == "__main__":
    main(sys.argv[1])
