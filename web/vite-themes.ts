// The themes folder (`../themes/<id>/`) as part of the site: served as is in development,
// copied into the build, and listed in `themes/index.json` (a static site cannot list a folder).
// A file's path inside `images/` or `audio/` without its extension is the name the game asks
// for (`images/ship/3.svg` is the picture `ship/3`); when several formats exist the first of
// IMAGE_EXT / AUDIO_EXT wins.
import { cpSync, existsSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { dirname, extname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import type { Plugin } from "vite";

const IMAGE_EXT = [".svg", ".png", ".jpg", ".jpeg", ".webp"];
const AUDIO_EXT = [".ogg", ".mp3", ".wav", ".mid"];
const TYPES: Record<string, string> = {
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".webp": "image/webp",
  ".css": "text/css",
  ".ogg": "audio/ogg",
  ".mp3": "audio/mpeg",
  ".wav": "audio/wav",
  ".mid": "audio/midi",
  ".md": "text/markdown",
};

function files(dir: string): string[] {
  if (!existsSync(dir)) return [];
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    return statSync(p).isDirectory() ? files(p) : [p];
  });
}

/** Name -> file (relative to the theme folder), the first format of `exts` winning. */
function pick(base: string, sub: string, exts: string[]): Record<string, string> {
  const out: Record<string, string> = {};
  const rank: Record<string, number> = {};
  for (const f of files(join(base, sub))) {
    const ext = extname(f).toLowerCase();
    const r = exts.indexOf(ext);
    if (r < 0) continue;
    const name = relative(join(base, sub), f).slice(0, -ext.length).split(sep).join("/");
    if (name in rank && rank[name] <= r) continue;
    rank[name] = r;
    out[name] = relative(base, f).split(sep).join("/");
  }
  return out;
}

export function themesIndex(root: string): unknown {
  const themes = readdirSync(root)
    .filter((id) => existsSync(join(root, id, "theme.json")))
    .sort()
    .map((id) => {
      const base = join(root, id);
      const def = JSON.parse(readFileSync(join(base, "theme.json"), "utf8"));
      // Animations: a strip of frames beside a .json with its frames, w, h and loop frame.
      const anims: Record<string, unknown> = {};
      for (const f of files(join(base, "images"))) {
        if (extname(f) === ".json") anims[relative(join(base, "images"), f).slice(0, -5).split(sep).join("/")] = JSON.parse(readFileSync(f, "utf8"));
      }
      return {
        id,
        def: { ...def, id },
        images: pick(base, "images", IMAGE_EXT),
        anims,
        audio: pick(base, "audio", AUDIO_EXT),
        langs: files(join(base, "text"))
          .filter((f) => f.endsWith(".json"))
          .map((f) => relative(join(base, "text"), f).slice(0, -5))
          .sort(),
        css: existsSync(join(base, "theme.css")),
      };
    });
  return { themes };
}

export function themesPlugin(dir = "../themes"): Plugin {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), dir);
  let outDir = "";
  return {
    name: "age-of-orion-themes",
    configResolved(c) {
      outDir = resolve(c.root, c.build.outDir);
    },
    configureServer(server) {
      server.middlewares.use("/themes", (req, res, next) => {
        const path = decodeURIComponent((req.url ?? "/").split("?")[0]);
        if (path === "/index.json") {
          res.setHeader("Content-Type", TYPES[".json"]);
          res.end(JSON.stringify(themesIndex(root)));
          return;
        }
        const file = resolve(root, `.${path}`);
        if (!file.startsWith(root + sep) || !existsSync(file) || statSync(file).isDirectory()) return next();
        res.setHeader("Content-Type", TYPES[extname(file).toLowerCase()] ?? "application/octet-stream");
        res.end(readFileSync(file));
      });
    },
    closeBundle() {
      const out = join(outDir, "themes");
      cpSync(root, out, { recursive: true });
      writeFileSync(join(out, "index.json"), JSON.stringify(themesIndex(root)));
    },
  };
}
