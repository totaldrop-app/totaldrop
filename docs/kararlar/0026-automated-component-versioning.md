# 0026: Single-source component versioning and deterministic release automation

- **Status:** Proposed, 15 November 2026 (#558)
- **Numbering:** Numbers `0020`–`0025` are allocated on open pull requests
  (`0020` #499, `0021` #514, `0022` #529, `0023` #542, `0024` #368, and
  `0025` #553).
- **Context:** TotalDrop releases its deliverables on independent schedules
  under Semantic Versioning 2.0 ([versioning.md](../versioning.md)):
  `Android` (`0.3.0-beta.7+14`), `Linux` (`0.3.0-beta.6`), `Core and CLI`
  (`0.2.0-beta.5`), and `Servers` (`0.1.0-beta.1`), with `Windows`
  ([0016](0016-windows-microsoft-store.md)), `iOS and macOS`
  ([0009](0009-ios-macos.md)), standalone package-manager channels
  ([0015](0015-paket-yoneticileri.md)), and modular repositories (#548) to
  follow.

  In the existing setup, version strings are maintained by hand across more
  than twenty locations (`Cargo.toml`, internal path-dependency `version =`
  fields, `Cargo.lock`, `fuzz/Cargo.lock`, `app/pubspec.yaml`,
  `app/rust/Cargo.toml`, `app/lib/main.dart`, `packaging/arch/PKGBUILD`,
  `packaging/rpm/totaldrop.spec`, `packaging/debian/changelog`,
  `packaging/linux/app.totaldrop.TotalDrop.metainfo.xml`, and Worker
  `package.json` / `package-lock.json` files). Auditing revealed concrete
  engineering gaps:
  1. `app/lib/main.dart`'s `TOTALDROP_VERSION` fallback and `Cargo.lock`'s
     workspace package entries were not checked in CI, and
     `android-release.yml` neither passed `--dart-define=TOTALDROP_VERSION`
     nor checked `origin/main` ancestry or SemVer monotonicity across all
     merged `android-v*` tags.
  2. Fourteen internal Cargo dependency `version =` declarations were not
     verified against `[workspace.package].version`, and `fuzz/Cargo.lock`
     sat outside workspace checks.
  3. `packaging/check-cli-version.sh` accepted only `0.MINOR.PATCH-beta.N`,
     which would break at `1.0.0-rc.1` and `1.0.0`.
  4. Native store projections for Windows MSIX (`MAJOR.MINOR.PATCH.BUILD`)
     and Apple (`CFBundleShortVersionString` without pre-release suffixes
     paired with monotonic `CFBundleVersion`) were not yet codified in
     executable tooling.

- **Options:**
  1. **Lockstep ecosystem versioning.** Give every client and server the
     same version on every release. Simple to explain, but forces no-op
     store submissions on unchanged platforms or blocks urgent platform
     fixes behind unrelated work, contradicting [versioning.md](../versioning.md).
  2. **Keep manual per-file edits and shell regex checks.** Preserves
     independent versions, but leaves multi-file bumps error-prone and
     scatters SemVer and store-projection logic across unrelated scripts.
  3. **Independent Semantic Versioning 2.0 backed by a canonical
     `versions.json` manifest, deterministic `changelog.d`-driven bump
     calculation, monotonic integer build numbers, and one zero-dependency
     `tools/version.mjs` verifier and bumper.**

- **Decision:** Adopt **Option 3**.
  1. **Canonical manifest (`versions.json`):** Every repository keeps a
     machine-readable `versions.json` at its root naming `stage`
     (`"pre-1.0"` until the five-platform and external-audit gate in
     [versioning.md](../versioning.md) is met, then `"stable"`) and each
     deliverable component's `version` (`MAJOR.MINOR.PATCH[-CHAN.N]`) plus,
     where store channels require an integer build counter, `build`
     (`>= 1`, currently `14` on Android for `0.3.0-beta.7+14`). Protocol
     wire negotiation (`TD0` / `v0.1`) stays separate from component SemVer.
  2. **Deterministic SemVer derivation (`plan` and `bump`):** Next versions
     are derived reproducibly from pending `changelog.d/` fragments and
     merged `<component>-v*` release tags:
     - While `stage` is `"pre-1.0"`, every component version has the form
       `0.MINOR.PATCH-beta.N` (`N >= 1`). Pending `added`, `changed`,
       `deprecated`, or `removed` entries propose a `minor` bump
       (`0.(MINOR+1).0-beta.1`); `fixed` or `security` entries alone propose
       a `patch` bump (`0.MINOR.(PATCH+1)-beta.1`); rebuilds with only
       `.none.md` entries or explicit same-series iterations (`--level
       prerelease`) raise `beta.(N+1)`.
     - From `1.0.0` (`stage: "stable"`), full SemVer 2.0 precedence
       (`MAJOR.MINOR.PATCH`, with `alpha.N < beta.N < rc.N < final`) applies
       across both `tools/version.mjs` and `packaging/check-cli-version.sh`.
  3. **Strictly monotonic build numbers:** Platforms whose stores or
     installers compare an integer build code (`android`, `windows`,
     `macos`, `ios`) increment `build` by `+1` on every published build and
     never reuse or lower it, even when `version` advances. On Android,
     App Bundles (`AAB`) carry `versionCode = build` (`< 1000` whenever
     Flutter per-ABI APK splits `1000 * abi + build` are used, and
     `<= 2100000000` for Google Play).
  4. **Deterministic native format projections:** Every package or store
     version string is a pure function of `(version, build)` verified by
     `node tools/version.mjs check`:
     - **Android:** `<version>+<build>` in `app/pubspec.yaml`; `<version>`
       in `app/rust/Cargo.toml`, `app/lib/main.dart` (`TOTALDROP_VERSION`
       default), `--dart-define=TOTALDROP_VERSION`, and `Cargo.lock`.
     - **Debian, RPM, and AppStream:** `<M.m.p>~<pre>` for pre-releases and
       `<M.m.p>` for final releases, with AppStream `type="development"` vs.
       `type="stable"`.
     - **Arch (`PKGBUILD`):** `pkgver` `<M.m.p>_<pre>` (`_ver=<version>`).
     - **Windows MSIX ([0016](0016-windows-microsoft-store.md)):** 4-part
       `uint16` tuple `<M>.<m>.<p>.<build>` (`0..65535` per field).
     - **Apple App Store ([0009](0009-ios-macos.md)):**
       `CFBundleShortVersionString = <M>.<m>.<p>` (without pre-release
       identifiers rejected by App Store Connect), `CFBundleVersion =
       <build>`, and full `<version>` in the user-facing `About` view.
  5. **Tag provenance and unchanged-code guards (`check --tag`):** Every
     `<component>-v<version>` release tag must sit on `origin/main`
     (`git merge-base --is-ancestor`), match `versions.json` and every
     manifest in the tree, exceed the highest SemVer and highest `build` of
     all previously merged `<component>-v*` tags (never only the nearest
     tag), and pass the CLI unchanged-code guard when shipping `linux-v*`.
  6. **Compatibility with modular repositories (#548):** `tools/version.mjs`
     validates and bumps whichever components are declared in a repository's
     `versions.json`, so the exact same tool and test suite work unchanged
     in the monorepo today and as a shared check across `totaldrop-core`,
     `totaldrop-android`, `totaldrop-linux`, `totaldrop-servers`,
     `totaldrop-cli-bsd`, `totaldrop-windows`, `totaldrop-macos`, and
     `totaldrop-ios-tvos`.

- **Consequences:**
  - **Gains:**
    - `node tools/version.mjs bump <component>` replaces error-prone
      multi-file hand edits, and `node tools/version.mjs check` runs on every
      pull request and release tag alongside the existing packaging checks.
    - Signed Android App Bundles receive `TOTALDROP_VERSION` explicitly from
      the verified `android-v*` tag and reject off-`main` tags, SemVer
      regressions, and reused `versionCode` values before touching signing
      secrets.
    - Moving from `0.x-beta.N` to `1.0.0-rc.1` and `1.0.0` requires flipping
      `"stage": "stable"` in `versions.json` rather than rewriting shell
      parsers.
  - **Costs and trade-offs:**
    - Every release still requires a reviewed preparation pull request running
      `node tools/version.mjs bump <component>` before cutting the tag so
      `versions.json`, lockfiles, and packaging recipes stay committed on
      `main` under branch protection (`bypass_actors: []`).
    - Adding a new packaging target or manifest format requires extending
      `tools/version.mjs` and `tools/test-version.mjs` in lockstep, and once
      modular repositories cut over (#548), the canonical copy of
      `tools/version.mjs` must be kept synchronized across repositories via CI.
