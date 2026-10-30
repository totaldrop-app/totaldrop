# Changelog

All notable changes to TotalDrop are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/) as described in
[docs/versioning.md](docs/versioning.md).

## [Unreleased]

### Changed

- **Site, README:** The download page and the README point at Android and
  Linux 0.3.0-beta.1 and list what is new: the Köşe mark and icon, 35
  languages, right-to-left screens and steadier QR reading on Linux. The
  apps are no longer described as English and Turkish only, and the
  roadmap adds "In your language" to Done (#132, #137).

### Fixed

- **CLI, translations:** The translation check rejected only the English
  no-words (n, no, not) among a language's yes answers (`cliYes`), so a
  draft could make a word for no, such as нет, nein or لا, mean yes. Each
  language now lists its words for no in `l10n/status.json` (`noWords`),
  and the check rejects a yes answer that is one of them, or the answer
  for no in a save prompt it can be shown with (`[д/Н]`, `[ن/ل]`). A save
  prompt must show ASCII `[yes/no]` answers, and `cliYes` must accept its
  answer for yes. Answers are compared exactly as the command line
  compares them: `fold_answer`, `is_yes` and `ALWAYS_YES` are now public
  in `totaldrop-l10n`, and the command line uses them (#129).

## [android-0.3.0-beta.1] and [linux-0.3.0-beta.1] - 2026-10-30

### Added

- **GitHub:** The README is rewritten around the new mark, with a light and
  dark lockup, current screenshots and a section on languages, and it is
  translated into the site's 35 languages (docs/readme/), each listed under
  its own name. CI checks every translation's links and language list and
  reports translations older than the English. A code of conduct and a
  support page complete the community files (#135).
- **Android, Linux, CLI, site:** Draft translations in 31 more languages, 33
  with the Portuguese and Chinese variants (ADR 0008): Arabic, Bengali,
  Chinese (Simplified and Traditional), Czech, Danish, Dutch, Finnish,
  French, German, Greek, Hebrew, Hindi, Hungarian, Indonesian, Italian,
  Japanese, Korean, Malay, Norwegian Bokmål, Persian, Polish, Portuguese
  (Brazil and Portugal), Romanian, Russian, Spanish, Swahili, Swedish, Thai,
  Ukrainian, Urdu and Vietnamese. They are made with Claude and not yet
  checked by native speakers, so the apps say so in Settings and keep the
  security texts in English until a language is reviewed: verification,
  wrong code, invalid offer, accept and decline, and the command line's save
  prompt and yes answer. The site shows a short note at the top of the page
  in a draft language, lists every language under its own name, and picks
  Portuguese from Portugal outside Brazil, as the apps do; the picker is as
  wide as the chosen language. The comparison table now says no copy is
  stored on a company's server, instead of "files stay off company servers",
  which the relay made too broad (#127).

### Changed

- **Android, Linux, site:** New mark, "Köşe" (corner), replacing "Ağ" on
  Android and "Dalga" on Linux: a circle with one square corner. The circle is the
  whole file and the corner makes it a drop, pointing the way it goes. While
  a file is on its way the mark turns a quarter at a time, 3.2 s a loop, the
  same on the site, Android and Linux, and it stays still when the system
  asks for less motion. The app icons are the paper mark on violet; the done
  screen shows a check inside the mark (#133).
- **Site (download page, roadmap, home), README:** The download page and
  the README point at Linux 0.2.0-beta.1, with the ARM64 .deb and .rpm,
  and the page lists what is new in Android 0.2.0-beta.2 and Linux
  0.2.0-beta.1. The roadmap moves privacy mode in the apps and Linux on
  ARM to Done, and the home page's summary shows privacy mode in the apps
  as done (#122).
- **Site:** Privacy policy version 4, for Android and Linux 0.2.0-beta.1
  and later: devices contact the relay only when no direct connection is
  chosen about 1.2 seconds after they start connecting, or at once when
  there is no direct path to try; a receiver no longer uses a relay only
  the sender chose; privacy mode is in the apps' Settings, skips STUN and
  never connects directly; the apps also store the privacy mode setting,
  and Android keeps the files you pick in its cache until the send
  is closed or stopped. Android 0.1.0-beta.5 and Linux 0.1.0-beta.4, the
  first versions with the relay, are still described, with the advice to
  update (#124).
- **Linux, Android:** Dependencies: zbus 6.1, toml 1.1, dirs 7 and rqrr
  0.11 (QR codes with mixed block sizes now decode) in the Linux app; the
  Android Gradle plugin 9.5.

### Fixed

- **Android:** In right-to-left languages the home screen's mark, the
  "Add files" and "Use the default" buttons stayed on the left, the
  version showed as "beta.2-0.2.0", and the server address ran right to
  left. They now follow the language's direction, and the version and the
  save folder keep their own (#127).
- **Rendezvous (Cloudflare):** About 1 in 15 new codes failed with HTTP 500
  when Cloudflare returned a transient error on a new mailbox's first
  request. Opening a mailbox and looking up a code are now tried again on a
  fresh stub, and if every try fails the client gets a readable error
  instead of a 500 (#119).

## [android-0.2.0-beta.2] - 2026-10-29

### Changed

- **Android, Linux, site:** New mark, "Ağ" (network): three equal devices,
  each joined to the other two, and a drop on the way between them. While
  a transfer runs the drop visits each device in turn; with reduced motion
  it stays still. It replaces "Dalga", whose waves spread from one centre
  and did not show that every device sends to every other (#116). The
  Linux app gets it with its next release.

## [android-0.2.0-beta.1] and [linux-0.2.0-beta.1] - 2026-10-29

### Added

- **Android, Linux:** Privacy mode in Settings ("Hide my address from the
  other device"): every transfer goes through the relay, so the other device
  never learns your IP address. It is slower, and the app asks before
  turning it on. The command line has had it as `--relay-only` (#39).

- **Linux:** .deb and .rpm packages for ARM64 (Snapdragon X, NVIDIA RTX
  Spark). The command line is now also tested on ARM64 Linux and Windows
  (#86).

- **Command line:** Speaks the system's language (`LANGUAGE`, `LC_ALL`,
  `LC_MESSAGES`, `LANG`) and writes numbers and sizes in the region's format
  (`LC_NUMERIC`), for example "1,5 MB". English and Turkish today; more
  languages follow. Right-to-left languages stay English in the terminal,
  and `--help` and detailed error messages are still English.
- **Command line:** File names from the other side show hidden characters
  (controls, direction and other format characters) as `<U+202E>`, so a
  name cannot disguise its extension.
- **Linux:** The app takes its text from `l10n/app`, the same strings as
  Android, in the desktop's language, with sizes and counts in the region's
  format. The layout is ready for right-to-left languages (it mirrors, and
  the code and the verification words stay left to right); tested with a
  pseudo-locale until the first such translation lands. File names and
  cancel reasons from the other side show hidden direction characters.
  Settings says when a translation is still a draft. Packages recommend the
  Noto fonts for scripts beyond Latin.
- **Linux:** Sizes now read like on Android: one decimal below 10 and SI
  "kB" ("12 kB", "1.5 MB"), where they had one decimal below 100 and "KB".
- **Android:** Chooses the language like the Linux app and the command line
  (once those languages exist, `zh-HK` will get Traditional Chinese and
  `pt-AO` European Portuguese), and
  lists the app's languages in Android 13+ per-app language settings. Sizes
  follow the device's region ("1,5 MB"), counts the app language
  ("1.234 öğe"). The code and the
  verification words stay left to right and are read by screen readers with
  an English voice. Names from the other side show hidden direction
  characters. Safety-critical text stays English while its translation is
  an unreviewed draft, and Settings says when a translation is a draft.
- **Site:** A language picker replaces the EN/TR button; the language
  follows the browser (`navigator.languages`, with likely scripts, so
  `zh-HK` would find Traditional Chinese) and translations load from
  `/i18n/<lang>.json` on the same domain. Pages are ready for right-to-left
  translations (CSS logical properties; the direction comes from the
  language's script), with codes and commands kept left to right; tested
  with a temporary Arabic file until a real one lands. English text shown
  inside another language keeps its own language and direction. Receive links accept Turkish İ/ı and
  Arabic-Indic digits in the code.
- **Core:** `totaldrop-l10n`, the shared interface text for the Rust
  programs: all strings live in `l10n/`, one source with Android.

### Fixed

- **Relay (Cloudflare):** The 32 GiB limit per connection no longer starts
  again from zero when the relay object sleeps; the count is kept with the
  connection, losing at most 1 MiB each time the object wakes (#110).

- **Command line:** Failures are explained in the system's language, in the
  same words as the apps, with the technical detail on a second line. A
  code that is not in use and a bad server address get their own message
  instead of "check your internet connection" (#106).

- **Linux:** Adding a large folder no longer freezes the window: its size
  is added up in the background and shows "Counting…" until then (#103).

- **Android, Linux:** Stop works at every stage: while the files are
  prepared, while waiting for the receiver, while connecting and during the
  transfer. Before, stopping before the transfer began did nothing and the
  other side later saw a stale offer or a connection error; now it is told
  that the transfer was stopped, and leaving an unanswered offer declines
  it (#87).

- **Command line:** Ctrl-C stops a transfer cleanly: once connected, the
  other side is told and unfinished files are removed; a second Ctrl-C quits
  at once and may leave them. Ctrl-C at the "Save?" question declines.
  `receive` checks that the destination exists, is a folder and can be
  written before it uses up the code (#87).

- **Core:** Stopping a transfer now tells the other device, which reports
  that the other side stopped instead of a connection error. A stopped or
  failed receive no longer keeps writing in the background: unfinished
  files are removed with the empty folders the transfer created, and no
  file is placed afterwards. A sender waits at most 10 minutes for the
  receiver's answer (#87).

- **Relay (self-hosted):** The Caddy configuration now passes
  `/v0/relay/<keys>` to the relay; it matched only the exact path
  `/v0/relay`, so no relay connection could open. The server's name is now
  an argument of `install.sh` instead of `relay.totaldrop.app` (#84).
- **Relay:** A device signing in no longer pushes back the sign-in
  timeout of the other socket in the same pair, and a finished pair leaves
  no alarm behind (#75).
- **Core:** The apps now connect to the relay only when it is needed: when
  a device has no direct connection 1.2 seconds after pairing, or at once
  in privacy mode. A transfer whose direct path is up within that time no
  longer reaches the relay at all, so the relay no longer sees the IP
  addresses of devices it does not carry; when a direct path comes up later
  and wins, the relay connection is closed as soon as the direct path is
  chosen (#63).
- **Core, CLI:** A receiver now uses the relay the sender names only when
  it is also the receiver's own relay setting, so a sender running its own
  relay can no longer learn the receiver's IP address that way. An empty
  `--relay` turns the relay off when receiving too. When different relays
  leave no path (or in privacy mode), the receiver stops at once with an
  error that says so, and so does the sender when it can tell, instead of
  timing out. A sender with this version no longer pairs with an older
  receiver that would have used a relay it does not use itself (#68).
- **Site:** The privacy policy (version 3) now says that the apps connect
  to the relay at the start of every transfer, not only when no direct path
  works, as version 2 said, and that in privacy mode a receiver still uses
  the relay the sender names. The android-0.1.0-beta.5 notes above said the
  relay sees "only single-use keys, sizes and timing"; it also sees both IP
  addresses. A coming version will connect only when needed (#63, #68).
- **Android:** The privacy text in Settings said files always go directly
  between the devices and only the rendezvous server sees your IP address;
  it now names the relay, STUN and the other device too (#63).

### Security

- **Rendezvous (Cloudflare):** A connection to a mailbox that has not
  claimed a side within 10 seconds is closed, and at most two such
  connections wait at once (a newer one replaces the oldest); before,
  anyone who knew a nameplate in use could hold idle connections on it until
  it expired. The directory hands out
  nameplates from a pool instead of reading every nameplate on each request
  (GHSA-r9gc-m8c8-2xqw).

- **Servers (self-hosted):** The Rust relay accepts only the pair path and
  forwards datagrams only to the other key of the pair, as the Cloudflare
  relay does; a ping-only connection is closed after 60 seconds without
  datagrams. The Rust rendezvous server closes a connection that does not
  allocate or claim within 10 seconds, pings or not, and counts a failed
  code once instead of twice toward the attack-wave threshold
  (GHSA-57x6-rcg9-m954).

- **Core, command line:** In privacy mode (`--relay-only`, and the apps'
  new setting) a device refuses a direct connection even from the paired
  device, which could only dial an address it learned some other way; the
  transfer stays on the relay (#39).

- **Build:** A Linux release starts only from a tag on `main` whose version
  matches `linux/Cargo.toml`, the PKGBUILD, the RPM spec, the Debian
  changelog and the AppStream metainfo; every PR checks that the files
  agree. All packages are built with the pinned Rust (the Arch package used
  Arch's own), the Android bundle with a pinned Flutter, from base images
  pinned by digest, and the source archive carries the last commit's time
  (#90).

- **Build:** Linux packages are built only in CI from a release tag and
  carry signed build provenance, so anyone can check that a package came
  from this repository: `gh attestation verify <file> --repo
  totaldrop-app/totaldrop`. The signing workflow is
  `.github/workflows/linux-release.yml`. The Android Play bundle is also built and
  signed with the upload key in CI; Google Play re-signs it, so for
  Android the provenance is for the project's own records.
- **Build:** Every GitHub Action in CI is pinned to a full commit SHA, so
  a moved action tag cannot change which action code runs. Dependabot proposes the
  updates, which are reviewed like any other change. `actions/checkout`
  moved to v7 and `actions/setup-java` to v6.
- **Build:** One `CI` check now runs on every pull request and is required
  to merge into `main`. It runs the workflows of the areas that changed and
  fails if any of them fails. Releases build from their own workflows,
  which run the same checks first.

## [android-0.1.0-beta.5] and [linux-0.1.0-beta.4] - 2026-10-28

### Added

- **Core:** transfers across the internet. Each device asks a STUN server
  (Cloudflare's by default) for its public address and both try to reach
  each other directly (NAT traversal); when no direct path opens, the
  encrypted QUIC packets go through the sender's relay over WebSocket on TCP
  443, several to a message. The relay sees only single-use keys, sizes and
  timing, and logs nothing.
- **Core, command line:** privacy mode, `--relay-only`: no IP address is
  announced and everything goes through the relay.
- **Server:** the relay on Cloudflare Workers (`relay.totaldrop.app`), one
  Durable Object per transfer that admits only its two keys; and
  `totaldrop-relay`, the same relay for self-hosting, with a STUN responder.

## [linux-0.1.0-beta.3] - 2026-10-28

### Fixed

- **Site:** The Turkish download page showed last release's version numbers;
  CI now checks that versions in a translation match English.

- **Linux:** sending no longer fails when this computer's firewall blocks
  incoming connections (as Omarchy's ufw does by default). Built on the core
  fix released in android-0.1.0-beta.4.

## [android-0.1.0-beta.4] - 2026-10-28

### Fixed

- **Core:** a transfer no longer fails when a firewall blocks incoming
  connections on the sending device (as Omarchy's ufw does by default). Both
  sides now listen and dial; the first connection that comes up is used.

## [android-0.1.0-beta.3] - 2026-10-27

### Changed

- **Android:** while sending, the mark plays the website's animation: the
  file travels the channel and a ring spreads from the drop.

## [linux-0.1.0-beta.2] - 2026-10-27

### Added

- **Linux:** first desktop app, Qt 6 and QML, native Wayland. Send with the
  desktop's file chooser or by drag and drop, a code and a QR code; receive
  by typing the code with word completion, pasting it, reading a QR code from
  an image or opening a TotalDrop link. Desktop notifications when a
  transfer ends in the background.
- **Linux:** follows the desktop's look: the Omarchy theme (live), the
  portal's colour scheme and accent, the Plasma palette with Breeze controls
  and icons, and Hyprland's corner rounding.
- **Linux:** separate packages for Arch, Debian/Ubuntu (.deb) and
  Fedora (.rpm).
- **Site:** `totaldrop.app/r#code`, the page TotalDrop QR codes and links
  open: it shows the code and opens the app with it. The code stays in the
  browser. The download page offers the Linux packages.

## [0.1.0-beta.2] - 2026-10-27

### Changed

- New mark, "Dalga": a drop at the centre, waves spreading from it, and the
  file leaving through a channel in the waves. It replaces the orbit mark in
  the app, the launcher icon and the site. While sending, the file travels
  the channel; with reduced motion the mark stays still.

## [0.1.0-beta.1] - 2026-10-27

First beta. Protocol v0 with the clarifications in
[docs/protocol-v0.1-notes.md](docs/protocol-v0.1-notes.md).

### Added

- Transfer by code between two devices that can reach each other directly
  (same machine or same network), end to end encrypted:
  CPace over ristretto255, Noise XXpsk3 with an ML-KEM-768 hybrid step,
  QUIC pinned to single-use raw public keys, STREAM-encrypted chunks
  verified with BLAKE3.
- Command-line client `totaldrop send` / `totaldrop receive`.
- Rendezvous server in Rust (self-hosting) and on Cloudflare Workers,
  serving `rendezvous.totaldrop.app`.
- Android app (Flutter, Material 3, follows the system colours), in English
  and Turkish.

### Security

- Issues found by the first internal security review were fixed before
  this release; see the commit "Fix issues from the security review".
