// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! The relay's wire format (spec section 6, "Relay").
//!
//! Two services help two devices find a path to each other:
//!
//! - **Address discovery** is plain STUN (RFC 8489): a device sends a
//!   Binding request from the socket it will use for QUIC to a STUN server
//!   (by default Cloudflare's), and the answer says which public address and
//!   port the server saw. Only the Binding method and XOR-MAPPED-ADDRESS are
//!   used.
//! - **Forwarding** runs over WebSocket (TLS on TCP port 443, which passes
//!   almost every firewall). Both devices of a transfer connect to
//!   `<relay URL>/<key A>-<key B>`, the two single-use endpoint keys in hex,
//!   lower one first, so a relay can put the two in one place and refuse any
//!   other key. Each proves it holds its key by signing a challenge, then
//!   sends QUIC datagrams to the other. The datagrams are already encrypted
//!   end to end, so the relay sees only keys, sizes and timing.
//!
//! WebSocket messages are binary:
//!
//! | Direction | Message |
//! | --- | --- |
//! | relay → device | `TDR0` ‖ 32-byte challenge |
//! | device → relay | `TDR0` ‖ 32-byte endpoint key ‖ 64-byte Ed25519 signature over [`AUTH_CONTEXT`] ‖ challenge |
//! | relay → device | `TDR0` ‖ `ok` |
//! | either | 32-byte endpoint key ‖ one or more (u16 big-endian length ‖ datagram) |
//! | relay → device | `TDR0` ‖ `probe` ‖ 16 random bytes |
//! | device → relay | the same probe, unchanged |
//!
//! In a datagram message the key is the destination when the device sends
//! and the source when the relay delivers; a relay only swaps the key and
//! passes the rest through. Several datagrams share one message so a relay
//! handles fewer, larger messages.
//!
//! After `ok`, a relay may put a probe between the datagram messages it
//! delivers, and the device sends it back as soon as it reads it. The device
//! sees the probe only after reading everything sent before it, so the
//! answer tells the relay how much it still holds for the device; a relay
//! that cannot see its own send buffer bounds it this way (#190). A probe is
//! 25 bytes and a datagram message at least 35, so the two never mix up;
//! devices from before probes ignore them. Protocol note 20 lists what a
//! relay that sends probes must do.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// Prefix of every relay control message.
pub const MAGIC: &[u8; 4] = b"TDR0";

/// Signed together with the challenge, so the signature means nothing
/// anywhere else.
pub const AUTH_CONTEXT: &[u8] = b"totaldrop relay v0 auth";

/// Signed together with the pair name to authenticate the HTTP Upgrade
/// request before holding a slot (#78). Does not include `Sec-WebSocket-Key`
/// because HTTP/WebSocket proxies (including Cloudflare edge and `workerd`)
/// regenerate that hop-by-hop header; freshness is still proved by `TDR0`.
pub const CONNECT_CONTEXT: &[u8] = b"totaldrop relay v0 connect";

/// HTTP request header carrying `<64-hex-key>.<128-hex-sig>` on Upgrade
/// so a relay can admit key holders ahead of unsigned waiters (#78).
pub const AUTH_HEADER: &str = "X-TotalDrop-Auth";

/// Largest datagram a relay forwards. QUIC never sends more than about
/// 1,500 bytes per datagram; the margin covers jumbo paths.
pub const MAX_DATAGRAM: usize = 9_000;

/// Largest WebSocket message either side sends or accepts.
pub const MAX_MESSAGE: usize = 64 * 1024;

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

/// What a device signs to prove it holds its key.
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

/// A probe with the relay's random `nonce`.
pub fn probe(nonce: &[u8; 16]) -> Vec<u8> {
    let mut m = MAGIC.to_vec();
    m.extend_from_slice(b"probe");
    m.extend_from_slice(nonce);
    m
}

/// Whether `m` is a probe, which the device sends back unchanged.
pub fn is_probe(m: &[u8]) -> bool {
    m.len() == 25 && m.starts_with(MAGIC) && m.get(4..9) == Some(b"probe".as_slice())
}

/// What a device signs for [`AUTH_HEADER`] on the HTTP Upgrade request:
/// [`CONNECT_CONTEXT`] ‖ `pair_name` (#78).
pub fn connect_signed_bytes(pair_name: &str) -> Vec<u8> {
    let cap = CONNECT_CONTEXT.len().saturating_add(pair_name.len());
    let mut m = Vec::with_capacity(cap);
    m.extend_from_slice(CONNECT_CONTEXT);
    m.extend_from_slice(pair_name.as_bytes());
    m
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex_exact<const N: usize>(s: &str) -> Option<[u8; N]> {
    let expected = N.checked_mul(2)?;
    if s.len() != expected || !s.bytes().all(|c| matches!(c, b'0'..=b'9' | b'a'..=b'f')) {
        return None;
    }
    let mut out = [0u8; N];
    for (i, b) in out.iter_mut().enumerate() {
        let at = i.checked_mul(2)?;
        *b = u8::from_str_radix(s.get(at..at.checked_add(2)?)?, 16).ok()?;
    }
    Some(out)
}

/// Format the [`AUTH_HEADER`] value: `<64-hex-key>.<128-hex-sig>` in
/// lowercase ASCII hex (193 bytes) (#78).
pub fn format_auth_header(key: &[u8; 32], signature: &[u8; 64]) -> String {
    format!("{}.{}", hex_bytes(key), hex_bytes(signature))
}

/// Parse a strict [`AUTH_HEADER`] value into `(endpoint_key, signature)`.
///
/// Accepts only exact 193-byte lowercase ASCII hex `<64-hex>.<128-hex>`;
/// rejects wrong lengths, uppercase hex, whitespace, extra dots, or
/// non-hex characters (#78).
pub fn parse_auth_header(value: &str) -> Option<([u8; 32], [u8; 64])> {
    if value.len() != 193 {
        return None;
    }
    let (key, sig) = value.split_once('.')?;
    Some((unhex_exact::<32>(key)?, unhex_exact::<64>(sig)?))
}

/// The path segment naming the pair of keys `a` and `b`: both in lowercase
/// hex, the lower key first, joined by `-`. Both devices compute the same.
pub fn pair_name(a: &[u8; 32], b: &[u8; 32]) -> String {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    format!("{}-{}", hex_bytes(lo), hex_bytes(hi))
}

/// The two keys of a pair name, if it is one.
pub fn parse_pair_name(s: &str) -> Option<([u8; 32], [u8; 32])> {
    let (a, b) = s.split_once('-')?;
    let (a, b) = (unhex_exact::<32>(a)?, unhex_exact::<32>(b)?);
    (a < b).then_some((a, b))
}

/// Collects datagrams for one key into messages of at most
/// [`MAX_MESSAGE`] bytes.
#[derive(Debug)]
pub struct Batch {
    message: Vec<u8>,
}

impl Batch {
    /// An empty message for `key`.
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            message: key.to_vec(),
        }
    }

    /// Whether nothing has been added yet.
    pub fn is_empty(&self) -> bool {
        self.message.len() <= 32
    }

    /// Add `datagram` if it fits; `false` when the message is full (or the
    /// datagram is empty or too large, in which case it should be dropped).
    pub fn push(&mut self, datagram: &[u8]) -> bool {
        let Ok(len) = u16::try_from(datagram.len()) else {
            return false;
        };
        if datagram.is_empty() || datagram.len() > MAX_DATAGRAM {
            return false;
        }
        let after = self
            .message
            .len()
            .saturating_add(2)
            .saturating_add(datagram.len());
        if after > MAX_MESSAGE {
            return false;
        }
        self.message.extend_from_slice(&len.to_be_bytes());
        self.message.extend_from_slice(datagram);
        true
    }

    /// The finished message.
    pub fn finish(self) -> Vec<u8> {
        self.message
    }
}

/// A datagram message with one datagram: `key` ‖ length ‖ `datagram`.
pub fn frame(key: &[u8; 32], datagram: &[u8]) -> Vec<u8> {
    let mut b = Batch::new(key);
    b.push(datagram);
    b.finish()
}

/// Split a datagram message into its key and datagrams. A malformed message
/// (no datagram, an empty or oversized one, a length running past the end)
/// is refused as a whole.
pub fn parse_frame(m: &[u8]) -> Option<([u8; 32], Vec<&[u8]>)> {
    if m.len() > MAX_MESSAGE {
        return None;
    }
    let (key, mut rest) = m.split_at_checked(32)?;
    let mut datagrams = Vec::new();
    while !rest.is_empty() {
        let (len, after) = rest.split_at_checked(2)?;
        let len = usize::from(u16::from_be_bytes(len.try_into().ok()?));
        if len == 0 || len > MAX_DATAGRAM {
            return None;
        }
        let (datagram, after) = after.split_at_checked(len)?;
        datagrams.push(datagram);
        rest = after;
    }
    if datagrams.is_empty() {
        return None;
    }
    Some((key.try_into().ok()?, datagrams))
}

/// The STUN magic cookie (RFC 8489).
const COOKIE: [u8; 4] = [0x21, 0x12, 0xa4, 0x42];
/// STUN message types.
const BINDING_REQUEST: [u8; 2] = [0x00, 0x01];
const BINDING_SUCCESS: [u8; 2] = [0x01, 0x01];
/// STUN attributes.
const MAPPED_ADDRESS: u16 = 0x0001;
const XOR_MAPPED_ADDRESS: u16 = 0x0020;

/// Default STUN server for address discovery.
pub const DEFAULT_STUN: &str = "stun.cloudflare.com:3478";

/// A STUN Binding request with transaction ID `id`.
pub fn stun_request(id: &[u8; 12]) -> [u8; 20] {
    let mut m = [0u8; 20];
    for (dst, src) in m.iter_mut().zip(
        BINDING_REQUEST
            .iter()
            .chain([0u8, 0].iter())
            .chain(COOKIE.iter())
            .chain(id.iter()),
    ) {
        *dst = *src;
    }
    m
}

/// The transaction ID of a Binding request, for a server answering it.
/// Requests carrying attributes are accepted; the attributes are ignored.
pub fn parse_stun_request(m: &[u8]) -> Option<[u8; 12]> {
    let header = m.get(..20)?;
    if header.get(..2)? != BINDING_REQUEST || header.get(4..8)? != COOKIE {
        return None;
    }
    let len = usize::from(u16::from_be_bytes(header.get(2..4)?.try_into().ok()?));
    if m.len() != 20usize.checked_add(len)? || len % 4 != 0 {
        return None;
    }
    header.get(8..20)?.try_into().ok()
}

fn xor_address(seen: SocketAddr, id: &[u8; 12]) -> Vec<u8> {
    let mut v = vec![0u8];
    let port = seen.port() ^ 0x2112;
    let ip = match seen.ip() {
        IpAddr::V6(ip) => ip
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(ip)),
        v4 => v4,
    };
    match ip {
        IpAddr::V4(ip) => {
            v.push(0x01);
            v.extend_from_slice(&port.to_be_bytes());
            v.extend(ip.octets().iter().zip(COOKIE.iter()).map(|(a, b)| a ^ b));
        }
        IpAddr::V6(ip) => {
            v.push(0x02);
            v.extend_from_slice(&port.to_be_bytes());
            let mask = COOKIE.iter().chain(id.iter());
            v.extend(ip.octets().iter().zip(mask).map(|(a, b)| a ^ b));
        }
    }
    v
}

/// A Binding success response telling the requester it was seen at `seen`.
pub fn stun_response(id: &[u8; 12], seen: SocketAddr) -> Vec<u8> {
    let value = xor_address(seen, id);
    let mut m = BINDING_SUCCESS.to_vec();
    let body = u16::try_from(value.len().saturating_add(4)).unwrap_or(0);
    m.extend_from_slice(&body.to_be_bytes());
    m.extend_from_slice(&COOKIE);
    m.extend_from_slice(id);
    m.extend_from_slice(&XOR_MAPPED_ADDRESS.to_be_bytes());
    m.extend_from_slice(&u16::try_from(value.len()).unwrap_or(0).to_be_bytes());
    m.extend_from_slice(&value);
    m
}

fn read_address(value: &[u8], xor: bool, id: &[u8; 12]) -> Option<SocketAddr> {
    let family = *value.get(1)?;
    let mut port = u16::from_be_bytes(value.get(2..4)?.try_into().ok()?);
    if xor {
        port ^= 0x2112;
    }
    let mask: Vec<u8> = COOKIE.iter().chain(id.iter()).copied().collect();
    let ip: IpAddr = match family {
        0x01 => {
            let raw: [u8; 4] = value.get(4..8)?.try_into().ok()?;
            let mut b = raw;
            if xor {
                for (x, m) in b.iter_mut().zip(mask.iter()) {
                    *x ^= m;
                }
            }
            Ipv4Addr::from(b).into()
        }
        0x02 => {
            let raw: [u8; 16] = value.get(4..20)?.try_into().ok()?;
            let mut b = raw;
            if xor {
                for (x, m) in b.iter_mut().zip(mask.iter()) {
                    *x ^= m;
                }
            }
            Ipv6Addr::from(b).into()
        }
        _ => return None,
    };
    Some(SocketAddr::new(ip, port))
}

/// The address in a Binding success response to the request with `id`.
/// XOR-MAPPED-ADDRESS is preferred; MAPPED-ADDRESS is read when it is the
/// only one.
pub fn parse_stun_response(m: &[u8], id: &[u8; 12]) -> Option<SocketAddr> {
    let header = m.get(..20)?;
    if header.get(..2)? != BINDING_SUCCESS
        || header.get(4..8)? != COOKIE
        || header.get(8..20)? != id
    {
        return None;
    }
    let len = usize::from(u16::from_be_bytes(header.get(2..4)?.try_into().ok()?));
    let mut attrs = m.get(20..20usize.checked_add(len)?)?;
    let mut plain = None;
    while attrs.len() >= 4 {
        let kind = u16::from_be_bytes(attrs.get(..2)?.try_into().ok()?);
        let size = usize::from(u16::from_be_bytes(attrs.get(2..4)?.try_into().ok()?));
        let value = attrs.get(4..4usize.checked_add(size)?)?;
        match kind {
            XOR_MAPPED_ADDRESS => return read_address(value, true, id),
            MAPPED_ADDRESS => plain = read_address(value, false, id),
            _ => {}
        }
        // Attributes are padded to four bytes.
        let padded = size.checked_add(3)?.checked_div(4)?.checked_mul(4)?;
        attrs = attrs.get(4usize.checked_add(padded)?.min(attrs.len())..)?;
    }
    plain
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing)]

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

    /// A probe has a fixed layout and never reads as a datagram message,
    /// so devices from before probes drop it (#190).
    #[test]
    fn probes_are_fixed_and_never_datagram_messages() {
        let p = probe(&[3u8; 16]);
        assert_eq!(&p[..9], b"TDR0probe");
        assert_eq!(p.len(), 25);
        assert!(is_probe(&p));
        assert_eq!(parse_frame(&p), None);
        assert!(!is_probe(&p[..24]));
        assert!(!is_probe(&[p.as_slice(), &[0]].concat()));
        assert!(!is_probe(&ok()));
        assert!(!is_probe(&challenge(&[0u8; 32])));
        assert!(!is_probe(&[b"TDR0probX".as_slice(), &[0; 16]].concat()));
        assert!(!is_probe(&[b"XDR0probe".as_slice(), &[0; 16]].concat()));
        assert!(!is_probe(&frame(&[0u8; 32], b"TDR0probe")));
    }

    #[test]
    fn pair_names_are_symmetric_and_strict() {
        let (a, b) = ([1u8; 32], [0xabu8; 32]);
        let n = pair_name(&a, &b);
        assert_eq!(n, pair_name(&b, &a));
        assert_eq!(parse_pair_name(&n), Some((a, b)));
        assert_eq!(parse_pair_name(&pair_name(&a, &a)), None);
        assert_eq!(parse_pair_name(&n.to_uppercase()), None);
        assert_eq!(parse_pair_name("x-y"), None);
    }

    #[test]
    fn auth_header_round_trip_and_strict_parsing() {
        let key: [u8; 32] = core::array::from_fn(|i| i as u8);
        let sig: [u8; 64] = core::array::from_fn(|i| 0xf0u8.wrapping_add(i as u8));
        let formatted = format_auth_header(&key, &sig);
        assert_eq!(formatted.len(), 193);
        assert_eq!(AUTH_HEADER, "X-TotalDrop-Auth");
        assert_eq!(parse_auth_header(&formatted), Some((key, sig)));

        // Exact signed bytes layout and domain separation from AUTH_CONTEXT.
        let pair = pair_name(&[0x01; 32], &[0x02; 32]);
        let signed = connect_signed_bytes(&pair);
        let mut expected = b"totaldrop relay v0 connect".to_vec();
        expected.extend_from_slice(pair.as_bytes());
        assert_eq!(signed, expected);
        assert_ne!(CONNECT_CONTEXT, AUTH_CONTEXT);

        // Wrong lengths and empty.
        assert_eq!(parse_auth_header(""), None);
        assert_eq!(parse_auth_header(&formatted[..192]), None);
        assert_eq!(parse_auth_header(&format!("{formatted}0")), None);

        // Uppercase hex in key or signature.
        let (k_hex, s_hex) = formatted.split_once('.').unwrap();
        assert_eq!(
            parse_auth_header(&format!("{}.{s_hex}", k_hex.to_uppercase())),
            None
        );
        assert_eq!(
            parse_auth_header(&format!("{k_hex}.{}", s_hex.to_uppercase())),
            None
        );

        // Whitespace and comma-joined duplicate headers.
        assert_eq!(parse_auth_header(&format!(" {formatted}")), None);
        assert_eq!(parse_auth_header(&format!("{formatted} ")), None);
        assert_eq!(
            parse_auth_header(&format!("{formatted}, {formatted}")),
            None
        );

        // Missing dot, wrong dot offset, extra dot.
        assert_eq!(parse_auth_header(&format!("{k_hex}0{s_hex}")), None);
        assert_eq!(
            parse_auth_header(&format!("{}.0{s_hex}", &k_hex[..63])),
            None
        );
        assert_eq!(
            parse_auth_header(&format!("{k_hex}0.{}", &s_hex[..127])),
            None
        );
        assert_eq!(
            parse_auth_header(&format!("{k_hex}.{}.{}", &s_hex[..63], &s_hex[64..])),
            None
        );

        // Non-hex ASCII and non-ASCII UTF-8 (same 193-byte length).
        assert_eq!(
            parse_auth_header(&format!("g{}.{s_hex}", &k_hex[1..])),
            None
        );
        assert_eq!(
            parse_auth_header(&format!("{k_hex}.z{}", &s_hex[1..])),
            None
        );
        // 'é' is 2 UTF-8 bytes; replacing 2 ASCII bytes preserves byte length 193.
        assert_eq!(
            parse_auth_header(&format!("é{}.{s_hex}", &k_hex[2..])),
            None
        );
    }

    #[test]
    fn batches() {
        let key = [9u8; 32];
        let mut b = Batch::new(&key);
        assert!(b.is_empty());
        assert!(b.push(b"one"));
        assert!(b.push(b"two!"));
        assert!(!b.push(b""));
        assert!(!b.push(&vec![0u8; MAX_DATAGRAM + 1]));
        let m = b.finish();
        let (k, d) = parse_frame(&m).unwrap();
        assert_eq!(k, key);
        assert_eq!(d, vec![b"one".as_slice(), b"two!".as_slice()]);
        // A full message refuses more.
        let mut b = Batch::new(&key);
        let big = vec![1u8; MAX_DATAGRAM];
        let mut n = 0;
        while b.push(&big) {
            n += 1;
        }
        assert_eq!(n, (MAX_MESSAGE - 32) / (MAX_DATAGRAM + 2));
        // Malformed messages.
        assert_eq!(parse_frame(&key), None);
        let mut cut = frame(&key, b"hello");
        cut.pop();
        assert_eq!(parse_frame(&cut), None);
        let mut zero = key.to_vec();
        zero.extend_from_slice(&[0, 0]);
        assert_eq!(parse_frame(&zero), None);
    }

    #[test]
    fn stun_round_trip() {
        let id = [3u8; 12];
        let req = stun_request(&id);
        assert_eq!(parse_stun_request(&req), Some(id));
        assert_eq!(parse_stun_request(&req[..19]), None);
        for seen in [
            "203.0.113.5:40000".parse().unwrap(),
            "[2001:db8::1]:443".parse().unwrap(),
            "[::ffff:198.51.100.7]:9".parse::<SocketAddr>().unwrap(),
        ] {
            let r = stun_response(&id, seen);
            let back = parse_stun_response(&r, &id).unwrap();
            assert_eq!(back.port(), seen.port());
        }
        let r = stun_response(&id, "203.0.113.5:1".parse().unwrap());
        assert_eq!(
            parse_stun_response(&r, &id),
            Some("203.0.113.5:1".parse().unwrap())
        );
        assert_eq!(parse_stun_response(&r, &[4u8; 12]), None);
    }

    #[test]
    fn stun_answer_from_rfc_5769() {
        // RFC 5769 section 2.2: an IPv4 response, 192.0.2.1:32853, with a
        // SOFTWARE attribute before XOR-MAPPED-ADDRESS (integrity ignored).
        let msg: [u8; 80] = [
            0x01, 0x01, 0x00, 0x3c, 0x21, 0x12, 0xa4, 0x42, 0xb7, 0xe7, 0xa7, 0x01, 0xbc, 0x34,
            0xd6, 0x86, 0xfa, 0x87, 0xdf, 0xae, 0x80, 0x22, 0x00, 0x0b, 0x74, 0x65, 0x73, 0x74,
            0x20, 0x76, 0x65, 0x63, 0x74, 0x6f, 0x72, 0x20, 0x00, 0x20, 0x00, 0x08, 0x00, 0x01,
            0xa1, 0x47, 0xe1, 0x12, 0xa6, 0x43, 0x00, 0x08, 0x00, 0x14, 0x2b, 0x91, 0xf5, 0x99,
            0xfd, 0x9e, 0x90, 0xc3, 0x8c, 0x74, 0x89, 0xf9, 0x2a, 0xf9, 0xba, 0x53, 0xf0, 0x6b,
            0xe7, 0xd7, 0x80, 0x28, 0x00, 0x04, 0xc0, 0x7d, 0x4c, 0x96,
        ];
        let id: [u8; 12] = msg[8..20].try_into().unwrap();
        assert_eq!(
            parse_stun_response(&msg, &id),
            Some("192.0.2.1:32853".parse().unwrap())
        );
    }
}
