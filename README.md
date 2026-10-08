# Minecraft Auto Fishing — Website Preview

An interactive website that introduces the Minecraft Auto Fishing project visually,
so players can understand how it works without reading long text documentation.

> **Status:** In development. The auto-fishing tool itself is not released yet;
> the website describes planned behavior only.

## Branches

| Branch | Purpose |
|---|---|
| `Source_Code` (default) | Source code of the auto-fishing tool |
| `Minecraft_Auto_Fishing_Website_Preview` | This website |

## Tech

Plain HTML, CSS and JavaScript, no dependencies. The heading font (*Press Start 2P*, SIL OFL 1.1)
is self-hosted, so the site makes no third-party requests except reading docs from GitHub.

A Content-Security-Policy `<meta>` on every page allows only same-origin scripts, styles, images and fonts,
and network requests only to the site itself and `raw.githubusercontent.com`. Because of this, pages must not
contain inline `<script>`, `<style>` or `style` attributes (`scripts/check-site.js` enforces it).

## Pages

| Page | Content |
|---|---|
| `index.html` | Landing page: hero, interactive fishing demo (manual vs. auto), how it works, features, getting started, FAQ |
| `docs.html` | Renders `docs/overview.md` from the `Source_Code` branch |
| `changelog.html` | Renders the tool's `Change Log.md` (`Source_Code`) and this branch's `Change Log.md` |

Docs and changelogs are fetched at runtime from `raw.githubusercontent.com`, so the repository stays the single source of truth. Push the `Source_Code` branch for doc changes to appear (GitHub caches raw files for about 5 minutes).

## Structure

```text
index.html
docs.html
changelog.html
css/
  tokens.css        Design tokens (colors, fonts, spacing)
  base.css          Reset, layout, responsive rules, reduced-motion
  components.css    Badge, buttons, hero icon, site navigation, footer, back-to-top
  prose.css         Typography for rendered Markdown
  demo.css          Demo scene, mode lever, stats, activity log
  sections.css      How it works, features, getting started, FAQ, scroll reveal
  fonts.css         @font-face for the self-hosted Press Start 2P
js/
  i18n.js           Fills data-i18n / data-i18n-attr from window.LOCALES
  main.js           Page bootstrap: i18n, scroll reveal, back-to-top
  markdown.js       Minimal, escape-first Markdown renderer (no raw HTML)
  doc-loader.js     Fetches Markdown from the repo into [data-doc] elements
  demo-engine.js    Demo rules as a DOM-free state machine (timings in CONFIG)
  demo.js           Connects the engine to the page; pauses when off screen
locales/
  en.js             English strings
tests/
  markdown.test.js  Renderer and XSS tests (node --test)
  demo-engine.test.js  Demo rules with a fake clock
scripts/
  check-site.js     i18n keys, file references, anchors, CSP compliance
  build.js          Builds _site/ for Pages: prerenders English text, stamps ?v=<sha> on CSS/JS URLs
  smoke-test.js     Live checks of the deployed site (deploy / weekly health modes)
.github/workflows/
  pages.yml         Test -> build -> deploy -> verify (GitHub Pages)
  codeql.yml        CodeQL scanning of the site JS and the workflows
assets/img/
  icon.webp             Project icon, transparent, 720px WebP (~45 KB)
  favicon.ico           Browser tab icon (16 / 32 / 48px)
  apple-touch-icon.png  iOS home-screen icon (180px, opaque)
  og-image.png          1200x630 social preview (Open Graph / Twitter)
assets/fonts/
  PressStart2P-*.woff2, OFL.txt
sitemap.xml
```

## Preview locally

`index.html` can be opened directly in a browser. `docs.html` and `changelog.html` fetch Markdown, so serve them over HTTP.

To preview **unpushed** docs, serve the parent folder (the folder names match the branch names) and pass `?source=` — this override only works on `localhost` / `127.0.0.1`:

```bash
cd ..
python -m http.server 8000
```

Then open http://localhost:8000/Minecraft_Auto_Fishing_Website_Preview/docs.html?source=http://localhost:8000/

## Tests and checks

```bash
node --test tests/*.test.js
node scripts/check-site.js
node scripts/build.js        # optional: builds _site/ locally (git-ignored)
```

Local builds stamp `?v=dev`, which the browser may cache. When testing CSS/JS changes in `_site/`,
set a unique version, e.g. `ASSET_VERSION=abc1234 node scripts/build.js`.

## CI/CD

`.github/workflows/pages.yml` runs on every push and pull request to this branch:

1. **Test** — unit tests and site checks (read-only permission).
2. **Build** — `scripts/build.js` copies an allowlist of files to `_site/` (tests, scripts and
   `.github/` are never published), writes the English text from `locales/en.js` into the HTML
   (no layout shift when JS runs; readable without JS and by crawlers) and appends `?v=<commit>`
   to CSS/JS URLs so a deploy never mixes new HTML with cached old files. Push only.
3. **Deploy** — publishes `_site/` with `actions/deploy-pages`. Push only, after tests pass.
4. **Verify** — `scripts/smoke-test.js deploy` waits for the new version on the live site, then checks
   every page, assets, the CSP tag, the Discord link, and that tests/scripts/README are not published.

Other automation:

| What | Where | When |
|---|---|---|
| CodeQL (JavaScript + GitHub Actions) | `codeql.yml` on this branch | every push / PR |
| Weekly health check (live site, external links, Discord invite) | `.github/workflows/health.yml` on `Source_Code` — runs this branch's `smoke-test.js health` | Mondays 01:17 UTC, or manually |
| Dependabot (keeps pinned action SHAs current on both branches) | `.github/dependabot.yml` on `Source_Code` | weekly, 7-day cooldown, opens PRs only |

Scheduled workflows and the Dependabot config must live on the default branch (`Source_Code`); GitHub ignores them elsewhere.

Security rules for the workflow:

- `permissions: {}` by default; each job opts into the minimum it needs.
- Only GitHub-owned actions, pinned to full commit SHAs (tag in the trailing comment).
- `persist-credentials: false`; no `pull_request_target`; no secrets; no package installs;
  no untrusted `${{ }}` expressions inside `run:`.
- Before changing the workflow, run `actionlint` and `zizmor --persona=pedantic` (both online
  and `--offline`); all must report zero findings.
## Adding a language

1. Copy `locales/en.js` to `locales/<lang>.js` and translate the values
   (keep the keys unchanged; set `window.LOCALES["<lang>"]`).
2. Add `<script src="locales/<lang>.js" defer></script>` to every page (`index.html`, `docs.html`, `changelog.html`)
   before `js/i18n.js`.
3. Call `I18n.apply("<lang>")`. Missing keys fall back to English.

## Credits

UI components adapted from [Uiverse.io](https://uiverse.io) (MIT):
`elijahgummer/kind-pig-24` (badge), `kamehame-ha/kind-otter-31` (GitHub button),
`chase2k25/rare-quail-40` (demo mode lever), `Yaya12085/grumpy-fox-39` (feature cards),
`vinodjangid07/afraid-falcon-17` (back-to-top button).
Discord mark from [Simple Icons](https://simpleicons.org) 16.34.0 (CC0).

## License

GPL-3.0. See `LICENSE`.
