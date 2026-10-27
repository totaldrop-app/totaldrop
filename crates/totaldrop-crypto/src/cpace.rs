//! CPace over ristretto255 with SHA-512 (spec section 5).
//!
//! This follows draft-irtf-cfrg-cpace exactly, for the initiator/responder
//! setting: the sender of a transfer is always the initiator `A`, the
//! receiver is always the responder `B`. Group arithmetic comes from
//! `curve25519-dalek`; this module only does the byte layout the draft
//! defines, and it is checked against the draft's own test vectors.
//!
//! Inputs, as fixed by the TotalDrop specification:
//! - `PRS` (the password) is the canonical form of the transfer code,
//!   for example `7-840-1499`, never the words;
//! - `sid` is the 16-byte mailbox ID from the rendezvous server;
//! - `CI` (channel identifier) is `totaldrop/0`.

// Group operations (`*` on points and scalars) are modular arithmetic, not
// integer arithmetic that could overflow.
#![allow(clippy::arithmetic_side_effects)]

use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::traits::IsIdentity;
use sha2::{Digest, Sha512};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::Error;

/// Domain separation of the ristretto255 group (draft section on ristretto255).
const DSI: &[u8] = b"CPaceRistretto255";
/// Domain separation for the final session key.
const DSI_ISK: &[u8] = b"CPaceRistretto255_ISK";
/// SHA-512 input block size, used for the zero padding of the generator string.
const S_IN_BYTES: usize = 128;

/// The channel identifier TotalDrop uses for every code transfer.
pub const CHANNEL_ID: &[u8] = b"totaldrop/0";

/// Size of one CPace message: one encoded ristretto255 element.
pub const MESSAGE_SIZE: usize = 32;

/// Which side of the exchange this party is. The order matters for the
/// transcript, so both sides must agree: sender = initiator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Party `A`; in TotalDrop, the sender.
    Initiator,
    /// Party `B`; in TotalDrop, the receiver.
    Responder,
}

/// Appends the unsigned LEB128 length of `data`, then `data` (draft `prepend_len`).
fn prepend_len(out: &mut Vec<u8>, data: &[u8]) {
    let mut length = data.len();
    loop {
        let low = u8::try_from(length & 0x7f).unwrap_or(0);
        length >>= 7;
        if length == 0 {
            out.push(low);
            break;
        }
        out.push(low | 0x80);
    }
    out.extend_from_slice(data);
}

/// Draft `lv_cat`: every argument with its length prepended.
fn lv_cat(parts: &[&[u8]]) -> Vec<u8> {
    let mut out = Vec::new();
    for part in parts {
        prepend_len(&mut out, part);
    }
    out
}

/// `len(prepend_len(data))`, computed without copying `data`.
fn len_of_prepend_len(data: &[u8]) -> usize {
    let mut n = data.len();
    let mut size = 1usize;
    while n >= 0x80 {
        n >>= 7;
        size = size.saturating_add(1);
    }
    size.saturating_add(data.len())
}

/// Draft `generator_string(DSI, PRS, CI, sid, s_in_bytes)`.
fn generator_string(prs: &[u8], ci: &[u8], sid: &[u8]) -> Vec<u8> {
    let used = 1usize
        .saturating_add(len_of_prepend_len(prs))
        .saturating_add(len_of_prepend_len(DSI));
    let zpad_len = S_IN_BYTES.saturating_sub(used);
    let zpad = vec![0u8; zpad_len];
    lv_cat(&[DSI, prs, &zpad, ci, sid])
}

/// Draft `calculate_generator`: hash the generator string with SHA-512 and
/// map the 64 bytes to the group with the ristretto255 one-way map.
fn calculate_generator(prs: &[u8], ci: &[u8], sid: &[u8]) -> RistrettoPoint {
    let digest = Sha512::digest(generator_string(prs, ci, sid));
    let mut wide = [0u8; 64];
    wide.copy_from_slice(&digest);
    let g = RistrettoPoint::from_uniform_bytes(&wide);
    wide.zeroize();
    g
}

/// Draft `sample_scalar` for ristretto255: 64 uniform bytes reduced mod the
/// group order.
fn sample_scalar() -> Result<Scalar, Error> {
    let mut wide = [0u8; 64];
    getrandom::fill(&mut wide).map_err(|_| Error::Random)?;
    let s = Scalar::from_bytes_mod_order_wide(&wide);
    wide.zeroize();
    Ok(s)
}

/// The 64-byte intermediate session key. Wiped from memory on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Isk([u8; 64]);

impl Isk {
    /// The raw key bytes, only for deriving further keys.
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl core::fmt::Debug for Isk {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Isk(..)")
    }
}

/// One side of a CPace run, between sending its message and receiving the
/// peer's. Consumed by [`Cpace::finish`], so a run cannot be reused.
pub struct Cpace {
    role: Role,
    secret: Scalar,
    own_message: [u8; MESSAGE_SIZE],
    sid: Vec<u8>,
}

impl Drop for Cpace {
    fn drop(&mut self) {
        self.secret.zeroize();
    }
}

impl core::fmt::Debug for Cpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Cpace")
            .field("role", &self.role)
            .finish_non_exhaustive()
    }
}

impl Cpace {
    /// Start a run. Returns the state and the message to send to the peer.
    /// `prs` is the canonical code, `sid` the mailbox ID.
    pub fn start(role: Role, prs: &[u8], sid: &[u8]) -> Result<(Self, [u8; MESSAGE_SIZE]), Error> {
        let secret = sample_scalar()?;
        Ok(Self::start_with_scalar(role, prs, CHANNEL_ID, sid, secret))
    }

    fn start_with_scalar(
        role: Role,
        prs: &[u8],
        ci: &[u8],
        sid: &[u8],
        secret: Scalar,
    ) -> (Self, [u8; MESSAGE_SIZE]) {
        let g = calculate_generator(prs, ci, sid);
        let own_message = (g * secret).compress().to_bytes();
        let state = Self {
            role,
            secret,
            own_message,
            sid: sid.to_vec(),
        };
        (state, own_message)
    }

    /// Finish with the peer's message. Fails if the message is not a valid
    /// group element or yields the identity (draft `scalar_mult_vfy`).
    /// Associated data is empty in TotalDrop.
    pub fn finish(self, peer_message: &[u8]) -> Result<Isk, Error> {
        let peer: [u8; MESSAGE_SIZE] = peer_message.try_into().map_err(|_| Error::Malformed)?;
        let point = CompressedRistretto(peer)
            .decompress()
            .ok_or(Error::Inauthentic)?;
        let shared = point * self.secret;
        if shared.is_identity() {
            return Err(Error::Inauthentic);
        }
        let mut k = shared.compress().to_bytes();
        let (ya, yb) = match self.role {
            Role::Initiator => (self.own_message, peer),
            Role::Responder => (peer, self.own_message),
        };
        let isk = isk_ir(&self.sid, &k, &ya, b"", &yb, b"");
        k.zeroize();
        Ok(isk)
    }
}

/// ISK = SHA-512(lv_cat(DSI_ISK, sid, K) || transcript_ir(Ya, ADa, Yb, ADb)).
fn isk_ir(sid: &[u8], k: &[u8], ya: &[u8], ada: &[u8], yb: &[u8], adb: &[u8]) -> Isk {
    let mut input = lv_cat(&[DSI_ISK, sid, k]);
    input.extend_from_slice(&lv_cat(&[ya, ada]));
    input.extend_from_slice(&lv_cat(&[yb, adb]));
    let digest = Sha512::digest(&input);
    input.zeroize();
    let mut out = [0u8; 64];
    out.copy_from_slice(&digest);
    Isk(out)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    //! Test vectors from draft-irtf-cfrg-cpace, section
    //! "Test vector for CPace using group ristretto255 and hash SHA-512".
    use super::*;

    fn hex(s: &str) -> Vec<u8> {
        let s: String = s.split_whitespace().collect();
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    fn scalar(s: &str) -> Scalar {
        let b: [u8; 32] = hex(s).try_into().unwrap();
        Scalar::from_canonical_bytes(b).unwrap()
    }

    const PRS: &[u8] = b"Password";
    const CI_HEX: &str = "0b415f696e69746961746f720b425f726573706f6e646572";
    const SID_HEX: &str = "7e4b4791d6a8ef019b936c79fb7f2c57";
    const YA_SECRET: &str = "da3d23700a9e5699258aef94dc060dfda5ebb61f02a5ea77fad53f4ff0976d08";
    const YB_SECRET: &str = "d2316b454718c35362d83d69df6320f38578ed5984651435e2949762d900b80d";

    #[test]
    fn prepend_len_vectors() {
        let mut out = Vec::new();
        prepend_len(&mut out, b"");
        assert_eq!(out, hex("00"));
        let mut out = Vec::new();
        prepend_len(&mut out, b"1234");
        assert_eq!(out, hex("0431323334"));
        let data: Vec<u8> = (0u8..128).collect();
        let mut out = Vec::new();
        prepend_len(&mut out, &data);
        assert_eq!(&out[..2], &hex("8001")[..]);
        assert_eq!(len_of_prepend_len(&data), 130);
        assert_eq!(len_of_prepend_len(&data[..127]), 128);
    }

    #[test]
    fn lv_cat_vector() {
        assert_eq!(
            lv_cat(&[b"1234", b"5", b"", b"678"]),
            hex("043132333401350003363738")
        );
    }

    #[test]
    fn generator_vector() {
        let ci = hex(CI_HEX);
        let sid = hex(SID_HEX);
        let gs = generator_string(PRS, &ci, &sid);
        assert_eq!(gs.len(), 170);
        assert_eq!(
            gs,
            hex("11435061636552697374726574746f3235350850617373776f726464
                 00000000000000000000000000000000000000000000000000000000
                 00000000000000000000000000000000000000000000000000000000
                 00000000000000000000000000000000000000000000000000000000
                 00000000000000000000000000000000180b415f696e69746961746f
                 720b425f726573706f6e646572107e4b4791d6a8ef019b936c79fb7f
                 2c57")
        );
        let g = calculate_generator(PRS, &ci, &sid);
        assert_eq!(
            g.compress().to_bytes().to_vec(),
            hex("222b6b195fe84b1652badb6f6a3ae3d24341e7306967f0b8115b40d5698c7e56")
        );
    }

    #[test]
    fn full_exchange_vector() {
        let ci = hex(CI_HEX);
        let sid = hex(SID_HEX);
        let (a, ya) = Cpace::start_with_scalar(Role::Initiator, PRS, &ci, &sid, scalar(YA_SECRET));
        let (b, yb) = Cpace::start_with_scalar(Role::Responder, PRS, &ci, &sid, scalar(YB_SECRET));
        assert_eq!(
            ya.to_vec(),
            hex("d6bac480f2c386c394efc7c47adb9925dcd2630b64f240c50f8d0eec482b9157")
        );
        assert_eq!(
            yb.to_vec(),
            hex("3ea7e0b19560d7c0b0f5734f63b955286dfa8232b5ebe63324e2d9e7433f7258")
        );

        // The draft's vector uses ADa = "ADa" and ADb = "ADb"; TotalDrop uses
        // empty AD, so check the ISK formula with the draft's AD directly.
        let k = hex("80b69a8a76457ab6a4d7f887a4bf6b55a2f80ac19c333f917a05fc9887c8b40f");
        let isk = isk_ir(&sid, &k, &ya, b"ADa", &yb, b"ADb");
        assert_eq!(
            isk.as_bytes().to_vec(),
            hex("b69effbf61b51d56401c0f65601abe428de8206feaaf0e32198896dc
                 ae7b35cd2b38950a39dfd5d4a79164614c2984f7daa460b588c1e80c
                 3fa2068af7900447")
        );

        // K itself, from both sides.
        let ka = (CompressedRistretto(yb).decompress().unwrap() * scalar(YA_SECRET))
            .compress()
            .to_bytes();
        let kb = (CompressedRistretto(ya).decompress().unwrap() * scalar(YB_SECRET))
            .compress()
            .to_bytes();
        assert_eq!(ka.to_vec(), k);
        assert_eq!(kb.to_vec(), k);

        // And both sides of a real run agree.
        let isk_a = a.finish(&yb).unwrap();
        let isk_b = b.finish(&ya).unwrap();
        assert_eq!(isk_a.as_bytes(), isk_b.as_bytes());
    }

    #[test]
    fn wrong_password_gives_different_keys() {
        let sid = [7u8; 16];
        let (a, ya) = Cpace::start(Role::Initiator, b"7-840-1499", &sid).unwrap();
        let (b, yb) = Cpace::start(Role::Responder, b"7-840-1498", &sid).unwrap();
        assert_ne!(
            a.finish(&yb).unwrap().as_bytes(),
            b.finish(&ya).unwrap().as_bytes()
        );
    }

    #[test]
    fn same_password_agrees() {
        let sid = [9u8; 16];
        let (a, ya) = Cpace::start(Role::Initiator, b"7-840-1499", &sid).unwrap();
        let (b, yb) = Cpace::start(Role::Responder, b"7-840-1499", &sid).unwrap();
        assert_eq!(
            a.finish(&yb).unwrap().as_bytes(),
            b.finish(&ya).unwrap().as_bytes()
        );
    }

    #[test]
    fn invalid_points_are_rejected() {
        let sid = [1u8; 16];
        // Identity element.
        let (a, _) = Cpace::start(Role::Initiator, b"1-2-3", &sid).unwrap();
        assert_eq!(a.finish(&[0u8; 32]).unwrap_err(), Error::Inauthentic);
        // Non-canonical encoding (all 0xff is not a valid field element).
        let (a, _) = Cpace::start(Role::Initiator, b"1-2-3", &sid).unwrap();
        assert_eq!(a.finish(&[0xffu8; 32]).unwrap_err(), Error::Inauthentic);
        // Wrong length.
        let (a, _) = Cpace::start(Role::Initiator, b"1-2-3", &sid).unwrap();
        assert_eq!(a.finish(&[1u8; 31]).unwrap_err(), Error::Malformed);
    }
}
