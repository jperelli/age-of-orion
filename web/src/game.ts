// The in-game screen: top bar, map, side panel and dialogs.
import { battleScreen } from "./battle";
import type { Ctx } from "./ctx";
import { designDialog } from "./designer";
import { councilDialog, deliverMessages, diplomacyDialog, spyChances, techRefName } from "./diplomacy";
import type { Game } from "./engine";
import { groundScreen, isGroundScreen } from "./ground";
import { MapView } from "./map";
import { newscast, newsType } from "./news";
import { needsChoice, researchDialog } from "./research";
import * as store from "./storage";
import type { Theme } from "./theme";
import { BUILD_STARGATE, FIELDS, SLIDERS, type Command, type EmpireView, type FleetView, type GameEvent, type PlanetView, type View } from "./types";
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
    // A loaded game may have a battle or decisions waiting.
    setTimeout(() => {
      if (this.view.in_battle) void this.endTurn();
      else if (this.view.alive && this.view.winner === null && !this.busy) {
        this.busy = true;
        void this.decisions().finally(() => (this.busy = false));
      }
    }, 0);
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

  private get ctx(): Ctx {
    return { game: this.game, theme: this.theme, view: () => this.view, command: (c) => this.command(c), refresh: () => this.refresh() };
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
        h("button", { onclick: () => this.openResearch() }, this.t("game.btn.research")),
        h("button", { onclick: () => this.openDesigner() }, this.t("game.btn.design")),
        h("button", { onclick: () => this.empireDialog() }, this.t("game.btn.empire")),
        h("button", { class: this.view.messages.length ? "attention" : "", onclick: () => this.rivalsDialog() }, this.t("game.btn.rivals")),
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
        h("div", { class: "targeting" }, h("p", {}, this.map.targeting.prompt ?? this.t("game.choose_destination")), h("button", { onclick: () => ((this.map.targeting = null), this.renderPanel()) }, this.t("btn.cancel"))),
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
      if (p.rebellion) rows.push([this.t("planet.unrest"), this.t("planet.rebellion", { n: p.rebels ?? 0 })]);
      else if (mine && p.rebels) rows.push([this.t("planet.unrest"), this.t("planet.rebels", { n: p.rebels })]);
      if (mine && p.reserve) rows.push([this.t("planet.reserve"), `${p.reserve} ${this.t("unit.bc")}`]);
    }
    if (p.nebula) rows.push([this.t("planet.nebula"), this.t("planet.nebula_effect")]);
    if (p.stargate) rows.push([this.t("planet.stargate"), this.t("planet.stargate_yes")]);
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
    const turns = Math.ceil(Math.max(0, c.build_cost - c.bc_to_ship) / Math.max(1, c.spending[0]));
    const gate = v.abilities.stargates && !p.stargate;
    const select = h(
      "select",
      { onchange: (e: Event) => this.command({ type: "set_build", planet: p.id, design: Number((e.target as HTMLSelectElement).value) }) },
      ...v.designs.map((d, i) => h("option", { value: i, selected: i === c.build }, `${d.name} (${d.stats.cost} ${this.t("unit.bc")})`)),
      gate || c.build === BUILD_STARGATE
        ? h("option", { value: BUILD_STARGATE, selected: c.build === BUILD_STARGATE }, `${this.t("colony.stargate")} (${v.stargate_cost} ${this.t("unit.bc")})`)
        : null,
    );
    const rally = c.rally !== null ? v.planets[c.rally] : null;
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
      h(
        "p",
        { class: "muted small" },
        rally ? this.t("colony.rally_to", { planet: rally.name }) : this.t("colony.rally_none"),
        " ",
        h("button", { class: "mini", onclick: () => this.chooseRally(p) }, this.t("colony.rally_set")),
        rally ? h("button", { class: "mini", onclick: () => this.command({ type: "set_rally", planet: p.id, to: null }) }, this.t("colony.rally_clear")) : null,
      ),
      p.rebellion ? h("p", { class: "warn small" }, this.t("colony.rebellion_hint")) : null,
      c.transport
        ? h(
            "p",
            { class: "small" },
            this.t("colony.transport_pending", { pop: c.transport[0], planet: v.planets[c.transport[1]].name }),
            " ",
            h("button", { class: "mini", onclick: () => this.command({ type: "send_transport", from: p.id, to: p.id, pop: 0 }) }, this.t("colony.transport_cancel")),
          )
        : null,
      h(
        "div",
        { class: "row" },
        h("button", { onclick: () => this.transportDialog(p), disabled: (p.pop ?? 0) < 2 }, this.t("colony.send_transport")),
        h("button", { onclick: () => this.reserveDialog(p), disabled: v.reserve <= 0 }, this.t("colony.transfer_reserve")),
      ),
    );
  }

  private async reserveDialog(p: PlanetView): Promise<void> {
    const v = this.view;
    const input = h("input", { type: "number", min: 1, max: v.reserve, value: Math.min(v.reserve, Math.max(1, p.prod ?? 1)) });
    const ok = await modal(this.t("reserve.title"), [h("p", {}, this.t("reserve.text", { reserve: v.reserve, planet: p.name })), input], [this.t("reserve.send"), this.t("btn.cancel")]);
    if (ok !== 0) return;
    const bc = Math.max(1, Math.min(v.reserve, Math.floor(Number(input.value) || 0)));
    this.command({ type: "transfer_reserve", planet: p.id, bc });
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

  private chooseRally(p: PlanetView): void {
    const v = this.view;
    this.map.targeting = {
      from: p,
      range: 99,
      speed: 1,
      prompt: this.t("colony.rally_choose", { planet: p.name }),
      ranges: false,
      label: () => this.t("colony.rally_label"),
      ok: (id) => id !== p.id && v.planets[id].owner === v.player,
      pick: (to) => {
        this.map.targeting = null;
        this.command({ type: "set_rally", planet: p.id, to });
      },
    };
    this.renderPanel();
  }

  private designName(f: FleetView | null, i: number): string {
    const own = !f || f.owner === this.view.player;
    return (own ? this.view.designs[i]?.name : f?.names[i]) ?? "?";
  }

  private shipsLine(f: FleetView): string {
    return f.ships
      .map((n, i) => (n > 0 ? `${n} ${this.designName(f, i)}` : ""))
      .filter(Boolean)
      .join(", ");
  }

  private fleetLimits(ships: number[]): { range: number; speed: number } {
    const v = this.view;
    const used = ships.map((n, i) => (n > 0 ? v.designs[i] : null)).filter((d) => d !== null);
    return {
      range: Math.min(...used.map((d) => d!.range)),
      speed: Math.min(...used.map((d) => d!.stats.speed)),
    };
  }

  private orbitControls(p: PlanetView): void {
    const v = this.view;
    const here = v.fleets.filter((f) => f.at === p.id);
    if (!here.length) return;
    append(this.panel, h("h3", {}, this.t("orbit.title")));
    for (const f of here) {
      if (f.owner !== v.player) {
        append(this.panel, h("p", { class: "enemy" }, `${this.empireName(f.owner)}: ${this.shipsLine(f)}`));
        continue;
      }
      const inputs = f.ships.map((n) => h("input", { type: "number", min: 0, max: n, value: n }));
      const rows = f.ships
        .map((n, i) =>
          n > 0 ? h("div", { class: "ship-row" }, h("label", {}, this.designName(f, i)), inputs[i], h("span", { class: "muted" }, `/ ${n}`)) : null,
        )
        .filter(Boolean) as HTMLElement[];
      const chosen = () => inputs.map((inp, i) => Math.max(0, Math.min(f.ships[i], Math.floor(Number(inp.value) || 0))));
      const scrap = () => {
        const ships = chosen();
        if (ships.some((n) => n > 0) && window.confirm(this.t("orbit.scrap_confirm", { ships: this.shipsLine({ ...f, ships }) }))) {
          this.command({ type: "scrap_ships", planet: p.id, ships });
        }
      };
      const send = () => {
        const ships = chosen();
        if (!ships.some((n) => n > 0)) return;
        const { range, speed } = this.fleetLimits(ships);
        this.map.targeting = {
          from: p,
          range,
          speed,
          label: (id) => (p.stargate && v.planets[id].stargate && v.planets[id].owner === v.player ? this.t("map.via_stargate") : null),
          ok: (id) => id !== p.id && v.planets[id].range <= range,
          pick: (to) => {
            this.map.targeting = null;
            this.command({ type: "send_fleet", from: p.id, to, ships });
          },
        };
        this.renderPanel();
      };
      append(
        this.panel,
        h(
          "div",
          { class: "fleet-box" },
          ...rows,
          h("div", { class: "row" }, h("button", { class: "primary", onclick: send }, this.t("orbit.send")), h("button", { onclick: scrap }, this.t("orbit.scrap"))),
        ),
      );
    }
  }

  private fleetPanel(f: FleetView): void {
    const v = this.view;
    const dest = f.dest !== null ? v.planets[f.dest] : null;
    const { range, speed } = this.fleetLimits(f.ships);
    append(this.panel, 
      h("h2", {}, this.t("fleet.title")),
      h("p", {}, this.shipsLine(f)),
      dest ? h("p", {}, this.t("fleet.heading", { planet: dest.name, turns: f.eta ?? 0 })) : null,
      h(
        "button",
        {
          onclick: () => {
            this.map.targeting = {
              from: f,
              range,
              speed,
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
      speed: v.abilities.engine_warp,
      ok: (id) => id !== p.id && v.planets[id].range <= range && v.planets[id].explored && v.planets[id].kind !== "not_habitable",
      pick: (to) => {
        this.map.targeting = null;
        this.command({ type: "send_transport", from: p.id, to, pop });
      },
    };
    this.renderPanel();
  }

  // ------------------------------------------------------------- turn

  private busy = false;

  private async endTurn(): Promise<void> {
    if (this.busy || document.querySelector(".overlay")) return;
    this.busy = true;
    try {
      await this.processTurn();
    } finally {
      this.busy = false;
    }
  }

  private async processTurn(): Promise<void> {
    this.map.targeting = null;
    this.game.endTurn();
    this.refresh();
    // Battles with the player interrupt the turn: fight each, then let the turn go on.
    for (let guard = 0; this.view.in_battle && guard < 200; guard++) {
      if (this.view.battle && this.view.battle.winner === null) {
        await store.put("kv", AUTOSAVE, this.game.save());
        await battleScreen(this.ctx);
      }
      this.game.endTurn();
      this.refresh();
    }
    await store.put("kv", AUTOSAVE, this.game.save());
    const v = this.view;
    for (const e of v.events) if (isGroundScreen(e, v.player)) await groundScreen(this.ctx, e);
    const visible = (e: GameEvent) => e.to.length === 0 || e.to.includes(v.player);
    const news = v.events.filter((e) => newsType(e) !== null && visible(e));
    await newscast(
      this.ctx,
      news.map((e) => ({
        type: newsType(e)!,
        text: this.eventText(e),
        ranking: ((e.ranking as number[] | undefined) ?? []).map((i) => this.t(`race.${v.empires[i].race}.plural`)),
      })),
    );
    const events = v.events.filter((e) => (!["ships_built", "factories_complete"].includes(e.type) || e.to.includes(v.player)) && !news.includes(e));
    if (events.length) {
      const list = h("ul", { class: "events" }, ...events.map((e) => h("li", { class: `ev-${e.type}`, onclick: () => this.focusEvent(e) }, this.eventText(e))));
      await modal(this.t("report.title", { year: v.year }), list, [this.t("btn.ok")], "report");
    }
    if (await this.checkEnd()) return;
    await this.decisions();
  }

  /** Votes, messages, spy successes and research choices waiting for the player. */
  private async decisions(): Promise<void> {
    await councilDialog(this.ctx);
    if (await this.checkEnd()) return;
    await deliverMessages(this.ctx);
    await spyChances(this.ctx);
    if (needsChoice(this.ctx)) await this.openResearch();
  }

  /** Shows the end of the game; true when the player has to leave. */
  private async checkEnd(): Promise<boolean> {
    const v = this.view;
    if (v.winner === v.player) {
      if (this.shownVictory) return false;
      this.shownVictory = true;
      await modal(this.t("end.victory_title"), h("p", {}, this.t(v.victory === "council" ? "end.victory_council" : "end.victory")), [this.t("btn.ok")]);
    } else if (!v.alive) {
      await modal(this.t("end.defeat_title"), h("p", {}, this.t("end.defeat")), [this.t("btn.ok")]);
      this.onQuit();
      return true;
    } else if (v.winner !== null) {
      const key = v.victory === "council" ? "end.lost_council" : "end.lost_to";
      await modal(this.t("end.defeat_title"), h("p", {}, this.t(key, { empire: this.empireName(v.winner) })), [this.t("btn.ok")]);
      this.onQuit();
      return true;
    }
    return false;
  }

  private shownVictory = false;

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
      else if (["empire", "attacker", "defender", "from", "to", "by", "owner", "other", "spy", "target", "against", "elected"].includes(k) && typeof val === "number") params[k] = this.empireName(val);
      else if (k === "design" && typeof val === "number") params.design = v.designs[val]?.name ?? "?";
      else if (k === "field" && typeof val === "string") params.field = this.t(`field.${val}`);
      else if (typeof val === "number" || typeof val === "string") params[k] = val;
    }
    if (e.type === "battle") {
      const sides = e.sides as number[];
      const losses = e.losses as number[];
      const nm = (s: number) => (s === -1 ? this.t("guardian.name") : s === -2 ? this.t("monster.crystal") : s === -3 ? this.t("monster.amoeba") : this.empireName(s));
      params.sides = sides.map(nm).join(" / ");
      params.losses = sides.map((s, i) => `${nm(s)} -${losses[i]}`).join(", ");
      params.result = e.winner === null ? this.t("event.battle.draw") : this.t("event.battle.won", { winner: nm(e.winner as number) });
    }
    if (e.type === "tech_discovered" || e.type === "tech_stolen") {
      params.tech = techRefName(this.ctx, [FIELDS.indexOf(e.field as never), e.tech as number]);
      if (e.type === "tech_stolen") return this.t(e.by === null ? "event.tech_stolen_unknown" : "event.tech_stolen", params);
      return this.t(`event.tech_discovered.${e.source}`, params);
    }
    if (e.type === "news") {
      // Also fills the placeholders of imported MOO1 news texts (1oom game_news_get_msg).
      const n1 = e.n1 as number;
      if (e.news === "virus") params.field = this.t(`field.${FIELDS[n1]}`);
      if (e.news === "coup") {
        params.personality = this.t(`trait1.${n1}`).toLowerCase();
        params.objective = this.t(`trait2.${e.n2}`).toLowerCase();
        params.an = n1 === 2 || n1 === 3 ? "n " : " ";
      }
      if (typeof e.empire === "number") {
        params.race = this.t(`race.${v.empires[e.empire].race}`);
        params.leader = v.empires[e.empire].leader;
      }
      if (typeof e.other === "number") params.other_race = this.t(`race.${v.empires[e.other].race}`);
      params.s = n1 === 1 ? "" : "s";
      params.s1 = n1 === 1 ? "s" : "";
      return this.t(`news.${e.news}.${e.stage}`, params);
    }
    if (e.type === "sabotaged") return this.t(e.by === null ? `event.sabotaged_unknown.${e.kind}` : `event.sabotaged.${e.kind}`, params);
    if (e.type === "spies_caught") return this.t(e.spy === v.player ? "event.spies_caught.ours" : "event.spies_caught.theirs", params);
    if (e.type === "council") {
      const c = e.candidates as number[];
      return this.t("event.council", { a: this.empireName(c[0]), b: this.empireName(c[1]) });
    }
    if (e.type === "council_result") {
      const c = e.candidates as number[];
      const votes = e.votes as number[];
      return this.t(e.elected === null ? "event.council_result.none" : "event.council_result.elected", {
        ...params,
        a: this.empireName(c[0]),
        b: this.empireName(c[1]),
        va: votes[0],
        vb: votes[1],
      });
    }
    if (e.type === "ground_battle") return this.t(e.won ? "event.ground_battle.won" : "event.ground_battle.lost", params);
    return this.t(`event.${e.type}`, params);
  }

  // ------------------------------------------------------------- dialogs

  private async openResearch(): Promise<void> {
    await researchDialog(this.ctx);
    this.refresh();
  }

  private async openDesigner(): Promise<void> {
    await designDialog(this.ctx);
    this.refresh();
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
          h(
            "td",
            {},
            p.colony && p.colony.spending[0] > 0
              ? p.colony.build === BUILD_STARGATE
                ? this.t("colony.stargate")
                : (v.designs[p.colony.build]?.name ?? "-")
              : "-",
          ),
        ),
      ),
    );
    const bombard = h("input", { type: "checkbox", checked: v.bombard });
    bombard.addEventListener("change", () => this.command({ type: "set_bombard", enabled: bombard.checked }));
    const tax = h("input", { type: "range", min: 0, max: 200, step: 5, value: v.tax });
    const taxVal = h("span", { class: "bc" });
    const showTax = () => (taxVal.textContent = this.t("empire.tax_value", { pct: (Number(tax.value) / 10).toFixed(1), bc: Math.floor((v.total_prod * Number(tax.value)) / 2000) }));
    showTax();
    tax.addEventListener("input", showTax);
    tax.addEventListener("change", () => this.command({ type: "set_tax", tenths: Number(tax.value) }));
    void modal(
      this.t("empire.title", { n: mine.length }),
      [
        table,
        h("div", { class: "slider-row" }, h("label", { title: this.t("empire.tax_hint") }, this.t("empire.tax")), tax, taxVal),
        h("p", { class: "muted small" }, this.t("empire.income", { trade: v.total_trade, spies: v.spying_maint, reserve: v.reserve })),
        h("label", { class: "checkbox" }, bombard, " ", this.t("empire.bombard")),
      ],
      [this.t("btn.close")],
      "wide",
    );
  }

  private async rivalsDialog(): Promise<void> {
    await deliverMessages(this.ctx);
    await diplomacyDialog(this.ctx);
    this.refresh();
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
