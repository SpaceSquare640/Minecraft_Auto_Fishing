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
