#!/usr/bin/env node
// Live checks against the deployed site. Node built-ins only (global fetch, Node 18+).
//
//   SITE_URL=https://.../ EXPECTED_VERSION=<sha> node scripts/smoke-test.js deploy
//   SITE_URL=https://.../ node scripts/smoke-test.js health
//
// deploy: run right after a Pages deploy. Retries while the CDN catches up, then checks that the
//         new version is live, every page works and nothing outside the allowlist is published.
// health: weekly. Same page checks (any version) plus external links and the Discord invite.
"use strict";

const MODE = process.argv[2];
const SITE = process.env.SITE_URL || "";
const EXPECTED_SITE = "https://spacesquare640.github.io/Minecraft_Auto_Fishing/";
const EXPECTED = (process.env.EXPECTED_VERSION || "").slice(0, 7);
const PAGES = ["", "docs.html", "changelog.html"];
const PRIVATE = ["tests/markdown.test.js", "scripts/build.js", "scripts/smoke-test.js", "scripts/carry-over.js", ".github/workflows/pages.yml", "README.md", "Change%20Log.md"];
const ASSETS = ["sitemap.xml", "assets/img/og-image.png", "assets/img/icon.webp", "assets/fonts/PressStart2P-latin.woff2", "LICENSE"];
const DISCORD_INVITE = "aaUQVJeCgC";
const DISCORD_GUILD_ID = "1150040065586770070"; // Player Club; a lapsed code re-claimed by another server must fail
const EXTERNAL = [
  "https://github.com/SpaceSquare640/Minecraft_Auto_Fishing",
  "https://github.com/SpaceSquare640/Minecraft_Auto_Fishing/blob/Source_Code/LICENSE",
  "https://raw.githubusercontent.com/SpaceSquare640/Minecraft_Auto_Fishing/Source_Code/docs/overview.md",
  "https://raw.githubusercontent.com/SpaceSquare640/Minecraft_Auto_Fishing/Source_Code/Change%20Log.md",
  "https://raw.githubusercontent.com/SpaceSquare640/Minecraft_Auto_Fishing/Minecraft_Auto_Fishing_Website_Preview/Change%20Log.md"
];

if (!["deploy", "health"].includes(MODE)) fail(`usage: smoke-test.js deploy|health (got "${MODE}")`);
if (SITE !== EXPECTED_SITE) fail(`SITE_URL must be exactly ${EXPECTED_SITE} (got "${SITE}")`);
if (MODE === "deploy" && !/^[0-9a-f]{7}$/.test(EXPECTED)) fail("EXPECTED_VERSION must be a git SHA in deploy mode");

const errors = [], warnings = [];
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
function fail(msg) { console.error(`✖ ${msg}`); process.exit(1); }

async function get(url, opts = {}) {
  const res = await fetch(url, { redirect: "follow", cache: "no-store", ...opts, signal: AbortSignal.timeout(10000) });
  return { status: res.status, finalUrl: res.url, text: opts.method === "HEAD" ? "" : await res.text() };
}

// Site requests must stay on the Pages origin: a redirect elsewhere (e.g. a lapsed custom domain)
// would otherwise make the checks pass against someone else's content.
async function site(path, opts) {
  const res = await get(SITE + path, opts);
  if (new URL(res.finalUrl).origin !== new URL(SITE).origin) throw new Error(`redirected off-site to ${new URL(res.finalUrl).origin}`);
  return res;
}

async function check(label, fn) {
  try { await fn(); } catch (e) { errors.push(`${label}: ${e.message}`); }
}

// Pages' CDN can briefly serve 503 or the previous version right after a deploy.
async function waitForVersion() {
  for (let attempt = 1; attempt <= 12; attempt++) {
    try {
      const { status, text } = await site(`?smoke=${Date.now()}`);
      if (status === 200 && text.includes(`name="build-version" content="${EXPECTED}"`)) return console.log(`✔ version ${EXPECTED} is live (attempt ${attempt})`);
      console.log(`… attempt ${attempt}: HTTP ${status}, version not live yet`);
    } catch (e) { console.log(`… attempt ${attempt}: ${e.message}`); }
    await sleep(10000);
  }
  errors.push(`version ${EXPECTED} not live after 12 attempts`);
}

// Every CSS/JS a page references must be content-hashed and load (relative or /Minecraft_Auto_Fishing/ paths).
async function checkAssets(label, html) {
  for (const m of html.matchAll(/\s(?:href|src)="(?:\/Minecraft_Auto_Fishing\/)?((?:css|js|locales)\/[^"]+)"/g)) {
    if (!/^(?:css|js|locales)\/[a-z0-9-]+\.[0-9a-f]{10}\.(?:css|js)$/.test(m[1])) { errors.push(`${label} → ${m[1]} is not content-hashed`); continue; }
    const { status } = await site(m[1], { method: "HEAD" });
    if (status !== 200) errors.push(`${label} → ${m[1]} HTTP ${status}`);
  }
}

async function checkPages() {
  for (const page of PAGES) await check(`/${page}`, async () => {
    const { status, text } = await site(page);
    if (status !== 200) return errors.push(`/${page} → HTTP ${status}`);
    if (!/<meta http-equiv="Content-Security-Policy"/.test(text)) errors.push(`/${page}: CSP meta missing`);
    if (!text.includes(`discord.gg/${DISCORD_INVITE}`)) errors.push(`/${page}: Discord link missing`);
    if (/<h1[^>]*>\s*<\/h1>/.test(text)) errors.push(`/${page}: empty <h1> (text not prerendered)`);
    if (MODE === "deploy" && !text.includes(`name="build-version" content="${EXPECTED}"`)) errors.push(`/${page}: not version ${EXPECTED}`);
    // every CSS/JS the page references must load (content-hashed names from scripts/build.js)
    await checkAssets(`/${page}`, text);
  });
  // unknown URLs must get the project's own 404 page, also below a sub-path
  await check("custom 404", async () => {
    const { status, text } = await site(`missing/deep/page-${Date.now()}`);
    if (status !== 404) errors.push(`unknown URL → HTTP ${status} (expected 404)`);
    if (!/name="robots" content="noindex"/.test(text) || !text.includes("discord.gg/")) errors.push("unknown URL does not show the custom 404 page");
    else await checkAssets("404 page", text);
  });
  for (const asset of ASSETS) await check(`/${asset}`, async () => {
    const { status } = await site(asset, { method: "HEAD" });
    if (status !== 200) errors.push(`/${asset} → HTTP ${status}`);
  });
  for (const file of PRIVATE) await check(`/${file}`, async () => {
    const { status } = await site(file, { method: "HEAD" });
    if (status !== 404) errors.push(`/${file} should not be published (HTTP ${status})`);
  });
}

async function checkExternal() {
  for (const url of EXTERNAL) {
    try {
      const { status } = await get(url, { method: "HEAD" });
      if (status !== 200) errors.push(`${url} → HTTP ${status}`);
    } catch (e) { errors.push(`${url} → ${e.message}`); }
  }
  // Public, unauthenticated invite lookup. 404 = invite revoked/expired (fail);
  // rate limits or bot protection (403/429) are reported as warnings, not failures.
  try {
    const { status, text } = await get(`https://discord.com/api/v10/invites/${DISCORD_INVITE}`);
    if (status === 200) {
      const data = JSON.parse(text);
      const guildId = data.guild && data.guild.id;
      if (guildId !== DISCORD_GUILD_ID) errors.push(`Discord invite now points to another server (guild ${JSON.stringify(guildId)})`);
      else if (data.expires_at) errors.push(`Discord invite is no longer permanent (expires ${JSON.stringify(data.expires_at)})`);
      else console.log("✔ Discord invite valid and points to the project server");
    } else if (status === 404) errors.push("Discord invite is invalid or revoked");
    else warnings.push(`Discord invite check inconclusive (HTTP ${status})`);
  } catch (e) { warnings.push(`Discord invite check failed: ${e.message}`); }
}

(async () => {
  if (MODE === "deploy") await waitForVersion();
  await checkPages();
  if (MODE === "health") await checkExternal();
  for (const w of warnings) console.log(`⚠ ${w}`);
  if (errors.length) { console.error(`✖ ${errors.length} problem(s):\n  - ${errors.join("\n  - ")}`); process.exit(1); }
  console.log(`✔ ${MODE} checks passed`);
})().catch((e) => fail(e.stack || e.message));