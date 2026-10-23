// The ground battle screen (1oom `uiground.c`): the invaders on the left, the defenders on
// the right, falling one by one in the order the game resolved the fight.

import { audio, TUNE } from "./audio";
import type { Ctx } from "./ctx";
import { techRefName } from "./diplomacy";
import { shipParts } from "./engine";
import type { GameEvent, TechRef } from "./types";
import { PLANET_TYPES, RACES } from "./types";
import { h } from "./ui";

const W = 320;
const H = 200;
const SCALE = 2;

interface GroundEvent extends GameEvent {
  planet: number;
  attacker: number;
  defender: number;
  won: boolean;
  attackers: number;
  defenders: number;
  landed_of: number;
  rebels: boolean;
  deaths: number[];
  gear: [TechRef[], TechRef[]];
  armor: [number, number];
  captured?: TechRef[];
  factories: number;
}

/** 1oom `ground_draw_item`: where soldier `i` of `n` stands (left side). */
function spot(i: number, n: number): [number, number] {
  const rows = Math.ceil(n / 15);
  const y0 = 120 - rows * 6;
  const row = Math.floor(i / 15);
  const col = i % 15;
  return [10 + col * 8 + (row & 1) * 4, y0 + (col & 1) * 2 + (row & 1) * 6 + Math.floor(row / 2) * 12];
}

export function isGroundScreen(e: GameEvent, player: number): e is GroundEvent {
  return e.type === "ground_battle" && Array.isArray(e.deaths) && (e.deaths as number[]).length > 0 && (e.attacker === player || e.defender === player);
}

export async function groundScreen(ctx: Ctx, e: GroundEvent): Promise<void> {
  const { theme } = ctx;
  const v = ctx.view();
  const t = (k: string, p?: Record<string, string | number>) => theme.t(k, p);
  const p = v.planets[e.planet];
  const raceName = (i: number) => t(`race.${v.empires[i].race}.plural`);
  const att = raceName(e.attacker);
  const def = raceName(e.defender);
  const canvas = h("canvas", { width: W * SCALE, height: H * SCALE, class: "ground-canvas" }) as HTMLCanvasElement;
  const g = canvas.getContext("2d")!;
  g.imageSmoothingEnabled = false;
  const status = h("div", { class: "ground-status" });
  // 1oom `game_ground_show_init`: armor and battle suit (or combat armor), shield, weapon.
  const gear = (side: 0 | 1) => {
    const [suit, shield, weapon] = e.gear[side] ?? [];
    const name = (r?: TechRef) => (r && r[1] ? techRefName(ctx, r) : null);
    const armorKey = shipParts().armor[(e.armor?.[side] ?? 0) * 2]?.key ?? "titanium";
    return [`${t(`part.armor.${armorKey}`)} ${name(suit) ?? t("ground.combat_armor")}`, name(shield), name(weapon)].filter(Boolean).join(", ");
  };
  const sides = h(
    "div",
    { class: "ground-sides" },
    h("div", {}, h("b", {}, t("ground.troops", { race: att })), h("div", { class: "small muted" }, gear(0))),
    h("div", {}, h("b", {}, e.rebels ? t("ground.rebels") : t("ground.troops", { race: def })), h("div", { class: "small muted" }, gear(1))),
  );
  const skip = h("button", {}, t("ground.skip"));
  const ok = h("button", { class: "primary", disabled: true }, t("btn.ok"));
  const overlay = h(
    "div",
    { class: "overlay" },
    h(
      "div",
      { class: "dialog ground", role: "dialog" },
      h("h2", {}, t("ground.title", { planet: p.name })),
      h("div", { class: "dialog-body" }, h("div", { class: "ground-stage" }, canvas, status), sides),
      h("div", { class: "dialog-buttons" }, ok, skip),
    ),
  );
  document.body.append(overlay);

  const bg = theme.img(`landing/invade/${PLANET_TYPES.indexOf(p.kind ?? "terran")}`);
  const soldierAlias = (i: number) => `ground/soldier/${RACES.indexOf(v.empires[i].race)}`;
  const soldier = (i: number) => theme.img(soldierAlias(i));
  const deathImg = theme.img("ground/death");
  /** Painted pictures (not MOO1's pixel art) are drawn at a fixed height, smoothed. */
  const painted = (alias: string) => !theme.pixel.has(alias);
  const world = theme.def.map !== "space";
  const left = [e.attacker, e.attackers];
  const right = [e.defender, e.defenders];
  let alive = [e.attackers, e.defenders];
  let fallen: [number, number] | null = null;

  const drawBackground = () => {
    if (bg) {
      g.drawImage(bg, 0, 0, W * SCALE, H * SCALE);
      return;
    }
    const sky = g.createLinearGradient(0, 0, 0, H * SCALE);
    if (world) {
      sky.addColorStop(0, "#7fa6c9");
      sky.addColorStop(0.55, "#d9c99a");
      sky.addColorStop(0.56, "#6b7f3a");
      sky.addColorStop(1, "#3f4f22");
    } else {
      sky.addColorStop(0, "#050815");
      sky.addColorStop(0.55, "#1b2a44");
      sky.addColorStop(0.56, "#4a4038");
      sky.addColorStop(1, "#2a231d");
    }
    g.fillStyle = sky;
    g.fillRect(0, 0, W * SCALE, H * SCALE);
    // Walls of the city in the distance.
    g.fillStyle = world ? "#5b5148" : "#3a4658";
    for (let x = 90; x < 230; x += 14) {
      const hgt = 18 + ((x * 7) % 13);
      g.fillRect(x * SCALE, (110 - hgt) * SCALE, 12 * SCALE, hgt * SCALE);
    }
  };

  const drawSoldier = (owner: number, x: number, y: number, flip: boolean) => {
    const img = soldier(owner);
    g.save();
    if (flip) {
      g.translate((x + 8) * SCALE, 0);
      g.scale(-1, 1);
      x = 0;
    }
    if (img && painted(soldierAlias(owner))) {
      const hgt = 20;
      const wid = (img.width / img.height) * hgt;
      g.imageSmoothingEnabled = true;
      g.drawImage(img, (x + 4 - wid / 2) * SCALE, (y + 16 - hgt) * SCALE, wid * SCALE, hgt * SCALE);
    } else if (img) {
      g.drawImage(img, x * SCALE, y * SCALE, img.width * SCALE, img.height * SCALE);
    } else {
      const c = theme.banner(v.empires[owner].banner);
      g.fillStyle = "#1a1410";
      g.fillRect((x + 2) * SCALE, (y + 2) * SCALE, 4 * SCALE, 4 * SCALE);
      g.fillStyle = c;
      g.fillRect((x + 1) * SCALE, (y + 6) * SCALE, 6 * SCALE, 6 * SCALE);
      g.fillStyle = "#2a2018";
      g.fillRect((x + 2) * SCALE, (y + 12) * SCALE, 1 * SCALE, 4 * SCALE);
      g.fillRect((x + 5) * SCALE, (y + 12) * SCALE, 1 * SCALE, 4 * SCALE);
      g.fillStyle = world ? "#c9c3b8" : "#9fe0ff";
      g.fillRect((x + 7) * SCALE, (y + 1) * SCALE, 1 * SCALE, 11 * SCALE);
    }
    g.restore();
  };

  const draw = () => {
    drawBackground();
    for (let i = 0; i < alive[0]; i++) {
      const [x, y] = spot(i, left[1]);
      drawSoldier(left[0], x, y, false);
    }
    for (let i = 0; i < alive[1]; i++) {
      const [x, y] = spot(i, right[1]);
      drawSoldier(right[0], 295 - x, y, true);
    }
    if (fallen) {
      const [side, i] = fallen;
      let [x, y] = spot(i, side === 0 ? left[1] : right[1]);
      if (side === 1) x = 295 - x - 5;
      if (deathImg && painted("ground/death")) {
        const wid = 18;
        const hgt = (deathImg.height / deathImg.width) * wid;
        g.imageSmoothingEnabled = true;
        g.drawImage(deathImg, (x - 5) * SCALE, (y + 16 - hgt) * SCALE, wid * SCALE, hgt * SCALE);
      } else if (deathImg) {
        const fw = Math.min(deathImg.width, 16);
        g.drawImage(deathImg, 0, 0, fw, deathImg.height, x * SCALE, y * SCALE, fw * SCALE, deathImg.height * SCALE);
      } else {
        g.fillStyle = "rgba(255,180,60,0.85)";
        g.beginPath();
        g.arc((x + 4) * SCALE, (y + 8) * SCALE, 6 * SCALE, 0, Math.PI * 2);
        g.fill();
      }
    }
    g.fillStyle = "#fff";
    g.strokeStyle = "#000";
    g.lineWidth = 3;
    g.font = `bold ${14}px ${getComputedStyle(document.documentElement).getPropertyValue("--font")}`;
    for (const [txt, x] of [[`${alive[0]}`, 10], [`${alive[1]}`, 300]] as [string, number][]) {
      g.textAlign = x < 100 ? "left" : "right";
      g.strokeText(txt, x * SCALE, 150 * SCALE);
      g.fillText(txt, x * SCALE, 150 * SCALE);
    }
  };

  status.textContent = e.rebels
    ? t("ground.reclaim", { n: e.attackers })
    : t("ground.landed", { n: e.attackers, sent: Math.max(e.landed_of, e.attackers), race: att, defender: def });
  audio.play(TUNE.ground);
  draw();

  let skipping = false;
  skip.onclick = () => (skipping = true);
  await new Promise((r) => setTimeout(r, 900));
  const per = Math.max(15, Math.min(80, 5000 / e.deaths.length));
  let sinceSound = 0;
  for (const d of e.deaths) {
    alive[d] -= 1;
    fallen = [d, alive[d]];
    if (skipping) continue;
    if (++sinceSound >= 3) {
      sinceSound = 0;
      audio.sfx("hit", 2);
    }
    draw();
    await new Promise((r) => setTimeout(r, per));
  }
  fallen = null;
  alive = [e.attackers_left as number, e.defenders_left as number];
  draw();
  const winner = e.won ? att : e.rebels ? t("ground.rebels") : def;
  const lines: string[] = [];
  if (e.won) lines.push(e.rebels ? t("ground.recaptured", { planet: p.name }) : t("ground.captured", { race: winner, planet: p.name }));
  else lines.push(t("ground.defended", { race: winner, planet: p.name }));
  if (e.won && !e.rebels && e.factories > 0) lines.push(t("ground.factories", { n: e.factories }));
  const captured = e.captured ?? [];
  if (captured.length) {
    lines.push(e.attacker === v.player ? `${t("ground.tech_found")} ${captured.map((r) => techRefName(ctx, r)).join(", ")}` : t("ground.tech_lost", { n: captured.length }));
  }
  status.textContent = lines.join(" ");
  const ours = e.won === (e.attacker === v.player);
  audio.play(ours ? TUNE.ground_win : TUNE.ground_lose);
  skip.remove();
  ok.disabled = false;
  ok.focus();
  await new Promise<void>((resolve) => {
    const done = () => {
      document.removeEventListener("keydown", onKey);
      overlay.remove();
      resolve();
    };
    const onKey = (k: KeyboardEvent) => {
      if (k.key === "Enter" || k.key === "Escape") done();
    };
    document.addEventListener("keydown", onKey);
    ok.onclick = done;
  });
}
