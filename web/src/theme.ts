// Themes, texts, images and mods. Every word and picture the UI shows comes from here:
// base texts < theme texts < mod texts, and mod images < imported LBX images < procedural art.
import * as store from "./storage";

export type Text = string | string[];

export interface ThemeDef {
  id: string;
  name: string;
  description: string;
  /** "world": terrain map with cities; "space": starfield with planets. */
  map: "world" | "space";
  /** Use images converted from the player's LBX files when available. */
  use_lbx: boolean;
  banners: string[];
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

export const THEMES = ["age", "classic"];
/** Languages with a text file (`text/<lang>.json`). */
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

function loadImage(url: string): Promise<HTMLImageElement | null> {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => resolve(null);
    img.src = url;
  });
}

export class Theme {
  readonly urls = new Map<string, string>();
  readonly images = new Map<string, HTMLImageElement>();

  constructor(
    readonly def: ThemeDef,
    readonly text: Record<string, Text>,
    readonly mod: ModData | null,
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

  banner(i: number): string {
    return this.def.banners[i % this.def.banners.length];
  }

  async addImage(alias: string, blob: Blob): Promise<void> {
    const url = URL.createObjectURL(blob);
    const img = await loadImage(url);
    if (img) {
      this.urls.set(alias, url);
      this.images.set(alias, img);
    }
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
    for (const u of this.urls.values()) URL.revokeObjectURL(u);
  }
}

export async function loadTheme(settings: Settings): Promise<Theme> {
  const mod = settings.mod ? ((await store.get<ModData>("mods", settings.mod)) ?? null) : null;
  const id = mod?.theme && THEMES.includes(mod.theme) ? mod.theme : settings.theme;
  const def = (await fetchJson<ThemeDef>(`themes/${id}/theme.json`)) ?? (await fetchJson<ThemeDef>("themes/age/theme.json"))!;
  const text: Record<string, Text> = {};
  for (const url of [
    "text/en.json",
    `text/${settings.lang}.json`,
    `themes/${def.id}/text/en.json`,
    `themes/${def.id}/text/${settings.lang}.json`,
  ]) {
    Object.assign(text, (await fetchJson<Record<string, Text>>(url)) ?? {});
  }
  if (mod) Object.assign(text, mod.text);
  const theme = new Theme(def, text, mod);

  if (def.use_lbx) {
    const manifest = await store.get<{ aliases: Record<string, string> }>("assets", "manifest.json");
    for (const [alias, path] of Object.entries(manifest?.aliases ?? {})) {
      if (alias.startsWith("music/") || alias.startsWith("sfx/")) continue;
      const blob = await store.get<Blob>("assets", path);
      if (!blob) continue;
      if (alias.startsWith("font/")) await theme.addFont(alias.slice(5), blob);
      else await theme.addImage(alias, blob);
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
