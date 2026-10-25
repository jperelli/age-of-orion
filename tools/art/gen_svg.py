#!/usr/bin/env python3
"""Draws the Classic theme's pictures as SVG files, every one the game asks for, and the few
pictures of the Age theme the game used to draw by itself (icons, map markers, mists).

    python3 tools/art/gen_svg.py

Writes themes/<theme>/images/<name>.svg (animations: a strip, with a .json of its frames
beside it). A picture a theme has in another format (Classic's
race portraits, Age's paintings) is never written over: an SVG would take its place.
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from svgart import icons, kit, land, people, screens, space, tech  # noqa: E402

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))


def main() -> None:
    for part in (icons, space, land, people, tech, screens):
        part.build()
    counts = {}
    for (theme, name), svg in sorted(kit.OUT.items()):
        base = os.path.join(ROOT, "themes", theme, "images", name)
        other = [base + e for e in (".png", ".jpg", ".jpeg", ".webp") if os.path.exists(base + e)]
        if other:
            sys.exit(f"{theme}: {name} already has {os.path.relpath(other[0], ROOT)}")
        os.makedirs(os.path.dirname(base), exist_ok=True)
        with open(base + ".svg", "w") as f:
            f.write(svg)
        counts[theme] = counts.get(theme, 0) + 1
        if (theme, name) in kit.ANIMS:
            with open(base + ".json", "w") as f:
                json.dump(kit.ANIMS[(theme, name)], f)
                f.write("\n")
    for theme, n in sorted(counts.items()):
        print(f"themes/{theme}/images: {n} SVG pictures")


if __name__ == "__main__":
    main()
