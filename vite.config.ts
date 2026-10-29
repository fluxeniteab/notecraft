import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: ["es2021", "chrome100", "safari13"],
    minify: "esbuild",
    sourcemap: false,
  },
  test: {
    environment: "jsdom",
    globals: true,
    css: true,
    include: ["src/**/*.test.ts"],
    exclude: ["node_modules", "deprecated", "native"],
  },
});
