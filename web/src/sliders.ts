// The slider control used everywhere (MOO1's): a name you click to lock the slider, a −
// and a + to move it in steps, the slider itself and its value. In a linked group the
// sliders share a total (spending, research) and moving one moves the others, except the
// locked ones (1oom `game_adjust_slider_group`).

import { h } from "./ui";

export interface SliderSpec {
  label: string;
  title?: string;
  value: number;
  /** Locked (only in groups that allow locking). */
  locked?: boolean;
  disabled?: boolean;
}

export interface SliderOpts {
  specs: SliderSpec[];
  min?: number;
  max?: number;
  /** Step of the slider itself. */
  step?: number;
  /** Step of − and + (MOO1: 4; Shift or Ctrl: 1). */
  plusStep?: number;
  /** Sliders share this total (linked group), or move on their own. */
  total?: number;
  /** Text after slider `i` for the values `v`. */
  format: (i: number, v: number[]) => string;
  /** New values, when a slider is let go or − / + is pressed. */
  onCommit: (v: number[]) => void;
  /** Called when the player clicks a name; without it names do not lock. */
  onLock?: (i: number, locked: boolean) => void;
  lockTitle?: string;
}

/**
 * 1oom `game_adjust_slider_group`: slider `i` takes `value` (at most what the locked ones
 * leave), the other unlocked ones keep theirs in order while there is room, and what is
 * left goes to the last unlocked slider (the first, when `i` is the last).
 */
export function adjustGroup(values: number[], i: number, value: number, locks: boolean[], total = 100): number[] {
  const v = [...values];
  let left = total;
  let first = -1;
  let last = -1;
  v.forEach((x, j) => {
    if (locks[j]) left -= x;
    else {
      if (first < 0) first = j;
      last = j;
    }
  });
  value = Math.max(0, Math.min(value, left));
  v[i] = value;
  left -= value;
  for (let j = 0; j < v.length; j++) {
    if (j === i || locks[j]) continue;
    if (v[j] <= left) left -= v[j];
    else {
      v[j] = left;
      left = 0;
    }
  }
  if (left > 0 && first >= 0) v[i !== last ? last : first] += left;
  return v;
}

export function sliders(o: SliderOpts): HTMLElement {
  const min = o.min ?? 0;
  const max = o.max ?? 100;
  const step = o.step ?? 1;
  const values = o.specs.map((s) => s.value);
  const locks = o.specs.map((s) => !!s.locked);
  const inputs: HTMLInputElement[] = [];
  const outs: HTMLElement[] = [];
  const rows: HTMLElement[] = [];
  const show = () =>
    values.forEach((x, j) => {
      inputs[j].value = String(x);
      outs[j].textContent = o.format(j, values);
    });
  const set = (i: number, x: number) => {
    const nv = o.total !== undefined ? adjustGroup(values, i, x, locks, o.total) : values.map((y, j) => (j === i ? Math.max(min, Math.min(max, x)) : y));
    nv.forEach((y, j) => (values[j] = y));
    show();
  };
  const el = h("div", { class: "sliders" });
  o.specs.forEach((s, i) => {
    const input = h("input", { type: "range", min, max, step, value: s.value, "aria-label": s.label }) as HTMLInputElement;
    const out = h("span", { class: "bc" });
    const minus = h("button", { class: "mini step", title: "−", "aria-label": `${s.label} −` }, "−") as HTMLButtonElement;
    const plus = h("button", { class: "mini step", title: "+", "aria-label": `${s.label} +` }, "+") as HTMLButtonElement;
    const lockable = !!o.onLock;
    const label = lockable
      ? (h("button", { class: "slider-label", title: [s.title, o.lockTitle].filter(Boolean).join(" · "), "aria-pressed": String(locks[i]) }, s.label) as HTMLButtonElement)
      : h("label", { class: "slider-label", title: s.title ?? "" }, s.label);
    const row = h("div", { class: `slider-row${locks[i] ? " locked" : ""}` }, label, minus, input, plus, out);
    const sync = () => {
      const off = locks[i] || !!s.disabled;
      input.disabled = off;
      minus.disabled = off;
      plus.disabled = off;
      row.classList.toggle("locked", locks[i]);
      if (label instanceof HTMLButtonElement) label.setAttribute("aria-pressed", String(locks[i]));
    };
    input.addEventListener("input", () => set(i, Number(input.value)));
    input.addEventListener("change", () => o.onCommit([...values]));
    const bump = (dir: number) => (e: MouseEvent) => {
      const d = e.shiftKey || e.ctrlKey ? step : (o.plusStep ?? Math.max(step, 4));
      set(i, values[i] + dir * d);
      o.onCommit([...values]);
    };
    minus.addEventListener("click", bump(-1));
    plus.addEventListener("click", bump(1));
    if (label instanceof HTMLButtonElement) {
      label.addEventListener("click", () => {
        locks[i] = !locks[i];
        sync();
        o.onLock?.(i, locks[i]);
      });
    }
    inputs.push(input);
    outs.push(out);
    rows.push(row);
    sync();
    el.append(row);
  });
  show();
  return el;
}

/** One slider on its own (tax, security, spies, volume). */
export function slider(o: Omit<SliderOpts, "specs" | "format" | "onCommit" | "total" | "onLock"> & {
  label: string;
  title?: string;
  value: number;
  disabled?: boolean;
  format: (v: number) => string;
  onCommit: (v: number) => void;
}): HTMLElement {
  return sliders({
    ...o,
    specs: [{ label: o.label, title: o.title, value: o.value, disabled: o.disabled }],
    format: (_, v) => o.format(v[0]),
    onCommit: (v) => o.onCommit(v[0]),
  });
}
