import { defineConfig } from "vite";

// Relative base so the same build works on GitHub Pages (sub path), in Tauri and from a zip.
export default defineConfig({
  base: "./",
  build: { target: "es2022", outDir: "dist", assetsInlineLimit: 0 },
  worker: { format: "es" },
  // The MOO1 comparison (`src/about.ts`) reads `../docs`.
  server: { port: 5173, fs: { allow: [".."] } },
});
