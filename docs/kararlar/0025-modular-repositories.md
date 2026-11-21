# 0025: Split the monorepo into modular repositories pinned to versioned `totaldrop-core` releases

- **Status:** Proposed (#548)
- **Numbering:** Numbers `0020`–`0024` are allocated on open pull requests
  (`0020` transfer resumption in #499, `0021` per-language site addresses in
  #514, `0022` Mineral web design system in #529, `0023` Android QR camera
  scanner in #542, and `0024` team donations via Open Collective in #368).
- **Context:** [0001](0001-tek-depo.md) placed the Rust protocol core,
  servers, static website, CLI, formal Tamarin models, fuzzers, and client
  apps in a single repository (`totaldrop-app/totaldrop`) so that a core
  security fix would reach every client in one pull request while each
  platform kept its own folder, version, and release tag.

  Since October 2026, TotalDrop's current and planned scope has grown across
  distinct build toolchains, OS runners, and release/deployment pipelines:
  Rust core crates and the shared 35-language catalog
  ([0008](0008-diller-ve-bolgeler.md)), Tamarin Prover + `cargo-fuzz`,
  Cloudflare Workers + self-hosted Rust OCI servers
  ([0003](0003-cloudflare-altyapi.md)), the 35-language Cloudflare Pages site,
  Linux/SteamOS Qt 6 (`deb`, `rpm`, Arch, Flatpak;
  [0006](0006-linux-qt.md), [0014](0014-steam-deck.md)), Android
  ([0010](0010-android-arka-plan-aktarim.md),
  [0011](0011-android-material-3-expressive.md)), Windows (`WinUI 3`;
  [0016](0016-windows-microsoft-store.md)), iOS and macOS
  ([0009](0009-ios-macos.md)), and standalone CLI binaries and package
  managers ([0015](0015-paket-yoneticileri.md)). Keeping every toolchain,
  lockfile, Dependabot queue, CodeQL matrix, and path-filtered workflow in
  one repository couples unrelated review queues, concentrates every signing
  key and deployment token in a single repo, and forces every `crates/` edit
  to fan out across every desktop and mobile OS runner before merge.

  Eleven dedicated repositories have been provisioned under the
  `totaldrop-app` GitHub organization alongside `.github` (#548). They
  currently hold initial synced snapshots (`sync c86c298`, plus #433 ported
  to `totaldrop-linux`), pre-ADR scaffold/prototype commits in the reserved
  client repos, and scaffolded `Deploy *` / `Package (*)` workflow files with
  no deployment environments or secrets; rule 4 ("Phase 0") and rule 6 govern
  them as frozen mirrors until each area's Phase 2/3 cutover PR (or future
  platform ADR) lands.

## Options considered

1. **Keep everything in `totaldrop-app/totaldrop` ([0001](0001-tek-depo.md)).**
   Preserves single-PR core edits, but scales poorly across platform
   toolchains, mixes unrelated Dependabot and CodeQL queues, and requires one
   repository to hold every store signing and Cloudflare deployment
   environment.
2. **Split by repository with floating `main` git dependencies.** Unblocks
   independent client CI, but breaks reproducible builds and crates.io
   publishing ([0015](0015-paket-yoneticileri.md)) and lets an unreviewed or
   breaking upstream commit silently alter downstream builds.
3. **Split into modular repositories pinned to immutable `core-vX.Y.Z`
   releases, backed by required pre-tag downstream canary CI, `N` / `N-1`
   server wire compatibility, automated bump pull requests, and an attested
   `security-floor.json` gate.**

## Decision

Adopt **Option 3**. Once the shared workflows and gates below are in place,
TotalDrop migrates in dependency order (rule 4) from the monorepo to eleven
repositories under `totaldrop-app` (plus `totaldrop-app/.github` for shared
reusable workflows, scripts, governance defaults, `SECURITY.md`, and
templates):

| Repository | Responsibility | `totaldrop-core` & `l10n` relationship |
| --- | --- | --- |
| `totaldrop-core` | `totaldrop-crypto`, `totaldrop-proto`, `totaldrop-core`, `totaldrop-l10n`, the canonical shared `l10n/` tree (`l10n/app/`, `l10n/cli/`, `l10n/status.json`, `l10n/tests/` per [0008](0008-diller-ve-bolgeler.md)), and system-wide protocol/threat-model/ADR docs | Source of truth (`core-vX.Y.Z` tags + attested `l10n` and `security-floor.json` release assets; crates.io per [0015](0015-paket-yoneticileri.md) step 5) |
| `totaldrop-formal-fuzz` | Symbolic security proofs in Tamarin Prover and `cargo-fuzz` targets against `totaldrop-proto` / `totaldrop-core` | Pinned `core-vX.Y.Z` + scheduled `main` canary |
| `totaldrop-servers` | Zero-knowledge rendezvous and relay servers for Cloudflare Workers (Durable Objects) and self-hosted Rust (`totaldrop-rendezvous`, `totaldrop-relay`), plus server health probes | Production servers link only `totaldrop-proto`; E2E tests cover `N` and `N-1` `totaldrop-cli` / `totaldrop-core` |
| `totaldrop-site` | Zero-tracker 35-language static site (`totaldrop.app`, `download.totaldrop.app`, `/privacy`, `/donate`, `/r`), `site/public/i18n/`, `site/privacy/`, and site health probe | Verifies `l10n/status.json` + `l10n/tests/*.json` from pinned `core-vX.Y.Z` `l10n` archive |
| `totaldrop-cli-bsd` | `totaldrop-cli` binary, shell completions/man pages, and [0015](0015-paket-yoneticileri.md) package-manager release automation (BSD/RISC-V ports subject to rule 6) | Pinned `core-vX.Y.Z` git tag + `Cargo.lock` `rev` until [0015](0015-paket-yoneticileri.md) step 5, then exact `=X.Y.Z` on crates.io |
| `totaldrop-linux` | Wayland Qt 6 + QML Linux and SteamOS desktop client (`deb`, `rpm`, Arch, Flatpak; [0006](0006-linux-qt.md), [0014](0014-steam-deck.md)) | Pinned `core-vX.Y.Z` + `l10n` archive |
| `totaldrop-android` | Flutter Material 3 Expressive Android client ([0011](0011-android-material-3-expressive.md)), `app/rust` FFI bridge, and Google Play release pipeline | Pinned `core-vX.Y.Z` + `l10n` archive |
| `totaldrop-windows` | Windows 11 `WinUI 3` client and Explorer extension ([0016](0016-windows-microsoft-store.md)) | Pinned `core-vX.Y.Z` + `l10n` archive |
| `totaldrop-macos` | Reserved for the macOS client ([0009](0009-ios-macos.md); UI stack subject to rule 6) | Pinned `core-vX.Y.Z` + `l10n` archive |
| `totaldrop-ios-tvos` | Reserved for the iOS/iPadOS client ([0009](0009-ios-macos.md); UI stack and any tvOS target subject to rule 6) | Pinned `core-vX.Y.Z` + `l10n` archive |
| `totaldrop-web-wasm` | Reserved repository for a future browser receiver (unimplemented until accepted by its own ADR and `T9` threat-model revision; see rule 6) | Reserved |

### Governance and technical rules

1. **Core security fixes, `crates.io` scope, attested `security-floor.json`, and embargoed patches:**
   - **Pinning discipline and `crates.io` scope ([0015](0015-paket-yoneticileri.md)):**
     Every `core-vX.Y.Z` release in `totaldrop-core` attaches a checksummed
     `totaldrop-l10n-vX.Y.Z.tar.gz` archive and an attested
     `security-floor.json` asset. Repository cutover **does not block** on
     crates.io publication: until [0015](0015-paket-yoneticileri.md) step 5
     ("crates.io and `cargo binstall`") resolves its packaging blockers,
     downstream workspaces (`totaldrop-formal-fuzz`, `totaldrop-servers`,
     `totaldrop-cli-bsd`, `totaldrop-linux`, `totaldrop-android`) pin the
     immutable `core-vX.Y.Z` git tag together with the exact 40-character
     commit `rev` in `Cargo.lock` — never a floating branch. Once `0015` step
     5 activates, `totaldrop-core` performs the one-time short-lived
     `publish-new` bootstrap (`0015` lines 131–137) and then publishes exact
     `0.x` versions of `totaldrop-crypto`, `totaldrop-proto`,
     `totaldrop-l10n` (with `l10n/` packaged in the crate tarball), and
     `totaldrop-core` via CI Trusted Publishing under co-owned
     `totaldrop-app` team ownership solely to support `cargo install
     totaldrop-cli` (`0015` lines 128–137). Each crate's `README.md` and
     rustdoc state explicitly that it is an internal TotalDrop crate with no
     external library SemVer stability promise; offering a supported public
     library API (`1.0`) is out of scope here and would require its own ADR.
   - **Automated downstream bumps:** Cutting `core-vX.Y.Z` triggers an
     automated workflow in `totaldrop-app/.github` that opens synchronized
     bump pull requests across every active downstream repository.
   - **Attested `security-floor.json` with deterministic offline behavior:**
     `security-floor.json` names the minimum safe `core` SemVer and allowed
     commit/crate SHA-256 digests and is **never fetched live from mutable
     `totaldrop-core@main`**. Instead:
     - Every `core-vX.Y.Z` GitHub Release attaches `security-floor.json`
       signed with keyless Sigstore build provenance
       (`actions/attest-build-provenance`, verified via `gh attestation
       verify --repo totaldrop-app/totaldrop-core`), and
       `totaldrop-app/.github` vendors the latest attested copy alongside its
       reusable workflow.
     - **On pull requests and `main` pushes (offline & deterministic):** The
       reusable **required check** reads only the vendored
       `security-floor.json` from the pinned `totaldrop-app/.github` workflow
       checkout, so neither a bad push to `totaldrop-core@main` nor a GitHub
       API/network outage can turn unrelated downstream PRs red.
     - **On release tags and daily scheduled CI:** The workflow also downloads
       the latest immutable `core-v*` release's `security-floor.json`,
       verifies its Sigstore attestation, and enforces the stricter of the
       vendored and attested floors. Release tag builds **fail closed** if
       the release asset cannot be fetched or verified; daily scheduled runs
       retry transient network errors with backoff and fail only that
       scheduled run (alerting maintainers on idle repos without blocking
       unrelated PRs).
   - **Embargoed security fixes (`GHSA` private forks):** Vulnerabilities
     under coordinated embargo never use public branches, public cross-repo
     canary dispatches, or pre-disclosure `security-floor.json` updates.
     Because GitHub disables Actions workflows inside temporary `GHSA`
     private forks (`*-ghsa-*`), maintainers collaborate on the patch in
     private `GHSA` forks on `totaldrop-core` (and on affected downstream
     repositories when client or server code also changes) and verify the
     cross-crate test suites locally (or in a locked-down private
     organization staging repository with zero public dispatches). At
     coordinated disclosure, maintainers merge the `totaldrop-core` `GHSA`
     fix, run the `core-release` canary (which accepts optional candidate ref
     inputs when companion downstream `GHSA` fixes land simultaneously), cut
     `core-vX.Y.Z` with the updated attested `security-floor.json`, land the
     downstream bump PRs and platform releases, and publish the `GHSA` /
     RustSec advisory.

2. **Required pre-tag canaries, server ↔ client wire compatibility, and E2E testing:**
   - Deterministic `TD0` wire, manifest, STREAM, and QR/link test vectors live
     in `totaldrop-core` and run in every client repository's CI.
   - **Hard required pre-tag canary gate:** Cutting a `core-vX.Y.Z` release is
     gated by a protected `core-release` environment (with a repository tag
     ruleset restricting `refs/tags/core-v*` creation to the release flow)
     whose tag-and-publish job declares `needs: [canary]`. That canary job
     runs builds and test suites against the candidate `totaldrop-core`
     commit SHA across `totaldrop-servers`, `totaldrop-cli-bsd`, and every
     active shipped GUI client (`totaldrop-linux` CXX-Qt bridge + tests and
     `totaldrop-android` `flutter_rust_bridge` + Flutter/JVM tests; reserved
     client repos join the required matrix in their cutover PR). The canary
     is a **blocking gate, not advisory**: any red downstream target prevents
     `core-vX.Y.Z` from being tagged or published.
   - **Server ↔ client wire-compatibility rule (`N` and `N-1`):** Production
     rendezvous and relay servers in `totaldrop-servers` (Cloudflare Workers
     and self-hosted Rust servers) **must** accept clients running at least
     the previous shipped `core-vX.Y.Z` release (`N-1`) alongside the
     incoming release (`N`) throughout a rollout. Breaking wire changes
     require an explicit protocol version or capability negotiation and a
     two-phase rollout (deploy dual-compatible servers first, ship updated
     clients, retire `N-1` only after a documented deprecation cycle). Both
     the `totaldrop-core` pre-tag canary and `totaldrop-servers` CI run
     **skewed-version E2E matrices** (`N-1 totaldrop-cli ↔ candidate server`,
     `candidate totaldrop-cli ↔ pinned/deployed server`, and `candidate ↔
     candidate`) across both Cloudflare Workers (`wrangler dev --local`) and
     the multi-OS Rust servers (Linux, macOS, Windows).

3. **Shared sources (`l10n/`, repo checks, agent/process files, cross-repo contracts):**
   - **Single `l10n/` catalog ([0008](0008-diller-ve-bolgeler.md)):**
     `l10n/app/`, `l10n/cli/`, `l10n/status.json`, and
     `l10n/tests/{messages,negotiation}.json` remain a single canonical tree
     in `totaldrop-core` (never forked per client repo). Non-Rust consumers
     (`flutter gen-l10n` in `totaldrop-android`, `desktop-entries.mjs` in
     `totaldrop-linux`, `.resw` generation in `totaldrop-windows`, and
     `check-i18n.mjs` in `totaldrop-site`) consume the checksum-verified
     `totaldrop-l10n-vX.Y.Z.tar.gz` matching their pinned `core-vX.Y.Z`
     version. Only `site/public/i18n/` and `site/privacy/` live in
     `totaldrop-site`.
   - **Shared CI gates in `totaldrop-app/.github`:** Reusable SHA-pinned
     workflows run `check-spdx.sh`, `reuse lint` (each repo carries its own
     `REUSE.toml` + `LICENSES/` matching `LICENSING.md`: `MIT OR Apache-2.0`
     for core crates, `l10n/`, and `fuzz/`; `AGPL-3.0-or-later` for servers;
     and `GPL-3.0-or-later` for `totaldrop-cli`, GUI apps, site, and
     `formal/`/shared tooling, keeping font/avatar/Cargokit overrides where
     applicable), `check-advisory-ignores.sh` (`deny.toml` <->
     `osv-scanner.toml` parity), `gitleaks` + signing-key filename ban,
     Node/toolchain lock checks (`check-versions.sh`), `changelog.mjs` (each
     repo owns its own `CHANGELOG.md` + `changelog.d/`), `check-doc-refs.mjs`
     (each repo owns its own `docs/ARCHITECTURE.{md,refs.json}` while
     system-wide threat model and ADRs stay in `totaldrop-core/docs/`), and a
     manifest-driven `check-layers.mjs` that runs in **both** `totaldrop-core`
     and `totaldrop-servers` (enforcing that `totaldrop-rendezvous` and
     `totaldrop-relay` link only `totaldrop-proto`, never `totaldrop-core` or
     `totaldrop-crypto`).
   - **Governance, process, `CODEOWNERS`, and coding-agent files:**
     - **`totaldrop-app/.github`:** Holds organization-wide community health
       defaults (`GOVERNANCE.md`, `CONTRIBUTING.md`, `SECURITY.md`,
       `PULL_REQUEST_TEMPLATE.md`, issue templates), the canonical working
       process (`docs/process.md`, replacing `docs/surec.md` per
       [0019](0019-english-by-default.md) and linked from
       `totaldrop-core/docs/process.md`), and the canonical copies of files
       that GitHub and coding agents resolve only inside each checked-out
       repository: `.github/CODEOWNERS` (`* @totaldrop-app/maintainers`,
       matching `GOVERNANCE.md` and `require_code_owner_review: true`), the
       shared `.github/copilot-instructions.md` review checklist baseline,
       and the shared cross-repo `.claude/skills/`
       (`software-engineering-discipline`, `security-engineering-analyst`,
       `fullstack-engineering`).
     - **Per-repository files (synced & drift-checked in CI):** Because
       GitHub, GitHub Copilot PR review, and Claude Code (#458) do not follow
       pointers across uncloned repositories, every repository carries synced
       copies of `.github/CODEOWNERS`, `.github/copilot-instructions.md`, and
       the three shared cross-repo `.claude/skills/`, verified byte-for-byte
       against `totaldrop-app/.github` in CI. Each repository also owns a
       concise repo-scoped `AGENTS.md` (with `CLAUDE.md` as a thin local stub
       pointing to repo-local `AGENTS.md` and `.claude/skills/`), plus the
       domain-specific `.claude/skills/` that govern its code
       (`applied-cryptography-review` in `totaldrop-core` and
       `totaldrop-formal-fuzz`, `web-design-pro` in `totaldrop-site`,
       `android-app-engineering` in `totaldrop-android`,
       `ios-app-engineering` in `totaldrop-ios-tvos`,
       `macos-app-engineering` in `totaldrop-macos`, and
       `windows-app-engineering` in `totaldrop-windows`).
   - **Cross-repo site/client contracts (`site.yml`):**
     - **App Links (`T19`, `test-assetlinks.mjs`):** A shared
       `assetlinks-contract.json` in `totaldrop-app/.github` pins the Android
       `applicationId`, Play signing SHA-256 fingerprint, and `/r` host/path
       filter; both `totaldrop-site` (`assetlinks.json`) and
       `totaldrop-android` (`build.gradle.kts` + `AndroidManifest.xml`) verify
       against it in CI.
     - **`security.txt` (`test-security-txt.mjs`):** Canonical `SECURITY.md`
       lives in `totaldrop-app/.github/SECURITY.md` and `totaldrop-site`
       validates `site/public/.well-known/security.txt` against it.
     - **Brand mark & READMEs:** Canonical `logo.svg` digest is checked across
       `totaldrop-site` (`favicons.mjs`, `brand-files.mjs`) and client icon
       sources; each repo keeps an English `README.md`, while the 35-language
       translated `docs/readme/` set moves to `totaldrop-core` (with
       organization profile links in `totaldrop-app/.github`) and stays gated
       by `check-readmes.mjs`.

4. **Interim frozen-mirror rule, history import, dependency-ordered cutover, and archival:**
   - **Phase 0 (interim frozen-mirror rule, in force until each area's Phase 2/3 cutover PR):**
     Until this record is `Accepted` and a specific area's Phase 2 or Phase 3
     cutover PR merges (or, for the four reserved repositories, until their
     own platform ADR and cutover PR land), `totaldrop-app/totaldrop` is the
     **sole canonical source of truth** and **sole deployment and release
     origin**. Every not-yet-cut-over `totaldrop-*` repository is a **frozen
     mirror**: no feature or bugfix commits land there directly, **no GitHub
     Actions environments or secrets** may be created in it, and every non-CI
     `Deploy *` or `Package (*)` workflow in that repository must be
     **disabled in GitHub Actions** (`gh workflow disable`) immediately upon
     acceptance of this record and stay disabled until that repository's
     cutover PR lands.
   - **History import vs. bootstrap commits (`main` ruleset procedure):** All
     eleven modular repositories already enforce active `main`
     (`non_fast_forward`, `deletion`, `required_linear_history`,
     `required_status_checks`, `bypass_actors: []`) and `main review`
     (`pull_request`, `bypass_actors: []`) rulesets, and currently hold no
     releases, tags, or external commit pins:
     - **Seven monorepo-backed repositories (`totaldrop-core` in Phase 2 plus
       the six Phase 3 repositories):** For each repository's cutover, the
       organization owners extract path-filtered git history from
       `totaldrop-app/totaldrop` (`git filter-repo`, preserving original
       authors, timestamps, and `#issue` references), temporarily lift
       `non_fast_forward` and `pull_request` on that repository's `main`
       rulesets once to force-push the filtered `main` history (cleanly
       discarding synthetic bootstrap commits and any unreviewed pre-cutover
       additions unless re-submitted via PR), immediately restore
       `non_fast_forward` and `pull_request` with `bypass_actors: []`, and
       then merge the modular CI/packaging wiring through the reviewed
       cutover PR on top before cutting its first tag.
     - **Four reserved repositories (`totaldrop-windows`, `totaldrop-macos`,
       `totaldrop-ios-tvos`, `totaldrop-web-wasm`):** These have no directory
       in `totaldrop-app/totaldrop` to filter history from; any pre-ADR
       scaffold or prototype commits currently on their `main` branches are
       non-authoritative under Phase 0 and rule 6, and will either be reset
       to a clean root commit under the same one-time owner ruleset procedure
       or re-submitted piece by piece via reviewed pull requests once each
       platform's required ADR is accepted.
   - **Phase 1 (`totaldrop-app/.github`):** Land reusable CI and attested
     `security-floor.json` workflows, shared check scripts, `CODEOWNERS` /
     Copilot / shared-skill drift checks, `assetlinks-contract.json`,
     `docs/process.md`, and organization `SECURITY.md`.
   - **Phase 2 (`totaldrop-core`):** Import `crates/totaldrop-{crypto,proto,l10n,core}`,
     `l10n/`, and cross-repo docs with filtered history (via the one-time
     `main` ruleset procedure above), land its cutover PR, and cut the
     initial `core-vX.Y.Z` tag + attested `l10n` and `security-floor.json`
     release assets while `totaldrop-app/totaldrop` remains buildable for
     not-yet-migrated areas.
   - **Phase 3 (downstream cutovers, one PR per area, zero dual-repo edits):**
     Migrate `totaldrop-formal-fuzz`, `totaldrop-servers`, `totaldrop-site`,
     `totaldrop-cli-bsd`, `totaldrop-linux`, and `totaldrop-android`. Each
     area imports filtered history via the one-time `main` ruleset procedure,
     lands its reviewed cutover PR with green CI, enables its release or
     deploy workflows and environments (rule 5), updates `gh attestation
     verify --repo` provenance instructions (`T12`, `docs/process.md`), and
     freezes/removes that directory in `totaldrop-app/totaldrop` on the same
     day. Before `crates/totaldrop-cli` leaves `totaldrop-linux`'s workspace,
     `.deb`, `.rpm`, and Arch packaging either transitions to standalone
     `totaldrop-cli` packages from `totaldrop-cli-bsd`
     ([0015](0015-paket-yoneticileri.md)) with package-level
     `Recommends`/`Depends` or builds `totaldrop-cli` at a pinned
     `cli-vX.Y.Z` source archive so upgrades never drop `/usr/bin/totaldrop`.
   - **Phase 4 (archival):** Once every active area has cut over and open PRs
     in `totaldrop-app/totaldrop` are landed or closed, retired environments
     and secrets are deleted, [0001](0001-tek-depo.md) becomes `Superseded by
     0025`, `README.md` points to the modular repositories, and
     `totaldrop-app/totaldrop` is archived **read-only**.

5. **Deployment environments, secrets split, and retiring `deploy.yml`:**
   - Least-privilege environment and secret assignment (narrowing `T11`):
     - **`totaldrop-site` (`cloudflare-site`):** Own locked `wrangler`
       (`npm ci --ignore-scripts`) + `probe-site.mjs` (checking `/`,
       `/.well-known/assetlinks.json`, `security.txt`, CSP/HSTS),
       `vars.CLOUDFLARE_ACCOUNT_ID`, and `CLOUDFLARE_SITE_API_TOKEN` scoped
       strictly to `Account -> Cloudflare Pages: Edit` (unable to deploy
       Workers or read email Workers).
     - **`totaldrop-servers` (`cloudflare-workers`):** Locked `wrangler` +
       `probe-servers.mjs` (`rendezvous` and `relay`),
       `vars.CLOUDFLARE_ACCOUNT_ID`, `CLOUDFLARE_WAF_READ_TOKEN` (zone
       `http_ratelimit` drift check, #526), and `CLOUDFLARE_WORKERS_API_TOKEN`
       scoped strictly to `Account -> Workers Scripts: Edit` and `Zone
       (totaldrop.app) -> Workers Routes: Edit`.
     - **`totaldrop-android` (`play-release`):** `ANDROID_UPLOAD_KEYSTORE` and
       `ANDROID_UPLOAD_PASSWORD`.
     - **`totaldrop-linux` (`linux-release`):** Keyless Sigstore provenance
       plus the AUR SSH key ([0015](0015-paket-yoneticileri.md)).
     - **`totaldrop-core` (`core-release`) & `totaldrop-cli-bsd` (`cli-release`):**
       Crates.io Trusted Publishing OIDC once [0015](0015-paket-yoneticileri.md)
       step 5 activates, plus the Homebrew/Scoop GitHub App key and winget
       token in `totaldrop-cli-bsd`.
   - **Retiring `deploy.yml` and revoking monorepo secrets:** If
     `totaldrop-site` and `totaldrop-servers` cut over in separate PRs, the
     first cutover removes only its migrated target (`site` or
     `rendezvous`/`relay`) from `totaldrop-app/totaldrop`'s `deploy.yml`, and
     the second cutover deletes `deploy.yml` entirely, revokes the old
     combined `CLOUDFLARE_API_TOKEN` in Cloudflare, and deletes the
     `cloudflare` environment from `totaldrop-app/totaldrop`. Likewise,
     `android-release.yml` + `play-release` and `linux-release.yml` +
     `linux-release` are removed from `totaldrop-app/totaldrop` in the exact
     cutover commits that activate them in `totaldrop-android` and
     `totaldrop-linux`.

6. **Scope boundary (repository topology vs. unapproved stack/product choices):**
   - This decision approves **repository layout and governance only**. Initial
     GitHub repository descriptions do **not** pre-approve new UI stacks,
     platforms, or protocol changes; each requires its own accepted ADR before
     implementation:
     - **`totaldrop-web-wasm`:** A browser receiver conflicts with threat
       model `T9` (both servers reject WebSockets carrying an `Origin` header
       with HTTP 403). Remains unimplemented until a dedicated ADR and `T9`
       threat-model revision are accepted.
     - **`totaldrop-macos` and `totaldrop-ios-tvos` UI stack & tvOS:**
       [0009](0009-ios-macos.md) step 2 explicitly deferred choosing between
       native SwiftUI and Flutter for iOS and macOS, and did not include
       `tvOS` or Finder extensions; those require dedicated ADRs.
     - **`totaldrop-android` UI stack & TV:** Today's Android client is
       Flutter ([0011](0011-android-material-3-expressive.md)); any move to
       Jetpack Compose or Android TV requires its own ADR.
     - **BSD ports and RISC-V targets (`totaldrop-cli-bsd`, `totaldrop-linux`):**
       Packaging channels outside [0015](0015-paket-yoneticileri.md) (such as
       FreeBSD/OpenBSD source ports or new CPU architecture tiers) require an
       amendment to `0015` or their own ADR.

## Consequences

### Gains

- **Smaller credential blast radius (`T11`, `T12`):** Android upload keys,
  Cloudflare Pages tokens, Cloudflare Workers tokens, and package-manager
  credentials live in separate repositories and protected environments, and
  legacy combined tokens are revoked at cutover.
- **Independent platform CI and review queues:** Client, server, formal-proof,
  and site changes run only their own toolchains and runners, while required
  pre-tag downstream canaries, automated bump PRs, and attested
  `security-floor.json` checks preserve [0001](0001-tek-depo.md)'s defense
  against shipping a client on a vulnerable core and
  [0008](0008-diller-ve-bolgeler.md)'s single localization catalog.

### Costs and trade-offs

- **Multi-step core security fix rollout:** Trading away [0001](0001-tek-depo.md)'s
  single-PR workspace means a `totaldrop-core` vulnerability fix now requires
  cutting `core-vX.Y.Z` (in private `GHSA` forks first when embargoed),
  landing `N` downstream bump PRs, and cutting `N` platform releases before
  every shipped client is patched.
- **Cross-repository ordering for protocol changes:** Changes spanning
  `totaldrop-proto` / `totaldrop-core` and `totaldrop-servers` can no longer
  land in one atomic commit; they require ordered PRs across repositories and
  strict `N` / `N-1` server wire compatibility during client rollouts.
- **13-repository administrative load and bus-factor surface:** Two
  maintainers (`GOVERNANCE.md`) now administer `totaldrop-app/.github`,
  eleven modular repositories, and the archived `totaldrop-app/totaldrop`
  monorepo (rulesets, `CODEOWNERS`, Dependabot, CodeQL/OSV-Scanner, and
  release/deploy environments), increasing the bus-factor cost already noted
  in `GOVERNANCE.md` unless shared workflows and automated drift checks in
  `totaldrop-app/.github` enforce configuration uniformly.
- **Transition state:** [0001](0001-tek-depo.md) remains `Accepted` and
  `totaldrop-app/totaldrop` remains authoritative for each area until this
  record is accepted, that area's Phase 2/3 cutover PR lands, and Phase 4
  archival completes.
