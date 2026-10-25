// The in-game screen, laid out like MOO1's: the map, the side panel and the button bar at the
// bottom (GAME, DESIGN, FLEET, MAP, RACES, PLANETS, TECH, NEXT TURN).
import { battleScreen } from "./battle";
import type { Ctx } from "./ctx";
import { designDialog } from "./designer";
import { councilDialog, deliverMessages, diplomacyDialog, techRefName } from "./diplomacy";
import type { Game } from "./engine";
import { groundScreen, isGroundScreen } from "./ground";
import { CITY_MARKS, MapView, cityLevel } from "./map";
import { growthLines, kindLines, raceLines, specialLines, starLines } from "./explain";
import { designPicture, iconRow, picImg, picUrl, uiIcon } from "./pictures";
import { explained, pictureSelect } from "./widgets";
import { newscast, newsType } from "./news";
import { bombAskScreen, bombShowScreen, caughtScreen, exploreScreen, landingScreen, newShipsScreen, sabotageDoneScreen, spyChances, stolenScreen } from "./screens";
import { adjustGroup, slider, sliders } from "./sliders";
import { chooseTechs, needsChoice, newTechScreen, techScreen } from "./research";
import * as store from "./storage";
import type { Theme } from "./theme";
import { BUILD_STARGATE, FIELDS, PLANET_TYPES, RACES, SLIDERS, type Command, type SabotageKind, type EmpireView, type FleetView, type GameEvent, type PlanetView, type TransportView, type View } from "./types";
import { append, clear, download, h, modal, pickFiles, toast } from "./ui";

export const AUTOSAVE = "autosave";

export class GameScreen {
  readonly el: HTMLElement;
  private view: View;
  private map: MapView;
  private panel: HTMLElement;
  private head: HTMLElement;
  private bar: HTMLElement;
  /** Ships chosen in the fleet deployment panel, by design (MOO1's FLEET DEPLOYMENT). */
  private deploy: { key: string; ships: number[] } | null = null;
  private keyHandler = (e: KeyboardEvent) => this.onKey(e);

  constructor(
    private game: Game,
    private theme: Theme,
    private onQuit: () => void,
    private onLoad: (save: string) => void,
  ) {
    this.view = this.fetchView();
    const canvas = h("canvas", { class: "map" });
    this.bar = h("footer", { class: "bottombar" });
    this.head = h("div", { class: "panel-head" });
    this.panel = h("div", { class: "panel-body" });
    this.el = h("div", { class: `game theme-${theme.def.id}` }, h("div", { class: "main" }, canvas, h("aside", { class: "panel" }, this.head, this.panel)), this.bar);
    this.map = new MapView(canvas, theme);
    this.map.onSelect = () => this.renderPanel();
    this.map.eta = (from, dest, speed, transport) => this.game.eta(from.x, from.y, dest, speed, transport);
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

  /** The core's view, with a name for the places not explored yet (the core gives none). */
  private fetchView(): View {
    const v = this.game.view();
    for (const p of v.planets) if (!p.explored) p.name = this.t("planet.unknown");
    return v;
  }

  private refresh(): void {
    this.view = this.fetchView();
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
      if (this.map.selection?.kind === "orbit") this.map.selection = { kind: "planet", id: this.map.selection.id };
      this.renderPanel();
    } else if (e.key === "Enter") {
      void this.endTurn();
    }
  }

  // ------------------------------------------------------------- bottom bar

  private renderBar(): void {
    const v = this.view;
    const me = v.empires[v.player];
    const stat = (icon: string, n: number, title: string) => h("span", { class: "stat", title }, h("img", { src: uiIcon(this.theme, icon), alt: "" }), String(n));
    clear(this.head).append(
      h("div", { class: "crest", style: `--banner:${this.theme.banner(me.banner)}` }),
      h("b", { class: "year" }, String(v.year)),
      h(
        "button",
        { class: "stats", title: this.t("game.stats_hint"), onclick: () => void this.treasuryDialog() },
        stat("gold", v.total_prod - v.maintenance, this.t("game.production", { bc: v.total_prod - v.maintenance, maint: v.maintenance })),
        stat("research", v.research, this.t("game.research", { rp: v.research })),
        stat("reserve", v.reserve, this.t("game.reserve", { bc: v.reserve })),
      ),
    );
    const button = (key: string, onclick: () => void, cls = "") => h("button", { class: cls, onclick }, this.t(key));
    clear(this.bar).append(
      button("game.btn.menu", () => void this.menuDialog()),
      button("game.btn.design", () => void this.openDesigner()),
      button("game.btn.fleets", () => void this.fleetsDialog()),
      button("game.btn.map", () => this.map.showAll()),
      button("game.btn.rivals", () => void this.rivalsDialog(), v.messages.length ? "attention" : ""),
      button("game.btn.empire", () => this.empireDialog()),
      button("game.btn.research", () => void this.openResearch()),
      button("game.btn.end_turn", () => void this.endTurn(), "primary end-turn"),
    );
  }

  // ------------------------------------------------------------- panel

  private renderPanel(): void {
    const s = this.map.selection;
    clear(this.panel);
    if (this.map.targeting && !this.map.targeting.inline) {
      this.panel.scrollTop = 0;
      append(this.panel, 
        h("div", { class: "targeting" }, h("p", {}, this.map.targeting.prompt ?? this.t("game.choose_destination")), h("button", { onclick: () => ((this.map.targeting = null), this.renderPanel()) }, this.t("btn.cancel"))),
      );
    }
    if (s?.kind !== "orbit" && s?.kind !== "fleet") this.deploy = null;
    // Fleet orders belong to the fleet shown; anything else selected drops them.
    const own = s?.kind === "orbit" ? `orbit/${s.id}` : s?.kind === "fleet" ? `fleet/${s.id}` : null;
    if (this.map.targeting?.key && this.map.targeting.key !== own) this.map.targeting = null;
    if (s?.kind === "planet") this.planetPanel(this.view.planets[s.id]);
    else if (s?.kind === "orbit") {
      const f = this.view.fleets.find((f) => f.at === s.id && f.owner === s.owner);
      if (f) this.orbitPanel(f);
      else {
        this.map.targeting = null;
        this.map.selection = { kind: "planet", id: s.id };
        this.planetPanel(this.view.planets[s.id]);
      }
    } else if (s?.kind === "fleet") {
      const f = this.view.fleets.find((f) => f.id === s.id);
      if (f) this.fleetPanel(f);
      else {
        this.map.targeting = null;
        this.overviewPanel();
      }
    } else if (s?.kind === "transport") {
      const tr = this.view.transports.find((t) => t.id === s.id);
      if (tr) this.transportPanel(tr);
      else this.overviewPanel();
    } else if (s?.kind === "nebula") this.nebulaPanel();
    else this.overviewPanel();
  }

  /** A nebula: what it does to the ships and planets inside (1oom `game_xy_is_in_nebula`). */
  private nebulaPanel(): void {
    const inside = this.view.planets.filter((p) => p.nebula && p.explored);
    append(
      this.panel,
      h("h2", {}, this.t("nebula.title")),
      h("ul", { class: "effects" }, ...["ships", "transports", "shields", "battle"].map((k) => h("li", {}, this.t(`nebula.${k}`)))),
      inside.length ? h("p", { class: "muted small" }, this.t("nebula.planets", { planets: inside.map((p) => p.name).join(", ") })) : null,
    );
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

  /** The painted view of a place, shown wide above its panel on a world map. */
  private placeView(p: PlanetView): HTMLElement | null {
    if (!p.explored || this.theme.def.map !== "world") return null;
    const alias = p.owner !== null ? `view/city/${cityLevel(p.pop ?? 0)}` : p.orion ? "view/orion" : `view/${p.kind}`;
    return picImg(this.theme, alias, "planet-view");
  }

  /** An empire's coat of arms. */
  private arms(owner: number): HTMLElement {
    return picImg(this.theme, `arms/${RACES.indexOf(this.view.empires[owner].race)}`, "arms");
  }

  /** The place's picture: the planet in space, the site or the city on a world map. */
  private planetPicture(p: PlanetView): HTMLElement {
    if (!p.explored || !p.kind) return h("div", { class: "planet-pic swatch kind-unknown" });
    if (this.theme.def.map === "world") return picImg(this.theme, p.owner !== null ? CITY_MARKS[cityLevel(p.pop ?? 0)] : `site/${p.kind}`, "planet-pic");
    return picImg(this.theme, `planet/${p.image ?? 0}`, "planet-pic");
  }

  /** Rows of facts about a place; land, climate, resources, soil and owner explain themselves when clicked. */
  private placeFacts(p: PlanetView, long: boolean): [string, string | number | HTMLElement][] {
    const v = this.view;
    const mine = p.owner === v.player;
    const rows: [string, string | number | HTMLElement][] = [];
    const ex = (key: string, value: string, lines: string[]) => explained(value, `${this.t(key)}: ${value}`, lines);
    if (p.kind) {
      rows.push([this.t("planet.terrain"), ex("planet.terrain", this.t(`planet.${p.kind}`), kindLines(this.theme, p.kind))]);
      rows.push([this.t("planet.region"), ex("planet.region", this.t(`star.${p.star}`), starLines(this.theme, p.star))]);
      if (p.special && (long || p.special !== "normal")) rows.push([this.t("planet.special"), ex("planet.special", this.t(`special.${p.special}`), specialLines(this.theme, p.special))]);
      const race = v.empires[p.owner ?? v.player].race;
      if (p.growth && (long || p.growth !== "normal")) rows.push([this.t("planet.growth"), ex("planet.growth", this.t(`growth.${p.growth}`), growthLines(this.theme, p.growth, race))]);
      if (!long) rows.push([this.t("planet.max_pop"), p.max_pop ?? 0]);
    }
    if (p.owner !== null) {
      const e = v.empires[p.owner];
      const name = this.empireName(p.owner);
      rows.push([
        this.t("planet.owner"),
        h("span", { class: "owner" }, this.arms(p.owner), explained(name, name, this.raceInfo(e.race))),
      ]);
      if (!long) {
        rows.push([this.t("planet.pop"), `${p.pop}${mine ? ` / ${p.max_pop}` : ""}`]);
        rows.push([this.t("planet.factories"), `${p.factories}${mine && p.colony ? ` / ${p.colony.max_factories}` : ""}`]);
        rows.push([this.t("planet.bases"), p.bases ?? 0]);
        if (mine && p.shield) rows.push([this.t("planet.shield"), p.shield]);
      }
      if (p.rebellion) rows.push([this.t("planet.unrest"), this.t("planet.rebellion", { n: p.rebels ?? 0 })]);
      else if (mine && p.rebels) rows.push([this.t("planet.unrest"), this.t("planet.rebels", { n: p.rebels })]);
      if (mine && p.reserve) rows.push([this.t("planet.reserve"), `${p.reserve} ${this.t("unit.bc")}`]);
    }
    if (p.nebula) rows.push([this.t("planet.nebula"), this.t("planet.nebula_effect")]);
    if (p.stargate) rows.push([this.t("planet.stargate"), this.t("planet.stargate_yes")]);
    if (!mine) rows.push([this.t("planet.distance"), p.range > 900 ? "-" : this.t("planet.parsecs", { n: p.range })]);
    return rows;
  }

  /** A race's summary, then what it does in the rules (without repeating the summary). */
  private raceInfo(race: EmpireView["race"]): string[] {
    const trait = this.t(`race.${race}.trait`);
    const lines = raceLines(this.theme, race, this.theme.mod?.rules);
    return lines.includes(trait) ? lines : [trait, ...lines];
  }

  private facts(rows: [string, string | number | HTMLElement][]): HTMLElement {
    return h("table", { class: "facts" }, ...rows.map(([k, val]) => h("tr", {}, h("th", {}, k), h("td", {}, val))));
  }

  private planetPanel(p: PlanetView): void {
    const v = this.view;
    const mine = p.owner === v.player;
    // Unexplored: only that something is there, and how far (MOO1 shows "UNEXPLORED").
    if (!p.explored) {
      append(
        this.panel,
        h("div", { class: "planet-head" }, h("div", { class: "planet-pic swatch kind-unknown" }), h("div", {}, h("h2", {}, p.name), h("div", { class: "muted" }, this.t("planet.something_there")))),
        this.facts([[this.t("planet.distance"), p.range > 900 ? "-" : this.t("planet.parsecs", { n: p.range })]]),
        h("p", { class: "muted" }, this.t("planet.unexplored")),
      );
      if (v.fleets.some((f) => f.at === p.id)) append(this.panel, h("p", { class: "muted small" }, this.t("orbit.select_hint")));
      return;
    }
    const view = this.placeView(p);
    const pic = view ?? this.planetPicture(p);
    pic.classList.add("opens");
    pic.title = this.t("planet.open_window");
    pic.addEventListener("click", () => void this.planetWindow(p));
    append(
      this.panel,
      view,
      h("div", { class: "planet-head" }, view ? null : pic, h("div", {}, h("h2", {}, p.name), p.orion ? h("div", { class: "muted" }, this.t("planet.orion")) : null)),
      this.facts(this.placeFacts(p, false)),
    );
    if (p.guardian) append(this.panel, h("p", { class: "warn" }, this.t("planet.guardian")));
    if (p.owner === null && p.kind !== "not_habitable" && !p.guardian) {
      append(this.panel, h("p", { class: p.colonizable ? "good" : "muted" }, this.t(p.colonizable ? "planet.can_colonize" : "planet.needs_tech")));
    }
    if (mine && p.colony) this.colonyControls(p);
    if (v.fleets.some((f) => f.at === p.id)) append(this.panel, h("p", { class: "muted small" }, this.t("orbit.select_hint")));
  }

  private colonyControls(p: PlanetView): void {
    const c = p.colony!;
    const v = this.view;
    const rows = sliders({
      specs: SLIDERS.map((name, i) => ({ label: this.t(`slider.short.${name}`), title: this.t(`slider.${name}`), value: c.sliders[i], locked: c.locks[i] })),
      total: 100,
      lockTitle: this.t("slider.lock_hint"),
      format: (i, s) => `${Math.floor(((p.prod ?? 0) * s[i]) / 100)} ${this.t("unit.bc")}`,
      onCommit: (s) => this.command({ type: "set_sliders", planet: p.id, sliders: s }),
      onLock: (i, locked) => this.command({ type: "lock_slider", planet: p.id, slider: i, locked }),
    });
    const turns = Math.ceil(Math.max(0, c.build_cost - c.bc_to_ship) / Math.max(1, c.spending[0]));
    const gate = v.abilities.stargates && !p.stargate;
    const banner = v.empires[v.player].banner;
    const select = pictureSelect(
      [
        ...v.designs.map((d, i) => ({ value: i, label: d.name, sub: `${d.stats.cost} ${this.t("unit.bc")}`, img: designPicture(this.theme, d.look, banner) })),
        ...(gate || c.build === BUILD_STARGATE
          ? [{ value: BUILD_STARGATE, label: this.t("colony.stargate"), sub: `${v.stargate_cost} ${this.t("unit.bc")}`, img: picUrl(this.theme, "stargate") }]
          : []),
      ],
      c.build,
      (design) => this.command({ type: "set_build", planet: p.id, design }),
      "build-select",
    ).el;
    const rally = c.rally !== null ? v.planets[c.rally] : null;
    append(this.panel, 
      h("h3", {}, this.t("colony.spending", { bc: p.prod ?? 0 })),
      rows,
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
    const locks = c.locks.map((l, i) => l && i !== 3);
    this.command({ type: "set_sliders", planet: p.id, sliders: adjustGroup(c.sliders, 3, c.clean_eco, locks) });
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

  /** A fleet's ships, one row per design with its picture and count. */
  private shipRows(f: FleetView, chosen?: number[], onChange?: (i: number, n: number) => void): HTMLElement {
    const v = this.view;
    const banner = v.empires[f.owner].banner;
    const rows = f.ships.map((n, i) => {
      if (n <= 0) return null;
      const [look] = f.looks[i] ?? [0, 0];
      const pic = h("img", { class: "ship-pic", src: designPicture(this.theme, look, banner), alt: "" });
      const count = chosen ? chosen[i] : n;
      const counter = onChange
        ? h(
            "span",
            { class: "ship-count" },
            h("button", { class: "mini step", disabled: count <= 0, onclick: (e: Event) => onChange(i, (e as MouseEvent).shiftKey ? 0 : count - 1) }, "−"),
            h("b", { title: this.t("orbit.all_none"), onclick: () => onChange(i, count === n ? 0 : n) }, `${count}`),
            h("button", { class: "mini step", disabled: count >= n, onclick: (e: Event) => onChange(i, (e as MouseEvent).shiftKey ? n : count + 1) }, "+"),
            h("span", { class: "muted small" }, `/ ${n}`),
          )
        : h("b", { class: "ship-count" }, `${n}`);
      return h("div", { class: "ship-row" }, pic, h("span", { class: "ship-name" }, this.designName(f, i)), counter);
    });
    return h("div", { class: "fleet-box" }, ...rows);
  }

  /** MOO1's fleet deployment: choose how many of each ship, a destination on the map, Accept. */
  private orbitPanel(f: FleetView): void {
    const v = this.view;
    const p = v.planets[f.at!];
    append(this.panel, h("h2", {}, this.t("orbit.title")), h("p", { class: "muted" }, p.name));
    if (f.owner !== v.player) {
      append(this.panel, h("p", { class: "enemy" }, this.empireName(f.owner)), this.shipRows(f));
      return;
    }
    const key = `orbit/${p.id}/${f.ships.join(",")}`;
    if (this.deploy?.key !== key) this.deploy = { key, ships: [...f.ships] };
    const ships = this.deploy.ships;
    const any = ships.some((n) => n > 0);
    const limits = any ? this.fleetLimits(ships) : { range: 0, speed: 1 };
    const t = this.map.targeting;
    const dest = t?.dest ?? null;
    if (!t || t.key !== `orbit/${p.id}`) {
      this.map.targeting = {
        key: `orbit/${p.id}`,
        from: p,
        inline: true,
        dest: null,
        range: limits.range,
        speed: limits.speed,
        label: (id) => (p.stargate && v.planets[id].stargate && v.planets[id].owner === v.player ? this.t("map.via_stargate") : null),
        ok: (id) => id !== p.id && v.planets[id].range <= limits.range,
        pick: (to) => {
          this.map.targeting!.dest = to;
          this.renderPanel();
        },
      };
    } else {
      t.range = limits.range;
      t.speed = limits.speed;
      t.ok = (id) => id !== p.id && v.planets[id].range <= limits.range;
      if (dest !== null && !t.ok(dest)) t.dest = null;
    }
    const target = this.map.targeting!.dest ?? null;
    const gate = target !== null && p.stargate && v.planets[target].stargate && v.planets[target].owner === v.player;
    const eta = target !== null ? (gate ? 1 : this.game.eta(p.x, p.y, target, limits.speed)) : 0;
    const change = (i: number, n: number) => {
      ships[i] = Math.max(0, Math.min(f.ships[i], n));
      this.renderPanel();
    };
    const close = () => {
      this.map.targeting = null;
      this.deploy = null;
      this.map.selection = { kind: "planet", id: p.id };
      this.renderPanel();
    };
    const scrap = () => {
      if (any && window.confirm(this.t("orbit.scrap_confirm", { ships: this.shipsLine({ ...f, ships }) }))) {
        this.map.targeting = null;
        this.command({ type: "scrap_ships", planet: p.id, ships: [...ships] });
      }
    };
    const accept = () => {
      if (!any || target === null) return;
      this.map.targeting = null;
      this.deploy = null;
      this.map.selection = { kind: "planet", id: p.id };
      if (!this.command({ type: "send_fleet", from: p.id, to: target, ships: [...ships] })) this.map.selection = { kind: "orbit", id: p.id, owner: f.owner };
    };
    append(
      this.panel,
      this.shipRows(f, ships, change),
      h("p", {}, target !== null ? this.t("orbit.destination", { planet: v.planets[target].name, turns: eta }) : this.t("orbit.choose_destination")),
      any ? h("p", { class: "muted small" }, this.t("orbit.limits", { range: limits.range, speed: limits.speed })) : null,
      h(
        "div",
        { class: "row" },
        h("button", { class: "primary", disabled: !any || target === null, onclick: accept }, this.t("orbit.accept")),
        h("button", { onclick: close }, this.t("btn.cancel")),
        h("button", { class: "mini", disabled: !any, onclick: scrap }, this.t("orbit.scrap")),
      ),
    );
  }

  /** A fleet on its way: its ships, where it goes and, when allowed, a new destination. */
  private fleetPanel(f: FleetView): void {
    const v = this.view;
    const dest = f.dest !== null ? v.planets[f.dest] : null;
    const { range, speed } = this.fleetLimits(f.ships);
    const t = this.map.targeting;
    const target = t && t.key === `fleet/${f.id}` ? (t.dest ?? null) : null;
    const accept = () => {
      if (target === null) return;
      this.map.targeting = null;
      this.command({ type: "redirect_fleet", fleet: f.id!, to: target });
    };
    append(
      this.panel,
      h("h2", {}, this.t("fleet.title")),
      this.shipRows(f),
      dest ? h("p", {}, this.t("fleet.heading", { planet: dest.name, turns: f.eta ?? 0 })) : null,
      f.redirect ? null : h("p", { class: "muted small" }, this.t("fleet.no_redirect")),
    );
    if (!f.redirect) return;
    if (t && t.key === `fleet/${f.id}`) {
      append(
        this.panel,
        h("p", {}, target !== null ? this.t("orbit.destination", { planet: v.planets[target].name, turns: this.game.eta(f.x, f.y, target, speed) }) : this.t("orbit.choose_destination")),
        h(
          "div",
          { class: "row" },
          h("button", { class: "primary", disabled: target === null, onclick: accept }, this.t("orbit.accept")),
          h("button", { onclick: () => ((this.map.targeting = null), this.renderPanel()) }, this.t("btn.cancel")),
        ),
      );
      return;
    }
    append(
      this.panel,
      h(
        "button",
        {
          onclick: () => {
            this.map.targeting = {
              key: `fleet/${f.id}`,
              from: f,
              inline: true,
              dest: null,
              range,
              speed,
              ok: (id) => v.planets[id].range <= range && id !== f.dest,
              pick: (to) => {
                this.map.targeting!.dest = to;
                this.renderPanel();
              },
            };
            this.renderPanel();
          },
        },
        this.t("fleet.redirect"),
      ),
    );
  }

  /** MOO1's planet screen: the land, the ships in orbit and how much it has of everything. */
  private async planetWindow(p: PlanetView): Promise<void> {
    const v = this.view;
    // In space MOO1's picture of the planet's surface (its colonization scene); on a world map the place's view.
    const scene = this.theme.def.map === "space" && p.kind ? picImg(this.theme, `landing/colonize/${PLANET_TYPES.indexOf(p.kind)}`) : null;
    const big = scene ?? this.placeView(p) ?? this.planetPicture(p);
    big.classList.add("window-picture");
    const stat = (label: string, value: string | number, row: HTMLElement | null) =>
      h("div", { class: "window-stat" }, h("div", {}, h("b", {}, label), " ", h("span", {}, String(value))), row);
    const stats: HTMLElement[] = [];
    if (p.owner !== null) {
      const mine = p.owner === v.player;
      const maxPop = mine ? (p.max_pop ?? 0) : 0;
      const maxFact = mine ? (p.colony?.max_factories ?? 0) : 0;
      // One icon per 5 people and per 10 factories (more per icon on big planets), faded up to the maximum.
      const perFact = Math.max(10, Math.ceil(Math.max(p.factories ?? 0, maxFact) / 400) * 10);
      stats.push(stat(this.t("planet.pop"), maxPop ? `${p.pop} / ${maxPop}` : (p.pop ?? 0), iconRow(this.theme, "pop", p.pop ?? 0, 5, maxPop)));
      stats.push(stat(this.t("planet.factories"), maxFact ? `${p.factories} / ${maxFact}` : (p.factories ?? 0), iconRow(this.theme, "factory", p.factories ?? 0, perFact, maxFact)));
      if (mine) stats.push(stat(this.t("planet.waste"), p.colony?.waste ?? 0, iconRow(this.theme, "waste", p.colony?.waste ?? 0, 2)));
      stats.push(stat(this.t("planet.bases"), p.bases ?? 0, iconRow(this.theme, "base", p.bases ?? 0, 1)));
      if (p.shield) stats.push(stat(this.t("planet.shield"), p.shield, iconRow(this.theme, "shield", p.shield, 5)));
    } else if (p.kind) {
      stats.push(stat(this.t("planet.max_pop"), p.max_pop ?? 0, iconRow(this.theme, "pop", 0, 5, p.max_pop ?? 0)));
    }
    const facts = this.facts(this.placeFacts(p, true));
    const fleets = v.fleets.filter((f) => f.at === p.id);
    const garrison = fleets.length
      ? [
          h("h3", {}, this.t("orbit.title")),
          ...fleets.map((f) =>
            h(
              "div",
              { class: "window-fleet" },
              h("div", { class: "muted small" }, this.empireName(f.owner)),
              h(
                "div",
                { class: "window-ships" },
                ...f.ships.map((n, i) => {
                  if (n <= 0) return null;
                  const [look] = f.looks[i] ?? [0, 0];
                  const src = designPicture(this.theme, look, v.empires[f.owner].banner);
                  return h("div", { class: "window-ship", title: this.designName(f, i) }, h("img", { src, alt: "" }), h("b", {}, `× ${n}`), h("div", { class: "small" }, this.designName(f, i)));
                }),
              ),
            ),
          ),
        ]
      : [];
    await modal(p.name, [h("div", { class: "planet-window" }, big, h("div", { class: "window-side" }, facts, ...stats)), ...garrison], [this.t("btn.close")], "wide");
  }

  /** The empire's income, research and treasury in full (the panel's header shows only the numbers). */
  private async treasuryDialog(): Promise<void> {
    const v = this.view;
    const row = (icon: string, text: string, hint?: string) =>
      h("div", { class: "treasury-row" }, h("img", { src: uiIcon(this.theme, icon), alt: "" }), h("div", {}, h("div", {}, text), hint ? h("div", { class: "muted small" }, hint) : null));
    await modal(
      this.t("game.year", { year: v.year }),
      [
        row("gold", this.t("game.production", { bc: v.total_prod - v.maintenance, maint: v.maintenance }), this.t("game.production_hint")),
        row("research", this.t("game.research", { rp: v.research })),
        row("reserve", this.t("game.reserve", { bc: v.reserve })),
      ],
      [this.t("btn.close")],
    );
  }

  /** MOO1's FLEET button: every fleet of the player, to find and select it on the map. */
  private async fleetsDialog(): Promise<void> {
    const v = this.view;
    const mine = v.fleets.filter((f) => f.owner === v.player);
    let picked: FleetView | null = null;
    let close: (i: number) => void = () => {};
    const rows = mine.map((f) => {
      const where = f.at !== null ? this.t("fleets.at", { planet: v.planets[f.at].name }) : this.t("fleets.to", { planet: f.dest !== null ? v.planets[f.dest].name : "?", turns: f.eta ?? 0 });
      return h(
        "tr",
        { class: "clickable", onclick: () => ((picked = f), close(0)) },
        h("td", {}, where),
        h(
          "td",
          {},
          ...f.ships.map((n, i) => (n > 0 ? h("span", { class: "fleet-chip", title: this.designName(f, i) }, h("img", { src: designPicture(this.theme, f.looks[i]?.[0] ?? 0, v.empires[v.player].banner), alt: "" }), `${n}`) : null)),
        ),
      );
    });
    const body = mine.length ? h("table", { class: "list fleets" }, ...rows) : h("p", { class: "muted" }, this.t("fleets.none"));
    await modal(this.t("fleets.title"), [h("p", { class: "muted small" }, this.t("fleets.hint")), body], [this.t("btn.close")], "wide", (c) => (close = c));
    if (picked) {
      const f = picked as FleetView;
      this.map.selection = f.at !== null ? { kind: "orbit", id: f.at, owner: f.owner } : { kind: "fleet", id: f.id! };
      this.map.centerOn(f.x, f.y);
      this.renderPanel();
    }
  }

  /** Transports in flight: where they go; with Hyperspace Communications, a new destination. */
  private transportPanel(tr: TransportView): void {
    const v = this.view;
    const range = v.abilities.fuel_range;
    append(
      this.panel,
      h("h2", {}, this.t("transport.panel_title")),
      h("p", {}, this.t("transport.heading", { pop: tr.pop ?? 0, planet: v.planets[tr.dest].name, turns: Math.max(1, tr.eta) })),
      tr.redirect ? null : h("p", { class: "muted small" }, this.t("transport.no_redirect")),
      h(
        "button",
        {
          disabled: !tr.redirect,
          onclick: () => {
            this.map.targeting = {
              from: { x: tr.x, y: tr.y },
              range,
              speed: v.abilities.engine_warp,
              transport: true,
              ok: (id) => v.planets[id].range <= range,
              pick: (to) => {
                this.map.targeting = null;
                this.command({ type: "redirect_transport", transport: tr.id, to });
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
      transport: true,
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
    // Events already shown as a screen this turn (by their index in the turn's events).
    const shown = new Set<number>();
    const landed = new Set<number>();
    // Battles and questions interrupt the turn: fight each battle, answer each question,
    // then let the turn go on.
    for (let guard = 0; this.view.in_battle && guard < 200; guard++) {
      const q = this.view.question;
      if (this.view.battle && this.view.battle.winner === null) {
        await store.put("kv", AUTOSAVE, this.game.save());
        await battleScreen(this.ctx);
      } else if (q?.type === "colonize") {
        const yes = await exploreScreen(this.ctx, q.planet, { colony: true, byScanner: false });
        this.command({ type: "answer", yes });
      } else if (q?.type === "bomb") {
        const yes = await bombAskScreen(this.ctx, q.planet, q.inbound);
        this.command({ type: "answer", yes });
      }
      this.game.endTurn();
      this.refresh();
      await this.turnScreens(shown, landed, true);
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
    await this.turnScreens(shown, landed, false);
    const own = (e: GameEvent) => e.to.includes(v.player);
    const screened = ["explored", "colony_founded", "bombed", "spies_caught", "tech_stolen", "sabotaged", "ships_built", "factories_complete"];
    const events = v.events.filter((e) => !screened.includes(e.type) && !news.includes(e) && visible(e));
    const others = v.events.filter((e) => e.type === "factories_complete" && own(e));
    if (events.length || others.length) {
      const list = h("ul", { class: "events" }, ...[...events, ...others].map((e) => h("li", { class: `ev-${e.type}`, onclick: () => this.focusEvent(e) }, this.eventText(e))));
      await modal(this.t("report.title", { year: v.year }), list, [this.t("btn.ok")], "report");
    }
    for (const e of v.events) if (e.type === "tech_discovered" && e.to.includes(v.player)) await newTechScreen(this.ctx, e, this.eventText(e));
    await newShipsScreen(this.ctx, v.events.filter((e) => e.type === "ships_built" && own(e)));
    if (await this.checkEnd()) return;
    await this.decisions();
  }

  /**
   * The turn's screens for the player, each once: new colonies landing, exploration
   * reports, bombings and, once the turn is over, spies caught, techs stolen and sabotage.
   * `during`: the turn is still interrupted (only what happened so far).
   */
  private async turnScreens(shown: Set<number>, landed: Set<number>, during: boolean): Promise<void> {
    const events = this.view.events;
    const me = this.view.player;
    for (let i = 0; i < events.length; i++) {
      const e = events[i];
      if (shown.has(i) || !e.to.includes(me)) continue;
      if (e.type === "colony_founded" && !landed.has(e.planet as number) && this.view.planets[e.planet as number].owner === me) {
        shown.add(i);
        landed.add(e.planet as number);
        await landingScreen(this.ctx, e.planet as number);
        this.refresh();
      } else if (e.type === "explored") {
        shown.add(i);
        await exploreScreen(this.ctx, e.planet as number, { colony: false, byScanner: !!e.by_scanner });
      } else if (e.type === "bombed") {
        shown.add(i);
        await bombShowScreen(this.ctx, e);
      }
    }
    if (during) return;
    const caught = events.filter((e, i) => e.type === "spies_caught" && e.to.includes(me) && !shown.has(i));
    await caughtScreen(this.ctx, caught);
    for (const e of events) {
      if (!e.to.includes(me)) continue;
      if (e.type === "tech_stolen") await stolenScreen(this.ctx, e);
      else if (e.type === "sabotaged" && e.by !== me) await sabotageDoneScreen(this.ctx, (e.by as number | null) ?? null, e.kind as SabotageKind, e.count as number, e.planet as number);
    }
  }

  /** Votes, messages, spy successes and research choices waiting for the player. */
  private async decisions(): Promise<void> {
    await councilDialog(this.ctx);
    if (await this.checkEnd()) return;
    await deliverMessages(this.ctx);
    await spyChances(this.ctx);
    if (needsChoice(this.ctx)) {
      await chooseTechs(this.ctx);
      this.refresh();
    }
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
    await techScreen(this.ctx);
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
    const tax = slider({
      label: this.t("empire.tax"),
      title: this.t("empire.tax_hint"),
      value: v.tax,
      max: 200,
      step: 5,
      plusStep: 5,
      format: (x) => this.t("empire.tax_value", { pct: (x / 10).toFixed(1), bc: Math.floor((v.total_prod * x) / 2000) }),
      onCommit: (x) => this.command({ type: "set_tax", tenths: x }),
    });
    void modal(
      this.t("empire.title", { n: mine.length }),
      [
        table,
        tax,
        h("p", { class: "muted small" }, this.t("empire.income", { trade: v.total_trade, spies: v.spying_maint, reserve: v.reserve })),
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
