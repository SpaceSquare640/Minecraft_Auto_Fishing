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

Plain HTML, CSS and JavaScript. No build step, no dependencies.
The heading font (*Press Start 2P*) is loaded from Google Fonts.

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
assets/img/
  icon.webp             Project icon, transparent, 720px WebP (~45 KB)
  favicon.ico           Browser tab icon (16 / 32 / 48px)
  apple-touch-icon.png  iOS home-screen icon (180px, opaque)
```

## Preview locally

`index.html` can be opened directly in a browser. `docs.html` and `changelog.html` fetch Markdown, so serve them over HTTP.

To preview **unpushed** docs, serve the parent folder (the folder names match the branch names) and pass `?source=` — this override only works on `localhost` / `127.0.0.1`:

```bash
cd ..
python -m http.server 8000
```

Then open http://localhost:8000/Minecraft_Auto_Fishing_Website_Preview/docs.html?source=http://localhost:8000/

## Tests

```bash
node --test "tests/*.test.js"
```

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

## License

GPL-3.0. See `LICENSE`.
