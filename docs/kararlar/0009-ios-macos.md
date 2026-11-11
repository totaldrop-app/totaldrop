# 0009: iOS and macOS with free CI; an Apple account only for release

- **Status:** Accepted, 28 October 2026
- **Context:** The project runs on donations alone and we do not want to
  spend money. The iOS and macOS apps need Xcode; Xcode only runs on macOS,
  while the development machine runs Linux. Signing and Apple notarization
  for the App Store, TestFlight and distribution outside the store need an
  Apple Developer account at USD 99 a year. The macOS app can be
  distributed without an account, but installing it is cumbersome.
- **Options:**
  1. A rented cloud Mac. A monthly fee; not needed.
  2. **Build, test and package on GitHub's standard macOS runners (free for
     an open-source repository, with Xcode and the simulators installed);
     the Apple account only when it comes to release.**
  3. Not start iOS and macOS at all until donations cover the account.
- **Decision:** Option 2.

## Order

1. The core is built in CI for iOS (`aarch64-apple-ios`,
   `aarch64-apple-ios-sim`) and macOS (`aarch64-apple-darwin`,
   `x86_64-apple-darwin`; one package with `lipo`); the tests run on macOS.
   No account needed.
2. A separate decision for the interface: a native app in SwiftUI, or the
   iOS/macOS target of the existing Flutter app. The criteria are how well
   it matches the system look (Dieter Rams' principles,
   [tasarim.md](../tasarim.md)) and system integrations such as the share
   menu.
3. Simulator tests in CI. No account needed.
4. macOS: CI produces an ad-hoc signed but not notarized package (on Apple
   Silicon, code with no signature at all does not run; the compiler adds
   the ad-hoc signature itself). Since macOS 15 there is no right-click >
   Open route. The download page explains the steps: try to open the app
   once, then System Settings > Privacy & Security > "Open Anyway", and
   confirm with the password or Touch ID. Before opening, the page also
   explains how to verify the package against the checksum published with
   the release and its signed build provenance (`gh attestation verify`).
5. Release: the Apple Developer account is taken out at this step. The
   project owner opens the account; the signing certificates and the App
   Store Connect key live only in CI secrets and are never written to the
   repository or the development machine. The signing and release jobs run
   only on release tags; pull requests from forks do not see the secrets.
   Once the account exists, the macOS package is also signed with a
   Developer ID and notarized.

## CI rules

- Standard runners only; the labels of paid larger runners are not used.
  The free runners are free while the repository stays public, and come
  with no service promise.
- A fixed image (for example `macos-15`) and a fixed Xcode version are
  used, not `macos-latest`; both are upgraded deliberately.
- A long queue does not count as a CI failure; the job is rerun.

## Risks

- On iOS, finding devices on the local network with Bonjour is free. Raw
  multicast or broadcast, however, needs an entitlement that Apple grants
  only to paid accounts, on request. If the discovery design depends on
  that entitlement, the iOS work stalls at steps 1–3; a separate decision
  is then written.
- Until the account comes, installing on macOS will be cumbersome; this
  slows adoption and lowers trust in a privacy-focused app.

## Consequences

- Without a Mac, the simulator cannot be watched live; the work relies on
  CI logs and screenshots. Trying on a real iPhone is limited until the
  account comes, apart from seven-day sideloading with a free Apple ID. If
  a Mac becomes available, it is used too.
- An account-free Homebrew install does not count as a route: Homebrew no
  longer accepts packages that are not notarized.
  - **Note ([0015](0015-paket-yoneticileri.md)):** this only applies to app
    packages (casks). The CLI can be installed without an account through a
    formula in our own tap: formula downloads are expected not to be
    quarantined, and the ad-hoc signature is expected to be enough. This is
    tried on macOS with the CLI's first release.
- Releases come only from CI, as on the other platforms
  ([0007](0007-surec.md)).
