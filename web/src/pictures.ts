// Pictures of ships and troops for the side panel and the city window: the theme's own
// (Age: `unit/<look>`), MOO1's imported ones (Classic: `ship/<look>`), or the battle
// screen's drawing.
import { drawShip, drawTroop } from "./battle";
import type { Theme } from "./theme";
import { h } from "./ui";

const drawn = new Map<string, string>();

/** The picture of a ship design: `look` 0..23 (hull * 6 + variant), `banner` its empire's. */
export function designPicture(theme: Theme, look: number, hull: number, banner: number): string {
  const world = theme.def.map === "world";
  const own = theme.url(world ? `unit/${look % 24}` : `ship/${banner * 24 + (look % 24)}`);
  if (own) return own;
  const color = theme.banner(banner);
  const key = `${world}/${hull}/${color}`;
  let url = drawn.get(key);
  if (!url) {
    const c = document.createElement("canvas");
    c.width = 64;
    c.height = 48;
    const g = c.getContext("2d")!;
    g.scale(2, 2);
    if (world) drawTroop(g, hull, color);
    else drawShip(g, hull, color);
    url = c.toDataURL();
    drawn.set(key, url);
  }
  return url;
}

/** Drawn icons (24x24) for when the theme has no `icon/<what>` picture. */
const ICONS: Record<"world" | "space", Record<string, string>> = {
  world: {
    pop: '<circle cx="12" cy="6" r="3.5" fill="#e3c9a0" stroke="#3b2a16"/><path d="M6 22l2-11h8l2 11z" fill="#8a5a2b" stroke="#3b2a16"/>',
    factory: '<path d="M3 22V12l9-6 9 6v10z" fill="#c9a36b" stroke="#3b2a16"/><rect x="15" y="3" width="3" height="7" fill="#7a6a5a" stroke="#3b2a16"/><rect x="10" y="15" width="4" height="7" fill="#5a3b1a"/>',
    waste: '<path d="M3 22c2-6 5-8 9-8s7 2 9 8z" fill="#6b5a3a" stroke="#2b2219"/><path d="M10 12c-2-3 1-5 0-8M14 12c2-3-1-5 1-8" stroke="#555" stroke-width="2" fill="none"/>',
    base: '<path d="M7 22V8h10v14z" fill="#b9ad97" stroke="#3d342a"/><path d="M6 8V4h2v2h2V4h4v2h2V4h2v4z" fill="#b9ad97" stroke="#3d342a"/><rect x="11" y="15" width="3" height="7" fill="#3d342a"/>',
    shield: '<path d="M2 22V10h3V7h3v3h3V7h3v3h3V7h3v3h2v12z" fill="#a59a88" stroke="#3d342a"/>',
  },
  space: {
    pop: '<circle cx="12" cy="7" r="4.5" fill="#9fc4ff" stroke="#123"/><rect x="9" y="5.5" width="6" height="3" rx="1" fill="#123"/><path d="M6 22l1-9h10l1 9z" fill="#3a66c4" stroke="#123"/>',
    factory: '<path d="M2 22V12l6 3v-3l6 3v-3l8 4v6z" fill="#8090a0" stroke="#123"/><rect x="17" y="4" width="3" height="10" fill="#6a7888" stroke="#123"/>',
    waste: '<rect x="6" y="6" width="12" height="16" rx="2" fill="#4caf50" stroke="#123"/><path d="M6 11h12M6 17h12" stroke="#123"/><circle cx="12" cy="14" r="2" fill="#d8ff6a"/>',
    base: '<path d="M4 22l3-6h10l3 6z" fill="#7a8796" stroke="#123"/><path d="M11 16V6l1-4 1 4v10z" fill="#e0e6ee" stroke="#123"/>',
    shield: '<path d="M2 21a10 10 0 0 1 20 0z" fill="rgba(90,180,255,0.45)" stroke="#5ab4ff" stroke-width="1.5"/>',
  },
};

function drawnIcon(theme: Theme, what: string): string | null {
  const body = ICONS[theme.def.map === "world" ? "world" : "space"][what];
  return body ? `data:image/svg+xml,${encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">${body}</svg>`)}` : null;
}

/**
 * A row of icons repeated to show an amount (the theme's `icon/<what>`, or a drawn one): one
 * icon per `per` (rounded up), at most `max` icons, and faded ones for the room left up to
 * `cap`; null with nothing to show.
 */
export function iconRow(theme: Theme, what: string, amount: number, per: number, cap = 0, max = 40): HTMLElement | null {
  const url = theme.url(`icon/${what}`) ?? drawnIcon(theme, what);
  const n = Math.min(max, Math.ceil(Math.max(0, amount) / per));
  const room = Math.max(0, Math.min(max, Math.ceil(cap / per)) - n);
  if (!url || n + room === 0) return null;
  const icon = (cls: string) => h("img", { src: url, alt: "", class: cls });
  return h("div", { class: `icon-row icon-${what}` }, ...Array.from({ length: n }, () => icon("")), ...Array.from({ length: room }, () => icon("room")));
}
