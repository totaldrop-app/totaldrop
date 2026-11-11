---
name: ios-app-engineering
description: "Use when working on TotalDrop for iPhone and iPad (app/ios/ or a future ios/ folder): UI toolkit choice, Human Interface Guidelines, share extension, background transfers, privacy manifest, CI builds on GitHub macOS runners and App Store release."
---

# iOS and iPadOS (TotalDrop)

Apply `software-engineering-discipline` for process. The plan is decision
0009 (`docs/kararlar/0009-ios-macos.md`): build and test on GitHub's free
macOS runners; the Apple Developer account is needed only for release. The
UI toolkit (SwiftUI or the Flutter app's iOS target) is a separate decision
record, judged by fit with the system and its integrations. CI uses a
pinned image (for example `macos-15`) and a pinned Xcode, never
`macos-latest`.

## Design

- The app looks like part of iOS (`docs/tasarim.md`, section 1): system
  navigation, sheets, SF Symbols, Dynamic Type, system colours, light and
  dark, increased contrast, reduced motion. Only the mark keeps its own
  colours and motion.
- Language and region from the system (decision 0008); RTL mirrors through
  leading/trailing; transfer code and comparison words stay LTR.

## Engineering

- The Rust core behind a typed FFI; errors arrive as Swift `throws` with the
  core's `Cause` keys, translated in the app.
- Transfers survive the app leaving the screen only through the APIs made for
  it (background `URLSession` is not usable for QUIC; record what is used and
  why in a decision). Never rely on staying alive.
- Share extension for files, photos, text and URLs ("Send with TotalDrop"),
  small memory footprint; local network permission string explained in
  every language. Local discovery uses Bonjour; raw multicast needs an
  entitlement Apple must grant (decision 0009).
- Keychain for secrets (`…ThisDeviceOnly` or stricter); privacy manifest
  (`PrivacyInfo.xcprivacy`) and App Privacy labels match the privacy policy;
  no tracking.

## Tests and release

- Swift Testing/XCTest for logic, XCUITest for the main flows and the
  extension, on the oldest supported and the newest iOS, iPhone and iPad
  simulators in CI.
- Signing certificates and App Store Connect keys only in CI secrets; the
  agent never handles the maintainer's Apple ID. TestFlight before release;
  export compliance answered for standard encryption.
- Store listing, screenshots and release notes in every supported language.
