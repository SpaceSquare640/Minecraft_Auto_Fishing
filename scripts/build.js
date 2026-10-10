#!/usr/bin/env node
// Builds the deployable site into _site/ (used by the GitHub Pages workflow).
//   ASSET_VERSION=<git sha> node scripts/build.js
// - copies an explicit allowlist only (tests/, scripts/, .github/ and docs are never published)
// - prerenders English text into the HTML
// - content-hashes CSS/JS file names (base.css -> base.3f9a2c1d0e.css) and points the HTML at them, so a
//   page can only ever load the exact CSS/JS it was built with. GitHub Pages ignores query strings, so
//   ?v= alone cannot guarantee that. Unhashed copies stay published for pages built before hashing, and
//   scripts/carry-over.js keeps the previous deploy's hashed files so cached older HTML still works.
"use strict";
const crypto = require("crypto");
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const ROOT = path.resolve(__dirname, "..");
const OUT = path.join(ROOT, "_site");
// app-preview/ is the Vite build of the app UI: its file names are already content-hashed.
const INCLUDE = ["index.html", "docs.html", "changelog.html", "404.html", "sitemap.xml", "LICENSE", "css", "js", "locales", "assets", "app-preview"];
const PAGES = ["index.html", "docs.html", "changelog.html", "404.html"];
const HASHED_DIRS = ["css", "js", "locales"];

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

// Content-hash every CSS/JS file: write name.<hash10>.ext next to the original.
const hashed = {};   // "css/base.css" -> "css/base.3f9a2c1d0e.css"
for (const dir of HASHED_DIRS) {
  for (const name of fs.readdirSync(path.join(OUT, dir))) {
    const m = /^([a-z0-9-]+)\.(css|js)$/.exec(name);
    if (!m) continue;
    const content = fs.readFileSync(path.join(OUT, dir, name));
    const hash = crypto.createHash("sha256").update(content).digest("hex").slice(0, 10);
    const target = `${m[1]}.${hash}.${m[2]}`;
    fs.writeFileSync(path.join(OUT, dir, target), content);
    hashed[`${dir}/${name}`] = `${dir}/${target}`;
  }
}

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

  const html = fs.readFileSync(file, "utf8")
    .replace(/(\s(?:href|src)=")((?:\/Minecraft_Auto_Fishing\/)?)((?:css|js|locales)\/[^"?#]+)"/g, (whole, attr, prefix, url) => {
      if (!hashed[url]) { console.error(`${page}: no hashed file for ${url}`); process.exit(1); }
      stamped++; return `${attr}${prefix}${hashed[url]}"`;
    })
    // marker the post-deploy smoke test waits for
    .replace('<meta charset="utf-8">', `<meta charset="utf-8">
  <meta name="build-version" content="${version}">`);
  // Any reference the rewrite did not catch (?v=, ./ prefix, single quotes, upper case) is a build error.
  const leftover = /["'\/=](?:\.\/)?(?:css|js|locales)\/[a-z0-9-]+\.(?:css|js)(?=["'?#\s>])/i.exec(html);
  if (leftover) { console.error(`${page}: unhashed asset reference ${leftover[0]}`); process.exit(1); }
  fs.writeFileSync(file, html);
}
console.log(`✔ built _site (version ${version}, ${Object.keys(hashed).length} files hashed, ${stamped} references rewritten, ${filled} strings prerendered)`);