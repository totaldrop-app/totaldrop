# 0002: Protocol v0

- **Status:** Accepted, 27 October 2026; external review before 1.0
- **Context:** Security and anonymity come first. The code must be short
  and easy to say out loud; the servers in between must not be able to read
  anything.
- **Decision:**
  - Transfer codes follow the Magic Wormhole model, with CPace (a balanced
    PAKE): an attacker gets a single guess and cannot try the code offline.
  - The connection is QUIC (TLS 1.3 with fresh, single-use raw keys for
    every transfer). Trust comes from the Noise XXpsk3 handshake inside it.
  - Right after the handshake, a hybrid post-quantum step with ML-KEM-768.
  - Files are encrypted piece by piece with the STREAM construction and
    verified with BLAKE3.
- **Why:** Traffic recorded today must stay unreadable even when quantum
  computers arrive; single-use keys stop the servers from linking transfers
  to one another.
- **Consequences:** We have our own CPace layer (there is no current Rust
  crate); this layer and the hybrid step are in scope for the independent
  audit before 1.0. Documents: the protocol draft and
  [protocol-v0.1-notes.md](../protocol-v0.1-notes.md).
