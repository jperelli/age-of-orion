// Runs the LBX importer (crates/lbx through WebAssembly) off the main thread.
import init, { LbxImporter } from "./wasm/aoo.js";

self.onmessage = async (e: MessageEvent<{ name: string; bytes: Uint8Array }[]>) => {
  try {
    await init();
    const imp = new LbxImporter();
    for (const f of e.data) imp.add(f.name, f.bytes);
    const n = imp.run();
    const files: { path: string; bytes: Uint8Array }[] = [];
    for (let i = 0; i < n; i++) files.push({ path: imp.path(i), bytes: imp.bytes(i) });
    const manifest = JSON.parse(imp.manifest()) as unknown;
    imp.free();
    (self as unknown as Worker).postMessage({ manifest, files }, files.map((f) => f.bytes.buffer as ArrayBuffer));
  } catch (err) {
    (self as unknown as Worker).postMessage({ error: String(err) });
  }
};
