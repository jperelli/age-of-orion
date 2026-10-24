// Ship design screen, laid out like MOO1's (1oom `uidesign.c`): the ship's picture with arrows
// to choose another, the four hull sizes with theirs, then the parts. The rules
// (1oom game_design.c) live in the Rust core: this only edits a design and asks the core for
// its cost, space and combat figures.
import type { Ctx } from "./ctx";
import { shipParts } from "./engine";
import { designPicture } from "./pictures";
import type { DesignStats, DesignView, ShipDesign } from "./types";
import { append, clear, h, keepScroll, modal } from "./ui";

const MAX_DESIGNS = 6;
const ROMAN = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"];

function blank(name: string): ShipDesign {
  return {
    name,
    hull: 1,
    look: 6,
    weapons: [0, 0, 0, 0],
    weapon_counts: [0, 0, 0, 0],
    engine: 0,
    specials: [0, 0, 0],
    shield: 0,
    jammer: 0,
    comp: 0,
    armor: 0,
    man: 0,
  };
}

function copy(d: ShipDesign): ShipDesign {
  return {
    name: d.name,
    hull: d.hull,
    look: d.look,
    weapons: [...d.weapons],
    weapon_counts: [...d.weapon_counts],
    engine: d.engine,
    specials: [...d.specials],
    shield: d.shield,
    jammer: d.jammer,
    comp: d.comp,
    armor: d.armor,
    man: d.man,
  };
}

export function designDialog(ctx: Ctx): Promise<number> {
  const t = (k: string, p?: Record<string, string | number>) => ctx.theme.t(k, p);
  const parts = shipParts();
  const part = (kind: string, key: string) => t(`part.${kind}.${key}`);
  let slot: number | null = null;
  let d = blank(t("designer.new_name"));
  const list = h("div", { class: "design-list" });
  const editor = h("div", { class: "design-editor" });

  const weaponLabel = (i: number) => {
    const w = parts.weapons[i];
    const dmg = w.damage_min === w.damage_max ? `${w.damage_max}` : `${w.damage_min}-${w.damage_max}`;
    const info = ctx.theme.has(`part.weapon.${w.key}.info`) ? ` ${t(`part.weapon.${w.key}.info`)}` : "";
    return `${part("weapon", w.key)} (${dmg})${info}`;
  };
  const statsLine = (s: DesignStats, dv?: DesignView) =>
    t("designer.summary", { cost: s.cost, hp: s.hp, speed: s.speed, attack: s.attack_level, defense: s.defense, count: dv?.count ?? 0 });

  const renderList = () => {
    const v = ctx.view();
    clear(list).append(h("h3", {}, t("designer.current", { n: v.designs.length, max: MAX_DESIGNS })));
    const banner = v.empires[v.player].banner;
    for (const [i, dv] of v.designs.entries()) {
      list.append(
        h(
          "div",
          { class: `design-item ${slot === i ? "selected" : ""}` },
          h("img", { class: "design-pic", src: designPicture(ctx.theme, dv.look, banner), alt: "" }),
          h("b", {}, dv.name),
          h("span", { class: "muted" }, ` ${part("hull", parts.hulls[dv.hull].key)}`),
          h("div", { class: "small" }, statsLine(dv.stats, dv)),
          h(
            "div",
            { class: "row" },
            h("button", { class: "mini", onclick: () => ((slot = i), (d = copy(dv)), render()) }, t("designer.edit")),
            h("button", { class: "mini", onclick: () => ((slot = null), (d = copy(dv)), (d.name = `${dv.name} II`), render()) }, t("designer.copy")),
            h(
              "button",
              {
                class: "mini",
                disabled: v.designs.length <= 1,
                onclick: () => {
                  if (window.confirm(t("designer.scrap_confirm", { name: dv.name, count: dv.count }))) {
                    ctx.command({ type: "scrap_design", design: i });
                    slot = null;
                    render();
                  }
                },
              },
              t("designer.scrap"),
            ),
          ),
        ),
      );
    }
    list.append(h("button", { onclick: () => ((slot = null), (d = blank(t("designer.new_name"))), render()) }, t("designer.new")));
  };

  type Slot = { kind: "comp" | "shield" | "jammer" | "armor" | "engine" | "man" } | { kind: "weapon" | "special"; i: number };
  const get = (x: ShipDesign, s: Slot): number =>
    s.kind === "weapon" ? x.weapons[s.i] : s.kind === "special" ? x.specials[s.i] : x[s.kind];
  const put = (x: ShipDesign, s: Slot, n: number) => {
    if (s.kind === "weapon") {
      x.weapons[s.i] = n;
      x.weapon_counts[s.i] = n ? Math.max(1, x.weapon_counts[s.i]) : 0;
    } else if (s.kind === "special") x.specials[s.i] = n;
    else if (s.kind === "engine") {
      x.engine = n;
      x.man = Math.min(x.man, n);
    } else x[s.kind] = n;
  };
  /**
   * MOO1's list window for a slot (1oom `ui_design_sel_*`): every part known, with what it
   * does, what it adds to the cost and the space it takes (with the engines it needs); the
   * ones that do not fit are greyed out.
   */
  const pick = async (s: Slot, title: string, options: number[]) => {
    const base = copy(d);
    put(base, s, 0);
    if (s.kind === "weapon") base.weapon_counts[s.i] = 0;
    const before = ctx.game.designStats(base);
    const current = get(d, s);
    const head = (keys: string[]) => h("tr", {}, ...keys.map((k) => h("th", {}, t(`designer.col.${k}`))));
    const cols = s.kind === "weapon" ? ["max", "name", "damage", "range", "notes", "cost", "space"] : s.kind === "man" ? ["class", "speed", "cost", "space"] : ["name", "notes", "cost", "space"];
    let close: (i: number) => void = () => {};
    const rows = options.map((n) => {
      const trial = copy(base);
      put(trial, s, n);
      if (s.kind === "weapon" && n) trial.weapon_counts[s.i] = 1;
      const st = ctx.game.designStats(trial);
      const fits = st.free_space >= 0 || n === current;
      const cost = n ? `${st.cost - before.cost}` : "";
      const space = n || s.kind === "man" ? `${before.free_space - st.free_space}` : "";
      let cells: string[];
      if (s.kind === "weapon") {
        const w = parts.weapons[n];
        const dmg = !n ? "" : w.damage_min === w.damage_max ? `${w.damage_max}` : `${w.damage_min}-${w.damage_max}`;
        const notes = n && ctx.theme.has(`part.weapon.${w.key}.info`) ? t(`part.weapon.${w.key}.info`) : "";
        cells = [n ? `${st.max_weapons[s.i]}` : "", n ? part("weapon", w.key) : t("designer.none"), dmg, n ? `${w.range}` : "", notes, cost, space];
      } else if (s.kind === "man") {
        cells = [t("designer.class", { n: ROMAN[n] ?? n + 1 }), `${Math.floor((n + 3) / 2)}`, cost, space];
      } else {
        const [name, notes] = partInfo(s.kind, n);
        cells = [name, notes, cost, space];
      }
      return h(
        "tr",
        {
          class: `${n === current ? "selected" : ""} ${fits ? "clickable" : "disabled"}`,
          onclick: () => {
            if (!fits) return;
            put(d, s, n);
            close(0);
          },
        },
        ...cells.map((c) => h("td", {}, c)),
      );
    });
    const r = await modal(title, h("table", { class: "list part-list" }, head(cols), ...rows), [t("btn.cancel")], "wide part-picker", (c) => (close = c));
    if (r === 0) keepScroll(editor, renderEditor);
  };
  /** A part's name and what it does, for the list windows and the slots. */
  const partInfo = (kind: string, n: number): [string, string] => {
    if (kind === "comp") return n ? [part("comp", parts.comps[n].key), t("effect.comp", { n: parts.comps[n].level })] : [t("designer.none"), ""];
    if (kind === "shield") return n ? [part("shield", parts.shields[n].key), t("effect.shield", { n: parts.shields[n].absorb })] : [t("designer.none"), ""];
    if (kind === "jammer") return n ? [part("jammer", parts.jammers[n].key), t("effect.jammer", { n: parts.jammers[n].level })] : [t("designer.none"), ""];
    if (kind === "armor") return [part("armor", parts.armor[n].key), t("effect.armor", { pct: parts.armor[n].armor })];
    if (kind === "engine") return [part("engine", parts.engines[n].key), t("effect.engine", { n: parts.engines[n].warp })];
    if (kind === "special") {
      const key = parts.specials[n]?.key;
      return n ? [part("special", key), ctx.theme.has(`part.special.${key}.info`) ? t(`part.special.${key}.info`) : ""] : [t("designer.none"), ""];
    }
    return ["", ""];
  };
  /** A slot of the design: its name and current part; clicking opens the list window. */
  const slotButton = (label: string, s: Slot, options: number[]) => {
    const n = get(d, s);
    const name =
      s.kind === "weapon" ? (n ? weaponLabel(n) : t("designer.none")) : s.kind === "man" ? t("designer.class", { n: ROMAN[n] ?? n + 1 }) : partInfo(s.kind, n)[0];
    return h(
      "div",
      { class: "field part-slot" },
      h("span", {}, label),
      h("button", { class: "part-button", onclick: () => void pick(s, label, options) }, name, h("span", { class: "caret" }, "▾")),
    );
  };

  const renderEditor = () => {
    const v = ctx.view();
    const k = v.parts;
    let stats = ctx.game.designStats(d);
    for (let i = 0; i < 4; i++) {
      if (d.weapons[i] && d.weapon_counts[i] === 0 && stats.max_weapons[i] > 0) d.weapon_counts[i] = 1;
    }
    const maxLabels = [0, 1, 2, 3].map(() => h("span", {}));
    const counts = [0, 1, 2, 3].map((i) => h("input", { type: "number", min: 0, max: 99, value: d.weapon_counts[i], disabled: !d.weapons[i] }));
    const figures = h("div", { class: "figures" });
    const full = slot === null && v.designs.length >= MAX_DESIGNS;
    const save = h(
      "button",
      {
        class: "primary",
        onclick: () => {
          if (slot !== null && v.designs[slot].count > 0 && !window.confirm(t("designer.replace_confirm", { name: v.designs[slot].name, count: v.designs[slot].count }))) {
            return;
          }
          if (ctx.command({ type: "save_design", slot, design: d })) {
            slot = slot ?? ctx.view().designs.length - 1;
            render();
          }
        },
      },
      slot === null ? t("designer.add") : t("designer.replace"),
    );
    // Typing in a field only refreshes the figures, so the inputs keep their focus.
    const update = () => {
      stats = ctx.game.designStats(d);
      maxLabels.forEach((l, i) => (l.textContent = t("designer.count", { max: stats.max_weapons[i] })));
      const free = stats.free_space;
      const usedPct = Math.max(0, Math.min(100, Math.round(((stats.space - free) * 100) / Math.max(1, stats.space))));
      append(
        clear(figures),
        h("div", { class: `space-bar ${free < 0 ? "over" : ""}` }, h("div", { style: `width:${usedPct}%` })),
        h("p", { class: free < 0 ? "warn" : "muted" }, t("designer.space_left", { free, total: stats.space })),
        h("p", {}, statsLine(stats)),
        h("p", { class: "muted small" }, t("designer.defense", { def: stats.defense, mdef: stats.missile_defense, absorb: stats.absorb })),
        stats.error ? h("p", { class: "warn" }, t(`design.error.${stats.error}`)) : null,
      );
      save.disabled = !!stats.error || full;
    };
    const banner = v.empires[v.player].banner;
    const variant = (d.look % 24) % 6;
    const lookAt = (hull: number, n: number) => hull * 6 + ((n % 6) + 6) % 6;
    const turn = (dir: number) => {
      d.look = lookAt(d.hull, variant + dir);
      keepScroll(editor, renderEditor);
    };
    // MOO1: the ship's picture, with arrows for the six pictures of its hull size.
    const look = h(
      "div",
      { class: "ship-look" },
      h("button", { class: "mini", title: t("designer.look_prev"), onclick: () => turn(-1) }, "‹"),
      h("img", { src: designPicture(ctx.theme, lookAt(d.hull, variant), banner), alt: "", title: t("designer.look_next"), onclick: () => turn(1) }),
      h("button", { class: "mini", title: t("designer.look_next"), onclick: () => turn(1) }, "›"),
    );
    const hulls = h(
      "div",
      { class: "hull-buttons" },
      ...parts.hulls.map((hl, i) =>
        h(
          "button",
          {
            class: i === d.hull ? "primary" : "",
            onclick: () => {
              d.hull = i;
              d.look = lookAt(i, variant);
              keepScroll(editor, renderEditor);
            },
          },
          h("img", { src: designPicture(ctx.theme, lookAt(i, variant), banner), alt: "" }),
          h("span", {}, part("hull", hl.key)),
          h("small", {}, t("designer.space", { n: hl.space })),
        ),
      ),
    );
    const name = h("input", { type: "text", value: d.name, maxlength: 24 });
    name.addEventListener("input", () => ((d.name = name.value), update()));
    counts.forEach((c, i) =>
      c.addEventListener("input", () => {
        d.weapon_counts[i] = Math.max(0, Math.min(99, Math.floor(Number(c.value) || 0)));
        update();
      }),
    );
    const weapons = [0, 1, 2, 3].map((i) =>
      h("div", { class: "weapon-row" }, slotButton(t("designer.weapon", { n: i + 1 }), { kind: "weapon", i }, [0, ...k.weapons]), h("label", { class: "field count" }, maxLabels[i], counts[i])),
    );
    const specials = [0, 1, 2].map((i) => slotButton(t("designer.special", { n: i + 1 }), { kind: "special", i }, [0, ...k.specials]));
    append(
      clear(editor),
      h("h3", {}, slot === null ? t("designer.new_title") : t("designer.edit_title", { name: v.designs[slot].name })),
      h("div", { class: "design-top" }, look, h("div", {}, h("label", { class: "field" }, h("span", {}, t("designer.name")), name), hulls)),
      h(
        "div",
        { class: "form" },
        slotButton(t("designer.comp"), { kind: "comp" }, [0, ...k.comps]),
        slotButton(t("designer.shield"), { kind: "shield" }, [0, ...k.shields]),
        slotButton(t("designer.jammer"), { kind: "jammer" }, [0, ...k.jammers]),
        slotButton(t("designer.armor"), { kind: "armor" }, k.armor),
        slotButton(t("designer.engine"), { kind: "engine" }, k.engines),
        slotButton(t("designer.man"), { kind: "man" }, Array.from({ length: d.engine + 1 }, (_, i) => i)),
      ),
      ...weapons,
      h("div", { class: "form" }, ...specials),
      figures,
      save,
      full ? h("p", { class: "muted small" }, t("designer.full")) : null,
    );
    update();
  };

  const render = () => keepScroll(editor, () => (renderList(), renderEditor()));
  render();
  return modal(t("designer.title"), h("div", { class: "designer" }, list, editor), [t("btn.close")], "huge");
}
