#!/usr/bin/env python3
"""Renders the sound effects to WAV files: Classic's tones and noises (the ones the game used
to synthesize) to themes/classic/audio/sfx/<effect>.wav, and the Age theme's steel, bows and
stone to themes/age/audio/sfx/<effect>.wav (without `button`, the click of every button,
which only Classic plays).

    python3 tools/sound/gen_sfx.py

Age's metal is modal synthesis: a strike is a short burst of noise exciting a few inharmonic
partials (a blade's, a plate's, a bell's), each decaying at its own rate and beating against
a slightly detuned twin. Seeded, so every run writes the same files.
"""
import math
import os
import random
import struct
import wave

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
RATE = 22050


def env(n, peak):
    """WebAudio's exponential ramp from `peak` down to 0.001 over n samples."""
    return [peak * (0.001 / peak) ** (i / n) for i in range(n)]


def tone(kind, f0, f1, dur, peak):
    n = int(RATE * dur)
    out, ph = [], 0.0
    e = env(n, peak)
    for i in range(n):
        f = f0 * (max(20, f1) / f0) ** (i / n)
        ph = (ph + f / RATE) % 1
        if kind == "sine":
            v = math.sin(ph * math.tau)
        elif kind == "square":
            v = 1.0 if ph < 0.5 else -1.0
        else:  # sawtooth
            v = 2 * ph - 1
        out.append(v * e[i])
    return out


def biquad(x, kind, f, q=1 / math.sqrt(2)):
    """RBJ biquad (lowpass, bandpass or highpass), as WebAudio's BiquadFilterNode."""
    w = math.tau * f / RATE
    a = math.sin(w) / (2 * q)
    c = math.cos(w)
    if kind == "lowpass":
        b = [(1 - c) / 2, 1 - c, (1 - c) / 2]
    elif kind == "highpass":
        b = [(1 + c) / 2, -(1 + c), (1 + c) / 2]
    else:
        b = [a, 0, -a]
    a0, a1, a2 = 1 + a, -2 * c, 1 - a
    y, x1, x2, y1, y2 = [], 0.0, 0.0, 0.0, 0.0
    for v in x:
        o = (b[0] * v + b[1] * x1 + b[2] * x2 - a1 * y1 - a2 * y2) / a0
        x2, x1, y2, y1 = x1, v, y1, o
        y.append(o)
    return y


def noise(dur, peak, f, kind="lowpass", seed=1):
    r = random.Random(seed)
    n = int(RATE * dur)
    e = env(n, peak)
    return [v * e[i] for i, v in enumerate(biquad([r.uniform(-1, 1) for _ in range(n)], kind, f))]


def add(*parts):
    n = max(len(p) for p in parts)
    return [sum(p[i] for p in parts if i < len(p)) for i in range(n)]


# ------------------------------------------------------------------ Age: steel, bows, stone


def silence(dur):
    return [0.0] * int(RATE * dur)


def at(t, part, total):
    """`part` placed at `t` seconds in a buffer of `total` seconds."""
    out = silence(total)
    o = int(RATE * t)
    for i, v in enumerate(part):
        if o + i < len(out):
            out[o + i] += v
    return out


def scale(x, peak):
    m = max(1e-9, max(abs(v) for v in x))
    return [v * peak / m for v in x]


def metal(partials, dur, seed=1, scrape=0.004, bright=5000):
    """Struck metal: (frequency, amplitude, decay in seconds) partials, each with a twin
    detuned by a few cents so it shimmers, after `scrape` seconds of bright noise."""
    r = random.Random(seed)
    n = int(RATE * dur)
    out = [0.0] * n
    for f, a, d in partials:
        if f >= RATE / 2:
            continue
        for det in (1.0, 1 + r.uniform(0.0012, 0.0035)):
            w = math.tau * f * det / RATE
            ph = r.random() * math.tau
            k = math.exp(-1 / (d * RATE))
            amp = a / 2
            for i in range(n):
                out[i] += amp * math.sin(ph + w * i)
                amp *= k
    burst = noise(scrape, 0.6, bright, "highpass", seed)
    return fade(add(out, burst), 0.25)


def fade(x, part=0.1):
    """Fades the last `part` of a sound out, so a ringing tail never stops dead."""
    n = len(x)
    m = max(1, int(n * part))
    return [v * (0.5 + 0.5 * math.cos(math.pi * (i - (n - m)) / m)) if i >= n - m else v for i, v in enumerate(x)]


# A steel blade's partials (a free bar: 1, 2.756, 5.404, 8.933) and an armour plate's.
BLADE = [1, 2.756, 5.404, 8.933]
PLATE = [1, 1.58, 2.31, 3.17, 4.12, 5.27]


def blade(f, dur, decay, seed):
    r = random.Random(seed)
    return metal([(f * k * r.uniform(0.99, 1.01), 1 / (1 + i * 0.6), decay / (1 + i * 0.5)) for i, k in enumerate(BLADE)], dur, seed, 0.006, 4000)


def plate(f, dur, decay, seed):
    r = random.Random(seed)
    return metal([(f * k * r.uniform(0.98, 1.02), 1 / (1 + i * 0.4), decay / (1 + i * 0.3)) for i, k in enumerate(PLATE)], dur, seed, 0.012, 2500)


def thud(f0, f1, dur, peak):
    """A body blow: a falling low sine (a shield, a wall, the ground)."""
    return tone("sine", f0, f1, dur, peak)


def pluck(f, dur, seed=1, damp=0.996):
    """A plucked string (Karplus-Strong): a bowstring's twang."""
    r = random.Random(seed)
    n = int(RATE * dur)
    period = max(2, int(RATE / f))
    buf = [r.uniform(-1, 1) for _ in range(period)]
    out = []
    for i in range(n):
        v = buf[i % period]
        nxt = buf[(i + 1) % period]
        buf[i % period] = damp * 0.5 * (v + nxt)
        out.append(v)
    e = env(n, 1.0)
    return [v * e[i] for i, v in enumerate(out)]


def swish(dur, f, peak, seed=1):
    """Air cut by a blade or an arrow: band-passed noise swelling and fading."""
    r = random.Random(seed)
    n = int(RATE * dur)
    x = biquad([r.uniform(-1, 1) for _ in range(n)], "bandpass", f, 1.2)
    return [v * peak * math.sin(math.pi * i / n) ** 2 for i, v in enumerate(x)]


def wood(dur, peak, seed=1):
    """An arrow or a stone hitting wood: a hollow knock."""
    return add(thud(320, 170, dur, peak), noise(dur * 0.4, peak * 0.8, 900, "bandpass", seed))


def clatter(n, span, total, seed=1):
    """Pieces of armour and weapons falling: `n` small clanks over `span` seconds."""
    r = random.Random(seed)
    out = silence(total)
    for k in range(n):
        t = r.uniform(0, span) * (k / n) ** 0.7
        piece = scale(plate(r.uniform(700, 2400), 0.35, r.uniform(0.06, 0.18), seed * 31 + k), r.uniform(0.25, 1.0) * (1 - 0.6 * k / n))
        out = add(out, at(t, piece, total))
    return out


def rubble(dur, peak, seed=1):
    """Stone cracking and falling: low rumble with grains of gravel."""
    r = random.Random(seed)
    out = noise(dur, peak, 500, "lowpass", seed)
    for k in range(40):
        t = r.uniform(0.02, dur * 0.8)
        out = add(out, at(t, noise(0.015, peak * r.uniform(0.2, 0.7) * (1 - t / dur), r.uniform(1500, 4000), "bandpass", seed + k), dur))
    return out


def age_arrow():
    """A bow loosed, the arrow's flight, then its head knocking into a shield."""
    total = 0.6
    twang = scale(pluck(98, 0.35, 3, 0.993), 0.6)
    fly = swish(0.24, 3400, 0.12, 4)
    hit = add(scale(wood(0.08, 1.0, 5), 0.6), scale(metal([(2650, 1, 0.05), (4210, 0.6, 0.03)], 0.12, 6, 0.002), 0.3))
    return scale(add(at(0, twang, total), at(0.04, fly, total), at(0.3, hit, total)), 0.28)


def age_volley():
    """A volley of arrows: several bows, whistling flights and knocks."""
    total = 0.95
    r = random.Random(7)
    out = silence(total)
    for k in range(6):
        t = r.uniform(0, 0.12)
        out = add(out, at(t, scale(swish(0.3, r.uniform(2500, 4200), 1, 10 + k), 0.12), total))
        out = add(out, at(t + 0.3 + r.uniform(0, 0.25), scale(wood(0.07, 1, 20 + k), r.uniform(0.3, 0.6)), total))
    out = add(out, at(0, scale(pluck(92, 0.3, 8, 0.993), 0.5), total), at(0.05, scale(pluck(104, 0.3, 9, 0.993), 0.4), total))
    return scale(out, 0.3)


def age_clash():
    """Sword on sword, then on armour: two blades ringing over a scrape."""
    total = 1.1
    a = scale(blade(1180, 1.1, 0.7, 11), 1.0)
    b = scale(blade(1530, 1.0, 0.55, 12), 0.7)
    scr = noise(0.035, 0.9, 3500, "bandpass", 13)
    return scale(add(a, at(0.004, b, total), scr, thud(190, 120, 0.05, 0.4)), 0.36)


def age_hit():
    """A blade biting into plate armour: a dull clank and the blow to the body."""
    total = 0.7
    clank = scale(plate(610, 0.7, 0.32, 21), 1.0)
    return scale(add(clank, noise(0.06, 0.8, 1800, "bandpass", 22), thud(150, 70, 0.14, 0.9)), 0.32)


def age_click():
    """A dagger tapping a buckle."""
    return scale(metal([(3150, 1, 0.04), (5230, 0.6, 0.025), (7400, 0.4, 0.015)], 0.1, 31, 0.002), 0.08)


def age_slash():
    """A sword swung through the air, its blade ringing faintly."""
    total = 0.45
    return scale(add(swish(0.22, 2200, 1.0, 41), at(0.12, scale(blade(2380, 0.33, 0.2, 42), 0.25), total)), 0.2)


def age_hammer():
    """A war hammer on plate: a heavy clang and a deep blow."""
    total = 0.9
    clang = scale(plate(420, 0.9, 0.45, 51), 1.0)
    return scale(add(clang, noise(0.05, 0.8, 1200, "bandpass", 52), thud(110, 50, 0.25, 1.2)), 0.36)


def age_fall():
    """A company falling: a crash, the ground's blow and armour clattering down."""
    total = 1.4
    boom = add(thud(85, 38, 0.6, 1.0), noise(0.9, 0.5, 380, "lowpass", 61))
    return scale(add(boom, clatter(9, 0.8, total, 62), at(0.02, scale(plate(500, 0.8, 0.35, 63), 0.5), total)), 0.45)


def age_siege():
    """A trebuchet's stone: its whistle coming in, the blow on the wall, the masonry falling."""
    total = 1.4
    whistle = swish(0.3, 900, 0.3, 71)
    blow = add(thud(65, 32, 0.7, 1.0), rubble(1.0, 0.45, 72))
    return scale(add(whistle, at(0.28, blow, total)), 0.46)


def age_bell():
    """A great bell struck once (hum, prime, tierce, quint, nominal)."""
    f = 392
    partials = [(f * 0.5, 0.7, 2.2), (f, 1.0, 1.6), (f * 1.19, 0.6, 1.1), (f * 1.5, 0.35, 0.9), (f * 2.0, 0.6, 0.8), (f * 2.51, 0.25, 0.5), (f * 3.0, 0.2, 0.35)]
    x = metal(partials, 2.2, 81, 0.008, 3000)
    n = len(x)
    return scale([v * min(1.0, (n - i) / (RATE * 0.3)) for i, v in enumerate(x)], 0.26)


AGE = {
    "beam": age_slash,
    "heavy_beam": age_hammer,
    "missile": age_volley,
    "explosion": age_fall,
    "hit": age_hit,
    "bomb": age_siege,
    "special": age_bell,
    "click": age_click,
    "arrow": age_arrow,
    "clash": age_clash,
}

# ------------------------------------------------------------------ Classic

EFFECTS = {
    "beam": lambda: tone("sawtooth", 1400, 300, 0.18, 0.12),
    "heavy_beam": lambda: tone("sawtooth", 600, 80, 0.4, 0.18),
    "missile": lambda: noise(0.35, 0.15, 900, "bandpass"),
    "explosion": lambda: add(noise(0.8, 0.4, 600), tone("sine", 90, 30, 0.6, 0.3)),
    "hit": lambda: noise(0.12, 0.2, 2500, "bandpass"),
    "bomb": lambda: tone("square", 300, 40, 0.5, 0.15),
    "special": lambda: tone("sine", 200, 1600, 0.5, 0.15),
    "click": lambda: tone("square", 900, 900, 0.04, 0.05),
    "arrow": lambda: noise(0.15, 0.18, 4000, "highpass"),
    "clash": lambda: add(tone("square", 1800, 1200, 0.08, 0.08), noise(0.1, 0.12, 3000, "bandpass")),
}


def write(path, samples):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    peak = max(1e-9, max(abs(v) for v in samples))
    k = min(1.0, 0.98 / peak)
    with wave.open(path, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(b"".join(struct.pack("<h", int(v * k * 32767)) for v in samples))


def main():
    global RATE
    for name, make in EFFECTS.items():
        write(os.path.join(ROOT, "themes/classic/audio/sfx", f"{name}.wav"), make())
    write(os.path.join(ROOT, "themes/classic/audio/sfx/button.wav"), EFFECTS["click"]())
    # Age's steel rings up high: twice the sample rate.
    RATE = 44100
    for name, make in AGE.items():
        write(os.path.join(ROOT, "themes/age/audio/sfx", f"{name}.wav"), fade(make(), 0.08))
    print(f"{len(EFFECTS) + 1} effects for Classic, {len(AGE)} for Age")


if __name__ == "__main__":
    main()
