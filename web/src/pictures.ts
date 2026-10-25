// Pictures by name for the screens: the theme's file (or MOO1's imported one), and a placeholder
// in its place when the theme does not have it, on the page and on canvases.
import type { Theme } from "./theme";
import { h } from "./ui";

/** A missing picture: a dashed frame with a question mark (scales to any box). */
export const MISSING_URL = `data:image/svg+xml,${encodeURIComponent(
  '<svg xmlns="http://www.w3.org/2000/svg" width="64" height="48" viewBox="0 0 64 48"><rect x="2" y="2" width="60" height="44" rx="4" fill="rgba(128,128,128,0.12)" stroke="#9a9a9a" stroke-width="2" stroke-dasharray="6 4"/><text x="32" y="33" font-family="sans-serif" font-size="22" font-weight="bold" text-anchor="middle" fill="#9a9a9a">?</text></svg>',
)}`;

/** The URL of picture `alias`, or the placeholder's. */
export function picUrl(theme: Theme, alias: string): string {
  return theme.url(alias) ?? MISSING_URL;
}

/** An image of picture `alias` (MOO1's pixel art drawn crisp); the placeholder, naming it, when missing. */
export function picImg(theme: Theme, alias: string, cls = ""): HTMLImageElement {
  const url = theme.url(alias);
  const c = [cls, url ? (theme.pixel.has(alias) ? "pixel" : "") : "art-missing"].filter(Boolean).join(" ");
  return h("img", { class: c, src: url ?? MISSING_URL, alt: "", title: url ? "" : alias }) as HTMLImageElement;
}

/** A dashed frame with a question mark filling a box of a canvas (a missing picture). */
export function drawPlaceholder(c: CanvasRenderingContext2D, x: number, y: number, w: number, hgt: number): void {
  const s = Math.min(w, hgt);
  c.save();
  c.fillStyle = "rgba(128,128,128,0.15)";
  c.fillRect(x, y, w, hgt);
  c.strokeStyle = "rgba(170,170,170,0.85)";
  c.lineWidth = Math.max(s / 24, 0.2);
  c.setLineDash([s / 8, s / 12]);
  c.strokeRect(x + c.lineWidth / 2, y + c.lineWidth / 2, w - c.lineWidth, hgt - c.lineWidth);
  c.setLineDash([]);
  c.fillStyle = "rgba(170,170,170,0.95)";
  c.font = `bold ${s * 0.6}px sans-serif`;
  c.textAlign = "center";
  c.textBaseline = "middle";
  c.fillText("?", x + w / 2, y + hgt / 2);
  c.restore();
}

/** Draws picture `alias` into a box of a canvas (smoothed unless it is MOO1's pixel art), or the placeholder. */
export function drawPic(c: CanvasRenderingContext2D, theme: Theme, alias: string, x: number, y: number, w: number, hgt: number): void {
  const img = theme.img(alias);
  if (!img) {
    // Still loading: nothing yet; not in the theme: the placeholder.
    if (!theme.hasPic(alias)) drawPlaceholder(c, x, y, w, hgt);
    return;
  }
  const smooth = c.imageSmoothingEnabled;
  c.imageSmoothingEnabled = !theme.pixel.has(alias);
  c.imageSmoothingQuality = "high";
  c.drawImage(img, x, y, w, hgt);
  c.imageSmoothingEnabled = smooth;
}

/** The picture of a ship design (`look` 0..23: hull * 6 + variant): a troop (`unit/<look>`) on a
 * world map, MOO1's ship in its empire's colour (`ship/<banner * 24 + look>`) in space. */
export function designAlias(theme: Theme, look: number, banner: number): string {
  return theme.def.map === "world" ? `unit/${look % 24}` : `ship/${banner * 24 + (look % 24)}`;
}

export function designPicture(theme: Theme, look: number, banner: number): string {
  return picUrl(theme, designAlias(theme, look, banner));
}

/** An interface icon (`ui/<name>`: gold, research, reserve). */
export function uiIcon(theme: Theme, name: string): string {
  return picUrl(theme, `ui/${name}`);
}

/**
 * A row of icons repeated to show an amount (the theme's `icon/<what>`): one icon per `per`
 * (rounded up), at most `max` icons, and faded ones for the room left up to `cap`; null with
 * nothing to show.
 */
export function iconRow(theme: Theme, what: string, amount: number, per: number, cap = 0, max = 40): HTMLElement | null {
  const url = picUrl(theme, `icon/${what}`);
  const n = Math.min(max, Math.ceil(Math.max(0, amount) / per));
  const room = Math.max(0, Math.min(max, Math.ceil(cap / per)) - n);
  if (n + room === 0) return null;
  const icon = (cls: string) => h("img", { src: url, alt: "", class: cls });
  return h("div", { class: `icon-row icon-${what}` }, ...Array.from({ length: n }, () => icon("")), ...Array.from({ length: room }, () => icon("room")));
}

/**
 * A page element playing an animation (MOO1's frames at `scale`, or filling its box when
 * `scale` is 0), from frame 0 to the last and then again from its loop frame, `fps` frames a
 * second; `onFrame` hears each frame shown (sounds on given frames).
 */
export function animSprite(theme: Theme, alias: string, scale = 2, fps = 8, onFrame?: (frame: number) => void): HTMLElement | null {
  const a = theme.anim(alias);
  if (!a) return null;
  const size = scale ? `width:${a.w * scale}px;height:${a.h * scale}px;background-size:${a.w * a.frames * scale}px ${a.h * scale}px` : `background-size:${a.frames * 100}% 100%`;
  const el = h("div", { class: `anim-sprite${theme.pixel.has(alias) ? " pixel" : ""}`, style: `${size};background-image:url("${a.url}")` });
  let f = 0;
  let shown = false;
  const started = performance.now();
  const tick = () => {
    if (!el.isConnected) {
      // Stops once removed, or if it never got shown.
      if (shown || performance.now() - started > 10000) clearInterval(timer);
      return;
    }
    shown = true;
    el.style.backgroundPosition = scale ? `${-f * a.w * scale}px 0` : `${(f * 100) / Math.max(1, a.frames - 1)}% 0`;
    onFrame?.(f);
    f = f + 1 >= a.frames ? a.loop : f + 1;
  };
  const timer = setInterval(tick, 1000 / fps);
  tick();
  return el;
}

/**
 * A picture whose frame the caller sets (`anim/<name>` when the theme animates it, else the
 * still `<name>`, else the placeholder), `w` x `h` MOO1 pixels at `scale`.
 */
export function sprite(theme: Theme, name: string, w: number, hgt: number, scale: number): { el: HTMLElement; frames: number; loop: number; set: (frame: number) => void } {
  const a = theme.anim(`anim/${name}`);
  if (a) {
    const el = h("div", {
      class: `anim-sprite${theme.pixel.has(`anim/${name}`) ? " pixel" : ""}`,
      style: `width:${a.w * scale}px;height:${a.h * scale}px;background-size:${a.w * a.frames * scale}px ${a.h * scale}px;background-image:url("${a.url}")`,
    });
    return { el, frames: a.frames, loop: a.loop, set: (f) => (el.style.backgroundPosition = `${-(f % a.frames) * a.w * scale}px 0`) };
  }
  const el = theme.hasPic(name) ? picImg(theme, name) : placeholder(theme, name, w * scale, hgt * scale);
  el.style.width = `${w * scale}px`;
  el.style.height = `${hgt * scale}px`;
  return { el, frames: 1, loop: 0, set: () => {} };
}

/** The space of a picture the theme does not have yet (`w` x `h`), naming the missing one. */
export function placeholder(theme: Theme, alias: string, w: number, hgt: number): HTMLElement {
  return h("div", { class: "art-placeholder", style: `width:${w}px;height:${hgt}px` }, h("span", {}, theme.t("art.missing")), h("code", {}, alias));
}

export interface Layer {
  alias: string;
  x?: number;
  y?: number;
  fps?: number;
}

/**
 * One of MOO1's screens rebuilt from its pictures (320x200 at `scale`): still pictures and
 * animations stacked in order at their positions. An `anim/<name>` layer the theme has only as
 * a still picture shows `<name>`. With a picture missing, the placeholder in its place.
 */
export function moo1Scene(theme: Theme, layers: Layer[], scale = 2, w = 320, hgt = 200): HTMLElement {
  const still = (a: string) => (theme.hasPic(a) ? a : a.replace(/^anim\//, ""));
  const missing = layers.map((l) => l.alias).find((a) => !theme.anims.has(a) && !theme.hasPic(still(a)));
  if (missing) return placeholder(theme, missing.replace(/^anim\//, ""), w * scale, hgt * scale);
  const box = h("div", { class: "moo1-scene", style: `width:${w * scale}px;height:${hgt * scale}px` });
  for (const l of layers) {
    const pos = `left:${(l.x ?? 0) * scale}px;top:${(l.y ?? 0) * scale}px`;
    const sprite = theme.anims.has(l.alias) ? animSprite(theme, l.alias, scale, l.fps) : null;
    if (sprite) {
      sprite.style.cssText += `;position:absolute;${pos}`;
      box.append(sprite);
      continue;
    }
    const alias = still(l.alias);
    const img = theme.img(alias);
    const size = img ? `;width:${img.naturalWidth * scale}px;height:${img.naturalHeight * scale}px` : "";
    box.append(h("img", { class: theme.pixel.has(alias) ? "pixel" : "", src: theme.url(alias)!, alt: "", style: `position:absolute;${pos}${size}` }));
  }
  return box;
}
