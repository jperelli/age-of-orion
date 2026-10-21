// What a technology gives, in words (the effects come from the core, `techinfo.rs`).

import type { Ctx } from "./ctx";
import { shipParts } from "./engine";
import type { FieldView, TechEffect } from "./types";

const HULLS = ["small", "medium", "large", "huge"];

function num(x: number): string {
  return Number.isInteger(x) || x >= 10 ? String(Math.round(x)) : x.toFixed(1);
}

function bc(tenths: number): string {
  return num(tenths / 10);
}

/** "a–b" over the four hull sizes (or one number when they are all equal). */
function span(v: number[] | undefined, div = 1): string {
  if (!v || !v.length) return "0";
  const a = v[0] / div;
  const b = v[v.length - 1] / div;
  return a === b ? num(a) : `${num(a)}–${num(b)}`;
}

/** One line per effect. */
export function effectLines(ctx: Ctx, field: FieldView["field"], effects: TechEffect[]): string[] {
  const t = (k: string, p?: Record<string, string | number>) => ctx.theme.t(k, p);
  const parts = shipParts() as unknown as Record<string, Record<string, unknown>[]>;
  const unit = t("unit.bc");
  const lines: string[] = [];
  for (const e of effects) {
    switch (e.type) {
      case "level":
        lines.push(t("effect.level", { field: t(`field.${field}`), from: e.from, to: e.to }));
        break;
      case "value": {
        const scale = e.what === "ind_waste_scale" ? 10 : 1;
        const p = { from: e.from * scale, to: e.to * scale, bc: unit, diff: Math.abs(e.to - e.from) * scale };
        const line = t(`effect.${e.what}`, p);
        lines.push(e.superseded ? t("effect.superseded", { text: line }) : line);
        break;
      }
      case "gain":
        lines.push(t(`effect.${e.what}`, { bc: unit, cost: e.cost }));
        break;
      case "colonize": {
        const line = t("effect.colonize", { kind: t(`planet.${e.kind}`) });
        lines.push(e.superseded ? t("effect.superseded", { text: line }) : line);
        break;
      }
      case "part": {
        const table = { weapon: "weapons", comp: "comps", jammer: "jammers", shield: "shields", armor: "armor", engine: "engines", special: "specials" }[e.kind];
        const part = parts[table]?.[e.index] as Record<string, unknown> | undefined;
        if (!part) break;
        const key = String(part.key);
        const name = t(`part.${e.kind === "comp" ? "comp" : e.kind}.${key}`);
        const cost = part.cost as number | number[];
        const space = part.space as number | number[];
        const costs = Array.isArray(cost)
          ? t("effect.part_costs", { cost: span(cost, 10), space: span(space as number[]), bc: unit, hulls: HULLS.map((x) => t(`part.hull.${x}`)).join("/") })
          : t("effect.part_cost", { cost: bc(cost), space: Number(space), bc: unit });
        let what = "";
        if (e.kind === "weapon") {
          const dmg = part.damage_min === part.damage_max ? `${part.damage_max}` : `${part.damage_min}–${part.damage_max}`;
          const shots = Number(part.shots) > 0 ? t("effect.weapon_shots", { n: Number(part.shots) }) : "";
          const extra = ctx.theme.has(`part.weapon.${key}.info`) ? t(`part.weapon.${key}.info`) : "";
          what = t(part.bomb || part.bio ? "effect.bomb" : "effect.weapon", { damage: dmg, range: Number(part.range) });
          what = [what, shots, extra].filter(Boolean).join(", ");
        } else if (e.kind === "comp") what = t("effect.comp", { n: Number(part.level) });
        else if (e.kind === "jammer") what = t("effect.jammer", { n: Number(part.level) });
        else if (e.kind === "shield") what = t("effect.shield", { n: Number(part.absorb) });
        else if (e.kind === "armor") what = t("effect.armor", { pct: Number(part.armor) });
        else if (e.kind === "engine") what = t("effect.engine", { n: Number(part.warp) });
        else if (e.kind === "special") what = ctx.theme.has(`part.special.${key}.info`) ? t(`part.special.${key}.info`) : "";
        const kind = t(`effect.kind.${e.kind}`);
        const named = kind.toLowerCase().split(/\s+/).some((w) => w.length > 2 && name.toLowerCase().includes(w));
        lines.push(t(named ? "effect.part_named" : "effect.part", { kind, name, what, costs }));
        break;
      }
    }
  }
  return lines;
}
