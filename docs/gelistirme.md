# Development layout

This document explains how the repository is organised, what happens on
each branch and how platforms are updated without breaking one another.
The path from an issue to a release, the definition of done and the review
rule are in [surec.md](surec.md); the high-level architecture of the
parts is in [mimari.md](mimari.md).

## Principle: one repository, independent platforms

Every platform uses the same Rust core and the same protocol. Because of
this, the code lives in one repository, on a single `main` branch.
Platforms are kept separate in these ways:

- **Folder:** each platform has its own folder.
- **Version:** each platform has its own version number and tag
  ([versioning.md](versioning.md)).
- **Build pipeline:** each platform has its own CI workflow. On every PR,
  `ci.yml` runs, checks which folders changed and calls only the relevant
  workflows. The final `CI` job is the single required check for `main`.
  Releases are built from tags by `android-release.yml` and
  `linux-release.yml`.
- **Package:** on Linux, the Arch, Debian and RPM packages are built from
  separate recipes, in separate containers.

A security fix goes into the core once and reaches every platform at the
same time. Each platform is released at its own pace.

## Folders

| Folder | Contents | CI |
| --- | --- | --- |
| `crates/` | Protocol core (`totaldrop-crypto`, `-proto`, `-core`) and CLI | `core.yml` |
| `linux/` | Linux desktop app (Qt 6, QML, CXX-Qt) | `linux.yml` |
| `packaging/` | Linux package recipes and build containers | `linux.yml` |
| `app/` | Android app (Flutter); iOS later | `android.yml` |
| `server/` | Rendezvous and relay servers (Rust and Cloudflare Workers versions), email Worker | `core.yml`, `server.yml` |
| `site/` | totaldrop.app, download.totaldrop.app and the QR link page | `site.yml` |
| `fuzz/` | Fuzz targets for the parsers | `fuzz.yml` (weekly) |
| `formal/` | Tamarin model of the account design and mutant checks (#251) | `formal.yml` |
| `docs/` | Design rules, versions, protocol notes, this document | |
| `tools/` | Repository tools: changelog, component versions, crate layer check ([ARCHITECTURE.md](ARCHITECTURE.md#1-katmanlar-ve-bağımlılık-yönü)), checking that `deny.toml` and `osv-scanner.toml` ignore the same advisories | `ci.yml` |
| `.github/` | Workflows; also OpenSSF Scorecard on `main` and weekly, and CodeQL code scanning on every PR and on `main` (GitHub default setup, in repository settings) | `ci.yml`, `scorecard.yml` |

At the root, `deny.toml` (cargo-deny) and `osv-scanner.toml` (OSV-Scanner,
used by OpenSSF Scorecard) ignore the same advisories for the same
reasons. `SECURITY-INSIGHTS.yml` summarises the security information in a
machine-readable form. Any PR that changes something it describes updates
it too.

When Windows and macOS arrive, they get their own folders (`windows/`,
`macos/`) and their own workflows.

## Branches

- **`main`:** always builds and passes tests. Direct push and force push
  are disabled; changes merge only through a PR, by rebasing.
- **Work branches:** short-lived, and the name says which platform:

  | Branch | What it is for |
  | --- | --- |
  | `feat/linux-...` | New feature in the Linux app |
  | `fix/android-...` | Bug fix on Android |
  | `feat/core-...` | Change in the core (affects every platform) |
  | `build/packaging-...` | Package recipes |
  | `docs/...` | Documentation |

- When the work is done, the branch is merged into `main` and deleted.
  Long-lived platform branches are not opened: if they drift apart, a fix
  in the core has to be ported to each one separately, and one gets
  forgotten.
- A change in the core runs every platform's workflow. A core change that
  breaks a platform cannot enter `main`: merging is blocked while the
  `CI` check is red.

## Local checks

CI is the required check. [pre-commit](https://pre-commit.com/) only shows
errors before you push (#376). There are two files:

- **`.pre-commit-config.yaml`** runs on every commit and needs no build
  tools. It checks for merge conflict markers, YAML and TOML syntax, large
  files, private keys and secrets (gitleaks), end-of-file newlines,
  trailing whitespace and LF line endings. It rejects `key.properties` as
  well as `.jks`, `.keystore`, `.p12`, `.pfx` and `.pepk` files. gitleaks
  inspects only the changes being committed, not the whole tree.
  `git commit --no-verify` skips these checks; the secrets and
  signing-file checks are mirrored by the `Secrets` job in CI and cannot
  be skipped (#378, [surec.md](surec.md#gizli-bilgiler)). Install it once:

  ```sh
  pre-commit install
  ```

- **`.pre-commit-config-extra.yaml`** is a local copy of the CI checks:
  `cargo fmt`, `clippy` and `test`, `dart format`, `flutter analyze`, the
  Workers' `tsc` check, site scripts, changelog, SPDX, REUSE, version
  checks, the crate layer check and checking the code references in
  [ARCHITECTURE.md](ARCHITECTURE.md) (the `check-doc-refs` hook only
  checks: it warns when lines have shifted, in which case update the
  numbers with `node tools/check-doc-refs.mjs --fix`, and fails when a
  cited line has changed, in which case re-read the claim and run
  `--accept`). It is not installed as a git hook; it is run by hand. Each
  hook needs its own tool (`cargo`, `dart`, `flutter`, `node`, `npm`);
  skip the ones you do not have installed with `SKIP`, and CI will still
  run them:

  ```sh
  pre-commit run -c .pre-commit-config-extra.yaml --all-files
  pre-commit run -c .pre-commit-config-extra.yaml cargo-fmt --all-files
  SKIP=flutter-analyze,worker-tsc pre-commit run -c .pre-commit-config-extra.yaml --all-files
  ```

  `cargo fmt` and `dart format` rewrite files in place (CI only checks);
  add the resulting changes to your commit. `clippy` and the tests for the
  Linux app (`linux/`) require Qt 6 and run only in CI. The QML tests for
  the UI components (`linux/tests/qml/`) do not require building Rust; on
  a Linux machine with Qt 6 installed they run headless via
  `linux/tests/qml/run.sh` (if `qmltestrunner` is outside
  `/usr/lib/qt6/bin`, for example on Fedora, set
  `QMLTESTRUNNER=/usr/lib64/qt6/bin/qmltestrunner`). The components are
  loaded with the stub module there (`stub/`, a fixed `Theme`) instead of
  the app's `app.totaldrop` module; when adding tests for a new component,
  add it to `stub/qmldir` and to the copy line in `run.sh`. Before
  `flutter analyze`, run `flutter pub get` inside `app/`; before the
  Workers' `tsc` check, run `npm ci` in each Worker folder.

  `changelog-require`, which checks that the branch adds a `changelog.d/`
  entry, runs only by hand, because it fails on `main` and before the
  entry is committed. It compares the branch against the last fetched
  `origin/main`:

  ```sh
  pre-commit run -c .pre-commit-config-extra.yaml changelog-require --hook-stage manual
  ```

When a check in a workflow changes, its counterpart in the extra file
changes in the same PR. Generated files are left out in both files with
the same `exclude` list. External hooks are pinned by commit SHA; those in
the main file are updated with `pre-commit autoupdate --freeze`. The REUSE
hook in the extra file stays on the same version as the `reuse` image in
`ci.yml` and is upgraded by hand together with that image.

## Linux packages

Each format is built from a separate recipe and in a separate container:

| Format | Recipe | Build container | Target |
| --- | --- | --- | --- |
| Arch | `packaging/arch/PKGBUILD` | `archlinux:latest` | Arch, Omarchy, EndeavourOS, Manjaro; AUR |
| Debian | `packaging/debian/` | `debian:trixie` (Qt 6.8) | Debian 13+, Ubuntu 25.04+ and derivatives |
| RPM | `packaging/rpm/totaldrop.spec` | `fedora:44` (Qt 6.11) | Fedora 44+ (openSUSE needs a separate recipe) |

Shared files (`.desktop`, AppStream, icons, install script) live under
`packaging/linux/`. All three packages install the same files to the same
locations.

To build a package locally (requires Docker):

```bash
packaging/build.sh arch   # or deb, rpm
ls dist/packages/
```

The Flatpak recipe lives under `packaging/flatpak/`
([decision 0014](kararlar/0014-steam-deck.md)). The build runs offline:
every crate in `Cargo.lock` is a source in the recipe. That list is not
kept in the repository; it is generated before building. To try it
locally, you need `flatpak-builder` and the Flathub remote:

```bash
node tools/flatpak-cargo-sources.mjs write
flatpak-builder --user --force-clean --install-deps-from=flathub \
  flatpak-build packaging/flatpak/app.totaldrop.TotalDrop.yml
```

Packages must be reproducible: a second build from the same source must
produce a byte-for-byte identical package. The `Reproducible` workflow
(`.github/workflows/reproducible.yml`) checks this: it builds each format
twice from two clean copies of the same commit and compares them. The
build path inside the container is always the same (`build.sh` mounts the
working directory at `/w`), so the check tests timestamps, ordering and
the build environment rather than path independence. It runs weekly,
manually, or whenever `packaging/`, `linux/build.rs`, `linux/Cargo.toml`
or `Cargo.lock` changes. If the packages differ, both are kept as job
artifacts so the difference can be inspected with `diffoscope`. Identical
output holds for the same build image: the distribution packages installed
inside the container change from day to day ([surec.md](surec.md), release
step 4). To test locally, run the same command in two clean copies of the
same commit (`build.sh` packages the working tree, including uncommitted
changes) and compare the output of `sha256sum dist/packages/*`.

`packaging/compare-packages.sh` performs the comparison: if the packages
are not identical, it compares the files inside them one by one and fails
on any difference. The single exception is a known difference: because
cxx-qt links Qt libraries in a random order, the `totaldrop-desktop`
binary can vary from build to build (#256). Even then, its read-only data
(including Qt resources), linked libraries and code size must still match;
if they do, the difference is logged only as a warning. When the binary
differs, package metadata (which contains the binary's checksum) is not
compared. The check builds only the x86_64 packages; the arm64 packages
come from the same scripts.

## Interface changes

[tasarim.md](tasarim.md) is binding. After every change that alters the
interface, the app is tested in the light and dark themes in these
environments and screenshots are attached to the PR: Omarchy, Hyprland
(without Omarchy) and KDE Plasma.
