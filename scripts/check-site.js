#!/usr/bin/env node
// Static checks for the website. Node built-ins only; run in CI before every deploy.
//   node scripts/check-site.js
// 1. every i18n key used in HTML / JS exists in locales/en.js
// 2. every local file referenced by HTML (href/src) and CSS (url()) exists
// 3. every in-page / cross-page #anchor exists
// 4. CSP compliance: each page has a CSP meta tag, no inline <script>/<style>, no style="" attributes
// 5. app-preview/ (copied by scripts/sync-app-preview.js): CSP, no inline code, every referenced file exists
"use strict";
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const ROOT = path.resolve(__dirname, "..");
const PAGES = ["index.html", "docs.html", "changelog.html", "404.html"];
const BASE = "/Minecraft_Auto_Fishing/";
// must match INCLUDE in scripts/build.js
const PUBLISHED = ["index.html", "docs.html", "changelog.html", "404.html", "sitemap.xml", "LICENSE", "css", "js", "locales", "assets", "app-preview"];   // absolute paths (used by 404.html) map to the repo root
const errors = [];
const read = (rel) => fs.readFileSync(path.join(ROOT, rel), "utf8");
const exists = (rel) => fs.existsSync(path.join(ROOT, rel));

// --- locale --------------------------------------------------------------
const sandbox = { window: {} };
vm.runInNewContext(read("locales/en.js"), sandbox, { filename: "locales/en.js" });
const en = sandbox.window.LOCALES && sandbox.window.LOCALES.en;
if (!en) { console.error("locales/en.js did not define window.LOCALES.en"); process.exit(1); }
const hasKey = (key) => key.split(".").reduce((n, k) => (n && typeof n === "object" ? n[k] : undefined), en) !== undefined;

function checkKey(key, where) {
  if (!hasKey(key)) errors.push(`${where}: missing i18n key "${key}"`);
}

// --- pages ---------------------------------------------------------------
const ids = {};
for (const page of PAGES) {
  ids[page] = new Set([...read(page).matchAll(/\sid="([^"]+)"/g)].map((m) => m[1]));
}

for (const page of PAGES) {
  const html = read(page);

  for (const m of html.matchAll(/data-i18n="([^"]+)"/g)) checkKey(m[1], page);
  for (const m of html.matchAll(/data-i18n-attr="([^"]+)"/g)) {
    for (const pair of m[1].split(";")) checkKey(pair.split(":")[1].trim(), page);
  }

  if (!/<meta http-equiv="Content-Security-Policy"/.test(html)) errors.push(`${page}: missing CSP meta tag`);
  // must match BRANCHES in js/doc-loader.js
  for (const m of html.matchAll(/data-branch="([^"]*)"/g)) {
    if (!["Source_Code", "Minecraft_Auto_Fishing_Website_Preview"].includes(m[1])) errors.push(`${page}: unknown data-branch "${m[1]}"`);
  }
  if (/<script(?![^>]*\ssrc=)[^>]*>/.test(html)) errors.push(`${page}: inline <script> (blocked by CSP)`);
  if (/<style[\s>]/.test(html)) errors.push(`${page}: inline <style> (blocked by CSP)`);
  if (/\sstyle="/.test(html)) errors.push(`${page}: style="" attribute (blocked by CSP)`);

  for (const m of html.matchAll(/\s(?:href|src)="([^"]+)"/g)) {
    const ref = m[1];
    if (/^(https?:|mailto:|data:)/.test(ref)) continue;
    const [filePart, hash] = ref.split("#");
    let file = filePart.split("?")[0];
    if (file.startsWith(BASE)) file = file.slice(BASE.length) || "./";
    else if (file.startsWith("/")) { errors.push(`${page}: absolute path outside the site "${ref}"`); continue; }
    if (file.split("/").includes("..")) { errors.push(`${page}: ".." in reference "${ref}"`); continue; }
    if (file && file !== "./" && !PUBLISHED.includes(file.split("/")[0])) { errors.push(`${page}: "${ref}" is not published (see build.js INCLUDE)`); continue; }
    const target = file === "" ? page : (file === "./" ? "index.html" : file);
    if (file !== "" && !exists(target)) { errors.push(`${page}: broken reference "${ref}"`); continue; }
    if (hash && ids[target] && !ids[target].has(hash)) errors.push(`${page}: missing anchor "${ref}"`);
  }
}

// --- App Preview: built by Vite in Source_Code, copied here by scripts/sync-app-preview.js --------
// It has its own i18n, so only the CSP rules and file references are checked.
{
  const page = "app-preview/index.html";
  if (!exists(page)) errors.push(`${page}: missing (run scripts/sync-app-preview.js)`);
  else {
    const html = read(page);
    if (!/<meta http-equiv="Content-Security-Policy"/.test(html)) errors.push(`${page}: no CSP meta tag (build it with npm run build:preview)`);
    if (/<script(?![^>]*\ssrc=)[^>]*>/.test(html)) errors.push(`${page}: inline <script> (blocked by CSP)`);
    if (/<style[\s>]/.test(html)) errors.push(`${page}: inline <style> (blocked by CSP)`);
    if (/\sstyle="/.test(html)) errors.push(`${page}: style="" attribute (blocked by CSP)`);
    const refs = [...html.matchAll(/\s(?:href|src)="([^"]+)"/g)].map((m) => ["app-preview", m[1]]);
    for (const name of fs.readdirSync(path.join(ROOT, "app-preview/assets")).filter((n) => n.endsWith(".css"))) {
      for (const m of read(`app-preview/assets/${name}`).matchAll(/url\(\s*["']?([^"')]+)["']?\s*\)/g)) refs.push(["app-preview/assets", m[1]]);
    }
    for (const [base, ref] of refs) {
      if (/^(https?:|data:)/.test(ref)) { errors.push(`${base}: external reference ${ref}`); continue; }
      const target = path.posix.normalize(`${base}/${ref.split(/[?#]/)[0]}`);
      if (!target.startsWith("app-preview/")) { errors.push(`${base}: reference outside app-preview/ "${ref}"`); continue; }
      if (!exists(target)) errors.push(`${base}: broken reference "${ref}"`);
    }
  }
}

// --- JS: literal t("key") calls -----------------------------------------
for (const file of fs.readdirSync(path.join(ROOT, "js"))) {
  const src = read(`js/${file}`);
  for (const m of src.matchAll(/\bt\(\s*"([a-z]\w*\.[\w.]*)"/g)) {   // keys always contain a dot
    if (!m[1].endsWith(".")) checkKey(m[1], `js/${file}`);   // "demo.log." + key is dynamic
  }
}

// --- CSS url() -------------------------------------------------------------
for (const file of fs.readdirSync(path.join(ROOT, "css"))) {
  for (const m of read(`css/${file}`).matchAll(/url\(\s*["']?([^"')]+)["']?\s*\)/g)) {
    if (/^(https?:|data:)/.test(m[1])) { errors.push(`css/${file}: external url() ${m[1]} (blocked by CSP)`); continue; }
    const rel = path.relative(ROOT, path.resolve(ROOT, "css", m[1]));
    if (!exists(rel)) errors.push(`css/${file}: broken url(${m[1]})`);
  }
}

if (errors.length) {
  console.error(`✖ ${errors.length} problem(s):\n  - ` + errors.join("\n  - "));
  process.exit(1);
}
console.log(`✔ site checks passed (${PAGES.length} pages)`);