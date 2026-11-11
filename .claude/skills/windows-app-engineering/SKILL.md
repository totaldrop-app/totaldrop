---
name: windows-app-engineering
description: Use when working on TotalDrop for Windows (a future windows/ folder): WinUI 3 and C# on the core's C ABI, Fluent design, Explorer and share integration, MSIX packaging and the Microsoft Store.
---

# Windows (TotalDrop)

Apply `software-engineering-discipline` for process. The plan is decision 0016
(`docs/kararlar/0016-windows-microsoft-store.md`): WinUI 3 (Windows App SDK)
in C# with Native AOT, bound to the Rust core through a C ABI; shipped only
as an MSIX through the Microsoft Store, which signs it; winget installs the
same package from the `msstore` source. No other Windows download exists
unless a new decision adds one.

## Design

- Windows 11 Fluent from the toolkit itself: system accent, light, dark and
  high contrast, Segoe UI Variable, Fluent icons, Mica and title bar
  integration. The theme follows the system live, including the caption
  buttons (`docs/tasarim.md`, section 1). Only the mark keeps its own colours
  and motion.
- Keyboard first; text scaling up to 225 %; per-monitor DPI.
- Native AOT: XAML written for trimming (`x:Bind`, no reflection
  `{Binding}`).

## Integration

- Explorer context menu "Send with TotalDrop" through `IExplorerCommand`
  with package identity; share target and share source
  (`DataTransferManager`); toast notifications with actions.
- Start at login only by user choice; single instance with activation
  forwarding.

## Security, tests and release

- No admin rights at install or run; secrets in DPAPI or Credential Manager.
- Unit tests for logic and UI automation for the main flows and the shell
  entry points, x64 and ARM64, in CI.
- Screenshots in light, dark and high contrast, 100 % and 200 % scaling, LTR
  and RTL in the PR. Store listing and release notes in every supported
  language; the agent never handles the maintainer's Partner Center
  password.
