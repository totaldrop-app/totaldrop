# 0015: Package managers rest first on CLI binaries built in CI

- **Status:** Accepted, 8 November 2026 (Ozan, #344)
- **Context:** Today TotalDrop is installed in two ways: the Linux packages
  on the release page (Arch, Debian, Fedora; Flathub is on the way,
  [0014](0014-steam-deck.md)) and `cargo install --git`. The CLI has no
  prebuilt binary; macOS and Windows users have to install the Rust
  toolchain for the CLI. We are not on Homebrew, winget, Scoop, the AUR or
  crates.io. On 7 November 2026 the names in these places were free: on
  crates.io `totaldrop`, `totaldrop-cli`, `-core`, `-crypto`, `-proto`,
  `-l10n`; on the AUR `totaldrop`; on winget the `TotalDrop` publisher
  folder. If someone else takes them first, users may mistake a package
  that TotalDrop did not publish for TotalDrop
  ([0012](0012-ad-ve-isaret.md)).

  The project runs on donations; no paid services are used
  ([0005](0005-bagis.md), [0009](0009-ios-macos.md)). Packages are built
  only in CI, from a tagged commit on `main` ([surec.md](../surec.md), the
  release list; [versioning.md](../versioning.md)). The only exception is
  Flathub: Flathub builds the package on its own infrastructure, but the
  same recipe is also built in our CI on every PR (0014).

- **Options:**
  1. Leave package managers to the community. The brand rule allows Linux
     distributions and the AUR, but where the publisher appears as the
     developer (winget, the Store), someone else takes the identity. We
     cannot check which binary the package installs.
  2. Each channel builds from source on its own infrastructure (Homebrew
     bottles, COPR, OBS). The binary that reaches users does not come out
     of our CI. Build provenance (attestation) and the reproducibility check
     do not cover these binaries.
  3. **CI is the single source: the release workflow produces the binaries,
     the checksums and the build provenance; the channels only keep a
     recipe that points to these files. Channels that build on their own
     infrastructure are exceptions and are named here.**
- **Decision:** Option 3.
- **Why:** Whichever channel it comes from, the file the user installs is
  the same as the file on the release page. `SHA256SUMS` and
  `gh attestation verify` give the same result on every channel. The
  reproducibility check covers these files too. The channel recipes are a
  few lines listing addresses and checksums, with no build logic. Taking
  over a channel cannot change the binary; it can only point to another
  address, and the checksum and the provenance catch that. Option 1 loses
  the identities. Option 2 means a separate build chain and a separate
  trust on every channel.

## The common base: CLI binaries

Every channel rests on the same archives. A new release workflow produces
them:

| Target | Runner | Archive |
| --- | --- | --- |
| `x86_64-unknown-linux-musl` | `ubuntu-24.04` | `.tar.gz` |
| `aarch64-unknown-linux-musl` | `ubuntu-24.04-arm` | `.tar.gz` |
| `universal-apple-darwin` (`aarch64` and `x86_64`, one file with `lipo`) | `macos-15` | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `windows-2025` | `.zip` |
| `aarch64-pc-windows-msvc` | `windows-11-arm` | `.zip` |

- The Linux binaries are statically linked with musl and run independently
  of the distribution. The protocol's cryptography is built on RustCrypto
  (`chacha20poly1305`, `curve25519-dalek`, `ed25519-dalek`, `ml-kem`),
  `snow` and `blake3`. `ring` is only for TLS and QUIC (`rustls`,
  `quinn`). `ring` supports musl, but needs a C compiler.
- The Windows binaries link the C runtime statically
  (`-C target-feature=+crt-static`). Otherwise the binary needs
  `vcruntime140.dll`, and the zip that winget or Scoop unpacks does not run
  on a clean machine.
- The macOS binary is ad-hoc signed again after `lipo` and `strip`. CI
  checks the signature with `codesign -v`: on Apple Silicon, code with no
  signature or a broken one does not run.
- The archive name is `totaldrop-cli-<version>-<target>`. It holds the
  `totaldrop` binary, the licence files and the README.
- The release comes out with `SHA256SUMS`, build provenance and a
  `.sigstore.json` file, as the Linux packages do.
- The tag is `cli-v<version>` (today `0.2.0-beta.5`). The CLI's version is
  separate from the Linux app's, and package managers must see the CLI's
  version. The `cli-v…` tag is optional in
  [versioning.md](../versioning.md) today; it becomes mandatory for the CLI
  binaries. The CLI is still released together with the Linux release: if
  the CLI's version was raised, a `cli-v…` tag is also put on the same
  commit. The release list in `surec.md` and `versioning.md` are updated
  with this step, in the PR that brings it in.
- The workflow is set up like `linux-release.yml`. It first checks that
  the tag is on `main` and that the versions match the tag
  (`packaging/check-cli-version.sh`). Then a draft release is opened in an
  approved environment.
- It only partly meets #86. #86 asks for building and testing in CI on
  every PR for `aarch64-unknown-linux-gnu` and `aarch64-pc-windows-msvc`.
  This decision builds the ARM64 binaries with musl, and only on a release
  tag. Building and testing ARM64 on PRs remains #86's work.

## When the channels are updated

The release workflow opens the release as a draft. A draft's files cannot
be downloaded publicly, and nobody has reviewed the draft yet. That is why
the channel updates are not part of the release workflow. A separate
workflow (`channels.yml`) runs on the `release: published` event, that is,
when a maintainer publishes the draft. Each channel is a separate job; the
failure of one does not stop the others. The jobs run in protected
environments (below, "Secrets"). If a channel fails, its job is rerun by
hand.

## Channels

| Order | Channel | What |
| --- | --- | --- |
| 1 | — | The CLI binaries and the `cli-v…` tag (above) |
| 2 | Homebrew tap | `totaldrop-app/homebrew-tap`, the `totaldrop-cli` formula |
| 2 | Scoop | `totaldrop-app/scoop-bucket`, `totaldrop-cli` |
| 3 | AUR | `totaldrop` (from source) and `totaldrop-bin`; with the Linux release |
| 4 | winget | `TotalDrop.CLI` |
| 5 | crates.io and `cargo binstall` | `totaldrop-cli` and the four crates it depends on; once the blockers are gone |
| 6 | Microsoft Store | The Windows app: [0016](0016-windows-microsoft-store.md) |
| 6 | Homebrew cask | The macOS app; with an Apple account and notarization (0009, step 5) |
| 6 | homebrew-core | After 1.0 |
| — | Flathub | 0014 |
| — | APT and RPM repositories | A separate decision |

Names: the CLI is called `totaldrop-cli` on every channel, and the command
is `totaldrop`. That way the macOS app's cask can later take the name
`totaldrop`; `brew install totaldrop` does not stay ambiguous. On the AUR,
`totaldrop` is the same as the Arch package, that is, the app and the CLI
together.

### crates.io and cargo binstall

- For `cargo install totaldrop-cli`, `totaldrop-cli` and the crates it
  depends on, `totaldrop-core`, `-crypto`, `-proto` and `-l10n`, are
  published.
- **First publication:** crates.io's Trusted Publishing route (GitHub OIDC)
  cannot create new crates. So the first publication of the five crates is
  also done from CI, in the `cli-release` environment. A short-lived
  crates.io token is used for it: its scope is only `publish-new`, limited
  to names matching `totaldrop*`. Right after, Trusted Publishing is set up
  for every crate and the token is revoked. No long-lived token is kept for
  later publications.
- **Blocker 1:** `totaldrop-l10n/build.rs` reads the translation files from
  outside the crate (`../../l10n`). `cargo package` only puts the crate
  folder into the package; as it is, the published crate does not build.
  The fix is chosen in a separate piece of work. Options: copying the
  translation files into the crate while packaging, or putting the
  generated Rust code into the crate.
- **Blocker 2:** The CLI's development dependencies (`totaldrop-relay`,
  `totaldrop-rendezvous`) are written with both a path and a version. A
  versioned development dependency must exist on crates.io; the servers,
  however, are not published. Either the version field is removed and
  `tools/check-component-versions.sh` changes accordingly, or the release
  removes them before `cargo publish`.
- **Blocker 3:** The crate folders have no licence files; the texts are
  under `LICENSES/` at the root. Every published crate must carry its own
  licence texts: MIT and Apache 2.0 for `-core`, `-crypto`, `-proto` and
  `-l10n`, GPL 3.0 for `-cli`.
- No placeholder is published while the names are free: crates.io's rules
  do not want empty crates published to reserve names. The real crates are
  published once the blockers are gone. The risk that someone else takes
  the names in the meantime is accepted.
- `[package.metadata.binstall]` is added to `totaldrop-cli`. Because the
  tag prefix is `cli-v`, `pkg-url` is written out explicitly; for macOS the
  target name is `universal-apple-darwin`. `cargo binstall totaldrop-cli`
  downloads our CI binary without building.

### Homebrew

- **Tap:** the `totaldrop-cli` formula in the `totaldrop-app/homebrew-tap`
  repository. `brew install totaldrop-app/tap/totaldrop-cli` downloads our
  CI archive on macOS (Apple Silicon, Intel) and Linux, checks the checksum
  and installs the `totaldrop` binary. There are no bottles; Homebrew
  builds nothing.
- On macOS the CLI binary is only ad-hoc signed (0009). We expect Homebrew
  not to put the quarantine flag on formula downloads, and the binary to
  run without being stopped by Gatekeeper. **To be verified:** tried on
  macOS 15 and 26 with the first release.
- 0009 says "an account-free Homebrew install does not count as a route".
  This decision narrows that sentence to app packages, that is, casks; the
  CLI formula is a route even without an account (Consequences).
- **Cask:** since Homebrew 5.0, the official `homebrew/cask` repository
  does not accept casks that fail the Gatekeeper check, and since
  September 2026 it disables them. Even if a cask in our own tap is exempt
  from this rule, macOS still quarantines it; the user faces the same
  cumbersome approval steps. So the macOS app's cask comes after the
  Developer ID signature and notarization (0009, step 5).
- **homebrew-core:** Homebrew wants stable releases; beta releases cannot
  enter `homebrew/core`. We apply after 1.0. At the time of applying,
  Homebrew's notability criteria are read again.
  - homebrew-core builds from source on its own infrastructure. This is an
    exception (below).
  - The formula is in Homebrew's repository; Homebrew's maintainers and
    bots change it. So the condition "the same recipe is also built in our
    CI on every PR" cannot be fully met as it is for Flathub. Instead, a
    mirror of the formula is built with `brew install --build-from-source`
    on every PR. The bottle that reaches users is built by Homebrew and
    signed with its own key; it does not carry our provenance. The user
    trusts Homebrew; this is written into the threat model, as for Flathub
    in T15.

### Scoop

- A JSON manifest for `totaldrop-cli` in the `totaldrop-app/scoop-bucket`
  repository. It points to our `.zip` archive and its checksum.
- There is no outside review; the bucket is ours. `channels.yml` updates
  the manifest.

### winget

- Manifests enter the `microsoft/winget-pkgs` repository through a PR;
  Microsoft's validation pipeline and reviewers merge them.
- **CLI:** the `TotalDrop.CLI` identifier. The installer type is `zip`, the
  nested installer type `portable`. winget unpacks the binary into its own
  folder and links the `totaldrop` command into the PATH.
- **App:** the Windows app is distributed only through the Microsoft Store
  ([0016](0016-windows-microsoft-store.md)). winget installs it from the
  `msstore` source, with the Store product ID. The `TotalDrop.TotalDrop`
  identifier in the community repository needs an installer address;
  because there is no package outside the Store, it is not published
  (0016).
- On winget, the manifest's `Publisher` field and the identifier's prefix
  show TotalDrop as the publisher. So winget joins the places where the
  publisher appears as the developer. There, only TotalDrop publishes
  TotalDrop; the list in 0012 and `marka.md` is updated accordingly
  (Consequences). The project takes the `TotalDrop.CLI` identifier before
  someone else does.
- The first manifest is submitted by hand, with `komac` or `wingetcreate`.
  `channels.yml` submits the later releases.
- Signing: winget accepts unsigned binaries, but Windows SmartScreen and
  Defender may warn. Because the CLI binary does not go through the Store,
  the Store's signature (0016) does not cover it. The free route is the
  SignPath Foundation's open-source signing programme. Its conditions: an
  OSI licence, a public repository, a CI build whose source is verified,
  two-factor authentication in the team. The certificate shows "SignPath
  Foundation" as the publisher, not TotalDrop. Whether that fits the brand
  rule (0012) is evaluated separately. Paid signing services are not an
  option under 0005. The CLI comes out unsigned in its first releases.
- **To be verified:** whether winget sorts a version string such as
  `1.0.0-beta.1` before `1.0.0`. With today's `0.x-beta` versions the order
  causes no problem; the question matters for the betas of 1.0.

### Microsoft Store

- The Windows app's distribution, the Store account, the publisher name and
  the signature are the subject of
  [0016](0016-windows-microsoft-store.md). This decision does not repeat
  what is decided there.
- The only consequence that concerns this decision: there is no separate
  winget manifest or Scoop package for the Windows app. Scoop and winget
  only have the CLI.

### AUR

- **`totaldrop`:** built from source from our `packaging/arch/PKGBUILD`;
  the app and the CLI come together. The PKGBUILD in the repository has
  `sha256sums=('SKIP')`. The AUR accepts `SKIP` only for VCS sources. So
  when `channels.yml` submits to the AUR, it writes the real SHA-256
  checksum of the tag archive.
- **`totaldrop-bin`:** unpacks and installs the `.pkg.tar.zst` from the
  release, built by our CI, and checks its checksum. Not a single built CLI
  binary but the whole package, with the app, is installed. It is only for
  `x86_64`: we build the Arch package only for this architecture. It
  carries `provides=(totaldrop)` and `conflicts=(totaldrop)`.
- Omarchy and Arch users install with AUR helpers (`yay`, `paru`).
- The user builds the `totaldrop` package on their own machine. It is not
  a binary we publish; it is a source distribution. It does not break the
  rule.

### APT and RPM repositories, others

- `apt install` and `dnf install` need a signed repository. Options: our
  own repository on Cloudflare R2 ([0003](0003-cloudflare-altyapi.md)),
  Fedora COPR, openSUSE OBS. The repository signing key and the update
  route are a separate decision.
- Left to the community: nixpkgs, Chocolatey, MacPorts and the Linux
  distributions' own repositories. The brand rule allows them (0012).
- On the Snap Store, only TotalDrop publishes TotalDrop (0012). Snap is not
  planned for now.
- F-Droid is Android's business; it is dealt with in a separate issue.

## Exceptions to the "built only in CI" rule

- Channels that install our CI output are not exceptions: the Homebrew
  tap, Scoop, winget, AUR `totaldrop-bin`, `cargo binstall`.
- Channels that distribute source are not exceptions: crates.io and AUR
  `totaldrop`. The user builds on their own machine.
- Channels that build on their own infrastructure and distribute binaries
  are exceptions: Flathub (0014), later homebrew-core, and COPR and OBS if
  chosen. Each builds only from the tagged source. If the recipe is in our
  repository (Flathub, COPR, OBS), the same recipe is also built in our CI
  on every PR. If the recipe is in the channel's repository
  (homebrew-core), a mirror of it is built on every PR, and the fact that
  the user trusts the channel is written into the threat model.
- **Changing the rule needs Ozan's approval.** `CLAUDE.md` and `surec.md`
  say "the only exception is Flathub" today. This decision turns the
  exceptions into a list. The two files are updated in the PR that first
  brings in such a channel (homebrew-core, or COPR/OBS). None of the first
  five steps in the order is an exception; until then the two files do not
  change.

## Secrets

- The channel jobs run only for published releases and in an environment
  that asks for approval. A `cli-release` environment is opened for the
  CLI channels; `linux-release` is used for the AUR. Pull requests from
  forks do not see the secrets.
- **crates.io:** for the first publication, a short-lived token scoped to
  `publish-new` (above), then Trusted Publishing.
- **Homebrew tap and Scoop bucket:** a GitHub App instead of long-lived
  deploy keys. The App is installed only on these two repositories and has
  only permission to write contents. The job gets a one-hour token with
  `actions/create-github-app-token`. The App's private key is in the
  environment's secret.
- **AUR:** an SSH key registered only with the AUR and used nowhere else.
  The project owner owns the AUR account; at least one co-maintainer is
  added, so that the package does not depend on a single person.
- **winget:** opening a PR needs a GitHub token that can fork
  `microsoft/winget-pkgs`. A separate bot account is opened for this token.
  - The account has no permissions on the TotalDrop repositories, and
    two-factor authentication is on.
  - The token is scoped only to `public_repo` and has a limited lifetime;
    it is renewed before it expires.
  - The bot account signs winget-pkgs' contributor licence agreement (CLA)
    once.
- None of them is written to the repository or to a development machine.

## Order

1. The CLI binaries, the `cli-v…` tag and the release workflow.
2. `channels.yml`, the Homebrew tap and the Scoop bucket.
3. AUR: `totaldrop` and `totaldrop-bin`.
4. winget: `TotalDrop.CLI`.
5. crates.io and `cargo binstall`, once the three blockers are gone.
6. The Windows app: the Store (0016). The macOS app: a cask (0009).
   homebrew-core after 1.0.

Each step comes with its own issue and PR. The download table in the README
and on the site is updated in the same PR.

## Risks

- winget PRs depend on Microsoft's review; a release may take days to reach
  winget.
- Unsigned Windows binaries get a SmartScreen warning. This lowers trust in
  a privacy-focused app.
- Every channel adds a secret and an attack surface. A taken-over tap,
  bucket or AUR package can direct users to another binary. `SHA256SUMS`
  and `gh attestation verify` catch this, but only if the user checks. The
  threat model writes this down (Consequences).
- The names are free now, but not taken. The crates.io names stay open
  until the blockers are gone.

## Consequences

- The CLI tag and the channel steps are added to the release list in
  `surec.md` and to `versioning.md`. This is done in the PR of step 1.
- The channels' recipes, and how they are tried, are added to
  `docs/gelistirme.md`.
- **Threat model:** T12 is updated in the PR of step 2.
  - The channels in this decision are added to the list of official
    channels.
  - The channels' keys and accounts are written down as assets: the GitHub
    App key, the AUR SSH key, the winget bot account, the crates.io
    owners.
  - How to check the file each channel installs with
    `gh attestation verify` is written down.
  - When homebrew-core arrives, trust in Homebrew is written down like
    Flathub in T15.
- **0009:** the sentence "an account-free Homebrew install does not count
  as a route" is narrowed to casks; a note pointing to this decision was
  added to 0009.
- **0012 and `marka.md`:** winget is added to the list of places where the
  publisher appears as the developer. This is done in the PR of step 4.
- **Rule change:** the phrase "the only exception is Flathub" in
  `CLAUDE.md` and `surec.md` is turned into a list of exceptions. This
  needs Ozan's approval and is done in the PR that first brings in such a
  channel (above).
- **When to look again:** if Homebrew starts to require a signature or
  notarization for formulae too; if winget starts to reject unsigned
  binaries; if the SignPath programme closes or its conditions change;
  once the Apple account is taken out.

## Sources

- Homebrew, acceptable formulae: [docs.brew.sh](https://docs.brew.sh/Acceptable-Formulae)
- Homebrew, acceptable casks: [docs.brew.sh](https://docs.brew.sh/Acceptable-Casks)
- Homebrew 5.0 and Gatekeeper: [Workbrew](https://workbrew.com/blog/what-homebrew-5-0-0-means-for-your-mac-fleet)
- crates.io Trusted Publishing: [crates.io](https://crates.io/docs/trusted-publishing)
- Free individual registration on the Microsoft Store: [Windows Developer Blog](https://blogs.windows.com/windowsdeveloper/2025/09/10/free-developer-registration-for-individual-developers-on-microsoft-store/)
- SignPath Foundation's conditions: [signpath.org](https://signpath.org/terms)
- winget manifest types: [winget-types](https://docs.rs/winget-types/latest/winget_types/installer/struct.InstallerManifest.html)
