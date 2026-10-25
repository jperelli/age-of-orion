// Minimal DOM helpers.

type Child = Node | string | number | null | undefined | false;
type Attrs = Record<string, string | number | boolean | EventListener | null | undefined>;

export function h<K extends keyof HTMLElementTagNameMap>(tag: K, attrs: Attrs = {}, ...children: Child[]): HTMLElementTagNameMap[K] {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (v === null || v === undefined || v === false) continue;
    if (k.startsWith("on") && typeof v === "function") el.addEventListener(k.slice(2), v as EventListener);
    else if (k === "class") el.className = String(v);
    else if (k in el && typeof v !== "string") (el as unknown as Record<string, unknown>)[k] = v;
    else el.setAttribute(k, v === true ? "" : String(v));
  }
  append(el, ...children);
  return el;
}

export function append(el: HTMLElement, ...children: Child[]): void {
  for (const c of children) {
    if (c === null || c === undefined || c === false) continue;
    el.append(c instanceof Node ? c : String(c));
  }
}

export function clear(el: HTMLElement): HTMLElement {
  el.replaceChildren();
  return el;
}

let closeLabel = "Close";

/** The tooltip of the dialogs' close button, in the current language. */
export function setCloseLabel(label: string): void {
  closeLabel = label;
}

/**
 * A dialog; resolves with the index of the button pressed (-1: closed with × or Escape).
 * `onOpen` gets a way to close it from inside its body, as if button `i` had been pressed.
 * With "required" among the classes it has no × and Escape does nothing (a choice must be
 * made).
 */
/** How many dialogs have opened (the player's stats). */
export const opened = { dialogs: 0 };

export function modal(title: string, body: Child | Child[], buttons: string[] = ["OK"], cls = "", onOpen?: (close: (i: number) => void) => void): Promise<number> {
  opened.dialogs++;
  return new Promise((resolve) => {
    const close = (i: number) => {
      document.removeEventListener("keydown", onKey);
      overlay.remove();
      resolve(i);
    };
    const required = cls.split(" ").includes("required");
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !required) close(-1);
      const typing = e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement;
      if (e.key === "Enter" && buttons.length === 1 && !typing) close(0);
    };
    document.addEventListener("keydown", onKey);
    const overlay = h(
      "div",
      { class: "overlay" },
      h(
        "div",
        { class: `dialog ${cls}`, role: "dialog" },
        h(
          "div",
          { class: "dialog-head" },
          h("h2", {}, title),
          required ? null : h("button", { class: "dialog-close", title: closeLabel, "aria-label": closeLabel, onclick: () => close(-1) }, "×"),
        ),
        h("div", { class: "dialog-body" }, ...(Array.isArray(body) ? body : [body])),
        h("div", { class: "dialog-buttons" }, ...buttons.map((b, i) => h("button", { class: i === 0 ? "primary" : "", onclick: () => close(i) }, b))),
      ),
    );
    document.body.append(overlay);
    overlay.querySelector<HTMLButtonElement>(".dialog-buttons button")?.focus();
    onOpen?.(close);
  });
}

/** Runs `fn` (which re-renders part of a dialog) keeping the dialog's scroll position. */
export function keepScroll(el: HTMLElement, fn: () => void): void {
  const sc = el.closest(".dialog-body");
  const top = sc?.scrollTop ?? 0;
  fn();
  if (sc) sc.scrollTop = top;
}

export function toast(msg: string): void {
  const t = h("div", { class: "toast" }, msg);
  document.body.append(t);
  setTimeout(() => t.classList.add("show"), 10);
  setTimeout(() => {
    t.classList.remove("show");
    setTimeout(() => t.remove(), 400);
  }, 2600);
}

export function download(name: string, data: BlobPart, type = "application/json"): void {
  const url = URL.createObjectURL(new Blob([data], { type }));
  const a = h("a", { href: url, download: name });
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function pickFiles(opts: { accept?: string; multiple?: boolean; directory?: boolean }): Promise<File[]> {
  return new Promise((resolve) => {
    const input = h("input", { type: "file", accept: opts.accept ?? "", multiple: !!opts.multiple });
    if (opts.directory) input.setAttribute("webkitdirectory", "");
    input.onchange = () => resolve(Array.from(input.files ?? []));
    input.click();
  });
}
