import { defineConfig } from "vite";

// Tauri serves the built files itself; the dev server is only for `tauri dev`.
export default defineConfig({
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: { outDir: "dist", target: "es2022" },
});
