---
name: applied-cryptography-review
description: Use when changing or reviewing TotalDrop's protocol and cryptography - crates/totaldrop-crypto, -proto, -core, fuzz/, formal/ (Tamarin), docs/protocol-v0.1-notes.md, the account design (docs/hesap.md) - or anything that derives keys, runs a handshake or encrypts data.
---

# Applied cryptography review (TotalDrop)

Apply `software-engineering-discipline` and `security-engineering-analyst`
as well. The goal is that the code does exactly what the specification and
`docs/protocol-v0.1-notes.md` say, and that every security claim is backed
by a test vector, a test or a proof.

## What TotalDrop uses (suite `TD0`)

- Transfer code to shared secret: CPace over ristretto255 with SHA-512,
  following draft-irtf-cfrg-cpace byte for byte; `PRS` is the canonical code,
  `sid` the 16-byte mailbox ID, `CI` `totaldrop/0`
  (`crates/totaldrop-crypto/src/cpace.rs`).
- Channel: Noise XXpsk3 inside QUIC/TLS 1.3 with raw Ed25519 public keys,
  then a post-quantum hybrid step with ML-KEM
  (`crates/totaldrop-crypto/src/channel.rs`).
- Key derivation: BLAKE3 `derive_key` with one fixed context string per
  purpose (`Purpose` in `crates/totaldrop-crypto/src/lib.rs`).
- Files: ChaCha20-Poly1305 in the STREAM construction, 1 MiB chunks, a
  per-entry key and nonce prefix.
- Libraries: audited RustCrypto, `curve25519-dalek`, `ed25519-dalek`,
  `ml-kem`, `blake3`, `zeroize`. No primitive is invented or reimplemented.

## Review checklist

1. **Spec match:** every byte layout, label and context string matches the
   specification and the notes; a deviation is a protocol note first.
2. **Known answers:** each primitive and construction is checked against
   upstream or RFC/draft test vectors in the tests.
3. **Domain separation:** a key derived for one purpose can never serve
   another; transcripts bind roles, versions and identities.
4. **Nonces:** never reused under one key; counters cannot wrap; the final
   chunk is marked so truncation and reordering fail.
5. **Authentication before use:** nothing decrypted or parsed is acted on
   before its tag and the peer's key are verified; errors do not reveal
   which check failed.
6. **Randomness and secrets:** OS randomness only; secrets zeroized and kept
   out of logs, `Debug` and error text; comparisons of secrets in constant
   time.
7. **Downgrade:** version and suite negotiation cannot be steered to a weaker
   option; old-client compatibility is written down.
8. **Parsers:** every decoder of peer or server data has a fuzz target in
   `fuzz/fuzz_targets/` and bounded allocation.
9. **Claims:** a claim in the threat model or on the site names the test or
   the Tamarin lemma (`formal/`, `formal/check.sh`, `formal/mutants.py`)
   that backs it.

## Limits

This review, and any automated security tool, does not replace the Tamarin
model or the independent human audit planned before 1.0 (#40). Say so in
the PR when a change needs one of them.
