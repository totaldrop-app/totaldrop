//! The secure handshake on the first QUIC stream (spec section 6):
//! Noise XXpsk3 with the PSK from CPace, then the ML-KEM-768 hybrid step,
//! then key confirmation under the new keys.

use totaldrop_crypto::channel::{MAX_SESSION_MESSAGE, Opener, Sealer, session, session_keys};
use totaldrop_crypto::identity::{DeviceKeys, kem_encapsulate};
use totaldrop_proto::wire::{Certificate, Control, MAX_FRAME};
use zeroize::Zeroize;

use crate::error::{Error, Result};
use crate::framing::{expect_frame, write_frame};

const NOISE_XXPSK3: &str = "Noise_XXpsk3_25519_ChaChaPoly_BLAKE2b";
const NOISE_MAX: usize = 65535;

/// An established secure session.
pub struct SecureChannel {
    /// Encrypts outgoing control messages.
    pub sealer: Sealer,
    /// Decrypts incoming control messages.
    pub opener: Opener,
    /// The control stream, sending half.
    pub send: quinn::SendStream,
    /// The control stream, receiving half.
    pub recv: quinn::RecvStream,
    /// Indices of the four verification words.
    pub verification: [u16; 4],
}

impl core::fmt::Debug for SecureChannel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("SecureChannel(..)")
    }
}

/// Write one control message.
pub async fn send_control(
    sealer: &mut Sealer,
    send: &mut quinn::SendStream,
    msg: &Control,
) -> Result<()> {
    let ct = sealer.seal(&msg.encode())?;
    write_frame(send, &ct).await
}

/// Read one control message.
pub async fn recv_control(opener: &mut Opener, recv: &mut quinn::RecvStream) -> Result<Control> {
    let ct = expect_frame(recv, MAX_SESSION_MESSAGE.saturating_add(64)).await?;
    let plain = opener.open(&ct)?;
    Ok(Control::decode(&plain)?)
}

/// Everything the handshake needs from the mailbox phase.
pub struct HandshakeInput<'a> {
    /// This device's keys.
    pub keys: &'a DeviceKeys,
    /// The peer's verified certificate, received in the `confirm` phase.
    pub peer: &'a Certificate,
    /// The Noise PSK derived from CPace.
    pub psk: [u8; 32],
    /// The mailbox ID, bound into the Noise prologue.
    pub mailbox_id: [u8; 16],
}

fn prologue(mailbox_id: &[u8; 16]) -> Vec<u8> {
    let mut p = b"totaldrop/0".to_vec();
    p.extend_from_slice(mailbox_id);
    p
}

fn noise_err(_: snow::Error) -> Error {
    // Any failure here means the peer does not hold the code or the keys.
    Error::WrongCode
}

async fn write_noise(state: &mut snow::HandshakeState, send: &mut quinn::SendStream) -> Result<()> {
    let mut buf = vec![0u8; NOISE_MAX];
    let n = state.write_message(&[], &mut buf).map_err(noise_err)?;
    buf.truncate(n);
    write_frame(send, &buf).await
}

async fn read_noise(state: &mut snow::HandshakeState, recv: &mut quinn::RecvStream) -> Result<()> {
    let msg = expect_frame(recv, NOISE_MAX).await?;
    let mut buf = vec![0u8; NOISE_MAX];
    let n = state.read_message(&msg, &mut buf).map_err(noise_err)?;
    if n != 0 {
        return Err(Error::Protocol("unexpected handshake payload"));
    }
    Ok(())
}

/// Run the handshake. `initiator` is the QUIC client (the receiver of the
/// transfer); it opens the control stream.
pub async fn handshake(
    conn: &quinn::Connection,
    input: HandshakeInput<'_>,
    initiator: bool,
) -> Result<SecureChannel> {
    let (mut send, mut recv) = if initiator {
        conn.open_bi().await?
    } else {
        conn.accept_bi().await?
    };
    let mut psk = input.psk;
    let params: snow::params::NoiseParams = NOISE_XXPSK3
        .parse()
        .map_err(|_| Error::Protocol("noise parameters"))?;
    let prologue = prologue(&input.mailbox_id);
    let builder = snow::Builder::new(params)
        .local_private_key(input.keys.static_secret())
        .map_err(noise_err)?
        .psk(3, &psk)
        .map_err(noise_err)?
        .prologue(&prologue)
        .map_err(noise_err)?;
    let mut state = if initiator {
        builder.build_initiator()
    } else {
        builder.build_responder()
    }
    .map_err(noise_err)?;
    psk.zeroize();

    // XX: -> e ; <- e, ee, s, es ; -> s, se, psk
    if initiator {
        write_noise(&mut state, &mut send).await?;
        read_noise(&mut state, &mut recv).await?;
        write_noise(&mut state, &mut send).await?;
    } else {
        read_noise(&mut state, &mut recv).await?;
        write_noise(&mut state, &mut send).await?;
        read_noise(&mut state, &mut recv).await?;
    }

    // The peer's static key must be the one in its signed certificate.
    if state.get_remote_static() != Some(&input.peer.static_key[..]) {
        return Err(Error::WrongCode);
    }
    let h = state.get_handshake_hash().to_vec();
    let mut transport = state.into_transport_mode().map_err(noise_err)?;

    // Post-quantum hybrid step: the initiator encapsulates to the
    // responder's ML-KEM key and sends the ciphertext as the first
    // transport message.
    let mut ss = if initiator {
        let (ct, ss) = kem_encapsulate(&input.peer.kem_key)?;
        let mut buf = vec![0u8; NOISE_MAX];
        let n = transport.write_message(&ct, &mut buf).map_err(noise_err)?;
        buf.truncate(n);
        write_frame(&mut send, &buf).await?;
        ss
    } else {
        let msg = expect_frame(&mut recv, NOISE_MAX).await?;
        let mut buf = vec![0u8; NOISE_MAX];
        let n = transport.read_message(&msg, &mut buf).map_err(noise_err)?;
        buf.truncate(n);
        input.keys.decapsulate(&buf)?
    };
    let (k_i2r, k_r2i) = session_keys(&h, &ss);
    let verification = totaldrop_crypto::channel::verification_indices(&h, &ss);
    ss.zeroize();
    drop(transport);
    let (mut sealer, mut opener) = session(k_i2r, k_r2i, initiator);

    // Key confirmation: nothing else flows until the peer proves it holds
    // the new keys.
    send_control(&mut sealer, &mut send, &Control::Confirm).await?;
    let confirm = expect_frame(&mut recv, MAX_FRAME).await?;
    let plain = opener.open(&confirm).map_err(|_| Error::WrongCode)?;
    if Control::decode(&plain)? != Control::Confirm {
        return Err(Error::Protocol("expected key confirmation"));
    }

    Ok(SecureChannel {
        sealer,
        opener,
        send,
        recv,
        verification,
    })
}
