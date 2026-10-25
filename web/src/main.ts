import "./style.css";
import { TUNE, audio } from "./audio";
import { Game, engineVersion, importLbxAsync, loadEngine } from "./engine";
import { AUTOSAVE, GameScreen } from "./game";
import * as store from "./storage";
import { LANGS, loadSettings, loadTheme, readModFolder, saveSettings, themeLabel, themeList, type ModData, type Settings, type Theme } from "./theme";
import { RACES, type Difficulty, type GalaxySize, type NewGameOptions, type Race } from "./types";
import { moo1Dialog } from "./about";
import { slider } from "./sliders";
import { picUrl } from "./pictures";
import { pictureSelect } from "./widgets";
import { clear, h, modal, pickFiles, setCloseLabel, toast } from "./ui";

const root = document.getElementById("app")!;
let settings: Settings;
let theme: Theme;
let screen: GameScreen | null = null;

function t(key: string, params?: Record<string, string | number>): string {
  return theme.t(key, params);
}

async function applyTheme(): Promise<void> {
  theme?.dispose();
  theme = await loadTheme(settings);
  document.documentElement.dataset.theme = theme.def.id;
  // The theme's own colours and fonts (`themes/<id>/theme.css`).
  document.getElementById("theme-css")?.remove();
  if (theme.css) document.head.append(h("link", { id: "theme-css", rel: "stylesheet", href: theme.css }));
  document.documentElement.lang = settings.lang;
  // MOO1's own fonts, when imported (FONTS.LBX).
  if (theme.fonts.has("3") && theme.fonts.has("4")) document.documentElement.dataset.moofonts = "1";
  else delete document.documentElement.dataset.moofonts;
  audio.setTheme(theme);
  document.title = t("app.title");
  setCloseLabel(t("btn.close"));
}

// MOO1 clicks every button (1oom `ui_sound_play_sfx_24`): the theme's `sfx/button`.
document.addEventListener("click", (e) => {
  if ((e.target as Element | null)?.closest?.("button")) audio.sfx("button");
});

function show(el: HTMLElement): void {
  screen?.destroy();
  screen = null;
  clear(root).append(el);
}

function playGame(game: Game): void {
  audio.stopMusic();
  const s = new GameScreen(game, theme, () => void menu(), (save) => loadSave(save));
  show(s.el);
  screen = s;
}

function loadSave(save: string): void {
  try {
    playGame(Game.load(save));
  } catch (e) {
    toast(t("error.load", { msg: String(e) }));
  }
}

async function menu(): Promise<void> {
  void audio.play(TUNE.menu);
  const autosave = await store.get<string>("kv", AUTOSAVE);
  const assets = await store.get<{ archives: Record<string, unknown> }>("assets", "manifest.json");
  const mods = (await store.keys("mods")).map(String);
  const themes = await themeList();
  const current = themes.find((e) => e.id === theme.def.id);
  const themeSelect = h(
    "select",
    {
      onchange: async (e: Event) => {
        settings.theme = (e.target as HTMLSelectElement).value;
        await saveSettings(settings);
        await applyTheme();
        void menu();
      },
    },
    ...themes.map((e) => h("option", { value: e.id, selected: e.id === theme.def.id }, themeLabel(theme, e, "name"))),
  );
  themeSelect.disabled = !!theme.mod?.theme;
  const langSelect = h(
    "select",
    {
      onchange: async (e: Event) => {
        settings.lang = (e.target as HTMLSelectElement).value;
        await saveSettings(settings);
        await applyTheme();
        void menu();
      },
    },
    ...LANGS.map((l) => h("option", { value: l, selected: l === settings.lang }, t(`lang.${l}`))),
  );
  const modSelect = h(
    "select",
    {
      onchange: async (e: Event) => {
        const v = (e.target as HTMLSelectElement).value;
        settings.mod = v || null;
        await saveSettings(settings);
        await applyTheme();
        void menu();
      },
    },
    h("option", { value: "", selected: !settings.mod }, t("menu.no_mod")),
    ...mods.map((id) => h("option", { value: id, selected: id === settings.mod }, id)),
  );
  const about = (e: Event) => {
    e.preventDefault();
    void moo1Dialog(theme, settings.lang);
  };
  show(
    h(
      "div",
      { class: "menu-screen" },
      h(
        "div",
        { class: "menu-card" },
        h("h1", {}, t("app.title")),
        h(
          "p",
          { class: "subtitle" },
          theme.def.map === "space" ? h("a", { href: "#", class: "about-link", title: t("app.moo1_link"), onclick: about }, t("app.subtitle")) : t("app.subtitle"),
        ),
        h(
          "div",
          { class: "menu-buttons" },
          autosave ? h("button", { class: "primary", onclick: () => loadSave(autosave) }, t("menu.continue")) : null,
          h("button", { class: autosave ? "" : "primary", onclick: () => newGame() }, t("menu.new")),
          h(
            "button",
            {
              onclick: async () => {
                const [f] = await pickFiles({ accept: ".json,application/json" });
                if (f) loadSave(await f.text());
              },
            },
            t("menu.load"),
          ),
          h("button", { onclick: () => importer() }, t("menu.import")),
          h("button", { onclick: () => modsScreen() }, t("menu.mods")),
        ),
        h(
          "div",
          { class: "menu-options" },
          h("label", {}, t("menu.theme"), themeSelect),
          h("label", {}, t("menu.mod"), modSelect),
          h("label", {}, t("menu.language"), langSelect),
          volume("menu.music", "music"),
          volume("menu.sfx", "sfx"),
        ),
        h("p", { class: "muted small" }, current ? themeLabel(theme, current, "description") : ""),
        h("p", { class: "muted small" }, assets ? t("menu.assets_imported", { n: Object.keys(assets.archives).length }) : t("menu.assets_none")),
        h(
          "p",
          { class: "footer small" },
          t("app.footer", { version: engineVersion() }),
          " ",
          h("a", { href: "#", class: "about-link", onclick: about }, t("app.moo1_link")),
          " · ",
          h("a", { href: "https://github.com/jperelli/age-of-orion", target: "_blank", rel: "noopener" }, "GitHub"),
        ),
      ),
    ),
  );
}

function newGame(): void {
  // A <label> would pass its clicks to the picture selects' first button.
  const field = (label: string, el: HTMLElement) => h(el.matches("input, select") ? "label" : "div", { class: "field" }, h("span", {}, label), el);
  const select = <T extends string>(values: T[], prefix: string, def: T) =>
    h("select", {}, ...values.map((v) => h("option", { value: v, selected: v === def }, t(`${prefix}${v}`))));
  const size = select<GalaxySize>(["small", "medium", "large", "huge"], "newgame.size.", "small");
  const diff = select<Difficulty>(["simple", "easy", "average", "hard", "impossible"], "newgame.difficulty.", "easy");
  const players = h("select", {}, ...[2, 3, 4, 5, 6].map((n) => h("option", { value: n, selected: n === 4 }, t("newgame.opponents_n", { n: n - 1 }))));
  const ai = h("select", {}, ...[0, 1].map((n) => h("option", { value: n }, t(`newgame.ai.${n}`))));
  const emblem = (i: number) => ({ img: picUrl(theme, `arms/${i}`), pixel: theme.pixel.has(`arms/${i}`) });
  const race = pictureSelect<Race | "">(
    [
      { value: "", label: t("newgame.random") },
      ...RACES.map((r, i) => ({ value: r, label: t(`race.${r}.plural`), sub: t(`race.${r}.trait`), ...emblem(i) })),
    ],
    "",
    undefined,
    "race-select",
  );
  const banner = pictureSelect<number | null>(
    [{ value: null, label: t("newgame.random") }, ...theme.def.banners.map((color, i) => ({ value: i, label: t(`banner.${i}`), swatch: color }))],
    null,
  );
  const leader = h("input", { type: "text", maxlength: 24, placeholder: t("newgame.leader_placeholder") });
  const seed = h("input", { type: "number", value: Math.floor(Math.random() * 1e9) });
  const start = () => {
    const r = race.value || null;
    const opts: NewGameOptions = {
      seed: Number(seed.value) || 1,
      galaxy_size: size.value as GalaxySize,
      difficulty: diff.value as Difficulty,
      players: Number(players.value),
      ai_id: Number(ai.value),
      race: r,
      banner: banner.value,
      star_names: theme.list("names.sites"),
      home_names: theme.list("names.homes"),
      leader_names: RACES.map((r, i) => theme.list("names.leaders")[i] || t(`race.${r}.leader`)),
      // MOO1's six names per race, when imported: every ruler gets one of them.
      leader_name_lists: theme.list("names.leaders.all").length >= 60 ? RACES.map((_, r) => theme.list("names.leaders.all").slice(r * 6, r * 6 + 6)) : [],
      leader: leader.value.trim() || null,
      orion_name: theme.has("names.orion") ? t("names.orion") : null,
      start_design_names: ["scout", "fighter", "destroyer", "bomber", "colony_ship"].map((k) => t(`design.${k}`)),
      ship_names: RACES.map((r) => theme.list(`names.ships.${r}`)),
      rules: theme.mod?.rules ?? undefined,
    };
    try {
      playGame(Game.create(opts));
    } catch (e) {
      toast(t("error.new_game", { msg: String(e) }));
    }
  };
  show(
    h(
      "div",
      { class: "menu-screen" },
      h(
        "div",
        { class: "menu-card wide" },
        h("h1", {}, t("newgame.title")),
        h(
          "div",
          { class: "form" },
          field(t("newgame.race"), race.el),
          field(t("newgame.banner"), banner.el),
          field(t("newgame.leader"), leader),
          field(t("newgame.size"), size),
          field(t("newgame.difficulty"), diff),
          field(t("newgame.opponents"), players),
          field(t("newgame.ai"), ai),
          field(t("newgame.seed"), seed),
        ),
        h("div", { class: "menu-buttons row" }, h("button", { onclick: () => menu() }, t("btn.back")), h("button", { class: "primary", onclick: start }, t("newgame.start"))),
      ),
    ),
  );
}

/** A volume slider stored in the settings. */
function volume(label: string, key: "music" | "sfx"): HTMLElement {
  return slider({
    label: t(label),
    value: Math.round(settings[key] * 100),
    plusStep: 10,
    format: (x) => `${x}%`,
    onCommit: async (x) => {
      settings[key] = x / 100;
      audio.setPrefs({ music: settings.music, sfx: settings.sfx });
      if (key === "sfx") audio.sfx("explosion");
      await saveSettings(settings);
    },
  });
}

async function importer(): Promise<void> {
  const status = h("div", { class: "import-status" });
  const gallery = h("div", { class: "gallery" });
  const showGallery = async () => {
    clear(gallery);
    const m = await store.get<{ aliases: Record<string, string>; warnings: string[]; archives: Record<string, unknown> }>("assets", "manifest.json");
    if (!m) return;
    status.textContent = t("import.done", { n: Object.keys(m.archives).length });
    const tunes = Object.keys(m.aliases).filter((a) => a.startsWith("music/")).length;
    if (tunes) gallery.append(h("p", { class: "small" }, t("import.music", { n: tunes })));
    for (const [alias, path] of Object.entries(m.aliases)) {
      if (alias.startsWith("music/") || alias.startsWith("anim/")) continue;
      const blob = await store.get<Blob>("assets", path);
      if (blob) gallery.append(h("img", { class: "pixel", src: URL.createObjectURL(blob), title: alias }));
    }
    for (const w of m.warnings) gallery.append(h("p", { class: "warn small" }, w));
  };
  const run = async (files: File[]) => {
    const lbx = files.filter((f) => /\.lbx$/i.test(f.name));
    if (!lbx.length) return toast(t("import.none"));
    status.textContent = t("import.working", { n: lbx.length });
    await new Promise((r) => setTimeout(r, 30));
    const input = await Promise.all(lbx.map(async (f) => ({ name: f.name, bytes: new Uint8Array(await f.arrayBuffer()) })));
    const result = await importLbxAsync(input);
    await store.clear("assets");
    await store.putMany(
      "assets",
      result.files.map((f) => [
        f.path,
        f.path === "manifest.json"
          ? JSON.parse(new TextDecoder().decode(f.bytes))
          : new Blob([f.bytes as BlobPart], { type: f.path.endsWith(".png") ? "image/png" : f.path.endsWith(".mid") ? "audio/midi" : "application/json" }),
      ]),
    );
    audio.reload();
    await applyTheme();
    await showGallery();
  };
  show(
    h(
      "div",
      { class: "menu-screen" },
      h(
        "div",
        { class: "menu-card wide" },
        h("h1", {}, t("import.title")),
        ...theme.list("import.text").map((l) => h("p", {}, l)),
        h(
          "div",
          { class: "menu-buttons row" },
          h("button", { class: "primary", onclick: async () => run(await pickFiles({ accept: ".lbx,.LBX", multiple: true })) }, t("import.pick_files")),
          h("button", { onclick: async () => run(await pickFiles({ directory: true })) }, t("import.pick_folder")),
          h(
            "button",
            {
              onclick: async () => {
                await store.clear("assets");
                await applyTheme();
                clear(gallery);
                status.textContent = t("import.removed");
              },
            },
            t("import.remove"),
          ),
          h("button", { onclick: () => menu() }, t("btn.back")),
        ),
        status,
        gallery,
      ),
    ),
  );
  await showGallery();
}

async function modsScreen(): Promise<void> {
  const list = h("ul", { class: "mods" });
  const refresh = async () => {
    clear(list);
    for (const id of (await store.keys("mods")).map(String)) {
      const m = await store.get<ModData>("mods", id);
      list.append(
        h(
          "li",
          {},
          h("b", {}, m?.name ?? id),
          " ",
          h("span", { class: "muted" }, m?.description ?? ""),
          " ",
          h(
            "button",
            {
              onclick: async () => {
                await store.del("mods", id);
                if (settings.mod === id) {
                  settings.mod = null;
                  await saveSettings(settings);
                  await applyTheme();
                }
                void refresh();
              },
            },
            t("mods.remove"),
          ),
        ),
      );
    }
  };
  show(
    h(
      "div",
      { class: "menu-screen" },
      h(
        "div",
        { class: "menu-card wide" },
        h("h1", {}, t("mods.title")),
        ...theme.list("mods.text").map((l) => h("p", {}, l)),
        list,
        h(
          "div",
          { class: "menu-buttons row" },
          h(
            "button",
            {
              class: "primary",
              onclick: async () => {
                const files = await pickFiles({ directory: true });
                if (!files.length) return;
                const mod = await readModFolder(files);
                await store.put("mods", mod.id, mod);
                settings.mod = mod.id;
                await saveSettings(settings);
                await applyTheme();
                toast(t("mods.installed", { name: mod.name }));
                void modsScreen();
              },
            },
            t("mods.add"),
          ),
          h("button", { onclick: () => menu() }, t("btn.back")),
        ),
      ),
    ),
  );
  await refresh();
}

async function main(): Promise<void> {
  root.textContent = "Loading...";
  try {
    await loadEngine();
    settings = await loadSettings();
    audio.setPrefs({ music: settings.music, sfx: settings.sfx });
    await applyTheme();
    await menu();
  } catch (e) {
    root.textContent = `Failed to start: ${String(e)}`;
    console.error(e);
    void modal("Error", String(e));
  }
}

void main();
