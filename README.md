<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/lockup-dark.svg">
  <img src="docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Any file, to anyone, anywhere.

Free and open source. End-to-end encrypted. No email, no phone number, no account.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Website](https://totaldrop.app)** · **[Download](https://totaldrop.app/download/)** · [Roadmap](https://totaldrop.app/roadmap/) · [Donate](https://totaldrop.app/donate/) · [Releases](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](docs/readme/README.ar.md) · [বাংলা](docs/readme/README.bn.md) · [Čeština](docs/readme/README.cs.md) · [Dansk](docs/readme/README.da.md) · [Deutsch](docs/readme/README.de.md) · [Ελληνικά](docs/readme/README.el.md) · **English** · [Español](docs/readme/README.es.md) · [فارسی](docs/readme/README.fa.md) · [Suomi](docs/readme/README.fi.md) · [Français](docs/readme/README.fr.md) · [עברית](docs/readme/README.he.md) · [हिन्दी](docs/readme/README.hi.md) · [Magyar](docs/readme/README.hu.md) · [Bahasa Indonesia](docs/readme/README.id.md) · [Italiano](docs/readme/README.it.md) · [日本語](docs/readme/README.ja.md) · [한국어](docs/readme/README.ko.md) · [Bahasa Melayu](docs/readme/README.ms.md) · [Norsk bokmål](docs/readme/README.nb.md) · [Nederlands](docs/readme/README.nl.md) · [Polski](docs/readme/README.pl.md) · [Português (Brasil)](docs/readme/README.pt-BR.md) · [Português (Portugal)](docs/readme/README.pt-PT.md) · [Română](docs/readme/README.ro.md) · [Русский](docs/readme/README.ru.md) · [Svenska](docs/readme/README.sv.md) · [Kiswahili](docs/readme/README.sw.md) · [ไทย](docs/readme/README.th.md) · [Türkçe](docs/readme/README.tr.md) · [Українська](docs/readme/README.uk.md) · [اردو](docs/readme/README.ur.md) · [Tiếng Việt](docs/readme/README.vi.md) · [简体中文](docs/readme/README.zh-Hans.md) · [繁體中文](docs/readme/README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="docs/assets/android.png" alt="TotalDrop on Android: the home screen in light and dark, and a transfer that arrived and was verified" width="820">

</div>

## Why TotalDrop

- **A short code is all it takes.** Pick files and get a code like `7-harbor-rocket`. Read it out or show the QR code; the other person types it in and the files arrive.
- **Encrypted end to end, and you can check it.** Both screens show the same four words before anything moves. If they differ, stop.
- **Nothing to sign up for.** No account, email or phone number. Optional accounts will be numbers only, Mullvad-style ([design](docs/hesap.md)).
- **Straight from device to device, anywhere.** The rendezvous server only introduces the two devices and keeps no logs; like any server, it sees their IP addresses. Devices connect directly across the internet wherever they can. When a router or firewall is in the way, a relay carries traffic it cannot read.
- **At home on your system.** The apps follow the system's light or dark theme, accent colour, language and reduced-motion setting. The Linux app is native Qt 6 on Wayland and also takes its fonts, corners and icons from KDE Plasma, Hyprland and Omarchy themes, live.
- **Free, for good.** Open source and funded by donations. No ads, no tracking, no paid tier for the basics.

<p align="center">
  <img src="docs/assets/linux.png" alt="The Linux app on Omarchy, in the desktop's colours, fonts and icons" width="560">
  <br><sub>The Linux app on Omarchy: colours, corners, fonts and icons come from the desktop.</sub>
</p>

## Download

| Platform | Get it | Notes |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.x86_64.rpm) | 0.3.0-beta.1, Qt 6.8 or newer |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop_0.3.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.3.0-beta.1/totaldrop-0.3.0.beta.1-1.fc43.aarch64.rpm) | 0.3.0-beta.1, Qt 6.8 or newer |
| **Android** | [Join the beta](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.3.0-beta.1, Android 15 or newer, Google Play by invitation |
| **Command line** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | Coming | Code signing for these is what donations pay for first |

Every package is built by CI from a tagged commit, never on a developer's machine. To check one before installing it, compare it with the release's `SHA256SUMS` and verify where it was built:

```bash
gh attestation verify totaldrop_0.3.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** The code has not had an independent security review yet, so please don't rely on it for sensitive files.

## How it works

1. The sender asks the rendezvous server for a short code. The code is the only secret the two people share.
2. Both devices turn the code into a strong key with **CPace**. An attacker gets one guess per transfer and cannot test codes offline.
3. They connect directly over **QUIC** (TLS 1.3 with pinned raw public keys) and run **Noise XXpsk3 with ML-KEM-768**, which is designed to hold up against future quantum computers.
4. Files are encrypted in chunks (STREAM), so every piece is authenticated as it arrives.

The protocol is described in *TotalDrop Protocol v0*. Decisions made while implementing it are in [docs/protocol-v0.1-notes.md](docs/protocol-v0.1-notes.md), and what each server can and cannot see is in the [threat model](docs/tehdit-modeli.md) (Turkish) and the [privacy policy](https://totaldrop.app/privacy).

## In your language

TotalDrop speaks 35 languages and follows your region's number formats. English and Turkish are reviewed; the other 33 are drafts that native speakers are checking, so the apps keep the security texts (verification, wrong code, accept and decline) in English until a language is reviewed. The site and the apps have them all, from Android and Linux 0.3.0-beta.1.

Spotted a clumsy sentence? [Help translate](docs/ceviri.md) (the guide is in Turkish).

## Support the project

TotalDrop stays free because people chip in. Donations pay for the relay, the code-signing certificates that Windows and macOS require, and an independent security audit before 1.0. Every cost, line by line, and what has been raised so far: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Sponsor on GitHub"></a>

You can also help by [reporting bugs](https://github.com/totaldrop-app/totaldrop/issues/new/choose), translating, or testing on your desktop.

## For developers

### Repository layout

| Path | What it is | License |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, key derivation, mailbox and session encryption, device keys, STREAM chunk encryption | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Transfer codes, limits, path safety, every wire message (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Client engine: rendezvous, QUIC with raw public keys, Noise XXpsk3 + ML-KEM-768, transfer | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Interface text for the Rust programs: language negotiation, ICU messages, plurals, regional numbers | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | The `totaldrop` command-line client | GPL-3.0-or-later |
| `l10n/` | Every interface string, shared by Android, Linux and the CLI ([translating](docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Linux desktop app: Qt 6 and QML over `totaldrop-core`, native Wayland | GPL-3.0-or-later |
| `packaging/` | Linux packages, each format on its own: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Android app (Flutter); iOS later | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app and the page QR links open | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Fuzz targets for every parser | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Rendezvous server in Rust, for self-hosting and tests | AGPL-3.0-or-later |
| `server/rendezvous-worker` | The same server on Cloudflare Workers, for `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

The libraries are permissive so anyone can build on the protocol; the apps are GPL and the servers AGPL. See [LICENSING.md](LICENSING.md).

Each platform has its own version, tag (`linux-v…`, `android-v…`) and CI workflow, so one can ship without touching the others. How branches and releases work: [docs/gelistirme.md](docs/gelistirme.md) (Turkish) and [docs/versioning.md](docs/versioning.md). Design rules: [docs/tasarim.md](docs/tasarim.md) and architecture decisions: [docs/kararlar](docs/kararlar/README.md), both in Turkish.

### Build

```bash
cargo test
cargo build --release
```

Try a transfer on one machine, in three terminals:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Both sides show the same four verification words. Unencrypted `ws://` is accepted only for a server on the same machine; anything else must be `wss://`.

The project always targets the newest stable Rust (pinned in `rust-toolchain.toml`); rustup installs it on the first build.

**Linux app** (needs Qt 6.8 or newer: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Contributing and security

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request, and our [code of conduct](CODE_OF_CONDUCT.md). Questions go to [SUPPORT.md](SUPPORT.md). Please report vulnerabilities privately as described in [SECURITY.md](SECURITY.md), never in a public issue.
