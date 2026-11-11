---
name: android-app-engineering
description: "Use when changing the TotalDrop Android app (app/, Flutter with the Rust core through flutter_rust_bridge): Material 3 and Material You, background transfers, share and \"Send with TotalDrop\" entry points, permissions, translations, tests and Play release builds."
---

# Android (TotalDrop)

Apply `software-engineering-discipline` for process. Binding: `docs/tasarim.md`
(Android row of the table in section 1), decisions 0010 (background
transfers) and 0011 (Material 3 Expressive) in `docs/kararlar/`, and the
rules in `app/README.md`.

## Architecture rules

- The app is a thin interface. No cryptography and no networking in Dart:
  keys, hashes, signatures and every connection go through the Rust core
  (`app/rust`, `flutter_rust_bridge`). Generated glue (`app/lib/src/rust/`,
  `frb_generated.rs`) is regenerated, never edited by hand.
- No analytics, crash reporters or advertising SDKs.
- Errors from the core arrive with a `cause` name that Dart translates and
  an English `detail` shown only under "more information"
  (`app/rust/src/api/transfer.rs`); no other raw error text reaches the UI.
- The mark's motion during a transfer is the site's, with the same values
  (`mark.dart`, `MarkMotion`; `docs/tasarim.md`, "Hareket").
- Transfer stages are an enum. After every `await`, check that generation and
  stage are unchanged before acting; an action runs only in the stage where
  it is valid (`docs/surec.md`, "Örtük durum makinesi").

## Platform

- Minimum Android 15 (`minSdk 35`); target the newest SDK the build uses.
- Background: a user-initiated job keeps process and network alive while a
  transfer runs, with a notification and a Stop button (decision 0010).
  Permissions for it: `RUN_USER_INITIATED_JOBS`, `POST_NOTIFICATIONS` (asked
  at the first transfer; the transfer works without it),
  `ACCESS_NETWORK_STATE`; the app as a whole also needs `INTERNET`
  (`app/android/app/src/main/AndroidManifest.xml`).
- Material You colours (`dynamic_color`) and Material 3; Expressive parts only
  as the official `material_ui` package provides them; icon buttons in text
  fields and list rows wrapped with `KeepIconButtonVariant` (#259).
- Shared items arrive as `content://` URIs. Only URIs served by another
  app's provider are read (never `file://` or a provider in TotalDrop's own
  process, which could expose the device key); each file is copied before
  the core reads it, and the grant is never made persistent
  (`app/android/app/src/main/kotlin/app/totaldrop/totaldrop/SharedFiles.kt`).

## Language and accessibility

- Strings in `l10n/app/` for every supported language; plurals and numbers
  through ICU and the locale, never `toStringAsFixed` or hand-made plurals.
- RTL mirrors; transfer code and comparison words stay LTR
  (`Directionality(textDirection: TextDirection.ltr)`, FSI…PDI in
  sentences); file names from the other side are direction-isolated with
  control characters made visible.
- TalkBack labels, 200 % font size without clipping, touch targets 48 dp.

## Tests and checks

- `flutter analyze`, `dart format` and `flutter test` clean (run
  `flutter pub get` in `app/` first).
- Widget tests draw every state: full list, long name, largest font, light
  and dark, Arabic and Japanese. Screenshots in the PR.

## Release

- Bump `version` with the build number (`+N`, higher on every upload) in
  `app/pubspec.yaml`, and the same version without `+N` in
  `app/rust/Cargo.toml` and `Cargo.lock` (`docs/surec.md`, "Sürüm listesi").
- Built only in CI (`.github/workflows/android-release.yml`); it produces the
  `.aab` as an artifact and does not upload to Play. The maintainer uploads
  and presses "Publish".
- `key.properties` and the upload key are never read or committed. A
  debug APK is signed with another key, so it does not install over the
  Play build.
- Play release notes in every supported language.
