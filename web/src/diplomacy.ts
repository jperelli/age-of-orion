// Diplomacy: the empires screen (relations, spies, security), audiences with computer
// rulers, their messages and proposals, spy successes and the Galactic Council.
import type { Ctx } from "./ctx";
import { shipParts } from "./engine";
import { techName } from "./research";
import { FIELDS, RACES, type AudienceOptions, type Incoming, type MsgParams, type RelationView, type Reply, type SabotageKind, type Talk, type TechRef } from "./types";
import { append, clear, h, keepScroll, modal, toast } from "./ui";

type T = (k: string, p?: Record<string, string | number>) => string;

const tr = (ctx: Ctx): T => (k, p) => ctx.theme.t(k, p);

export function empireName(ctx: Ctx, id: number): string {
  return ctx.theme.t(`race.${ctx.view().empires[id].race}.plural`);
}

export function techRefName(ctx: Ctx, [f, n]: TechRef): string {
  const key = n > 50 ? "advanced" : (shipParts().techs?.[f]?.[n] ?? "");
  return techName(ctx, FIELDS[f], key, n);
}

/** Attitude words for a relation value (-100..100). */
export function attitude(rel: number): string {
  if (rel <= -75) return "feud";
  if (rel <= -50) return "hostile";
  if (rel <= -25) return "troubled";
  if (rel < -5) return "tense";
  if (rel <= 5) return "neutral";
  if (rel <= 25) return "relaxed";
  if (rel <= 50) return "amiable";
  if (rel <= 75) return "friendly";
  return "harmony";
}

function portrait(ctx: Ctx, id: number): HTMLElement {
  const e = ctx.view().empires[id];
  const url = ctx.theme.url(`race/${RACES.indexOf(e.race)}`);
  return url
    ? h("img", { class: "portrait pixel", src: url, alt: "" })
    : h("div", { class: "crest big", style: `--banner:${ctx.theme.banner(e.banner)}` });
}

function offerText(ctx: Ctx, bc: number, tech: TechRef | null): string {
  const t = tr(ctx);
  if (tech) return techRefName(ctx, tech);
  return bc ? `${bc} ${t("unit.bc")}` : "";
}

/** The text of a diplomatic message (`diplo.<kind>`). */
export function msgText(ctx: Ctx, kind: number, from: number, p: Partial<MsgParams> = {}): string {
  const t = tr(ctx);
  const v = ctx.view();
  const params: Record<string, string | number> = {
    leader: v.empires[from].leader,
    race: empireName(ctx, from),
    me: empireName(ctx, v.player),
    year: v.year,
    bc: p.bc ?? 0,
  };
  if (p.planet !== null && p.planet !== undefined) params.planet = v.planets[p.planet]?.name ?? "?";
  if (p.field !== null && p.field !== undefined) params.field = t(`field.${FIELDS[p.field]}`);
  if (p.empire !== null && p.empire !== undefined) params.empire = empireName(ctx, p.empire);
  if (p.tech) params.tech = techRefName(ctx, p.tech);
  params.gift = p.tech ? techRefName(ctx, p.tech) : `${p.bc ?? 0} ${t("unit.bc")}`;
  let text = t(`diplo.${kind}`, params);
  const offer = offerText(ctx, p.offer_bc ?? 0, p.offer_tech ?? null);
  if (offer && kind >= 24 && kind <= 30) text += " " + t("diplo.offer", { offer });
  return text;
}

// ------------------------------------------------------------- empires screen

export function diplomacyDialog(ctx: Ctx): Promise<number> {
  const t = tr(ctx);
  const body = h("div", { class: "diplomacy" });
  const render = () => keepScroll(body, draw);
  const draw = () => {
    const v = ctx.view();
    const sec = h("input", { type: "range", min: 0, max: 200, step: 5, value: v.security });
    const secVal = h("span", { class: "bc" }, `${(v.security / 10).toFixed(1)}%`);
    sec.addEventListener("input", () => (secVal.textContent = `${(Number(sec.value) / 10).toFixed(1)}%`));
    sec.addEventListener("change", () => (ctx.command({ type: "set_security", tenths: Number(sec.value) }), render()));
    clear(body);
    append(
      body,
      h(
        "div",
        { class: "diplo-top" },
        h("div", { class: "slider-row" }, h("label", { title: t("diplomacy.security_hint") }, t("diplomacy.security")), sec, secVal),
        h("p", { class: "muted small" }, t("diplomacy.upkeep", { spies: v.spying_maint, trade: v.total_trade })),
        v.final_war ? h("p", { class: "warn" }, t("diplomacy.final_war")) : null,
      ),
    );
    append(body, card(ctx, v.player, null, render));
    for (const r of v.relations) append(body, card(ctx, r.empire, r, render));
  };
  draw();
  return modal(t("rivals.title"), body, [t("btn.close")], "wide");
}

function card(ctx: Ctx, id: number, r: RelationView | null, render: () => void): HTMLElement {
  const t = tr(ctx);
  const v = ctx.view();
  const e = v.empires[id];
  const lines: (HTMLElement | null)[] = [
    h("b", { style: `color:${ctx.theme.banner(e.banner)}` }, empireName(ctx, id), id === v.player ? ` ${t("rivals.you")}` : ""),
    h("div", {}, t("rivals.leader", { name: e.leader })),
    h("div", { class: "muted" }, t(`race.${e.race}.trait`)),
    h("div", {}, e.alive ? t("rivals.colonies", { n: e.known_colonies }) : t("rivals.eliminated")),
  ];
  const extra: (HTMLElement | null)[] = [];
  if (r && e.alive) {
    if (!r.met) {
      lines.push(h("div", { class: "muted" }, t("diplomacy.not_met")));
    } else {
      if (r.personality !== null) {
        lines.push(h("div", { class: "muted small" }, t(`trait1.${r.personality}`) + (r.objective !== null ? ` · ${t(`trait2.${r.objective}`)}` : "")));
      }
      lines.push(
        h("div", { class: `treaty treaty-${r.treaty}` }, t(`treaty.${r.treaty}`), r.ceasefire > 0 ? ` (${t("diplomacy.ceasefire", { n: r.ceasefire })})` : ""),
        e.ai ? h("div", {}, t("diplomacy.attitude", { a: t(`attitude.${attitude(r.relation)}`) })) : null,
        r.trade_bc ? h("div", {}, t("diplomacy.trade", { bc: r.trade_bc, pct: r.trade_percent })) : null,
        r.contact ? null : h("div", { class: "muted small" }, t("diplomacy.no_contact")),
      );
      extra.push(spyBox(ctx, r, render));
      if (r.audience) {
        extra.push(
          h(
            "button",
            {
              class: "primary",
              onclick: async () => {
                await audienceDialog(ctx, id);
                render();
              },
            },
            t("diplomacy.audience"),
          ),
        );
      }
    }
  }
  return h("div", { class: `rival ${e.alive ? "" : "dead"}` }, portrait(ctx, id), h("div", { class: "rival-info" }, ...lines), extra.length ? h("div", { class: "rival-actions" }, ...extra) : null);
}

function spyBox(ctx: Ctx, r: RelationView, render: () => void): HTMLElement {
  const t = tr(ctx);
  const slider = h("input", { type: "range", min: 0, max: 100, step: 1, value: r.spying, disabled: !r.contact });
  const val = h("span", { class: "bc" }, `${(r.spying / 10).toFixed(1)}%`);
  slider.addEventListener("input", () => (val.textContent = `${(Number(slider.value) / 10).toFixed(1)}%`));
  slider.addEventListener("change", () => (ctx.command({ type: "set_spying", target: r.empire, tenths: Number(slider.value) }), render()));
  const mode = h(
    "select",
    { onchange: (ev: Event) => (ctx.command({ type: "set_spy_mode", target: r.empire, mode: (ev.target as HTMLSelectElement).value as RelationView["spy_mode"] }), render()) },
    ...(["hide", "espionage", "sabotage"] as const).map((m) => h("option", { value: m, selected: r.spy_mode === m }, t(`spy.mode.${m}`))),
  );
  const report = r.spy_levels
    ? h("div", { class: "muted small" }, t("spy.levels"), " ", FIELDS.map((f, i) => `${t(`field.${f}`)} ${r.spy_levels![i]}`).join(", "))
    : null;
  return h(
    "div",
    { class: "spy-box" },
    h("div", {}, t("spy.count", { n: r.spies })),
    h("div", { class: "slider-row" }, h("label", {}, t("spy.spending")), slider, val),
    h("label", { class: "field" }, h("span", {}, t("spy.mission")), mode),
    report,
  );
}

// ------------------------------------------------------------- audience

/** An audience with a computer ruler (1oom `game_audience`). */
export function audienceDialog(ctx: Ctx, ai: number): Promise<number> {
  const t = tr(ctx);
  const log = h("div", { class: "audience-log" });
  const menu = h("div", { class: "audience-menu" });
  const say = (text: string, who: "them" | "me" = "them") => {
    append(log, h("p", { class: `say-${who}` }, text));
    log.scrollTop = log.scrollHeight;
  };
  const rel = () => ctx.view().relations.find((r) => r.empire === ai);
  const opts = (): AudienceOptions | null => rel()?.audience ?? null;
  const first = opts();
  if (!first) return modal(t("audience.title", { race: empireName(ctx, ai) }), h("p", {}, t("audience.unavailable")), [t("btn.close")]);
  say(msgText(ctx, first.greeting, ai));

  const button = (label: string, enabled: boolean, fn: () => void) => h("button", { disabled: !enabled, onclick: fn }, label);
  const back = () => button(t("audience.back"), true, main);

  const handle = (reply: Reply | string, afterOk?: () => void) => {
    ctx.refresh();
    if (typeof reply === "string") {
      toast(t(`error.${reply}`));
      main();
      return;
    }
    if (reply.sweetener) {
      const gift = offerText(ctx, reply.sweetener.bc, reply.sweetener.tech);
      say(t("audience.sweetener", { gift }));
      clear(menu).append(
        button(t("audience.agree"), true, () => handle(ctx.game.talk(ai, { type: "accept_sweetener" }))),
        button(t("audience.forget"), true, () => handle(ctx.game.talk(ai, { type: "decline_sweetener" }))),
      );
      return;
    }
    if (reply.kind) say(msgText(ctx, reply.kind, ai, reply.params));
    else if (reply.accepted) say(t("audience.done"));
    if (afterOk) afterOk();
    else main();
  };
  const ask = (label: string, talk: Talk) => {
    say(label, "me");
    handle(ctx.game.talk(ai, talk));
  };

  const main = () => {
    const o = opts();
    clear(menu);
    if (!o || !o.talks) return;
    if (o.later) {
      say(msgText(ctx, 74, ai));
      return;
    }
    menu.append(
      button(t("audience.treaty"), o.nap || o.alliance || o.peace || o.war_on.length > 0 || o.break_alliance_with.length > 0, treatyMenu),
      button(t("audience.trade"), o.trade.length > 0, tradeMenu),
      button(t("audience.threaten"), o.threaten || o.break_treaty || o.break_trade, threatMenu),
      button(t("audience.tribute"), true, tributeMenu),
      button(t("audience.exchange"), o.exchange, () => {
        say(t("audience.exchange"), "me");
        const reply = ctx.game.talk(ai, { type: "tech_offers" });
        if (typeof reply !== "string" && reply.tech_deals.length) {
          ctx.refresh();
          techMenu(reply);
        } else handle(reply);
      }),
    );
  };

  const treatyMenu = () => {
    const o = opts()!;
    clear(menu).append(
      button(t("audience.nap"), o.nap, () => ask(t("audience.nap"), { type: "propose_nap" })),
      button(t("audience.alliance"), o.alliance, () => ask(t("audience.alliance"), { type: "propose_alliance" })),
      button(t("audience.peace"), o.peace, () => ask(t("audience.peace"), { type: "propose_peace" })),
      ...o.war_on.map((x) =>
        button(t("audience.war_on", { empire: empireName(ctx, x) }), true, () => ask(t("audience.war_on", { empire: empireName(ctx, x) }), { type: "declare_war_on", target: x })),
      ),
      ...o.break_alliance_with.map((x) =>
        button(t("audience.break_with", { empire: empireName(ctx, x) }), true, () =>
          ask(t("audience.break_with", { empire: empireName(ctx, x) }), { type: "break_alliance_with", target: x }),
        ),
      ),
      back(),
    );
  };
  const tradeMenu = () => {
    const o = opts()!;
    clear(menu).append(...o.trade.map((bc) => button(t("audience.trade_bc", { bc }), true, () => ask(t("audience.trade_bc", { bc }), { type: "propose_trade", bc }))), back());
  };
  const threatMenu = () => {
    const o = opts()!;
    clear(menu).append(
      button(t("audience.break_treaty"), o.break_treaty, () => {
        if (window.confirm(t("audience.break_confirm"))) ask(t("audience.break_treaty"), { type: "break_treaty" });
      }),
      button(t("audience.break_trade"), o.break_trade, () => ask(t("audience.break_trade"), { type: "break_trade" })),
      button(t("audience.threat"), o.threaten, () => ask(t("audience.threat"), { type: "threaten" })),
      back(),
    );
  };
  const tributeMenu = () => {
    const o = opts()!;
    clear(menu).append(
      ...o.tribute.map((bc) => button(`${bc} ${t("unit.bc")}`, true, () => ask(t("audience.tribute_bc", { bc }), { type: "tribute_bc", bc }))),
      button(t("audience.tribute_tech"), true, () => {
        const reply = ctx.game.talk(ai, { type: "tribute_techs" });
        if (typeof reply !== "string" && reply.techs.length) {
          clear(menu).append(
            ...reply.techs.map((x) =>
              button(techRefName(ctx, x), true, () => ask(t("audience.tribute_give", { tech: techRefName(ctx, x) }), { type: "tribute_tech", field: x[0], tech: x[1] })),
            ),
            back(),
          );
        } else handle(reply);
      }),
      back(),
    );
  };
  const techMenu = (reply: Reply) => {
    say(t("audience.exchange_which"));
    clear(menu).append(
      ...reply.tech_deals.map((d) =>
        button(techRefName(ctx, d.want), true, () => {
          say(t("audience.exchange_give"));
          clear(menu).append(
            ...d.give.map((g) =>
              button(techRefName(ctx, g), true, () =>
                ask(t("audience.exchange_deal", { want: techRefName(ctx, d.want), give: techRefName(ctx, g) }), { type: "exchange_tech", want: d.want, give: g }),
              ),
            ),
            back(),
          );
        }),
      ),
      back(),
    );
  };

  main();
  const head = h("div", { class: "audience-head" }, portrait(ctx, ai), h("div", {}, h("b", {}, ctx.view().empires[ai].leader), h("div", { class: "muted" }, empireName(ctx, ai))));
  return modal(t("audience.title", { race: empireName(ctx, ai) }), [head, log, menu], [t("audience.goodbye")], "wide audience");
}

// ------------------------------------------------------------- messages

/** Shows the computer rulers' messages and proposals one by one and sends the answers. */
export async function deliverMessages(ctx: Ctx): Promise<void> {
  for (let guard = 0; guard < 40; guard++) {
    const v = ctx.view();
    const first = v.messages[0];
    if (!first) return;
    const [index, m] = first;
    const answer = await showMessage(ctx, m);
    if (!ctx.command({ type: "answer_message", index, accept: answer.accept, pick: answer.pick })) {
      ctx.command({ type: "answer_message", index, accept: false });
    }
  }
}

async function showMessage(ctx: Ctx, m: Incoming): Promise<{ accept: boolean; pick: TechRef | null }> {
  const t = tr(ctx);
  const head = h("div", { class: "audience-head" }, portrait(ctx, m.from), h("div", {}, h("b", {}, ctx.view().empires[m.from].leader), h("div", { class: "muted" }, empireName(ctx, m.from))));
  const text = h("p", { class: "say-them" }, msgText(ctx, m.kind, m.from, m.params));
  const title = t("audience.title", { race: empireName(ctx, m.from) });
  if (!m.needs_answer) {
    await modal(title, [head, text], [t("btn.ok")], "audience");
    return { accept: false, pick: null };
  }
  if (m.kind === 29 && m.params.techs.length) {
    const select = h("select", {}, ...m.params.techs.map((x, i) => h("option", { value: i }, techRefName(ctx, x))));
    const r = await modal(title, [head, text, h("label", { class: "field" }, h("span", {}, t("audience.pick_tech")), select)], [t("audience.accept"), t("audience.reject")], "audience");
    return { accept: r === 0, pick: r === 0 ? m.params.techs[Number(select.value)] : null };
  }
  const r = await modal(title, [head, text], [t("audience.accept"), t("audience.reject")], "audience");
  return { accept: r === 0, pick: null };
}

// ------------------------------------------------------------- spies

/** Lets the player use what the spies achieved this turn. Skipped chances lapse. */
export async function spyChances(ctx: Ctx): Promise<void> {
  const t = tr(ctx);
  // Last first: using a chance removes it, which does not move the earlier ones.
  for (const [index, c] of [...ctx.view().spy_chances].reverse()) {
    const target = empireName(ctx, c.target);
    if (c.type === "steal") {
      const r = await modal(
        t("spy.steal_title"),
        h("p", {}, t("spy.steal_text", { empire: target })),
        [...c.techs.map((x) => `${t(`field.${FIELDS[x[0]]}`)}: ${techRefName(ctx, x)}`), t("spy.skip")],
      );
      if (r >= 0 && r < c.techs.length && ctx.command({ type: "steal_tech", index, field: c.techs[r][0] })) {
        toast(t("spy.stolen", { tech: techRefName(ctx, c.techs[r]) }));
      }
      continue;
    }
    const planets = ctx.view().planets.filter((p) => p.owner === c.target);
    if (!planets.length) continue;
    const select = h(
      "select",
      {},
      ...planets.map((p) => h("option", { value: p.id }, `${p.name} (${t("planet.factories")} ${p.factories ?? "?"}, ${t("planet.bases")} ${p.bases ?? "?"})`)),
    );
    const kinds: SabotageKind[] = ["factories", "bases", "revolt"];
    const r = await modal(
      t("spy.sabotage_title"),
      [h("p", {}, t("spy.sabotage_text", { empire: target })), h("label", { class: "field" }, h("span", {}, t("spy.target")), select)],
      [...kinds.map((k) => t(`spy.sabotage.${k}`)), t("spy.skip")],
    );
    if (r < 0 || r >= 3) continue;
    const planet = Number(select.value);
    const before = ctx.view().planets[planet];
    if (ctx.command({ type: "sabotage", index, planet, kind: kinds[r] })) {
      const after = ctx.view().planets[planet];
      const n =
        kinds[r] === "factories" ? (before.factories ?? 0) - (after.factories ?? 0) : kinds[r] === "bases" ? (before.bases ?? 0) - (after.bases ?? 0) : 0;
      toast(t(`spy.done.${kinds[r]}`, { n, planet: after.name }));
    }
  }
}

// ------------------------------------------------------------- council

/** The Galactic Council: vote, then accept or refuse the result (1oom `game_election`). */
export async function councilDialog(ctx: Ctx): Promise<void> {
  const t = tr(ctx);
  let v = ctx.view();
  let el = v.election;
  if (!el) return;
  const [a, b] = el.candidates;
  const name = (x: number) => t("council.candidate", { leader: v.empires[x].leader, race: empireName(ctx, x) });
  if (!el.awaiting_acceptance && !el.has_voted[v.player]) {
    const rows = v.empires
      .filter((e) => e.alive)
      .map((e) => h("tr", {}, h("td", {}, empireName(ctx, e.id)), h("td", {}, t("council.votes", { n: el!.votes[e.id] ?? 0 }))));
    const r = await modal(
      t("council.title"),
      [h("p", {}, t("council.text", { a: name(a), b: name(b), need: Math.floor(((el.total + 1) * 2) / 3), total: el.total })), h("table", { class: "list" }, ...rows), h("p", {}, t("council.your_votes", { n: el.votes[v.player] ?? 0 }))],
      [t("council.vote_for", { race: empireName(ctx, a) }), t("council.vote_for", { race: empireName(ctx, b) }), t("council.abstain")],
      "wide",
    );
    ctx.command({ type: "vote", candidate: r === 0 ? a : r === 1 ? b : null });
    v = ctx.view();
    const res = [...v.events].reverse().find((e) => e.type === "council_result");
    if (res) {
      const got = res.votes as number[];
      const elected = res.elected as number | null;
      await modal(
        t("council.title"),
        h("p", {}, t(elected === null ? "council.nobody" : "council.elected", { a: name(a), b: name(b), va: got[0], vb: got[1], winner: elected === null ? "" : name(elected) })),
        [t("btn.ok")],
      );
    }
  }
  v = ctx.view();
  el = v.election;
  if (el?.awaiting_acceptance && el.elected !== null) {
    const r = await modal(t("council.title"), h("p", {}, t("council.accept_text", { winner: name(el.elected) })), [t("council.accept"), t("council.refuse")]);
    ctx.command({ type: "accept_council", accept: r !== 1 });
  }
}
