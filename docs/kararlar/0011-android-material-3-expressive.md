# 0011: Android moves to Material 3 Expressive step by step, with the official package

- **Status:** Accepted, 4 November 2026 (#215)
- **Context:** The Android app uses standard Material 3
  (`useMaterial3: true`, `app/lib/ui/core/theme.dart`). The colours come
  from Material You (`dynamic_color`, `app/lib/main.dart`). With Android
  16's QPR1 update (September 2025), Google's system interface (Pixel)
  moved to Material 3 Expressive; other manufacturers' interfaces follow at
  their own pace. The Expressive design system brings larger and variable
  corners, shape morphing, spring motion, button groups and a new loading
  indicator.
  The [design rules](../tasarim.md) ask for the app to look like part of the
  system on every platform. So we need to decide whether, and how, to adopt
  Expressive.

  The situation in Flutter (4 November 2026):
  - The Flutter team is not developing Expressive inside the framework. New
    Expressive work will be done in packages split out of the framework
    (`material_ui`)
    ([flutter/flutter#168813](https://github.com/flutter/flutter/issues/168813),
    update of 29 July 2025; the issue is open). In June 2026 the team said
    it would share the details of the Expressive plan once the split is
    finished.
  - The app already uses `material_ui` (`^1.8.0`, latest version 1.10.0).
    The package added the `StyleVariant.material3Expressive` option
    (1.2.0), but its own documentation says this support is "in
    development" and that the look may change between versions once the
    option is on. What has arrived so far: the Expressive `IconButton`
    (1.5.0) and the shape-morphing `MaterialShapeBorder` and
    `MaterialShapes` (1.8.0); both are in the app's locked version (1.8.0).
    The motion scheme, the loading indicator and button groups are not in
    the changelog yet
    ([changelog](https://pub.dev/packages/material_ui/changelog)).
  - In the community there are packages that rewrite the whole component
    set (`material_3_expressive`, `material_expressive`). They are
    maintained by individual developers, not by Google or the Flutter
    team.

  The app uses few components: `FilledButton`, `OutlinedButton`,
  `TextButton`, `IconButton`, `AppBar`, `Card`, `AlertDialog`, `TextField`,
  `ListTile`, `ExpansionTile`, `SwitchListTile`, `Divider`, `SnackBar`,
  `Chip`, `LinearProgressIndicator`. The theme is deliberately shadowless:
  surfaces are separated by colour and a thin line, and every elevation is
  0.

- **Options:**
  1. **Stay on standard Material 3.** No extra work, but on devices with an
     Expressive interface the app looks behind the system. It contradicts
     the "match the system one hundred percent" rule.
  2. **Move now with a community package.** Every Expressive component
     arrives today. But the components replace Flutter's: the correctness
     of behaviour such as accessibility, right-to-left layout and the
     system text scale is left to a single maintainer. When the official
     package brings these components, a second migration is needed. In a
     security-focused app, tying the whole interface to a small dependency
     also increases the supply-chain risk.
  3. **Write our own Expressive components.** Adds no dependency, but the
     maintenance and the accessibility testing fall to us. When the
     official components arrive, this code is thrown away too.
  4. **Move step by step with the official `material_ui` package.** As the
     package brings an Expressive piece, we turn it on. For the pieces it
     does not bring, we stay on standard Material 3.

- **Decision:** Option 4.
  - `material_ui` is kept up to date (Dependabot PRs). The changelog of
    every new version is read for Expressive items.
  - Each Expressive piece the package brings (today `IconButton` and
    `MaterialShapes`, both in 1.8.0) is turned on in its own issue and PR.
    At every step, screenshots are taken on Android in the light and the
    dark theme, with Material You colours and with the fallback Aurora
    palette, in left-to-right and right-to-left languages.
  - `StyleVariant.material3Expressive` is not turned on for the whole app
    until the package declares it stable. Before it is turned on, which
    components it changes is verified from the package's source.
  - Community Expressive packages are not added. We do not write an
    Expressive component ourselves because the package has not brought it
    yet.

- **Why:** It keeps the app on Flutter's official route; as components
  arrive, it moves closer to the system's look by itself. The interface's
  accessibility, right-to-left layout and text-scale behaviour stay in
  code that the Flutter team tests. No new dependency is added and no
  second migration is needed. The pieces still waiting may look missing,
  but they do not look broken: standard Material 3 still looks like part
  of the system on devices with an Expressive interface.

- **What does not change:**
  - The mark (Köşe), its colours and its spin during a transfer do not
    change ([design rules](../tasarim.md), "Motion"). Expressive's loading
    indicator or wavy progress bar does not replace the mark. If the
    progress bar arrives, it may only replace today's
    `LinearProgressIndicator`.
  - The shadowless theme stays: elevations stay 0, and surfaces are
    separated by colour and a thin line. Expressive defaults that use
    shadows or surface tint are adjusted to this rule.
  - The typefaces are the brand's (Bricolage Grotesque and Atkinson
    Hyperlegible); the size follows the system text scale.
  - Every new Expressive motion, like the mark, respects the system's
    reduce-motion setting ("Remove animations" on Android; "Reduce
    motion" in the design rules).

- **Consequences:**
  - The Android line in `docs/tasarim.md` is tied to this decision.
  - Issues for new Expressive steps are opened as `material_ui` versions
    come out. The first candidate: trying the Expressive `IconButton`,
    which is already in the locked version (1.8.0). `material_ui` upgrades
    arrive as separate PRs through Dependabot.
  - Linux, the site and the other platforms are not affected by this
    decision.

- **Review condition:** When `material_ui` brings the motion scheme, the
  loading indicator and button groups, or declares its Expressive support
  stable. It is also looked at again if the Flutter team changes its
  Expressive plan.
