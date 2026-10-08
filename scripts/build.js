#!/usr/bin/env node
// Builds the deployable site into _site/ (used by the GitHub Pages workflow).
//   ASSET_VERSION=<git sha> node scripts/build.js
// - copies an explicit allowlist only (tests/, scripts/, .github/ and docs are never published)
// - appends ?v=<version> to local CSS/JS URLs so a deploy never mixes new HTML with cached old JS
"use strict";
const fs = require("fs");
const path = require("path");

const ROOT = path.resolve(__dirname, "..");
const OUT = path.join(ROOT, "_site");
const INCLUDE = ["index.html", "docs.html", "changelog.html", "sitemap.xml", "LICENSE", "css", "js", "locales", "assets"];
const PAGES = ["index.html", "docs.html", "changelog.html"];

const raw = process.env.ASSET_VERSION || "dev";
if (!/^(dev|[0-9a-f]{7,40})$/.test(raw)) {
  console.error(`ASSET_VERSION must be a git SHA, got "${raw}"`);
  process.exit(1);
}
const version = raw.slice(0, 7);

fs.rmSync(OUT, { recursive: true, force: true });
fs.mkdirSync(OUT);
// A committed symlink (e.g. assets/x -> /proc/self/environ) would be dereferenced when the
// artifact is packed, publishing files outside the allowlist. Refuse to build instead.
for (const entry of INCLUDE) {
  fs.cpSync(path.join(ROOT, entry), path.join(OUT, entry), {
    recursive: true,
    filter: (src) => {
      if (fs.lstatSync(src).isSymbolicLink()) {
        console.error(`refusing to publish symlink: ${path.relative(ROOT, src)}`);
        process.exit(1);
      }
      return true;
    }
  });
}

let stamped = 0;
for (const page of PAGES) {
  const file = path.join(OUT, page);
  const html = fs.readFileSync(file, "utf8").replace(
    /(\s(?:href|src)=")((?:css|js|locales)\/[^"?#]+)"/g,
    (_, attr, url) => { stamped++; return `${attr}${url}?v=${version}"`; }
  );
  fs.writeFileSync(file, html);
}
console.log(`✔ built _site (version ${version}, ${stamped} asset URLs stamped)`);