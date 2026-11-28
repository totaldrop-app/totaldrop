# Versions and releases

## Version numbers

TotalDrop uses [Semantic Versioning 2.0](https://semver.org/).

- **Every platform has its own version.** The Linux app, the Android app,
  the CLI and the server are released separately. Updating one platform
  does not wait for the others and does not break them.
- Where the version is written:

  | Component | File |
  | --- | --- |
  | Core libraries and CLI | `Cargo.toml`, `[workspace.package]` |
  | Linux app | `linux/Cargo.toml` |
  | Android app | `app/pubspec.yaml`; the Rust library `app/rust/Cargo.toml` carries the same version, without the build number |
  | Servers (rendezvous and relay) | `server/totaldrop-{rendezvous,relay}/Cargo.toml`, `server/{rendezvous,relay}-worker/package.json`; all four have the same version |
  | Mail Worker | `server/mail-worker/package.json`; not part of the protocol, carries its own version and is only deployed, not released |

- The servers and `app/rust` do not inherit the workspace's version: when
  the CLI's version goes up, they must not appear with a version that was
  never released. `tools/check-component-versions.sh` checks this and the
  equalities above on every PR; `packaging/check-versions.sh` checks the
  Linux packages. The servers' version started at 0.1.0-beta.1 (#146); the
  0.2.0-beta.1 they inherited from the workspace before was never tagged or
  packaged.

- **The CLI is released with the Linux packages.** Every `linux-v…` tag is
  also the release of the CLI version in that commit; a `cli-v…` tag is
  optional. In a Linux release, the CLI version cannot be lower than the
  highest one released so far, and cannot stay the same if the CLI code
  changed (`packaging/check-cli-version.sh`). Both `tools/version.mjs` and
  `packaging/check-cli-version.sh` enforce `0.MINOR.PATCH-beta.N` before 1.0
  and full SemVer 2.0 (`alpha.N < beta.N < rc.N < final`) from 1.0.0 onwards
  ([0026](kararlar/0026-automated-component-versioning.md)).

- **Beta until 1.0.** Until five platforms (Linux, Windows, macOS, Android,
  iOS) are released and the independent security review is finished, every
  version is a pre-release: `0.MINOR.PATCH-beta.N`. The apps show "beta"
  next to the version number.
- During the beta, a new feature or an incompatible change raises `MINOR`,
  a fix raises `PATCH`, and a new build of the same version raises
  `beta.N`.
- **The protocol** has a separate version (`v0`, then the `v0.1` drafts),
  because devices negotiate it with each other (specification section 11).
  Every app writes in its release notes which protocol versions it speaks.
  This is what lets different platforms work together; the app versions do
  not need to be the same.

## Single source of truth (`versions.json` and `tools/version.mjs`)

[`versions.json`](../versions.json) is the canonical machine-readable record
of the release stage (`"pre-1.0"` or `"stable"`) and each component's
`version` (and monotonic integer `build` where required, such as Android's
`versionCode`; see [0026](kararlar/0026-automated-component-versioning.md)).

- `node tools/version.mjs check` verifies `versions.json` against every
  manifest, internal Cargo path dependency, `Cargo.lock`, `fuzz/Cargo.lock`,
  `app/lib/main.dart` (`TOTALDROP_VERSION` fallback), Worker lockfile, and
  Linux packaging recipe on every pull request.
- `node tools/version.mjs plan [<component>]` inspects pending entries in
  `changelog.d/` and merged `<component>-v*` tags to compute the next SemVer
  and build number deterministically.
- `node tools/version.mjs bump <component> [--level minor|patch|prerelease]`
  updates `versions.json` and every downstream file for `<component>` in one
  atomic step instead of editing files by hand.
- `node tools/version.mjs check --tag <platform>-v<version>` runs on release
  tags in CI to verify `origin/main` ancestry and strict SemVer and build
  number monotonicity across all prior release tags.

## Package and store versions

Every native package or store version is derived deterministically from
`(version, build)` (`node tools/version.mjs project <component>`):

| Format | `0.3.0-beta.7` (build `14`) is written as |
| --- | --- |
| Android (`app/pubspec.yaml`) | `0.3.0-beta.7+14` (`versionName` `0.3.0-beta.7`, `versionCode` `14`) |
| Arch (`PKGBUILD`) | `pkgver=0.3.0_beta.7` |
| Debian (`debian/changelog`) | `0.3.0~beta.7-1` |
| RPM (`totaldrop.spec`) | `Version: 0.3.0~beta.7` |
| AppStream | `0.3.0~beta.7` (`type="development"` before final, `type="stable"` on final) |
| Windows MSIX (`Package.appxmanifest`) | `0.3.0.14` (four `uint16` fields) |
| Apple (`Info.plist` / Xcode) | `CFBundleShortVersionString` `0.3.0`, `CFBundleVersion` `14` (full SemVer shown in About) |

## Build numbers

- The Android `versionCode`, Windows MSIX fourth tuple field, and iOS/macOS
  `CFBundleVersion` are a single integer (`build` in `versions.json`) that
  goes up by one with every published build. On Android it comes after the
  `+` in `app/pubspec.yaml` (for example `0.3.0-beta.7+14`).
- A build number is never reused; an old package cannot be installed over
  a newer one.

## Git

- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/),
  and the scope names the platform: `feat(linux):`, `fix(android):`,
  `security(core):`, `docs:`, `ci(linux):`, `build(packaging):`. An
  incompatible change gets a `!` (`feat(core)!:`) or a `BREAKING CHANGE:`
  note.
- Every release is a signed, annotated tag on `main` that starts with the
  platform's name:

  | Platform | Tag |
  | --- | --- |
  | Core | `core-v0.2.0-beta.5` |
  | Linux | `linux-v0.3.0-beta.6` |
  | Android | `android-v0.3.0-beta.7` |
  | CLI (optional, see above) | `cli-v0.2.0-beta.5` |
  | Servers | `server-v0.1.0-beta.1` |

- [`CHANGELOG.md`](../CHANGELOG.md) follows the [Keep a
  Changelog](https://keepachangelog.com/) format. Every entry names its
  platform; security fixes sit under the "Security" heading. Unreleased
  entries wait in a separate file for each change, under
  [`changelog.d/`](../changelog.d/README.md); at release they are moved
  into CHANGELOG.md.

## Making a release

The step-by-step release list is in [surec.md](surec.md#sürüm-listesi).
In short: inspect the planned bump with `node tools/version.mjs plan`, raise
the component's version and recipes atomically with
`node tools/version.mjs bump <component>`, move the `changelog.d/` entries
under the release heading with `tools/changelog.mjs release`, verify with
`node tools/version.mjs check`, review and merge the release PR, and tag the
commit on `main`. CI verifies the tag with
`node tools/version.mjs check --tag`, builds the packages from the tag, adds
build provenance and opens a draft release (marking it as a GitHub pre-release
only when the version carries a `-` pre-release suffix). Nothing built on a
personal machine is published.

On Android, the signed Play package is built from the `android-v*` tag with
`--dart-define=TOTALDROP_VERSION=<version>`, only with the upload key in the
secrets of the `play-release` environment; the package is downloaded as a
GitHub Actions artifact and uploaded to the Play Console.
