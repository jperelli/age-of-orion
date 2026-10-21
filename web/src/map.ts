// Canvas renderer of the game map: a terrain world with cities (theme map "world") or the
// classic starfield (theme map "space"). Pan with drag, zoom with the wheel.
import type { Theme } from "./theme";
import type { FleetView, MonsterView, PlanetType, StarType, TransportView, View } from "./types";

export type Selection = { kind: "planet"; id: number } | { kind: "fleet"; id: number } | { kind: "transport"; id: number } | null;

export interface Targeting {
  from: { x: number; y: number };
  range: number;
  speed: number;
  ok: (planet: number) => boolean;
  pick: (planet: number) => void;
  /** Text shown in the side panel while choosing. */
  prompt?: string;
  /** Draw the dashed range circles (default true). */
  ranges?: boolean;
  /** Label next to a valid target; `null` shows the travel time. */
  label?: (planet: number) => string | null;
}

const BIOME: Record<PlanetType, string> = {
  not_habitable: "#8b8378",
  radiated: "#7a6a8a",
  toxic: "#6f7d3a",
  inferno: "#a3482e",
  dead: "#8a6b4f",
  tundra: "#c9d6dc",
  barren: "#9a8f7a",
  minimal: "#a7a76a",
  desert: "#e0c27a",
  steppe: "#b7b25c",
  arid: "#c9a95a",
  ocean: "#5fa3b8",
  jungle: "#2f7a3a",
  terran: "#6fae4a",
};

const STAR: Record<StarType, string> = {
  yellow: "#ffe36b",
  red: "#ff6a4d",
  green: "#7dff8a",
  white: "#f4f4ff",
  blue: "#7fb6ff",
  neutron: "#c58cff",
};

/** Deterministic PRNG (mulberry32) for terrain and decorations. */
function prng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function valueNoise(seed: number, w: number, h: number, cell: number): (x: number, y: number) => number {
  const r = prng(seed);
  const gw = Math.ceil(w / cell) + 2;
  const gh = Math.ceil(h / cell) + 2;
  const g = Array.from({ length: gw * gh }, r);
  const s = (t: number) => t * t * (3 - 2 * t);
  return (x, y) => {
    const fx = x / cell, fy = y / cell;
    const ix = Math.floor(fx), iy = Math.floor(fy);
    const tx = s(fx - ix), ty = s(fy - iy);
    const v = (i: number, j: number) => g[(((j % gh) + gh) % gh) * gw + (((i % gw) + gw) % gw)];
    const a = v(ix, iy) + (v(ix + 1, iy) - v(ix, iy)) * tx;
    const b = v(ix, iy + 1) + (v(ix + 1, iy + 1) - v(ix, iy + 1)) * tx;
    return a + (b - a) * ty;
  };
}

export class MapView {
  readonly ctx: CanvasRenderingContext2D;
  view: View | null = null;
  selection: Selection = null;
  targeting: Targeting | null = null;
  onSelect: (s: Selection) => void = () => {};
  private zoom = 1;
  private panX = 0;
  private panY = 0;
  private hover: number | null = null;
  private terrain: HTMLCanvasElement | null = null;
  private terrainKey = "";
  private fog: HTMLCanvasElement | null = null;
  private dirty = true;
  private fitted = false;
  private sized = false;
  private readonly S = 4; // terrain pixels per map unit

  constructor(
    readonly canvas: HTMLCanvasElement,
    private theme: Theme,
  ) {
    this.ctx = canvas.getContext("2d")!;
    this.bind();
    const loop = () => {
      if (this.dirty || this.selection || this.targeting) this.draw();
      requestAnimationFrame(loop);
    };
    requestAnimationFrame(loop);
    new ResizeObserver(() => this.resize()).observe(canvas);
  }

  setView(v: View): void {
    this.view = v;
    if (!this.fitted) this.fit();
    this.dirty = true;
  }

  setTheme(t: Theme): void {
    this.theme = t;
    this.terrainKey = "";
    this.dirty = true;
  }

  centerOn(x: number, y: number): void {
    this.panX = this.canvas.width / 2 - x * this.zoom;
    this.panY = this.canvas.height / 2 - y * this.zoom;
    this.dirty = true;
  }

  private resize(): void {
    const r = this.canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    this.canvas.width = Math.max(1, Math.round(r.width * dpr));
    this.canvas.height = Math.max(1, Math.round(r.height * dpr));
    this.sized = r.width > 10 && r.height > 10;
    if (!this.fitted) this.fit();
    this.dirty = true;
  }

  private fit(): void {
    const v = this.view;
    if (!v || !this.sized) return;
    this.zoom = Math.min(this.canvas.width / v.width, this.canvas.height / v.height);
    this.panX = (this.canvas.width - v.width * this.zoom) / 2;
    this.panY = (this.canvas.height - v.height * this.zoom) / 2;
    this.fitted = true;
  }

  private toMap(sx: number, sy: number): [number, number] {
    return [(sx - this.panX) / this.zoom, (sy - this.panY) / this.zoom];
  }

  private screen(e: PointerEvent | WheelEvent | MouseEvent): [number, number] {
    const r = this.canvas.getBoundingClientRect();
    const dpr = this.canvas.width / r.width;
    return [(e.clientX - r.left) * dpr, (e.clientY - r.top) * dpr];
  }

  private pick(sx: number, sy: number): Selection {
    const v = this.view;
    if (!v) return null;
    const [mx, my] = this.toMap(sx, sy);
    const near = (x: number, y: number, px: number) => Math.hypot(x - mx, y - my) * this.zoom < px;
    const f = v.fleets.find((f) => f.id !== null && f.owner === v.player && near(f.x, f.y, 10));
    if (f && f.id !== null && !this.targeting) return { kind: "fleet", id: f.id };
    const tr = v.transports.find((t) => t.owner === v.player && near(t.x, t.y, 10));
    if (tr && !this.targeting) return { kind: "transport", id: tr.id };
    let best: number | null = null;
    let bd = Infinity;
    for (const p of v.planets) {
      const d = Math.hypot(p.x - mx, p.y - my) * this.zoom;
      if (d < bd && d < 22) {
        bd = d;
        best = p.id;
      }
    }
    return best === null ? null : { kind: "planet", id: best };
  }

  private bind(): void {
    let down: { x: number; y: number; px: number; py: number; moved: boolean } | null = null;
    this.canvas.addEventListener("pointerdown", (e) => {
      const [x, y] = this.screen(e);
      down = { x, y, px: this.panX, py: this.panY, moved: false };
      this.canvas.setPointerCapture(e.pointerId);
    });
    this.canvas.addEventListener("pointermove", (e) => {
      const [x, y] = this.screen(e);
      if (down) {
        if (Math.hypot(x - down.x, y - down.y) > 6) down.moved = true;
        if (down.moved) {
          this.panX = down.px + x - down.x;
          this.panY = down.py + y - down.y;
          this.dirty = true;
        }
      }
      const s = this.pick(x, y);
      const h = s?.kind === "planet" ? s.id : null;
      if (h !== this.hover) {
        this.hover = h;
        this.dirty = true;
      }
    });
    this.canvas.addEventListener("pointerup", (e) => {
      const [x, y] = this.screen(e);
      if (down && !down.moved) {
        const s = this.pick(x, y);
        if (this.targeting) {
          if (s?.kind === "planet" && this.targeting.ok(s.id)) this.targeting.pick(s.id);
        } else {
          this.selection = s;
          this.onSelect(s);
        }
        this.dirty = true;
      }
      down = null;
    });
    this.canvas.addEventListener(
      "wheel",
      (e) => {
        e.preventDefault();
        const [x, y] = this.screen(e);
        const [mx, my] = this.toMap(x, y);
        const v = this.view;
        const min = v ? Math.min(this.canvas.width / v.width, this.canvas.height / v.height) * 0.6 : 0.5;
        this.zoom = Math.min(40, Math.max(min, this.zoom * Math.exp(-e.deltaY * 0.0015)));
        this.panX = x - mx * this.zoom;
        this.panY = y - my * this.zoom;
        this.dirty = true;
      },
      { passive: false },
    );
  }

  // ---------------------------------------------------------------- terrain

  private buildTerrain(v: View): HTMLCanvasElement {
    const S = this.S;
    const W = v.width, H = v.height;
    const R = 2; // land mask samples per map unit
    const seed = v.planets.reduce((a, p) => (a * 31 + p.x * 7 + p.y) >>> 0, 17);
    const n1 = valueNoise(seed, W, H, 22);
    const n2 = valueNoise(seed + 1, W, H, 9);
    const n3 = valueNoise(seed + 2, W, H, 4);
    const forest = valueNoise(seed + 3, W, H, 6);
    const w = W * R, hh = H * R;
    const bump = new Float32Array(w * hh);
    for (const p of v.planets) {
      for (let y = Math.max(0, (p.y - 30) * R); y < Math.min(hh, (p.y + 30) * R); y++) {
        for (let x = Math.max(0, (p.x - 30) * R); x < Math.min(w, (p.x + 30) * R); x++) {
          const d2 = (p.x - x / R) ** 2 + (p.y - y / R) ** 2;
          if (d2 < 900) bump[y * w + x] += 0.55 * Math.exp(-d2 / 260);
        }
      }
    }
    const land = new Float32Array(W * H);
    const small = document.createElement("canvas");
    small.width = w;
    small.height = hh;
    const sctx = small.getContext("2d")!;
    const img = sctx.createImageData(w, hh);
    for (let sy = 0; sy < hh; sy++) {
      for (let sx = 0; sx < w; sx++) {
        const x = sx / R, y = sy / R;
        const n = n1(x, y) * 0.55 + n2(x, y) * 0.3 + n3(x, y) * 0.15;
        const edge = Math.min(x, y, W - x, H - y) / 14;
        const val = n * 0.9 + bump[sy * w + sx] - 0.55 + Math.min(0, edge - 1) * 0.4;
        if (sx % R === 0 && sy % R === 0) land[Math.floor(y) * W + Math.floor(x)] = val;
        let c: [number, number, number];
        if (val < -0.12) c = [38, 84, 116];
        else if (val < 0) c = [62, 118, 146];
        else if (val < 0.04) c = [214, 194, 140];
        else {
          const f = forest(x, y);
          c = val > 0.55 ? [150, 146, 104] : f > 0.66 ? [80, 120, 56] : [112 + f * 30, 156 + f * 18, 78];
        }
        img.data.set([c[0], c[1], c[2], 255], (sy * w + sx) * 4);
      }
    }
    sctx.putImageData(img, 0, 0);
    const big = document.createElement("canvas");
    big.width = W * S;
    big.height = H * S;
    const g = big.getContext("2d")!;
    g.imageSmoothingEnabled = true;
    g.imageSmoothingQuality = "high";
    g.drawImage(small, 0, 0, W * S, H * S);
    const r = prng(seed + 9);
    const at = (x: number, y: number) => land[Math.min(H - 1, Math.max(0, Math.floor(y))) * W + Math.min(W - 1, Math.max(0, Math.floor(x)))];
    const nearSite = (x: number, y: number) => v.planets.some((p) => (p.x - x) ** 2 + (p.y - y) ** 2 < 60);
    for (let k = 0; k < W * H * 0.09; k++) {
      const x = r() * W, y = r() * H;
      const val = at(x, y);
      if (nearSite(x, y)) continue;
      if (val > 0.6 && r() < 0.6) {
        // mountains
        g.fillStyle = "#7d7563";
        g.beginPath();
        g.moveTo(x * S - 7, y * S + 5);
        g.lineTo(x * S, y * S - 8);
        g.lineTo(x * S + 7, y * S + 5);
        g.fill();
        g.fillStyle = "#efeee6";
        g.beginPath();
        g.moveTo(x * S - 2.5, y * S - 3);
        g.lineTo(x * S, y * S - 8);
        g.lineTo(x * S + 2.5, y * S - 3);
        g.fill();
      } else if (val > 0.08 && forest(x, y) > 0.62) {
        // forests
        g.fillStyle = r() > 0.5 ? "#2f5a2a" : "#3b6b30";
        g.beginPath();
        g.arc(x * S, y * S, 2.2 + r() * 1.6, 0, Math.PI * 2);
        g.fill();
      } else if (val < -0.14 && r() < 0.08) {
        g.strokeStyle = "rgba(255,255,255,0.18)";
        g.lineWidth = 1;
        g.beginPath();
        g.moveTo(x * S - 4, y * S);
        g.quadraticCurveTo(x * S, y * S - 2, x * S + 4, y * S);
        g.stroke();
      }
    }
    return big;
  }

  private buildStarfield(v: View): HTMLCanvasElement {
    const S = this.S;
    const c = document.createElement("canvas");
    c.width = v.width * S;
    c.height = v.height * S;
    const g = c.getContext("2d")!;
    g.fillStyle = "#03040a";
    g.fillRect(0, 0, c.width, c.height);
    const r = prng(v.width * 977 + v.height);
    for (let i = 0; i < v.width * v.height * 0.05; i++) {
      const a = r();
      g.fillStyle = `rgba(255,255,255,${0.15 + a * 0.6})`;
      g.fillRect(r() * c.width, r() * c.height, a > 0.95 ? 2 : 1, a > 0.95 ? 2 : 1);
    }
    return c;
  }

  // ---------------------------------------------------------------- drawing

  private draw(): void {
    this.dirty = false;
    const v = this.view;
    const ctx = this.ctx;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.fillStyle = this.theme.def.map === "world" ? "#26506e" : "#000";
    ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    if (!v) return;
    const key = `${this.theme.def.map}:${v.width}:${v.height}:${v.planets.length}:${v.planets[0]?.x}`;
    if (key !== this.terrainKey) {
      this.terrain = this.theme.def.map === "world" ? this.buildTerrain(v) : this.buildStarfield(v);
      this.terrainKey = key;
    }
    const z = this.zoom;
    ctx.setTransform(z, 0, 0, z, this.panX, this.panY);
    const bg = this.theme.img("map/background");
    if (bg) ctx.drawImage(bg, 0, 0, v.width, v.height);
    else if (this.terrain) ctx.drawImage(this.terrain, 0, 0, v.width, v.height);

    const world = this.theme.def.map === "world";
    this.drawNebulae(v);
    if (world) this.drawBiomes(v);
    this.drawFog(v);
    if (this.targeting && this.targeting.ranges !== false) this.drawRanges(v);
    for (const f of v.fleets) if (f.dest !== null) this.drawRoute(v, f.x, f.y, f.dest, this.theme.banner(v.empires[f.owner].banner));
    for (const t of v.transports) this.drawRoute(v, t.x, t.y, t.dest, this.theme.banner(v.empires[t.owner].banner));
    for (const p of v.planets) (world ? this.drawSite : this.drawStar).call(this, v, p.id);
    for (const p of v.planets) this.drawOrbit(v, p.id);
    for (const f of v.fleets) if (f.at === null) this.drawFleet(v, f, f.x, f.y);
    for (const t of v.transports) this.drawCaravan(v, t);
    for (const m of v.monsters ?? []) this.drawMonster(m);
    this.drawSelection(v);
    this.drawLabels(v);
    if (this.targeting && this.hover !== null) this.drawTargetLine(v, this.hover);
  }

  private px(n: number): number {
    return n / this.zoom;
  }

  /** Nebulae: purple clouds in space, drifting mists on the world map. */
  private drawNebulae(v: View): void {
    const ctx = this.ctx;
    const world = this.theme.def.map === "world";
    for (const n of v.nebulae) {
      const img = this.theme.img(`nebula/${n.kind}`);
      if (img) {
        ctx.drawImage(img, n.x, n.y, img.width / 2, img.height / 2);
        continue;
      }
      for (const [x0, y0, x1, y1] of n.rects) {
        const cx = (x0 + x1) / 2, cy = (y0 + y1) / 2;
        const rx = Math.max(4, (x1 - x0) / 2 + 4), ry = Math.max(4, (y1 - y0) / 2 + 4);
        const g = ctx.createRadialGradient(cx, cy, 0, cx, cy, Math.max(rx, ry));
        g.addColorStop(0, world ? "rgba(214,214,232,0.55)" : "rgba(170,90,220,0.45)");
        g.addColorStop(1, world ? "rgba(214,214,232,0)" : "rgba(120,50,190,0)");
        ctx.fillStyle = g;
        ctx.beginPath();
        ctx.ellipse(cx, cy, rx, ry, 0, 0, Math.PI * 2);
        ctx.fill();
      }
    }
  }

  private drawGate(x: number, y: number, u: number): void {
    const ctx = this.ctx;
    const img = this.theme.img("stargate");
    if (img) {
      ctx.drawImage(img, x - u, y - u, u * 2, u * 2);
      return;
    }
    ctx.strokeStyle = this.theme.def.map === "world" ? "#e8d27a" : "#7fd8ff";
    ctx.lineWidth = this.px(2);
    ctx.beginPath();
    ctx.arc(x, y, u * 0.8, Math.PI, 0);
    ctx.lineTo(x + u * 0.8, y + u * 0.6);
    ctx.moveTo(x - u * 0.8, y);
    ctx.lineTo(x - u * 0.8, y + u * 0.6);
    ctx.stroke();
  }

  private drawBiomes(v: View): void {
    const ctx = this.ctx;
    for (const p of v.planets) {
      if (!p.explored || !p.kind) continue;
      const g = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, 11);
      g.addColorStop(0, BIOME[p.kind] + "dd");
      g.addColorStop(1, BIOME[p.kind] + "00");
      ctx.fillStyle = g;
      ctx.beginPath();
      ctx.arc(p.x, p.y, 11, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  /** Darkens everything outside the scanner range of own cities and armies. */
  private drawFog(v: View): void {
    const fog = (this.fog ??= document.createElement("canvas"));
    if (fog.width !== this.canvas.width || fog.height !== this.canvas.height) {
      fog.width = this.canvas.width;
      fog.height = this.canvas.height;
    }
    const g = fog.getContext("2d")!;
    g.setTransform(1, 0, 0, 1, 0, 0);
    g.globalCompositeOperation = "source-over";
    g.clearRect(0, 0, fog.width, fog.height);
    g.fillStyle = this.theme.def.map === "world" ? "rgba(20,16,10,0.34)" : "rgba(0,0,0,0.4)";
    g.fillRect(0, 0, fog.width, fog.height);
    g.setTransform(this.zoom, 0, 0, this.zoom, this.panX, this.panY);
    g.globalCompositeOperation = "destination-out";
    const r = v.abilities.scanner_range * 11;
    const hole = (x: number, y: number, rad: number) => {
      const gr = g.createRadialGradient(x, y, rad * 0.8, x, y, rad);
      gr.addColorStop(0, "rgba(0,0,0,1)");
      gr.addColorStop(1, "rgba(0,0,0,0)");
      g.fillStyle = gr;
      g.beginPath();
      g.arc(x, y, rad, 0, Math.PI * 2);
      g.fill();
    };
    for (const p of v.planets) if (p.owner === v.player) hole(p.x, p.y, r);
    for (const f of v.fleets) if (f.owner === v.player) hole(f.x, f.y, 22);
    const ctx = this.ctx;
    ctx.save();
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.drawImage(fog, 0, 0);
    ctx.restore();
  }

  private drawRanges(v: View): void {
    const t = this.targeting!;
    const ctx = this.ctx;
    ctx.save();
    ctx.fillStyle = "rgba(255,240,180,0.10)";
    ctx.strokeStyle = "rgba(255,240,180,0.55)";
    ctx.lineWidth = this.px(1.5);
    ctx.setLineDash([this.px(6), this.px(4)]);
    ctx.beginPath();
    for (const p of v.planets) {
      if (p.owner !== v.player) continue;
      ctx.moveTo(p.x + t.range * 11, p.y);
      ctx.arc(p.x, p.y, t.range * 11, 0, Math.PI * 2);
    }
    ctx.fill("nonzero");
    ctx.stroke();
    ctx.restore();
  }

  private drawRoute(v: View, x: number, y: number, dest: number, color: string): void {
    const p = v.planets[dest];
    const ctx = this.ctx;
    ctx.save();
    ctx.strokeStyle = color;
    ctx.globalAlpha = 0.8;
    ctx.lineWidth = this.px(1.5);
    ctx.setLineDash([this.px(4), this.px(4)]);
    ctx.beginPath();
    ctx.moveTo(x, y);
    ctx.lineTo(p.x, p.y);
    ctx.stroke();
    ctx.restore();
  }

  private drawSite(v: View, id: number): void {
    const p = v.planets[id];
    const ctx = this.ctx;
    const s = this.px(1);
    const u = Math.max(1.6, Math.min(4, 12 / this.zoom + 1.2));
    if (!p.explored) {
      ctx.fillStyle = "rgba(60,55,45,0.85)";
      ctx.beginPath();
      ctx.arc(p.x, p.y, u * 1.1, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = "#d8cfb8";
      ctx.font = `bold ${u * 1.5}px Georgia, serif`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText("?", p.x, p.y + s * 0.5);
      return;
    }
    const custom = this.theme.img(p.owner !== null ? `site/city` : `site/${p.kind}`);
    if (custom) {
      ctx.drawImage(custom, p.x - u * 2, p.y - u * 2, u * 4, u * 4);
    } else if (p.owner !== null) {
      this.drawCity(p.x, p.y, u, this.theme.banner(v.empires[p.owner].banner), p.pop ?? 0, (p.bases ?? 0) > 0);
    } else if (p.kind === "not_habitable") {
      ctx.fillStyle = "#6e665a";
      ctx.beginPath();
      ctx.moveTo(p.x - u * 1.6, p.y + u);
      ctx.lineTo(p.x, p.y - u * 1.6);
      ctx.lineTo(p.x + u * 1.6, p.y + u);
      ctx.fill();
    } else {
      // unclaimed site: a camp of tents
      ctx.fillStyle = "#f2e6c8";
      ctx.strokeStyle = "#5b4630";
      ctx.lineWidth = s;
      for (const dx of [-0.8, 0.8]) {
        ctx.beginPath();
        ctx.moveTo(p.x + dx * u - u * 0.7, p.y + u * 0.6);
        ctx.lineTo(p.x + dx * u, p.y - u * 0.7);
        ctx.lineTo(p.x + dx * u + u * 0.7, p.y + u * 0.6);
        ctx.closePath();
        ctx.fill();
        ctx.stroke();
      }
    }
    if (p.guardian) this.drawGuardian(p.x, p.y - u * 2.2, u);
    if (p.stargate) this.drawGate(p.x - u * 2.4, p.y + u * 0.4, u);
  }

  private drawCity(x: number, y: number, u: number, color: string, pop: number, walls: boolean): void {
    const ctx = this.ctx;
    const s = this.px(1);
    const houses = Math.min(6, 1 + Math.floor(pop / 15));
    if (walls) {
      ctx.strokeStyle = "#5c5348";
      ctx.lineWidth = u * 0.45;
      ctx.beginPath();
      ctx.arc(x, y, u * 2, 0, Math.PI * 2);
      ctx.stroke();
    }
    for (let i = 0; i < houses; i++) {
      const a = (i / houses) * Math.PI * 2 + 0.6;
      const hx = x + Math.cos(a) * u * 1.25, hy = y + Math.sin(a) * u * 1.0;
      ctx.fillStyle = "#e9dcc0";
      ctx.fillRect(hx - u * 0.45, hy - u * 0.3, u * 0.9, u * 0.6);
      ctx.fillStyle = "#9b3d25";
      ctx.beginPath();
      ctx.moveTo(hx - u * 0.55, hy - u * 0.3);
      ctx.lineTo(hx, hy - u * 0.75);
      ctx.lineTo(hx + u * 0.55, hy - u * 0.3);
      ctx.fill();
    }
    // keep
    ctx.fillStyle = "#b9ad97";
    ctx.strokeStyle = "#3d342a";
    ctx.lineWidth = s;
    ctx.fillRect(x - u * 0.6, y - u * 0.9, u * 1.2, u * 1.5);
    ctx.strokeRect(x - u * 0.6, y - u * 0.9, u * 1.2, u * 1.5);
    // banner
    ctx.strokeStyle = "#2b2219";
    ctx.beginPath();
    ctx.moveTo(x, y - u * 0.9);
    ctx.lineTo(x, y - u * 2.2);
    ctx.stroke();
    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.moveTo(x, y - u * 2.2);
    ctx.lineTo(x + u * 1.1, y - u * 1.9);
    ctx.lineTo(x, y - u * 1.6);
    ctx.fill();
  }

  private drawGuardian(x: number, y: number, u: number): void {
    const ctx = this.ctx;
    const img = this.theme.img("guardian");
    if (img) {
      ctx.drawImage(img, x - u * 2, y - u * 2, u * 4, u * 4);
      return;
    }
    ctx.fillStyle = "#8f1d14";
    ctx.beginPath();
    ctx.moveTo(x - u * 2, y - u * 0.4);
    ctx.lineTo(x - u * 0.4, y - u * 0.2);
    ctx.lineTo(x, y - u * 0.9);
    ctx.lineTo(x + u * 0.4, y - u * 0.2);
    ctx.lineTo(x + u * 2, y - u * 0.4);
    ctx.lineTo(x + u * 0.5, y + u * 0.5);
    ctx.lineTo(x - u * 0.5, y + u * 0.5);
    ctx.closePath();
    ctx.fill();
  }

  /** Space monsters (1oom crystal and amoeba): a blue crystal and a green blob. */
  private drawMonster(m: MonsterView): void {
    const ctx = this.ctx;
    const u = Math.max(1.6, Math.min(4, 12 / this.zoom + 1.2));
    const img = this.theme.img(`monster/${m.kind}`);
    if (img) {
      ctx.drawImage(img, m.x - u * 2, m.y - u * 2, u * 4, u * 4);
      return;
    }
    ctx.save();
    ctx.translate(m.x, m.y);
    if (m.kind === "crystal") {
      ctx.fillStyle = "#7fd3ff";
      ctx.strokeStyle = "#e8f8ff";
      ctx.lineWidth = this.px(1);
      ctx.beginPath();
      ctx.moveTo(0, -u * 1.8);
      ctx.lineTo(u * 1.1, 0);
      ctx.lineTo(0, u * 1.8);
      ctx.lineTo(-u * 1.1, 0);
      ctx.closePath();
      ctx.fill();
      ctx.stroke();
    } else {
      ctx.fillStyle = "rgba(90, 210, 90, 0.85)";
      ctx.beginPath();
      for (let i = 0; i <= 12; i++) {
        const a = (i / 12) * Math.PI * 2;
        const r = u * (1.3 + 0.35 * Math.sin(i * 2.7));
        if (i === 0) ctx.moveTo(Math.cos(a) * r, Math.sin(a) * r);
        else ctx.lineTo(Math.cos(a) * r, Math.sin(a) * r);
      }
      ctx.closePath();
      ctx.fill();
    }
    ctx.restore();
  }

  private drawStar(v: View, id: number): void {
    const p = v.planets[id];
    const ctx = this.ctx;
    const r = Math.max(1.4, Math.min(3, 8 / this.zoom + 1));
    const g = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, r * 2.4);
    g.addColorStop(0, STAR[p.star]);
    g.addColorStop(0.4, STAR[p.star] + "aa");
    g.addColorStop(1, STAR[p.star] + "00");
    ctx.fillStyle = g;
    ctx.beginPath();
    ctx.arc(p.x, p.y, r * 2.4, 0, Math.PI * 2);
    ctx.fill();
    if (p.owner !== null) {
      ctx.strokeStyle = this.theme.banner(v.empires[p.owner].banner);
      ctx.lineWidth = this.px(2);
      ctx.beginPath();
      ctx.arc(p.x, p.y, r * 2.2, 0, Math.PI * 2);
      ctx.stroke();
    }
    if (p.stargate) this.drawGate(p.x - r * 3, p.y, r * 1.4);
    if (p.guardian) {
      ctx.strokeStyle = "#ff3b2f";
      ctx.lineWidth = this.px(1.5);
      ctx.setLineDash([this.px(3), this.px(2)]);
      ctx.beginPath();
      ctx.arc(p.x, p.y, r * 3.2, 0, Math.PI * 2);
      ctx.stroke();
      ctx.setLineDash([]);
    }
  }

  private drawOrbit(v: View, id: number): void {
    const p = v.planets[id];
    const here = v.fleets.filter((f) => f.at === id);
    here.forEach((f, i) => this.drawFleet(v, f, p.x + 5 + i * 3.5, p.y - 2 + i * 3));
  }

  private drawFleet(v: View, f: FleetView, x: number, y: number): void {
    const ctx = this.ctx;
    const u = Math.max(1.2, Math.min(3, 9 / this.zoom + 0.9));
    const color = this.theme.banner(v.empires[f.owner].banner);
    const custom = this.theme.img("army");
    if (custom) {
      ctx.drawImage(custom, x - u * 1.5, y - u * 1.5, u * 3, u * 3);
    } else if (this.theme.def.map === "world") {
      ctx.strokeStyle = "#2b2219";
      ctx.lineWidth = this.px(1.2);
      ctx.beginPath();
      ctx.moveTo(x, y + u);
      ctx.lineTo(x, y - u * 1.4);
      ctx.stroke();
      ctx.fillStyle = color;
      ctx.strokeStyle = "#1a140e";
      ctx.beginPath();
      ctx.rect(x, y - u * 1.4, u * 1.5, u * 1.1);
      ctx.fill();
      ctx.stroke();
    } else {
      ctx.fillStyle = color;
      ctx.beginPath();
      ctx.moveTo(x + u * 1.3, y);
      ctx.lineTo(x - u, y - u * 0.8);
      ctx.lineTo(x - u * 0.5, y);
      ctx.lineTo(x - u, y + u * 0.8);
      ctx.closePath();
      ctx.fill();
    }
    const n = f.ships.reduce((a, b) => a + b, 0);
    if (this.zoom > 3) {
      ctx.fillStyle = "#fff";
      ctx.strokeStyle = "#000";
      ctx.lineWidth = this.px(2.5);
      ctx.font = `bold ${this.px(11)}px sans-serif`;
      ctx.textAlign = "left";
      ctx.textBaseline = "middle";
      ctx.strokeText(String(n), x + u * 1.7, y - u * 0.8);
      ctx.fillText(String(n), x + u * 1.7, y - u * 0.8);
    }
  }

  private drawCaravan(v: View, t: TransportView): void {
    const ctx = this.ctx;
    const u = Math.max(1.2, Math.min(3, 9 / this.zoom + 0.9));
    const custom = this.theme.img("caravan");
    if (custom) {
      ctx.drawImage(custom, t.x - u * 1.5, t.y - u * 1.5, u * 3, u * 3);
      return;
    }
    ctx.fillStyle = this.theme.banner(v.empires[t.owner].banner);
    ctx.strokeStyle = "#1a140e";
    ctx.lineWidth = this.px(1);
    ctx.fillRect(t.x - u, t.y - u * 0.7, u * 2, u);
    ctx.strokeRect(t.x - u, t.y - u * 0.7, u * 2, u);
    ctx.fillStyle = "#2b2219";
    for (const dx of [-0.6, 0.6]) {
      ctx.beginPath();
      ctx.arc(t.x + dx * u, t.y + u * 0.45, u * 0.35, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  private drawSelection(v: View): void {
    const s = this.selection;
    if (!s) return;
    let x: number, y: number;
    if (s.kind === "planet") ({ x, y } = v.planets[s.id]);
    else {
      const f = s.kind === "fleet" ? v.fleets.find((f) => f.id === s.id) : v.transports.find((t) => t.id === s.id);
      if (!f) return;
      ({ x, y } = f);
    }
    const ctx = this.ctx;
    const t = performance.now() / 300;
    ctx.strokeStyle = "#ffe9a8";
    ctx.lineWidth = this.px(2);
    ctx.beginPath();
    ctx.arc(x, y, this.px(16 + Math.sin(t) * 2), 0, Math.PI * 2);
    ctx.stroke();
  }

  private drawLabels(v: View): void {
    const ctx = this.ctx;
    const world = this.theme.def.map === "world";
    const size = this.px(world ? 13 : 12);
    ctx.font = `${world ? "bold " : ""}${size}px ${world ? "Georgia, 'Palatino Linotype', serif" : "sans-serif"}`;
    ctx.textAlign = "center";
    ctx.textBaseline = "top";
    ctx.lineWidth = this.px(3);
    for (const p of v.planets) {
      const y = p.y + this.px(14) + 2;
      ctx.strokeStyle = world ? "rgba(30,22,12,0.85)" : "#000";
      ctx.fillStyle = p.owner !== null ? this.theme.banner(v.empires[p.owner].banner) : p.explored ? "#f3ead2" : "#a49c88";
      ctx.strokeText(p.name, p.x, y);
      ctx.fillText(p.name, p.x, y);
    }
  }

  private drawTargetLine(v: View, id: number): void {
    const t = this.targeting!;
    const p = v.planets[id];
    const ok = t.ok(id);
    const ctx = this.ctx;
    ctx.save();
    ctx.strokeStyle = ok ? "#ffe9a8" : "#ff5b4a";
    ctx.lineWidth = this.px(2);
    ctx.beginPath();
    ctx.moveTo(t.from.x, t.from.y);
    ctx.lineTo(p.x, p.y);
    ctx.stroke();
    const eta = Math.max(1, Math.ceil(Math.hypot(p.x - t.from.x, p.y - t.from.y) / (11 * t.speed)));
    const label = ok ? (t.label?.(id) ?? this.theme.t("map.eta", { turns: eta })) : this.theme.t("map.out_of_range");
    ctx.font = `bold ${this.px(13)}px sans-serif`;
    ctx.textAlign = "left";
    ctx.textBaseline = "bottom";
    ctx.lineWidth = this.px(3);
    ctx.strokeStyle = "#000";
    ctx.strokeText(label, p.x + this.px(12), p.y - this.px(8));
    ctx.fillStyle = ok ? "#ffe9a8" : "#ff5b4a";
    ctx.fillText(label, p.x + this.px(12), p.y - this.px(8));
    ctx.restore();
  }

  refresh(): void {
    this.dirty = true;
  }
}
