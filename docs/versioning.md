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
  changed (`packaging/check-cli-version.sh`). This check only recognises
  the `0.MINOR.PATCH-beta.N` form; it must be changed at 1.0.

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

## Package versions

A pre-release is written so that, in every package format, it sorts before
the final release:

| Format | `0.1.0-beta.2` is written as |
| --- | --- |
| Arch (`PKGBUILD`) | `pkgver=0.1.0_beta.2` |
| Debian (`debian/changelog`) | `0.1.0~beta.2-1` |
| RPM (`totaldrop.spec`) | `Version: 0.1.0~beta.2` |
| AppStream | `0.1.0~beta.2` |

## Build numbers

- The Android `versionCode` and the iOS/macOS build number are a single
  integer that goes up by one with every published build. It comes after
  the `+` in `app/pubspec.yaml` (for example `0.1.0-beta.2+2`).
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
  | Linux | `linux-v0.1.0-beta.2` |
  | Android | `android-v0.1.0-beta.2` |
  | CLI (optional, see above) | `cli-v0.1.0-beta.2` |
  | Servers | `server-v0.1.0-beta.2` |

- [`CHANGELOG.md`](../CHANGELOG.md) follows the [Keep a
  Changelog](https://keepachangelog.com/) format. Every entry names its
  platform; security fixes sit under the "Security" heading. Unreleased
  entries wait in a separate file for each change, under
  [`changelog.d/`](../changelog.d/README.md); at release they are moved
  into CHANGELOG.md.

## Making a release

The step-by-step release list is in [surec.md](surec.md#sürüm-listesi).
In short: the version and the package recipes are raised, the
`changelog.d/` entries are moved under the release heading with
`tools/changelog.mjs release`, the release PR is reviewed and merged, and
the commit on `main` is tagged. CI builds the packages from the tag, adds
build provenance and opens a draft release. Nothing built on a personal
machine is published.

On Android, the signed Play package is built from the `android-v*` tag,
only with the upload key in the secrets of the `play-release` environment;
the package is downloaded as a GitHub Actions artifact and uploaded to the
Play Console.
