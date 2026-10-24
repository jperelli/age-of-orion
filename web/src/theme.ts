// Themes, texts, images, sounds and mods. Every word, picture and sound the UI uses comes from
// here. Each theme is a folder of `themes/` (listed in `themes/index.json` at build time):
// `images/<name>.<svg|png|jpg|webp>`, `audio/<music|sfx>/<name>.<ogg|mp3|wav|mid>` and
// `text/<lang>.json`. Classic's folder has everything and its texts are the base of every theme.
// A picture comes from the mod if it has it, else from the player's MOO1 files (Classic), else
// from the theme's folder; with none of them the screen shows a placeholder.
import * as store from "./storage";

export type Text = string | string[];

export interface ThemeDef {
  id: string;
  /** Shown when the texts have no `theme.<id>` key; a string or one per language. */
  name: string | Record<string, string>;
  description: string | Record<string, string>;
  /** "world": terrain map with cities; "space": starfield with planets. */
  map: "world" | "space";
  /** Lay MOO1's pictures, sounds and texts (imported from the player's LBX files) over the theme. */
  use_lbx: boolean;
  banners: string[];
}

export interface Anim {
  frames: number;
  w: number;
  h: number;
  loop: number;
}

/** A theme folder as listed in `themes/index.json`: its files by name. */
export interface ThemeEntry {
  id: string;
  def: ThemeDef;
  images: Record<string, string>;
  anims: Record<string, Anim>;
  audio: Record<string, string>;
  langs: string[];
  css: boolean;
}

export interface ModData {
  id: string;
  name: string;
  description?: string;
  theme?: string;
  text: Record<string, Text>;
  rules?: unknown;
  images: Record<string, Blob>;
}

export interface Settings {
  theme: string;
  mod: string | null;
  lang: string;
  /** Volumes, 0..1. */
  music: number;
  sfx: number;
}

/** The theme whose texts are the base of every other and whose folder has every file. */
export const BASE_THEME = "classic";
/** Languages with a text file (`themes/classic/text/<lang>.json`). */
export const LANGS = ["en", "es"];

const DEFAULT_SETTINGS: Settings = {
  theme: "age",
  mod: null,
  lang: LANGS.find((l) => navigator.language?.toLowerCase().startsWith(l)) ?? "en",
  music: 0.5,
  sfx: 0.5,
};

export async function loadSettings(): Promise<Settings> {
  return { ...DEFAULT_SETTINGS, ...((await store.get<Settings>("kv", "settings")) ?? {}) };
}

export const saveSettings = (s: Settings) => store.put("kv", "settings", s);

async function fetchJson<T>(url: string): Promise<T | null> {
  try {
    const r = await fetch(url);
    return r.ok ? ((await r.json()) as T) : null;
  } catch {
    return null;
  }
}

let index: Promise<ThemeEntry[]> | null = null;

/** Every theme folder. */
export function themeList(): Promise<ThemeEntry[]> {
  index ??= fetchJson<{ themes: ThemeEntry[] }>("themes/index.json").then((i) => i?.themes ?? []);
  return index;
}

/** A theme's name or description in `lang`, from its texts or its theme.json. */
export function themeLabel(theme: Theme, entry: ThemeEntry, what: "name" | "description"): string {
  const key = what === "name" ? `theme.${entry.id}` : `theme.${entry.id}.description`;
  if (theme.has(key)) return theme.t(key);
  const v = entry.def[what];
  return typeof v === "string" ? v : (v[theme.lang] ?? v.en ?? Object.values(v)[0] ?? entry.id);
}

function loadImage(url: string): Promise<HTMLImageElement | null> {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => resolve(null);
    img.src = url;
  });
}

/** Draws frame `frame` of an imported animation sheet on a canvas. */
export function drawFrame(c: CanvasRenderingContext2D, a: Anim & { img: HTMLImageElement | null }, frame: number, x: number, y: number, w = a.w, h = a.h): void {
  if (a.img) c.drawImage(a.img, (frame % a.frames) * a.w, 0, a.w, a.h, x, y, w, h);
}

export class Theme {
  readonly urls = new Map<string, string>();
  readonly images = new Map<string, HTMLImageElement>();
  /** Sounds by name (`music/<tune>`, `sfx/<effect>`): URL of the theme's file. */
  readonly sounds = new Map<string, string>();
  /** The theme's own stylesheet (colours, fonts), if it has one. */
  css: string | null = null;

  constructor(
    readonly def: ThemeDef,
    readonly text: Record<string, Text>,
    readonly mod: ModData | null,
    readonly lang: string,
  ) {}

  /** Text for `key` with `{name}` placeholders replaced; the key itself when missing. */
  t(key: string, params: Record<string, string | number> = {}): string {
    const v = this.text[key];
    const s = typeof v === "string" ? v : Array.isArray(v) ? v.join(", ") : key;
    return s.replace(/\{(\w+)\}/g, (m, k: string) => (k in params ? String(params[k]) : m));
  }

  has(key: string): boolean {
    return key in this.text;
  }

  list(key: string): string[] {
    const v = this.text[key];
    return Array.isArray(v) ? v : [];
  }

  img(alias: string): HTMLImageElement | null {
    return this.images.get(alias) ?? null;
  }

  url(alias: string): string | null {
    return this.urls.get(alias) ?? null;
  }

  /** Whether some layer has the picture `alias` (it may still be loading). */
  hasPic(alias: string): boolean {
    return this.urls.has(alias);
  }

  banner(i: number): string {
    return this.def.banners[i % this.def.banners.length];
  }

  /** Aliases whose picture is pixel art (from LBX files), drawn without smoothing. */
  readonly pixel = new Set<string>();

  /** Animations (`anim/...`): horizontal sheets of `frames` frames of `w` x `h`, looping back
   * to frame `loop`. A theme's own come with a .json beside the picture. */
  readonly anims = new Map<string, Anim>();

  anim(alias: string): (Anim & { url: string; img: HTMLImageElement | null }) | null {
    const a = this.anims.get(alias);
    const url = this.urls.get(alias);
    return a && url ? { ...a, url, img: this.images.get(alias) ?? null } : null;
  }

  /** A picture known only by its URL, decoded by the browser when shown (big animations). */
  addImageLazy(alias: string, blob: Blob): void {
    this.urls.set(alias, URL.createObjectURL(blob));
    this.pixel.add(alias);
  }

  async addImage(alias: string, blob: Blob, pixel = false): Promise<void> {
    const url = URL.createObjectURL(blob);
    const img = await loadImage(url);
    if (img) {
      this.urls.set(alias, url);
      this.images.set(alias, img);
      if (pixel) this.pixel.add(alias);
      else this.pixel.delete(alias);
    }
  }

  /** A picture of the theme's folder (not a blob: no URL to revoke). Its URL is usable at
   * once; canvas drawing gets it when it has loaded, unless an imported or mod picture took
   * its place meanwhile. */
  addImageUrl(alias: string, url: string): void {
    const src = new URL(url, document.baseURI).href;
    this.urls.set(alias, src);
    this.pixel.delete(alias);
    void loadImage(src).then((img) => {
      if (img && this.urls.get(alias) === src) this.images.set(alias, img);
    });
  }

  /** Fonts converted from FONTS.LBX ("MOO1 0" .. "MOO1 5"). */
  fonts = new Set<string>();

  async addFont(n: string, blob: Blob): Promise<void> {
    try {
      const face = new FontFace(`MOO1 ${n}`, await blob.arrayBuffer());
      await face.load();
      document.fonts.add(face);
      this.fonts.add(n);
    } catch {
      // A font the browser rejects: the theme's own font stays.
    }
  }

  dispose(): void {
    for (const u of this.urls.values()) if (u.startsWith("blob:")) URL.revokeObjectURL(u);
  }
}

export async function loadTheme(settings: Settings): Promise<Theme> {
  const mod = settings.mod ? ((await store.get<ModData>("mods", settings.mod)) ?? null) : null;
  const themes = await themeList();
  const wanted = mod?.theme && themes.some((e) => e.id === mod.theme) ? mod.theme : settings.theme;
  const entry = themes.find((e) => e.id === wanted) ?? themes.find((e) => e.id === DEFAULT_SETTINGS.theme) ?? themes[0];
  const base = themes.find((e) => e.id === BASE_THEME);
  const def = entry.def;
  const text: Record<string, Text> = {};
  const layers = [...(base && base !== entry ? [base] : []), entry];
  for (const e of layers) {
    for (const lang of ["en", ...(settings.lang !== "en" ? [settings.lang] : [])]) {
      if (e.langs.includes(lang)) Object.assign(text, (await fetchJson<Record<string, Text>>(`themes/${e.id}/text/${lang}.json`)) ?? {});
    }
  }
  if (mod) Object.assign(text, mod.text);
  const theme = new Theme(def, text, mod, settings.lang);
  theme.css = entry.css ? `themes/${entry.id}/theme.css` : null;
  for (const [alias, a] of Object.entries(entry.anims)) theme.anims.set(alias, a);
  for (const [alias, path] of Object.entries(entry.images)) theme.addImageUrl(alias, `themes/${entry.id}/${path}`);
  for (const [alias, path] of Object.entries(entry.audio)) theme.sounds.set(alias, new URL(`themes/${entry.id}/${path}`, document.baseURI).href);

  if (def.use_lbx) {
    const manifest = await store.get<{ aliases: Record<string, string>; frames?: Record<string, number[]> }>("assets", "manifest.json");
    for (const [alias, [frames, w, h, loop]] of Object.entries(manifest?.frames ?? {})) theme.anims.set(alias, { frames, w, h, loop });
    for (const [alias, path] of Object.entries(manifest?.aliases ?? {})) {
      if (alias.startsWith("music/") || alias.startsWith("sfx/")) continue;
      const blob = await store.get<Blob>("assets", path);
      if (!blob) continue;
      if (alias.startsWith("font/")) await theme.addFont(alias.slice(5), blob);
      // Small animations are drawn on canvases (stars, ships); the big ones play in the page.
      else if (alias.startsWith("anim/") && !/^anim\/(star|ship)\//.test(alias)) theme.addImageLazy(alias, blob);
      else await theme.addImage(alias, blob, true);
    }
    // EVENTMSG.LBX: entry (type - 1) * 7 + stage is the news of 1oom `game_news_get_msg`.
    // MOO1's own texts are English: other languages keep their translations.
    const english = settings.lang === "en";
    const events = english ? await store.get<Blob>("assets", "text/lbx/eventmsg.json") : null;
    if (events) {
      const all = JSON.parse(await events.text()) as string[];
      const raw: [string, string][] = [["â", "{s}"], ["à", "{other_race}"], ["å", "{field}"], ["ç", "{tech}"], ["ê", "{personality}"], ["ë", "{objective}"], ["î", "{leader}"], ["ì", "{s1}"], ["Ä", "{an}"]];
      all.forEach((msg, i) => {
        const kind = NEWS_KINDS[Math.floor(i / 7)];
        const key = `news.${kind}.${i % 7}`;
        if (!msg || !kind || mod?.text[key]) return;
        text[key] = raw.reduce((m, [c, tok]) => m.split(c).join(tok), msg).replace(/\s+/g, " ").trim();
      });
    }
    // RESEARCH.LBX: tech descriptions, field * 50 + tech - 1.
    const research = english ? await store.get<Blob>("assets", "text/lbx/research.json") : null;
    if (research) {
      const all = JSON.parse(await research.text()) as string[];
      all.forEach((d, i) => {
        const key = `techinfo.${Math.floor(i / 50)}.${(i % 50) + 1}`;
        if (d && !mod?.text[key]) text[key] = d.replace(/\s+/g, " ").trim();
      });
    }
    const names = await store.get<Blob>("assets", "text/lbx/names.json");
    if (names && !mod?.text["names.leaders"]) {
      const all = JSON.parse(await names.text()) as string[];
      text["names.leaders"] = Array.from({ length: 10 }, (_, r) => all[r * 6] ?? "");
    }
  }
  for (const [alias, blob] of Object.entries(mod?.images ?? {})) await theme.addImage(alias, blob);
  return theme;
}

/** News kinds in the order of MOO1's EVENTMSG.LBX (1oom `GAME_NEWS_*` minus one). */
const NEWS_KINDS = ["plague", "quake", "nova", "accident", "assassin", "virus", "comet", "pirates", "derelict", "rebellion", "crystal", "amoeba", "enviro", "rich", "support", "poor", "orion", "coup", "stars", "stats", "genocide", "guardian"];

/** Reads a mod folder chosen with `<input webkitdirectory>`. */
export async function readModFolder(files: File[]): Promise<ModData> {
  const rel = (f: File) => (f.webkitRelativePath || f.name).split("/").slice(1).join("/") || f.name;
  const byPath = new Map(files.map((f) => [rel(f), f]));
  const json = async <T>(p: string): Promise<T | undefined> => {
    const f = byPath.get(p);
    return f ? (JSON.parse(await f.text()) as T) : undefined;
  };
  const meta = (await json<{ id?: string; name?: string; description?: string; theme?: string }>("mod.json")) ?? {};
  const text: Record<string, Text> = {};
  for (const [p, f] of byPath) {
    if (/^text\/[\w-]+\.json$/.test(p)) Object.assign(text, JSON.parse(await f.text()));
  }
  const images: Record<string, Blob> = {};
  for (const [p, f] of byPath) {
    const m = /^images\/(.+)\.png$/i.exec(p);
    if (m) images[m[1]] = f;
  }
  const name = meta.name ?? files[0]?.webkitRelativePath.split("/")[0] ?? "mod";
  return {
    id: meta.id ?? name.toLowerCase().replace(/[^a-z0-9]+/g, "-"),
    name,
    description: meta.description,
    theme: meta.theme,
    text,
    rules: await json("rules.json"),
    images,
  };
}
