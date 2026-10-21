// Research screen: choose the project of each field, share research between fields and see
// what is already known (1oom research rules: investment earns interest and a breakthrough
// becomes likely once it exceeds the cost).
import type { Ctx } from "./ctx";
import { sliders } from "./sliders";
import { effectLines } from "./techinfo";
import { FIELDS, type FieldView, type TechChoice } from "./types";
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

function techInfo(ctx: Ctx, field: number, tech: number): HTMLElement | null {
  const d = info(ctx, field, tech);
  return d ? h("p", { class: "muted small techinfo" }, d) : null;
}

/** A tech with its cost and what it gives. */
function techCard(ctx: Ctx, f: FieldView, c: TechChoice, cost: boolean, cls = ""): HTMLElement {
  const t = (k: string, p?: Record<string, string | number>) => ctx.theme.t(k, p);
  const lines = effectLines(ctx, f.field, c.effects);
  return h(
    "div",
    { class: `tech-card ${cls}` },
    h("div", { class: "tech-head" }, h("b", {}, techName(ctx, f, c.key, c.tech)), cost ? h("span", { class: "muted" }, ` ${c.cost} ${t("unit.rp")}`) : null),
    lines.length ? h("ul", { class: "effects" }, ...lines.map((l) => h("li", {}, l))) : null,
    techInfo(ctx, FIELDS.indexOf(f.field), c.tech),
  );
}

export function needsChoice(ctx: Ctx): boolean {
  return ctx.view().tech.some((f) => f.project === 0 && f.choices.length > 0);
}

export function researchDialog(ctx: Ctx): Promise<number> {
  const t = (k: string, p?: Record<string, string | number>) => ctx.theme.t(k, p);
  const body = h("div", { class: "research" });
  const title = () => t("research.title", { rp: ctx.view().research });
  const render = () => keepScroll(body, () => draw());
  const draw = () => {
    const v = ctx.view();
    clear(body).append(h("p", { class: "muted" }, t("research.text", { rp: v.research })));
    body.append(
      sliders({
        specs: v.tech.map((f, i) => ({ label: t(`field.${FIELDS[i]}`), value: f.alloc, locked: f.locked })),
        total: 100,
        lockTitle: t("slider.lock_hint"),
        format: (i, a) => `${a[i]}% · ${Math.floor((v.research * a[i]) / 100)} ${t("unit.rp")}`,
        onCommit: (a) => {
          ctx.command({ type: "set_research", alloc: a });
          render();
        },
        onLock: (i, locked) => ctx.command({ type: "lock_research", field: i, locked }),
      }),
    );
    if (v.tech.some((f) => f.project === 0 && f.choices.length > 0)) {
      body.append(
        h(
          "button",
          {
            class: "primary",
            onclick: () => {
              for (const [i, f] of ctx.view().tech.entries()) {
                const cheapest = [...f.choices].sort((a, b) => a.cost - b.cost)[0];
                if (f.project === 0 && cheapest) ctx.command({ type: "set_research_project", field: i, tech: cheapest.tech });
              }
              render();
            },
          },
          t("research.auto"),
        ),
      );
    }
    for (const [i, f] of v.tech.entries()) {
      const pct = f.project !== 0 && f.cost > 0 ? Math.min(100, Math.floor((f.invested * 100) / f.cost)) : 0;
      const select = h(
        "select",
        {
          class: f.project === 0 ? "attention" : "",
          onchange: (e: Event) => {
            ctx.command({ type: "set_research_project", field: i, tech: Number((e.target as HTMLSelectElement).value) });
            render();
          },
        },
        f.project === 0 ? h("option", { value: "", selected: true }, t("research.choose")) : null,
        ...f.choices.map((c) => h("option", { value: c.tech, selected: c.tech === f.project }, `${techName(ctx, f, c.key, c.tech)} (${c.cost} ${t("unit.rp")})`)),
      );
      const project = f.choices.find((c) => c.tech === f.project);
      const others = f.choices.filter((c) => c.tech !== f.project);
      const choices = others.length
        ? h("details", { class: "known" }, h("summary", {}, t("research.compare", { n: others.length })), ...others.map((c) => techCard(ctx, f, c, true)))
        : null;
      const known = h(
        "details",
        { class: "known" },
        h("summary", {}, t("research.known", { n: f.completed.length })),
        ...[...f.completed].reverse().map((c) => techCard(ctx, f, c, false, "known-tech")),
      );
      body.append(
        h(
          "div",
          { class: "research-field" },
          h("div", { class: "research-row" }, h("label", {}, t(`field.${FIELDS[i]}`), h("small", {}, t("research.level", { n: f.level }))), select),
          f.project !== 0
            ? h("div", { class: "progress", title: `${f.invested} / ${f.cost} ${t("unit.rp")}` }, h("div", { style: `width:${pct}%` }))
            : h("p", { class: "warn small" }, t("research.saved", { rp: f.invested })),
          project ? techCard(ctx, f, project, true, "current") : null,
          choices,
          known,
        ),
      );
    }
    const a = v.abilities;
    body.append(
      h("p", { class: "muted small" }, t("research.switch_warning")),
      h(
        "ul",
        { class: "muted abilities" },
        h("li", {}, t("research.range", { n: a.fuel_range })),
        h("li", {}, t("research.speed", { n: a.engine_warp })),
        h("li", {}, t("research.workers", { n: a.colonist_oper_factories })),
        h("li", {}, t("research.colonize", { kind: t(`planet.${a.colony_for}`) })),
      ),
    );
  };
  render();
  return modal(title(), body, [t("btn.close")], "wide");
}
