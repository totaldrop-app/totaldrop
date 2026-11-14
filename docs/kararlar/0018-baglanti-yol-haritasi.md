# 0018: The connection layer stays on our own stack; direct paths grow piece by piece

- **Status:** Accepted, 10 November 2026 (Ozan, #401)
- **Context:** On 10 November 2026 Ozan asked whether iroh and libp2p suit
  TotalDrop, and which of the open ideas in this field we could combine.
  Today's connection layer works like this
  ([0002](0002-protokol-v0.md), [0003](0003-cloudflare-altyapi.md),
  [protocol notes](../protocol-v0.1-notes.md) 9, 10, 20, 21):

  - The two devices pair in a rendezvous mailbox with CPace and share their
    local addresses, the public addresses they learned through STUN, and
    their relays, in an authenticated `HINTS`.
  - Every transfer opens QUIC (quinn) on both sides with a fresh, single-use
    Ed25519 key. Trust comes from the Noise XXpsk3 and ML-KEM-768 step
    inside QUIC.
  - QUIC talks through a single socket wrapper (`PathSocket`,
    `crates/totaldrop-core/src/path.rs`): direct paths go to the real UDP
    socket, the relay path to the WebSocket relay connection. The paths
    race, and the first one whose handshake completes wins. If the direct
    path stays silent for 8 seconds, the connection moves to the relay. If
    the other side is still sending directly, it moves back; if both sides
    have moved, it stays on the relay until the connection ends. Moving
    from the relay to a direct path that opens later is not done yet
    (protocol note 22, #185).
  - The relay is one Durable Object per transfer on Cloudflare Workers, and
    accepts only those two keys. For those who want to run it on their own
    server, there is also a Rust relay that speaks the same protocol.
    Address discovery uses `stun.cloudflare.com`.

  Reading the code turned up a gap: the QUIC socket binds only to IPv4
  (`transport::bind_socket`, `0.0.0.0`). `HINTS` already carries 16-byte
  addresses, but IPv6 paths are never tried. The stall in #338 was caused
  by the app going into the background, not by the transport layer. #341
  (receiving being slower than sending) is still open. The socket buffers
  were enlarged (7745be5, 614f243; `SOCKET_BUFFER` 4 MiB); their effect on
  #341 has not been measured yet. The issue is about the transport layer,
  but does not depend on iroh or libp2p. On 3 November the relay path was
  measured at about 3 MB/s on Ozan's Wi-Fi connection (#185).

- **Options:**
  1. **iroh** (n0, [1.4.1](https://crates.io/crates/iroh/1.4.1), MIT or
     Apache-2.0; [version 1.0](https://crates.io/crates/iroh/1.0.0)
     15 June 2026). Mature NAT traversal, port mapping on the router, QUIC
     multipath, a ready relay. Where it does not fit:
     - Its relay (`iroh-relay`) is a Rust server and does not run on
       Cloudflare Workers. Either we operate a server, or we connect to
       n0's rate-limited public relays or its paid hosted relay. 0003 chose
       to put the relay on Workers precisely so that there is no new
       machine to update and protect; a server to operate is also a new
       cost that, under [0005](0005-bagis.md), must first be written on the
       donation page.
     - The identity is a long-lived `EndpointId`, published through DNS
       (pkarr), mDNS and the DHT. n0 itself writes that these keys should
       not be considered secret; blinded address lookup is only an
       experiment
       ([Blinded address lookup](https://iroh.computer/blog/blinded-address-lookup),
       13 October 2026). For us the key is new for every transfer and there
       is no discovery. If we turned discovery off and generated a key per
       transfer, most of what iroh brings would be gone.
     - It brings nothing on the cryptography side: CPace, Noise and ML-KEM
       still stay on top.
     - Moving means rewriting `path.rs`, `relay.rs`, `transport.rs`, the two
       relay servers and protocol notes 20–21. The scope of the independent
       audit changes and the dependency tree grows (the Flatpak cargo
       sources, `cargo-deny`). Because the roadmap asks for stability first,
       we do not take this risk before the pre-1.0 audit.
     - `iroh-blobs` is still experimental. Our own piecewise transfer with
       STREAM and BLAKE3 does the same job.
  2. **libp2p** (`rust-libp2p`
     [0.57.1](https://crates.io/crates/libp2p/0.57.1), MIT). Not 1.0, and
     makes breaking changes often. Circuit relay v2 is only for
     coordinating hole punching; the Rust implementation's default circuit
     limits (2 minutes, 128 KiB) are not enough to carry files. DCUtR's
     success rate is about 70%, and behind symmetric NAT about 40% in a
     small sample (536 peers)
     ([Trautwein et al., a large-scale measurement](https://arxiv.org/html/2510.27500v1);
     the symmetric NAT rate from
     [ProbeLab](https://probelab.io/blog/can-libp2p-punch-through-nats/)).
     The Identify protocol announces the device's addresses and version to
     the other side. It is designed for DHT and gossip networks; no part
     of it fits us.
  3. **Keep our own stack and add the open ideas piece by piece.** Today's
     structure (racing paths, address sharing through `HINTS`, falling back
     to the relay and returning) is the skeleton for these pieces; each one
     grows the protocol in a backward-compatible way.

- **Decision:** Option 3. The cryptography, pairing (CPace), rendezvous and
  the Workers relay stay as they are. The only exception is step 6: on the
  same network with no internet, the CPace messages are carried over the
  local network instead of the rendezvous mailbox; the messages and CPace
  stay the same, and only the channel that carries them changes. Each of
  the steps below comes as a separate PR with its own issue, branch, a test
  that shows the bug or the gap, and a device trial. The order follows the
  gain and the cost; the first three go into the "Sırada" (next) milestone,
  the rest into "Sonra" (later). In privacy mode (relay only, threat model
  T6) none of these steps announces addresses or opens ports; the relay
  paths (the Workers relay and the TURN of step 4) are used in privacy mode
  too and see the IP address.

  1. **IPv6.** A dual-stack socket; IPv4 and IPv6 paths race (Happy
     Eyeballs, [RFC 8305](https://www.rfc-editor.org/rfc/rfc8305)). IPv6
     mostly has no NAT (NPTv6 and NAT66 exist, but are rare); when both
     devices have IPv6, and the firewalls in between (the operators' and
     the home routers') let traffic through when both sides send at the
     same time, the direct path opens without any hole punching. How often
     that happens is measured with device trials. The IPv6 address
     announced must be the operating system's temporary address
     ([RFC 8981](https://www.rfc-editor.org/rfc/rfc8981)) rather than the
     permanent one; the permanent address could make the device
     recognisable across transfers. For ordering and trying the
     candidates, `str0m`'s ICE module (MIT or Apache-2.0) is looked at; if
     it does not fit, today's race is kept.
  2. **Port mapping on the router.** UPnP-IGD, NAT-PMP
     ([RFC 6886](https://www.rfc-editor.org/rfc/rfc6886)) and PCP
     ([RFC 6887](https://www.rfc-editor.org/rfc/rfc6887)); a short-lived
     mapping that is released when the transfer ends. n0's standalone
     `portmapper` crate (MIT or Apache-2.0) is evaluated first. The opened
     port leads only to that transfer's QUIC endpoint, and that endpoint
     only accepts the key announced in `HINTS`; it is still written into
     the threat model.
  3. **Listening for network changes.** Instead of waiting for 8 seconds of
     silence, when the operating system says the network changed (Android
     `ConnectivityManager`, Linux NetworkManager), the addresses are learned
     again and the paths are tried again at once. Magic Wormhole's
     "dilation" idea and the `netwatch` crate (MIT or Apache-2.0) are taken
     as examples. It is dealt with together with the "moving to a direct
     path that opens later" gap in #185 and protocol note 22.
  4. **A measurement: TURN as a second relay.** Cloudflare Realtime TURN
     works over UDP and may beat the speed of our WebSocket relay (#185).
     0003 already writes it down as the second path to add "if the relay is
     too slow". The price is the first 1,000 GB a month free, then USD 0.05
     per GB for traffic from Cloudflare to the client
     ([Cloudflare Realtime TURN](https://developers.cloudflare.com/realtime/turn/));
     under 0003 and 0005, the cost beyond the free quota is covered by
     donations. This step is first a measurement: the speed, the expected
     traffic and what the server sees (IP addresses, duration, bytes) are
     written down; putting it into use needs a separate decision and a
     privacy policy update. TURN is a relay too: if it is put into use, it
     can be used in privacy mode as well and, like the Workers relay, sees
     the IP address, the duration and the byte count; this is written into
     threat model T6.
  5. **Birthday punching for symmetric NAT.** The method Tailscale
     describes: one side opens many ports, the other side tries random
     ports. The number of packets and the time are limited; the relay
     always stays as the fallback.
  6. **Serverless local network discovery.** Two devices on the same
     network find each other even without internet (LocalSend's multicast
     idea). The announcement carries no device name or permanent identity,
     and contains nothing that helps try the code offline: if a value
     derived from the code is broadcast, anyone on the same network can try
     the short code offline and break the single-guess protection that
     CPace gives. Pairing is still done with CPace; the announcement only
     says where to meet, and the CPace messages travel over this local
     channel instead of rendezvous (see Decision). Because it is a new
     transport for the pairing messages, this step is in scope for the
     audit. On iOS, raw multicast needs a paid entitlement
     ([0009](0009-ios-macos.md)). The protocol note and the threat model
     are written first.
  7. **Tor in privacy mode.** The Tor path that stands as "not yet" in the
     protocol notes ([#39](https://github.com/totaldrop-app/totaldrop/issues/39))
     is built with `arti` (MIT or Apache-2.0), through an onion service;
     OnionShare is a working example of this. It is slow, and only
     optional.
  8. **Later.** Once quinn supports QUIC's address discovery, NAT traversal
     and multipath drafts, they can replace STUN and the path switching we
     do by hand. For proximity without internet, Wi-Fi Aware and Wi-Fi
     Direct on Android.

  Only the design is taken from these projects, not their code: Tailscale,
  Syncthing, LocalSend, Magic Wormhole, OnionShare, Briar, rquickshare.
  The connection code lives in the protocol libraries (`totaldrop-core`,
  `-proto`, `-crypto`), and these are MIT or Apache-2.0
  ([LICENSING.md](../../LICENSING.md)); no copyleft-licensed dependency
  (e.g. the EUPL-1.2 licensed `magic-wormhole` crate) enters them. Every
  crate to be used is written up in the PR that adds it, with its licence,
  maintenance status and alternatives, and passes `deny.toml`.

- **Why:** Our privacy and cost model (single-use keys, no discovery, a
  serverless relay) is absent from iroh and libp2p from the start; fitting
  them to this model brings more work and risk than they would bring. The
  pieces that would most increase the chance of a direct connection (IPv6,
  port mapping, reacting to network changes) are small, and can be added to
  today's structure.

- **Consequences:**
  - In steps 1–3 the cryptography, pairing and relay protocol do not
    change; the cryptographic scope of the pre-1.0 audit is not affected by
    these steps (there is no Tamarin model of the transfer protocol; the
    existing models are for the account design). Step 6 carries the CPace
    messages over a new path (the local network), and step 7 adds a new
    path (Tor); both are in scope for the audit. As new kinds of path are
    added, the threat model and the protocol notes are updated in the same
    PR.
  - If something the servers see changes (e.g. TURN), the privacy policy is
    updated in 35 languages before the release.
  - There is no telemetry; the effect of each step is measured by hand with
    device trials: whether the direct path or the relay was used, and the
    speed, are tried before and after with the same devices (Jonsbo, POCO,
    the work PC).

- **Review condition:**
  - If device trials show that, even after these steps, a direct connection
    often cannot be made.
  - If the iroh relay becomes able to run in a serverless environment (e.g.
    Workers), or if iroh offers an identity with a per-transfer key and no
    discovery as a supported route.
  - If quinn supports the QUIC NAT traversal or multipath drafts.
