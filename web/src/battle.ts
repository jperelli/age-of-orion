// The tactical battle screen (1oom ui/classic/uibattle.c): a 10x8 grid, the stacks of both
// sides, missiles in flight and the controls of the current stack. The core decides
// everything; this screen sends orders and animates the battle log the core returns.
import { TUNE, audio } from "./audio";
import type { Ctx } from "./ctx";
import { shipParts } from "./engine";
import { CITY_MARKS, cityLevel } from "./map";
import { type Battle, type BattleAction, type BattleAnim, type BattleItem, type View } from "./types";
import { append, clear, h } from "./ui";

const CW = 32;
const CH = 24;
const W = 10;
const H = 8;

interface Shown {
  item: BattleItem;
  x: number;
  y: number;
  alpha: number;
  flash: number;
  /** An animation of the theme's troop pictures: marching while it moves, attacking once. */
  anim?: { kind: "march" | "attack"; start: number; until: number };
}

/** Frames of a troop animation (`unit/<look>/<kind>/<frame>`). */
const ANIM_FRAMES = 4;

interface Effect {
  until: number;
  draw: (c: CanvasRenderingContext2D, left: number) => void;
}

/** Plays the battle the turn waits for until it is over. */
export async function battleScreen(ctx: Ctx): Promise<void> {
  const screen = new BattleScreen(ctx);
  try {
    await screen.run();
  } finally {
    screen.close();
  }
}

class BattleScreen {
  private el: HTMLElement;
  private canvas: HTMLCanvasElement;
  private g: CanvasRenderingContext2D;
  private side: HTMLElement;
  private status: HTMLElement;
  private bt: Battle;
  private shown = new Map<number, Shown>();
  private missiles = new Map<number, { x: number; y: number; w: number; dx: number; dy: number }>();
  private effects: Effect[] = [];
  private hover: { x: number; y: number } | null = null;
  private scale = 2;
  private speed = 1;
  private skipping = false;
  private animating = false;
  private resolveAction: ((a: BattleAction | null) => void) | null = null;
  private keyHandler = (e: KeyboardEvent) => this.onKey(e);
  private raf = 0;
  private stars: [number, number, number][] = [];
  private world: boolean;

  constructor(private ctx: Ctx) {
    this.bt = ctx.view().battle!;
    this.world = ctx.theme.def.map === "world";
    this.canvas = h("canvas", { class: "battle-canvas" });
    this.g = this.canvas.getContext("2d")!;
    this.side = h("aside", { class: "battle-side" });
    this.status = h("div", { class: "battle-status" });
    const planet = ctx.view().planets[this.bt.planet];
    this.el = h(
      "div",
      { class: `overlay battle-overlay theme-${ctx.theme.def.id}` },
      h(
        "div",
        { class: "battle" },
        h("header", { class: "battle-head" }, h("h2", {}, this.t("battle.title", { planet: planet.name })), this.sidesLabel()),
        h("div", { class: "battle-main" }, h("div", { class: "battle-field" }, this.canvas, this.status), this.side),
      ),
    );
    document.body.append(this.el);
    document.addEventListener("keydown", this.keyHandler, true);
    this.canvas.addEventListener("mousemove", (e) => {
      this.hover = this.cellAt(e);
      if (!this.animating) this.renderSide();
    });
    this.canvas.addEventListener("mouseleave", () => (this.hover = null));
    this.canvas.addEventListener("click", (e) => this.onClick(e));
    window.addEventListener("resize", () => this.resize());
    let seed = this.bt.planet * 7919 + 13;
    const rnd = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
    for (let i = 0; i < 90; i++) this.stars.push([rnd() * W * CW, rnd() * H * CH, rnd()]);
    this.sync();
    this.resize();
    const loop = () => {
      this.draw();
      this.raf = requestAnimationFrame(loop);
    };
    this.raf = requestAnimationFrame(loop);
    audio.play(TUNE.battle);
  }

  close(): void {
    cancelAnimationFrame(this.raf);
    document.removeEventListener("keydown", this.keyHandler, true);
    this.el.remove();
    audio.stopMusic();
  }

  private t(key: string, params?: Record<string, string | number>): string {
    return this.ctx.theme.t(key, params);
  }

  private get view(): View {
    return this.ctx.view();
  }

  private partyName(party: number): string {
    if (party === -1) return this.t("guardian.name");
    if (party === -2) return this.t("monster.crystal");
    if (party === -3) return this.t("monster.amoeba");
    return this.t(`race.${this.view.empires[party].race}.plural`);
  }

  private partyColor(party: number): string {
    return party < 0 ? "#b33" : this.ctx.theme.banner(this.view.empires[party].banner);
  }

  private sidesLabel(): HTMLElement {
    const [l, r] = this.bt.s;
    const tag = (p: number) => h("span", { class: "battle-party", style: `--banner:${this.partyColor(p)}` }, this.partyName(p));
    return h("div", { class: "battle-sides" }, tag(l.party), h("span", { class: "muted" }, this.t("battle.versus")), tag(r.party));
  }

  private itemName(it: BattleItem): string {
    if (it.design === null) {
      if (it.name === "guardian") return this.t("guardian.name");
      if (it.name === "crystal") return this.t("monster.crystal");
      if (it.name === "amoeba") return this.t("monster.amoeba");
    }
    return it.name;
  }

  private mySide(): "l" | "r" {
    return this.bt.s[0].human ? "l" : "r";
  }

  // ------------------------------------------------------------- flow

  async run(): Promise<void> {
    if (!this.bt.started) {
      const mode = await this.intro();
      this.send({ type: "start_battle", mode });
      await this.animate();
    }
    while (this.bt.winner === null) {
      this.renderSide();
      const action = await new Promise<BattleAction | null>((r) => (this.resolveAction = r));
      this.resolveAction = null;
      if (!action) continue;
      if (action.type === "auto") this.speed = 3;
      audio.sfx("click");
      this.send({ type: "battle", action });
      await this.animate();
    }
    await this.result();
  }

  private send(cmd: Parameters<Ctx["game"]["command"]>[0]): void {
    const err = this.ctx.game.command(cmd);
    this.ctx.refresh();
    const bt = this.view.battle;
    if (bt) this.bt = bt;
    if (err) this.status.textContent = this.t(`error.${err}`);
  }

  private intro(): Promise<"fight" | "auto" | "retreat"> {
    return new Promise((resolve) => {
      const count = (side: "l" | "r") =>
        this.bt.item
          .filter((i, k) => k > 0 && i.side === side)
          .map((i) => h("li", {}, `${i.num} × ${this.itemName(i)}`));
      const planet = this.bt.item[0];
      append(clear(this.side),
        h("h3", {}, this.t("battle.intro")),
        h("p", {}, this.partyName(this.bt.s[0].party)),
        h("ul", { class: "battle-list" }, ...count("l")),
        h("p", {}, this.partyName(this.bt.s[1].party)),
        h("ul", { class: "battle-list" }, ...count("r")),
        this.bt.planet_side !== "none" && planet.num > 0 ? h("p", { class: "muted" }, this.t("battle.planet_defends", { planet: planet.name, bases: planet.num })) : null,
        h(
          "div",
          { class: "battle-buttons col" },
          h("button", { class: "primary", onclick: () => resolve("fight") }, this.t("battle.fight")),
          h("button", { onclick: () => ((this.speed = 4), resolve("auto")) }, this.t("battle.auto_resolve")),
          h("button", { onclick: () => ((this.speed = 4), resolve("retreat")) }, this.t("battle.retreat_all")),
        ),
      );
    });
  }

  private result(): Promise<void> {
    return new Promise((resolve) => {
      const won = this.bt.winner === this.mySide();
      const planet = this.view.planets[this.bt.planet];
      append(clear(this.side),
        h("h3", { class: won ? "good" : "warn" }, this.t(won ? "battle.won" : "battle.lost")),
        h("p", {}, this.t("battle.after", { turns: this.bt.num_turn })),
        this.bt.planet_side !== "none" && planet.pop !== null ? h("p", { class: "muted" }, this.t("battle.planet_after", { planet: planet.name, pop: planet.pop ?? 0, bases: planet.bases ?? 0 })) : null,
        h("div", { class: "battle-buttons col" }, h("button", { class: "primary", onclick: () => resolve() }, this.t("btn.continue"))),
      );
      const key = (e: KeyboardEvent) => {
        if (e.key === "Enter" || e.key === "Escape") {
          document.removeEventListener("keydown", key);
          resolve();
        }
      };
      document.addEventListener("keydown", key);
    });
  }

  // ------------------------------------------------------------- input

  private act(a: BattleAction): void {
    if (this.animating || !this.bt.waiting || this.bt.winner !== null) return;
    this.resolveAction?.(a);
  }

  private onKey(e: KeyboardEvent): void {
    if (!document.body.contains(this.el)) return;
    const k = e.key.toLowerCase();
    if (this.animating && (k === " " || k === "escape")) {
      this.skipping = true;
      e.preventDefault();
      return;
    }
    const map: Record<string, BattleAction> = { d: { type: "done" }, w: { type: "wait" }, m: { type: "missile" }, s: { type: "special" }, r: { type: "retreat" }, a: { type: "auto" } };
    if (map[k] && this.bt.started) {
      e.preventDefault();
      e.stopPropagation();
      this.act(map[k]);
    }
  }

  private cellAt(e: MouseEvent): { x: number; y: number } | null {
    const r = this.canvas.getBoundingClientRect();
    const x = Math.floor(((e.clientX - r.left) / r.width) * W);
    const y = Math.floor(((e.clientY - r.top) / r.height) * H);
    return x >= 0 && x < W && y >= 0 && y < H ? { x, y } : null;
  }

  private onClick(e: MouseEvent): void {
    const c = this.cellAt(e);
    if (!c || !this.bt.started) return;
    const a = this.bt.waiting ? this.bt.area[c.y]?.[c.x] ?? 0 : 0;
    if (a === 1 || a >= 30) this.act({ type: "cell", x: c.x, y: c.y });
    else {
      this.hover = c;
      this.renderSide();
    }
  }

  // ------------------------------------------------------------- side panel

  private itemAt(x: number, y: number): BattleItem | undefined {
    return this.bt.item.find((i) => i.side !== "none" && i.sx === x && i.sy === y && (i.num > 0 || i.design !== null));
  }

  private stats(it: BattleItem): HTMLElement {
    const parts = shipParts();
    const weapons = it.wpn
      .filter((w) => w.n > 0 && w.t > 0)
      .map((w) => {
        const shots = w.numshots >= 0 ? ` (${this.t("battle.shots", { n: w.numshots })})` : "";
        return h("li", {}, `${w.n} × ${this.t(`part.weapon.${parts.weapons[w.t].key}`)}${shots}`);
      });
    const specials = it.specials.filter((s) => s > 0).map((s) => h("li", { class: "muted" }, this.t(`part.special.${parts.specials[s].key}`)));
    const isPlanet = it.design === null && it.side === this.bt.planet_side && this.bt.item[0].uid === it.uid;
    return h(
      "div",
      { class: "battle-stats" },
      h("b", {}, `${it.num} × ${this.itemName(it)}`),
      h(
        "table",
        {},
        h("tr", {}, h("td", {}, this.t("battle.hp")), h("td", {}, `${Math.max(0, it.hp1 - it.hploss)} / ${it.hp2}`)),
        h("tr", {}, h("td", {}, this.t("battle.attack")), h("td", {}, String(it.complevel))),
        h("tr", {}, h("td", {}, this.t("battle.defense")), h("td", {}, `${it.defense} / ${it.misdefense}`)),
        h("tr", {}, h("td", {}, this.t("battle.shield")), h("td", {}, String(it.absorb))),
        isPlanet ? null : h("tr", {}, h("td", {}, this.t("battle.speed")), h("td", {}, `${Math.max(0, it.actman)} / ${it.man - it.unman}`)),
        it.stasisby ? h("tr", {}, h("td", { colspan: 2, class: "warn" }, this.t("battle.in_stasis"))) : null,
      ),
      weapons.length ? h("ul", { class: "battle-list" }, ...weapons) : null,
      specials.length ? h("ul", { class: "battle-list" }, ...specials) : null,
    );
  }

  private renderSide(): void {
    if (!this.bt.started || (this.bt.winner !== null && !this.animating)) return;
    const bt = this.bt;
    if (this.animating) {
      append(
        clear(this.side),
        h("p", { class: "muted small" }, this.t("battle.round", { n: bt.num_turn })),
        h("h3", { class: "muted" }, this.t("battle.fighting")),
        h("div", { class: "battle-buttons" }, h("button", { onclick: () => (this.skipping = true), title: "Space" }, this.t("battle.skip"))),
      );
      return;
    }
    const cur = bt.item[bt.cur_item];
    const mine = bt.waiting && !this.animating && cur;
    const hovered = this.hover ? this.itemAt(this.hover.x, this.hover.y) : undefined;
    const btn = (label: string, key: string, a: BattleAction, opts: { on?: boolean; disabled?: boolean } = {}) =>
      h("button", { class: opts.on ? "on" : "", disabled: !mine || !!opts.disabled, onclick: () => this.act(a), title: key.toUpperCase() }, label);
    const special = cur && (cur.stasis > 0 || cur.pulsar > 0) && bt.cur_item !== 0;
    append(clear(this.side),
      h("p", { class: "muted small" }, this.t("battle.round", { n: bt.num_turn })),
      mine ? h("h3", {}, this.t("battle.your_turn")) : h("h3", { class: "muted" }, this.t("battle.enemy_turn")),
      mine ? this.stats(cur) : null,
      h(
        "div",
        { class: "battle-buttons" },
        btn(this.t("battle.btn.done"), "d", { type: "done" }),
        btn(this.t("battle.btn.wait"), "w", { type: "wait" }),
        cur && cur.missile >= 0
          ? btn(this.t(bt.cur_item === 0 ? "battle.btn.base_weapon" : cur.missile === 1 ? "battle.btn.missiles_on" : "battle.btn.missiles_off"), "m", { type: "missile" }, { on: cur.missile === 1 })
          : null,
        special ? btn(this.t(cur.stasis > 0 ? "battle.btn.stasis" : "battle.btn.pulsar"), "s", { type: "special" }, { on: bt.special_button === 1 }) : null,
        btn(this.t("battle.btn.retreat"), "r", { type: "retreat" }, { disabled: bt.cur_item === 0 }),
        btn(this.t("battle.btn.auto"), "a", { type: "auto" }),
      ),
      mine ? h("p", { class: "muted small" }, this.t("battle.help")) : null,
      hovered && hovered !== cur ? h("div", { class: "battle-hover" }, this.stats(hovered)) : null,
    );
  }

  // ------------------------------------------------------------- animation

  /** Positions from the battle state. */
  private sync(): void {
    const keep = new Map(this.shown);
    this.shown.clear();
    for (const it of this.bt.item) {
      if (it.side === "none" || it.sx < 0) continue;
      const old = keep.get(it.uid);
      this.shown.set(it.uid, { item: it, x: it.sx * CW, y: it.sy * CH, alpha: it.cloak === 1 ? 0.35 : 1, flash: old?.flash ?? 0 });
    }
    this.missiles.clear();
    for (const m of this.bt.missiles) this.missiles.set(m.id, { x: m.x, y: m.y, w: m.wpnt, dx: 1, dy: 0 });
  }

  private wait(ms: number): Promise<void> {
    if (this.skipping) return Promise.resolve();
    return new Promise((r) => setTimeout(r, ms / this.speed));
  }

  private tween(ms: number, f: (k: number) => void): Promise<void> {
    if (this.skipping) {
      f(1);
      return Promise.resolve();
    }
    const dur = ms / this.speed;
    const t0 = performance.now();
    return new Promise((resolve) => {
      const step = () => {
        const k = Math.min(1, (performance.now() - t0) / dur);
        f(k);
        if (k < 1) requestAnimationFrame(step);
        else resolve();
      };
      requestAnimationFrame(step);
    });
  }

  private center(uid: number): [number, number] {
    const s = this.shown.get(uid);
    return s ? [s.x + CW / 2, s.y + CH / 2] : [W * CW / 2, H * CH / 2];
  }

  private effect(ms: number, draw: Effect["draw"]): void {
    if (this.skipping) return;
    this.effects.push({ until: performance.now() + ms / this.speed, draw });
  }

  private async animate(): Promise<void> {
    const log = this.bt.log;
    this.animating = true;
    this.skipping = false;
    this.renderSide();
    // Positions from before this step: replay from the shown state.
    const parts = shipParts();
    let i = 0;
    while (i < log.length) {
      const a = log[i];
      if (a.type === "missile_step") {
        // All consecutive missile steps form one frame.
        const frame: Extract<BattleAnim, { type: "missile_step" }>[] = [];
        while (i < log.length && log[i].type === "missile_step") frame.push(log[i++] as Extract<BattleAnim, { type: "missile_step" }>);
        const start = frame.map((f) => ({ f, from: this.missiles.get(f.id) }));
        await this.tween(60, (k) => {
          for (const { f, from } of start) {
            if (!from) continue;
            const m = this.missiles.get(f.id);
            if (m) {
              m.dx = f.x - from.x || m.dx;
              m.dy = f.y - from.y;
            }
            this.missiles.set(f.id, { x: from.x + (f.x - from.x) * k, y: from.y + (f.y - from.y) * k, w: from.w, dx: m?.dx ?? 1, dy: m?.dy ?? 0 });
          }
        });
        continue;
      }
      i++;
      await this.play(a, parts);
    }
    this.skipping = false;
    this.sync();
    this.animating = false;
    this.renderSide();
  }

  private async play(a: BattleAnim, parts: ReturnType<typeof shipParts>): Promise<void> {
    switch (a.type) {
      case "turn": {
        const s = this.shown.get(a.uid);
        if (s) s.flash = performance.now() + 300 / this.speed;
        return this.wait(80);
      }
      case "move":
      case "repulse": {
        const s = this.shown.get(a.uid);
        if (!s) return;
        const [x0, y0] = [s.x, s.y];
        // Painted troops walk a little slower, so their march shows.
        const ms = a.type === "move" ? (this.world && this.ctx.theme.img(`unit/${s.item.look % 24}/march/0`) ? 260 : 140) : 200;
        if (a.type === "move") this.troopAnim(s, "march", ms);
        return this.tween(ms, (k) => {
          s.x = x0 + (a.x * CW - x0) * k;
          s.y = y0 + (a.y * CH - y0) * k;
        });
      }
      case "teleport": {
        const s = this.shown.get(a.uid);
        if (!s) return;
        await this.tween(150, (k) => (s.alpha = 1 - k));
        s.x = a.x * CW;
        s.y = a.y * CH;
        return this.tween(150, (k) => (s.alpha = k));
      }
      case "beam": {
        this.troopAnim(this.shown.get(a.from), "attack", 320);
        const [x0, y0] = this.center(a.from);
        const [x1, y1] = this.center(a.to);
        const w = parts.weapons[a.weapon];
        const heavy = w.damage_max >= 15;
        audio.sfx(this.world ? (heavy ? "clash" : "arrow") : heavy ? "heavy_beam" : "beam", this.world ? undefined : w.sound);
        const color = beamColor(w.tech, this.world);
        if (this.world) {
          // A volley: several projectiles flying to the target.
          this.effect(260, (c, left) => {
            const k = 1 - left / (260 / this.speed);
            c.strokeStyle = color;
            c.lineWidth = heavy ? 2.5 : 1.2;
            for (let j = 0; j < 4; j++) {
              const kk = Math.max(0, Math.min(1, k * 1.4 - j * 0.12));
              const px = x0 + (x1 - x0) * kk + (j - 1.5) * 2;
              const py = y0 + (y1 - y0) * kk - Math.sin(kk * Math.PI) * 10 + (j - 1.5);
              const ang = Math.atan2(y1 - y0, x1 - x0);
              c.beginPath();
              c.moveTo(px - Math.cos(ang) * 5, py - Math.sin(ang) * 5);
              c.lineTo(px, py);
              c.stroke();
            }
          });
        } else {
          this.effect(180, (c) => {
            c.strokeStyle = color;
            c.lineWidth = heavy ? 2.5 : 1.3;
            c.shadowColor = color;
            c.shadowBlur = 6;
            c.beginPath();
            c.moveTo(x0, y0);
            c.lineTo(x1, y1);
            c.stroke();
            c.shadowBlur = 0;
          });
        }
        return this.wait(this.world ? 220 : 160);
      }
      case "bomb": {
        const [x1, y1] = this.center(a.to);
        audio.sfx("bomb");
        this.effect(400, (c, left) => {
          const k = 1 - left / (400 / this.speed);
          c.fillStyle = this.world ? "#ff7a1a" : "#ffd25a";
          for (let j = 0; j < 6; j++) {
            c.beginPath();
            c.arc(x1 - 10 + j * 4, y1 - 20 + k * 22 + (j % 2) * 3, 1.6, 0, Math.PI * 2);
            c.fill();
          }
        });
        return this.wait(380);
      }
      case "damage": {
        const [x, y] = this.center(a.uid);
        const s = this.shown.get(a.uid);
        if (s) {
          s.item = { ...s.item, num: Math.max(0, s.item.num - a.killed) };
          s.flash = performance.now() + 200 / this.speed;
        }
        audio.sfx("hit");
        this.effect(700, (c, left) => {
          const k = 1 - left / (700 / this.speed);
          c.font = "bold 9px sans-serif";
          c.textAlign = "center";
          c.fillStyle = "#ff5a4a";
          c.strokeStyle = "#000";
          c.lineWidth = 2;
          const txt = `-${a.amount}`;
          c.strokeText(txt, x, y - 6 - k * 10);
          c.fillText(txt, x, y - 6 - k * 10);
        });
        return this.wait(120);
      }
      case "missile_launch": {
        this.troopAnim(this.shown.get(a.from), "attack", 320);
        audio.sfx(this.world ? "arrow" : "missile", this.world ? undefined : parts.weapons[a.weapon].sound);
        this.missiles.set(a.id, { x: a.x, y: a.y, w: a.weapon, dx: 1, dy: 0 });
        return this.wait(60);
      }
      case "missile_gone":
        this.missiles.delete(a.id);
        return;
      case "missile_shield": {
        const [x, y] = this.center(a.uid);
        this.effect(250, (c) => {
          c.strokeStyle = "#7fd3ff";
          c.lineWidth = 1.5;
          c.beginPath();
          c.arc(x, y, 14, 0, Math.PI * 2);
          c.stroke();
        });
        return this.wait(80);
      }
      case "explosion":
        this.boom(a.x + 4, a.y + 4, 6);
        return this.wait(60);
      case "destroyed": {
        const [x, y] = this.center(a.uid);
        audio.sfx("explosion");
        this.boom(x, y, 16);
        await this.wait(250);
        this.shown.delete(a.uid);
        return;
      }
      case "retreat": {
        const s = this.shown.get(a.uid);
        if (!s) return;
        const dir = s.item.side === "l" ? -1 : 1;
        const x0 = s.x;
        await this.tween(300, (k) => {
          s.x = x0 + dir * k * CW * 2;
          s.alpha = 1 - k;
        });
        this.shown.delete(a.uid);
        return;
      }
      case "special": {
        const [x0, y0] = this.center(a.from);
        const [x1, y1] = a.to !== null ? this.center(a.to) : [x0, y0];
        audio.sfx("special");
        const label = this.t(`battle.special.${a.kind}`);
        this.effect(600, (c, left) => {
          const k = 1 - left / (600 / this.speed);
          c.strokeStyle = SPECIAL_COLOR[a.kind] ?? "#fff";
          c.lineWidth = 2;
          if (a.to === null || a.kind.includes("pulsar")) {
            c.beginPath();
            c.arc(x0, y0, 6 + k * 34, 0, Math.PI * 2);
            c.stroke();
          } else {
            c.setLineDash([3, 3]);
            c.beginPath();
            c.moveTo(x0, y0);
            c.lineTo(x1, y1);
            c.stroke();
            c.setLineDash([]);
            c.beginPath();
            c.arc(x1, y1, 4 + k * 10, 0, Math.PI * 2);
            c.stroke();
          }
          c.font = "bold 8px sans-serif";
          c.textAlign = "center";
          c.fillStyle = "#fff";
          c.fillText(label, x1, y1 - 16);
        });
        return this.wait(500);
      }
      case "cloak": {
        const s = this.shown.get(a.uid);
        if (s) await this.tween(200, (k) => (s.alpha = a.on ? 1 - 0.65 * k : 0.35 + 0.65 * k));
        return;
      }
    }
  }

  /** Starts a troop animation (marching runs on through consecutive steps). */
  private troopAnim(s: Shown | undefined, kind: "march" | "attack", ms: number): void {
    if (!s || !this.world) return;
    const now = performance.now();
    const keep = s.anim?.kind === kind && kind === "march" && s.anim.until > now - 60;
    s.anim = { kind, start: keep ? s.anim!.start : now, until: now + ms / this.speed };
  }

  /** The troop picture of a stack now: a frame of its animation, else its still picture. */
  private unitPicture(s: Shown, now: number): HTMLImageElement | null {
    const it = s.item;
    const base = it.design === null ? `unit/${it.name}` : `unit/${it.look % 24}`;
    const a = s.anim;
    if (a && a.until > now) {
      const f =
        a.kind === "march"
          ? Math.floor((now - a.start) / (110 / this.speed)) % ANIM_FRAMES
          : Math.min(ANIM_FRAMES - 1, Math.floor(((now - a.start) / (a.until - a.start)) * ANIM_FRAMES));
      const frame = this.ctx.theme.img(`${base}/${a.kind}/${f}`);
      if (frame) return frame;
    }
    return this.ctx.theme.img(base);
  }

  private boom(x: number, y: number, r: number): void {
    const img = this.ctx.theme.img("battle/explosion/0");
    this.effect(450, (c, left) => {
      const k = 1 - left / (450 / this.speed);
      if (img) {
        const s = r * 2 * (0.6 + k);
        c.globalAlpha = 1 - k;
        c.drawImage(img, x - s / 2, y - s / 2, s, s);
        c.globalAlpha = 1;
        return;
      }
      const g = c.createRadialGradient(x, y, 0, x, y, r * (0.5 + k));
      if (this.world) {
        g.addColorStop(0, `rgba(235,215,170,${0.9 * (1 - k)})`);
        g.addColorStop(0.55, `rgba(150,120,80,${0.7 * (1 - k)})`);
        g.addColorStop(1, "rgba(90,70,40,0)");
      } else {
        g.addColorStop(0, `rgba(255,240,180,${1 - k})`);
        g.addColorStop(0.5, `rgba(255,120,30,${0.8 * (1 - k)})`);
        g.addColorStop(1, "rgba(120,20,0,0)");
      }
      c.fillStyle = g;
      c.beginPath();
      c.arc(x, y, r * (0.5 + k), 0, Math.PI * 2);
      c.fill();
    });
  }

  // ------------------------------------------------------------- drawing

  private resize(): void {
    const box = this.canvas.parentElement!.getBoundingClientRect();
    const avail = Math.max(320, Math.min(box.width - 8, ((window.innerHeight - 150) * W * CW) / (H * CH)));
    this.scale = avail / (W * CW);
    const dpr = window.devicePixelRatio || 1;
    this.canvas.style.width = `${W * CW * this.scale}px`;
    this.canvas.style.height = `${H * CH * this.scale}px`;
    this.side.style.height = `${H * CH * this.scale + 24}px`;
    this.canvas.width = Math.round(W * CW * this.scale * dpr);
    this.canvas.height = Math.round(H * CH * this.scale * dpr);
  }

  private draw(): void {
    const c = this.g;
    const k = this.canvas.width / (W * CW);
    c.setTransform(k, 0, 0, k, 0, 0);
    c.imageSmoothingEnabled = false;
    this.drawBackground(c);
    const bt = this.bt;
    const now = performance.now();
    // Grid and the reach of the current stack.
    c.strokeStyle = this.world ? "rgba(40,30,10,0.18)" : "rgba(120,160,255,0.12)";
    c.lineWidth = 0.5;
    for (let x = 0; x <= W; x++) {
      c.beginPath();
      c.moveTo(x * CW, 0);
      c.lineTo(x * CW, H * CH);
      c.stroke();
    }
    for (let y = 0; y <= H; y++) {
      c.beginPath();
      c.moveTo(0, y * CH);
      c.lineTo(W * CW, y * CH);
      c.stroke();
    }
    if (bt.waiting && !this.animating && bt.winner === null) {
      for (let y = 0; y < H; y++) {
        for (let x = 0; x < W; x++) {
          const a = bt.area[y][x];
          if (a === 1) {
            c.fillStyle = this.world ? "rgba(255,240,170,0.22)" : "rgba(90,200,255,0.16)";
            c.fillRect(x * CW + 1, y * CH + 1, CW - 2, CH - 2);
          } else if (a >= 30) {
            c.strokeStyle = "#ff4a3a";
            c.lineWidth = 1.2;
            c.strokeRect(x * CW + 1.5, y * CH + 1.5, CW - 3, CH - 3);
          }
        }
      }
      if (this.hover) {
        c.strokeStyle = "rgba(255,255,255,0.6)";
        c.lineWidth = 0.8;
        c.strokeRect(this.hover.x * CW + 0.5, this.hover.y * CH + 0.5, CW - 1, CH - 1);
      }
    }
    for (const [x, y, look] of bt.rocks) this.drawRock(c, x * CW, y * CH, look);
    const cur = bt.item[bt.cur_item];
    for (const s of this.shown.values()) {
      const isPlanet = bt.item[0].uid === s.item.uid && bt.planet_side !== "none";
      if (isPlanet) this.drawPlanet(c, s);
      else this.drawItem(c, s, !this.animating && bt.waiting && cur?.uid === s.item.uid, now);
    }
    for (const m of this.missiles.values()) this.drawMissile(c, m);
    this.effects = this.effects.filter((e) => e.until > now);
    for (const e of this.effects) e.draw(c, e.until - now);
  }

  private drawBackground(c: CanvasRenderingContext2D): void {
    const p = this.view.planets[this.bt.planet];
    const n = p.nebula ? 0 : 1 + (this.bt.planet % 4);
    const field = this.world ? this.ctx.theme.img(`battle/field/${p.nebula ? "mists" : FIELD[p.kind ?? "terran"] ?? "green"}`) : null;
    const img = field ?? this.ctx.theme.img(`battle/bg/${n}`) ?? this.ctx.theme.img("battle/bg");
    if (img) {
      this.paint(c, img, 0, 0, W * CW, H * CH, field !== null);
      return;
    }
    if (this.world) {
      const g = c.createLinearGradient(0, 0, 0, H * CH);
      g.addColorStop(0, p.nebula ? "#8d9a92" : "#9db467");
      g.addColorStop(1, p.nebula ? "#6f7d77" : "#73914a");
      c.fillStyle = g;
      c.fillRect(0, 0, W * CW, H * CH);
      for (const [x, y, b] of this.stars) {
        c.fillStyle = b > 0.5 ? "rgba(60,80,30,0.35)" : "rgba(200,210,120,0.25)";
        c.fillRect(x, y, 2 + b * 3, 1);
      }
      if (p.nebula) {
        c.fillStyle = "rgba(230,235,240,0.25)";
        c.fillRect(0, 0, W * CW, H * CH);
      }
    } else {
      c.fillStyle = p.nebula ? "#1b0f2a" : "#05070f";
      c.fillRect(0, 0, W * CW, H * CH);
      for (const [x, y, b] of this.stars) {
        c.fillStyle = `rgba(255,255,255,${0.3 + b * 0.7})`;
        c.fillRect(x, y, b > 0.9 ? 1.5 : 0.8, b > 0.9 ? 1.5 : 0.8);
      }
      if (p.nebula) {
        const g = c.createRadialGradient(W * CW * 0.6, H * CH * 0.4, 10, W * CW * 0.6, H * CH * 0.4, 200);
        g.addColorStop(0, "rgba(160,60,200,0.35)");
        g.addColorStop(1, "rgba(160,60,200,0)");
        c.fillStyle = g;
        c.fillRect(0, 0, W * CW, H * CH);
      }
    }
  }

  /** Draws a picture; painted (not pixel art) ones are smoothed. */
  private paint(c: CanvasRenderingContext2D, img: HTMLImageElement, x: number, y: number, w: number, h: number, smooth: boolean): void {
    c.imageSmoothingEnabled = smooth;
    c.imageSmoothingQuality = "high";
    c.drawImage(img, x, y, w, h);
    c.imageSmoothingEnabled = false;
  }

  /** A painted sprite fitted into a w x h box centred on (cx, cy). */
  private sprite(c: CanvasRenderingContext2D, img: HTMLImageElement, cx: number, cy: number, w: number, h: number): void {
    const k = Math.min(w / img.width, h / img.height);
    this.paint(c, img, cx - (img.width * k) / 2, cy - (img.height * k) / 2, img.width * k, img.height * k, true);
  }

  private drawRock(c: CanvasRenderingContext2D, x: number, y: number, look: number): void {
    const alias = `battle/rock/${look}`;
    const img = this.ctx.theme.img(alias);
    if (img && this.ctx.theme.pixel.has(alias)) {
      c.drawImage(img, x + (CW - img.width) / 2, y + (CH - img.height) / 2);
      return;
    }
    if (img) {
      this.sprite(c, img, x + CW / 2, y + CH / 2, CW, CH);
      return;
    }
    c.fillStyle = this.world ? "#7c766a" : "#5b544c";
    c.strokeStyle = this.world ? "#3d382f" : "#2a2622";
    c.lineWidth = 0.8;
    c.beginPath();
    const pts = 7;
    for (let i = 0; i <= pts; i++) {
      const a = (i / pts) * Math.PI * 2;
      const r = 8 + ((look * 5 + i * 3) % 4);
      const px = x + CW / 2 + Math.cos(a) * r;
      const py = y + CH / 2 + Math.sin(a) * r * 0.75;
      if (i === 0) c.moveTo(px, py);
      else c.lineTo(px, py);
    }
    c.fill();
    c.stroke();
    if (this.world) {
      c.fillStyle = "#4f7a35";
      c.beginPath();
      c.arc(x + CW / 2 + 6, y + CH / 2 - 5, 4, 0, Math.PI * 2);
      c.fill();
    }
  }

  private drawPlanet(c: CanvasRenderingContext2D, s: Shown): void {
    const p = this.view.planets[this.bt.planet];
    const party = this.bt.s[s.item.side === "l" ? 0 : 1].party;
    const color = this.partyColor(party);
    const cx = s.x + CW / 2;
    const cy = s.y + CH / 2;
    const img = p.image !== null ? this.ctx.theme.img(`planet/${p.image}`) : null;
    const mark = this.world ? this.ctx.theme.img(CITY_MARKS[cityLevel(p.pop ?? 0)]) : null;
    if (img && !this.world) {
      c.drawImage(img, cx - 15, cy - 11, 30, 22);
    } else if (mark) {
      this.sprite(c, mark, cx, cy + 1, CW + 4, CH + 2);
      pennant(c, cx - 9, cy - 4, color);
    } else if (this.world) {
      // A walled town that fits its square.
      c.fillStyle = "#b9a37a";
      c.strokeStyle = "#4a3b26";
      c.lineWidth = 0.8;
      c.fillRect(cx - 11, cy - 6, 22, 15);
      c.strokeRect(cx - 11, cy - 6, 22, 15);
      for (const tx of [-14, 10]) {
        c.fillRect(cx + tx, cy - 9, 4, 18);
        c.strokeRect(cx + tx, cy - 9, 4, 18);
      }
      c.strokeStyle = "#2b2219";
      c.beginPath();
      c.moveTo(cx, cy - 6);
      c.lineTo(cx, cy - 12);
      c.stroke();
      c.fillStyle = color;
      c.fillRect(cx, cy - 12, 6, 3.5);
    } else {
      const g = c.createRadialGradient(cx - 4, cy - 4, 1, cx, cy, 12);
      g.addColorStop(0, "#9fc4ff");
      g.addColorStop(1, "#1d3a7a");
      c.fillStyle = g;
      c.beginPath();
      c.arc(cx, cy, 11, 0, Math.PI * 2);
      c.fill();
      c.strokeStyle = color;
      c.lineWidth = 1.2;
      c.stroke();
    }
    this.label(c, s.x + CW - 1, s.y + CH - 1, s.item.num > 0 ? `${s.item.num}` : "", color);
  }

  private drawItem(c: CanvasRenderingContext2D, s: Shown, current: boolean, now: number): void {
    const it = s.item;
    const sideIdx = it.side === "l" ? 0 : 1;
    const color = this.partyColor(this.bt.s[sideIdx].party);
    c.globalAlpha = s.alpha;
    if (current) {
      const pulse = 0.5 + 0.5 * Math.sin(now / 180);
      c.strokeStyle = `rgba(255,230,90,${0.5 + pulse * 0.5})`;
      c.lineWidth = 1.5;
      c.strokeRect(s.x + 0.75, s.y + 0.75, CW - 1.5, CH - 1.5);
    }
    if (s.flash > now) {
      c.fillStyle = "rgba(255,255,255,0.25)";
      c.fillRect(s.x, s.y, CW, CH);
    }
    const img = this.ctx.theme.img(`ship/${it.look}`);
    const unit = this.world ? this.unitPicture(s, now) : null;
    c.save();
    if (it.side === "r") {
      c.translate(s.x + CW, s.y);
      c.scale(-1, 1);
    } else c.translate(s.x, s.y);
    if (unit) {
      this.sprite(c, unit, CW / 2 + 1, CH / 2 + 1, CW - 2, CH - 2);
      if (it.design !== null) pennant(c, 3, 9, color);
    } else if (img) c.drawImage(img, (CW - img.width) / 2, (CH - img.height) / 2);
    else if (it.design === null) this.drawMonster(c, it);
    else if (this.world) drawTroop(c, it.hull, color);
    else drawShip(c, it.hull, color);
    c.restore();
    if (it.stasisby) {
      c.fillStyle = "rgba(150,220,255,0.35)";
      c.fillRect(s.x + 2, s.y + 2, CW - 4, CH - 4);
    }
    c.globalAlpha = 1;
    this.label(c, s.x + CW - 1, s.y + CH - 1, String(it.num), color);
    // Damage of the front ship.
    if (it.hp2 > 0 && (it.hploss > 0 || it.hp1 < it.hp2)) {
      const k = Math.max(0, (it.hp1 - it.hploss) / it.hp2);
      c.fillStyle = "rgba(0,0,0,0.6)";
      c.fillRect(s.x + 3, s.y + 1, CW - 6, 2);
      c.fillStyle = k > 0.5 ? "#4cd14c" : k > 0.25 ? "#e6c33a" : "#e6463a";
      c.fillRect(s.x + 3, s.y + 1, (CW - 6) * k, 2);
    }
  }

  private drawMonster(c: CanvasRenderingContext2D, it: BattleItem): void {
    if (this.world) {
      // Catalan Company (gold and red), Great Company (black), the Immortals (Persian purple).
      drawTroop(c, 3, it.name === "crystal" ? "#d9a21f" : it.name === "amoeba" ? "#262626" : "#6a2c91");
      return;
    }
    if (it.name === "crystal") {
      c.fillStyle = "#7fd3ff";
      c.strokeStyle = "#e8f8ff";
      c.lineWidth = 0.8;
      c.beginPath();
      c.moveTo(16, 1);
      c.lineTo(26, 12);
      c.lineTo(16, 23);
      c.lineTo(6, 12);
      c.closePath();
      c.fill();
      c.stroke();
    } else if (it.name === "amoeba") {
      c.fillStyle = "rgba(90,210,90,0.9)";
      c.beginPath();
      for (let i = 0; i <= 12; i++) {
        const a = (i / 12) * Math.PI * 2;
        const r = 9 + 2.5 * Math.sin(i * 2.7 + performance.now() / 400);
        const x = 16 + Math.cos(a) * r;
        const y = 12 + Math.sin(a) * r * 0.8;
        if (i === 0) c.moveTo(x, y);
        else c.lineTo(x, y);
      }
      c.fill();
    } else {
      // The Guardian.
      c.fillStyle = "#8f1d14";
      c.beginPath();
      c.moveTo(2, 10);
      c.lineTo(12, 11);
      c.lineTo(16, 4);
      c.lineTo(20, 11);
      c.lineTo(30, 10);
      c.lineTo(20, 18);
      c.lineTo(12, 18);
      c.closePath();
      c.fill();
    }
  }

  private drawMissile(c: CanvasRenderingContext2D, m: { x: number; y: number; w: number; dx: number; dy: number }): void {
    const parts = shipParts();
    const w = parts.weapons[m.w];
    const ang = (Math.atan2(m.dy, m.dx) * 180) / Math.PI;
    const a = (ang + 360) % 360;
    const dir = [2, 3, 4, 5, 6, 7, 0, 1][Math.round(a / 45) % 8];
    const img = this.ctx.theme.img(`missile/${w.miss_type}/${dir}`);
    if (img) {
      c.drawImage(img, m.x - img.width / 2, m.y - img.height / 2);
      return;
    }
    const r = (ang * Math.PI) / 180;
    if (this.world) {
      drawProjectile(c, w.key, m.x, m.y, r, performance.now());
    } else {
      c.strokeStyle = "rgba(255,160,60,0.7)";
      c.lineWidth = 1;
      c.beginPath();
      c.moveTo(m.x - Math.cos(r) * 6, m.y - Math.sin(r) * 6);
      c.lineTo(m.x, m.y);
      c.stroke();
      c.fillStyle = "#fff6c8";
      c.fillRect(m.x - 1, m.y - 1, 2, 2);
    }
  }

  private label(c: CanvasRenderingContext2D, x: number, y: number, text: string, color: string): void {
    if (!text) return;
    c.font = "bold 7px sans-serif";
    c.textAlign = "right";
    c.textBaseline = "bottom";
    c.lineWidth = 2;
    c.strokeStyle = "#000";
    c.strokeText(text, x, y);
    c.fillStyle = color;
    c.fillText(text, x, y);
  }
}

const SPECIAL_COLOR: Record<string, string> = {
  stasis: "#9fe8ff",
  pulsar: "#ffe14a",
  ion_pulsar: "#ffa94a",
  stream: "#7dff7a",
  neutron_stream: "#3cff9a",
  warp_dissipator: "#d58bff",
  black_hole: "#c060ff",
  tech_nullifier: "#ff6ad5",
};

function beamColor(tech: number, world: boolean): string {
  if (world) return tech > 30 ? "#ff6a1a" : tech > 15 ? "#5a3b1a" : "#3b2a16";
  if (tech > 40) return "#ff5aff";
  if (tech > 25) return "#5ab4ff";
  if (tech > 12) return "#5aff7a";
  return "#ff4a3a";
}

/** A spaceship silhouette by hull size (facing right), in a 32x24 cell. */
function drawShip(c: CanvasRenderingContext2D, hull: number, color: string): void {
  const s = [0.45, 0.62, 0.8, 1][hull] ?? 0.6;
  c.save();
  c.translate(16, 12);
  c.scale(s, s);
  c.fillStyle = color;
  c.strokeStyle = "#0b0b0b";
  c.lineWidth = 1.2;
  c.beginPath();
  c.moveTo(14, 0);
  c.lineTo(-8, -8);
  c.lineTo(-4, -2);
  c.lineTo(-13, -4);
  c.lineTo(-13, 4);
  c.lineTo(-4, 2);
  c.lineTo(-8, 8);
  c.closePath();
  c.fill();
  c.stroke();
  c.fillStyle = "rgba(255,255,255,0.6)";
  c.fillRect(2, -1.2, 5, 2.4);
  c.fillStyle = "#ffb347";
  c.fillRect(-15, -2, 2, 4);
  c.restore();
}

/** A unit of soldiers under a banner; bigger hulls are bigger formations. */
function drawTroop(c: CanvasRenderingContext2D, hull: number, color: string): void {
  const n = [2, 3, 4, 6][hull] ?? 3;
  const cols = Math.min(3, n);
  for (let i = 0; i < n; i++) {
    const x = 9 + (i % cols) * 6 + Math.floor(i / cols) * 3;
    const y = 9 + Math.floor(i / cols) * 6;
    c.fillStyle = "#2b2219";
    c.fillRect(x - 1, y + 2, 3, 6);
    c.fillStyle = "#e3c9a0";
    c.beginPath();
    c.arc(x + 0.5, y + 1, 1.6, 0, Math.PI * 2);
    c.fill();
    c.fillStyle = color;
    c.fillRect(x - 2, y + 3, 5, 3);
    c.strokeStyle = "#5a4a32";
    c.lineWidth = 0.6;
    c.beginPath();
    c.moveTo(x + 3, y - 3);
    c.lineTo(x + 3, y + 8);
    c.stroke();
  }
  // Banner.
  c.strokeStyle = "#2b2219";
  c.lineWidth = 0.8;
  c.beginPath();
  c.moveTo(5, 3);
  c.lineTo(5, 20);
  c.stroke();
  c.fillStyle = color;
  c.fillRect(5, 3, 6, 4);
  if (hull >= 3) {
    c.fillStyle = "#6d5a3c";
    c.fillRect(22, 13, 8, 6);
    c.fillStyle = "#3b2f20";
    c.beginPath();
    c.arc(23.5, 20, 1.6, 0, Math.PI * 2);
    c.arc(28.5, 20, 1.6, 0, Math.PI * 2);
    c.fill();
  }
}

/** The battlefield of each kind of land (the Age theme's `battle/field/*`). */
const FIELD: Record<string, string> = {
  terran: "green",
  ocean: "green",
  jungle: "forest",
  desert: "dry",
  arid: "dry",
  steppe: "dry",
  minimal: "dry",
  tundra: "snow",
  barren: "rocky",
  dead: "rocky",
  not_habitable: "rocky",
  inferno: "ash",
  radiated: "ash",
  toxic: "marsh",
};

/** A small pennant in an empire's colour, on a pole standing at (x, y). */
function pennant(c: CanvasRenderingContext2D, x: number, y: number, color: string): void {
  c.strokeStyle = "#2b2219";
  c.lineWidth = 0.7;
  c.beginPath();
  c.moveTo(x, y);
  c.lineTo(x, y - 8);
  c.stroke();
  c.fillStyle = color;
  c.strokeStyle = "#1a140e";
  c.lineWidth = 0.4;
  c.beginPath();
  c.moveTo(x, y - 8);
  c.lineTo(x + 6, y - 6.5);
  c.lineTo(x, y - 5);
  c.closePath();
  c.fill();
  c.stroke();
}

/** A missile of the Age theme by weapon: javelins, axes, darts, fire arrows, stones, rockets, volleys. */
function drawProjectile(c: CanvasRenderingContext2D, key: string, x: number, y: number, r: number, now: number): void {
  const dx = Math.cos(r), dy = Math.sin(r);
  const shaft = (len: number, color: string, width: number) => {
    c.strokeStyle = color;
    c.lineWidth = width;
    c.beginPath();
    c.moveTo(x - dx * len, y - dy * len);
    c.lineTo(x, y);
    c.stroke();
  };
  const flame = (fx: number, fy: number, size: number) => {
    const g = c.createRadialGradient(fx, fy, 0, fx, fy, size);
    g.addColorStop(0, "rgba(255,240,150,0.95)");
    g.addColorStop(0.5, "rgba(255,120,20,0.8)");
    g.addColorStop(1, "rgba(160,30,0,0)");
    c.fillStyle = g;
    c.beginPath();
    c.arc(fx, fy, size, 0, Math.PI * 2);
    c.fill();
  };
  const stone = (size: number, color: string) => {
    c.fillStyle = color;
    c.strokeStyle = "#2e2a24";
    c.lineWidth = 0.5;
    c.beginPath();
    c.arc(x, y, size, 0, Math.PI * 2);
    c.fill();
    c.stroke();
  };
  if (key.startsWith("hyper_v")) {
    // a spinning throwing axe
    const a = now / 60;
    c.strokeStyle = "#5a3b1a";
    c.lineWidth = 1;
    c.beginPath();
    c.moveTo(x - Math.cos(a) * 3, y - Math.sin(a) * 3);
    c.lineTo(x + Math.cos(a) * 3, y + Math.sin(a) * 3);
    c.stroke();
    c.fillStyle = "#c9c9c9";
    c.beginPath();
    c.arc(x + Math.cos(a) * 3, y + Math.sin(a) * 3, 1.6, 0, Math.PI * 2);
    c.fill();
  } else if (key.startsWith("hyper_x")) {
    shaft(4, "#5a3b1a", 0.8);
    c.fillStyle = "#d9d9d9";
    c.fillRect(x - 0.8, y - 0.8, 1.6, 1.6);
  } else if (key.startsWith("merculite")) {
    shaft(6, "#5a3b1a", 0.8);
    flame(x, y, 2.6);
  } else if (key.startsWith("scatter_pack")) {
    // a hail of arrows
    for (let i = -1; i <= 1; i++) {
      const ox = -dy * i * 2.5, oy = dx * i * 2.5;
      c.strokeStyle = "#4a3218";
      c.lineWidth = 0.6;
      c.beginPath();
      c.moveTo(x + ox - dx * 5, y + oy - dy * 5);
      c.lineTo(x + ox, y + oy);
      c.stroke();
    }
  } else if (key.startsWith("pulson") || key === "anti_matter_torp") {
    stone(2.2, "#8a8378");
  } else if (key.startsWith("hercular")) {
    stone(3, "#7a7366");
  } else if (key === "hellfire_torpedo") {
    stone(2.6, "#5a4a3a");
    flame(x - dx * 2, y - dy * 2, 3.5);
  } else if (key.startsWith("zeon") || key === "proton_torpedo") {
    // a rocket and its sparks
    shaft(5, "#7a2a1a", 1.4);
    flame(x - dx * 6, y - dy * 6, 2.4);
  } else if (key === "plasma_torpedo") {
    // a fire ship: a burning hulk
    c.fillStyle = "#4a2e16";
    c.beginPath();
    c.ellipse(x, y, 4, 2, r, 0, Math.PI * 2);
    c.fill();
    flame(x, y - 1, 4);
  } else {
    // javelins and barbed javelins
    shaft(8, "#5a3b1a", 1);
    c.fillStyle = "#cfcfcf";
    c.beginPath();
    c.moveTo(x + dx * 2, y + dy * 2);
    c.lineTo(x - dy * 1.2, y + dx * 1.2);
    c.lineTo(x + dy * 1.2, y - dx * 1.2);
    c.closePath();
    c.fill();
  }
}
