# Design rules

The full brand book lives in the TotalDrop design system. This file holds the binding rules to follow when writing code. Its public summary is the site's brand page (`site/public/brand/`, totaldrop.app/brand); when a rule for the mark, colour, type or buttons changes, that page and the downloadable files (`site/tools/brand-files.mjs --png`) are updated in the same PR.

## 1. Match the system one hundred percent

On every platform, TotalDrop looks like part of that system. Colour, the light or dark theme, the accent colour, the text scale, icons and window behaviour come from the system. The Aurora palette is used only when the system gives no colour at all. The only thing that does not change is the mark (Köşe) and its colours.

When you change the system theme, the app switches to the new theme without a restart.

| Environment | Colour source | Control style and icons | Window |
|---|---|---|---|
| Android 15 and later | Material You (dynamic_color) | Material 3; Expressive pieces as the official `material_ui` package brings them ([decision 0011](kararlar/0011-android-material-3-expressive.md)). Enabled today: the icon buttons (Settings, Paste, Remove, Back): pressing turns them into a rounded square, and with "Remove animations" on, the change is instant. The icon buttons in a text field and in a list row are wrapped with `KeepIconButtonVariant`; `material_ui` 1.8.0 drops the variant in these places ([#259](https://github.com/totaldrop-app/totaldrop/issues/259)) | System |
| Omarchy | `~/.local/state/omarchy/current/theme/colors.toml`; read again live when the theme changes | Fusion, colours from the theme | Hyprland draws it; the app draws no frame of its own |
| Hyprland (without Omarchy) | The Qt palette if there is one (hyprqt6engine or qt6ct); otherwise the GTK theme (`gtk.css` by the `gtk-theme` name, with `~/.config/gtk-4.0/gtk.css` on top), light/dark and the accent from the portal (the theme's own accent wins if it has one) | The system Qt style, otherwise Fusion in the GTK manner | Hyprland draws it |
| KDE Plasma | Plasma's Qt palette and `kdeglobals` (accent included) | `org.kde.desktop` (qqc2-desktop-style), Breeze icons | KWin draws it (server-side decorations) |
| GNOME and others | On GNOME the libadwaita palette, elsewhere the GTK theme; on both, `~/.config/gtk-4.0/gtk.css` on top. Light/dark and the accent from the portal; if `gtk.css` gives an accent, it wins. Read again live when the theme or `gtk.css` changes | Fusion in the GTK manner (buttons, fields and switches are drawn like GTK, icons in the text colour) | System |

After every change, this is checked with screenshots in all of these environments, in the light and the dark theme.

### Language and region

The language, the regional formats and the text direction also come from the system ([decision 0008](kararlar/0008-diller-ve-bolgeler.md)):

- The interface opens in the system's preferred language; if there is no translation, it falls back to the closest variant, and if there is none, to English.
- Numbers and sizes are formatted by the regional setting, plurals by the rules of the language shown. The code does not use `toStringAsFixed`, a hand-written decimal separator, or hand-written plurals such as "1 file / 2 files".
- In right-to-left languages (Arabic, Persian, Hebrew, Urdu) the layout is mirrored. Icons that show a direction (the back arrow) are mirrored; the mark (Köşe) and logos are not.
- Text boxes grow with long translations: in German and Finnish, text can be 30–40% longer than in English; no fixed width is given.
- The transfer code and the verification words are shown in English in every language and isolated left to right (in Flutter `Directionality(textDirection: ltr)` and FSI…PDI inside a sentence; in Qt kept out of `LayoutMirroring`). Automatic capitalisation and correction are off in the code field.
- File names from the other side enter a sentence with direction isolation; the directional control characters in them are made visible.

Besides the light and the dark theme, interface changes are tried in at least one right-to-left language (Arabic) and one CJK language (Japanese).

## 2. The mark: Köşe

A circle with one square corner. The circle is wholeness (*Total*); the square corner turns it into a drop (*Drop*) and shows the direction the file goes. The corner is also the corner of a file and of a device's screen. While a transfer runs, the mark turns quarter by quarter, and the corner points in each direction in turn: from every device to every device. The master file is `app/assets/logo.svg`; the Android icons are generated from it by `packaging/render-icons.sh`, the site's `favicon.ico` and `apple-touch-icon.png` by `site/tools/favicons.mjs`.

- **Construction (a 112-unit canvas):** a circle centred at (55, 57) with radius 42; its top-right quarter is filled with a square corner (corner at 97, 15). Path: `M55 15H97V57A42 42 0 1 1 55 15Z`. The circle is shifted one unit left and down from the centre of the canvas: because the corner side is heavier, the shape looks centred this way.
- **Colours:** purple (#6D4AE0) on a light background, bright purple (#A68BFF) on a dark background. A single colour; no shadow, gradient or outline.
- **App icon:** the mark in paper colour (#F4F2FB) on a purple background; the mark is half the width of the tile (0.68 scale: the Linux 128-unit icon, the legacy Android icon, the Play icon). In the Android adaptive icon, the mark sits in the centre at 0.44 scale: the corner (the point farthest from the centre) is 25.5 dp from the centre, staying 7.5 dp inside the 66 dp safe circle and 10.5 dp inside the 72 dp circle the launcher shows; in no mask does the tip look as if it touches the edge ([#261](https://github.com/totaldrop-app/totaldrop/issues/261)). White in the monochrome layer. The symbolic icon is the mark alone.
- **Done badge:** a paper-coloured check mark on the circle of the resting mark (in the light theme; ink in the dark theme), stroke width 9.
- **Clear space and minimum size:** at least half the radius (21) of clear space on every side; the minimum size is 16 px. No separate drawing is needed for small sizes.
- **Never:** round the corner, turn the circle into an ellipse, put the mark inside a frame or ring, paint it another colour, or spin it while resting.
- **Direction:** not mirrored in right-to-left languages; at rest the corner is always at the top right.

### Motion

While a transfer runs, the mark turns a quarter turn clockwise around the circle's centre (55, 57) and pauses; after four quarters it is back where it started. The website (`site/public/styles.css`, `mk-turn`), Android (`mark.dart`, `MarkMotion`) and Linux (`Mark.qml`) use the same values:

- **Cycle:** 3.2 seconds, endless.
- **Quarters:** 0.8 seconds each: a 90° turn of 0.6 seconds on the `cubic-bezier(0.65, 0, 0.35, 1)` curve, then a 0.2 second pause.
- **Reduce motion:** if this setting is on in the system, the mark stays at rest.

In the hero section of the website, around the mark and outside its clear space, a generic device line icon stands in each direction the corner points to at the end of each quarter: a desktop at the top right (the resting direction), a phone at the bottom right, a laptop at the bottom left, a tablet at the top left. When the corner reaches a direction, that icon turns to the accent colour (`dv-lit`, the same 3.2 second cycle). The icons are not part of the mark: the mark's path, colour and motion do not change, and the apps have no icons. No platform or manufacturer logos (Apple, Android, Windows) are used. With reduce motion on, the icons stay equal and still. They are decoration and are hidden from screen readers. The icons do not count as a frame or a ring: they do not surround the mark, are not connected to one another, stand separately in the directions the corner points to, and stay outside the mark's clear space.
