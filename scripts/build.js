#!/usr/bin/env node
// Builds the deployable site into _site/ (used by the GitHub Pages workflow).
//   ASSET_VERSION=<git sha> node scripts/build.js
// - copies an explicit allowlist only (tests/, scripts/, .github/ and docs are never published)
// - appends ?v=<version> to local CSS/JS URLs so a deploy never mixes new HTML with cached old JS
"use strict";
const fs = require("fs");
const path = require("path");
const vm = require("vm");

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

// Prerender English text into data-i18n elements and data-i18n-attr attributes. Source HTML keeps
// only keys (locales/en.js stays the single source); the deployed HTML has real text, which avoids
// a layout shift when JS fills it in and lets crawlers / no-JS visitors read the page.
const sandbox = { window: {} };
vm.runInNewContext(fs.readFileSync(path.join(ROOT, "locales/en.js"), "utf8"), sandbox);
const en = sandbox.window.LOCALES.en;
const lookup = (key) => {
  const v = key.split(".").reduce((n, k) => (n && typeof n === "object" ? n[k] : undefined), en);
  if (typeof v !== "string") { console.error(`missing i18n key: ${key}`); process.exit(1); }
  return v;
};
const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

let stamped = 0, filled = 0;
for (const page of PAGES) {
  const file = path.join(OUT, page);
  let src = fs.readFileSync(file, "utf8");
  // <tag ... data-i18n="key"></tag>  (only empty elements; <title> already has static text)
  src = src.replace(/(<([a-z0-9]+)\b[^>]*\sdata-i18n="([^"]+)"[^>]*>)(<\/\2>)/g, (_, open, tag, key, close) => {
    filled++; return open + esc(lookup(key)) + close;
  });
  // data-i18n-attr="attr:key;attr:key" -> add the attributes unless already present
  src = src.replace(/<[a-z0-9]+\b[^>]*\sdata-i18n-attr="([^"]+)"[^>]*>/g, (tagHtml, spec) => {
    const add = spec.split(";").map((p) => p.split(":").map((x) => x.trim()))
      .filter(([attr]) => !new RegExp(`\\s${attr}="`).test(tagHtml))
      .map(([attr, key]) => { filled++; return ` ${attr}="${esc(lookup(key))}"`; }).join("");
    return tagHtml.replace(/\sdata-i18n-attr=/, `${add} data-i18n-attr=`);
  });
  fs.writeFileSync(file, src);

  const html = fs.readFileSync(file, "utf8").replace(
    /(\s(?:href|src)=")((?:css|js|locales)\/[^"?#]+)"/g,
    (_, attr, url) => { stamped++; return `${attr}${url}?v=${version}"`; }
  );
  fs.writeFileSync(file, html);
}
console.log(`✔ built _site (version ${version}, ${stamped} asset URLs stamped, ${filled} strings prerendered)`);