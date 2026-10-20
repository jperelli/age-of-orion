// Typed wrapper around the WebAssembly build of the Rust core (crates/wasm).
import init, { LbxImporter, WasmGame, version } from "./wasm/aoo.js";
import type { Command, NewGameOptions, View } from "./types";

let ready: Promise<unknown> | null = null;

export function loadEngine(): Promise<unknown> {
  ready ??= init();
  return ready;
}

export function engineVersion(): string {
  return version();
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
