//! Message encryption for the mailbox (spec section 5) and for the secure
//! session after the post-quantum hybrid step (spec section 6).

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use zeroize::Zeroize;

use crate::cpace::{Isk, Role};
use crate::{Error, Purpose, derive_key};

/// Largest message either cipher accepts: 1 MiB plus 1 KiB of headroom,
/// the same bound as the stream framing (spec section 2).
pub const MAX_MESSAGE: usize = (1 << 20) + 1024;

fn cipher(mut key: [u8; 32]) -> ChaCha20Poly1305 {
    let c = ChaCha20Poly1305::new(&Key::from(key));
    key.zeroize();
    c
}

/// Encrypts mailbox messages under `K_mailbox = derive_key("totaldrop TD0
/// mailbox", ISK)`.
///
/// The 96-bit nonce is a direction byte (`0x00` sender to receiver, `0x01`
/// receiver to sender) followed by an 88-bit big-endian counter, so the two
/// directions can never collide. The phase name is authenticated as
/// associated data, so a message cannot be replayed under another phase.
/// The rendezvous server delivers messages in order, so the receiving counter
/// must match exactly; a dropped, repeated or reordered message fails.
pub struct MailboxCipher {
    aead: ChaCha20Poly1305,
    send_dir: u8,
    recv_dir: u8,
    send_ctr: u64,
    recv_ctr: u64,
}

impl core::fmt::Debug for MailboxCipher {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("MailboxCipher(..)")
    }
}

fn mailbox_nonce(dir: u8, ctr: u64) -> Nonce {
    let mut n = [0u8; 12];
    n[0] = dir;
    n[4..].copy_from_slice(&ctr.to_be_bytes());
    Nonce::from(n)
}

impl MailboxCipher {
    /// Derive the mailbox key from CPace's output. The sender is the
    /// CPace initiator.
    pub fn new(isk: &Isk, role: Role) -> Self {
        let (send_dir, recv_dir) = match role {
            Role::Initiator => (0x00, 0x01),
            Role::Responder => (0x01, 0x00),
        };
        Self {
            aead: cipher(derive_key(Purpose::Mailbox, isk.as_bytes())),
            send_dir,
            recv_dir,
            send_ctr: 0,
            recv_ctr: 0,
        }
    }

    /// Encrypt the next outgoing message of `phase`.
    pub fn seal(&mut self, phase: &str, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
        if plaintext.len() > MAX_MESSAGE {
            return Err(Error::Malformed);
        }
        let nonce = mailbox_nonce(self.send_dir, self.send_ctr);
        self.send_ctr = self.send_ctr.checked_add(1).ok_or(Error::Malformed)?;
        self.aead
            .encrypt(
                &nonce,
                Payload {
                    msg: plaintext,
                    aad: phase.as_bytes(),
                },
            )
            .map_err(|_| Error::Malformed)
    }

    /// Decrypt the next incoming message of `phase`.
    pub fn open(&mut self, phase: &str, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        if ciphertext.len() > MAX_MESSAGE.saturating_add(crate::TAG_SIZE) {
            return Err(Error::Malformed);
        }
        let nonce = mailbox_nonce(self.recv_dir, self.recv_ctr);
        let plain = self
            .aead
            .decrypt(
                &nonce,
                Payload {
                    msg: ciphertext,
                    aad: phase.as_bytes(),
                },
            )
            .map_err(|_| Error::Inauthentic)?;
        self.recv_ctr = self.recv_ctr.checked_add(1).ok_or(Error::Malformed)?;
        Ok(plain)
    }
}

/// The two session keys of spec section 6:
/// `k_i2r = derive_key("totaldrop TD0 session i2r", h || ss)` and
/// `k_r2i = derive_key("totaldrop TD0 session r2i", h || ss)`,
/// where `h` is the Noise handshake hash and `ss` the ML-KEM shared secret.
pub fn session_keys(handshake_hash: &[u8], kem_secret: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let mut material = Vec::with_capacity(handshake_hash.len().saturating_add(32));
    material.extend_from_slice(handshake_hash);
    material.extend_from_slice(kem_secret);
    let keys = (
        derive_key(Purpose::SessionI2R, &material),
        derive_key(Purpose::SessionR2I, &material),
    );
    material.zeroize();
    keys
}

/// The four verification words of spec section 6, as indices into the
/// 2048-word list: `derive_key("totaldrop TD0 sas", h || ss)` read as four
/// big-endian 11-bit numbers.
pub fn verification_indices(handshake_hash: &[u8], kem_secret: &[u8; 32]) -> [u16; 4] {
    let mut material = Vec::with_capacity(handshake_hash.len().saturating_add(32));
    material.extend_from_slice(handshake_hash);
    material.extend_from_slice(kem_secret);
    let sas = derive_key(Purpose::Sas, &material);
    material.zeroize();
    let mut bits = 0u64;
    for byte in &sas[..6] {
        bits = (bits << 8) | u64::from(*byte);
    }
    // 48 bits read; keep the top 44.
    bits >>= 4;
    let mut out = [0u16; 4];
    for (i, slot) in out.iter_mut().enumerate() {
        let shift = 33u32.saturating_sub(11u32.saturating_mul(u32::try_from(i).unwrap_or(0)));
        *slot = u16::try_from((bits >> shift) & 0x7ff).unwrap_or(0);
    }
    out
}

/// Largest message on the secure session: an `offer` may carry a manifest
/// of up to 16 MiB (spec section 11), plus 1 KiB of headroom.
pub const MAX_SESSION_MESSAGE: usize = (16 << 20) + 1024;

fn session_nonce(ctr: u64) -> Nonce {
    let mut n = [0u8; 12];
    n[4..].copy_from_slice(&ctr.to_le_bytes());
    Nonce::from(n)
}

/// The sending half of a secure session, under `k_i2r` or `k_r2i`.
///
/// Nonces follow Noise: four zero bytes and a 64-bit little-endian counter,
/// one counter per direction. The stream is ordered (one QUIC stream), so
/// the receiving counter must match exactly.
pub struct Sealer {
    aead: ChaCha20Poly1305,
    ctr: u64,
}

/// The receiving half of a secure session.
pub struct Opener {
    aead: ChaCha20Poly1305,
    ctr: u64,
}

impl core::fmt::Debug for Sealer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Sealer(..)")
    }
}

impl core::fmt::Debug for Opener {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Opener(..)")
    }
}

impl Sealer {
    /// Encrypt the next outgoing message.
    pub fn seal(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
        if plaintext.len() > MAX_SESSION_MESSAGE {
            return Err(Error::Malformed);
        }
        let nonce = session_nonce(self.ctr);
        self.ctr = self.ctr.checked_add(1).ok_or(Error::Malformed)?;
        self.aead
            .encrypt(&nonce, plaintext)
            .map_err(|_| Error::Malformed)
    }
}

impl Opener {
    /// Decrypt the next incoming message.
    pub fn open(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        if ciphertext.len() > MAX_SESSION_MESSAGE.saturating_add(crate::TAG_SIZE) {
            return Err(Error::Malformed);
        }
        let nonce = session_nonce(self.ctr);
        let plain = self
            .aead
            .decrypt(&nonce, ciphertext)
            .map_err(|_| Error::Inauthentic)?;
        self.ctr = self.ctr.checked_add(1).ok_or(Error::Malformed)?;
        Ok(plain)
    }
}

/// Build both halves of a session from the hybrid session keys.
/// `initiator` is true on the side that started the Noise handshake.
pub fn session(k_i2r: [u8; 32], k_r2i: [u8; 32], initiator: bool) -> (Sealer, Opener) {
    let (send, recv) = if initiator {
        (k_i2r, k_r2i)
    } else {
        (k_r2i, k_i2r)
    };
    (
        Sealer {
            aead: cipher(send),
            ctr: 0,
        },
        Opener {
            aead: cipher(recv),
            ctr: 0,
        },
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::cpace::Cpace;

    fn isk_pair() -> (Isk, Isk) {
        let sid = [3u8; 16];
        let (a, ya) = Cpace::start(Role::Initiator, b"5-1-2", &sid).unwrap();
        let (b, yb) = Cpace::start(Role::Responder, b"5-1-2", &sid).unwrap();
        (a.finish(&yb).unwrap(), b.finish(&ya).unwrap())
    }

    #[test]
    fn mailbox_round_trip_both_ways() {
        let (ia, ib) = isk_pair();
        let mut s = MailboxCipher::new(&ia, Role::Initiator);
        let mut r = MailboxCipher::new(&ib, Role::Responder);
        for i in 0..3u8 {
            let ct = s.seal("hints", &[i; 10]).unwrap();
            assert_eq!(r.open("hints", &ct).unwrap(), vec![i; 10]);
            let ct = r.seal("hints", &[i; 5]).unwrap();
            assert_eq!(s.open("hints", &ct).unwrap(), vec![i; 5]);
        }
    }

    #[test]
    fn mailbox_rejects_wrong_phase_replay_and_reflection() {
        let (ia, ib) = isk_pair();
        let mut s = MailboxCipher::new(&ia, Role::Initiator);
        let mut r = MailboxCipher::new(&ib, Role::Responder);
        let ct = s.seal("confirm", b"x").unwrap();
        assert!(r.open("hints", &ct).is_err());
        let mut r2 = MailboxCipher::new(&ib, Role::Responder);
        assert!(r2.open("confirm", &ct).is_ok());
        assert!(r2.open("confirm", &ct).is_err(), "replay must fail");
        // A message reflected back to its own author must fail.
        let mut s2 = MailboxCipher::new(&ia, Role::Initiator);
        assert!(s2.open("confirm", &ct).is_err());
        drop(r);
    }

    #[test]
    fn session_round_trip_and_tamper() {
        let (i2r, r2i) = session_keys(&[1u8; 64], &[2u8; 32]);
        assert_ne!(i2r, r2i);
        let (mut a_out, mut a_in) = session(i2r, r2i, true);
        let (mut b_out, mut b_in) = session(i2r, r2i, false);
        let ct = a_out.seal(b"hello").unwrap();
        assert_eq!(b_in.open(&ct).unwrap(), b"hello");
        let mut bad = a_out.seal(b"again").unwrap();
        bad[0] ^= 1;
        assert!(b_in.open(&bad).is_err());
        let ct = b_out.seal(b"back").unwrap();
        assert_eq!(a_in.open(&ct).unwrap(), b"back");
        // A message cannot be reflected back to its author.
        let ct = a_out.seal(b"mirror").unwrap();
        assert!(a_in.open(&ct).is_err());
    }

    #[test]
    fn verification_indices_are_11_bit_and_deterministic() {
        let a = verification_indices(&[5u8; 64], &[6u8; 32]);
        let b = verification_indices(&[5u8; 64], &[6u8; 32]);
        assert_eq!(a, b);
        assert!(a.iter().all(|i| *i < 2048));
        assert_ne!(a, verification_indices(&[5u8; 64], &[7u8; 32]));
    }

    #[test]
    fn verification_indices_bit_layout() {
        // Recompute from the raw bytes independently.
        let mut m = vec![5u8; 64];
        m.extend_from_slice(&[6u8; 32]);
        let s = derive_key(Purpose::Sas, &m);
        let bits: u64 = s[..6].iter().fold(0, |acc, b| (acc << 8) | u64::from(*b));
        let expect = [
            (bits >> 37) & 0x7ff,
            (bits >> 26) & 0x7ff,
            (bits >> 15) & 0x7ff,
            (bits >> 4) & 0x7ff,
        ];
        let got = verification_indices(&[5u8; 64], &[6u8; 32]);
        for i in 0..4 {
            assert_eq!(u64::from(got[i]), expect[i]);
        }
    }
}
