// The research screens, as in MOO1 (1oom `uitech.c`, `uinewtech.c`): the TECH screen shows the
// techs known in each field on the left and shares research between the fields on the right;
// a field's next project is chosen in its own dialog when it has none, and stays until it is
// discovered (investment earns interest and a breakthrough becomes likely once it exceeds the
// cost). The technology tree shows what is known, and nothing beyond.
import type { Ctx } from "./ctx";
import { moo1Scene, placeholder } from "./pictures";
import { sliders } from "./sliders";
import { effectLines } from "./techinfo";
import { shipParts } from "./engine";
import { FIELDS, RACES, type FieldView, type GameEvent, type TechChoice } from "./types";
import { clear, h, keepScroll, modal } from "./ui";

export function techName(ctx: Ctx, f: FieldView | string, key: string, tech?: number): string {
  const field = typeof f === "string" ? f : f.field;
  return key === "advanced" ? ctx.theme.t("tech.advanced", { n: tech ?? 0 }) : ctx.theme.t(`tech.${field}.${key}`);
}

/** Description of a tech (from RESEARCH.LBX in the Classic theme, or a mod): `techinfo.<field>.<n>`. */
function info(ctx: Ctx, field: number, tech: number): string | null {
  const key = `techinfo.${field}.${tech}`;
  return tech > 0 && ctx.theme.has(key) ? ctx.theme.t(key) : null;
}

/** MOO1's blueprint of a tech (TECHNO.LBX, imported), if any. */
function techPicture(ctx: Ctx, field: number, tech: number): HTMLElement | null {
  const url = ctx.theme.url(`tech/${field}/${tech > 50 ? "advanced" : tech}`);
  return url ? h("img", { class: "tech-pic pixel", src: url, alt: "" }) : null;
}

/** A tech with its cost and what it gives. */
function techCard(ctx: Ctx, f: FieldView, c: TechChoice, cost: boolean, cls = ""): HTMLElement {
  const t = (k: string, p?: Record<string, string | number>) => ctx.theme.t(k, p);
  const lines = effectLines(ctx, f.field, c.effects);
  const d = info(ctx, FIELDS.indexOf(f.field), c.tech);
  return h(
    "div",
    { class: `tech-card ${cls}` },
    techPicture(ctx, FIELDS.indexOf(f.field), c.tech),
    h("div", { class: "tech-head" }, h("b", {}, techName(ctx, f, c.key, c.tech)), cost ? h("span", { class: "muted" }, ` ${c.cost} ${t("unit.rp")}`) : null),
    lines.length ? h("ul", { class: "effects" }, ...lines.map((l) => h("li", {}, l))) : null,
    d ? h("p", { class: "muted small techinfo" }, d) : null,
  );
}

/** Percent of the project's cost invested (a breakthrough may come from 100% on). */
function progress(f: FieldView): number {
  return f.project !== 0 && f.cost > 0 ? Math.floor((f.invested * 100) / f.cost) : 0;
}

export function needsChoice(ctx: Ctx): boolean {
  return ctx.view().tech.some((f) => f.project === 0 && f.choices.length > 0);
}

/** MOO1's new-tech dialog: the next project of field `i`, chosen among what can be researched. */
async function chooseTech(ctx: Ctx, i: number): Promise<void> {
  const t = (k: string, p?: Record<string, string | number>) => ctx.theme.t(k, p);
  const f = ctx.view().tech[i];
  if (f.project !== 0 || !f.choices.length) return;
  let close: (i: number) => void = () => {};
  const list = h(
    "div",
    { class: "tech-choices" },
    ...[...f.choices]
      .sort((a, b) => a.cost - b.cost)
      .map((c) =>
        h(
          "div",
          { class: "tech-choice" },
          techCard(ctx, f, c, true),
          h(
            "button",
            {
              class: "primary",
              onclick: () => {
                ctx.command({ type: "set_research_project", field: i, tech: c.tech });
                close(0);
              },
            },
            t("research.pick"),
          ),
        ),
      ),
  );
  // MOO1's laboratory with the race's scientist at work (1oom `uinewtech.c`), when imported.
  const v = ctx.view();
  const lab = moo1Scene(ctx.theme, [{ alias: "newtech/lab/0" }, { alias: `anim/newtech/${RACES.indexOf(v.empires[v.player].race)}`, fps: 6 }], 1.5);
  await modal(
    t("research.new_title", { field: t(`field.${f.field}`) }),
    [lab, h("p", {}, t("research.new_text", { field: t(`field.${f.field}`) })), list],
    [t("research.later")],
    "wide",
    (c) => (close = c),
  );
}

/** 1oom `TECHSOURCE_*` of a discovery: the laboratory and the scientist shown. */
const SOURCE_SCENE: Record<string, number> = { research: 0, espionage: 1, artifacts: 2, orion: 2, trade: 4 };

/**
 * MOO1's new-tech screen (1oom `ui_newtech`), with MOO1's pictures imported: the laboratory
 * (the spies' den for a stolen tech, the ruins for a found one) with the tech's blueprint on
 * the table, the scientist (or spy) at work, then what the tech gives. A theme without the
 * pictures shows the same screen with a placeholder in their place.
 */
export async function newTechScreen(ctx: Ctx, e: GameEvent, text: string): Promise<void> {
  const v = ctx.view();
  const field = FIELDS.indexOf(e.field as FieldView["field"]);
  const tech = e.tech as number;
  const source = SOURCE_SCENE[e.source as string] ?? 0;
  const race = RACES.indexOf(v.empires[source === 4 && typeof e.from === "number" ? e.from : v.player].race);
  const picture = `tech/${field}/${tech > 50 ? "advanced" : tech}`;
  if (field < 0) return;
  const scene =
    (ctx.theme.img(picture) &&
      moo1Scene(
        ctx.theme,
        [{ alias: `newtech/lab/${source === 4 ? 0 : source}` }, { alias: picture, x: 145, y: 54 }, { alias: `anim/newtech/${source === 4 ? race : source * 10 + race}`, fps: 6 }],
        1.5,
      )) ||
    placeholder(ctx.theme, picture, 480, 300);
  const f = v.tech[field];
  const c = f.completed.find((x) => x.tech === tech);
  const key = tech > 50 ? "advanced" : (shipParts().techs?.[field]?.[tech] ?? "");
  await modal(
    ctx.theme.t("newtech.title"),
    [scene, h("p", {}, text), c ? techCard(ctx, f, c, false, "current no-pic") : h("h3", {}, techName(ctx, f, key, tech))],
    [ctx.theme.t("btn.ok")],
    "wide",
  );
}

/** The new-tech dialogs of every field that needs a project. */
export async function chooseTechs(ctx: Ctx): Promise<void> {
  for (let i = 0; i < FIELDS.length; i++) await chooseTech(ctx, i);
}

/** MOO1's TECH screen. */
export function techScreen(ctx: Ctx): Promise<number> {
  const t = (k: string, p?: Record<string, string | number>) => ctx.theme.t(k, p);
  const body = h("div", { class: "tech-screen" });
  let field = 0;
  let picked: number | null = null;
  const render = () => keepScroll(body, () => draw());
  const draw = () => {
    const v = ctx.view();
    const f = v.tech[field];
    // Left half: the fields, what is known in the selected one, and what the selected tech gives.
    const fields = h(
      "div",
      { class: "tech-fields" },
      ...v.tech.map((x, i) =>
        h(
          "button",
          { class: i === field ? "primary" : "", onclick: () => ((field = i), (picked = null), render()) },
          t(`field.${x.field}`),
          h("small", {}, ` ${x.level}`),
        ),
      ),
    );
    const known = [...f.completed].sort((a, b) => a.tech - b.tech);
    const current = f.choices.find((c) => c.tech === f.project) ?? null;
    const entries = [...known, ...(current ? [current] : [])];
    const sel = entries.find((c) => c.tech === picked) ?? current ?? known[known.length - 1] ?? null;
    const list = h(
      "ol",
      { class: "tech-list" },
      ...entries.map((c) =>
        h(
          "li",
          {
            class: [c === current ? "current" : "", c === sel ? "selected" : ""].join(" "),
            onclick: () => ((picked = c.tech), render()),
          },
          techName(ctx, f, c.key, c.tech),
          c === current ? h("span", { class: "muted" }, ` · ${t("research.in_progress", { pct: progress(f) })}`) : null,
        ),
      ),
    );
    const left = h(
      "div",
      { class: "tech-left" },
      fields,
      h("div", { class: "tech-level muted small" }, t("research.level", { n: f.level }), " · ", t("research.known", { n: f.completed.length })),
      list,
      h("div", { class: "tech-desc" }, sel ? techCard(ctx, f, sel, sel === current, sel === current ? "current" : "known-tech") : null),
      h("button", { class: "tree-button", onclick: () => void techTree(ctx) }, t("research.tree")),
    );
    // Right half: research shared between the fields, each with its project.
    const group = sliders({
      specs: v.tech.map((x, i) => ({ label: t(`field.${FIELDS[i]}`), value: x.alloc, locked: x.locked })),
      total: 100,
      lockTitle: t("slider.lock_hint"),
      format: (i, a) => `${Math.floor((v.research * a[i]) / 100)} ${t("unit.rp")}`,
      onCommit: (a) => {
        ctx.command({ type: "set_research", alloc: a });
        render();
      },
      onLock: (i, locked) => ctx.command({ type: "lock_research", field: i, locked }),
    });
    [...group.children].forEach((row, i) => {
      const x = v.tech[i];
      const project = x.choices.find((c) => c.tech === x.project);
      const pct = progress(x);
      const line = project
        ? h(
            "div",
            { class: "tech-project", onclick: () => ((field = i), (picked = project.tech), render()) },
            h("span", {}, techName(ctx, x, project.key, project.tech)),
            h("span", { class: pct >= 100 ? "good" : "muted" }, pct >= 100 ? t("research.breakthrough") : `${pct}%`),
            h("div", { class: "progress" }, h("div", { style: `width:${Math.min(100, pct)}%` })),
          )
        : x.choices.length
          ? h(
              "div",
              { class: "tech-project" },
              h("button", { class: "mini attention", onclick: async () => (await chooseTech(ctx, i), render()) }, t("research.choose")),
              x.invested ? h("span", { class: "muted small" }, t("research.saved", { rp: x.invested })) : null,
            )
          : h("div", { class: "tech-project muted small" }, t("research.nothing"));
      row.after(line);
    });
    const a = v.abilities;
    const right = h(
      "div",
      { class: "tech-right" },
      group,
      h("p", { class: "tech-total" }, t("research.total", { rp: v.research })),
      h(
        "ul",
        { class: "muted small abilities" },
        h("li", {}, t("research.range", { n: a.fuel_range })),
        h("li", {}, t("research.speed", { n: a.engine_warp })),
        h("li", {}, t("research.workers", { n: a.colonist_oper_factories })),
        h("li", {}, t("research.colonize", { kind: t(`planet.${a.colony_for}`) })),
      ),
    );
    clear(body).append(left, right);
  };
  render();
  return modal(t("research.screen_title"), body, [t("btn.close")], "huge tech");
}

const SVG = "http://www.w3.org/2000/svg";

function svg<K extends keyof SVGElementTagNameMap>(tag: K, attrs: Record<string, string | number>, text?: string): SVGElementTagNameMap[K] {
  const e = document.createElementNS(SVG, tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, String(v));
  if (text !== undefined) e.textContent = text;
  return e;
}

/** A tech name on at most two lines. */
function twoLines(name: string, width = 22): string[] {
  if (name.length <= width) return [name];
  const words = name.split(" ");
  let a = "";
  while (words.length && (a + " " + words[0]).trim().length <= width) a = (a + " " + words.shift()).trim();
  const b = words.join(" ");
  return [a || name.slice(0, width), b.length > width ? b.slice(0, width - 1) + "…" : b];
}

/**
 * The technology tree: one branch per field, the tiers of five levels from left to right, and
 * on each tier the techs known (the project in progress dashed). Nothing not yet discovered.
 */
async function techTree(ctx: Ctx): Promise<void> {
  const t = (k: string, p?: Record<string, string | number>) => ctx.theme.t(k, p);
  const v = ctx.view();
  const tierOf = (tech: number) => Math.ceil(tech / 5);
  const lanes = v.tech.map((f) => {
    const nodes = [...f.completed].sort((a, b) => a.tech - b.tech).map((c) => ({ c, current: false }));
    const cur = f.choices.find((c) => c.tech === f.project);
    if (cur) nodes.push({ c: cur, current: true });
    const byTier = new Map<number, typeof nodes>();
    for (const n of nodes) byTier.set(tierOf(n.c.tech), [...(byTier.get(tierOf(n.c.tech)) ?? []), n]);
    return { f, byTier, rows: Math.max(1, ...[...byTier.values()].map((x) => x.length)) };
  });
  const tiers = Math.max(1, ...lanes.flatMap((l) => [...l.byTier.keys()]));
  const [W, NH, GAP, HEAD] = [158, 34, 6, 130];
  const laneH = lanes.map((l) => l.rows * (NH + GAP) + 18);
  const width = HEAD + tiers * W + 10;
  const height = laneH.reduce((a, b) => a + b, 0) + 24;
  const root = svg("svg", { width, height, class: "tech-tree-svg" });
  for (let k = 1; k <= tiers; k++) {
    root.append(svg("text", { x: HEAD + (k - 1) * W + (W - 10) / 2, y: 14, "text-anchor": "middle", class: "tier" }, t("research.tier", { a: k * 5 - 4, b: k * 5 })));
  }
  const detail = h("div", { class: "tech-desc" }, h("p", { class: "muted small" }, t("research.tree_hint")));
  let y = 24;
  lanes.forEach((l, li) => {
    const top = y;
    const mid = top + NH / 2 + 6;
    root.append(svg("text", { x: 4, y: mid + 4, class: "lane" }, t(`field.${l.f.field}`)));
    const last = Math.max(0, ...l.byTier.keys());
    if (last > 0) root.append(svg("line", { x1: HEAD - 12, y1: mid, x2: HEAD + (last - 1) * W + 8, y2: mid, class: "trunk" }));
    for (const [k, nodes] of l.byTier) {
      nodes.forEach((n, j) => {
        const x = HEAD + (k - 1) * W + 8;
        const ny = top + 6 + j * (NH + GAP);
        root.append(svg("path", { d: `M ${x} ${mid} V ${ny + NH / 2} H ${x + 6}`, class: "branch" }));
        const g = svg("g", { class: `node${n.current ? " current" : ""}`, transform: `translate(${x + 6} ${ny})` });
        g.append(svg("rect", { width: W - 24, height: NH, rx: 4 }));
        const lines = twoLines(techName(ctx, l.f, n.c.key, n.c.tech));
        lines.forEach((s, i) => g.append(svg("text", { x: 6, y: lines.length === 1 ? NH / 2 + 4 : 14 + i * 13 }, s)));
        g.addEventListener("click", () => detail.replaceChildren(techCard(ctx, l.f, n.c, n.current, n.current ? "current" : "known-tech")));
        root.append(g);
      });
    }
    y += laneH[li];
  });
  await modal(t("research.tree_title"), h("div", { class: "tree-wrap" }, h("div", { class: "tech-tree" }, root), detail), [t("btn.close")], "huge");
}
