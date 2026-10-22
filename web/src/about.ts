// "What is the same as Master of Orion, and what differs": docs/MOO1.md (or its Spanish
// version) shown in a dialog, so the game and the repository read the same text.
import type { Theme } from "./theme";
import { h, modal } from "./ui";

const DOCS: Record<string, () => Promise<{ default: string }>> = {
  en: () => import("../../docs/MOO1.md?raw"),
  es: () => import("../../docs/MOO1.es.md?raw"),
};
/** Links between the documents point to the repository. */
const REPO_DOCS = "https://github.com/jperelli/age-of-orion/blob/main/docs/";

export async function moo1Dialog(theme: Theme, lang: string): Promise<void> {
  const [{ marked }, doc] = await Promise.all([import("marked"), (DOCS[lang] ?? DOCS.en)()]);
  // The document's title becomes the dialog's.
  const [, title = "", md = doc.default] = /^# (.*)\n([\s\S]*)$/.exec(doc.default) ?? [];
  const body = h("div", { class: "doc" });
  body.innerHTML = await marked.parse(md);
  for (const a of body.querySelectorAll("a")) {
    const href = a.getAttribute("href") ?? "";
    if (!/^[a-z]+:/i.test(href)) a.href = new URL(href, REPO_DOCS).href;
    a.target = "_blank";
    a.rel = "noopener";
  }
  await modal(title, body, [theme.t("btn.close")], "huge");
}
