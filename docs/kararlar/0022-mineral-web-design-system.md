# 0022: Mineral surface and colour system for the website

- **Status:** Accepted, 15 November 2026 (Ozan, #529)
- **Context:** On the website (`site/public/styles.css`), the initial `Aurora`
  palette painted the page ground (`#F4F2FB` / `#140E33`), raised surfaces
  (`#E2DCF6` / `#1E1648`), text (`#22184F` / `#B9B1E0`), buttons, links, and
  the **Köşe** mark in variations of the same violet hue. That created two
  problems:

  1. Because every surface, heading, button, link, and border carried violet,
     the **Köşe** mark (`#6D4AE0` light / `#A68BFF` dark) lost figure-ground
     separation against the page, and dark mode relied heavily on purple text
     and purple fills.
  2. The shipped Android and Linux apps (`docs/assets/android.png`,
     `docs/assets/linux.png`) take neutral surfaces and calm system accents
     from Material You, Plasma, Hyprland, Omarchy, and GNOME, leaving the
     violet **Köşe** mark as the single unmistakable brand anchor. The website
     looked visually unrelated to the real apps.

- **Options considered:**

  1. *Keep the monochromatic violet wash (`Aurora`) on the website.* No token
     changes, but purple-on-purple surfaces remain visually heavy and diverge
     from how the apps actually look.
  2. *Recolour the **Köşe** mark itself.* Rejected: `docs/tasarim.md` fixes
     the mark's geometry and colours (`#6D4AE0` / `#A68BFF`) across app icons,
     favicons, and downloadable lockups.
  3. *Introduce `Mineral` for the website's grounds, cards, and interactive
     accents while keeping **Köşe** untouched.* Neutral Alabaster/Paper/Stone
     (`#F7F7F5` / `#FFFFFF` / `#EEEEEC`) in light mode and
     Obsidian/Graphite/Slate (`#0F1116` / `#161922` / `#1E222E`) in dark mode,
     paired with Cobalt (`#1E47B8`) and Ice Cobalt (`#7DB4FF`) for actions,
     links, and focus rings, and `--mark` (`#6D4AE0` / `#A68BFF`) reserved
     exclusively for **Köşe**.

- **Decision:** Option 3.

  - `site/public/styles.css` defines `--mark` separately from `--accent` and
    adds `--surface` alongside `--bg` and `--bg-raised` so cards, transfer
    specimens, app frames, and tables stand apart cleanly with 1 px `--rule`
    hairlines and zero shadows, gradients, or blur.
  - Every text and interactive token meets WCAG AA (and primary/secondary text
    and accents exceed 7:1 AAA on `--bg` in both themes).
  - `docs/tasarim.md` and `site/public/brand/index.html` document `Mineral`
    and its verified contrast ratios, while keeping `Aurora` documented as the
    built-in app fallback when an operating environment exposes no system
    palette. Downloadable mark and logo lockups (`site/tools/brand-files.mjs`)
    and favicons (`site/tools/favicons.mjs`) stay unchanged.

- **Consequences:**

  - The **Köşe** mark stands out immediately in both light and dark themes,
    and the website matches the calm, native appearance of the shipped apps.
  - Dark mode uses neutral obsidian/graphite surfaces and crisp ice-cobalt
    interactive controls instead of purple washes.
