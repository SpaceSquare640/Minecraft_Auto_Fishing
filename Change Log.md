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
