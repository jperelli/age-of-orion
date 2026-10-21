// Music and sound effects, synthesized with WebAudio.
//
// Music: the tunes of MUSIC.LBX, converted to MIDI by the importer (crates/lbx/src/music.rs),
// played by a small General MIDI style synthesizer (no SoundFont needed). Sound effects
// are procedural. Nothing plays until the first click or key press (browser autoplay rules).
import * as store from "./storage";
import { RACES, type Race } from "./types";

export interface AudioPrefs {
  music: number;
  sfx: number;
}

// 1oom ui/classic: which tune each screen plays.
export const TUNE = {
  menu: 1,
  ending_win: 2,
  ending_lose: 3,
  battle: 8,
  news: 9,
  ground: 0x26,
  ground_win: 0xc,
  ground_lose: 0xb,
  council: 0x27,
} as const;

/** 1oom `tbl_audience_music`, by race (MOO1 order, as `RACES`). */
export const AUDIENCE_TUNE = [0x16, 0x14, 0x22, 0x20, 0x1c, 0x12, 0x18, 0x24, 0x1a, 0x1e];

export function audienceTune(race: Race): number {
  return AUDIENCE_TUNE[RACES.indexOf(race)] ?? AUDIENCE_TUNE[0];
}

// ------------------------------------------------------------- MIDI

interface MidiEvent {
  time: number;
  data: number[];
}

interface Tune {
  events: MidiEvent[];
  length: number;
}

function readVar(d: Uint8Array, p: { i: number }): number {
  let v = 0;
  for (let k = 0; k < 4; k++) {
    const b = d[p.i++];
    v = (v << 7) | (b & 0x7f);
    if (!(b & 0x80)) break;
  }
  return v;
}

/** Parses a standard MIDI file into channel events with times in seconds. */
export function parseMidi(d: Uint8Array): Tune {
  const u32 = (i: number) => (d[i] << 24) | (d[i + 1] << 16) | (d[i + 2] << 8) | d[i + 3];
  const tracks = (d[10] << 8) | d[11];
  const division = (d[12] << 8) | d[13];
  const raw: { tick: number; data: number[]; tempo?: number }[] = [];
  let i = 8 + u32(4);
  for (let t = 0; t < tracks && i < d.length; t++) {
    const len = u32(i + 4);
    const p = { i: i + 8 };
    const end = p.i + len;
    let tick = 0;
    let running = 0;
    while (p.i < end) {
      tick += readVar(d, p);
      let st = d[p.i];
      if (st & 0x80) p.i++;
      else st = running;
      running = st;
      if (st === 0xff) {
        const type = d[p.i++];
        const l = readVar(d, p);
        if (type === 0x51) raw.push({ tick, data: [], tempo: (d[p.i] << 16) | (d[p.i + 1] << 8) | d[p.i + 2] });
        p.i += l;
        if (type === 0x2f) break;
      } else if (st === 0xf0 || st === 0xf7) {
        p.i += readVar(d, p);
      } else {
        const n = (st & 0xf0) === 0xc0 || (st & 0xf0) === 0xd0 ? 1 : 2;
        raw.push({ tick, data: [st, ...d.subarray(p.i, p.i + n)] });
        p.i += n;
      }
    }
    i = end;
  }
  raw.sort((a, b) => a.tick - b.tick);
  let tempo = 500000;
  let lastTick = 0;
  let time = 0;
  const events: MidiEvent[] = [];
  for (const e of raw) {
    time += ((e.tick - lastTick) * tempo) / 1e6 / division;
    lastTick = e.tick;
    if (e.tempo) tempo = e.tempo;
    else events.push({ time, data: e.data });
  }
  return { events, length: time };
}

// ------------------------------------------------------------- synthesizer

interface Voice {
  ch: number;
  note: number;
  osc: OscillatorNode[];
  gain: GainNode;
  release: number;
  start: number;
}

interface Instrument {
  wave: OscillatorType;
  attack: number;
  decay: number;
  sustain: number;
  release: number;
  detune?: number;
  filter?: number;
}

/** A rough timbre per General MIDI family (program / 8). */
const FAMILIES: Instrument[] = [
  { wave: "triangle", attack: 0.005, decay: 0.9, sustain: 0.15, release: 0.3 }, // piano
  { wave: "sine", attack: 0.002, decay: 0.6, sustain: 0.0, release: 0.4 }, // chromatic percussion
  { wave: "square", attack: 0.02, decay: 0.1, sustain: 0.7, release: 0.1, filter: 2500 }, // organ
  { wave: "sawtooth", attack: 0.004, decay: 0.5, sustain: 0.1, release: 0.2, filter: 3000 }, // guitar
  { wave: "triangle", attack: 0.005, decay: 0.4, sustain: 0.5, release: 0.1 }, // bass
  { wave: "sawtooth", attack: 0.12, decay: 0.2, sustain: 0.8, release: 0.4, detune: 7, filter: 2200 }, // strings
  { wave: "sawtooth", attack: 0.1, decay: 0.3, sustain: 0.7, release: 0.5, detune: 6, filter: 1800 }, // ensemble
  { wave: "sawtooth", attack: 0.03, decay: 0.2, sustain: 0.7, release: 0.15, filter: 2000 }, // brass
  { wave: "square", attack: 0.03, decay: 0.2, sustain: 0.6, release: 0.15, filter: 1800 }, // reed
  { wave: "sine", attack: 0.04, decay: 0.2, sustain: 0.7, release: 0.2 }, // pipe
  { wave: "square", attack: 0.01, decay: 0.2, sustain: 0.6, release: 0.15, filter: 3500 }, // synth lead
  { wave: "sawtooth", attack: 0.3, decay: 0.5, sustain: 0.7, release: 0.8, detune: 9, filter: 1500 }, // pad
  { wave: "triangle", attack: 0.2, decay: 0.5, sustain: 0.5, release: 0.8, detune: 12 }, // synth fx
  { wave: "sawtooth", attack: 0.01, decay: 0.4, sustain: 0.3, release: 0.3, filter: 2500 }, // ethnic
  { wave: "sine", attack: 0.002, decay: 0.3, sustain: 0.0, release: 0.2 }, // percussive
  { wave: "triangle", attack: 0.05, decay: 0.3, sustain: 0.4, release: 0.4 }, // sound effects
];

const MAX_VOICES = 40;

class Synth {
  private voices: Voice[] = [];
  private program = new Array(16).fill(0);
  private volume = new Array(16).fill(100);
  private expression = new Array(16).fill(127);
  private bend = new Array(16).fill(0);
  private pan: StereoPannerNode[] = [];
  private noise: AudioBuffer;

  constructor(
    private ctx: AudioContext,
    out: AudioNode,
  ) {
    for (let c = 0; c < 16; c++) {
      const p = ctx.createStereoPanner();
      p.connect(out);
      this.pan.push(p);
    }
    this.noise = ctx.createBuffer(1, ctx.sampleRate, ctx.sampleRate);
    const data = this.noise.getChannelData(0);
    for (let i = 0; i < data.length; i++) data[i] = Math.random() * 2 - 1;
  }

  reset(): void {
    this.allOff(this.ctx.currentTime);
    this.program.fill(0);
    this.volume.fill(100);
    this.expression.fill(127);
    this.bend.fill(0);
  }

  allOff(t: number): void {
    for (const v of this.voices) this.stopVoice(v, t, 0.05);
    this.voices = [];
  }

  event(data: number[], t: number): void {
    const st = data[0] & 0xf0;
    const ch = data[0] & 0x0f;
    if (st === 0x90 && data[2] > 0) this.noteOn(ch, data[1], data[2], t);
    else if (st === 0x80 || st === 0x90) this.noteOff(ch, data[1], t);
    else if (st === 0xc0) this.program[ch] = data[1];
    else if (st === 0xb0) {
      if (data[1] === 7) this.volume[ch] = data[2];
      else if (data[1] === 11) this.expression[ch] = data[2];
      else if (data[1] === 10) this.pan[ch].pan.setValueAtTime((data[2] - 64) / 64, t);
      else if (data[1] === 123 || data[1] === 120) for (const v of this.voices.filter((v) => v.ch === ch)) this.stopVoice(v, t, 0.05);
    } else if (st === 0xe0) {
      this.bend[ch] = (((data[2] << 7) | data[1]) - 8192) / 8192;
      for (const v of this.voices) if (v.ch === ch) for (const o of v.osc) o.detune.setValueAtTime(this.bend[ch] * 200, t);
    }
  }

  private level(ch: number, vel: number): number {
    return (vel / 127) * (this.volume[ch] / 127) * (this.expression[ch] / 127);
  }

  private noteOn(ch: number, note: number, vel: number, t: number): void {
    if (this.voices.length >= MAX_VOICES) {
      const old = this.voices.shift()!;
      this.stopVoice(old, t, 0.02);
    }
    if (ch === 9) {
      this.drum(note, this.level(ch, vel), t);
      return;
    }
    const ins = FAMILIES[this.program[ch] >> 3];
    const freq = 440 * Math.pow(2, (note - 69) / 12);
    const gain = this.ctx.createGain();
    const peak = 0.16 * this.level(ch, vel);
    gain.gain.setValueAtTime(0, t);
    gain.gain.linearRampToValueAtTime(peak, t + ins.attack);
    gain.gain.setTargetAtTime(peak * ins.sustain, t + ins.attack, ins.decay / 3);
    let dest: AudioNode = gain;
    if (ins.filter) {
      const f = this.ctx.createBiquadFilter();
      f.type = "lowpass";
      f.frequency.value = ins.filter;
      gain.connect(f);
      dest = f;
    }
    dest.connect(this.pan[ch]);
    const osc: OscillatorNode[] = [];
    for (const d of ins.detune ? [-ins.detune, ins.detune] : [0]) {
      const o = this.ctx.createOscillator();
      o.type = ins.wave;
      o.frequency.value = freq;
      o.detune.value = d + this.bend[ch] * 200;
      o.connect(gain);
      o.start(t);
      osc.push(o);
    }
    this.voices.push({ ch, note, osc, gain, release: ins.release, start: t });
  }

  private noteOff(ch: number, note: number, t: number): void {
    const i = this.voices.findIndex((v) => v.ch === ch && v.note === note);
    if (i < 0) return;
    const [v] = this.voices.splice(i, 1);
    this.stopVoice(v, t, v.release);
  }

  private stopVoice(v: Voice, t: number, release: number): void {
    const at = Math.max(t, v.start);
    v.gain.gain.cancelScheduledValues(at);
    v.gain.gain.setTargetAtTime(0, at, release / 3);
    for (const o of v.osc) o.stop(at + release * 2 + 0.05);
  }

  /** General MIDI percussion, roughly. */
  private drum(note: number, level: number, t: number): void {
    const g = this.ctx.createGain();
    g.connect(this.pan[9]);
    const kick = note === 35 || note === 36;
    const tom = [41, 43, 45, 47, 48, 50].includes(note);
    if (kick || tom) {
      const o = this.ctx.createOscillator();
      const f0 = kick ? 120 : 90 + (note - 41) * 18;
      o.frequency.setValueAtTime(f0, t);
      o.frequency.exponentialRampToValueAtTime(f0 * 0.35, t + 0.2);
      g.gain.setValueAtTime(0.5 * level, t);
      g.gain.exponentialRampToValueAtTime(0.001, t + 0.3);
      o.connect(g);
      o.start(t);
      o.stop(t + 0.35);
      return;
    }
    const src = this.ctx.createBufferSource();
    src.buffer = this.noise;
    const f = this.ctx.createBiquadFilter();
    const hat = [42, 44, 46].includes(note);
    const cymbal = [49, 51, 52, 55, 57, 59].includes(note);
    f.type = hat || cymbal ? "highpass" : "bandpass";
    f.frequency.value = hat ? 7000 : cymbal ? 5000 : 1800;
    const len = note === 46 ? 0.25 : hat ? 0.06 : cymbal ? 0.9 : 0.18;
    g.gain.setValueAtTime((hat ? 0.12 : 0.3) * level, t);
    g.gain.exponentialRampToValueAtTime(0.001, t + len);
    src.connect(f);
    f.connect(g);
    src.start(t);
    src.stop(t + len + 0.05);
  }
}

// ------------------------------------------------------------- player

class AudioOut {
  ctx: AudioContext | null = null;
  private musicGain: GainNode | null = null;
  private sfxGain: GainNode | null = null;
  private synth: Synth | null = null;
  private prefs: AudioPrefs = { music: 0.5, sfx: 0.5 };
  private tune: Tune | null = null;
  private tuneId = -1;
  private loops = false;
  private next = 0;
  private startAt = 0;
  private timer: number | null = null;
  private manifest: { aliases: Record<string, string>; looping?: string[] } | null | undefined;
  private wanted = -1;

  setPrefs(p: AudioPrefs): void {
    this.prefs = p;
    if (this.musicGain) this.musicGain.gain.value = p.music;
    if (this.sfxGain) this.sfxGain.gain.value = p.sfx;
    if (p.music === 0) this.stopMusic();
    else if (this.wanted >= 0 && this.tuneId !== this.wanted) void this.play(this.wanted);
  }

  /** Creates the audio context on the first user gesture. */
  unlock(): void {
    if (!this.ctx) {
      try {
        this.ctx = new AudioContext();
      } catch {
        return;
      }
      this.musicGain = this.ctx.createGain();
      this.musicGain.gain.value = this.prefs.music;
      this.musicGain.connect(this.ctx.destination);
      this.sfxGain = this.ctx.createGain();
      this.sfxGain.gain.value = this.prefs.sfx;
      this.sfxGain.connect(this.ctx.destination);
      this.synth = new Synth(this.ctx, this.musicGain);
      if (this.wanted >= 0) void this.play(this.wanted);
      this.preloadSamples([0x24, 2, 0x11, 0x13, 0x25, 6]);
    }
    if (this.ctx.state === "suspended") void this.ctx.resume();
  }

  /** Forgets the cached asset list (after an import). */
  reload(): void {
    this.manifest = undefined;
    this.samples.clear();
  }

  async hasMusic(): Promise<boolean> {
    this.manifest ??= (await store.get<{ aliases: Record<string, string> }>("assets", "manifest.json")) ?? null;
    return !!this.manifest && Object.keys(this.manifest.aliases).some((a) => a.startsWith("music/"));
  }

  /** Plays MUSIC.LBX tune `n` (if imported); the same tune keeps playing. */
  async play(n: number): Promise<void> {
    this.wanted = n;
    if (!this.ctx || !this.synth || this.prefs.music === 0 || this.tuneId === n) return;
    this.manifest ??= (await store.get<{ aliases: Record<string, string>; looping?: string[] }>("assets", "manifest.json")) ?? null;
    const path = this.manifest?.aliases[`music/${n}`];
    if (!path) return this.stopMusic();
    const blob = await store.get<Blob>("assets", path);
    if (!blob || this.wanted !== n) return;
    const tune = parseMidi(new Uint8Array(await blob.arrayBuffer()));
    this.stopMusic();
    this.wanted = n;
    this.tune = tune;
    this.tuneId = n;
    this.loops = !!this.manifest?.looping?.includes(`music/${n}`);
    this.synth.reset();
    this.next = 0;
    this.startAt = this.ctx.currentTime + 0.1;
    this.timer = window.setInterval(() => this.pump(), 50);
    this.pump();
  }

  stopMusic(): void {
    if (this.timer !== null) clearInterval(this.timer);
    this.timer = null;
    this.tune = null;
    this.tuneId = -1;
    if (this.ctx && this.synth) this.synth.allOff(this.ctx.currentTime);
  }

  private pump(): void {
    const ctx = this.ctx;
    const tune = this.tune;
    if (!ctx || !tune || !this.synth) return;
    const horizon = ctx.currentTime + 0.25;
    while (true) {
      if (this.next >= tune.events.length) {
        if (!this.loops || tune.length <= 0) {
          if (ctx.currentTime > this.startAt + tune.length + 1) this.stopMusic();
          return;
        }
        this.startAt += tune.length;
        this.next = 0;
      }
      const e = tune.events[this.next];
      const t = this.startAt + e.time;
      if (t > horizon) return;
      this.synth.event(e.data, Math.max(t, ctx.currentTime));
      this.next++;
    }
  }

  // --------------------------------------------------------- effects

  private env(dur: number, peak: number): GainNode | null {
    if (!this.ctx || !this.sfxGain || this.prefs.sfx === 0) return null;
    const g = this.ctx.createGain();
    const t = this.ctx.currentTime;
    g.gain.setValueAtTime(peak, t);
    g.gain.exponentialRampToValueAtTime(0.001, t + dur);
    g.connect(this.sfxGain);
    return g;
  }

  private tone(type: OscillatorType, f0: number, f1: number, dur: number, peak: number): void {
    const g = this.env(dur, peak);
    if (!g || !this.ctx) return;
    const o = this.ctx.createOscillator();
    const t = this.ctx.currentTime;
    o.type = type;
    o.frequency.setValueAtTime(f0, t);
    o.frequency.exponentialRampToValueAtTime(Math.max(20, f1), t + dur);
    o.connect(g);
    o.start(t);
    o.stop(t + dur + 0.02);
  }

  private noise(dur: number, peak: number, freq: number, type: BiquadFilterType = "lowpass"): void {
    const g = this.env(dur, peak);
    if (!g || !this.ctx) return;
    const ctx = this.ctx;
    const buf = ctx.createBuffer(1, Math.ceil(ctx.sampleRate * dur), ctx.sampleRate);
    const d = buf.getChannelData(0);
    for (let i = 0; i < d.length; i++) d[i] = Math.random() * 2 - 1;
    const src = ctx.createBufferSource();
    src.buffer = buf;
    const f = ctx.createBiquadFilter();
    f.type = type;
    f.frequency.value = freq;
    src.connect(f);
    f.connect(g);
    src.start();
  }

  // ------------------------------------------------------ MOO1 samples

  private samplesOn = false;
  private samples = new Map<number, AudioBuffer | null>();

  /** Plays the imported SOUNDFX.LBX samples instead of synthesized effects (Classic theme). */
  useSamples(on: boolean): void {
    this.samplesOn = on;
    this.samples.clear();
  }

  /** Plays SOUNDFX.LBX sample `n` (1oom `ui_sound_play_sfx`); false if not available. */
  sample(n: number): boolean {
    if (!this.samplesOn || !this.ctx || !this.sfxGain || this.prefs.sfx === 0) return false;
    const buf = this.samples.get(n);
    if (buf === undefined) {
      this.samples.set(n, null);
      void this.loadSample(n);
      return false;
    }
    if (!buf) return false;
    const src = this.ctx.createBufferSource();
    src.buffer = buf;
    src.connect(this.sfxGain);
    src.start();
    return true;
  }

  private async loadSample(n: number): Promise<void> {
    this.manifest ??= (await store.get<{ aliases: Record<string, string> }>("assets", "manifest.json")) ?? null;
    const path = this.manifest?.aliases[`sfx/${n}`];
    const blob = path ? await store.get<Blob>("assets", path) : null;
    if (!blob || !this.ctx) return;
    try {
      this.samples.set(n, await this.ctx.decodeAudioData(await blob.arrayBuffer()));
    } catch {
      this.samples.set(n, null);
    }
  }

  /** Preloads the samples the game uses, so the first effect is not silent. */
  preloadSamples(ns: number[]): void {
    if (!this.samplesOn || !this.ctx) return;
    for (const n of ns) if (!this.samples.has(n)) {
      this.samples.set(n, null);
      void this.loadSample(n);
    }
  }

  /**
   * A sound effect: MOO1's sample `sample` when imported (Classic theme), otherwise a
   * synthesized one. 1oom numbers: 0x24 click, 2 hit, 0x11 explosion, 0x13 special,
   * weapons their own (`tbl_shiptech_weap[].sound`).
   */
  sfx(kind: "beam" | "heavy_beam" | "missile" | "explosion" | "hit" | "bomb" | "special" | "click" | "arrow" | "clash", sample?: number): void {
    const def: Partial<Record<typeof kind, number>> = { click: 0x24, hit: 2, explosion: 0x11, special: 0x13, bomb: 0x25 };
    const n = sample ?? def[kind];
    if (n !== undefined && this.sample(n)) return;
    switch (kind) {
      case "beam":
        return this.tone("sawtooth", 1400, 300, 0.18, 0.12);
      case "heavy_beam":
        return this.tone("sawtooth", 600, 80, 0.4, 0.18);
      case "missile":
        return this.noise(0.35, 0.15, 900, "bandpass");
      case "explosion":
        this.noise(0.8, 0.4, 600);
        return this.tone("sine", 90, 30, 0.6, 0.3);
      case "hit":
        return this.noise(0.12, 0.2, 2500, "bandpass");
      case "bomb":
        return this.tone("square", 300, 40, 0.5, 0.15);
      case "special":
        return this.tone("sine", 200, 1600, 0.5, 0.15);
      case "click":
        return this.tone("square", 900, 900, 0.04, 0.05);
      case "arrow":
        return this.noise(0.15, 0.18, 4000, "highpass");
      case "clash":
        this.tone("square", 1800, 1200, 0.08, 0.08);
        return this.noise(0.1, 0.12, 3000, "bandpass");
    }
  }
}

export const audio = new AudioOut();

for (const ev of ["pointerdown", "keydown"]) window.addEventListener(ev, () => audio.unlock(), { capture: true });
