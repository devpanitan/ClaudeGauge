import { defineConfig } from "vite";

// Vite config tuned for Tauri:
// - fixed dev port so tauri.conf.json devUrl matches
// - do not clear the screen so Rust/cargo logs stay visible
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "es2020",
    minify: "esbuild",
    outDir: "dist",
    emptyOutDir: true,
  },
});
