<div align="center">

<img src="packaging/linux/app.totaldrop.TotalDrop.svg" width="96" height="96" alt="">

# TotalDrop

**Any file, to anyone, anywhere.**<br>
Free, open source, end-to-end encrypted. No email, no phone number, no account.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

[Website](https://totaldrop.app) · [Download](https://totaldrop.app/download/) · [Roadmap](https://totaldrop.app/roadmap/) · [Donate](https://totaldrop.app/donate/) · [Releases](https://github.com/totaldrop-app/totaldrop/releases) · [Türkçe](#türkçe)

</div>

<p align="center">
  <img src="docs/assets/android.png" alt="TotalDrop on Android: the home screen in light and dark, and a short code waiting for the receiver" width="820">
</p>

## Why TotalDrop

- **A short code is all it takes.** Pick files, get a code like `7-harbor-rocket`, read it out or show the QR. The other side types it and the files arrive.
- **Encrypted end to end, checked by you.** Both screens show the same four words before anything moves. If they differ, stop.
- **Nothing to sign up for.** No account, email or phone number. Optional accounts will be numbers only, Mullvad-style ([design](docs/hesap.md)).
- **Straight from device to device, anywhere.** The rendezvous server only introduces the two devices and keeps no logs. Devices connect directly across the internet wherever they can; when a router or firewall is in the way, the relay carries traffic it cannot read.
- **At home on your desktop.** The Linux app is native Qt 6 and Wayland and follows your theme on KDE Plasma, Hyprland and Omarchy, live.
- **Free, for good.** Open source and funded by donations. No ads, no tracking, no paid tier for the basics.

<p align="center">
  <img src="docs/assets/linux.png" alt="The Linux app following two desktop themes: TotalDrop's own Aurora theme and an Omarchy theme" width="820">
  <br><sub>The Linux app takes its colours, corners and icons from the desktop: here TotalDrop's own theme and an Omarchy theme.</sub>
</p>

## Download

| Platform | Get it | Notes |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | Version 0.2.0-beta.1, Qt 6.8 or newer |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | Qt 6.8 or newer; .deb and .rpm only |
| **Android** | [Join the beta](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | Version 0.2.0-beta.2, Android 15 or newer, on Google Play by invitation |
| **Command line** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| Windows, macOS, iOS | Coming | Signing for these is what donations pay for first |

Every Linux release carries a `SHA256SUMS` file.

> [!NOTE]
> **Beta.** The code has not had an independent security review yet, so please don't rely on it for sensitive files.

## How it works

1. The sender asks the rendezvous server for a short code. The code is the only secret the two people share.
2. Both devices turn the code into a strong key with **CPace**: an attacker gets a single guess per transfer and cannot test codes offline.
3. They connect directly over **QUIC** (TLS 1.3 with pinned raw public keys) and run **Noise XXpsk3 with ML-KEM-768**, which stays safe against future quantum computers.
4. Files are encrypted in chunks (STREAM), so every piece is authenticated as it arrives.

The protocol is written down in *TotalDrop Protocol v0*; decisions made while implementing it are in [docs/protocol-v0.1-notes.md](docs/protocol-v0.1-notes.md).

## Support the project

TotalDrop stays free because people chip in. Donations pay for the relay, the code-signing certificates Windows and macOS demand, and an independent security audit before 1.0. Every cost, line by line, and what has been raised so far: [totaldrop.app/donate](https://totaldrop.app/donate/).

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

Each platform has its own version, tag (`linux-v…`, `android-v…`) and CI workflow, so one can ship without touching the others. How branches and releases work: [docs/gelistirme.md](docs/gelistirme.md) (Turkish) and [docs/versioning.md](docs/versioning.md). Design rules: [docs/tasarim.md](docs/tasarim.md).

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

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. Please report vulnerabilities privately as described in [SECURITY.md](SECURITY.md), never in a public issue.

## Türkçe

**Her dosyayı, herkese, her yere.** TotalDrop ücretsiz, açık kaynak ve uçtan uca şifreli bir dosya gönderme uygulamasıdır. E-posta, telefon numarası ya da hesap gerekmez: dosyaları seçin, `7-harbor-rocket` gibi kısa bir kod alın, karşı taraf kodu yazsın. İki ekranda aynı dört kelime görünür; farklıysa durun.

Linux uygulaması KDE Plasma, Hyprland ve Omarchy temalarına canlı olarak uyar. Android sürümü Google Play'de kapalı beta olarak test ediliyor. Proje bağışlarla yaşıyor: [GitHub Sponsors](https://github.com/sponsors/ozdil). Geliştirme düzeni için [docs/gelistirme.md](docs/gelistirme.md).
