#!/usr/bin/env node
// CI only, after scripts/build.js: copies the hashed CSS/JS files that the CURRENTLY LIVE pages
// reference into _site/, so visitors holding cached older HTML (GitHub Pages caches pages for
// 10 minutes) never request a file that the new deploy removed. Keeps exactly one previous generation.
// Best effort: if the live site cannot be read, the deploy continues without carry-over.
// A file is only kept if its content matches the hash in its name, so nothing can be published
// under a content-hashed name that the repo did not once build.
"use strict";
const crypto = require("crypto");
const fs = require("fs");
const path = require("path");

const SITE = "https://spacesquare640.github.io/Minecraft_Auto_Fishing/";
const OUT = path.resolve(__dirname, "..", "_site");
const PAGES = ["", "docs.html", "changelog.html", "404.html"];
// Only content-hashed asset paths: css/*.css, js/*.js, locales/*.js. No traversal, nothing else.
const ASSET = /^(?:\/Minecraft_Auto_Fishing\/)?((?:css\/[a-z0-9-]+\.([0-9a-f]{10})\.css)|(?:(?:js|locales)\/[a-z0-9-]+\.([0-9a-f]{10})\.js))$/;
const MAX_BYTES = 512 * 1024;

const get = (url) => fetch(url, { redirect: "error", cache: "no-store", signal: AbortSignal.timeout(10000) });

// Read at most MAX_BYTES; cancel the download as soon as the cap is exceeded.
async function readCapped(res) {
  const declared = Number(res.headers.get("content-length") || 0);
  if (declared > MAX_BYTES) { await res.body?.cancel(); return null; }
  const chunks = []; let total = 0;
  for await (const chunk of res.body) {
    total += chunk.length;
    if (total > MAX_BYTES) { await res.body.cancel().catch(() => {}); return null; }
    chunks.push(chunk);
  }
  return Buffer.concat(chunks);
}

(async () => {
  const wanted = new Map();   // rel path -> expected hash
  for (const page of PAGES) {
    try {
      const res = await get(`${SITE}${page}?carry=${Date.now()}`);   // bypass stale CDN copies
      if (!res.ok) continue;
      for (const m of (await res.text()).matchAll(/\s(?:href|src)="([^"]+)"/g)) {
        const hit = ASSET.exec(m[1]);
        if (hit) wanted.set(hit[1], hit[2] || hit[3]);
      }
    } catch (e) { console.log(`⚠ could not read live /${page}: ${e.message}`); }
  }
  let copied = 0;
  for (const [rel, hash] of wanted) {
    const dest = path.join(OUT, rel);
    if (!dest.startsWith(OUT + path.sep) || fs.existsSync(dest)) continue;   // unchanged files already exist
    try {
      const res = await get(SITE + rel);
      const body = res.ok ? await readCapped(res) : null;
      if (!body) { console.log(`⚠ skipped ${rel} (HTTP ${res.status} or over ${MAX_BYTES} bytes)`); continue; }
      const actual = crypto.createHash("sha256").update(body).digest("hex").slice(0, 10);
      if (actual !== hash) { console.log(`⚠ skipped ${rel} (content hash ${actual} does not match its name)`); continue; }
      fs.writeFileSync(dest, body);
      copied++;
    } catch (e) { console.log(`⚠ could not fetch ${rel}: ${e.message}`); }
  }
  console.log(`✔ carry-over: ${wanted.size} live hashed files referenced, ${copied} copied from the previous deploy`);
})();
