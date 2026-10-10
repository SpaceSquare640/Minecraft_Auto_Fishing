#!/usr/bin/env node
// Copies the App Preview (the real app UI on a simulated backend) into app-preview/.
// Local tool only: CI never builds the app, because this branch installs no packages.
//   1. Source_Code/app:  npm run build:preview
//   2. this branch:      node scripts/sync-app-preview.js [path to Source_Code/app/dist-preview]
"use strict";
const fs = require("fs");
const path = require("path");

const ROOT = path.resolve(__dirname, "..");
const OUT = path.join(ROOT, "app-preview");
const SRC = path.resolve(process.argv[2] || path.join(ROOT, "..", "Source_Code", "app", "dist-preview"));
const ALLOWED = /\.(html|js|css|woff2|webp|png|svg)$/;

function fail(message) {
  console.error(`✖ ${message}`);
  process.exit(1);
}

const index = path.join(SRC, "index.html");
if (!fs.existsSync(index)) fail(`${index} not found: run "npm run build:preview" in Source_Code/app first`);
if (!/<meta http-equiv="Content-Security-Policy"/.test(fs.readFileSync(index, "utf8"))) {
  fail("the preview has no CSP meta tag: build it with \"npm run build:preview\", not \"npm run build\"");
}

// Collect first, so a bad file aborts before app-preview/ is touched.
const files = [];
(function walk(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isSymbolicLink()) fail(`refusing to copy symlink ${full}`);
    if (entry.isDirectory()) walk(full);
    else if (ALLOWED.test(entry.name)) files.push(path.relative(SRC, full));
    else fail(`unexpected file type ${full}`);
  }
})(SRC);

fs.rmSync(OUT, { recursive: true, force: true });
for (const rel of files) {
  fs.mkdirSync(path.dirname(path.join(OUT, rel)), { recursive: true });
  fs.copyFileSync(path.join(SRC, rel), path.join(OUT, rel));
}
console.log(`✔ copied ${files.length} files into app-preview/`);