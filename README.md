# TotalDrop

Send any file to anyone, anywhere. Free, open source, end-to-end encrypted. No email, no phone number, no account required.

Her dosyayı, herkese, her yere gönder. Ücretsiz, açık kaynak, uçtan uca şifreli. E-posta yok, telefon yok, hesap zorunlu değil.

> **Status:** phase 1, first working slice. Two devices can transfer files and folders by code, end to end encrypted, when they can reach each other directly (same machine or same network). The relay for transfers across the internet comes next. The code has not been reviewed independently yet: do not rely on it for sensitive files.

## Repository layout

| Path | What it is | License |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Suite TD0: CPace, key derivation, mailbox and session encryption, device keys, STREAM chunk encryption | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Transfer codes, limits, path safety, every wire message (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Client engine: rendezvous, QUIC with raw public keys, Noise XXpsk3 + ML-KEM-768, transfer | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | The `totaldrop` command-line client | GPL-3.0-or-later |
| `app/` | Flutter app for Linux, Windows, macOS, Android, iOS (not created yet) | GPL-3.0-or-later |
| `fuzz/` | Fuzz targets for every parser | MIT OR Apache-2.0 |

| `server/totaldrop-rendezvous` | Rendezvous server in Rust, for self-hosting and tests | AGPL-3.0-or-later |
| `server/rendezvous-worker` | The same rendezvous server on Cloudflare Workers, for `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

The relay and store servers will join `server/`. See [LICENSING.md](LICENSING.md).

## Build

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

The project always targets the newest stable Rust (currently 1.99.1, pinned in `rust-toolchain.toml`); rustup installs it automatically on the first build. Dependencies are kept current by Dependabot every week.

## Protocol

The protocol is specified in *TotalDrop Protocol v0*, approved on 27 October 2026. Implementation follows it; where v0 leaves a choice open, the decision is recorded in [docs/protocol-v0.1-notes.md](docs/protocol-v0.1-notes.md) for the v0.1 draft.

## Security

Please report vulnerabilities privately, as described in [SECURITY.md](SECURITY.md). Never open a public issue for a security problem.
