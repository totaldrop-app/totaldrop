# The relay server

This document describes how two devices on different networks reach each
other. Its counterpart in the protocol is section 6 of the v0 draft
("Relay").

## Purpose

Two devices on the same network already find each other. Over the
internet, however, devices are mostly behind a modem (NAT) or a firewall
and cannot reach each other directly. The relay server does two jobs:

1. **Address discovery:** The device sends a small UDP packet to a STUN
   server (by default Cloudflare's free `stun.cloudflare.com` server); the
   server answers "I see you at this address and port". The device reports
   this public address to the other side in the encrypted `HINTS` message.
   By sending packets to each other's public address at the same time, the
   two devices punch through most modems (NAT hole punching) and connect
   directly.
2. **Relaying:** If no direct path opens at all (e.g. strict corporate
   networks, some kinds of NAT at mobile operators), the encrypted QUIC
   packets go through the server. The connection is a WebSocket over TCP
   443, so it gets through almost every network.

## Security and privacy

- Everything that passes through the server is end-to-end encrypted (QUIC
  + Noise + ML-KEM). The server cannot see file names, contents or identity
  keys.
- The device connects to the server with a fresh, single-use endpoint key
  generated for each transfer, and proves it by signing with that key. The
  server cannot link two transfers to each other.
- The two devices meet at an address that carries the names of the two
  single-use keys (`/v0/relay/<key A>-<key B>`). There the server accepts
  only these two keys and carries packets only between them; nobody else
  can get in between.
- The server keeps no record of addresses, keys or traffic; it writes
  nothing to disk.
- Every transfer is limited: at most 32 GiB and 6 hours.
- The receiver uses the relay server that the sender reported in `HINTS`.
  This address can only be `wss://`.

## Choosing the path

Both devices try these at the same time:

| Path | When |
| --- | --- |
| Local addresses | at once |
| Public address (through address discovery) | at once, for hole punching |
| Relay server | after 1.2 seconds, if no direct path has opened |

The WebSocket connection to the relay server is also opened only once these
1.2 seconds have passed and there is still no direct connection; in privacy
mode, at once. Each device counts this time on its own clock, from the
moment pairing finishes; so one may connect to the relay server while the
other does not. If the direct path wins within this time, no connection to
the relay server is made at all, and the server does not see that
transfer. If the direct path arrives later and wins, the relay connection
that was opened is closed as soon as the path is chosen; in that short
time the server has seen the device's IP address. This behaviour came with
Android and Linux 0.2.0-beta.1
([#63](https://github.com/totaldrop-app/totaldrop/issues/63)); earlier
releases open the connection at the start of every transfer.

The relay path goes through the sender's relay server. The receiving
device uses it only if it is the same as the server in its own settings;
otherwise it tries only the direct paths; in privacy mode, or if the sender
reported no address at all, it stops with an error
([#68](https://github.com/totaldrop-app/totaldrop/issues/68)). From the
next release on, the sender follows the same rule and does not connect to
its own server at all if the receiving device will not use the same server
([#301](https://github.com/totaldrop-app/totaldrop/issues/301)); earlier
releases still connect if the direct path does not open within 1.2 seconds.
This has a side effect: a 0.1 receiver set to another server (or to no
server) could still use the sender's server before; now, if there is no
direct path, the transfer ends with a timeout.

The first connection that opens is used. In privacy mode (`--relay-only`
on the command line, Settings → "Hide my address from the other device" in
the apps) the device reports no IP address at all and uses only the relay
server.

A connection made directly is moved to the relay server if its path later
goes silent ([#185](https://github.com/totaldrop-app/totaldrop/issues/185)):
the Wi-Fi drops, the NAT mapping expires, or the network lets the handshake
through and then carries nothing. While idle, both sides send a QUIC
keep-alive packet every 2 seconds; if a device receives not a single packet
from the connection's direct address for 8 seconds, it opens the relay
connection and sends the packets meant for that address through the relay
server. Packets from the relay server are handed to QUIC as if they came
from that direct address; so from QUIC's point of view the address does not
change, and the two sides move separately, each on its own clock. If only
one direction dies, the side that stops hearing the other first moves; the
other follows one wait period later, when it stops hearing it. Both stay
within QUIC's 30-second idle timeout.

If the other side is still sending on the direct path (it had only paused,
or never moved), the device returns to the direct path with the first
direct packet from it. If both sides have moved, nothing goes to the direct
address any more; the transfer stays on the relay server until it ends. A
device tells that it was itself paused (frozen in the background) from its
clock running at least 2 seconds late; it does not count that silence as
the path's silence, and gives the other side 2 more seconds before moving.
If QUIC moves to a new address of the other side (the NAT mapping changed),
the watched address changes with it.

Moving exists only if both sides use the same relay server: if the
receiver's relay server differs from the sender's, or it has none, neither
side moves, because the other side never comes there. When the direct path
wins, the relay connection is put on standby: if it was opened during
setup, it is closed at that moment, and later reopened only when the
connection moves to the relay server. If the connection returns to the
direct path, the relay connection closes after sending nothing for 30
seconds. If the relay connection has given up (the server refused, could
not be reached, or the limits were reached), the device does not move, or,
if it has moved, returns to the direct path. The relay server sees a
transfer that started directly only when one side moves. After the move,
QUIC's congestion state is the one left from the direct path; there is
packet loss in the first seconds, and QUIC recovers from it by itself.

The relay connection gets no answer to the packets it sends while the
other side has not arrived yet; meanwhile it sends a ping every 2 seconds.
As long as the relay server answers the ping, the connection is not
considered dead, so it can wait for a side that arrives late.

A transfer that starts over the relay, however, does not move to a direct
path later. That move (QUIC connection migration) is the next step.

## Implementation

| Part | Location |
| --- | --- |
| Message format | `crates/totaldrop-proto/src/relay.rs` |
| Client: address discovery, relay connection | `crates/totaldrop-core/src/relay.rs` |
| Client: one QUIC socket, two paths | `crates/totaldrop-core/src/path.rs` |
| Server (Cloudflare) | `server/relay-worker` |
| Server (on your own machine) | `server/totaldrop-relay`, setup files under `deploy/` |

The client gives QUIC its own socket (`PathSocket`). This socket sends
normal packets to the real UDP socket, and those addressed to
`240.84.68.1:1` to the relay server. 240.0.0.0/4 is a reserved block and is
routed nowhere. QUIC sees the relay server as one of the other device's
addresses.

## Server

- **Address:** `wss://relay.totaldrop.app/v0/relay`, on Cloudflare Workers.
  Each transfer connects two WebSockets to each other in its own Durable
  Object. Several QUIC packets are carried in a single WebSocket message;
  this both raises the speed and lowers the number of requests.
- **Measurement:** in a local test setup, 100 MB over the relay path alone
  in 8 seconds (about 12 MB/s).
- **Address discovery:** Cloudflare's STUN server, `stun.cloudflare.com:3478`.
- **For those who want to run it on their own server:**
  `server/totaldrop-relay` speaks the same protocol and also answers STUN
  itself. There are systemd, Caddy and nftables files under `deploy/`; the
  setup script asks for the server's name: `sh install.sh relay.example.org`.
- **Later:** if usage grows or the speed is not enough, Cloudflare Realtime
  TURN can be added as a second relay path (1,000 GB a month free, then
  USD 0.05 per GB).
