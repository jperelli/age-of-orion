// Controls shared across screens: a select whose options have pictures, and a small popup
// explaining a value.
import { h } from "./ui";

export interface PictureOption<T> {
  value: T;
  label: string;
  /** Picture URL; `swatch` draws a colour instead. */
  img?: string | null;
  swatch?: string;
  /** Smaller second line. */
  sub?: string;
  pixel?: boolean;
}

export interface PictureSelect<T> {
  el: HTMLElement;
  readonly value: T;
}

function optionBody<T>(o: PictureOption<T>): HTMLElement[] {
  const pic = o.img
    ? h("img", { class: `pic${o.pixel ? " pixel" : ""}`, src: o.img, alt: "" })
    : o.swatch
      ? h("span", { class: "pic swatch", style: `background:${o.swatch}` })
      : h("span", { class: "pic none" }, "?");
  return [pic, h("span", { class: "text" }, h("span", {}, o.label), o.sub ? h("small", {}, o.sub) : null)];
}

/** A select showing a picture next to each option (empires, banners, ships...). */
export function pictureSelect<T>(options: PictureOption<T>[], value: T, onChange?: (v: T) => void, cls = ""): PictureSelect<T> {
  let current = options.find((o) => o.value === value) ?? options[0];
  const button = h("button", { type: "button", class: "picture-select-button", "aria-haspopup": "listbox" });
  const list = h("div", { class: "picture-select-list", role: "listbox" });
  const el = h("div", { class: `picture-select ${cls}` }, button, list);
  const show = () => button.replaceChildren(...optionBody(current), h("span", { class: "caret" }, "▾"));
  const close = () => {
    el.classList.remove("open");
    document.removeEventListener("pointerdown", outside, true);
    document.removeEventListener("keydown", onKey, true);
  };
  const outside = (e: Event) => {
    if (!el.contains(e.target as Node)) close();
  };
  const onKey = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.stopPropagation();
      close();
    }
  };
  button.addEventListener("click", () => {
    if (el.classList.contains("open")) return close();
    list.replaceChildren(
      ...options.map((o) =>
        h(
          "button",
          {
            type: "button",
            role: "option",
            class: `picture-option${o === current ? " selected" : ""}`,
            onclick: () => {
              current = o;
              show();
              close();
              onChange?.(o.value);
            },
          },
          ...optionBody(o),
        ),
      ),
    );
    el.classList.add("open");
    list.querySelector<HTMLElement>(".selected")?.scrollIntoView({ block: "nearest" });
    document.addEventListener("pointerdown", outside, true);
    document.addEventListener("keydown", onKey, true);
  });
  show();
  return {
    el,
    get value() {
      return current.value;
    },
  };
}

let openPopover: HTMLElement | null = null;

function closePopover(): void {
  openPopover?.remove();
  openPopover = null;
}

// Any click closes the popup (the value that opened it toggles it itself).
document.addEventListener("pointerdown", (e) => {
  if (openPopover && !(e.target as Element)?.closest?.(".explain")) closePopover();
});
document.addEventListener(
  "keydown",
  (e) => {
    if (e.key === "Escape" && openPopover) {
      e.stopPropagation();
      closePopover();
    }
  },
  true,
);

/** A value that explains itself in a small popup when clicked. */
export function explained(label: string | HTMLElement, title: string, lines: string[]): HTMLElement {
  const el = h("button", { type: "button", class: "explain" }, label);
  el.addEventListener("click", () => {
    const again = openPopover?.dataset.for === title;
    closePopover();
    if (again) return;
    const box = h("div", { class: "popover", role: "tooltip" }, h("b", {}, title), ...lines.map((l) => h("p", {}, l)));
    box.dataset.for = title;
    document.body.append(box);
    const r = el.getBoundingClientRect();
    const w = Math.min(320, window.innerWidth - 16);
    box.style.width = `${w}px`;
    box.style.left = `${Math.max(8, Math.min(window.innerWidth - w - 8, r.right - w))}px`;
    const below = r.bottom + 6;
    box.style.top = `${below + box.offsetHeight > window.innerHeight ? Math.max(8, r.top - box.offsetHeight - 6) : below}px`;
    openPopover = box;
  });
  return el;
}
