# Change Log

All notable changes to the website branch are recorded here.

## 2026-10-08

- Added site skeleton (`index.html`) with hero section and a placeholder for the interactive demo — Website
- Added design tokens, base layout, responsive rules (mobile / tablet / desktop) and reduced-motion handling — CSS
- Added status badge, Minecraft-style block button and GitHub button (adapted from Uiverse.io, MIT) — Components
- Added i18n layer (`data-i18n`, `data-i18n-attr`) with English locale; text is no longer hard-coded in HTML — i18n
- Added project icon asset — Assets
- Updated README with structure, local preview and translation guide — Docs

## 2026-10-08 (icon update)

- Replaced the white-background `icon.jpg` with a transparent `icon.png` (512px); removed the white item-frame card and added an outline-following drop shadow — Hero / Assets
- Added `favicon.ico` (16 / 32 / 48px) and `apple-touch-icon.png` (180px) — Assets
- Removed `assets/img/icon.jpg` (still available in git history at `50f64ca`) — Assets

## 2026-10-08 (WebP)

- Hero icon switched from `icon.png` (512px, 493 KB) to `icon.webp` (720px, quality 85, ~45 KB) — sharper on high-DPI screens and about 91% smaller — Hero / Assets
- Removed `assets/img/icon.png` (still available in git history at `de36a7e`) — Assets

## 2026-10-08 (Docs & Changelog pages)

- Added `docs.html` and `changelog.html`; content is loaded live from the repository's Markdown files — Pages
- Added site navigation (Home · Docs · Changelog · GitHub) to all pages — Navigation
- Added `js/markdown.js`, an escape-first Markdown renderer (raw HTML is never rendered; only http/https/mailto links), and `js/doc-loader.js` with loading and error states — JS
- Added `css/prose.css` for rendered documents — CSS
- Added `tests/markdown.test.js` (8 tests, including XSS cases) — Tests
- Updated README with pages, local preview of unpushed docs and test command — Docs

## 2026-10-08 (Interactive demo)

- Added the interactive fishing demo: pixel-art pond scene, Manual / Auto mode lever, Cast / Reel in button, manual vs. auto results table and activity log — Demo
- Manual mode: 1.0 s bite window, reaction time, "too slow" and "too early" outcomes; Auto mode loops on its own with Pause / Resume — Demo
- Demo pauses when scrolled out of view or when the tab is hidden; reduced-motion users get a static scene that is still playable — Accessibility
- Screen readers hear the bite and results in Manual mode, and only start / pause in Auto mode — Accessibility
- `i18n.t()` now fills `{name}` placeholders — i18n
- Added `tests/demo-engine.test.js` (10 tests with a fake clock) — Tests

## 2026-10-08 (Home sections & footer)

- Added How it works (4 steps), Features (with Planned / Available status), Getting started (coming soon) and FAQ sections to the home page — Home
- Added a shared footer to all pages: links, GPL-3.0 license link, non-affiliation notice, Uiverse.io credits — Footer
- Added a back-to-top button that moves keyboard focus to the top navigation — Navigation
- Added scroll reveal for cards; content stays visible without JavaScript and under reduced motion — UX
- Added a shared pixel-icon sprite for step and feature cards — Assets

## 2026-10-08 (License)

- Added `LICENSE` (GPL-3.0) — Repository

## 2026-10-08 (M4: security, performance, CI/CD)

- Self-hosted the Press Start 2P font (latin + latin-ext, OFL 1.1); removed Google Fonts requests — Privacy / Performance
- Added a Content-Security-Policy meta tag and an explicit referrer policy to all pages — Security
- Added Open Graph / Twitter card tags, a 1200x630 social preview image, canonical URLs and `sitemap.xml` — SEO / Sharing
- Cards and other hidden-until-scrolled content are always visible when printing — UX
- Added `scripts/check-site.js` (i18n keys, references, anchors, CSP compliance) and `scripts/build.js` (allowlist build with `?v=<sha>` cache busting) — Tooling
- Added the GitHub Actions workflow `pages.yml`: test, build and deploy to GitHub Pages with least-privilege permissions and SHA-pinned official actions — CI/CD
- Security review fixes: linear-time heading parsing (was cubic on long whitespace), emphasis can no longer alter link URLs, exact-origin check for same-site links, build refuses symlinks, CSP `connect-src` scoped to this repository, workflow concurrency per branch with a fixed deploy group — Security
- Added 4 security regression tests (22 tests in total) — Tests

## 2026-10-08 (M4b: quality pass)

- Build now prerenders English text into the deployed HTML; fixes layout shift (CLS up to 0.38 → 0) and makes pages readable without JavaScript — Performance / SEO
- Docs area reserves space while loading; heading font uses `font-display: optional` with preload, so it never swaps mid-read — Performance
- Primary button red darkened to `#cc2e2e` (white text 5.25:1, WCAG AA) — Accessibility
- Activity log: `role="log"` moved from the `<ol>` to a wrapper `<div>` (valid ARIA) — Accessibility
- Long single words (e.g. future translations) now wrap instead of being clipped; layout grid tracks can no longer exceed the viewport — Layout
- Lighthouse (mobile and desktop, all pages): Accessibility, Best Practices, SEO 100; 0 failed audits — QA

## 2026-10-08 (Discord)

- Added "Join Discord" buttons (hero and Getting started) and a Discord link in the footer of every page; invite https://discord.gg/aaUQVJeCgC (permanent) — Community
- Discord mark from Simple Icons 16.34.0 (CC0), used unmodified in brand color #5865F2 (white text 4.61:1) — Assets

## 2026-10-08 (CI automation)

- Added a post-deploy **verify** job: `scripts/smoke-test.js` checks the live version, pages, assets, CSP, Discord link and that private files are not published — CI/CD
- Added CodeQL scanning for the site JavaScript and the GitHub Actions workflows — Security
- Weekly health check (`Source_Code`: `.github/workflows/health.yml`) and Dependabot for GitHub Actions on both branches (`Source_Code`: `.github/dependabot.yml`); they live on the default branch because GitHub only runs schedules from there — Repository
- `doc-loader.js` only loads allow-listed branches and builds URLs from that constant list (resolves CodeQL `js/xss-through-dom`); `check-site.js` validates `data-branch` values — Security

## 2026-10-08 (Site review fixes)

- Added a custom 404 page with navigation, a home button and the Discord link (was GitHub's generic page) — UX
- Declared the site as dark (`color-scheme: dark`): scrollbars and built-in controls now match — UI
- Added a copyright line to the footer: © 2026 SpaceSquare640 · Licensed under GPL-3.0 — Legal
- CSS/JS now use content-hashed file names, and the previous deploy's files are carried over, so a page never loads mismatched files after a deploy (GitHub Pages ignores `?v=` query strings) — Reliability
- Security review fixes: carried-over files must match the hash in their name and are size-capped while streaming; the build fails on any unhashed CSS/JS reference; `check-site.js` only accepts published paths; the smoke test checks the 404 page's assets; the 404 page allows no network requests (`connect-src 'none'`) — Security

## 2026-10-08 (Change log policy)

- The tool's change log (`Source_Code`) now lists only player-visible changes; repository and CI changes are recorded here — Repository
