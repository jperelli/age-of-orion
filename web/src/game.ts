// The in-game screen: top bar, map, side panel and dialogs.
import type { Game } from "./engine";
import { MapView } from "./map";
import * as store from "./storage";
import type { Theme } from "./theme";
import { FIELDS, RACES, SLIDERS, type Command, type EmpireView, type FleetView, type GameEvent, type PlanetView, type View } from "./types";
import { append, clear, download, h, modal, pickFiles, toast } from "./ui";

export const AUTOSAVE = "autosave";

export class GameScreen {
  readonly el: HTMLElement;
  private view: View;
  private map: MapView;
  private panel: HTMLElement;
  private bar: HTMLElement;
  private keyHandler = (e: KeyboardEvent) => this.onKey(e);

  constructor(
    private game: Game,
    private theme: Theme,
    private onQuit: () => void,
    private onLoad: (save: string) => void,
  ) {
    this.view = game.view();
    const canvas = h("canvas", { class: "map" });
    this.bar = h("header", { class: "topbar" });
    this.panel = h("aside", { class: "panel" });
    this.el = h("div", { class: `game theme-${theme.def.id}` }, this.bar, h("div", { class: "main" }, canvas, this.panel));
    this.map = new MapView(canvas, theme);
    this.map.onSelect = () => this.renderPanel();
    this.map.setView(this.view);
    document.addEventListener("keydown", this.keyHandler);
    const home = this.view.planets.find((p) => p.owner === this.view.player);
    if (home) this.map.selection = { kind: "planet", id: home.id };
    this.render();
  }

  destroy(): void {
    document.removeEventListener("keydown", this.keyHandler);
    this.game.free();
  }

  private t(key: string, params?: Record<string, string | number>): string {
    return this.theme.t(key, params);
  }

  private empireName(e: EmpireView | number): string {
    const em = typeof e === "number" ? this.view.empires[e] : e;
    return this.t(`race.${em.race}.plural`);
  }

  private refresh(): void {
    this.view = this.game.view();
    this.map.setView(this.view);
    this.render();
  }

  private render(): void {
    this.renderBar();
    this.renderPanel();
  }

  private command(cmd: Command): boolean {
    const err = this.game.command(cmd);
    if (err) {
      toast(this.t(`error.${err}`));
      return false;
    }
    this.refresh();
    return true;
  }

  private onKey(e: KeyboardEvent): void {
    if (document.querySelector(".overlay") || (e.target as HTMLElement)?.tagName === "INPUT") return;
    if (e.key === "Escape" && this.map.targeting) {
      this.map.targeting = null;
      this.renderPanel();
    } else if (e.key === "Enter") {
      void this.endTurn();
    }
  }

  // ------------------------------------------------------------- top bar

  private renderBar(): void {
    const v = this.view;
    const me = v.empires[v.player];
    clear(this.bar).append(
      h("div", { class: "crest", style: `--banner:${this.theme.banner(me.banner)}` }),
      h(
        "div",
        { class: "stats" },
        h("b", {}, this.t("game.year", { year: v.year })),
        h("span", { title: this.t("game.production_hint") }, this.t("game.production", { bc: v.total_prod - v.maintenance, maint: v.maintenance })),
        h("span", {}, this.t("game.research", { rp: v.research })),
        h("span", {}, this.t("game.reserve", { bc: v.reserve })),
      ),
      h(
        "nav",
        {},
        h("button", { onclick: () => this.researchDialog() }, this.t("game.btn.research")),
        h("button", { onclick: () => this.empireDialog() }, this.t("game.btn.empire")),
        h("button", { onclick: () => this.rivalsDialog() }, this.t("game.btn.rivals")),
        h("button", { onclick: () => this.menuDialog() }, this.t("game.btn.menu")),
        h("button", { class: "primary end-turn", onclick: () => this.endTurn() }, this.t("game.btn.end_turn")),
      ),
    );
  }

  // ------------------------------------------------------------- panel

  private renderPanel(): void {
    const s = this.map.selection;
    clear(this.panel);
    if (this.map.targeting) {
      this.panel.scrollTop = 0;
      append(this.panel, 
        h("div", { class: "targeting" }, h("p", {}, this.t("game.choose_destination")), h("button", { onclick: () => ((this.map.targeting = null), this.renderPanel()) }, this.t("btn.cancel"))),
      );
    }
    if (s?.kind === "planet") this.planetPanel(this.view.planets[s.id]);
    else if (s?.kind === "fleet") {
      const f = this.view.fleets.find((f) => f.id === s.id);
      if (f) this.fleetPanel(f);
      else this.overviewPanel();
    } else this.overviewPanel();
  }

  private overviewPanel(): void {
    const v = this.view;
    const me = v.empires[v.player];
    append(this.panel, 
      h("h2", {}, this.empireName(me)),
      h("p", { class: "muted" }, this.t("game.leader", { name: me.leader })),
      h("p", {}, this.t("game.hint_select")),
    );
  }

  private planetPicture(p: PlanetView): HTMLElement {
    const url = p.image !== null ? this.theme.url(`planet/${p.image}`) : null;
    if (url && this.theme.def.use_lbx) return h("img", { class: "planet-pic pixel", src: url, alt: "" });
    const site = p.kind ? this.theme.url(p.owner !== null ? "site/city" : `site/${p.kind}`) : null;
    if (site) return h("img", { class: "planet-pic", src: site, alt: "" });
    return h("div", { class: `planet-pic swatch kind-${p.kind ?? "unknown"} star-${p.star}` });
  }

  private planetPanel(p: PlanetView): void {
    const v = this.view;
    const mine = p.owner === v.player;
    const rows: [string, string | number][] = [];
    if (p.explored && p.kind) {
      rows.push([this.t("planet.terrain"), this.t(`planet.${p.kind}`)]);
      rows.push([this.t("planet.region"), this.t(`star.${p.star}`)]);
      if (p.special && p.special !== "normal") rows.push([this.t("planet.special"), this.t(`special.${p.special}`)]);
      if (p.growth && p.growth !== "normal") rows.push([this.t("planet.growth"), this.t(`growth.${p.growth}`)]);
      rows.push([this.t("planet.max_pop"), p.max_pop ?? 0]);
    } else {
      rows.push([this.t("planet.region"), this.t(`star.${p.star}`)]);
    }
    if (p.owner !== null) {
      rows.push([this.t("planet.owner"), this.empireName(p.owner)]);
      rows.push([this.t("planet.pop"), `${p.pop}${mine ? ` / ${p.max_pop}` : ""}`]);
      rows.push([this.t("planet.factories"), `${p.factories}${mine && p.colony ? ` / ${p.colony.max_factories}` : ""}`]);
      rows.push([this.t("planet.bases"), p.bases ?? 0]);
      if (mine && p.shield) rows.push([this.t("planet.shield"), p.shield]);
    }
    if (!mine) rows.push([this.t("planet.distance"), p.range > 900 ? "-" : this.t("planet.parsecs", { n: p.range })]);
    append(this.panel, 
      h("div", { class: "planet-head" }, this.planetPicture(p), h("div", {}, h("h2", {}, p.name), p.orion ? h("div", { class: "muted" }, this.t("planet.orion")) : null)),
      h("table", { class: "facts" }, ...rows.map(([k, val]) => h("tr", {}, h("th", {}, k), h("td", {}, val)))),
    );
    if (!p.explored) append(this.panel, h("p", { class: "muted" }, this.t("planet.unexplored")));
    if (p.guardian) append(this.panel, h("p", { class: "warn" }, this.t("planet.guardian")));
    if (p.explored && p.owner === null && p.kind !== "not_habitable" && !p.guardian) {
      append(this.panel, h("p", { class: p.colonizable ? "good" : "muted" }, this.t(p.colonizable ? "planet.can_colonize" : "planet.needs_tech")));
    }
    if (mine && p.colony) this.colonyControls(p);
    this.orbitControls(p);
  }

  private colonyControls(p: PlanetView): void {
    const c = p.colony!;
    const v = this.view;
    const sliders = [...c.sliders];
    const rows = SLIDERS.map((name, i) => {
      const val = h("span", { class: "bc" }, `${c.spending[i]} ${this.t("unit.bc")}`);
      const input = h("input", { type: "range", min: 0, max: 100, value: sliders[i] });
      input.addEventListener("input", () => {
        const nv = Number(input.value);
        const others = sliders.map((x, j) => (j === i ? 0 : x));
        const total = others.reduce((a, b) => a + b, 0);
        const rest = 100 - nv;
        for (let j = 0; j < sliders.length; j++) {
          if (j !== i) sliders[j] = total > 0 ? Math.round((others[j] * rest) / total) : Math.round(rest / (sliders.length - 1));
        }
        sliders[i] = nv;
        rows.forEach((r, j) => {
          (r.querySelector("input") as HTMLInputElement).value = String(sliders[j]);
          r.querySelector(".bc")!.textContent = `${Math.floor(((p.prod ?? 0) * sliders[j]) / 100)} ${this.t("unit.bc")}`;
        });
      });
      input.addEventListener("change", () => this.command({ type: "set_sliders", planet: p.id, sliders }));
      return h("div", { class: "slider-row" }, h("label", {}, this.t(`slider.${name}`)), input, val);
    });
    const d = v.designs[c.build];
    const turns = d ? Math.ceil(Math.max(0, d.cost - c.bc_to_ship) / Math.max(1, c.spending[0])) : 0;
    const select = h(
      "select",
      { onchange: (e: Event) => this.command({ type: "set_build", planet: p.id, design: Number((e.target as HTMLSelectElement).value) }) },
      ...v.designs.map((d, i) => h("option", { value: i, selected: i === c.build }, `${this.t(`design.${d.key}`)} (${d.cost} ${this.t("unit.bc")})`)),
    );
    append(this.panel, 
      h("h3", {}, this.t("colony.spending", { bc: p.prod ?? 0 })),
      ...rows,
      h(
        "p",
        { class: c.sliders[3] < c.clean_eco ? "warn small" : "muted small" },
        this.t("colony.clean", { n: c.clean_eco }),
        " ",
        c.sliders[3] !== c.clean_eco ? h("button", { class: "mini", onclick: () => this.setClean(p) }, this.t("colony.set_clean")) : null,
      ),
      c.waste > 0 ? h("p", { class: "warn" }, this.t("colony.waste", { n: c.waste })) : null,
      h("p", { class: "muted" }, this.t("colony.research", { rp: c.research })),
      h("h3", {}, this.t("colony.build")),
      select,
      h("p", { class: "muted" }, c.spending[0] > 0 ? this.t("colony.build_eta", { turns }) : this.t("colony.build_none")),
      h("button", { onclick: () => this.transportDialog(p), disabled: (p.pop ?? 0) < 2 }, this.t("colony.send_transport")),
    );
  }

  /** Sets ecology to the clean level and rescales the other sliders. */
  private setClean(p: PlanetView): void {
    const c = p.colony!;
    const eco = c.clean_eco;
    const others = c.sliders.map((x, i) => (i === 3 ? 0 : x));
    const total = others.reduce((a, b) => a + b, 0) || 1;
    const sliders = others.map((x, i) => (i === 3 ? eco : Math.round((x * (100 - eco)) / total)));
    this.command({ type: "set_sliders", planet: p.id, sliders });
  }

  private shipsLine(ships: number[]): string {
    return ships
      .map((n, i) => (n > 0 ? `${n} ${this.t(`design.${this.view.designs[i].key}`)}` : ""))
      .filter(Boolean)
      .join(", ");
  }

  private orbitControls(p: PlanetView): void {
    const v = this.view;
    const here = v.fleets.filter((f) => f.at === p.id);
    if (!here.length) return;
    append(this.panel, h("h3", {}, this.t("orbit.title")));
    for (const f of here) {
      if (f.owner !== v.player) {
        append(this.panel, h("p", { class: "enemy" }, `${this.empireName(f.owner)}: ${this.shipsLine(f.ships)}`));
        continue;
      }
      const inputs = f.ships.map((n) => h("input", { type: "number", min: 0, max: n, value: n }));
      const rows = f.ships
        .map((n, i) =>
          n > 0 ? h("div", { class: "ship-row" }, h("label", {}, this.t(`design.${v.designs[i].key}`)), inputs[i], h("span", { class: "muted" }, `/ ${n}`)) : null,
        )
        .filter(Boolean) as HTMLElement[];
      const send = () => {
        const ships = inputs.map((inp, i) => Math.max(0, Math.min(f.ships[i], Math.floor(Number(inp.value) || 0))));
        if (!ships.some((n) => n > 0)) return;
        const range = Math.min(...ships.map((n, i) => (n > 0 ? v.designs[i].range : Infinity)));
        this.map.targeting = {
          from: p,
          range,
          speed: v.abilities.speed,
          ok: (id) => id !== p.id && v.planets[id].range <= range,
          pick: (to) => {
            this.map.targeting = null;
            this.command({ type: "send_fleet", from: p.id, to, ships });
          },
        };
        this.renderPanel();
      };
      append(this.panel, h("div", { class: "fleet-box" }, ...rows, h("button", { class: "primary", onclick: send }, this.t("orbit.send"))));
    }
  }

  private fleetPanel(f: FleetView): void {
    const v = this.view;
    const dest = f.dest !== null ? v.planets[f.dest] : null;
    const range = Math.min(...f.ships.map((n, i) => (n > 0 ? v.designs[i].range : Infinity)));
    append(this.panel, 
      h("h2", {}, this.t("fleet.title")),
      h("p", {}, this.shipsLine(f.ships)),
      dest ? h("p", {}, this.t("fleet.heading", { planet: dest.name, turns: f.eta ?? 0 })) : null,
      h(
        "button",
        {
          onclick: () => {
            this.map.targeting = {
              from: f,
              range,
              speed: v.abilities.speed,
              ok: (id) => v.planets[id].range <= range,
              pick: (to) => {
                this.map.targeting = null;
                this.command({ type: "redirect_fleet", fleet: f.id!, to });
              },
            };
            this.renderPanel();
          },
        },
        this.t("fleet.redirect"),
      ),
    );
  }

  private async transportDialog(p: PlanetView): Promise<void> {
    const v = this.view;
    const max = Math.floor((p.pop ?? 0) / 2);
    const input = h("input", { type: "number", min: 1, max, value: max });
    const ok = await modal(this.t("transport.title"), [h("p", {}, this.t("transport.text", { max })), input], [this.t("transport.choose"), this.t("btn.cancel")]);
    if (ok !== 0) return;
    const pop = Math.max(1, Math.min(max, Math.floor(Number(input.value) || 0)));
    const range = v.abilities.fuel_range;
    this.map.targeting = {
      from: p,
      range,
      speed: v.abilities.speed,
      ok: (id) => id !== p.id && v.planets[id].range <= range && v.planets[id].explored && v.planets[id].kind !== "not_habitable",
      pick: (to) => {
        this.map.targeting = null;
        this.command({ type: "send_transport", from: p.id, to, pop });
      },
    };
    this.renderPanel();
  }

  // ------------------------------------------------------------- turn

  private async endTurn(): Promise<void> {
    if (document.querySelector(".overlay")) return;
    this.map.targeting = null;
    this.game.endTurn();
    await store.put("kv", AUTOSAVE, this.game.save());
    this.refresh();
    const v = this.view;
    const events = v.events.filter((e) => !["ships_built", "factories_complete"].includes(e.type) || e.to.includes(v.player));
    if (v.winner === v.player) {
      await modal(this.t("end.victory_title"), h("p", {}, this.t("end.victory")), [this.t("btn.ok")]);
    } else if (!v.alive) {
      await modal(this.t("end.defeat_title"), h("p", {}, this.t("end.defeat")), [this.t("btn.ok")]);
      return this.onQuit();
    } else if (v.winner !== null) {
      await modal(this.t("end.defeat_title"), h("p", {}, this.t("end.lost_to", { empire: this.empireName(v.winner) })), [this.t("btn.ok")]);
      return this.onQuit();
    }
    if (events.length) {
      const list = h("ul", { class: "events" }, ...events.map((e) => h("li", { class: `ev-${e.type}`, onclick: () => this.focusEvent(e) }, this.eventText(e))));
      await modal(this.t("report.title", { year: v.year }), list, [this.t("btn.ok")], "report");
    }
  }

  private focusEvent(e: GameEvent): void {
    if (typeof e.planet === "number") {
      const p = this.view.planets[e.planet];
      this.map.selection = { kind: "planet", id: p.id };
      this.map.centerOn(p.x, p.y);
      this.renderPanel();
    }
  }

  eventText(e: GameEvent): string {
    const v = this.view;
    const params: Record<string, string | number> = {};
    for (const [k, val] of Object.entries(e)) {
      if (k === "planet" && typeof val === "number") params.planet = v.planets[val].name;
      else if (["empire", "attacker", "defender", "from", "to", "by", "owner"].includes(k) && typeof val === "number") params[k] = this.empireName(val);
      else if (k === "design" && typeof val === "number") params.design = this.t(`design.${v.designs[val].key}`);
      else if (k === "field" && typeof val === "string") params.field = this.t(`field.${val}`);
      else if (typeof val === "number" || typeof val === "string") params[k] = val;
    }
    if (e.type === "battle") {
      const sides = e.sides as number[];
      const losses = e.losses as number[];
      const nm = (s: number) => (s < 0 ? this.t("guardian.name") : this.empireName(s));
      params.sides = sides.map(nm).join(" / ");
      params.losses = sides.map((s, i) => `${nm(s)} -${losses[i]}`).join(", ");
      params.result = e.winner === null ? this.t("event.battle.draw") : this.t("event.battle.won", { winner: nm(e.winner as number) });
    }
    if (e.type === "ground_battle") return this.t(e.won ? "event.ground_battle.won" : "event.ground_battle.lost", params);
    return this.t(`event.${e.type}`, params);
  }

  // ------------------------------------------------------------- dialogs

  private researchDialog(): void {
    const v = this.view;
    const alloc = [...v.tech.alloc];
    const rows = FIELDS.map((f, i) => {
      const pct = Math.min(100, Math.floor((v.tech.invested[i] * 100) / Math.max(1, v.tech.next_cost[i])));
      const input = h("input", { type: "range", min: 0, max: 100, value: alloc[i] });
      const val = h("span", { class: "bc" }, `${alloc[i]}%`);
      input.addEventListener("input", () => {
        alloc[i] = Number(input.value);
        val.textContent = `${alloc[i]}%`;
      });
      input.addEventListener("change", () => this.command({ type: "set_research", alloc }));
      return h(
        "div",
        { class: "research-row" },
        h("label", {}, this.t(`field.${f}`), h("small", {}, this.t("research.level", { n: v.tech.level[i] }))),
        h("div", { class: "progress", title: `${v.tech.invested[i]} / ${v.tech.next_cost[i]}` }, h("div", { style: `width:${pct}%` })),
        input,
        val,
      );
    });
    const a = v.abilities;
    const facts = h(
      "ul",
      { class: "muted abilities" },
      h("li", {}, this.t("research.range", { n: a.fuel_range })),
      h("li", {}, this.t("research.speed", { n: a.speed })),
      h("li", {}, this.t("research.workers", { n: a.colonist_oper_factories })),
      h("li", {}, this.t("research.colonize", { kind: this.t(`planet.${a.colony_for}`) })),
    );
    void modal(this.t("research.title", { rp: v.research }), [...rows, facts], [this.t("btn.close")], "wide");
  }

  private empireDialog(): void {
    const v = this.view;
    const mine = v.planets.filter((p) => p.owner === v.player);
    const table = h(
      "table",
      { class: "list" },
      h("tr", {}, ...["planet.name", "planet.pop", "planet.factories", "planet.bases", "colony.prod", "colony.build"].map((k) => h("th", {}, this.t(k)))),
      ...mine.map((p) =>
        h(
          "tr",
          {
            class: "clickable",
            onclick: () => {
              this.map.selection = { kind: "planet", id: p.id };
              this.map.centerOn(p.x, p.y);
              this.renderPanel();
              document.querySelector<HTMLButtonElement>(".overlay .dialog-buttons button")?.click();
            },
          },
          h("td", {}, p.name),
          h("td", {}, `${p.pop}/${p.max_pop}`),
          h("td", {}, p.factories ?? 0),
          h("td", {}, p.bases ?? 0),
          h("td", {}, p.prod ?? 0),
          h("td", {}, p.colony && p.colony.spending[0] > 0 ? this.t(`design.${v.designs[p.colony.build].key}`) : "-"),
        ),
      ),
    );
    void modal(this.t("empire.title", { n: mine.length }), table, [this.t("btn.close")], "wide");
  }

  private rivalsDialog(): void {
    const v = this.view;
    const cards = v.empires.map((e) => {
      const url = this.theme.url(`race/${RACES.indexOf(e.race)}`);
      return h(
        "div",
        { class: `rival ${e.alive ? "" : "dead"}` },
        url ? h("img", { class: "portrait pixel", src: url, alt: "" }) : h("div", { class: "crest big", style: `--banner:${this.theme.banner(e.banner)}` }),
        h(
          "div",
          {},
          h("b", { style: `color:${this.theme.banner(e.banner)}` }, this.empireName(e), e.id === v.player ? ` ${this.t("rivals.you")}` : ""),
          h("div", {}, this.t("rivals.leader", { name: e.leader })),
          h("div", { class: "muted" }, this.t(`race.${e.race}.trait`)),
          h("div", {}, e.alive ? this.t("rivals.colonies", { n: e.known_colonies }) : this.t("rivals.eliminated")),
        ),
      );
    });
    void modal(this.t("rivals.title"), cards, [this.t("btn.close")], "wide");
  }

  private async menuDialog(): Promise<void> {
    const r = await modal(
      this.t("game.btn.menu"),
      h("p", {}, this.t("menu.ingame_text")),
      [this.t("menu.save_file"), this.t("menu.load_file"), this.t("menu.help"), this.t("menu.quit"), this.t("btn.close")],
    );
    if (r === 0) download(`age-of-orion-${this.view.year}.json`, this.game.save());
    if (r === 1) {
      const [f] = await pickFiles({ accept: ".json,application/json" });
      if (f) this.onLoad(await f.text());
    }
    if (r === 2) await modal(this.t("menu.help"), this.theme.list("help.lines").map((l) => h("p", {}, l)), [this.t("btn.close")], "wide");
    if (r === 3) this.onQuit();
  }
}
