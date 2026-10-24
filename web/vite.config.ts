import { defineConfig } from "vite";
import { themesPlugin } from "./vite-themes";

// Relative base so the same build works on GitHub Pages (sub path), in Tauri and from a zip.
export default defineConfig({
  base: "./",
  build: { target: "es2022", outDir: "dist", assetsInlineLimit: 0 },
  worker: { format: "es" },
  plugins: [themesPlugin()],
  // The MOO1 comparison (`src/about.ts`) reads `../docs`.
  server: { port: 5173, fs: { allow: [".."] } },
});
