// MOO1's smaller screens around a turn: exploration (1oom `ui_explore`), the landing of a new
// colony (`ui_landing`), bombing (`ui_bomb_ask`, `ui_bomb_show`), espionage (`ui_spy_steal`,
// `ui_spy_stolen`), sabotage (`ui_spy_sabotage_ask`, `ui_spy_sabotage_done`), new ships
// (`ui_newships`) and the spies caught (`ui_caught`). Pictures by name, with the placeholder
// for those the theme lacks.
import { TUNE, audio } from "./audio";
import type { Ctx } from "./ctx";
import { empireName, techRefName } from "./diplomacy";
import { growthLines, kindLines, specialLines } from "./explain";
import { designPicture, picImg, placeholder, sprite } from "./pictures";
import { newTechScreen } from "./research";
import { FIELDS, PLANET_TYPES, RACES, type GameEvent, type PlanetView, type SabotageKind, type SabotageTarget, type SpyChance } from "./types";
import { h, modal } from "./ui";

const S = 2; // MOO1 pixels to screen pixels

function t(ctx: Ctx, k: string, p?: Record<string, string | number>): string {
  return ctx.theme.t(k, p);
}

const typeOf = (p: PlanetView) => Math.max(0, PLANET_TYPES.indexOf(p.kind ?? "terran"));

/** A race's singular name ("Klackon"), as in "the Klackon colony". */
const raceOf = (ctx: Ctx, empire: number) => t(ctx, `race.${ctx.view().empires[empire].race}`);

/** A box of MOO1 pixels at scale S holding a picture and things placed on it. */
function stage(w: number, hgt: number, ...children: (HTMLElement | null)[]): HTMLElement {
  return h("div", { class: "moo1-scene stage", style: `width:${w * S}px;height:${hgt * S}px` }, ...children);
}

function at(el: HTMLElement, x: number, y: number): HTMLElement {
  el.style.position = "absolute";
  el.style.left = `${x * S}px`;
  el.style.top = `${y * S}px`;
  return el;
}

/** A picture `w` x `h` MOO1 pixels, or the placeholder naming it. */
function pic(ctx: Ctx, name: string, w: number, hgt: number): HTMLElement {
  const el = ctx.theme.hasPic(name) ? picImg(ctx.theme, name) : placeholder(ctx.theme, name, w * S, hgt * S);
  el.style.width = `${w * S}px`;
  el.style.height = `${hgt * S}px`;
  return el;
}

/** What MOO1's exploration screen says about a planet: its land, growth and special. */
function planetLines(ctx: Ctx, p: PlanetView): HTMLElement[] {
  const v = ctx.view();
  const out: HTMLElement[] = [h("h3", {}, p.name), h("div", {}, t(ctx, `planet.${p.kind}`))];
  if (p.kind && p.kind !== "not_habitable") {
    out.push(h("div", { class: "muted small" }, kindLines(ctx.theme, p.kind)[0] ?? ""));
    if (p.growth && p.growth !== "normal") out.push(h("div", { class: "small" }, growthLines(ctx.theme, p.growth, v.empires[v.player].race).join(" ")));
    if (p.special && p.special !== "normal") out.push(h("div", { class: "small" }, specialLines(ctx.theme, p.special).join(" ")));
  }
  return out;
}

// ------------------------------------------------------------------ exploration and landing

/**
 * MOO1's exploration screen: the planet's land (`explore/<type>`, with a colony when someone
 * holds it) and what the scouts found; with a colony ship there, "Build a new colony?" (true
 * for yes).
 */
export async function exploreScreen(ctx: Ctx, planet: number, o: { colony: boolean; byScanner: boolean }): Promise<boolean> {
  const p = ctx.view().planets[planet];
  const name = `explore/${typeOf(p)}${p.owner !== null ? "/colony" : ""}`;
  const text = h(
    "div",
    { class: "screen-text" },
    h("p", { class: "lead" }, t(ctx, o.colony ? "explore.colony" : o.byScanner ? "explore.scanners" : "explore.scouts")),
    ...planetLines(ctx, p),
  );
  const r = await modal(
    t(ctx, "explore.title"),
    h("div", { class: "screen-row" }, pic(ctx, name, 84, 102), text),
    o.colony ? [t(ctx, "btn.yes"), t(ctx, "btn.no")] : [t(ctx, "btn.continue")],
    o.colony ? "required" : "",
  );
  return o.colony && r === 0;
}

/**
 * MOO1's landing of a new colony (1oom `ui_landing`): the transport comes down on the
 * planet's land, the colonists walk out with their banner, then the colony gets its name.
 */
export async function landingScreen(ctx: Ctx, planet: number): Promise<void> {
  const v = ctx.view();
  const p = v.planets[planet];
  const me = v.empires[v.player];
  const bg = pic(ctx, `landing/colonize/${typeOf(p)}`, 320, 200);
  const ship = sprite(ctx.theme, "landing/transport", 142, 103, S);
  const walk = sprite(ctx.theme, `landing/walk/${me.banner}`, 62, 200, S);
  walk.el.style.visibility = "hidden";
  const caption = at(h("div", { class: "scene-caption" }, t(ctx, "landing.caption", { year: v.year, race: empireName(ctx, v.player) })), 0, 4);
  caption.style.width = `${320 * S}px`;
  const input = h("input", { type: "text", maxlength: 16, value: p.name, class: "scene-input" }) as HTMLInputElement;
  const naming = at(h("div", { class: "scene-namebox" }, h("div", {}, t(ctx, "landing.name")), input), 115, 81);
  naming.style.visibility = "hidden";
  const box = stage(320, 200, at(bg, 0, 0), at(ship.el, 0, -100), at(walk.el, 0, 0), caption, naming);
  audio.play(TUNE.colonize);
  let frame = 0;
  let shipFrame = 0;
  let walkFrame = 0;
  let timer = 0;
  const done = () => {
    clearInterval(timer);
    ship.el.style.top = `${100 * S}px`;
    ship.set(ship.frames - 1);
    walk.el.style.visibility = "visible";
    naming.style.visibility = planet === orionOf(ctx) ? "hidden" : "visible";
    input.focus();
    input.select();
  };
  const tick = () => {
    if (!box.isConnected) return;
    // 1oom `landing_draw_cb1`: four pixels down a frame for 50 frames, landed from then on,
    // the colonists walking out from frame 55, the name asked at 65.
    if (frame < 50) {
      ship.el.style.top = `${(frame * 4 - 100) * S}px`;
      ship.set(shipFrame);
      shipFrame = shipFrame + 1 >= ship.frames ? ship.loop : shipFrame + 1;
    } else ship.set(ship.frames - 1);
    if (frame >= 55) {
      walk.el.style.visibility = "visible";
      walk.set(walkFrame);
      walkFrame = walkFrame + 1 >= walk.frames ? walk.loop : walkFrame + 1;
    }
    if (++frame >= 65) done();
  };
  timer = window.setInterval(tick, 66);
  box.addEventListener("click", () => frame < 65 && ((frame = 65), done()));
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") (box.closest(".dialog")?.querySelector(".dialog-buttons button") as HTMLButtonElement | null)?.click();
  });
  await modal(t(ctx, "landing.title"), box, [t(ctx, "btn.ok")], "wide required");
  clearInterval(timer);
  audio.stopMusic();
  const name = input.value.trim();
  if (name && name !== p.name) ctx.command({ type: "rename_planet", planet, name });
}

function orionOf(ctx: Ctx): number {
  return ctx.view().planets.find((p) => p.orion)?.id ?? -1;
}

// ------------------------------------------------------------------ bombing

/** The bombs falling on a planet (`anim/bombard`), with MOO1's sounds on frames 3, 6 and 15. */
function bombs(ctx: Ctx, animate: boolean): HTMLElement {
  const s = sprite(ctx.theme, "bombard", 84, 102, S);
  if (!animate || s.frames < 2) return s.el;
  let f = 0;
  const timer = window.setInterval(() => {
    if (!s.el.isConnected) {
      if (f > 0) clearInterval(timer);
      return;
    }
    s.set(f);
    if (f === 3) audio.sfx("bomb");
    if (f === 6) audio.sfx("hit");
    if (f === 15) audio.sfx("explosion");
    f = f + 1 >= s.frames ? s.loop : f + 1;
  }, 1000 / 12);
  return s.el;
}

/** MOO1's "Bomb the enemy planet?" (true for yes). */
export async function bombAskScreen(ctx: Ctx, planet: number, inbound: number): Promise<boolean> {
  const v = ctx.view();
  const p = v.planets[planet];
  audio.play(TUNE.bomb);
  const r = await modal(
    t(ctx, "bomb.title"),
    h(
      "div",
      { class: "screen-row" },
      bombs(ctx, false),
      h(
        "div",
        { class: "screen-text" },
        h("p", { class: "lead" }, t(ctx, "bomb.ask", { race: raceOf(ctx, p.owner ?? 0), planet: p.name })),
        h("table", { class: "facts" }, h("tr", {}, h("th", {}, t(ctx, "planet.pop")), h("td", {}, String(p.pop ?? "?"))), h("tr", {}, h("th", {}, t(ctx, "planet.factories")), h("td", {}, String(p.factories ?? "?")))),
        inbound ? h("p", { class: "muted" }, t(ctx, "bomb.inbound", { n: inbound })) : null,
      ),
    ),
    [t(ctx, "bomb.yes"), t(ctx, "btn.cancel")],
    "required",
  );
  audio.stopMusic();
  return r === 0;
}

/** MOO1's bombing result, for the attacker or the colony's owner. */
export async function bombShowScreen(ctx: Ctx, e: GameEvent): Promise<void> {
  const v = ctx.view();
  const planet = e.planet as number;
  const p = v.planets[planet];
  const destroyed = v.events.some((x) => x.type === "colony_destroyed" && x.planet === planet);
  audio.play(destroyed ? TUNE.bomb_destroyed : TUNE.bomb);
  const mine = e.attacker === v.player;
  const lines = [
    t(ctx, mine ? "bomb.done" : "bomb.done_by", { race: empireName(ctx, e.attacker as number), planet: p.name }),
    t(ctx, "bomb.losses", { pop: e.pop_killed as number, factories: e.factories_destroyed as number }),
    destroyed ? t(ctx, "bomb.destroyed", { planet: p.name }) : "",
  ].filter(Boolean);
  await modal(t(ctx, "bomb.title"), h("div", { class: "screen-row" }, bombs(ctx, true), h("div", { class: "screen-text" }, ...lines.map((l, i) => h("p", { class: i === 0 ? "lead" : "" }, l)))), [t(ctx, "btn.continue")]);
  audio.stopMusic();
}

// ------------------------------------------------------------------ espionage

/**
 * MOO1's espionage screen (1oom `ui_spy_steal`): the agents' panel (`spy/espionage`, 216x139)
 * with the target's portrait and a button per field the spies can steal from; then the tech
 * stolen, in the spies' den.
 */
export async function stealScreen(ctx: Ctx, index: number, c: Extract<SpyChance, { type: "steal" }>): Promise<void> {
  const v = ctx.view();
  const race = v.empires[c.target].race;
  const chosen = { field: -1 };
  const head = at(h("div", { class: "stage-text upper" }, t(ctx, "spy.esp_head", { race: t(ctx, `race.${race}`) })), 0, 4);
  head.style.width = `${216 * S}px`;
  const choose = at(h("div", { class: "stage-text" }, t(ctx, "spy.esp_choose")), 17, 59);
  const portrait = picImg(ctx.theme, `race/${RACES.indexOf(race)}`, "stage-portrait");
  portrait.style.width = `${40 * S}px`;
  portrait.style.height = `${34 * S}px`;
  let close: (i: number) => void = () => {};
  const buttons = FIELDS.map((field, i) => {
    const tech = c.techs.find((x) => x[0] === i);
    const b = h("button", { class: "stage-button", disabled: !tech, onclick: () => ((chosen.field = i), close(0)) }, t(ctx, `field.${field}`)) as HTMLButtonElement;
    b.style.width = `${86 * S}px`;
    b.style.height = `${12 * S}px`;
    return at(b, Math.floor(i / 3) * 102 + 14, (i % 3) * 22 + 72);
  });
  const panel = stage(216, 139, at(pic(ctx, "spy/espionage", 216, 139), 0, 0), head, at(portrait, 96, 19), choose, ...buttons);
  audio.play(TUNE.espionage);
  await modal(t(ctx, "spy.steal_title"), panel, [t(ctx, "spy.skip")], "wide", (cl) => (close = cl));
  audio.stopMusic();
  const tech = c.techs.find((x) => x[0] === chosen.field);
  if (!tech || !ctx.command({ type: "steal_tech", index, field: chosen.field })) return;
  const text = t(ctx, "spy.stolen", { tech: techRefName(ctx, tech) });
  await newTechScreen(ctx, { to: [v.player], type: "tech_discovered", field: FIELDS[tech[0]], tech: tech[1], source: "espionage" }, text);
}

/** MOO1's notice of a tech stolen from us (1oom `ui_spy_stolen`). */
export async function stolenScreen(ctx: Ctx, e: GameEvent): Promise<void> {
  const by = e.by as number | null;
  const race = by === null ? t(ctx, "spy.unknown") : raceOf(ctx, by);
  const field = FIELDS.indexOf(e.field as (typeof FIELDS)[number]);
  await modal(t(ctx, "spy.stolen_title", { race }), [h("p", { class: "lead" }, t(ctx, "spy.stolen_text", { race })), h("h3", {}, techRefName(ctx, [field, e.tech as number]))], [t(ctx, "btn.continue")]);
}

// ------------------------------------------------------------------ sabotage

/** The colony under the explosion: the bottom of its picture (1oom draws it clipped). */
function sabotageScene(ctx: Ctx, p: { kind: PlanetView["kind"] }, kind: SabotageKind | null, animate: boolean): HTMLElement {
  const colony = pic(ctx, `explore/${Math.max(0, PLANET_TYPES.indexOf(p.kind ?? "terran"))}/colony`, 84, 102);
  colony.classList.add("sabotage-colony");
  if (!kind) return colony;
  const name = kind === "bases" ? "sabotage/bases" : "sabotage/factories";
  const s = sprite(ctx.theme, name, 82, 33, S);
  if (animate && s.frames > 1) {
    let f = 0;
    const sound = kind === "bases" ? 5 : 11;
    const timer = window.setInterval(() => {
      if (!s.el.isConnected) {
        if (f > 0) clearInterval(timer);
        return;
      }
      s.set(f);
      if (f === sound) audio.sfx("explosion");
      f = f + 1 >= s.frames ? s.loop : f + 1;
    }, 1000 / 12);
  }
  return h("div", { class: "sabotage-scene" }, s.el, colony);
}

/** What a sabotage did, as MOO1 words it (1oom `sabotage_done_draw_cb`). */
function sabotageText(ctx: Ctx, by: number | null, kind: SabotageKind, n: number, p: PlanetView, known: { factories: number; bases: number } | null): string {
  const v = ctx.view();
  const who = by === null ? t(ctx, "sab.unknown") : by === v.player ? t(ctx, "sab.yours") : t(ctx, "sab.theirs", { race: raceOf(ctx, by) });
  const have = known ?? { factories: p.factories ?? 1, bases: p.bases ?? 1 };
  if (n <= 0 && kind === "factories" && !have.factories) return t(ctx, "sab.nofact");
  if (n <= 0 && kind === "bases" && !have.bases) return t(ctx, "sab.nobases");
  if (n <= 0) return `${who} ${t(ctx, "sab.failed")}`;
  if (kind === "revolt") return `${who} ${p.rebellion ? t(ctx, "sab.revolt") : t(ctx, "sab.rebels", { n, unrest: p.pop ? Math.floor(((p.rebels ?? 0) * 100) / p.pop) : 0 })}`;
  return `${who} ${t(ctx, kind === "bases" ? "sab.bases" : "sab.factories", { n })}`;
}

/**
 * MOO1's sabotage screen (1oom `ui_spy_sabotage_ask`): choose one of the target's colonies
 * and what to destroy; then the explosion and what it did.
 */
export async function sabotageScreen(ctx: Ctx, index: number, c: Extract<SpyChance, { type: "sabotage" }>, planets: SabotageTarget[]): Promise<void> {
  const v = ctx.view();
  if (!planets.length) return;
  let i = 0;
  const side = h("div", { class: "screen-text" });
  const picture = h("div", {});
  const draw = () => {
    const p = planets[i];
    picture.replaceChildren(sabotageScene(ctx, p, null, false));
    side.replaceChildren(
      h("p", { class: "lead" }, t(ctx, "sab.choose")),
      h(
        "div",
        { class: "sab-pick" },
        h("button", { class: "mini", disabled: planets.length < 2, onclick: () => ((i = (i + planets.length - 1) % planets.length), draw()) }, "‹"),
        h("b", {}, p.name),
        h("button", { class: "mini", disabled: planets.length < 2, onclick: () => ((i = (i + 1) % planets.length), draw()) }, "›"),
      ),
      ...(p.last_report ? [h("div", { class: "muted small" }, t(ctx, "sab.last"))] : []),
      h(
        "table",
        { class: "facts" },
        ...([
          ["planet.pop", p.pop],
          ["planet.factories", p.factories],
          ["planet.bases", p.bases],
        ] as const).map(([k, n]) => h("tr", {}, h("th", {}, t(ctx, k)), h("td", {}, String(n ?? "?")))),
      ),
    );
  };
  draw();
  const kinds: SabotageKind[] = ["factories", "bases", "revolt"];
  audio.play(TUNE.sabotage);
  const r = await modal(
    t(ctx, "spy.sabotage_title"),
    [h("p", {}, t(ctx, "spy.sabotage_text", { empire: empireName(ctx, c.target) })), h("div", { class: "screen-row" }, picture, side)],
    [...kinds.map((k) => t(ctx, `spy.sabotage.${k}`)), t(ctx, "spy.skip")],
    "wide",
  );
  audio.stopMusic();
  if (r < 0 || r >= 3) return;
  const planet = planets[i].planet;
  const before = ctx.view().planets[planet];
  if (!ctx.command({ type: "sabotage", index, planet, kind: kinds[r] })) return;
  const after = ctx.view().planets[planet];
  const n =
    kinds[r] === "factories"
      ? (before.factories ?? 0) - (after.factories ?? 0)
      : kinds[r] === "bases"
        ? (before.bases ?? 0) - (after.bases ?? 0)
        : Math.max(0, (after.rebels ?? 0) - (before.rebels ?? 0));
  const known = planets[i].last_report ? null : planets[i];
  await sabotageDoneScreen(ctx, v.player, kinds[r], n, planet, known ?? { factories: 1, bases: 1 });
}

/** MOO1's sabotage result (1oom `ui_spy_sabotage_done`), for the saboteur or the victim. */
export async function sabotageDoneScreen(ctx: Ctx, by: number | null, kind: SabotageKind, n: number, planet: number, known: { factories: number; bases: number } | null = null): Promise<void> {
  const p = ctx.view().planets[planet];
  audio.play(TUNE.sabotage_done);
  await modal(
    t(ctx, "spy.sabotage_title"),
    h("div", { class: "screen-row" }, sabotageScene(ctx, p, kind, n > 0 && kind !== "revolt"), h("div", { class: "screen-text" }, h("h3", {}, p.name), h("p", { class: "lead" }, sabotageText(ctx, by, kind, n, p, known)))),
    [t(ctx, "btn.continue")],
  );
  audio.stopMusic();
}

// ------------------------------------------------------------------ new ships, spies caught

/** MOO1's "Fleet Production" (1oom `ui_newships`): the ships built this year, by design. */
export async function newShipsScreen(ctx: Ctx, events: GameEvent[]): Promise<void> {
  const v = ctx.view();
  const count = new Map<number, number>();
  for (const e of events) count.set(e.design as number, (count.get(e.design as number) ?? 0) + (e.count as number));
  if (!count.size) return;
  const me = v.empires[v.player];
  const cells = [...count].map(([d, n]) => {
    const des = v.designs[d];
    return h(
      "div",
      { class: "newship" },
      h("div", { class: "newship-pic" }, h("img", { src: designPicture(ctx.theme, des?.look ?? 0, me.banner), alt: "" }), h("b", {}, String(n))),
      h("div", { class: "small" }, des?.name ?? "?"),
    );
  });
  await modal(t(ctx, "newships.title", { year: v.year }), h("div", { class: "newships" }, ...cells), [t(ctx, "btn.continue")]);
}

/** MOO1's table of spies caught (1oom `ui_caught`): theirs we caught and ours they caught. */
export async function caughtScreen(ctx: Ctx, events: GameEvent[]): Promise<void> {
  const v = ctx.view();
  const rows = new Map<number, [number, number]>();
  for (const e of events) {
    const spy = e.spy as number;
    const target = e.target as number;
    const other = spy === v.player ? target : spy;
    const r = rows.get(other) ?? [0, 0];
    if (spy === v.player) r[1] += e.count as number;
    else r[0] += e.count as number;
    rows.set(other, r);
  }
  if (!rows.size) return;
  await modal(
    t(ctx, "caught.title"),
    h(
      "table",
      { class: "list" },
      h("tr", {}, h("th", {}, ""), h("th", {}, t(ctx, "caught.theirs")), h("th", {}, t(ctx, "caught.ours"))),
      ...[...rows].map(([e, [a, b]]) => h("tr", {}, h("td", {}, empireName(ctx, e)), h("td", {}, String(a)), h("td", {}, String(b)))),
    ),
    [t(ctx, "btn.continue")],
  );
}

/** Lets the player use what the spies achieved this turn: what to steal, what to sabotage.
 * Skipped chances lapse. */
export async function spyChances(ctx: Ctx): Promise<void> {
  // Last first: using a chance removes it, which does not move the earlier ones.
  const v = ctx.view();
  const chances = v.spy_chances.map((c, k) => [c, v.spy_targets[k] ?? []] as const);
  for (const [[index, c], targets] of [...chances].reverse()) {
    if (c.type === "steal") await stealScreen(ctx, index, c);
    else await sabotageScreen(ctx, index, c, targets);
  }
}
