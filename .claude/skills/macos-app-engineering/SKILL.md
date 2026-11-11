---
name: macos-app-engineering
description: "Use when working on TotalDrop for the Mac (a future macos/ folder): UI toolkit choice, Mac Human Interface Guidelines, Finder and Share menu integration, sandbox, ad-hoc and notarized builds on GitHub macOS runners."
---

# macOS (TotalDrop)

Apply `software-engineering-discipline` for process; share code with
`ios-app-engineering` where it fits, but design for the Mac. The plan is
decision 0009 (`docs/kararlar/0009-ios-macos.md`): the core is built for
`aarch64-apple-darwin` and `x86_64-apple-darwin` (one `lipo` binary) on
GitHub's macOS runners; until the Apple account exists, CI ships an ad-hoc
signed, not notarized package and the download page says how to open it.
CI uses a pinned image (for example `macos-15`) and Xcode, never
`macos-latest`.

## Design

- A Mac app: standard menus and shortcuts, Settings window (⌘,), resizable
  windows, keyboard navigation, drag and drop in and out, context menus.
- System colours and accent, light/dark, increased contrast, reduced
  transparency and motion (`docs/tasarim.md`, section 1). Only the mark keeps
  its own colours and motion.
- Language and region from the system, including the per-app language
  (decision 0008).

## Integration

- Share menu extension and a Finder Quick Action/Service "Send with
  TotalDrop" for files, folders and selected text; drop onto the Dock icon.
- `NSOpenPanel`/`NSSavePanel` and security-scoped bookmarks; App Sandbox with
  the narrowest entitlements (network client and server, user-selected
  files); Hardened Runtime.
- Login item only when the user turns it on (`SMAppService`).

## Tests and release

- Unit and UI tests on the oldest supported and the newest macOS in CI.
- With the account: Developer ID signing, `notarytool submit --wait`,
  `stapler staple`, verified with `spctl -a -vv`; keys only in CI secrets.
  The agent never handles the maintainer's Apple ID.
