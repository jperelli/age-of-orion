// The stats screen (opened from the year in the panel, and at the end of the game): charts of
// what the empire had each year and did (crates/core/src/stats.rs), the totals, the ships
// built by design and the record against each rival. Once the game is over every empire
// shows, side by side; the player's own counts (clicks, time played...) only for the player.
import type { Ctx } from "./ctx";
import type { EmpireStats, StatsView } from "./types";
import { h, modal } from "./ui";

/** What an empire has, sampled each year. */
const SERIES = ["population", "colonies", "factories", "production", "reserve", "ships", "bases", "techs", "explored", "trade"];
/** What it did, in this order (others the core may add come after). */
const DEEDS = [
  "colonies_founded",
  "worlds_conquered",
  "worlds_lost",
  "colonies_destroyed",
  "colonies_razed",
  "battles",
  "battles_won",
  "battles_lost",
  "ships_built",
  "ships_destroyed",
  "ships_lost",
  "monsters_beaten",
  "monsters_lost_to",
  "bombings",
  "bombed",
  "pop_killed_by_bombs",
  "factories_bombed",
  "invasions",
  "invasions_won",
  "invasions_lost",
  "invaded",
  "invasions_repelled",
  "invasions_fallen",
  "rebellions_put_down",
  "rebellions_lost",
  "troops_lost_in_transit",
  "techs_researched",
  "techs_stolen",
  "techs_lost_to_spies",
  "techs_traded",
  "techs_found",
  "techs_captured",
  "spies_caught",
  "spies_lost",
  "sabotages",
  "sabotaged",
  "councils",
  "council_votes",
  "council_elected",
  "stargates_built",
  "guardian_defeated",
  "orion_colonized",
];
const VERSUS = ["battles_won", "battles_lost", "ships_destroyed", "ships_lost", "worlds_taken", "worlds_lost"];

interface Line {
  label: string;
  color: string;
  values: number[];
}

/** A total's value at each year, from its history of changes; the current year's is the
 * running total (what happened so far this year too). */
function totalsByYear(e: EmpireStats, key: string, years: number): number[] {
  const hist = e.history[key] ?? [];
  const out: number[] = [];
  let j = 0;
  let cur = 0;
  for (let i = 0; i < years; i++) {
    while (j < hist.length && hist[j][0] <= i) cur = hist[j++][1];
    out.push(cur);
  }
  if (years) out[years - 1] = e.totals[key] ?? cur;
  return out;
}

const perYear = (v: number[]) => v.map((x, i) => (i ? x - v[i - 1] : 0));

/** A round step for about `ticks` grid lines up to `max`; whole numbers only. */
function niceStep(max: number, ticks: number): number {
  const raw = Math.max(1, max) / ticks;
  const p = 10 ** Math.floor(Math.log10(raw));
  return Math.max(1, [1, 2, 5, 10].map((m) => m * p).find((s) => s >= raw) ?? 10 * p);
}

/** A chart of values by year: lines (several empires), or bars (one empire, per year). */
function drawChart(canvas: HTMLCanvasElement, years: number[], lines: Line[], bars: boolean, fmt: (n: number) => string, hover: number | null, empty: string): void {
  const dpr = window.devicePixelRatio || 1;
  const W = canvas.clientWidth || 640;
  const H = canvas.clientHeight || 280;
  canvas.width = Math.round(W * dpr);
  canvas.height = Math.round(H * dpr);
  const g = canvas.getContext("2d")!;
  g.setTransform(dpr, 0, 0, dpr, 0, 0);
  g.clearRect(0, 0, W, H);
  const css = getComputedStyle(canvas);
  const ink = css.color || "#333";
  const L = 56, R = 12, T = 10, B = 26;
  const n = years.length;
  const all = lines.flatMap((l) => l.values);
  const max = Math.max(1, ...all);
  const min = Math.min(0, ...all);
  const step = niceStep(max - min, 4);
  const top = Math.ceil(max / step) * step;
  const bottom = Math.floor(min / step) * step;
  const x = (i: number) => L + (n <= 1 ? (W - L - R) / 2 : (i * (W - L - R)) / (n - 1));
  const y = (v: number) => T + ((top - v) * (H - T - B)) / Math.max(1, top - bottom);
  g.font = "11px sans-serif";
  g.fillStyle = ink;
  g.strokeStyle = ink;
  g.globalAlpha = 0.18;
  g.lineWidth = 1;
  for (let v = bottom; v <= top + 1e-9; v += step) {
    g.beginPath();
    g.moveTo(L, y(v));
    g.lineTo(W - R, y(v));
    g.stroke();
  }
  g.globalAlpha = 1;
  g.textAlign = "right";
  g.textBaseline = "middle";
  for (let v = bottom; v <= top + 1e-9; v += step) g.fillText(fmt(v), L - 6, y(v));
  g.textAlign = "center";
  g.textBaseline = "top";
  const span = years.length ? years[n - 1] - years[0] : 0;
  const ys = [1, 2, 5, 10, 20, 25, 50, 100, 200, 500].find((s) => span / s <= 8) ?? 1000;
  for (let i = 0; i < n; i++) if (years[i] % ys === 0 || i === 0) g.fillText(String(years[i]), x(i), H - B + 6);
  if (n < 2) {
    g.textAlign = "center";
    g.textBaseline = "middle";
    g.font = "13px sans-serif";
    g.fillText(empty, (L + W - R) / 2, (T + H - B) / 2);
    return;
  }
  if (bars && lines.length === 1) {
    const w = Math.max(1, ((W - L - R) / Math.max(1, n)) * 0.8);
    g.fillStyle = lines[0].color;
    lines[0].values.forEach((v, i) => {
      if (v) g.fillRect(x(i) - w / 2, Math.min(y(v), y(0)), w, Math.abs(y(v) - y(0)));
    });
  } else {
    for (const l of lines) {
      g.strokeStyle = l.color;
      g.lineWidth = 2;
      g.beginPath();
      l.values.forEach((v, i) => (i ? g.lineTo(x(i), y(v)) : g.moveTo(x(i), y(v))));
      g.stroke();
      if (lines.length === 1 && n > 1) {
        g.lineTo(x(n - 1), y(0));
        g.lineTo(x(0), y(0));
        g.closePath();
        g.globalAlpha = 0.15;
        g.fillStyle = l.color;
        g.fill();
        g.globalAlpha = 1;
      }
    }
  }
  if (hover !== null && hover < n) {
    g.strokeStyle = ink;
    g.globalAlpha = 0.5;
    g.lineWidth = 1;
    g.beginPath();
    g.moveTo(x(hover), T);
    g.lineTo(x(hover), H - B);
    g.stroke();
    g.globalAlpha = 1;
    for (const l of lines) {
      g.fillStyle = l.color;
      g.beginPath();
      g.arc(x(hover), y(l.values[hover] ?? 0), 3.5, 0, Math.PI * 2);
      g.fill();
    }
  }
}

/** The stats screen: during the game the player's empire, every empire once it is over. */
export async function statsScreen(ctx: Ctx): Promise<void> {
  const theme = ctx.theme;
  const t = (k: string, p?: Record<string, string | number>) => theme.t(k, p);
  const data: StatsView = ctx.game.stats();
  const v = ctx.view();
  const years = data.years;
  const me = data.empires.find((e) => e.empire === data.player) ?? data.empires[0];
  // The player's empire first.
  const shown = data.all ? [me, ...data.empires.filter((e) => e !== me)] : [me];
  const name = (e: EmpireStats) => t(`race.${e.race}.plural`);
  const color = (e: EmpireStats) => theme.banner(e.banner);
  const label = (k: string) => {
    const s = t(`stats.metric.${k}`);
    return s === `stats.metric.${k}` ? k.replace(/^ui_/, "").replace(/_/g, " ") : s;
  };
  const num = new Intl.NumberFormat(theme.lang);
  const fmt = (k: string) => (n: number) =>
    k === "ui_seconds" ? `${Math.floor(n / 3600)}:${String(Math.floor((n % 3600) / 60)).padStart(2, "0")}` : Math.abs(n) >= 10000 ? `${num.format(Math.round(n / 1000))}k` : num.format(n);

  const has = (k: string) => shown.some((e) => (e.totals[k] ?? 0) !== 0);
  const deeds = [...DEEDS, ...Object.keys(me.totals).filter((k) => !k.startsWith("ui_") && !DEEDS.includes(k))].filter(has);
  const mine = Object.keys(me.totals).filter((k) => k.startsWith("ui_")).sort();

  // ---- charts
  let metric = "population";
  let cumulative = true;
  let hover: number | null = null;
  const canvas = h("canvas", { class: "stats-chart" }) as HTMLCanvasElement;
  const legend = h("div", { class: "stats-legend" });
  const heading = h("h3", {});
  const toggle = h("div", { class: "stats-toggle" });
  const readout = h("div", { class: "stats-readout muted small" });
  const linesOf = (): Line[] => {
    const series = SERIES.includes(metric);
    const who = metric.startsWith("ui_") ? [me] : shown;
    return who.map((e) => {
      const raw = series ? (e.series[metric] ?? []) : totalsByYear(e, metric, years.length);
      return { label: name(e), color: color(e), values: !series && !cumulative ? perYear(raw) : raw };
    });
  };
  const draw = () => {
    const lines = linesOf();
    drawChart(canvas, years, lines, !cumulative, fmt(metric), hover, t("stats.empty"));
    const at = hover ?? years.length - 1;
    readout.textContent = years.length ? `${years[at]}: ${lines.map((l) => (lines.length > 1 ? `${l.label} ${fmt(metric)(l.values[at] ?? 0)}` : fmt(metric)(l.values[at] ?? 0))).join(" · ")}` : "";
  };
  const select = (k: string) => {
    metric = k;
    heading.textContent = label(k);
    list.querySelectorAll("button").forEach((b) => b.classList.toggle("active", b.dataset.k === k));
    const series = SERIES.includes(k);
    toggle.replaceChildren(
      ...(series
        ? []
        : [
            h("button", { class: `mini${cumulative ? " active" : ""}`, onclick: () => ((cumulative = true), select(metric)) }, t("stats.total")),
            h("button", { class: `mini${cumulative ? "" : " active"}`, onclick: () => ((cumulative = false), select(metric)) }, t("stats.per_year")),
          ]),
    );
    legend.replaceChildren(...(k.startsWith("ui_") ? [] : shown.length > 1 ? shown.map((e) => h("span", {}, h("i", { style: `background:${color(e)}` }), name(e))) : []));
    draw();
  };
  const group = (title: string, keys: string[]) =>
    keys.length ? [h("div", { class: "stats-group" }, title), ...keys.map((k) => h("button", { class: "stats-pick", "data-k": k, onclick: () => select(k) }, label(k)))] : [];
  const list = h("div", { class: "stats-list" }, ...group(t(data.all ? "stats.group.empires" : "stats.group.empire"), SERIES), ...group(t("stats.group.deeds"), deeds), ...group(t("stats.group.you"), mine));
  canvas.addEventListener("mousemove", (e) => {
    const r = canvas.getBoundingClientRect();
    const i = Math.round(((e.clientX - r.left - 56) / Math.max(1, r.width - 68)) * (years.length - 1));
    const next = Math.max(0, Math.min(years.length - 1, i));
    if (next !== hover) {
      hover = next;
      draw();
    }
  });
  canvas.addEventListener("mouseleave", () => ((hover = null), draw()));
  const charts = h("div", { class: "stats-charts" }, list, h("div", { class: "stats-plot" }, h("div", { class: "stats-plot-head" }, heading, toggle), canvas, legend, readout));

  // ---- totals
  const cell = (k: string, e: EmpireStats) => h("td", {}, fmt(k)((SERIES.includes(k) ? e.series[k]?.at(-1) : e.totals[k]) ?? 0));
  const totals = h(
    "table",
    { class: "list stats-table" },
    h("tr", {}, h("th", {}, ""), ...shown.map((e) => h("th", { style: `color:${color(e)}` }, name(e)))),
    ...[...SERIES, ...deeds].map((k) => h("tr", {}, h("td", {}, label(k)), ...shown.map((e) => cell(k, e)))),
    ...mine.map((k) => h("tr", { class: "muted" }, h("td", {}, label(k)), cell(k, me), ...shown.slice(1).map(() => h("td", {}, "")))),
  );

  // ---- ships built
  const ships = h(
    "div",
    { class: "stats-ships" },
    ...shown.map((e) => {
      const rows = Object.entries(e.ships_built).sort((a, b) => b[1] - a[1]);
      return h(
        "div",
        {},
        h("h3", { style: `color:${color(e)}` }, name(e)),
        rows.length ? h("table", { class: "list" }, ...rows.map(([d, n]) => h("tr", {}, h("td", {}, d), h("td", {}, num.format(n))))) : h("p", { class: "muted" }, t("stats.none")),
      );
    }),
  );

  // ---- against each rival (the player's record)
  const rivals = v.empires.filter((e) => e.id !== me.empire);
  const versus = h(
    "table",
    { class: "list stats-table" },
    h("tr", {}, h("th", {}, ""), ...VERSUS.map((k) => h("th", {}, t(`stats.versus.${k}`)))),
    ...rivals.map((r) => h("tr", {}, h("td", { style: `color:${theme.banner(r.banner)}` }, t(`race.${r.race}.plural`)), ...VERSUS.map((k) => h("td", {}, num.format(me.versus[k]?.[r.id] ?? 0))))),
  );

  const tabs: [string, HTMLElement][] = [
    ["charts", charts],
    ["totals", totals],
    ["ships", ships],
    ["rivals", versus],
  ];
  const pane = h("div", { class: "stats-pane" });
  const bar = h("div", { class: "stats-tabs" });
  const open = (i: number) => {
    pane.replaceChildren(tabs[i][1]);
    bar.querySelectorAll("button").forEach((b, j) => b.classList.toggle("active", i === j));
    if (i === 0) requestAnimationFrame(draw);
  };
  bar.append(...tabs.map(([k], i) => h("button", { class: "mini", onclick: () => open(i) }, t(`stats.tab.${k}`))));
  const resize = new ResizeObserver(() => pane.contains(canvas) && draw());
  resize.observe(canvas);
  await modal(
    data.all ? t("stats.title_all", { year: v.year }) : t("stats.title", { year: v.year }),
    [bar, pane],
    [t("btn.close")],
    "huge stats",
    () => {
      open(0);
      select(metric);
    },
  );
  resize.disconnect();
}
