// Typed wrapper around the WebAssembly build of the Rust core (crates/wasm).
import init, { LbxImporter, WasmGame, parts, version } from "./wasm/aoo.js";
import type { Command, DesignStats, NewGameOptions, Parts, Reply, ShipDesign, Talk, View } from "./types";

let ready: Promise<unknown> | null = null;

export function loadEngine(): Promise<unknown> {
  ready ??= init();
  return ready;
}

export function engineVersion(): string {
  return version();
}

let partsCache: Parts | null = null;

/** Static ship part tables. */
export function shipParts(): Parts {
  partsCache ??= JSON.parse(parts()) as Parts;
  return partsCache;
}

export class Game {
  private constructor(private g: WasmGame) {}

  static create(opts: NewGameOptions): Game {
    return new Game(new WasmGame(JSON.stringify(opts)));
  }

  static load(save: string): Game {
    return new Game(WasmGame.load(save));
  }

  save(): string {
    return this.g.save();
  }

  view(): View {
    return JSON.parse(this.g.view()) as View;
  }

  /** Returns the error code when the core rejects the command. */
  command(cmd: Command): string | null {
    try {
      this.g.command(JSON.stringify(cmd));
      return null;
    } catch (e) {
      return e instanceof Error ? e.message : String(e);
    }
  }

  /** An audience move; returns the reply or the error code. */
  talk(withEmpire: number, t: Talk): Reply | string {
    try {
      return JSON.parse(this.g.talk(withEmpire, JSON.stringify(t))) as Reply;
    } catch (e) {
      return e instanceof Error ? e.message : String(e);
    }
  }

  designStats(d: ShipDesign): DesignStats {
    return JSON.parse(this.g.designStats(JSON.stringify(d))) as DesignStats;
  }

  endTurn(): void {
    this.g.endTurn();
  }

  free(): void {
    this.g.free();
  }
}

export interface ImportedFile {
  path: string;
  bytes: Uint8Array;
}

/**
 * Converts LBX files in a Web Worker so the page stays responsive; falls back to the main
 * thread where module workers are not available.
 */
export function importLbxAsync(files: { name: string; bytes: Uint8Array }[]): Promise<{ manifest: unknown; files: ImportedFile[] }> {
  return new Promise((resolve) => {
    let worker: Worker;
    try {
      worker = new Worker(new URL("./import-worker.ts", import.meta.url), { type: "module" });
    } catch {
      resolve(importLbx(files));
      return;
    }
    worker.onmessage = (e: MessageEvent<{ manifest: unknown; files: ImportedFile[] } | { error: string }>) => {
      worker.terminate();
      resolve("error" in e.data ? importLbx(files) : e.data);
    };
    worker.onerror = () => {
      worker.terminate();
      resolve(importLbx(files));
    };
    worker.postMessage(files);
  });
}

/** Converts LBX files to PNG/JSON assets in WebAssembly. */
export function importLbx(files: { name: string; bytes: Uint8Array }[]): { manifest: unknown; files: ImportedFile[] } {
  const imp = new LbxImporter();
  try {
    for (const f of files) imp.add(f.name, f.bytes);
    const n = imp.run();
    const out: ImportedFile[] = [];
    for (let i = 0; i < n; i++) out.push({ path: imp.path(i), bytes: imp.bytes(i) });
    return { manifest: JSON.parse(imp.manifest()), files: out };
  } finally {
    imp.free();
  }
}
