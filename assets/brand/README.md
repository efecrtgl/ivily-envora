<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="logos/ivily-envora-horizontal-on-dark.svg">
    <img src="logos/ivily-envora-horizontal-on-light.svg" height="64" alt="Ivily Envora">
  </picture>
</p>

# Ivily Envora — brand assets

Official logos, icons and visuals for Ivily Envora, part of the Ivily ecosystem.
The full brand guide (rationale, construction, colour, typography, usage rules) lives in the Ivily design workspace; this folder contains everything needed to use the brand from this repository.

<p align="center"><img src="previews/ivily-envora-brand-sheet.png" width="760" alt="Ivily Envora brand sheet"></p>

## Identity in one paragraph

**Symbol — key = value rows that form an e.** Environment configuration is, at heart, a list of **KEY = VALUE** rows. The symbol draws three of them hanging from a shared spine, which together form an **E** for Envora.

**Signal colour — Vault Jade:** `#3DD9A6` on dark, `#067A56` on light. Vault Jade marks *protected and healthy* states: masked values, verified backups, successfully applied changes, the active profile. Do not use it for generic buttons.

## Contents

| Path | What | Use |
|---|---|---|
| `logos/ivily-envora-horizontal-*.svg` | **Primary logo** (symbol + "Ivily Envora") | READMEs, websites, documents |
| `logos/ivily-envora-vertical-*.svg` | Stacked logo | Splash screens, square placements |
| `logos/ivily-envora-logotype-*.svg` | Name only | Next to an existing symbol |
| `logos/ivily-envora-symbol-*.svg` | Symbol only | Title bars, favicons, tight spaces |
| `logos/png/` | Transparent PNG renders | Where SVG is not accepted |
| `icons/ivily-envora-app-icon.svg` | Application icon master (1024) | Source for every icon size |
| `icons/ivily-envora-app-icon-small.svg` | Optical variant for 16–32 px | Small icon sizes |
| `icons/ivily-envora-app-icon-light.svg` | Light container variant | Light marketing contexts |
| `icons/png/` | App icon PNG 16 → 1024 | Stores, websites, docs |
| `icons/ivily-envora.ico` | Windows icon (16, 24, 32, 48, 64, 128, 256) | Installer, shortcuts |
| `icons/tauri/` | Drop-in set with `src-tauri/icons/` file names | Adopting the icon in the app |
| `banners/ivily-envora-readme-banner-{dark,light}.svg` | README header, 1600 × 560 | This repository's README |
| `banners/ivily-envora-social-preview.png` | 1280 × 640 | GitHub → Settings → Social preview |
| `illustrations/ivily-envora-product-visual-*.{svg,png}` | Product visual — **illustration, not a screenshot** | Storytelling, docs |
| `patterns/ivily-envora-pattern-tile-*.svg` | Seamless tile | Backgrounds at 4–8 % opacity |
| `ecosystem/` | Ivily parent mark, ecosystem lineup and diagram | "Part of Ivily" sections |
| `palette.json`, `tokens.css` | Colour and type tokens | Web pages, future UI work |

Colourway suffixes: `-on-dark`, `-on-light`, `-mono-white`, `-mono-black`.

## Using the logo in Markdown

```html
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/brand/logos/ivily-envora-horizontal-on-dark.svg">
  <img src="assets/brand/logos/ivily-envora-horizontal-on-light.svg" height="48" alt="Ivily Envora">
</picture>
```

## Adopting the new application icon

The application currently ships its existing icon (`src-tauri/icons/`) and its in-app logo is unchanged. Switching is a deliberate product decision. When ready:

```powershell
Copy-Item assets/brand/icons/tauri/* src-tauri/icons/ -Force
# or regenerate every platform size from the master:
npx tauri icon assets/brand/icons/ivily-envora-app-icon.svg -o src-tauri/icons
```

Then rebuild and check the taskbar, installer and Start-menu icons at 100 % and 150 % scaling.

## Rules

- Keep clear space of ¼ of the symbol height around every logo. Minimum symbol size 16 px.
- Use the colourway that matches the background; never recolour, outline, add effects, rotate or stretch.
- Only Vault Jade may appear as a colour in Ivily Envora brand materials.
- Product visuals are illustrations and must be labelled as such.

---

<sub>Generated from the Ivily design workspace. Typefaces in logos: Instrument Sans and JetBrains Mono (SIL Open Font License 1.1), converted to outlines.</sub>
