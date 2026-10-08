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

## Structure

```text
index.html
css/
  tokens.css        Design tokens (colors, fonts, spacing)
  base.css          Reset, layout, responsive rules, reduced-motion
  components.css    Badge, buttons, item frame
js/
  i18n.js           Fills data-i18n / data-i18n-attr from window.LOCALES
  main.js           Page bootstrap
locales/
  en.js             English strings
assets/img/
  icon.jpg          Project icon
```

## Preview locally

Open `index.html` directly in a browser — no server required.

Alternatively, serve the folder:

```bash
python -m http.server 8000
```

Then visit http://localhost:8000.

## Adding a language

1. Copy `locales/en.js` to `locales/<lang>.js` and translate the values
   (keep the keys unchanged; set `window.LOCALES["<lang>"]`).
2. Add `<script src="locales/<lang>.js" defer></script>` to `index.html`
   before `js/i18n.js`.
3. Call `I18n.apply("<lang>")`. Missing keys fall back to English.

## Credits

UI components adapted from [Uiverse.io](https://uiverse.io) (MIT):
`elijahgummer/kind-pig-24` (badge), `kamehame-ha/kind-otter-31` (GitHub button).
