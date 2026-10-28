//! The relay's wire format (spec section 6, "Relay").
//!
//! A relay does two things:
//!
//! - **Address discovery** over UDP, at the same host and port as the relay's
//!   WebSocket URL: a device sends a [`PROBE_LEN`]-byte
//!   probe from the socket it will use for QUIC, and the relay answers with
//!   the address and port it saw. The answer is shorter than the probe, so
//!   the relay cannot be used to amplify traffic.
//! - **Forwarding** over WebSocket (TLS on TCP port 443, which passes almost
//!   every firewall): a device proves it holds a single-use endpoint key by
//!   signing a challenge, then sends QUIC datagrams addressed to another
//!   endpoint key. The datagrams are already encrypted end to end, so the
//!   relay sees only keys, sizes and timing.
//!
//! WebSocket messages are binary:
//!
//! | Direction | Message |
//! | --- | --- |
//! | relay → device | `TDR0` ‖ 32-byte challenge |
//! | device → relay | `TDR0` ‖ 32-byte endpoint key ‖ 64-byte Ed25519 signature over [`AUTH_CONTEXT`] ‖ challenge |
//! | relay → device | `TDR0` ‖ `ok` |
//! | either | 32-byte endpoint key ‖ datagram |
//!
//! In a datagram frame the key is the destination when the device sends and
//! the source when the relay delivers.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// Prefix of every relay control message.
pub const MAGIC: &[u8; 4] = b"TDR0";

/// Prefix of discovery probes and answers.
pub const PROBE_MAGIC: &[u8; 4] = b"TDD0";

/// Signed together with the challenge, so the signature means nothing
/// anywhere else.
pub const AUTH_CONTEXT: &[u8] = b"totaldrop relay v0 auth";

/// Size of a discovery probe: magic, 12-byte nonce, zero padding.
pub const PROBE_LEN: usize = 64;

/// Largest datagram a relay forwards. QUIC never sends more than about
/// 1,500 bytes per datagram; the margin covers jumbo paths.
pub const MAX_DATAGRAM: usize = 9_000;

/// Largest WebSocket message either side accepts.
pub const MAX_MESSAGE: usize = 32 + MAX_DATAGRAM;

/// The challenge message the relay sends first.
pub fn challenge(nonce: &[u8; 32]) -> Vec<u8> {
    let mut m = MAGIC.to_vec();
    m.extend_from_slice(nonce);
    m
}

/// Read a challenge message.
pub fn parse_challenge(m: &[u8]) -> Option<[u8; 32]> {
    m.strip_prefix(MAGIC.as_slice())?.try_into().ok()
}

/// What a device signs to prove it holds `key`.
pub fn signed_bytes(challenge: &[u8; 32]) -> Vec<u8> {
    let mut m = AUTH_CONTEXT.to_vec();
    m.extend_from_slice(challenge);
    m
}

/// The device's answer to a challenge.
pub fn hello(key: &[u8; 32], signature: &[u8; 64]) -> Vec<u8> {
    let mut m = MAGIC.to_vec();
    m.extend_from_slice(key);
    m.extend_from_slice(signature);
    m
}

/// Read a device's answer: its endpoint key and signature.
pub fn parse_hello(m: &[u8]) -> Option<([u8; 32], [u8; 64])> {
    let rest = m.strip_prefix(MAGIC.as_slice())?;
    if rest.len() != 96 {
        return None;
    }
    let (key, sig) = rest.split_at(32);
    Some((key.try_into().ok()?, sig.try_into().ok()?))
}

/// The relay's acceptance.
pub fn ok() -> Vec<u8> {
    let mut m = MAGIC.to_vec();
    m.extend_from_slice(b"ok");
    m
}

/// Whether `m` is the relay's acceptance.
pub fn is_ok(m: &[u8]) -> bool {
    m == ok().as_slice()
}

/// A datagram frame: `key` ‖ `datagram`.
pub fn frame(key: &[u8; 32], datagram: &[u8]) -> Vec<u8> {
    let mut m = Vec::with_capacity(32usize.saturating_add(datagram.len()));
    m.extend_from_slice(key);
    m.extend_from_slice(datagram);
    m
}

/// Split a datagram frame. Empty and oversized datagrams are refused.
pub fn parse_frame(m: &[u8]) -> Option<([u8; 32], &[u8])> {
    if m.len() <= 32 || m.len() > MAX_MESSAGE {
        return None;
    }
    let (key, datagram) = m.split_at(32);
    Some((key.try_into().ok()?, datagram))
}

/// A discovery probe carrying `nonce`.
pub fn probe(nonce: &[u8; 12]) -> [u8; PROBE_LEN] {
    let mut p = [0u8; PROBE_LEN];
    for (dst, src) in p.iter_mut().zip(PROBE_MAGIC.iter().chain(nonce.iter())) {
        *dst = *src;
    }
    p
}

/// The nonce of a well-formed probe.
pub fn parse_probe(p: &[u8]) -> Option<[u8; 12]> {
    if p.len() != PROBE_LEN {
        return None;
    }
    let rest = p.strip_prefix(PROBE_MAGIC.as_slice())?;
    rest.get(..12)?.try_into().ok()
}

/// The answer to a probe: magic ‖ nonce ‖ 4 or 6 ‖ IP bytes ‖ port.
pub fn answer(nonce: &[u8; 12], seen: SocketAddr) -> Vec<u8> {
    let mut a = PROBE_MAGIC.to_vec();
    a.extend_from_slice(nonce);
    match seen.ip() {
        IpAddr::V4(ip) => {
            a.push(4);
            a.extend_from_slice(&ip.octets());
        }
        IpAddr::V6(ip) => match ip.to_ipv4_mapped() {
            Some(v4) => {
                a.push(4);
                a.extend_from_slice(&v4.octets());
            }
            None => {
                a.push(6);
                a.extend_from_slice(&ip.octets());
            }
        },
    }
    a.extend_from_slice(&seen.port().to_be_bytes());
    a
}

/// Read an answer to the probe with `nonce`.
pub fn parse_answer(a: &[u8], nonce: &[u8; 12]) -> Option<SocketAddr> {
    let rest = a.strip_prefix(PROBE_MAGIC.as_slice())?;
    let rest = rest.strip_prefix(nonce.as_slice())?;
    let (&family, rest) = rest.split_first()?;
    let (ip, rest): (IpAddr, &[u8]) = match family {
        4 => {
            let (ip, rest) = rest.split_at_checked(4)?;
            let ip: [u8; 4] = ip.try_into().ok()?;
            (Ipv4Addr::from(ip).into(), rest)
        }
        6 => {
            let (ip, rest) = rest.split_at_checked(16)?;
            let ip: [u8; 16] = ip.try_into().ok()?;
            (Ipv6Addr::from(ip).into(), rest)
        }
        _ => return None,
    };
    let port: [u8; 2] = rest.try_into().ok()?;
    Some(SocketAddr::new(ip, u16::from_be_bytes(port)))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn control_messages_round_trip() {
        let nonce = [7u8; 32];
        assert_eq!(parse_challenge(&challenge(&nonce)), Some(nonce));
        assert_eq!(parse_challenge(b"TDR0short"), None);
        let (key, sig) = ([1u8; 32], [2u8; 64]);
        assert_eq!(parse_hello(&hello(&key, &sig)), Some((key, sig)));
        assert_eq!(parse_hello(&challenge(&nonce)), None);
        assert!(is_ok(&ok()));
        assert!(!is_ok(b"TDR0no"));
    }

    #[test]
    fn frames() {
        let key = [9u8; 32];
        let f = frame(&key, b"quic");
        assert_eq!(parse_frame(&f), Some((key, b"quic".as_slice())));
        assert_eq!(parse_frame(&key), None);
        assert_eq!(parse_frame(&vec![0u8; MAX_MESSAGE + 1]), None);
    }

    #[test]
    fn discovery_round_trip_and_no_amplification() {
        let nonce = [3u8; 12];
        let p = probe(&nonce);
        assert_eq!(parse_probe(&p), Some(nonce));
        assert_eq!(parse_probe(&p[..63]), None);
        for seen in [
            "203.0.113.5:40000".parse().unwrap(),
            "[2001:db8::1]:443".parse().unwrap(),
            "[::ffff:198.51.100.7]:9".parse().unwrap(),
        ] {
            let a = answer(&nonce, seen);
            assert!(a.len() < PROBE_LEN);
            let back = parse_answer(&a, &nonce).unwrap();
            assert_eq!(back.port(), seen.port());
        }
        let a = answer(&nonce, "203.0.113.5:1".parse().unwrap());
        assert_eq!(parse_answer(&a, &[4u8; 12]), None);
    }
}
