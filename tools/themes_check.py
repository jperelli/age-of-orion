#!/usr/bin/env python3
"""Lists what each theme lacks of Classic's files (Classic has every picture and sound the game
uses); the game shows a placeholder for a missing picture and plays nothing for a missing sound.

    python3 tools/themes_check.py [theme]
"""
import os
import re
import sys
from collections import defaultdict

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
THEMES = os.path.join(ROOT, "themes")
EXT = {"images": (".svg", ".png", ".jpg", ".jpeg", ".webp"), "audio": (".ogg", ".mp3", ".wav", ".mid")}


def names(theme, sub):
    base = os.path.join(THEMES, theme, sub)
    out = set()
    for d, _, files in os.walk(base):
        for f in files:
            stem, ext = os.path.splitext(f)
            if ext.lower() in EXT[sub]:
                out.add(os.path.relpath(os.path.join(d, stem), base).replace(os.sep, "/"))
    return out


def family(name):
    return "/".join("#" if re.fullmatch(r"\d+", p) else p for p in name.split("/"))


def main():
    only = sys.argv[1:]
    for theme in sorted(os.listdir(THEMES)):
        if theme == "classic" or not os.path.exists(os.path.join(THEMES, theme, "theme.json")) or (only and theme not in only):
            continue
        print(f"{theme}:")
        for sub in ("images", "audio"):
            missing = sorted(names("classic", sub) - names(theme, sub))
            groups = defaultdict(list)
            for m in missing:
                groups[family(m)].append(m)
            print(f"  {sub}: {len(names(theme, sub))} files, {len(missing)} of Classic's missing")
            for fam, items in sorted(groups.items()):
                print(f"    {fam}: {len(items)}" if len(items) > 1 else f"    {items[0]}")


if __name__ == "__main__":
    main()
