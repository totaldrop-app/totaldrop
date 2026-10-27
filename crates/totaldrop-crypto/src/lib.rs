//! Cryptographic building blocks of the TotalDrop protocol, suite `TD0`.
//!
//! This crate implements only what the protocol specification fixes exactly:
//! the key-derivation contexts (spec section 3) and the content encryption of
//! file chunks in the STREAM construction (spec section 8). It never invents
//! primitives; it composes audited RustCrypto and BLAKE3 implementations.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Size of one plaintext chunk, fixed at 1 MiB in protocol version 0.
pub const CHUNK_SIZE: usize = 1 << 20;

/// Size of the Poly1305 authentication tag appended to every chunk.
pub const TAG_SIZE: usize = 16;

/// Purposes of [`derive_key`]. Each maps to one fixed context string, so a key
/// derived for one purpose can never be reused for another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    /// Per-entry content key `k_i` (spec section 8).
    ContentKey,
    /// Per-entry 7-byte nonce prefix (spec section 8).
    NoncePrefix,
    /// Key for encrypted mailbox messages (spec section 5).
    Mailbox,
    /// Pre-shared key for the Noise XXpsk3 handshake (spec section 5).
    NoisePsk,
    /// Session key, initiator to responder (spec section 6).
    SessionI2R,
    /// Session key, responder to initiator (spec section 6).
    SessionR2I,
    /// Verification words shown when saving a contact (spec section 6).
    Sas,
    /// Content key of a link container (spec section 9).
    LinkContent,
    /// Manifest key of a link container (spec section 9).
    LinkManifest,
    /// Daily inbox address for offline delivery (spec section 9).
    Inbox,
    /// Sync blob key for accounts (spec section 10).
    Sync,
}

impl Purpose {
    /// The exact context string. These strings are part of the protocol and
    /// must never be translated or changed.
    pub const fn context(self) -> &'static str {
        match self {
            Purpose::ContentKey => "totaldrop TD0 content key",
            Purpose::NoncePrefix => "totaldrop TD0 nonce prefix",
            Purpose::Mailbox => "totaldrop TD0 mailbox",
            Purpose::NoisePsk => "totaldrop TD0 noise psk",
            Purpose::SessionI2R => "totaldrop TD0 session i2r",
            Purpose::SessionR2I => "totaldrop TD0 session r2i",
            Purpose::Sas => "totaldrop TD0 sas",
            Purpose::LinkContent => "totaldrop TD0 link content",
            Purpose::LinkManifest => "totaldrop TD0 link manifest",
            Purpose::Inbox => "totaldrop TD0 inbox",
            Purpose::Sync => "totaldrop TD0 sync",
        }
    }
}

/// BLAKE3 `derive_key` with the protocol's context string for `purpose`.
pub fn derive_key(purpose: Purpose, material: &[u8]) -> [u8; 32] {
    blake3::derive_key(purpose.context(), material)
}

/// Errors returned by this crate. They deliberately carry no detail that
/// could help an attacker distinguish failure causes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// A chunk failed authentication: it was altered, reordered, truncated,
    /// or encrypted under a different key.
    Inauthentic,
    /// A chunk exceeded [`CHUNK_SIZE`] or the input was otherwise malformed.
    Malformed,
    /// The operating system's random number generator failed.
    Random,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Error::Inauthentic => "chunk failed authentication",
            Error::Malformed => "malformed input",
            Error::Random => "random number generator failed",
        })
    }
}

impl std::error::Error for Error {}

/// The 32-byte content key `CK` of one transfer. Wiped from memory on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct ContentKey([u8; 32]);

impl ContentKey {
    /// A fresh random content key from the operating system CSPRNG.
    pub fn generate() -> Result<Self, Error> {
        let mut k = [0u8; 32];
        getrandom::fill(&mut k).map_err(|_| Error::Random)?;
        Ok(Self(k))
    }

    /// Wrap existing key bytes, for example a key received in an `offer`.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// The raw key bytes, for sending inside an encrypted `offer`.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl core::fmt::Debug for ContentKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("ContentKey(..)")
    }
}

/// A fresh random 16-byte transfer ID.
pub fn new_transfer_id() -> Result<[u8; 16], Error> {
    let mut id = [0u8; 16];
    getrandom::fill(&mut id).map_err(|_| Error::Random)?;
    Ok(id)
}

/// Encrypts and decrypts the chunks of one entry (one file) of a transfer,
/// following spec section 8:
///
/// ```text
/// k_i      = derive_key("totaldrop TD0 content key",  CK || transfer_id || u64be(i))
/// prefix_i = derive_key("totaldrop TD0 nonce prefix", CK || transfer_id || u64be(i))[0..7]
/// nonce    = prefix_i || u32be(chunk_index) || last_flag
/// AAD      = transfer_id || u64be(i) || u32be(chunk_index)
/// ```
pub struct EntryCipher {
    cipher: ChaCha20Poly1305,
    prefix: [u8; 7],
    transfer_id: [u8; 16],
    entry: u64,
}

impl EntryCipher {
    /// Set up the cipher for entry `entry` of the transfer `transfer_id`.
    pub fn new(ck: &ContentKey, transfer_id: &[u8; 16], entry: u64) -> Self {
        let mut material = [0u8; 32 + 16 + 8];
        let (ck_part, rest) = material.split_at_mut(32);
        let (id_part, entry_part) = rest.split_at_mut(16);
        ck_part.copy_from_slice(&ck.0);
        id_part.copy_from_slice(transfer_id);
        entry_part.copy_from_slice(&entry.to_be_bytes());

        let mut key = derive_key(Purpose::ContentKey, &material);
        let prefix_full = derive_key(Purpose::NoncePrefix, &material);
        material.zeroize();

        let mut prefix = [0u8; 7];
        prefix.copy_from_slice(prefix_full.get(..7).unwrap_or(&[0u8; 7]));
        let cipher = ChaCha20Poly1305::new(&Key::from(key));
        key.zeroize();

        Self {
            cipher,
            prefix,
            transfer_id: *transfer_id,
            entry,
        }
    }

    fn nonce(&self, chunk_index: u32, last: bool) -> Nonce {
        let mut n = [0u8; 12];
        let (p, rest) = n.split_at_mut(7);
        let (idx, flag) = rest.split_at_mut(4);
        p.copy_from_slice(&self.prefix);
        idx.copy_from_slice(&chunk_index.to_be_bytes());
        flag.copy_from_slice(&[u8::from(last)]);
        Nonce::from(n)
    }

    fn aad(&self, chunk_index: u32) -> [u8; 28] {
        let mut a = [0u8; 28];
        let (id, rest) = a.split_at_mut(16);
        let (entry, idx) = rest.split_at_mut(8);
        id.copy_from_slice(&self.transfer_id);
        entry.copy_from_slice(&self.entry.to_be_bytes());
        idx.copy_from_slice(&chunk_index.to_be_bytes());
        a
    }

    /// Encrypt one chunk. `last` must be true exactly for the final chunk of
    /// the entry; an empty file is a single empty final chunk.
    pub fn seal(&self, chunk_index: u32, last: bool, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
        if plaintext.len() > CHUNK_SIZE {
            return Err(Error::Malformed);
        }
        let aad = self.aad(chunk_index);
        self.cipher
            .encrypt(
                &self.nonce(chunk_index, last),
                Payload {
                    msg: plaintext,
                    aad: &aad,
                },
            )
            .map_err(|_| Error::Malformed)
    }

    /// Decrypt and authenticate one chunk. The receiver knows from the
    /// manifest which chunk is the last one, so truncation, reordering and
    /// chunks moved between entries or transfers all fail here.
    pub fn open(&self, chunk_index: u32, last: bool, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        if ciphertext.len() > CHUNK_SIZE.saturating_add(TAG_SIZE) || ciphertext.len() < TAG_SIZE {
            return Err(Error::Malformed);
        }
        let aad = self.aad(chunk_index);
        self.cipher
            .decrypt(
                &self.nonce(chunk_index, last),
                Payload {
                    msg: ciphertext,
                    aad: &aad,
                },
            )
            .map_err(|_| Error::Inauthentic)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn setup() -> (ContentKey, [u8; 16]) {
        (ContentKey::from_bytes([7u8; 32]), [9u8; 16])
    }

    #[test]
    fn context_strings_are_fixed() {
        assert_eq!(Purpose::ContentKey.context(), "totaldrop TD0 content key");
        assert_eq!(Purpose::Sas.context(), "totaldrop TD0 sas");
        assert_ne!(
            derive_key(Purpose::ContentKey, b"x"),
            derive_key(Purpose::NoncePrefix, b"x")
        );
    }

    #[test]
    fn round_trip() {
        let (ck, id) = setup();
        let c = EntryCipher::new(&ck, &id, 0);
        let ct = c.seal(0, false, b"hello").unwrap();
        assert_eq!(ct.len(), 5 + TAG_SIZE);
        assert_eq!(c.open(0, false, &ct).unwrap(), b"hello");
    }

    #[test]
    fn empty_file_is_one_final_chunk() {
        let (ck, id) = setup();
        let c = EntryCipher::new(&ck, &id, 3);
        let ct = c.seal(0, true, b"").unwrap();
        assert_eq!(c.open(0, true, &ct).unwrap(), b"");
    }

    #[test]
    fn tampering_is_detected() {
        let (ck, id) = setup();
        let c = EntryCipher::new(&ck, &id, 0);
        let mut ct = c.seal(0, false, b"hello").unwrap();
        ct[0] ^= 1;
        assert_eq!(c.open(0, false, &ct), Err(Error::Inauthentic));
    }

    #[test]
    fn reordering_is_detected() {
        let (ck, id) = setup();
        let c = EntryCipher::new(&ck, &id, 0);
        let first = c.seal(0, false, b"first").unwrap();
        assert_eq!(c.open(1, false, &first), Err(Error::Inauthentic));
    }

    #[test]
    fn truncation_is_detected() {
        let (ck, id) = setup();
        let c = EntryCipher::new(&ck, &id, 0);
        let not_last = c.seal(4, false, b"middle").unwrap();
        // An attacker drops the real last chunk and claims this one is last.
        assert_eq!(c.open(4, true, &not_last), Err(Error::Inauthentic));
    }

    #[test]
    fn chunks_do_not_move_between_entries_or_transfers() {
        let (ck, id) = setup();
        let a = EntryCipher::new(&ck, &id, 0);
        let b = EntryCipher::new(&ck, &id, 1);
        let other = EntryCipher::new(&ck, &[1u8; 16], 0);
        let ct = a.seal(0, true, b"secret").unwrap();
        assert_eq!(b.open(0, true, &ct), Err(Error::Inauthentic));
        assert_eq!(other.open(0, true, &ct), Err(Error::Inauthentic));
    }

    #[test]
    fn oversized_chunks_are_rejected() {
        let (ck, id) = setup();
        let c = EntryCipher::new(&ck, &id, 0);
        let big = vec![0u8; CHUNK_SIZE + 1];
        assert_eq!(c.seal(0, true, &big), Err(Error::Malformed));
        assert_eq!(c.open(0, true, &[0u8; 3]), Err(Error::Malformed));
    }

    #[test]
    fn random_keys_differ() {
        let a = ContentKey::generate().unwrap();
        let b = ContentKey::generate().unwrap();
        assert_ne!(a.as_bytes(), b.as_bytes());
        assert_eq!(format!("{a:?}"), "ContentKey(..)");
    }
}
