"""Age theme pictures for the map, the city panel and the battle screen, made with Google's
Nano Banana (Gemini image generation) instead of the local model of gen_art.py.

    GEMINI_API_KEY=... python3 tools/art/gen_gemini.py [name prefix...]

Same names and subjects as gen_art.py (subjects.py); writes OUT/<name>-s<TRY>.png (TRY=1 by
default, another number for another attempt), which finish_art.py turns into theme
pictures. Sprites are asked on plain white so finish_art.py can cut them out; troops, map
markers and coats of arms are drawn with an approved picture of their kind as a style
reference (REFS below), so each set looks alike.
"""

import base64
import io
import json
import os
import sys
import time
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor

from PIL import Image

from subjects import (ARMS_P, COUNCIL, FALLEN, FIELDS, HALLS, HERALD, ICONS, MARKS, NEWS, PORTRAITS_AGE, PORTRAITS_CLASSIC,
                      RACES, ROCKS, SIEGES, SOLDIERS, SPECIAL, UNITS, VIEWS)

MODEL = os.environ.get("MODEL", "gemini-nano-banana-2.1")
OUT = os.environ.get("OUT", "/tmp/art/gemini")
TRY = int(os.environ.get("TRY", "1"))
URL = f"https://generativelanguage.googleapis.com/v1beta/models/{MODEL}:generateContent"

WHITE = "Isolated on a plain pure white background, no ground, no shadow, no text, no frame."
VIEW = ("A wide panoramic medieval landscape oil painting of {}. Golden hour light, rich detail, "
        "no people, no text, no frame, no border.")
FIELD = ("A top-down orthographic view looking straight down at {}, used as the background of a "
         "medieval strategy game battle grid. Hand-painted, muted natural colours, even flat lighting, "
         "a seamless ground texture with no horizon, no perspective, no people, no buildings, no text.")
UNIT = ("{}. Full body, side view, marching or facing to the right. Painted medieval concept art for a "
        "strategy game sprite, in the same painting style, scale and lighting as the reference image. " + WHITE)
ROCK = ("{}, seen from the side and slightly above. Painted illustration of an obstacle on a medieval "
        "strategy game battlefield, in the same painting style as the reference image. " + WHITE)
MARK = ("{}, as a small painted miniature token for a strategy game map, seen from above at an angle "
        "(isometric), standing on a small round base of grass, in the same style as the reference image. "
        "Isolated on a plain pure white background, no text.")
ARMS = ("A medieval coat of arms on a heater shield: {}. Flat heraldic illustration with bold simple shapes "
        "and a dark outline, the same shield shape and style as the reference image, centered. "
        "Isolated on a plain pure white background, no text, no letters, no crown, no supporters.")

PORTRAIT_AGE = ("A painted head-and-shoulders portrait of {}. A historical court portrait in oils, historically "
                "accurate costume, realistic, dramatic chiaroscuro lighting, dark background, in the same painting "
                "style as the reference portrait (only its style, not its face). No text, no frame.")
PORTRAIT_CLASSIC = ("A painted head-and-shoulders portrait of {}, the ruler of an alien empire in a space strategy game. "
                    "Painted science fiction illustration, dramatic lighting, a starship bridge in the background, "
                    "in the same painting style as the reference portrait (only its style, not its face). No text, no frame.")
NOT_CHRISTIAN = {"mrrshan", "silicoid", "sakkra", "psilon", "klackon"}
SIEGE = ("A wide painted background for the siege screen of a medieval strategy game, seen from the side: in the "
         "middle distance the walls, gate and towers of a besieged medieval city; in front, {}, an empty open "
         "battlefield where two armies will meet in the lower half of the picture. Oil painting, overcast "
         "dramatic light, no people, no text, no frame.")
SOLDIER = ("{}. Full body, standing in profile facing to the right, ready to fight. Painted medieval concept art "
           "for a strategy game sprite, in the same painting style as the reference image. " + WHITE)
NEWS_PIC = ("A medieval illuminated manuscript miniature showing {}. Rich colours, gold leaf, a simple coloured "
            "background, in the same style as the reference image, no text, no letters, no frame or border.")
# Animated battle troops: four frames of the troop in its own approved picture, in a 2x2 grid.
ANIM = ("Four animation frames of exactly the troop in the reference image, {}, arranged in a 2 by 2 grid of "
        "four equal cells read left to right, top to bottom. The same figures, colours, equipment, size and "
        "position in every cell, facing to the right, each cell holding the whole troop once. Plain pure white "
        "background, no grid lines, no borders, no captions, no text, no numbers.")
ANIMS = {
    "march": "marching to the right in a walk cycle: only legs, arms and weapons move from frame to frame",
    "attack": "attacking: frame 1 readying their weapons, frame 2 striking or loosing, frame 3 following "
              "through, frame 4 recovering to their guard",
}
HALL = ("{}, seen from the entrance, an empty throne at the far end, no people. Painted medieval interior, warm "
        "torchlight, oil painting, no text, no frame.")

# name -> (prompt, aspect ratio, reference picture or None)
jobs: dict[str, tuple[str, str, str | None]] = {}
for k, p in VIEWS.items():
    jobs[f"view-{k}"] = (VIEW.format(p), "16:9", None)
for k, p in FIELDS.items():
    jobs[f"field-{k}"] = (FIELD.format(p), "16:9", None)
for i, p in enumerate(UNITS):
    jobs[f"unit-{i:02}"] = (UNIT.format(p[0].upper() + p[1:]), "4:3", "unit-13")
for k, p in SPECIAL.items():
    jobs[f"unit-{k}"] = (UNIT.format(p[0].upper() + p[1:]), "4:3", "unit-13")
for i, p in enumerate(ROCKS):
    jobs[f"rock-{i}"] = (ROCK.format(p[0].upper() + p[1:]), "4:3", "unit-13")
for k, p in MARKS.items():
    jobs[f"mark-{k}"] = (MARK.format(p[0].upper() + p[1:]), "1:1", None if k == "castle" else "mark-castle")
for k, p in ARMS_P.items():
    jobs[f"arms-{k}"] = (ARMS.format(p), "1:1", None if k == "human" else "arms-human")


for k, p in SIEGES.items():
    jobs[f"siege-{k}"] = (SIEGE.format(p), "16:9", None)
for k, p in SOLDIERS.items():
    jobs[f"soldier-{k}"] = (SOLDIER.format(p[0].upper() + p[1:]), "3:4", "unit-13")
jobs["fallen"] = (SOLDIER.format(FALLEN[0].upper() + FALLEN[1:]).replace("standing in profile facing to the right, ready to fight", "lying on the ground"), "4:3", "unit-13")
jobs["herald"] = (SOLDIER.format(HERALD[0].upper() + HERALD[1:]).replace(", ready to fight", ""), "3:4", "unit-13")
for k, p in NEWS.items():
    jobs[f"news-{k}"] = (NEWS_PIC.format(p), "4:3", None if k == "plague" else "news-plague")
for k, p in HALLS.items():
    jobs[f"hall-{k}"] = (HALL.format(p[0].upper() + p[1:]), "16:9", None)
jobs["council"] = (HALL.format(COUNCIL[0].upper() + COUNCIL[1:]).replace(", an empty throne at the far end, no people", ""), "16:9", None)

for i in range(len(UNITS)):
    for a, p in ANIMS.items():
        jobs[f"anim-{i:02}-{a}"] = (ANIM.format(p), "4:3", f"unit-{i:02}")
for k in SPECIAL:
    for a, p in ANIMS.items():
        jobs[f"anim-{k}-{a}"] = (ANIM.format(p), "4:3", f"unit-{k}")

ICON = ("{}. A small painted icon for a strategy game, simple and readable at a small size, in the same style "
        "as the reference image. " + WHITE)
for theme, icons in ICONS.items():
    for k, p in icons.items():
        jobs[f"icon-{theme}-{k}"] = (ICON.format(p[0].upper() + p[1:]), "1:1", None if k == "pop" else f"icon-{theme}-pop")

# Race portraits go to OUT/portraits as <theme>-<n>-<race>-s<TRY>.png, for finish_portraits.py.
for i, r in enumerate(RACES):
    extra = " No Christian crosses." if r in NOT_CHRISTIAN else ""
    jobs[f"portraits/age-{i}-{r}"] = (PORTRAIT_AGE.format(PORTRAITS_AGE[r]) + extra, "4:5", None if i == 0 else "portraits/age-0-human")
    jobs[f"portraits/classic-{i}-{r}"] = (PORTRAIT_CLASSIC.format(PORTRAITS_CLASSIC[r].replace(", not human", "")), "4:5", None if i == 0 else "portraits/classic-0-human")


def ref_path(name: str) -> str:
    return f"{OUT}/{name}-s1.png"


def generate(name: str) -> str:
    prompt, aspect, ref = jobs[name]
    parts: list[dict] = [{"text": prompt}]
    if ref and ref != name:
        if not os.path.exists(ref_path(ref)):
            return f"{name}: no reference picture {ref} yet"
        with open(ref_path(ref), "rb") as f:
            parts.append({"inlineData": {"mimeType": "image/png", "data": base64.b64encode(f.read()).decode()}})
    body = {"contents": [{"parts": parts}], "generationConfig": {"responseModalities": ["IMAGE"], "imageConfig": {"aspectRatio": aspect}}}
    for attempt in range(6):
        req = urllib.request.Request(URL, data=json.dumps(body).encode(), headers={"Content-Type": "application/json", "x-goog-api-key": os.environ["GEMINI_API_KEY"]})
        t = time.time()
        try:
            resp = json.load(urllib.request.urlopen(req, timeout=300))
        except urllib.error.HTTPError as e:
            err = json.loads(e.read() or b"{}").get("error", {})
            if e.code in (429, 500, 503) and attempt < 5:
                time.sleep(10 * (attempt + 1))
                continue
            return f"{name}: HTTP {e.code} {err.get('status')} {(err.get('message') or '')[:200]}"
        except Exception as e:  # network hiccup
            if attempt < 5:
                time.sleep(5)
                continue
            return f"{name}: {e}"
        for part in resp.get("candidates", [{}])[0].get("content", {}).get("parts", []):
            if "inlineData" in part:
                img = Image.open(io.BytesIO(base64.b64decode(part["inlineData"]["data"]))).convert("RGB")
                img.save(f"{OUT}/{name}-s{TRY}.png")
                return f"{name} {time.time() - t:.0f}s {img.size[0]}x{img.size[1]}"
        reason = resp.get("candidates", [{}])[0].get("finishReason") or resp.get("promptFeedback")
        return f"{name}: no image ({reason})"
    return f"{name}: gave up"


def main() -> None:
    os.makedirs(f"{OUT}/portraits", exist_ok=True)
    only = sys.argv[1:]
    todo = [n for n in jobs if (not only or any(n.startswith(o) for o in only)) and not os.path.exists(f"{OUT}/{n}-s{TRY}.png")]
    # References first, then everything else in parallel.
    for ref in ("unit-13", "mark-castle", "arms-human", "news-plague", "icon-age-pop", "icon-classic-pop",
                "portraits/age-0-human", "portraits/classic-0-human"):
        if ref in todo:
            print(generate(ref), flush=True)
            todo.remove(ref)
    with ThreadPoolExecutor(int(os.environ.get("WORKERS", "4"))) as pool:
        for line in pool.map(generate, todo):
            print(line, flush=True)


if __name__ == "__main__":
    main()
