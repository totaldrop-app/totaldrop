# TotalDrop

Send any file to anyone, anywhere. Free, open source, end-to-end encrypted. No email, no phone number, no account required.

Her dosyayı, herkese, her yere gönder. Ücretsiz, açık kaynak, uçtan uca şifreli. E-posta yok, telefon yok, hesap zorunlu değil.

> **Status:** early skeleton. The protocol rules and chunk encryption exist and are tested; sending over the internet arrives in phase 1. Do not use this for real files yet.

## Repository layout

| Path | What it is | License |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: key-derivation contexts and STREAM chunk encryption | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Transfer codes, limits, path safety | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | The `totaldrop` command-line client | GPL-3.0-or-later |
| `app/` | Flutter app for Linux, Windows, macOS, Android, iOS (not created yet) | GPL-3.0-or-later |
| `fuzz/` | Fuzz targets for every parser | MIT OR Apache-2.0 |

Servers (rendezvous, relay, store) will live in `server/` under AGPL-3.0-or-later. See [LICENSING.md](LICENSING.md).

## Build

```bash
cargo test
cargo run -p totaldrop-cli -- code 7-harbor-rocket
```

The project always targets the newest stable Rust (currently 1.99.1, pinned in `rust-toolchain.toml`); rustup installs it automatically on the first build. Dependencies are kept current by Dependabot every week.

## Protocol

The protocol is specified in *TotalDrop Protocol v0*, approved on 27 October 2026. Implementation follows it exactly; changes collect in the v0.1 draft before they reach the code.

## Security

Please report vulnerabilities privately, as described in [SECURITY.md](SECURITY.md). Never open a public issue for a security problem.
