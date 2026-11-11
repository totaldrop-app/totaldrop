---
name: web-design-pro
description: "Use when changing the TotalDrop website (site/, totaldrop.app): pages, styles, translations of the site, the brand page, accessibility, performance and SEO. For site/functions/ and site/public/_headers also load fullstack-engineering and security-engineering-analyst."
---

# Web design (TotalDrop site)

Work as a senior web designer who also ships the HTML and CSS. Apply
`software-engineering-discipline` for process. `docs/tasarim.md` is binding
(Dieter Rams: less but better); `site/README.md` explains how the site is
built.

## Rules that are easy to miss

- **Every language.** User-facing text ships in every language the site
  supports, never only Turkish and English. After changing a text, search
  every language file for it. `site/tools/check-i18n.mjs`,
  `check-readmes.mjs` and `privacy.mjs` check structure and numbers, not
  meaning.
- **The mark's motion** is the same everywhere: `mk-turn` in
  `site/public/styles.css`, 3.2 s loop, four 0.8 s quarters (0.6 s turn with
  `cubic-bezier(0.65, 0, 0.35, 1)`, 0.2 s hold), still under
  `prefers-reduced-motion` (`docs/tasarim.md`, "Motion").
- **Brand changes** update the brand page (`site/public/brand/`) and its
  downloads (`site/tools/brand-files.mjs --png`) in the same PR.
- **No third parties.** No trackers, external fonts, CDNs or inline
  script/style; the CSP in `site/public/_headers` stays strict
  (`script-src 'self'`, `style-src 'self'`).
- **Claims match the code.** A size, limit, duration or price on the site is
  backed by code and a test (`docs/surec.md`, "Söz ile kod uyuşmuyor").
- **Asset versions:** after changing `styles.css`, `app.js`, `lang.js` or
  `msg.js`, run `node tools/asset-versions.mjs --write` and
  `node tools/privacy.mjs --write` in `site/`, or visitors keep stale files
  (`site/README.md`).
- **Download links** change with every release, in every language; check
  each link after the change.

## Craft

RTL mirroring, LTR codes and light/dark come from `docs/tasarim.md`; the
other numbers here (320 px, 44 px, Web Vitals budgets) are recommended
practice, not yet written there.

- Design tokens in `:root`; light and dark with `prefers-color-scheme`; no
  hard-coded colours in components. WCAG AA contrast; colour never the only
  signal.
- Mobile first, no horizontal scroll at 320 px; CSS logical properties so
  right-to-left languages mirror; codes and comparison words isolated LTR
  (`dir="ltr"`, `<bdi>`). The mark is never mirrored.
- Semantic HTML first, ARIA only to fill gaps; visible focus; touch targets
  at least 44 px; language picker with native names, never flags.
- Essential content works without JavaScript. Core Web Vitals budgets: LCP
  < 2.5 s, INP < 200 ms, CLS < 0.1.

## Review and delivery

Screenshots in the PR in light and dark, in at least one RTL (Arabic) and
one CJK (Japanese) language (`docs/tasarim.md`); phone (390 px) and desktop
(1280 px) widths are the usual choice. Validate HTML and check every link
and asset exists.
