// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Messages for asking the home router to forward a UDP port: PCP
//! ([RFC 6887](https://www.rfc-editor.org/rfc/rfc6887)) and its
//! predecessor NAT-PMP ([RFC 6886](https://www.rfc-editor.org/rfc/rfc6886)).
//!
//! Only what a client needs for one UDP mapping is here: the request, its
//! release (the same request with lifetime 0) and the router's answer.
//! Parsers take any bytes from the network and refuse what does not
//! answer the request we sent.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// The port a PCP or NAT-PMP server listens on, on the gateway.
pub const SERVER_PORT: u16 = 5351;

/// IANA protocol number of UDP, the only protocol we map.
const UDP: u8 = 17;

/// PCP version 2.
const PCP_VERSION: u8 = 2;
/// PCP MAP opcode; the answer sets the top bit.
const PCP_MAP: u8 = 1;
const PCP_RESPONSE: u8 = 0x80;
/// Size of a MAP request or answer without options.
pub const PCP_MAP_LEN: usize = 60;
/// Largest PCP message (RFC 6887, section 7).
pub const PCP_MAX_LEN: usize = 1100;
/// Result code of a server that speaks another version.
const PCP_UNSUPP_VERSION: u8 = 1;

/// NAT-PMP version 0.
const PMP_VERSION: u8 = 0;
/// NAT-PMP opcodes: public address and UDP mapping; answers add 128.
const PMP_ADDRESS: u8 = 0;
const PMP_MAP_UDP: u8 = 1;
const PMP_RESPONSE: u8 = 128;

/// What the router said to a mapping request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// The mapping holds: the router forwards `external` to our port for
    /// `lifetime` seconds. A lifetime of 0 confirms a release.
    Mapped {
        /// The router's public address and the port it opened.
        external: (IpAddr, u16),
        /// Seconds the mapping lasts unless renewed.
        lifetime: u32,
    },
    /// The router refused with this result code (it may not allow mapping,
    /// may lack resources, or the port may be taken).
    Refused(u16),
    /// A PCP request reached a server that only speaks NAT-PMP, or the
    /// other way round; try the other protocol.
    OtherVersion,
}

/// The address in a PCP message: IPv4 is written as an IPv4-mapped IPv6
/// address (RFC 6887, section 5).
fn pcp_address(ip: IpAddr) -> [u8; 16] {
    match ip {
        IpAddr::V4(v4) => v4.to_ipv6_mapped().octets(),
        IpAddr::V6(v6) => v6.octets(),
    }
}

fn read_pcp_address(b: [u8; 16]) -> IpAddr {
    IpAddr::V6(Ipv6Addr::from(b)).to_canonical()
}

/// A PCP MAP request for UDP on `internal_port`, sent from `client` (the
/// source address the gateway will see). Lifetime 0 releases the mapping
/// made with the same `nonce`.
pub fn pcp_map(
    nonce: &[u8; 12],
    client: IpAddr,
    internal_port: u16,
    lifetime: u32,
) -> [u8; PCP_MAP_LEN] {
    let mut m = [0u8; PCP_MAP_LEN];
    let mut w = Writer(&mut m, 0);
    w.put(&[PCP_VERSION, PCP_MAP, 0, 0]);
    w.put(&lifetime.to_be_bytes());
    w.put(&pcp_address(client));
    w.put(nonce);
    w.put(&[UDP, 0, 0, 0]);
    w.put(&internal_port.to_be_bytes());
    // No suggested external port or address: the router picks. "No
    // address" is the all-zeros address of the client's own family
    // (RFC 6887, section 11.1), `::ffff:0.0.0.0` for IPv4.
    w.put(&0u16.to_be_bytes());
    let any = match client {
        IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        IpAddr::V6(_) => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
    };
    w.put(&pcp_address(any));
    m
}

/// The router's answer to [`pcp_map`] with `nonce` and `internal_port`.
/// Anything else, including an answer to another request, is `None`.
pub fn parse_pcp_map(m: &[u8], nonce: &[u8; 12], internal_port: u16) -> Option<Answer> {
    if m.len() < 4 || m.len() > PCP_MAX_LEN || !m.len().is_multiple_of(4) {
        return None;
    }
    // A NAT-PMP server answers a version it does not know with its own
    // version and result code 1 (RFC 6886, section 3.5).
    if m.first() == Some(&PMP_VERSION) {
        return (m.get(1)? & PMP_RESPONSE != 0 && be16(m, 2)? == 1).then_some(Answer::OtherVersion);
    }
    if *m.first()? != PCP_VERSION || *m.get(1)? != PCP_RESPONSE | PCP_MAP {
        // An unsupported-version answer may carry only the header.
        return None;
    }
    let result = *m.get(3)?;
    if result == PCP_UNSUPP_VERSION {
        return Some(Answer::OtherVersion);
    }
    if m.len() < PCP_MAP_LEN
        || m.get(24..36)? != nonce
        || *m.get(36)? != UDP
        || be16(m, 40)? != internal_port
    {
        return None;
    }
    if result != 0 {
        return Some(Answer::Refused(u16::from(result)));
    }
    let lifetime = be32(m, 4)?;
    let port = be16(m, 42)?;
    let ip = read_pcp_address(m.get(44..60)?.try_into().ok()?);
    if lifetime != 0 && (port == 0 || ip.is_unspecified()) {
        return None;
    }
    Some(Answer::Mapped {
        external: (ip, port),
        lifetime,
    })
}

/// The NAT-PMP request for the router's public address.
pub fn pmp_address() -> [u8; 2] {
    [PMP_VERSION, PMP_ADDRESS]
}

/// The router's public address from the answer to [`pmp_address`]:
/// `Ok` with the address, or `Err` with the result code.
pub fn parse_pmp_address(m: &[u8]) -> Option<Result<Ipv4Addr, u16>> {
    if *m.first()? != PMP_VERSION || *m.get(1)? != PMP_RESPONSE | PMP_ADDRESS {
        return None;
    }
    let result = be16(m, 2)?;
    if result != 0 {
        return Some(Err(result));
    }
    if m.len() != 12 {
        return None;
    }
    let ip: [u8; 4] = m.get(8..12)?.try_into().ok()?;
    let ip = Ipv4Addr::from(ip);
    (!ip.is_unspecified()).then_some(Ok(ip))
}

/// A NAT-PMP request to map UDP on `internal_port`. Lifetime 0 releases it.
pub fn pmp_map(internal_port: u16, lifetime: u32) -> [u8; 12] {
    let mut m = [0u8; 12];
    let mut w = Writer(&mut m, 0);
    w.put(&[PMP_VERSION, PMP_MAP_UDP, 0, 0]);
    w.put(&internal_port.to_be_bytes());
    // No suggested external port; a release must send 0 here too.
    w.put(&0u16.to_be_bytes());
    w.put(&lifetime.to_be_bytes());
    m
}

/// The answer to [`pmp_map`] for `internal_port`: `Ok` with the external
/// port and lifetime, or `Err` with the result code. NAT-PMP gives the
/// public address only through [`pmp_address`].
pub fn parse_pmp_map(m: &[u8], internal_port: u16) -> Option<Result<(u16, u32), u16>> {
    if *m.first()? != PMP_VERSION || *m.get(1)? != PMP_RESPONSE | PMP_MAP_UDP {
        return None;
    }
    let result = be16(m, 2)?;
    // Answers, refusals too, have all 16 bytes (RFC 6886, section 3.5)
    // and name our internal port; anything else is someone else's.
    if m.len() != 16 || be16(m, 8)? != internal_port {
        return None;
    }
    if result != 0 {
        return Some(Err(result));
    }
    let port = be16(m, 10)?;
    let lifetime = be32(m, 12)?;
    if lifetime != 0 && port == 0 {
        return None;
    }
    Some(Ok((port, lifetime)))
}

fn be16(m: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        m.get(at..at.checked_add(2)?)?.try_into().ok()?,
    ))
}

fn be32(m: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        m.get(at..at.checked_add(4)?)?.try_into().ok()?,
    ))
}

/// Writes into a buffer whose size the caller fixed for exactly these bytes.
struct Writer<'a>(&'a mut [u8], usize);

impl Writer<'_> {
    fn put(&mut self, bytes: &[u8]) {
        let end = self.1.saturating_add(bytes.len());
        if let Some(dst) = self.0.get_mut(self.1..end) {
            dst.copy_from_slice(bytes);
        }
        self.1 = end;
    }
}

/// Answers as a router would send them, for tests of clients. Not part of
/// the protocol API.
#[doc(hidden)]
pub mod answer {
    use super::*;

    /// A PCP MAP answer to a request (`req` as built by [`pcp_map`]).
    pub fn pcp(
        req: &[u8; PCP_MAP_LEN],
        result: u8,
        external: (IpAddr, u16),
        lifetime: u32,
    ) -> [u8; PCP_MAP_LEN] {
        let mut m = [0u8; PCP_MAP_LEN];
        let mut w = Writer(&mut m, 0);
        w.put(&[PCP_VERSION, PCP_RESPONSE | PCP_MAP, 0, result]);
        w.put(&lifetime.to_be_bytes());
        w.put(&1u32.to_be_bytes());
        w.put(&[0u8; 12]);
        // Nonce, protocol and internal port as in the request.
        w.put(req.get(24..42).unwrap_or(&[0u8; 18]));
        w.put(&external.1.to_be_bytes());
        w.put(&pcp_address(external.0));
        m
    }

    /// A NAT-PMP public address answer.
    pub fn pmp_address(result: u16, ip: Ipv4Addr) -> [u8; 12] {
        let mut m = [0u8; 12];
        let mut w = Writer(&mut m, 0);
        w.put(&[PMP_VERSION, PMP_RESPONSE | PMP_ADDRESS]);
        w.put(&result.to_be_bytes());
        w.put(&1u32.to_be_bytes());
        w.put(&ip.octets());
        m
    }

    /// A NAT-PMP mapping answer.
    pub fn pmp_map(result: u16, internal_port: u16, external_port: u16, lifetime: u32) -> [u8; 16] {
        let mut m = [0u8; 16];
        let mut w = Writer(&mut m, 0);
        w.put(&[PMP_VERSION, PMP_RESPONSE | PMP_MAP_UDP]);
        w.put(&result.to_be_bytes());
        w.put(&1u32.to_be_bytes());
        w.put(&internal_port.to_be_bytes());
        w.put(&external_port.to_be_bytes());
        w.put(&lifetime.to_be_bytes());
        m
    }

    /// What a NAT-PMP server sends back to a PCP request.
    pub fn pmp_unsupported_version() -> [u8; 8] {
        let mut m = [0u8; 8];
        let mut w = Writer(&mut m, 0);
        w.put(&[PMP_VERSION, PMP_RESPONSE | PCP_MAP]);
        w.put(&1u16.to_be_bytes());
        m
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing)]

    use super::*;

    /// A fresh random nonce per run, as `core` draws one per mapping.
    static NONCE: std::sync::LazyLock<[u8; 12]> = std::sync::LazyLock::new(fresh_nonce);

    fn fresh_nonce() -> [u8; 12] {
        let mut n = [0u8; 12];
        getrandom::fill(&mut n).unwrap();
        n
    }
    const PUBLIC: Ipv4Addr = Ipv4Addr::new(203, 0, 113, 7);

    fn client() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20))
    }

    /// The request layout from RFC 6887, sections 7.1 and 11.1.
    #[test]
    fn pcp_request_layout() {
        let m = pcp_map(&NONCE, client(), 40_000, 120);
        assert_eq!(&m[..4], &[2, 1, 0, 0]);
        assert_eq!(&m[4..8], &120u32.to_be_bytes());
        assert_eq!(
            &m[8..24],
            &Ipv4Addr::new(192, 168, 1, 20).to_ipv6_mapped().octets()
        );
        assert_eq!(&m[24..36], &*NONCE);
        assert_eq!(&m[36..40], &[17, 0, 0, 0]);
        assert_eq!(&m[40..42], &40_000u16.to_be_bytes());
        assert_eq!(&m[42..44], &[0, 0]);
        assert_eq!(&m[44..60], &Ipv4Addr::UNSPECIFIED.to_ipv6_mapped().octets());
        let v6 = pcp_map(&NONCE, "2001:db8::2".parse().unwrap(), 40_000, 120);
        assert_eq!(&v6[44..60], &[0u8; 16]);
    }

    #[test]
    fn pcp_answer_is_read_and_matched() {
        let req = pcp_map(&NONCE, client(), 40_000, 120);
        let ok = answer::pcp(&req, 0, (IpAddr::V4(PUBLIC), 51_000), 120);
        assert_eq!(
            parse_pcp_map(&ok, &NONCE, 40_000),
            Some(Answer::Mapped {
                external: (IpAddr::V4(PUBLIC), 51_000),
                lifetime: 120
            })
        );
        // Another request's nonce or port, or our own request echoed back.
        assert_eq!(parse_pcp_map(&ok, &fresh_nonce(), 40_000), None);
        assert_eq!(parse_pcp_map(&ok, &NONCE, 40_001), None);
        assert_eq!(parse_pcp_map(&req, &NONCE, 40_000), None);
        // A refusal (NOT_AUTHORIZED is 2) and a release confirmation.
        let no = answer::pcp(&req, 2, (IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0), 0);
        assert_eq!(parse_pcp_map(&no, &NONCE, 40_000), Some(Answer::Refused(2)));
        let released = answer::pcp(&req, 0, (IpAddr::V4(PUBLIC), 51_000), 0);
        assert!(matches!(
            parse_pcp_map(&released, &NONCE, 40_000),
            Some(Answer::Mapped { lifetime: 0, .. })
        ));
        // A success that names no port or address is not a mapping.
        let empty = answer::pcp(&req, 0, (IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0), 120);
        assert_eq!(parse_pcp_map(&empty, &NONCE, 40_000), None);
    }

    #[test]
    fn pcp_options_after_the_answer_are_ignored() {
        let req = pcp_map(&NONCE, client(), 40_000, 120);
        let mut m = answer::pcp(&req, 0, (IpAddr::V4(PUBLIC), 51_000), 120).to_vec();
        m.extend_from_slice(&[0x7f, 0, 0, 4, 1, 2, 3, 4]);
        assert!(matches!(
            parse_pcp_map(&m, &NONCE, 40_000),
            Some(Answer::Mapped { .. })
        ));
        m.push(0);
        assert_eq!(
            parse_pcp_map(&m, &NONCE, 40_000),
            None,
            "not a multiple of four"
        );
        assert_eq!(
            parse_pcp_map(&vec![0u8; PCP_MAX_LEN + 4], &NONCE, 40_000),
            None
        );
    }

    #[test]
    fn other_version_is_recognised() {
        assert_eq!(
            parse_pcp_map(&answer::pmp_unsupported_version(), &NONCE, 40_000),
            Some(Answer::OtherVersion)
        );
        let mut pcp_only = [0u8; 24];
        pcp_only[0] = 2;
        pcp_only[1] = 0x81;
        pcp_only[3] = 1;
        assert_eq!(
            parse_pcp_map(&pcp_only, &NONCE, 40_000),
            Some(Answer::OtherVersion)
        );
    }

    /// The request layouts from RFC 6886, sections 3.2 and 3.3.
    #[test]
    fn pmp_request_layout() {
        assert_eq!(pmp_address(), [0, 0]);
        let m = pmp_map(40_000, 120);
        assert_eq!(&m[..4], &[0, 1, 0, 0]);
        assert_eq!(&m[4..6], &40_000u16.to_be_bytes());
        assert_eq!(&m[6..8], &[0, 0]);
        assert_eq!(&m[8..12], &120u32.to_be_bytes());
        assert_eq!(&pmp_map(40_000, 0)[8..12], &[0; 4]);
    }

    #[test]
    fn pmp_answers_are_read_and_matched() {
        assert_eq!(
            parse_pmp_address(&answer::pmp_address(0, PUBLIC)),
            Some(Ok(PUBLIC))
        );
        assert_eq!(
            parse_pmp_address(&answer::pmp_address(3, PUBLIC)),
            Some(Err(3))
        );
        assert_eq!(
            parse_pmp_address(&answer::pmp_address(0, Ipv4Addr::UNSPECIFIED)),
            None
        );
        assert_eq!(parse_pmp_address(&pmp_address()), None, "our own request");

        assert_eq!(
            parse_pmp_map(&answer::pmp_map(0, 40_000, 51_000, 120), 40_000),
            Some(Ok((51_000, 120)))
        );
        assert_eq!(
            parse_pmp_map(&answer::pmp_map(0, 40_001, 51_000, 120), 40_000),
            None
        );
        assert_eq!(
            parse_pmp_map(&answer::pmp_map(2, 40_000, 0, 0), 40_000),
            Some(Err(2))
        );
        assert_eq!(
            parse_pmp_map(&answer::pmp_map(0, 40_000, 0, 120), 40_000),
            None
        );
        assert_eq!(
            parse_pmp_map(&answer::pmp_map(0, 40_000, 0, 0), 40_000),
            Some(Ok((0, 0))),
            "released"
        );
        assert_eq!(
            parse_pmp_map(&pmp_map(40_000, 120), 40_000),
            None,
            "our own request"
        );
        // A refusal must be whole and name our port.
        assert_eq!(
            parse_pmp_map(&answer::pmp_map(5, 40_000, 0, 0), 40_000),
            Some(Err(5))
        );
        assert_eq!(parse_pmp_map(&[0, 129, 0, 5], 40_000), None);
        assert_eq!(
            parse_pmp_map(&answer::pmp_map(5, 40_001, 0, 0), 40_000),
            None
        );
    }

    #[test]
    fn short_and_empty_input_is_refused() {
        for n in 0..PCP_MAP_LEN {
            let m = vec![0u8; n];
            let _ = parse_pcp_map(&m, &NONCE, 40_000);
            let _ = parse_pmp_map(&m, 40_000);
            let _ = parse_pmp_address(&m);
        }
        assert_eq!(parse_pcp_map(&[], &NONCE, 1), None);
        assert_eq!(parse_pmp_map(&[], 1), None);
        assert_eq!(parse_pmp_address(&[]), None);
    }
}
