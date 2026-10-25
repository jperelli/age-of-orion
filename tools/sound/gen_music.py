#!/usr/bin/env python3
"""Composes the music of the Classic and Age themes as MIDI files, for the game's General MIDI
style synthesizer: themes/<theme>/audio/music/<tune>.mid, one per screen (menu, battle, news,
ground, council, endings, colonization, bombing, espionage, sabotage) and one per race for
audiences (audience/<race>, MOO1 order).

    python3 tools/sound/gen_music.py

Each tune is a chord progression in a key and mode, arranged for a few instruments: pads,
arpeggios, a bass line, a melody built from a motif (A A' B A'') and drums. Age's are
medieval: modes, drones and open fifths, melodies moving by step (doubled a fourth below for
organum), lute, recorder, shawm, sackbut, fiddle, bells, tabor and war drums. Seeded, so
every run writes the same files.
"""
import os
import random
import struct

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
TPQ = 480

MODES = {
    "major": [0, 2, 4, 5, 7, 9, 11],
    "minor": [0, 2, 3, 5, 7, 8, 10],
    "dorian": [0, 2, 3, 5, 7, 9, 10],
    "phrygian": [0, 1, 3, 5, 7, 8, 10],
    "lydian": [0, 2, 4, 6, 7, 9, 11],
    "mixolydian": [0, 2, 4, 5, 7, 9, 10],
    "harmonic": [0, 2, 3, 5, 7, 8, 11],
    "whole": [0, 2, 4, 6, 8, 10, 12],
    "hijaz": [0, 1, 4, 5, 7, 8, 10],
    "penta": [0, 2, 4, 7, 9],
}

# General MIDI programs (the synthesizer picks a timbre per family of 8).
P = dict(piano=0, celesta=8, vibes=11, marimba=12, organ=19, guitar=24, bass=33, synbass=38, strings=48, tremolo=44, harp=46, choir=52,
         brass=61, horn=60, tuba=58, trumpet=56, reed=68, flute=73, pan=75, lead=80, saw=81, pad=89, warm=89, sitar=104, timpani=47,
         lute=24, recorder=74, shawm=68, sackbut=57, fiddle=40, viol=42, dulcimer=15, bells=14, church=19, drone=20, koto=107, harpsichord=6)


class Song:
    def __init__(self, bpm, key, mode, beats=4):
        self.bpm, self.key, self.scale, self.beats = bpm, key, MODES[mode], beats
        self.tracks = {}
        self.end = 0

    def ch(self, ch, program, volume=100, pan=64):
        t = self.tracks.setdefault(ch, [])
        if ch != 9:
            t.append((0, bytes([0xC0 | ch, program])))
        t.append((0, bytes([0xB0 | ch, 7, volume])))
        t.append((0, bytes([0xB0 | ch, 10, pan])))
        return ch

    def note(self, ch, beat, dur, pitch, vel=90):
        on, off = round(beat * TPQ), round((beat + dur) * TPQ) - 8
        pitch = max(0, min(127, pitch))
        self.tracks[ch].append((on, bytes([0x90 | ch, pitch, vel])))
        self.tracks[ch].append((max(on + 1, off), bytes([0x80 | ch, pitch, 0])))

    def degree(self, d, octave=0):
        """Pitch of scale degree d (0 the tonic; may be negative or above 6)."""
        n = len(self.scale)
        return self.key + 12 * (octave + d // n) + self.scale[d % n]

    def chord(self, d, octave=0, size=3):
        return [self.degree(d + 2 * k, octave) for k in range(size)]

    def bytes(self):
        out = [b"MThd" + struct.pack(">IHHH", 6, 1, len(self.tracks) + 1, TPQ)]
        tempo = round(60_000_000 / self.bpm)
        meta = [(0, b"\xff\x51\x03" + tempo.to_bytes(3, "big"))]
        tracks = [meta] + [self.tracks[c] for c in sorted(self.tracks)]
        # A last event at the end of the last bar, so a looping tune repeats on the beat.
        tracks[-1].append((round(self.end * TPQ), bytes([0xB0 | sorted(self.tracks)[-1], 11, 127])))
        for ev in tracks:
            data, last = b"", 0
            for tick, e in sorted(ev, key=lambda x: (x[0], x[1][0] & 0xF0 == 0x90)):
                data += var(tick - last) + e
                last = tick
            data += var(0) + b"\xff\x2f\x00"
            out.append(b"MTrk" + struct.pack(">I", len(data)) + data)
        return b"".join(out)


def var(n):
    b = [n & 0x7F]
    n >>= 7
    while n:
        b.append(0x80 | (n & 0x7F))
        n >>= 7
    return bytes(reversed(b))


RHYTHMS4 = [[1, 1, 1, 1], [1.5, 0.5, 1, 1], [2, 1, 1], [1, 0.5, 0.5, 2], [0.5, 0.5, 1, 2], [1, 1, 2], [3, 1], [0.75, 0.25, 1, 1, 1], [1, 0.5, 0.5, 1, 1]]
RHYTHMS3 = [[1, 1, 1], [2, 1], [1.5, 0.5, 1], [1, 0.5, 0.5, 1], [3]]


def melody(s, r, ch, prog, octave, vel=96, modal=False, shift=0):
    """A melody over the progression: a two-bar motif (A), the same rhythm over the next
    chords (A'), a new phrase (B) and the motif again ending on the tonic (A''). `modal`:
    moving mostly by step, closing from the step above the tonic (Age); `shift` moves it by
    scale degrees (organum's voice a fourth below)."""
    if modal:
        return modal_melody(s, r, ch, prog, octave, vel, shift)
    bars = len(prog)
    rh = RHYTHMS4 if s.beats == 4 else RHYTHMS3
    motif = [r.choice(rh), r.choice(rh)]
    other = [r.choice(rh), r.choice(rh)]
    cur = prog[0] + r.choice([0, 2, 4])
    for bar in range(bars):
        phrase = (bar // 2) % 4
        pattern = (other if phrase == 2 else motif)[bar % 2]
        beat = bar * s.beats
        chord_tones = {(prog[bar] + k) % 7 for k in (0, 2, 4)}
        for i, d in enumerate(pattern):
            last = bar == bars - 1 and i == len(pattern) - 1
            if last:
                cur = 7 * round(cur / 7)  # land on the tonic
            elif i == 0 and cur % 7 not in chord_tones:
                cur = min((c for c in range(cur - 3, cur + 4) if c % 7 in chord_tones), key=lambda c: abs(c - cur))
            elif i > 0:
                cur += r.choice([-2, -1, -1, 1, 1, 2, 0, 3, -3])
                cur = max(-3, min(10, cur))
            s.note(ch, beat, d * (0.95 if not last else 1), s.degree(cur, octave), vel - (0 if i == 0 else 10))
            beat += d


def modal_melody(s, r, ch, prog, octave, vel, shift):
    n = len(s.scale)
    bars = len(prog)
    rh = RHYTHMS4 if s.beats == 4 else RHYTHMS3
    motif = [r.choice(rh), r.choice(rh)]
    other = [r.choice(rh), r.choice(rh)]
    cur = prog[0] + r.choice([0, 2, n])
    for bar in range(bars):
        phrase = (bar // 2) % 4
        pattern = (other if phrase == 2 else motif)[bar % 2]
        beat = bar * s.beats
        tones = {(prog[bar] + k) % n for k in (0, 2, 4)} | {0}
        for i, d in enumerate(pattern):
            last = bar == bars - 1 and i == len(pattern) - 1
            before = bar == bars - 1 and i == len(pattern) - 2
            if last:
                cur = n * round(cur / n)
            elif before:
                cur = n * round(cur / n) + 1  # the step above the tonic, then down to it
            elif i == 0 and cur % n not in tones:
                cur = min((c for c in range(cur - 2, cur + 3) if c % n in tones), key=lambda c: abs(c - cur), default=cur)
            elif i > 0:
                cur += r.choice([-1, -1, -1, 1, 1, 1, -2, 2, 0, 3, -3, 4] if r.random() < 0.15 else [-1, -1, 1, 1, -2, 2, 0])
                cur = max(-2, min(n + 4, cur))
            s.note(ch, beat, d * (0.95 if not last else 1), s.degree(cur + shift, octave), vel - (0 if i == 0 else 12))
            beat += d


def drone(s, ch, prog, octave, vel=55):
    """The tonic and its fifth held under everything (bagpipe, hurdy-gurdy, organ)."""
    for bar in range(len(prog)):
        for p in (s.degree(0, octave), s.degree(0, octave) + 7):
            s.note(ch, bar * s.beats, s.beats, p, vel)


def fifths(s, ch, prog, octave, vel=60):
    """Open fifths and octaves on each bar's chord root, no thirds."""
    for bar, d in enumerate(prog):
        root = s.degree(d, octave)
        for p in (root, root + 7, root + 12):
            s.note(ch, bar * s.beats, s.beats, p, vel)


def lute(s, ch, prog, octave, vel=78):
    """A lute's broken chords: the root low, then the chord plucked up and down."""
    for bar, d in enumerate(prog):
        b = bar * s.beats
        c = s.chord(d, octave)
        seq = [c[0] - 12, c[1], c[2], c[1]] if s.beats == 4 else [c[0] - 12, c[1], c[2]]
        for i in range(s.beats * 2):
            p = seq[(i // 2) % len(seq)] if i % 2 == 0 else c[2] + (12 if i % 4 == 1 else 0)
            s.note(ch, b + i * 0.5, 0.5 if i % 2 == 0 else 0.45, p, vel if i % 2 == 0 else vel - 18)


def bells(s, ch, prog, octave, every=2, vel=90):
    """A bell tolling the tonic every `every` bars, its fifth between."""
    for bar in range(len(prog)):
        if bar % every == 0:
            s.note(ch, bar * s.beats, s.beats * every, s.degree(0, octave), vel)
        elif every > 1 and bar % every == every // 2:
            s.note(ch, bar * s.beats, s.beats, s.degree(0, octave) - 5, vel - 25)


def pads(s, ch, prog, octave, vel=60):
    for bar, d in enumerate(prog):
        for p in s.chord(d, octave):
            s.note(ch, bar * s.beats, s.beats, p, vel)


def arps(s, ch, prog, octave, step=0.5, vel=70):
    for bar, d in enumerate(prog):
        notes = s.chord(d, octave) + [s.degree(d + 7, octave)]
        seq = notes + notes[-2:0:-1]
        n = int(s.beats / step)
        for i in range(n):
            s.note(ch, bar * s.beats + i * step, step, seq[i % len(seq)], vel)


def bass(s, ch, prog, octave, style="root", vel=95):
    for bar, d in enumerate(prog):
        root, fifth = s.degree(d, octave), s.degree(d + 4, octave)
        b = bar * s.beats
        if style == "eighths":
            for i in range(s.beats * 2):
                s.note(ch, b + i * 0.5, 0.5, root if i % 4 != 3 else fifth, vel)
        elif style == "walk":
            for i in range(s.beats):
                s.note(ch, b + i, 1, [root, fifth, root + 12, fifth][i % 4], vel)
        else:
            s.note(ch, b, s.beats / 2, root, vel)
            s.note(ch, b + s.beats / 2, s.beats / 2, fifth if s.beats == 4 else root, vel - 10)


def stabs(s, ch, prog, octave, beats, vel=100):
    for bar, d in enumerate(prog):
        for bt in beats:
            for p in s.chord(d, octave):
                s.note(ch, bar * s.beats + bt, 0.4, p, vel)


def drums(s, bars, style):
    ch = s.ch(9, 0, 100)
    for bar in range(bars):
        b = bar * s.beats
        if style == "battle":
            for i in range(8):
                s.note(ch, b + i * 0.5, 0.25, 42, 60)
            for bt in (0, 1.5, 2, 3.5):
                s.note(ch, b + bt, 0.25, 36, 110)
            for bt in (1, 3):
                s.note(ch, b + bt, 0.25, 38, 100)
            if bar % 4 == 3:
                for i in range(4):
                    s.note(ch, b + 3 + i * 0.25, 0.25, 45 - i * 2, 90)
        elif style == "march":
            for bt in (0, 2):
                s.note(ch, b + bt, 0.25, 36, 100)
            for bt in (1, 3, 3.5, 3.75):
                s.note(ch, b + bt, 0.25, 38, 85 if bt != 1 else 100)
        elif style == "light":
            for i in range(s.beats * 2):
                s.note(ch, b + i * 0.5, 0.25, 42, 45)
            s.note(ch, b, 0.25, 36, 80)
            if s.beats == 4:
                s.note(ch, b + 2, 0.25, 38, 60)
        elif style == "toms":
            for bt, n in ((0, 41), (1, 45), (1.5, 45), (2, 41), (3, 48), (3.5, 45)):
                if bt < s.beats:
                    s.note(ch, b + bt, 0.25, n, 90)
        elif style == "timpani":
            s.note(ch, b, 0.5, 41, 100)
            if bar % 2 == 1:
                s.note(ch, b + s.beats - 1, 0.5, 43, 80)
        elif style == "tabor":
            hits = ((0, 45, 100), (1.5, 47, 70), (2, 45, 90), (3, 47, 70), (3.5, 47, 60)) if s.beats == 4 else ((0, 45, 100), (1, 47, 70), (2, 47, 70), (2.5, 47, 55))
            for bt, n, v in hits:
                s.note(ch, b + bt, 0.25, n, v)
        elif style == "estampie":
            for bt, n, v in ((0, 45, 105), (1, 47, 75), (1.5, 47, 60), (2, 45, 85)) if s.beats == 3 else ((0, 45, 105), (1, 47, 75), (2, 45, 90), (2.5, 47, 60), (3, 47, 75)):
                s.note(ch, b + bt, 0.25, n, v)
            for i in range(s.beats * 2):
                if i % 2:
                    s.note(ch, b + i * 0.5, 0.25, 54, 70)
        elif style == "war":
            for bt in (0, 2):
                s.note(ch, b + bt, 0.5, 36, 115)
            for bt in (1, 1.5, 3):
                s.note(ch, b + bt, 0.25, 41, 95)
            if bar % 2 == 1:
                for i in range(8):
                    s.note(ch, b + 2 + i * 0.25, 0.25, 38, 50 + i * 8)
            else:
                s.note(ch, b + 3.5, 0.25, 38, 80)
        elif style == "gallop":
            for bt in range(s.beats):
                for off, v in ((0, 100), (0.5, 70), (0.75, 80)):
                    s.note(ch, b + bt + off, 0.25, 41 if off == 0 else 45, v)
        elif style == "darbuka":
            for bt, n, v in ((0, 41, 105), (0.5, 76, 70), (1.5, 76, 80), (2, 41, 95), (3, 76, 85), (3.5, 54, 60)):
                if bt < s.beats:
                    s.note(ch, b + bt, 0.25, n, v)
        elif style == "blocks":
            for bt in range(s.beats):
                s.note(ch, b + bt, 0.25, 76 if bt == 0 else 77, 90 if bt == 0 else 65)
                s.note(ch, b + bt + 0.5, 0.25, 77, 45)
        elif style == "dirge":
            s.note(ch, b, 1, 36, 100)
            if s.beats == 4:
                s.note(ch, b + 2.5, 0.5, 41, 60)
        if bar == 0 and style in ("battle", "timpani"):
            s.note(ch, b, 1, 49, 90)


def song(name, bpm, key, mode, prog, parts, beats=4, repeat=1, seed=None, modal=False):
    s = Song(bpm, key, mode, beats)
    r = random.Random(seed or name)
    prog = prog * repeat
    ch = 0
    for kind, *args in parts:
        if kind == "drums":
            drums(s, len(prog), args[0])
            continue
        prog_no, octave = args[0], args[1]
        c = s.ch(ch, P[prog_no], args[2] if len(args) > 2 else 100, [64, 40, 88, 56, 72, 30, 98][ch % 7])
        ch += 1 if ch != 8 else 2
        if kind == "melody":
            melody(s, r, c, prog, octave, modal=modal)
        elif kind == "organum":
            # The melody, and the same a fourth below on its own channel.
            state = r.getstate()
            melody(s, r, c, prog, octave, modal=True)
            r.setstate(state)
            c2 = s.ch(ch, P[prog_no], (args[2] if len(args) > 2 else 100) - 15, 40)
            ch += 1 if ch != 8 else 2
            melody(s, r, c2, prog, octave, vel=84, modal=True, shift=-3)
        elif kind == "drone":
            drone(s, c, prog, octave)
        elif kind == "fifths":
            fifths(s, c, prog, octave)
        elif kind == "lute":
            lute(s, c, prog, octave)
        elif kind == "bells":
            bells(s, c, prog, octave, args[3] if len(args) > 3 else 2)
        elif kind == "pads":
            pads(s, c, prog, octave)
        elif kind == "arps":
            arps(s, c, prog, octave, args[3] if len(args) > 3 else 0.5)
        elif kind == "bass":
            bass(s, c, prog, octave, args[3] if len(args) > 3 else "root")
        elif kind == "stabs":
            stabs(s, c, prog, octave, args[3])
    s.end = len(prog) * beats
    return s


# Progressions as scale degrees, one per bar (0 tonic).
TUNES = {
    "menu": (72, 50, "minor", [0, 5, 2, 6, 0, 5, 6, 4], [("pads", "strings", 0), ("melody", "horn", 1), ("arps", "harp", 1, 90, 0.5), ("bass", "bass", -1), ("drums", "timpani")], 4, 2),
    "battle": (138, 52, "minor", [0, 5, 6, 4, 0, 5, 3, 4], [("bass", "synbass", -1, 100, "eighths"), ("stabs", "brass", 0, 100, [0, 1.5, 3]), ("pads", "tremolo", 1, 70), ("melody", "trumpet", 1), ("drums", "battle")], 4, 2),
    "news": (118, 60, "major", [0, 3, 4, 0, 5, 3, 4, 4], [("arps", "piano", 1, 90, 0.5), ("melody", "lead", 1), ("bass", "bass", -1, 100, "walk"), ("drums", "light")], 4, 1),
    "ground": (108, 57, "minor", [0, 0, 5, 4, 0, 3, 4, 4], [("pads", "strings", 0, 80), ("melody", "horn", 0), ("bass", "tuba", -1), ("drums", "march")], 4, 2),
    "ground_win": (100, 60, "major", [0, 3, 4, 0], [("stabs", "brass", 0, 110, [0, 1, 2]), ("melody", "trumpet", 1), ("bass", "tuba", -1), ("drums", "timpani")], 4, 1),
    "ground_lose": (66, 57, "minor", [0, 5, 3, 4, 0], [("pads", "strings", 0, 80), ("melody", "horn", 0), ("bass", "bass", -1)], 4, 1),
    "council": (64, 55, "major", [0, 3, 4, 0, 5, 1, 4, 0], [("pads", "organ", 0, 90), ("melody", "choir", 1), ("bass", "organ", -1)], 4, 2),
    "ending_win": (92, 60, "major", [0, 4, 5, 3, 0, 3, 4, 0], [("pads", "strings", 0), ("melody", "brass", 1), ("arps", "harp", 1, 80, 0.5), ("bass", "tuba", -1), ("drums", "timpani")], 4, 2),
    "ending_lose": (58, 48, "minor", [0, 5, 3, 4, 0, 5, 6, 0], [("pads", "strings", 0, 80), ("melody", "reed", 1), ("bass", "bass", -1)], 4, 1),
    "colonize": (100, 55, "major", [0, 4, 5, 3, 0, 3, 4, 0], [("pads", "strings", 0, 80), ("melody", "horn", 1), ("arps", "harp", 1, 80, 0.5), ("bass", "bass", -1)], 4, 1),
    "bomb": (88, 50, "minor", [0, 0, 5, 4, 0, 6, 4, 4], [("stabs", "brass", 0, 100, [0, 2.5]), ("pads", "tremolo", 0, 70), ("bass", "tuba", -1), ("drums", "timpani")], 4, 1),
    "bomb_destroyed": (60, 48, "harmonic", [0, 5, 3, 4, 0, 5, 4, 0], [("pads", "strings", 0, 80), ("melody", "choir", 1), ("bass", "bass", -1), ("drums", "timpani")], 4, 1),
    "espionage": (112, 50, "dorian", [0, 0, 3, 3, 0, 0, 4, 3], [("bass", "bass", -1, 100, "walk"), ("arps", "vibes", 1, 70, 0.5), ("melody", "reed", 1), ("drums", "light")], 4, 1),
    "sabotage": (124, 52, "phrygian", [0, 1, 0, 6, 0, 1, 4, 0], [("bass", "synbass", -1, 100, "eighths"), ("stabs", "strings", 0, 90, [0, 1.5, 3]), ("melody", "lead", 1), ("drums", "battle")], 4, 1),
    "sabotage_done": (96, 50, "minor", [0, 5, 4, 0], [("stabs", "brass", 0, 110, [0, 1, 2]), ("melody", "trumpet", 1), ("bass", "tuba", -1), ("drums", "timpani")], 4, 1),
}
# Audiences, by race (MOO1 order): a short loop in each race's character.
AUDIENCE = [
    (96, 53, "major", [0, 3, 4, 0], [("pads", "strings", 0), ("melody", "horn", 1), ("bass", "bass", -1)]),  # human
    (108, 52, "phrygian", [0, 1, 0, 6], [("arps", "sitar", 1, 90, 0.5), ("melody", "reed", 1), ("drums", "toms")]),  # mrrshan
    (70, 48, "whole", [0, 1, 2, 1], [("pads", "pad", 0), ("arps", "vibes", 1, 80, 1), ("melody", "celesta", 2)]),  # silicoid
    (100, 45, "minor", [0, 6, 5, 4], [("bass", "tuba", 0), ("melody", "brass", 1), ("drums", "toms")]),  # sakkra
    (80, 55, "lydian", [0, 1, 0, 4], [("pads", "warm", 0), ("arps", "celesta", 1, 70, 0.5), ("melody", "lead", 1)]),  # psilon
    (112, 55, "mixolydian", [0, 6, 3, 0], [("arps", "harp", 0, 80, 0.5), ("melody", "flute", 2), ("bass", "bass", -1)]),  # alkari
    (126, 50, "dorian", [0, 3, 0, 6], [("arps", "marimba", 1, 90, 0.25), ("bass", "synbass", -1, 100, "eighths"), ("drums", "light")]),  # klackon
    (88, 43, "minor", [0, 3, 4, 0], [("bass", "tuba", 0), ("stabs", "brass", 1, 100, [0, 2]), ("melody", "horn", 1), ("drums", "march")]),  # bulrathi
    (120, 52, "minor", [0, 5, 6, 0], [("bass", "synbass", -1, 100, "eighths"), ("arps", "saw", 1, 70, 0.25), ("melody", "lead", 1), ("drums", "light")]),  # meklar
    (66, 50, "harmonic", [0, 5, 4, 0], [("pads", "pad", 0), ("melody", "strings", 1), ("bass", "bass", -1)]),  # darlok
]


# The Age theme: the same screens, medieval. Keys as MIDI notes (50 is D).
AGE_TUNES = {
    "menu": (76, 50, "dorian", [0, 6, 2, 6, 0, 3, 4, 0], [("drone", "drone", -1, 60), ("lute", "lute", 0, 85), ("melody", "recorder", 1), ("bass", "viol", -1, 80), ("drums", "tabor")], 4, 2),
    "battle": (132, 52, "minor", [0, 0, 6, 6, 5, 5, 6, 0], [("drone", "drone", -1, 70), ("stabs", "sackbut", 0, 100, [0, 1.5, 3]), ("melody", "shawm", 1), ("bass", "viol", -1, 100, "eighths"), ("drums", "war")], 4, 2),
    "news": (120, 55, "mixolydian", [0, 6, 0, 3, 0, 6, 4, 0], [("fifths", "sackbut", -1, 70), ("melody", "trumpet", 1), ("lute", "lute", 0, 75), ("drums", "tabor")], 4, 1),
    "ground": (104, 45, "minor", [0, 0, 5, 6, 0, 3, 4, 0], [("drone", "drone", 0, 65), ("melody", "sackbut", 1), ("bass", "viol", 0, 90, "root"), ("drums", "war")], 4, 2),
    "ground_win": (108, 50, "mixolydian", [0, 3, 6, 0], [("fifths", "sackbut", 0, 100), ("melody", "trumpet", 1), ("drums", "estampie")], 4, 1),
    "ground_lose": (60, 57, "minor", [0, 5, 3, 4, 0], [("drone", "drone", -1, 55), ("melody", "recorder", 1), ("bells", "bells", 0, 70, 2)], 4, 1),
    "council": (60, 50, "dorian", [0, 3, 0, 6, 0, 3, 4, 0], [("drone", "church", -1, 70), ("organum", "choir", 1, 95), ("bells", "bells", 0, 60, 4)], 4, 2),
    "ending_win": (116, 55, "mixolydian", [0, 6, 0, 3, 0, 6, 4, 0], [("drone", "drone", -1, 60), ("melody", "shawm", 1), ("lute", "lute", 0, 80), ("fifths", "sackbut", -1, 70), ("drums", "estampie")], 3, 2),
    "ending_lose": (52, 52, "phrygian", [0, 1, 0, 6, 0, 5, 1, 0], [("drone", "church", -1, 65), ("organum", "choir", 1, 90), ("bells", "bells", -1, 80, 2), ("drums", "dirge")], 4, 1),
    "colonize": (96, 53, "major", [0, 4, 3, 0, 5, 3, 4, 0], [("drone", "drone", -1, 60), ("melody", "recorder", 1), ("lute", "lute", 0, 80), ("drums", "estampie")], 3, 1),
    "bomb": (84, 52, "phrygian", [0, 0, 1, 0, 6, 6, 1, 0], [("drone", "drone", -1, 70), ("stabs", "sackbut", 0, 105, [0, 2.5]), ("melody", "shawm", 0), ("drums", "war")], 4, 1),
    "bomb_destroyed": (56, 45, "minor", [0, 5, 3, 4, 0, 5, 4, 0], [("drone", "church", 0, 65), ("melody", "choir", 1), ("bells", "bells", 0, 85, 2), ("drums", "dirge")], 4, 1),
    "espionage": (108, 50, "dorian", [0, 0, 3, 3, 0, 0, 4, 3], [("lute", "lute", 0, 75), ("melody", "recorder", 1), ("bass", "viol", -1, 85, "walk"), ("drums", "blocks")], 4, 1),
    "sabotage": (120, 52, "phrygian", [0, 1, 0, 6, 0, 1, 4, 0], [("drone", "drone", -1, 65), ("arps", "fiddle", 1, 80, 0.25), ("melody", "shawm", 1), ("drums", "war")], 4, 1),
    "sabotage_done": (96, 45, "minor", [0, 5, 4, 0], [("stabs", "sackbut", 0, 110, [0, 1, 2]), ("melody", "trumpet", 1), ("drums", "war")], 4, 1),
}
# Audiences, by race in MOO1 order, each in the character of Age's peoples.
AGE_AUDIENCE = [
    (84, 55, "major", [0, 3, 4, 0], [("drone", "church", -1, 70), ("organum", "choir", 1, 90), ("fifths", "sackbut", -1, 60)]),  # Imperials (Holy Roman Empire)
    (126, 50, "penta", [0, 0, 3, 4], [("drone", "drone", -1, 70), ("melody", "fiddle", 1), ("drums", "gallop")]),  # Mongols
    (96, 52, "hijaz", [0, 1, 0, 6], [("drone", "drone", -1, 60), ("arps", "sitar", 1, 85, 0.5), ("melody", "pan", 1), ("drums", "darbuka")]),  # Ghurids
    (92, 48, "penta", [0, 3, 1, 0], [("arps", "marimba", 1, 85, 0.5), ("melody", "flute", 2), ("bells", "bells", 0, 70, 2), ("drums", "blocks")]),  # Khmer
    (88, 50, "hijaz", [0, 1, 6, 0], [("lute", "lute", 0, 80), ("melody", "pan", 1), ("drums", "darbuka")]),  # Abbasids
    (100, 45, "minor", [0, 6, 5, 6], [("drone", "drone", 0, 70), ("melody", "horn", 1), ("arps", "lute", 0, 70, 1), ("drums", "war")]),  # Danes
    (84, 55, "penta", [0, 2, 3, 0], [("arps", "koto", 0, 80, 0.5), ("melody", "flute", 1), ("drums", "blocks")]),  # Song
    (132, 50, "hijaz", [0, 1, 0, 4], [("drone", "drone", -1, 70), ("melody", "shawm", 1), ("drums", "tabor")]),  # Bulgars
    (60, 52, "phrygian", [0, 1, 6, 0], [("drone", "church", -1, 75), ("organum", "choir", 1, 95)]),  # Byzantines
    (100, 50, "dorian", [0, 6, 3, 4], [("lute", "lute", 0, 80), ("melody", "recorder", 1), ("bass", "viol", -1, 80), ("drums", "tabor")]),  # Venetians
]


def main():
    for theme, tunes, audience in (("classic", TUNES, AUDIENCE), ("age", AGE_TUNES, AGE_AUDIENCE)):
        age = theme == "age"
        out = os.path.join(ROOT, f"themes/{theme}/audio/music")
        os.makedirs(os.path.join(out, "audience"), exist_ok=True)
        for name, (bpm, key, mode, prog, parts, beats, rep) in tunes.items():
            with open(os.path.join(out, f"{name}.mid"), "wb") as f:
                f.write(song(name, bpm, key, mode, prog, parts, beats, rep, f"age-{name}" if age else None, age).bytes())
        for r, (bpm, key, mode, prog, parts) in enumerate(audience):
            with open(os.path.join(out, f"audience/{r}.mid"), "wb") as f:
                f.write(song(f"audience{r}", bpm, key, mode, prog, parts, 4, 2, f"age-audience{r}" if age else None, age).bytes())
        print(f"{len(tunes) + len(audience)} tunes in themes/{theme}/audio/music")


if __name__ == "__main__":
    main()
