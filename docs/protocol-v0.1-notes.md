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
6. **Mailbox lifetime.** Unclaimed nameplates expire after 15 minutes, as v0
   says. In addition, every mailbox is deleted after one hour at the latest.
   Mailbox bodies are capped at 64 KiB, and each side may post 32 messages.
7. **Attack waves.** When 100 or more `scary` closes happen within 10 minutes,
   every allocation is delayed by 2 seconds.

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
14. **Entries.** A directory has size 0 and no hash, and a file must have a
    hash. The total size in the offer must equal the sum of all entries.
15. **Data stream header.** A CBOR map: key 1 is the transfer ID, key 2 the
    entry index, key 3 the first chunk.
16. **Placement.**
    - The receiver never merges into or overwrites anything already on disk.
    - If a top-level item already exists, it is saved as `name (2)`,
      `name (3)` and so on. The same applies to a file that appears at its
      final path during the transfer.
    - Verified files move into place from a temporary file in the same
      folder.
    - No entry may sit beneath a file entry.
17. **Device keys in the command-line client.** Until keystore support
    arrives, the command-line client generates new device keys for every
    transfer, so it never stores an unprotected private key on disk.

## Not in this slice yet

These parts of v0 are not implemented yet:

- the relay, and therefore address discovery and NAT traversal. Direct
  connections work on the same machine or the same network;
- privacy mode (Tor, relay only, padding);
- resuming a transfer (the `accept` bitmaps are always empty);
- links and offline delivery, saved contacts and accounts.
