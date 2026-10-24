#!/usr/bin/env python3
"""Renders the sound effects (the tones and noises the game used to synthesize) to WAV files:
themes/classic/audio/sfx/<effect>.wav, and the same for the Age theme (without `button`, the
click of every button, which only Classic plays).

    python3 tools/sound/gen_sfx.py
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
    for name, make in EFFECTS.items():
        s = make()
        write(os.path.join(ROOT, "themes/classic/audio/sfx", f"{name}.wav"), s)
        write(os.path.join(ROOT, "themes/age/audio/sfx", f"{name}.wav"), s)
    write(os.path.join(ROOT, "themes/classic/audio/sfx/button.wav"), EFFECTS["click"]())
    print(f"{len(EFFECTS) + 1} effects for Classic, {len(EFFECTS)} for Age")


if __name__ == "__main__":
    main()
