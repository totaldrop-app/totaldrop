//! Device keys (spec section 4) and the ML-KEM-768 half of the hybrid
//! handshake (spec section 6).

use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use ml_kem::kem::{Decapsulate, Encapsulate, KeyExport};
use ml_kem::{EncapsulationKey, MlKem768, Seed};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::Error;

/// Size of an ML-KEM-768 encapsulation key.
pub const KEM_PUBLIC_SIZE: usize = 1184;
/// Size of an ML-KEM-768 ciphertext.
pub const KEM_CIPHERTEXT_SIZE: usize = 1088;

fn random<const N: usize>() -> Result<[u8; N], Error> {
    let mut b = [0u8; N];
    getrandom::fill(&mut b).map_err(|_| Error::Random)?;
    Ok(b)
}

/// The three key pairs of one device: an Ed25519 signing key, an X25519
/// static key for Noise and an ML-KEM-768 key. Secret parts are wiped from
/// memory on drop.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct DeviceKeys {
    signing_seed: [u8; 32],
    static_secret: [u8; 32],
    kem_seed: [u8; 64],
}

impl core::fmt::Debug for DeviceKeys {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("DeviceKeys(..)")
    }
}

impl DeviceKeys {
    /// Generate fresh keys from the operating system CSPRNG.
    pub fn generate() -> Result<Self, Error> {
        Ok(Self {
            signing_seed: random()?,
            static_secret: random()?,
            kem_seed: random()?,
        })
    }

    fn signing_key(&self) -> SigningKey {
        SigningKey::from_bytes(&self.signing_seed)
    }

    /// The Ed25519 public key; the device's identity.
    pub fn signing_public(&self) -> [u8; 32] {
        self.signing_key().verifying_key().to_bytes()
    }

    /// The X25519 public key used as the Noise static key.
    pub fn static_public(&self) -> [u8; 32] {
        MontgomeryPoint::mul_base_clamped(self.static_secret).to_bytes()
    }

    /// The X25519 private key, only for handing to the Noise implementation.
    pub fn static_secret(&self) -> &[u8; 32] {
        &self.static_secret
    }

    fn kem_decapsulation_key(&self) -> ml_kem::DecapsulationKey<MlKem768> {
        ml_kem::DecapsulationKey::<MlKem768>::from_seed(Seed::from(self.kem_seed))
    }

    /// The 1184-byte ML-KEM-768 encapsulation key.
    pub fn kem_public(&self) -> Vec<u8> {
        self.kem_decapsulation_key()
            .encapsulation_key()
            .to_bytes()
            .to_vec()
    }

    /// Sign `message` with the device's Ed25519 key.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.signing_key().sign(message).to_bytes()
    }

    /// Recover the ML-KEM shared secret from a ciphertext sent to this device.
    pub fn decapsulate(&self, ciphertext: &[u8]) -> Result<[u8; 32], Error> {
        if ciphertext.len() != KEM_CIPHERTEXT_SIZE {
            return Err(Error::Malformed);
        }
        let ss = self
            .kem_decapsulation_key()
            .decapsulate_slice(ciphertext)
            .map_err(|_| Error::Malformed)?;
        let mut out = [0u8; 32];
        out.copy_from_slice(&ss);
        Ok(out)
    }
}

/// Encapsulate a fresh shared secret to a peer's ML-KEM-768 key. Returns the
/// ciphertext to send and the shared secret.
pub fn kem_encapsulate(peer_kem_public: &[u8]) -> Result<(Vec<u8>, [u8; 32]), Error> {
    let key = ml_kem::kem::Key::<EncapsulationKey<MlKem768>>::try_from(peer_kem_public)
        .map_err(|_| Error::Malformed)?;
    let ek = EncapsulationKey::<MlKem768>::new(&key).map_err(|_| Error::Malformed)?;
    let (ct, ss) = ek.encapsulate();
    let mut out = [0u8; 32];
    out.copy_from_slice(&ss);
    Ok((ct.to_vec(), out))
}

/// Verify an Ed25519 signature strictly (RFC 8032 with the stricter checks
/// that reject malleable and small-order encodings).
pub fn verify_signature(public: &[u8; 32], message: &[u8], signature: &[u8]) -> Result<(), Error> {
    let key = VerifyingKey::from_bytes(public).map_err(|_| Error::Inauthentic)?;
    let sig = Signature::from_slice(signature).map_err(|_| Error::Inauthentic)?;
    key.verify_strict(message, &sig)
        .map_err(|_| Error::Inauthentic)
}

/// Device ID: the first 32 bytes of BLAKE3(signing key) (spec section 4).
pub fn device_id(signing_public: &[u8; 32]) -> [u8; 32] {
    *blake3::hash(signing_public).as_bytes()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn kem_round_trip() {
        let keys = DeviceKeys::generate().unwrap();
        let public = keys.kem_public();
        assert_eq!(public.len(), KEM_PUBLIC_SIZE);
        let (ct, ss) = kem_encapsulate(&public).unwrap();
        assert_eq!(ct.len(), KEM_CIPHERTEXT_SIZE);
        assert_eq!(keys.decapsulate(&ct).unwrap(), ss);
    }

    #[test]
    fn kem_rejects_bad_sizes() {
        let keys = DeviceKeys::generate().unwrap();
        assert!(kem_encapsulate(&[0u8; 10]).is_err());
        assert!(keys.decapsulate(&[0u8; 10]).is_err());
    }

    #[test]
    fn signatures() {
        let keys = DeviceKeys::generate().unwrap();
        let sig = keys.sign(b"manifest");
        verify_signature(&keys.signing_public(), b"manifest", &sig).unwrap();
        assert!(verify_signature(&keys.signing_public(), b"other", &sig).is_err());
        let other = DeviceKeys::generate().unwrap();
        assert!(verify_signature(&other.signing_public(), b"manifest", &sig).is_err());
    }

    #[test]
    fn static_key_matches_x25519() {
        let keys = DeviceKeys::generate().unwrap();
        // X25519(sk, 9) via the Montgomery ladder must match mul_base_clamped.
        let base = MontgomeryPoint(curve25519_dalek::constants::X25519_BASEPOINT.to_bytes());
        assert_eq!(
            base.mul_clamped(*keys.static_secret()).to_bytes(),
            keys.static_public()
        );
    }
}
