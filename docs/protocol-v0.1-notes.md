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
4. **Rendezvous wire format.** These are CBOR maps whose key `0` is the type.
   - Client messages: `allocate`; `claim` with key 1 = nameplate; `add` with
     key 1 = phase and key 2 = body; `close` with key 1 = mood.
   - Server messages: `allocated` with key 1 = nameplate; `claimed` with key 1
     = mailbox ID; `message` with key 1 = phase and key 2 = body;
     `peer_closed` with key 1 = mood; `error` with key 1 = reason, after which
     the server closes the connection.
5. **Claiming.**
   - A client can claim only a nameplate that has been allocated, and the
     allocator claims first. Anyone else claiming an empty nameplate is
     guessing, and is refused.
   - A connection holds at most one nameplate.
   - A client that disconnects without `CLOSE` counts as a `lonely` close.
   - Every request is checked against the mailbox ID the connection joined,
     not only the nameplate number. A connection whose nameplate expired is
     told and disconnected, and can never act on the number's next owner.
     Only the allocator or a side of the mailbox can close it.
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

## Section 6: connections and the handshake

8. **Who connects.** The sender listens. The receiver is both the QUIC client
   and the Noise initiator, so it is also the side that encapsulates to the
   other's ML-KEM key.
9. **Raw public keys.**
   - Each side presents its endpoint key as an Ed25519 SubjectPublicKeyInfo.
   - Each side accepts only the exact key announced in the other's
     authenticated `HINTS`. Client authentication is mandatory.
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
    - A receiver gives up after 60 seconds with no data at all.
    - No entry may sit beneath a file entry.
19. **Device keys in the command-line client.** Until keystore support
    arrives, the command-line client generates new device keys for every
    transfer, so it never stores an unprotected private key on disk.

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

- the relay, and therefore address discovery and NAT traversal. Direct
  connections work on the same machine or the same network;
- privacy mode (Tor, relay only, padding);
- resuming a transfer (the `accept` bitmaps are always empty);
- links and offline delivery, saved contacts and accounts.
