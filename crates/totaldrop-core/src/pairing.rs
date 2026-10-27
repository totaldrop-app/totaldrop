//! The mailbox phase of a code transfer (spec section 5): CPace, key
//! confirmation with the device certificates, and encrypted `HINTS`.

use std::time::{SystemTime, UNIX_EPOCH};

use totaldrop_crypto::channel::MailboxCipher;
use totaldrop_crypto::cpace::{Cpace, Role};
use totaldrop_crypto::identity::{DeviceKeys, verify_signature};
use totaldrop_crypto::{Purpose, derive_key};
use totaldrop_proto::TransferCode;
use totaldrop_proto::wire::{Certificate, Hints, Mood};

use crate::error::{Error, Result};
use crate::rendezvous::Mailbox;

/// Protocol versions this implementation speaks.
pub const VERSIONS: [u64; 1] = [0];

/// Build and sign this device's certificate (spec section 4).
pub fn issue_certificate(keys: &DeviceKeys) -> Certificate {
    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut cert = Certificate {
        version: 0,
        signing_key: keys.signing_public(),
        static_key: keys.static_public(),
        kem_key: keys.kem_public(),
        created,
        signature: [0u8; 64],
    };
    cert.signature = keys.sign(&cert.signed_bytes());
    cert
}

/// Check a peer's certificate signature over its own fields.
pub fn verify_certificate(cert: &Certificate) -> Result<()> {
    verify_signature(&cert.signing_key, &cert.signed_bytes(), &cert.signature)
        .map_err(|_| Error::WrongCode)
}

/// What the mailbox phase produces.
pub struct Paired {
    /// The verified certificate of the other device.
    pub peer: Certificate,
    /// The other device's connection hints.
    pub hints: Hints,
    /// The Noise PSK.
    pub psk: [u8; 32],
    /// The mailbox ID (CPace session ID, Noise prologue).
    pub mailbox_id: [u8; 16],
}

impl core::fmt::Debug for Paired {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Paired(..)")
    }
}

impl Drop for Paired {
    fn drop(&mut self) {
        zeroize::Zeroize::zeroize(&mut self.psk);
    }
}

/// Run CPace, key confirmation and the hints exchange in a claimed mailbox.
/// On a failed confirmation the mailbox is closed `scary` and
/// [`Error::WrongCode`] returned. The mailbox is always closed on return.
pub async fn exchange(
    mut mailbox: Mailbox,
    role: Role,
    code: &TransferCode,
    mailbox_id: [u8; 16],
    keys: &DeviceKeys,
    own_hints: &Hints,
) -> Result<Paired> {
    match run(&mut mailbox, role, code, mailbox_id, keys, own_hints).await {
        Ok(paired) => {
            mailbox.close(Mood::Happy).await;
            Ok(paired)
        }
        Err(Error::WrongCode) => {
            mailbox.close(Mood::Scary).await;
            Err(Error::WrongCode)
        }
        Err(e) => {
            mailbox.close(Mood::Lonely).await;
            Err(e)
        }
    }
}

async fn run(
    mailbox: &mut Mailbox,
    role: Role,
    code: &TransferCode,
    mailbox_id: [u8; 16],
    keys: &DeviceKeys,
    own_hints: &Hints,
) -> Result<Paired> {
    // CPace receives the canonical code, never the words.
    let prs = code.canonical();
    let (cpace, own_msg) = Cpace::start(role, prs.as_bytes(), &mailbox_id)?;
    mailbox.add("pake", own_msg.to_vec()).await?;
    let peer_msg = mailbox.expect("pake").await?;
    let isk = cpace.finish(&peer_msg).map_err(|_| Error::WrongCode)?;

    let mut cipher = MailboxCipher::new(&isk, role);
    let psk = derive_key(Purpose::NoisePsk, isk.as_bytes());
    drop(isk);

    // Key confirmation: the first encrypted message carries the certificate.
    let own_cert = issue_certificate(keys);
    mailbox
        .add("confirm", cipher.seal("confirm", &own_cert.encode())?)
        .await?;
    // A peer that leaves after seeing our confirmation may have taken its
    // one guess offline; count it as a failed confirmation, so the
    // nameplate is retired and the attempt is visible.
    let sealed = match mailbox.expect("confirm").await {
        Err(Error::PeerLeft) => return Err(Error::WrongCode),
        other => other?,
    };
    let plain = cipher
        .open("confirm", &sealed)
        .map_err(|_| Error::WrongCode)?;
    let peer = Certificate::decode(&plain).map_err(|_| Error::WrongCode)?;
    verify_certificate(&peer)?;

    mailbox
        .add("hints", cipher.seal("hints", &own_hints.encode())?)
        .await?;
    let sealed = mailbox.expect("hints").await?;
    let plain = cipher
        .open("hints", &sealed)
        .map_err(|_| Error::WrongCode)?;
    let hints = Hints::decode(&plain)?;
    if !hints.versions.iter().any(|v| VERSIONS.contains(v)) {
        return Err(Error::Protocol("no common protocol version"));
    }

    Ok(Paired {
        peer,
        hints,
        psk,
        mailbox_id,
    })
}
