# Protocol v0.1 draft: notes from the first implementation

The v0 specification was approved on 27 October 2026. Writing the first
working transfer showed where v0 leaves a choice open. This file records how
the code decides each one. Every item is a clarification, not a change of
security design. Each one goes into the v0.1 draft for review before it is
treated as final.

## Section 5: transfer codes and rendezvous

1. **CPace roles and inputs.** The sender is always party A, the initiator,
   and the receiver is always party B. The code uses the initiator/responder
   transcript (`transcript_ir`). Associated data is empty on both sides. Each
   `pake` message body is the raw 32-byte encoded ristretto255 element.
2. **CPace implementation.** The only Rust CPace package, `cpace` 0.1.0, dates
   from 2020 and depends on `curve25519-dalek` 2. TotalDrop therefore carries
   its own CPace layer in `totaldrop-crypto/src/cpace.rs`:
   - group arithmetic comes from `curve25519-dalek` 5.0;
   - hashing is SHA-512 from RustCrypto;
   - the code is checked against the test vectors of `draft-irtf-cfrg-cpace`;
   - this layer is in scope for the external review required before 1.0.
3. **Mailbox encryption.**
   - Nonce: a direction byte (`0x00` sender to receiver, `0x01` receiver to
     sender), then an 88-bit big-endian counter.
   - The phase name (`confirm`, `hints`) is authenticated as associated data,
     so a message cannot be replayed under another phase.
   - Counters must match exactly, because the server delivers in order.
4. **Two endpoints instead of one connection.** The server can run as many
   small objects (Cloudflare Durable Objects), so allocation and mailboxes
   are separate WebSocket addresses:
   - `/v0/allocate`: the client sends `allocate` and gets `allocated` with
     key 1 = nameplate and key 2 = a random 16-byte allocator token. Then
     the connection ends.
   - `/v0/mailbox/<nameplate>`: `claim` (key 1 = nameplate, key 2 =
     allocator token, optional), then `add` (key 1 = phase, key 2 = body)
     and `close` (key 1 = mood).
   - Server messages on a mailbox: `claimed` (key 1 = mailbox ID),
     `message` (key 1 = phase, key 2 = body), `peer_closed` (key 1 = mood),
     and `error` (key 1 = reason), after which the server closes the
     connection.
   - Clients ping every 20 seconds while they wait, so idle connections are
     not dropped. A connection must allocate or claim within 10 seconds of
     opening; pings do not extend that.
5. **Claiming.**
   - The first claim of a nameplate must carry its allocator token. Only
     the sender has it, so anyone else claiming an empty nameplate is
     refused. A nameplate that was never allocated cannot be claimed.
     Both servers compare the token without an early exit (#428).
   - A connection claims once, and only the mailbox of its address.
   - A client that disconnects without `CLOSE` counts as a `lonely` close.
   - After `CLOSE` the server ends the connection. The client waits for
     that, at most 2 seconds in all, before it hangs up itself. Against the
     Cloudflare server, a client that hung up at once sometimes lost both
     its last `add` and its `close`: the other side got a `lonely`
     `peer_closed` without the hints and gave up. That happened in 6 of
     650 exchanges, and in none of 650 with the wait (#191). The exact
     cause on the server side is not confirmed.
   - Every mailbox has its own random ID, so a connection whose nameplate
     expired can never act on the number's next owner. Only a side of the
     mailbox can close it.
   - A freed nameplate rests for 10 minutes before reuse, while other
     numbers are free.
6. **Mailbox lifetime.** Unclaimed nameplates expire after 15 minutes, as v0
   says. In addition, every mailbox is deleted after one hour at the latest.
   - Mailbox bodies are capped at 16 KiB, and each side may post 32 messages.
   - While the other side has not arrived, at most 8 messages and 32 KiB are
     buffered per mailbox.
7. **Scary closes and attack waves.**
   - A `scary` close retires a nameplate only when it comes from one of two
     sides that have both posted a message, so nobody can retire numbers at
     will.
   - A peer that leaves after the `pake` messages but before key
     confirmation is treated as a failed confirmation: the other side
     closes `scary`. Such a peer may have taken its one guess offline.
   - When 100 or more counted `scary` closes happen within 10 minutes, every
     allocation is delayed by 2 seconds.
   - The WebSocket handshake and every server write must finish within 10
     seconds.
   - Each client may open 60 connections (allocations and mailboxes
     together) at once and one more every second after that (threat model
     T9). A client is an IPv4 address or an IPv6 /64. Past its rate the
     server answers `error` with the reason `too many requests` and closes
     the connection; the refused connection is not counted. The Rust
     server then reads what the client sent for up to 2 seconds, until the
     client closes too, so that the client's unread first message does not
     turn the close into a reset, in which Windows drops the refusal
     (#442). The counters are kept in memory only, under a hash keyed at
     random when the server starts, and a client is forgotten once it is
     back at a full burst. The Rust server counts every routed connection
     when it opens, the Worker every allocation and every mailbox opening;
     a client making ordinary transfers sees no difference.
8. **Reading a typed code.** Before a code is split into its number and
   words, invisible format characters (direction marks, isolates, zero-width
   characters, the soft hyphen, the byte order mark; `INVISIBLE_FORMAT` in
   totaldrop-proto) are dropped, and Arabic-Indic (U+0660 to U+0669) and
   Extended Arabic-Indic (U+06F0 to U+06F9) digits are read as ASCII digits.
   CPace always gets the canonical ASCII form (`7-840-1499`), so another
   implementation that skips these steps stays compatible; it only rejects
   more of what people type.

## Section 6: connections and the handshake

8. **Who connects.** Both sides listen and dial on the same UDP socket, so
   a firewall in front of either device still leaves one way through.
   - The receiver dials the sender's announced addresses at once and also
     accepts the sender dialling it. It keeps the first connection that
     completes and opens the control stream on it.
   - The sender accepts and, after 400 ms, also dials the receiver's
     addresses. The connection on which the receiver opens the control stream
     is the one; the sender closes the others.
   - The receiver is always the Noise initiator, whichever side opened the
     QUIC connection, so it is also the side that encapsulates to the other's
     ML-KEM key.
   - Older clients (0.1.0-beta.2 and earlier) only dial (receiver) or only
     listen (sender); they keep working with newer ones in that direction.
9. **Raw public keys.**
   - Each side presents its endpoint key as an Ed25519 SubjectPublicKeyInfo.
   - Each side accepts only the exact key announced in the other's
     authenticated `HINTS`, in both QUIC roles. Client authentication is
     mandatory.
   - Session resumption and 0-RTT are disabled.
   - The TLS server name is always `totaldrop`.
10. **`HINTS` encoding.**

    | Key | Field |
    | --- | --- |
    | 1 | Versions |
    | 2 | Endpoint key |
    | 3 | Public addresses |
    | 4 | Local addresses |
    | 5 | Relay URL (optional) |

    Each address is `[ip bytes (4 or 16), port]`. At most 8 addresses are
    accepted per list.
    - Since #403 the QUIC socket is dual-stack where the system has IPv6:
      one port for both families. Local addresses are loopback, the IPv4
      address of the default-route interface and the IPv6 address the
      system would send from; public addresses are what STUN answers over
      IPv4 and, on a dual-stack socket, over IPv6. An IPv6 address is
      announced only if another device can reach it (not loopback,
      unspecified, link-local or IPv4-mapped) and its interface part is
      not built from the network card's hardware address (modified
      EUI-64), which would stay the same on every network. The address
      is the system's own choice of source; where Linux has no temporary
      addresses (RFC 8981) it is a stable-privacy one (RFC 7217), the same
      on one network (#405). A device dials the other's addresses once
      each, in canonical form, and leaves out unspecified and IPv6
      link-local ones and IPv6 targets it would reach only from an address
      it does not announce. A side whose socket is IPv4 only fails just
      the dial to an IPv6 address; the other paths race on. IPv6 needs no
      NAT punching, but home routers usually keep a stateful firewall that
      drops a dial nobody asked for; it opens because both sides dial each
      other (note 8). On a dual-stack socket QUIC sees IPv4 addresses, the
      relay stand-in among them, as IPv4-mapped IPv6 ones.
    - Since #406 a device also asks its router for a port: the gateway of
      the default IPv4 route with the lowest metric (read from
      `/proc/net/route` on Linux; other systems ask no one yet) gets a PCP
      MAP request (RFC 6887) for UDP on the QUIC socket's port, with no
      suggested external port or address (`::ffff:0.0.0.0`), and a NAT-PMP
      one (RFC 6886) if it answers "unsupported version" or has not
      answered the first PCP request within 250 ms (PCP keeps its three
      tries meanwhile; the first mapping either gives is used). Once one
      has won, the other, if its mapping request is already out, keeps
      listening for its answer within its own tries, for at most 1.6
      seconds, and releases a mapping it was given on another external
      port at once. One on the winner's port is the same mapping in a
      router that keeps one table for both; it is released under both
      protocols when the transfer ends, since such a router may now hold
      it under the loser's name. If the winner's mapping cannot be used,
      the loser's goes too. An answer that comes after the loser's tries
      is not seen; that mapping closes when its lifetime runs out (#425).
      A process that exits soon after the transfer waits for the loser
      too, at most 1.6 seconds, so the second mapping is still released
      (#461).
      The external address it grants goes into the public list, unless it is
      not reachable from the internet (private, 100.64.0.0/10 shared
      address space, loopback, link-local, reserved), which means a second
      NAT behind the router; such a mapping is released at once. The
      request asks for 120 seconds; the router may grant more or less. It
      is renewed at half of what was granted (at most every 30 minutes),
      tried again every 5 seconds after an unanswered renewal until the
      grant runs out, and released with lifetime 0 when the transfer ends
      (sent twice, 250 ms apart; a process that exits waits up to 1.6
      seconds for the second copy, #502); a crash leaves the port open until the granted lifetime
      runs out.
      Asking starts with STUN; once STUN has answered, `HINTS` wait at most
      300 ms more for the router (longer, until the attempt ends, once the
      gateway has answered over UPnP; see below), and the whole attempt
      ends after 1.6 seconds (a mapping that comes after `HINTS` were sent
      is released).
      NAT-PMP alone may need longer than that: it starts after 250 ms and
      has up to 750 ms for the address and 750 ms for the mapping, so the
      1.6-second end can cut off a mapping request the router already
      got; that mapping is never seen and stays open until its lifetime
      runs out (#425).
      A PCP answer must carry the request's random 12-byte nonce; NAT-PMP
      has none, so only the source address (the gateway, on a connected
      socket) and the internal port are checked. The router's epoch is not
      checked yet. Privacy mode asks nothing. The mapped port leads only to
      the transfer's QUIC endpoint, which accepts only the endpoint key in
      `HINTS`.
    - The same gateway is also searched over UPnP IGD (`igd-next`) from
      the start: an SSDP search sent to the gateway's own address on port
      1900 (not to the multicast group), then its device description and
      the `WANIPConnection`/`WANPPPConnection` control URL and service
      description. These requests only read; they go to the gateway on
      every transfer outside privacy mode, also where PCP answers. SOAP
      actions follow only if neither PCP nor NAT-PMP has mapped a port 250
      ms after the first request: `GetExternalIPAddress`, then the
      mapping. A UPnP mapping takes about five requests; at home
      (miniupnpd) the search alone took about 85 ms, so a mapping that
      started only at 250 ms came a few milliseconds after `HINTS` and was
      released at once. Once the gateway has answered the search, `HINTS`
      therefore wait for UPnP until it gives a mapping or gives up, at
      most until the attempt ends. The description and control URLs must be on
      the gateway's address; an answer describing a device anywhere else
      is not used (any LAN host can send an answer, but it can only point
      at the gateway). The external address must pass the same "reachable
      from the internet" check. `AddAnyPortMapping` (or
      `AddPortMapping` with a random external port) asks for UDP to the
      QUIC socket's port for 120 seconds, described as `TotalDrop`; it is
      renewed with `AddPortMapping` on the same external port every 60
      seconds, tried again every 5 seconds until the lease would run out,
      and removed with `DeletePortMapping` when the transfer ends. A
      router that answers it offers only permanent mappings (UPnP error
      725) gets none; one that silently ignores the lease keeps the port
      until it is deleted or the router restarts, so a crash can leave it
      open. The first mapping wins; a PCP refusal ends the attempt, and a
      UPnP mapping that completes after a PCP or NAT-PMP one is deleted
      at once. A PCP or NAT-PMP request still in flight when UPnP wins is
      not waited for; the router keeps what it made until its lease (120
      seconds asked) runs out. The search and the HTTP requests end with
      the 1.6-second attempt; a mapping request cut off there may leave a
      mapping until its 120-second lease runs out. An HTTP response is
      read up to 1 MiB.
11. **Session after the hybrid step.**
    - Each direction has its own ChaCha20-Poly1305 key (`k_i2r`, `k_r2i`).
    - Nonces follow Noise: four zero bytes, then a 64-bit little-endian
      counter.
    - Key confirmation is the control message `{0: "confirm"}`, which each
      side sends first under the new keys.
12. **Verification words.** These are the first 44 bits of
    `derive_key("totaldrop TD0 sas", h || ss)`, as four big-endian 11-bit
    indices into the BIP-39 English list.

## Section 8: transfer protocol

13. **Message size.** Section 2 caps every frame at 1 MiB plus 1 KiB, but an
    `offer` may carry a manifest of up to 16 MiB (section 11). Control
    messages are therefore allowed up to 16 MiB plus 1 KiB. Data chunks keep
    the 1 MiB plus 1 KiB limit.
14. **Path characters.** Besides v0's rules, segments must not contain `:`,
    `<`, `>`, `"`, `|`, `?` or `*`. On Windows, `C:` would name another
    drive and `name:stream` an alternate data stream. The receiver also
    checks that every final path is the destination followed only by plain
    names.
15. **CBOR maps.** A map may have at most 64 keys. Duplicate keys are found
    in linear time.
16. **Entries.** A directory has size 0 and no hash, and a file must have a
    hash. The total size in the offer must equal the sum of all entries.
17. **Data stream header.** A CBOR map: key 1 is the transfer ID, key 2 the
    entry index, key 3 the first chunk.
18. **Placement.**
    - The receiver never merges into or overwrites anything already on disk.
    - If a top-level item already exists, it is saved as `name (2)`,
      `name (3)` and so on. The same applies to a file that appears at its
      final path during the transfer.
    - Verified files move into place from a temporary file in the same
      folder, through a hard link that fails if the name exists. On a
      filesystem without hard links, a checked rename is used instead.
    - A receiver gives up after 60 seconds with no data at all. A sender
      waits at most 10 minutes for the receiver to accept or decline.
    - A side that stops sends `cancel` (reason `stopped`) on the control
      stream, best effort within 2 seconds (a sender first waits up to 2
      seconds more for its data streams to reset), then closes the
      connection. The
      other side reports that the other device stopped. A stopped receiver
      removes its partial files and the empty folders this transfer created;
      files already placed stay. A receiver whose sender cancels does the
      same; files it verified before it handled the cancel stay too.
    - A sender that stops first resets every data stream still open with
      error code 1 (`RESET_STREAM`), then sends `cancel`. Data not yet sent,
      or not yet read by the receiver when the reset arrives, is dropped. A
      dropped stream would end cleanly instead and let the file complete.
      Data already on the wire ahead of the reset can still complete a
      file; it then counts as verified before the cancel. The receiver
      treats a data stream reset with code 1 like the cancel (#193). A
      stream the sender could not finish, for example because the file
      could not be read, is reset with code 2; the receiver stops and
      reports that the sender could not send a file.
    - No entry may sit beneath a file entry.
19. **Device keys in the command-line client.** Until keystore support
    arrives, the command-line client generates new device keys for every
    transfer, so it never stores an unprotected private key on disk.

## Section 6: the relay

20. **Relay wire format** (`totaldrop-proto/src/relay.rs`).
    - Both devices connect to `<relay URL>/<key A>-<key B>`: the two
      single-use endpoint keys in lowercase hex, the lower first. A relay may
      admit only those two keys there, and the Cloudflare relay does.
    - The relay sends `TDR0 || challenge` (32 random bytes); the device
      answers `TDR0 || endpoint key || Ed25519 signature over "totaldrop
      relay v0 auth" || challenge`; the relay answers `TDR0 || "ok"`.
    - Then every binary message is `key || (u16 length || datagram)+`: the
      destination key when the device sends, the source key when the relay
      delivers. Several datagrams share a message, at most 64 KiB. One
      connection per key; datagrams for a key that is not there are dropped.
    - A new sign-in with a key that already has a connection on the same
      pair replaces it: the old socket is closed and the other side stays.
      A socket can die without the relay noticing (a Wi-Fi switch, NAT
      rebinding), and only the key's holder can sign in. When a signed-in
      connection ends any other way (the device leaves, or the relay closes
      it for a limit or a bad message), the relay closes the other
      signed-in side too; both clients then connect again. Sockets still
      signing in are left alone (#190).
    - After `ok`, a relay may put a probe, `TDR0 || "probe" || 16 random
      bytes`, between the messages it delivers; the device sends it straight
      back, unchanged. The device reads the probe only after everything sent
      before it, so the answer tells the relay how much it still holds for
      that device. The Cloudflare relay cannot see its send buffer and
      bounds it this way: a probe after the first bytes it gives a device
      and then every 4 MiB, a count that survives hibernation (never low;
      4 MiB high for each wake until the next answered probe; #426), and
      datagrams for a connection with more than 32 MiB after its last
      answered probe, or for a key with more than 64 MiB over all its
      connections (a replaced one counts until it is gone), are dropped
      (#190). A relay that sends probes must:
      recognise an answer (exactly those 25 bytes) before its datagram
      check, and neither forward it nor count it against the byte limit;
      accept only a nonce it has outstanding; bound by bytes given since the
      last answered probe, not by how long an answer takes (a device with a
      congested upload answers late); and accept answers in the same
      version that starts sending probes. Devices from before 0.3.0-beta.4
      never answer, so a connection of theirs through the Cloudflare relay
      gets at most 32 MiB: a receiver loses it after 32 MiB of a file, a
      sender after 32 MiB of acknowledgements (an estimated 1 GiB of
      file), and the transfer then ends at QUIC's idle timeout.
      The self-hosted relay bounds its queue itself, sends no probes and
      closes a pair on an unexpected 25-byte message.
    - Address discovery is plain STUN (RFC 8489), Binding with
      XOR-MAPPED-ADDRESS, sent from the QUIC socket, over IPv4 and, from a
      dual-stack socket, over IPv6 at the same time (#403). IPv6 is asked
      only where a dial would go out from an address the device announces,
      so its hardware (EUI-64) address never reaches the STUN server. A
      round waits 700 ms; once one family has answered, the other gets
      150 ms more, and a second round is sent only when nothing answered.
      The default server is `stun.cloudflare.com:3478`; the self-hosted
      relay answers STUN too.
21. **Which relay.** Each side announces its relay in `HINTS` field 5.
    The relay path runs over the sender's relay. A receiver uses it only if
    it is `wss://` (or
    `ws://` on the same machine) and the same as the receiver's own relay
    setting (the case of scheme and host, a default port and a trailing `/`
    aside); otherwise the receiver tries direct paths only. When that
    leaves no path, or in privacy mode, it stops with an error at once; a
    sender that can see the mismatch in the receiver's `HINTS` does too. A
    receiver with no relay set uses none. The sender, by the same rule,
    links to its relay only when the receiver will use it too; otherwise
    it tries direct paths only and never contacts the relay (#301).
22. **Paths.** Direct addresses are tried at once; the relay path 1.2
    seconds later, or at once when no address was announced. The first
    connection whose handshake completes is used. A path that has not
    answered gets a fresh handshake attempt every 4 seconds, each allowed 10
    seconds, until the side gives up (20 seconds for the receiver): QUIC
    waits longer and longer between resending an unanswered handshake
    packet, so a path that starts working late would otherwise go unnoticed
    (#185). Moving a connection from the relay to a direct path that opens
    later (QUIC migration) is not done yet.
    - The other way round is done: a direct connection whose path goes quiet
      moves onto the relay (#185). Each side sends a QUIC keep-alive after 2
      seconds with nothing to send. A side that has received no datagram at
      all from the connection's direct address for 8 seconds opens its relay
      link and from then on sends what QUIC addresses to that direct address
      through the relay. Once a side watches its connection, datagrams from
      the relay are always handed to QUIC as if they came from that direct
      address, moved or not. QUIC therefore sees no address change and
      neither side migrates; each side moves on its own, and when only one
      direction dies, the side that stops hearing moves first and the other
      follows one wait later, both inside the 30-second idle timeout.
    - A side that moved goes back to sending directly as soon as a datagram
      arrives from the direct address again, which happens only if the other
      side still sends directly (it was paused, or never moved). Once both
      sides moved, nothing goes to the direct address, and the connection
      stays on the relay until it ends. A side whose own clock ran late by 2
      seconds or more (it was paused itself) gives the other side 2 more
      seconds before it moves. The watched address follows QUIC's remote
      address, so a peer whose NAT mapping changed is not taken for a silent
      one.
    - On going back, a side sends the last 4 datagrams it had sent through
      the relay once more, now to the direct address, with the ECN mark
      QUIC gave them, and only if that is still the connection's address
      (#435). They are QUIC packets as QUIC built them, so a copy the other
      side already has is dropped there as a duplicate (RFC 9000, section
      12.3); no new kind of message goes on the wire, only up to 4 extra
      copies. Without this, what went to the relay was lost when the other
      side was not on it, and QUIC, which doubled its wait before each
      resend while the path was gone, could leave the transfer standing
      still for 10 seconds or more. When one of them asks for an
      acknowledgement, which the last packets of a side that was sending
      usually do, its acknowledgement resets that wait and shows
      everything older as lost, which QUIC resends at once.
    - A side moves only when both use the same relay (item 21): the sender
      checks the receiver's `HINTS` field 5, and the receiver has a relay
      path only in that case. When a direct path wins, the relay link is
      parked: a connection to the relay it already has is closed at once,
      and it connects again only while the connection has moved, starting
      with the datagram that needs it; datagrams queued for the relay
      meanwhile are dropped. Back on the direct path, the link closes its
      connection after 30 seconds without sending. A side whose relay link
      has given up (refused, unreachable, or out of the limits below) does
      not move, and goes back to the direct path if it had moved. A direct
      datagram that arrives within 0.5 seconds of the move does not bring
      the side back: it may have been on its way already.
    - A relay link that sent datagrams and got nothing back pings the relay
      every 2 seconds; a pong counts as an answer. The relay drops datagrams
      while the other device is not there, so without this a link waiting
      for a side that moves later would be taken for a dead one.
    - A stream may have 8 MiB in flight and a connection 16 MiB, so a path
      with a long round trip is not held to window / RTT (#185). An honest
      client never has more than that in flight on one connection, which
      also bounds what a relay holds for it. The Cloudflare relay also
      enforces a bound of its own with probes (item 20, #190).
    - A relay link that loses its connection, or does not get one, connects
      again after 0.5 seconds, doubling up to 4 seconds, and gives up after
      10 tries in a row that did not sign in. It keeps the limits below
      over all its own connections together; the relays count them per
      connection (#190). When datagrams went out and nothing at all (no
      datagram, no pong to those pings) came back for 10 seconds, the link
      drops the connection and connects again: a dead TCP connection is
      otherwise reported only after minutes, long past QUIC's idle timeout.
      No write to the relay waits longer than those 10 seconds either,
      the keep-alive ping and the closing of the connection included: a
      relay that stopped reading would otherwise hold the link in a write
      with a full send buffer, where it cannot notice anything (#424).
23. **Relay limits.** Per connection: 32 GiB and 6 hours; ten seconds to
    sign in. Both accept only the pair path `/v0/relay/<key A>-<key B>` and
    forward only to the other key of that pair. The self-hosted relay also
    caps each connection at 8 MiB/s and closes it after 60 seconds without
    datagrams (pings do not count), which ends the other side too. The
    Cloudflare relay drops datagrams for a connection with 32 MiB
    unanswered (item 20). Neither keeps anything on disk or logs client
    data.

## Section 8, continued: progress

24. **The receiver reports progress** (#211). Before, the sender counted a
    chunk as sent once QUIC had taken it. QUIC takes data up to the
    receiver's connection window (16 MiB) beyond what the receiver has
    read, and the receiver writes up to four chunks after reading them, so
    the sender's bar could run about 20 MiB ahead of what had arrived, and
    a small file showed as done at once.
    - A sender that understands reports sets `offer` field 7 (`true`).
      The field is outside the signed fields 1–4; like everything on the
      control stream, the session keys of section 6 (the handshake)
      protect it.
    - A receiver that sees field 7 sets `accept` field 3 (`true`) and then
      sends `{0: "received", 1: transfer ID, 2: bytes}`: the bytes it has
      decrypted and written to partial files so far, over all files,
      including a file whose hash later fails. It sends one at most every
      250 ms, and only when the number changed.
    - The sender shows only what `received` reports. A report above the
      bytes the receiver asked for, not larger than the one before, or for
      another transfer is a protocol error. When every `entry_done` has
      come, the sender shows everything as sent.
    - Older versions: an old receiver ignores field 7 and never sends
      `received`, which it would not have understood either way. An old
      sender ignores `accept` field 3, and a new receiver never sends
      `received` to it, since an old sender would stop on the unknown
      message. With an old receiver, the sender shows a lower bound:
      the files the receiver confirmed with `entry_done`, or the bytes it
      has handed to QUIC less 32 MiB (the bound above, rounded up),
      whichever is larger, checked every 250 ms. A file smaller than that
      shows no progress until it has arrived whole.
    - Absent fields 7 and 3 mean `false`. A report reveals nothing the
      other side does not see anyway: the data arriving.

## Section 8, continued: texts

25. **A text is a file marked to be shown** (#313). A person can send a
    short text, such as a link, instead of files.
    - The sender offers it as one file entry (UTF-8, at most one chunk,
      1 MiB), with the same content key, chunk encryption and hash as any
      file, and sets `offer` field 8 (`true`).
    - Like field 7, field 8 is outside the signed fields 1–4 and is
      protected by the session keys. It changes only how the receiver
      shows what the signed manifest holds, so the signed bytes of a text
      offer are those of the same file offer, and an old receiver's
      signature check is unchanged.
    - A receiver takes an offer as a text only when field 8 is set and
      the manifest holds exactly one entry: a file at the top level (a
      path of one name) of 1 byte to 1 MiB. Anything else marked as a text
      is received as files.
    - A receiver that shows texts receives it in memory and writes nothing
      to disk. It accepts entry 0, reads its one data stream, checks the
      hash and that the content is UTF-8, and reports the entry with
      `entry_done`. A hash that does not match, or content that is not
      UTF-8, is reported as failed (`ok = false`) and nothing is shown or
      saved; the sender says that one file failed.
    - A receiver may also save a text like any file (the core's default
      `accept` does): then it is placed in the destination under its name.
    - The command line checks the destination folder before it connects,
      when it does not yet know that a text will come; a text then goes to
      standard output and the folder is not used.
    - Older versions: an old receiver ignores field 8 and saves the text
      as the file it also is; every sender names it `text.txt`
      (`totaldrop_core::TEXT_FILE`). An old sender never sets it. Absent
      means `false`.
    - The receiver shows a text as data: it never opens a link on its own.
      The command line names control, format and separator characters
      (`U+202E`, `U+001B`, a `\r` inside a line) instead of letting a
      terminal act on them; the joiners `U+200C`/`U+200D` stay, since
      scripts need them. The apps show a text the same way, as plain
      text, and copy it as it came (#326). Look-alike letters are not
      caught.

## Known limits, to be closed before 1.0

- The rendezvous server has no per-client rate limiting yet. Adding it
  without logging addresses needs a design: short-lived in-memory counters.
- One control message of up to 16 MiB can cost about 1 GiB of memory while
  it is decoded. Only a peer that holds the code can send it. A streaming
  manifest format for v0.1 would remove this.
- Secrets inside library types (snow's handshake state, ML-KEM's shared key
  type) are wiped only as far as those libraries do it.

## Not in this slice yet

These parts of v0 are not implemented yet:

- moving a relayed connection to a direct path that opens later;
- the parts of privacy mode beyond relay only: Tor and size padding;
- resuming a transfer (the `accept` bitmaps are always empty);
- links and offline delivery, saved contacts and accounts.
