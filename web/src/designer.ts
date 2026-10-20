// Ship design screen (1oom game_design.c rules live in the Rust core: this only edits a
// design and asks the core for its cost, space and combat figures).
import type { Ctx } from "./ctx";
import { shipParts } from "./engine";
import type { DesignStats, DesignView, ShipDesign } from "./types";
import { append, clear, h, keepScroll, modal } from "./ui";

const MAX_DESIGNS = 6;

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
    for (const [i, dv] of v.designs.entries()) {
      list.append(
        h(
          "div",
          { class: `design-item ${slot === i ? "selected" : ""}` },
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

  const select = (label: string, options: [number, string][], value: number, set: (n: number) => void) =>
    h(
      "label",
      { class: "field" },
      h("span", {}, label),
      h(
        "select",
        { onchange: (e: Event) => (set(Number((e.target as HTMLSelectElement).value)), keepScroll(editor, renderEditor)) },
        ...options.map(([i, text]) => h("option", { value: i, selected: i === value }, text)),
      ),
    );

  const renderEditor = () => {
    const v = ctx.view();
    const k = v.parts;
    let stats = ctx.game.designStats(d);
    for (let i = 0; i < 4; i++) {
      if (d.weapons[i] && d.weapon_counts[i] === 0 && stats.max_weapons[i] > 0) d.weapon_counts[i] = 1;
    }
    const none: [number, string] = [0, t("designer.none")];
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
    const name = h("input", { type: "text", value: d.name, maxlength: 24 });
    name.addEventListener("input", () => ((d.name = name.value), update()));
    counts.forEach((c, i) =>
      c.addEventListener("input", () => {
        d.weapon_counts[i] = Math.max(0, Math.min(99, Math.floor(Number(c.value) || 0)));
        update();
      }),
    );
    const weapons = [0, 1, 2, 3].map((i) =>
      h(
        "div",
        { class: "weapon-row" },
        select(t("designer.weapon", { n: i + 1 }), [none, ...k.weapons.map((w): [number, string] => [w, weaponLabel(w)])], d.weapons[i], (n) => {
          d.weapons[i] = n;
          d.weapon_counts[i] = n ? Math.max(1, d.weapon_counts[i]) : 0;
        }),
        h("label", { class: "field count" }, maxLabels[i], counts[i]),
      ),
    );
    const specials = [0, 1, 2].map((i) =>
      select(
        t("designer.special", { n: i + 1 }),
        [none, ...k.specials.map((s): [number, string] => [s, part("special", parts.specials[s].key)])],
        d.specials[i],
        (n) => (d.specials[i] = n),
      ),
    );
    append(
      clear(editor),
      h("h3", {}, slot === null ? t("designer.new_title") : t("designer.edit_title", { name: v.designs[slot].name })),
      h("label", { class: "field" }, h("span", {}, t("designer.name")), name),
      h(
        "div",
        { class: "form" },
        select(t("designer.hull"), parts.hulls.map((hl, i): [number, string] => [i, `${part("hull", hl.key)} (${t("designer.space", { n: hl.space })})`]), d.hull, (n) => (d.hull = n)),
        select(t("designer.comp"), [none, ...k.comps.map((c): [number, string] => [c, part("comp", parts.comps[c].key)])], d.comp, (n) => (d.comp = n)),
        select(t("designer.shield"), [none, ...k.shields.map((c): [number, string] => [c, part("shield", parts.shields[c].key)])], d.shield, (n) => (d.shield = n)),
        select(t("designer.jammer"), [none, ...k.jammers.map((c): [number, string] => [c, part("jammer", parts.jammers[c].key)])], d.jammer, (n) => (d.jammer = n)),
        select(t("designer.armor"), k.armor.map((c): [number, string] => [c, part("armor", parts.armor[c].key)]), d.armor, (n) => (d.armor = n)),
        select(t("designer.engine"), k.engines.map((c): [number, string] => [c, `${part("engine", parts.engines[c].key)} (${t("designer.warp", { n: parts.engines[c].warp })})`]), d.engine, (n) => {
          d.engine = n;
          d.man = Math.min(d.man, n);
        }),
        select(t("designer.man"), Array.from({ length: d.engine + 1 }, (_, i): [number, string] => [i, String(i + 1)]), d.man, (n) => (d.man = n)),
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
