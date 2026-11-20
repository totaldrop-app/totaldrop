# Architecture

This document gives a high-level overview of TotalDrop's parts and how a
transfer moves through them. The details live in other documents: the
protocol in [protocol-v0.1-notes.md](protocol-v0.1-notes.md), threats and
trust boundaries in [tehdit-modeli.md](tehdit-modeli.md), the reasons
behind decisions in [kararlar/](kararlar/README.md) and the repository
layout in [gelistirme.md](gelistirme.md). A code-level map of the same
structure (layer diagram, flow traces with `file:line` references, data
model, configuration, tests and hot spots) is in
[ARCHITECTURE.md](ARCHITECTURE.md).

## Parts

```
 Sending device                                 Receiving device
┌──────────────────────┐                       ┌──────────────────────┐
│ App (interface)      │                       │ App (interface)      │
│ Linux: Qt 6 + QML    │                       │ Android: Flutter     │
├──────────────────────┤                       ├──────────────────────┤
│ totaldrop-core       │ ── QUIC + Noise ────▶ │ totaldrop-core       │
│ totaldrop-proto      │   (direct or over     │ totaldrop-proto      │
│ totaldrop-crypto     │    the relay)         │ totaldrop-crypto     │
└──────────┬───────────┘                       └───────────┬──────────┘
           │ WebSocket (TLS); UDP for STUN                 │
           ▼                                               ▼
     ┌──────────────────┐   ┌────────────────────┐   ┌────────────────┐
     │ Rendezvous       │   │ Relay              │   │ STUN           │
     │ code and         │   │ forwards encrypted │   │ public address │
     │ mailbox          │   │ traffic            │   │ discovery      │
     └──────────────────┘   └────────────────────┘   └────────────────┘
```

**Core (Rust, `crates/`).** Every platform uses the same code, so the
protocol and cryptography live in one place:

- `totaldrop-crypto`: CPace, key derivation, mailbox and session
  encryption, device keys, STREAM chunk encryption.
- `totaldrop-proto`: transfer codes, limits, path safety and wire messages
  (CBOR; relay frames use their own binary format). Data arriving at the
  client from the peer and from the servers is parsed here; these parsers
  have fuzz targets under `fuzz/`.
- `totaldrop-core`: the client engine. It talks to rendezvous, sets up the
  QUIC connection with raw public keys, runs the Noise XXpsk3 + ML-KEM-768
  handshake, sends and receives files and falls back to the relay server
  when needed.
- `totaldrop-l10n`: interface strings for the Rust programs (35
  languages).

**Apps.** The interface is built with each platform's own toolkit, calls
into the core and does no cryptography:

- Linux (`linux/`): Qt 6 and QML; bound to the core through a CXX-Qt
  bridge ([0006](kararlar/0006-linux-qt.md)).
- Android (`app/`): Flutter; bound to the core through `app/rust` and
  flutter_rust_bridge. Transfers continue in the background as a
  user-initiated job ([0010](kararlar/0010-android-arka-plan-aktarim.md)).
- Command line (`crates/totaldrop-cli`): a thin wrapper around the core.

**Servers (`server/`).** The project has two servers. Neither can see file
contents and neither keeps logs. Each has two implementations: Rust for
self-hosting and tests, and Cloudflare Workers with Durable Objects for
`totaldrop.app` ([0003](kararlar/0003-cloudflare-altyapi.md)). Both
implementations pass the same end-to-end test suite.

- Rendezvous (`totaldrop-rendezvous`, `rendezvous-worker`): allocates the
  numeric part of the code (the nameplate) and holds the mailbox that
  carries encrypted messages until the two devices meet. The device
  generates the rest of the code; the server never sees it.
- Relay (`totaldrop-relay`, `relay-worker`): forwards encrypted QUIC
  traffic between the two devices. It accepts only the two keys for that
  transfer, through a signed challenge.

The email Worker (`mail-worker`), which forwards email sent to project
addresses (such as `hello@totaldrop.app`) to the maintainers, also lives
under `server/`. It is not part of the protocol and the apps never use it
([Deployment](surec.md#e-posta-yönlendirmesi)).

STUN is not a project server: devices learn their public address by
default from Cloudflare's STUN server (`stun.cloudflare.com`, UDP) or from
a self-hosted relay server. Details are in [aktarma.md](aktarma.md).

**Site (`site/`).** totaldrop.app, the download page and the page opened
by QR links are hosted on Cloudflare Pages and load nothing from outside.
Sending is done only from the apps, not from a browser.

## A transfer

1. The sender allocates a number (nameplate) from rendezvous, generates
   the rest of the code on their device and shares the code with the other
   side.
2. Both devices meet in the mailbox and derive a shared key from the code
   with CPace. The server knows neither the code nor the key; messages in
   the mailbox are encrypted with this key.
3. Through the mailbox, the devices exchange their addresses and
   single-use endpoint keys. The direct path and the relay path are tried
   together and the first one to answer is used; because of this, the
   relay server can see both devices' IP addresses even on direct
   transfers ([threat model](tehdit-modeli.md) T5, #63).
4. Inside QUIC, a Noise XXpsk3 + ML-KEM-768 handshake runs; both sides see
   four verification words.
5. The sender sends the file list (manifest) and the receiver accepts it;
   files stream encrypted with STREAM in 1 MiB chunks, each verified as it
   arrives. After BLAKE3 verification, the file is placed in the receive
   folder.

## Building and deployment

Everything is built from this repository on GitHub Actions: the Linux
packages and the Android package from tags, and the servers and the site
from `main`, with the owner's approval ([surec.md](surec.md)). Nothing
built on a personal machine is ever published.
