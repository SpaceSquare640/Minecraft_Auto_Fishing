import { readFileSync } from "node:fs";
import { defineConfig, type Plugin } from "vite";

const { version } = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf8")) as { version: string };

// The website App Preview (`npm run build:preview`) is a static page, so it carries its own CSP;
// the app's CSP comes from tauri.conf.json. The only network request is the manual update check.
const PREVIEW_CSP =
  "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; " +
  "connect-src https://api.github.com; base-uri 'none'; form-action 'none'; object-src 'none'";

function previewCsp(): Plugin {
  return {
    name: "app-preview-csp",
    apply: "build",
    transformIndexHtml: (html) =>
      html.replace('<meta charset="utf-8">', `<meta charset="utf-8">\n    <meta http-equiv="Content-Security-Policy" content="${PREVIEW_CSP}">`),
  };
}

// Tauri serves the built files itself; the dev server is only for `tauri dev`.
export default defineConfig(({ mode }) => ({
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: { outDir: "dist", target: "es2022" },
  define: { __APP_VERSION__: JSON.stringify(version) },
  plugins: mode === "app-preview" ? [previewCsp()] : [],
}));