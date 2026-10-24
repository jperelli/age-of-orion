// The GNN newscast (1oom `uinews.c`): the turn's news, one item at a time, on the TV of
// NEWSCAST.LBX when imported; in the Age theme, the herald's proclamations.

import { audio, TUNE } from "./audio";
import type { Ctx } from "./ctx";
import type { GameEvent } from "./types";
import { animSprite, picImg, picUrl } from "./pictures";
import { h } from "./ui";

/** 1oom `GAME_NEWS_*` numbers (NEWSCAST.LBX icon 3 + number). */
export const NEWS_TYPES: Record<string, number> = {
  plague: 1,
  quake: 2,
  nova: 3,
  accident: 4,
  assassin: 5,
  virus: 6,
  comet: 7,
  pirates: 8,
  derelict: 9,
  rebellion: 10,
  crystal: 11,
  amoeba: 12,
  enviro: 13,
  rich: 14,
  support: 15,
  poor: 16,
  orion: 17,
  coup: 18,
  stars: 19,
  stats: 20,
  genocide: 21,
  guardian: 22,
};

/** The news type of an event shown on the newscast, if it is one. */
export function newsType(e: GameEvent): number | null {
  if (e.type === "news") return NEWS_TYPES[e.news as string] ?? null;
  if (e.type === "orion_colonized") return NEWS_TYPES.orion;
  if (e.type === "guardian_defeated") return NEWS_TYPES.guardian;
  if (e.type === "empire_eliminated") return NEWS_TYPES.genocide;
  return null;
}

export interface NewsItem {
  type: number;
  text: string;
  ranking: string[];
}

export async function newscast(ctx: Ctx, items: NewsItem[]): Promise<void> {
  if (!items.length) return;
  const { theme } = ctx;
  const t = (k: string, p?: Record<string, string | number>) => theme.t(k, p);
  // MOO1's GNN newscast in space, the Herald on a world map.
  const tv = theme.def.map === "space";
  const screen = h("div", { class: "news-screen" });
  const text = h("div", { class: "news-text" });
  const next = h("button", { class: "primary" }, t("btn.next"));
  const body = h("div", { class: `news ${tv ? "news-tv" : "news-herald"}` });
  if (tv) {
    body.style.backgroundImage = `url("${picUrl(theme, "news/tv")}")`;
    // The newscaster talks and the globe turns (1oom `ui_news_cb1`), when animated.
    const talking = animSprite(theme, "anim/news/anchor", 0, 8);
    const globe = animSprite(theme, "anim/news/world", 0, 8);
    screen.append(talking ? Object.assign(talking, { className: `${talking.className} news-anchor` }) : picImg(theme, "news/anchor", "news-anchor"));
    screen.append(globe ? Object.assign(globe, { className: `${globe.className} news-globe` }) : picImg(theme, "news/world", "news-globe"));
  } else {
    screen.append(h("div", { class: "news-masthead" }, picImg(theme, "news/herald", "news-herald-figure"), t("news.masthead")));
  }
  const icon = h("img", { class: "news-icon", alt: "" }) as HTMLImageElement;
  screen.append(icon);
  body.append(screen, text);
  const overlay = h(
    "div",
    { class: "overlay" },
    h("div", { class: "dialog news-dialog", role: "dialog" }, h("h2", {}, t("news.title")), h("div", { class: "dialog-body" }, body), h("div", { class: "dialog-buttons" }, next)),
  );
  document.body.append(overlay);
  audio.play(TUNE.news);
  for (let i = 0; i < items.length; i++) {
    const it = items[i];
    const alias = `news/icon/${it.type}`;
    icon.classList.toggle("painted", !theme.pixel.has(alias));
    icon.src = picUrl(theme, alias);
    text.replaceChildren(
      h("p", {}, it.text),
      it.ranking.length ? h("ol", { class: "news-ranking" }, ...it.ranking.map((r) => h("li", {}, r))) : "",
    );
    next.textContent = i + 1 < items.length ? t("btn.next") : t("btn.ok");
    await new Promise<void>((resolve) => {
      const done = () => {
        document.removeEventListener("keydown", onKey);
        resolve();
      };
      const onKey = (k: KeyboardEvent) => {
        if (k.key === "Enter" || k.key === " " || k.key === "Escape") {
          k.preventDefault();
          done();
        }
      };
      document.addEventListener("keydown", onKey);
      next.onclick = done;
    });
  }
  overlay.remove();
}
