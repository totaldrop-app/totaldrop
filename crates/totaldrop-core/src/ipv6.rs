// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Temporary IPv6 source address preference (RFC 8981) on Linux (#405).
//!
//! Android, iOS, macOS and Windows already prefer temporary IPv6 addresses by
//! default. Linux prefers them only when `use_tempaddr = 2`; otherwise the
//! kernel selects a stable-privacy address (RFC 7217) or, on older
//! configurations, a hardware-derived EUI-64 address. When a usable
//! non-deprecated temporary address (`IFA_F_TEMPORARY`) exists on the same
//! interface and prefix as the system's chosen source, this module selects it
//! for `HINTS`, STUN probes, and outbound QUIC datagrams (via `IPV6_PKTINFO`).

use std::net::{IpAddr, Ipv6Addr, SocketAddr};

/// Flags from Linux `<linux/if_addr.h>` as printed in `/proc/net/if_inet6`.
const IFA_F_TEMPORARY: u32 = 0x01;
const IFA_F_DADFAILED: u32 = 0x08;
const IFA_F_DEPRECATED: u32 = 0x20;
const IFA_F_TENTATIVE: u32 = 0x40;

/// One row of Linux `/proc/net/if_inet6`.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InterfaceV6 {
    pub(crate) addr: Ipv6Addr,
    pub(crate) ifindex: u32,
    pub(crate) prefix_len: u8,
    pub(crate) scope: u8,
    pub(crate) flags: u32,
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn parse_hex_u8(s: &str) -> Option<u8> {
    s.bytes()
        .all(|b| b.is_ascii_hexdigit())
        .then(|| u8::from_str_radix(s, 16).ok())
        .flatten()
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn parse_hex_u32(s: &str) -> Option<u32> {
    s.bytes()
        .all(|b| b.is_ascii_hexdigit())
        .then(|| u32::from_str_radix(s, 16).ok())
        .flatten()
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn parse_ipv6_hex(hex: &str) -> Option<Ipv6Addr> {
    if hex.len() != 32 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let mut octets = [0u8; 16];
    for (i, slot) in octets.iter_mut().enumerate() {
        let at = i.checked_mul(2)?;
        let end = at.checked_add(2)?;
        let pair = hex.get(at..end)?;
        *slot = u8::from_str_radix(pair, 16).ok()?;
    }
    Some(Ipv6Addr::from(octets))
}

/// Parse Linux's `/proc/net/if_inet6`.
///
/// Unlike `/proc/net/route`, `/proc/net/if_inet6` has no header line; every
/// line carries six whitespace-separated fields:
/// `<32-hex IPv6> <ifindex-hex> <prefix_len-hex> <scope-hex> <flags-hex> <ifname>`.
/// Blank and malformed lines are ignored.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) fn parse_if_inet6(input: &str) -> Vec<InterfaceV6> {
    let mut out = Vec::new();
    for line in input.lines() {
        let mut parts = line.split_whitespace();
        let (
            Some(addr_hex),
            Some(ifindex_hex),
            Some(prefix_hex),
            Some(scope_hex),
            Some(flags_hex),
            Some(_ifname),
            None,
        ) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        )
        else {
            continue;
        };
        let (Some(addr), Some(ifindex), Some(prefix_len), Some(scope), Some(flags)) = (
            parse_ipv6_hex(addr_hex),
            parse_hex_u32(ifindex_hex),
            parse_hex_u8(prefix_hex),
            parse_hex_u8(scope_hex),
            parse_hex_u32(flags_hex),
        ) else {
            continue;
        };
        if prefix_len > 128 {
            continue;
        }
        out.push(InterfaceV6 {
            addr,
            ifindex,
            prefix_len,
            scope,
            flags,
        });
    }
    out
}

/// Whether `ip` is a Unique Local Address (`fc00::/7`, RFC 4193).
fn is_ula(ip: Ipv6Addr) -> bool {
    ip.segments().first().is_some_and(|&s| s & 0xfe00 == 0xfc00)
}

/// Whether `e` is a usable non-deprecated temporary IPv6 address that this
/// device may announce and send from.
fn is_usable_temp(e: &InterfaceV6) -> bool {
    const UNUSABLE: u32 = IFA_F_DEPRECATED | IFA_F_TENTATIVE | IFA_F_DADFAILED;
    let ipv4_compat = !e.addr.is_loopback() && e.addr.segments().get(..6) == Some(&[0; 6]);
    (e.flags & IFA_F_TEMPORARY) != 0
        && (e.flags & UNUSABLE) == 0
        && crate::transport::announceable_v6(e.addr)
        && !e.addr.is_multicast()
        && !ipv4_compat
}

/// Whether two IPv6 addresses share their top 64 bits.
fn same_prefix64(a: Ipv6Addr, b: Ipv6Addr) -> bool {
    a.octets().get(..8) == b.octets().get(..8)
}

/// Pick a usable temporary IPv6 address matching the interface and scope of
/// `sys_src` from parsed `/proc/net/if_inet6` entries.
///
/// Honors `sys_src` immediately when it is already a usable temporary address
/// (`use_tempaddr = 2`). Otherwise prefers a temporary address on the same
/// interface, scope, and `/64` prefix; for global-scope routes (`scope == 0`)
/// falls back to another temporary address on the same interface with matching
/// ULA vs. global-unicast class.
pub(crate) fn select_temporary(entries: &[InterfaceV6], sys_src: Ipv6Addr) -> Option<Ipv6Addr> {
    let sys = entries.iter().find(|e| e.addr == sys_src)?;
    if is_usable_temp(sys) {
        return Some(sys_src);
    }
    if let Some(found) = entries.iter().find(|e| {
        e.ifindex == sys.ifindex
            && e.scope == sys.scope
            && same_prefix64(e.addr, sys.addr)
            && is_usable_temp(e)
    }) {
        return Some(found.addr);
    }
    if sys.scope == 0x00 {
        return entries
            .iter()
            .find(|e| {
                e.ifindex == sys.ifindex
                    && e.scope == 0x00
                    && is_ula(e.addr) == is_ula(sys.addr)
                    && is_usable_temp(e)
            })
            .map(|e| e.addr);
    }
    None
}

#[cfg(target_os = "linux")]
fn read_if_inet6() -> Option<Vec<InterfaceV6>> {
    let text = std::fs::read_to_string("/proc/net/if_inet6").ok()?;
    Some(parse_if_inet6(&text))
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn preferred_with_entries(entries: Option<&[InterfaceV6]>, sys_src: Ipv6Addr) -> Option<Ipv6Addr> {
    if !sys_src.is_loopback()
        && let Some(entries) = entries
        && let Some(temp) = select_temporary(entries, sys_src)
    {
        return Some(temp);
    }
    crate::transport::announceable_v6(sys_src).then_some(sys_src)
}

/// Outbound IPv6 source policy for one destination (#405).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum V6Source {
    /// Stamp `addr` via `IPV6_PKTINFO`; `safe_default` is `true` only when
    /// falling back to `src_ip: None` cannot leak a non-announceable (EUI-64)
    /// source.
    Temporary { addr: Ipv6Addr, safe_default: bool },
    /// Kernel default (`src_ip: None`) is safe (no route, loopback, or
    /// `announceable_v6`).
    SystemDefault,
    /// Kernel default is unsafe (e.g. modified EUI-64) and no temporary IPv6
    /// is available: suppress direct sends rather than leaking it on the wire.
    UnsafeDefault,
}

fn source_policy_with_entries(entries: Option<&[InterfaceV6]>, sys_src: Ipv6Addr) -> V6Source {
    if sys_src.is_loopback() {
        return V6Source::SystemDefault;
    }
    let safe_default = crate::transport::announceable_v6(sys_src);
    if let Some(entries) = entries
        && let Some(addr) = select_temporary(entries, sys_src)
    {
        return V6Source::Temporary { addr, safe_default };
    }
    if safe_default {
        V6Source::SystemDefault
    } else {
        V6Source::UnsafeDefault
    }
}

/// Returns the IPv6 address when `addr` is a routable unicast IPv6 destination
/// (not IPv4, loopback, unspecified, IPv4-mapped, or multicast).
pub(crate) fn routable_v6_ip(addr: SocketAddr) -> Option<Ipv6Addr> {
    let SocketAddr::V6(v6) = addr else {
        return None;
    };
    let ip = *v6.ip();
    (!ip.is_loopback()
        && !ip.is_unspecified()
        && ip.to_ipv4_mapped().is_none()
        && !ip.is_multicast())
    .then_some(ip)
}

/// Decide the outbound IPv6 source policy for direct sends to `target` (#405).
pub(crate) fn source_policy_v6(target: SocketAddr) -> V6Source {
    if routable_v6_ip(target).is_none() {
        return V6Source::SystemDefault;
    }
    let Some(IpAddr::V6(sys_src)) = crate::transport::source_for(target) else {
        return V6Source::SystemDefault;
    };
    #[cfg(target_os = "linux")]
    let entries = read_if_inet6();
    #[cfg(not(target_os = "linux"))]
    let entries: Option<Vec<InterfaceV6>> = None;
    source_policy_with_entries(entries.as_deref(), sys_src)
}

/// Whether the kernel's current default source for `target` is an
/// unannounceable non-loopback IPv6 address (e.g. modified EUI-64).
pub(crate) fn is_unsafe_default_v6(target: SocketAddr) -> bool {
    matches!(
        crate::transport::source_for(target),
        Some(IpAddr::V6(s)) if !s.is_loopback() && !crate::transport::announceable_v6(s)
    )
}

/// Whether sending a plain IPv6 STUN probe to `server` without `IPV6_PKTINFO`
/// uses a loopback or [`crate::transport::announceable_v6`] source address
/// (never modified EUI-64).
pub(crate) fn is_safe_default_v6_for_stun(server: SocketAddr) -> bool {
    matches!(
        crate::transport::source_for(server),
        Some(IpAddr::V6(s)) if s.is_loopback() || crate::transport::announceable_v6(s)
    )
}

/// A usable RFC 8981 temporary IPv6 source address to stamp via `IPV6_PKTINFO`
/// when sending to `target` (#405).
///
/// Returns `None` for IPv4, IPv4-mapped IPv6, loopback, unspecified, and
/// multicast targets, when no route exists, or on non-Linux systems (which
/// already prefer temporary IPv6 addresses natively).
pub(crate) fn temporary_source_v6(target: SocketAddr) -> Option<Ipv6Addr> {
    #[cfg(target_os = "linux")]
    {
        routable_v6_ip(target)?;
        let IpAddr::V6(sys_src) = crate::transport::source_for(target)? else {
            return None;
        };
        if sys_src.is_loopback() {
            return None;
        }
        select_temporary(&read_if_inet6()?, sys_src)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = target;
        None
    }
}

/// The IPv6 source address this device should announce or expect to send from
/// when reaching `target`: a usable temporary address on Linux when one exists,
/// otherwise the system's own source if [`crate::transport::announceable_v6`]
/// accepts it.
pub(crate) fn preferred_source_v6(target: SocketAddr) -> Option<Ipv6Addr> {
    routable_v6_ip(target)?;
    let IpAddr::V6(sys_src) = crate::transport::source_for(target)? else {
        return None;
    };
    #[cfg(target_os = "linux")]
    {
        let entries = read_if_inet6();
        preferred_with_entries(entries.as_deref(), sys_src)
    }
    #[cfg(not(target_os = "linux"))]
    {
        crate::transport::announceable_v6(sys_src).then_some(sys_src)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

    use super::*;

    const IFA_F_OPTIMISTIC: u32 = 0x04;
    const IFA_F_PERMANENT: u32 = 0x80;
    const IFA_F_STABLE_PRIVACY: u32 = 0x800;

    fn entry(addr: &str, ifindex: u32, scope: u8, flags: u32) -> InterfaceV6 {
        InterfaceV6 {
            addr: addr.parse().unwrap(),
            ifindex,
            prefix_len: 64,
            scope,
            flags,
        }
    }

    #[test]
    fn parse_if_inet6_reads_multi_interface_table_without_header() {
        let table = "\
00000000000000000000000000000001 01 80 10 80       lo
fe800000000000001c2d3e4f5a6b7c8d 02 40 20 80   wlan0
2a0204e02d8a1b005d1c8f3e91a26b47 02 40 00 800  wlan0
2A0204E02D8A1B00ABCDEF0123456789 02 40 00 01   wlan0
";
        let got = parse_if_inet6(table);
        assert_eq!(got.len(), 4);
        assert_eq!(
            got[0],
            InterfaceV6 {
                addr: Ipv6Addr::LOCALHOST,
                ifindex: 1,
                prefix_len: 128,
                scope: 0x10,
                flags: IFA_F_PERMANENT,
            }
        );
        assert_eq!(
            got[1],
            InterfaceV6 {
                addr: "fe80::1c2d:3e4f:5a6b:7c8d".parse().unwrap(),
                ifindex: 2,
                prefix_len: 64,
                scope: 0x20,
                flags: IFA_F_PERMANENT,
            }
        );
        assert_eq!(
            got[2],
            InterfaceV6 {
                addr: "2a02:4e0:2d8a:1b00:5d1c:8f3e:91a2:6b47".parse().unwrap(),
                ifindex: 2,
                prefix_len: 64,
                scope: 0x00,
                flags: IFA_F_STABLE_PRIVACY,
            }
        );
        assert_eq!(
            got[3],
            InterfaceV6 {
                addr: "2a02:4e0:2d8a:1b00:abcd:ef01:2345:6789".parse().unwrap(),
                ifindex: 2,
                prefix_len: 64,
                scope: 0x00,
                flags: IFA_F_TEMPORARY,
            }
        );
    }

    #[test]
    fn parse_if_inet6_ignores_blank_and_malformed_lines() {
        let table = concat!(
            "\n",
            "   \n",
            "too short 01 40 00\n",
            "00000000000000000000000000000001 01 80 10 80 lo extra_token\n",
            "0000000000000000000000000000001 01 80 10 80 lo\n",
            "0000000000000000000000000000000g 01 80 10 80 lo\n",
            "00000000000000000000000000000001 01 81 10 80 lo\n",
            "00000000000000000000000000000001 zz 40 00 01 eth0\n",
            "00000000000000000000000000000001 02 zz 00 01 eth0\n",
            "00000000000000000000000000000001 02 40 zz 01 eth0\n",
            "00000000000000000000000000000001 02 40 00 +1 eth0\n",
            "20010db8000000000000000000000001 03 80 00 01 eth0\n",
        );
        let got = parse_if_inet6(table);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].addr, "2001:db8::1".parse::<Ipv6Addr>().unwrap());
        assert_eq!(got[0].ifindex, 3);
        assert_eq!(got[0].prefix_len, 128);
        assert_eq!(got[0].flags, IFA_F_TEMPORARY);
    }

    #[test]
    fn select_temporary_prefers_temp_over_stable_privacy_and_eui64() {
        let stable: Ipv6Addr = "2a02:4e0:2d8a:1b00:5d1c:8f3e:91a2:6b47".parse().unwrap();
        let eui64: Ipv6Addr = "2a02:4e0:2d8a:1b00:21a:2bff:fe3c:4d5e".parse().unwrap();
        let temp: Ipv6Addr = "2a02:4e0:2d8a:1b00:abcd:ef01:2345:6789".parse().unwrap();
        let entries = vec![
            entry(&stable.to_string(), 2, 0x00, IFA_F_STABLE_PRIVACY),
            entry(&eui64.to_string(), 2, 0x00, IFA_F_PERMANENT),
            entry(&temp.to_string(), 2, 0x00, IFA_F_TEMPORARY),
        ];
        assert_eq!(select_temporary(&entries, stable), Some(temp));
        assert_eq!(select_temporary(&entries, eui64), Some(temp));
    }

    #[test]
    fn select_temporary_honors_sys_src_when_already_usable_temp() {
        let temp_old: Ipv6Addr = "2a02:4e0:2d8a:1b00:1111:2222:3333:4444".parse().unwrap();
        let temp_sys: Ipv6Addr = "2a02:4e0:2d8a:1b00:5555:6666:7777:8888".parse().unwrap();
        let entries = vec![
            entry(&temp_old.to_string(), 2, 0x00, IFA_F_TEMPORARY),
            entry(&temp_sys.to_string(), 2, 0x00, IFA_F_TEMPORARY),
        ];
        assert_eq!(select_temporary(&entries, temp_sys), Some(temp_sys));
    }

    #[test]
    fn select_temporary_skips_unusable_flags_and_unannounceable_addresses() {
        let stable: Ipv6Addr = "2a02:4e0:2d8a:1b00:5d1c:8f3e:91a2:6b47".parse().unwrap();
        let good: Ipv6Addr = "2a02:4e0:2d8a:1b00:9999:8888:7777:6666".parse().unwrap();
        let entries = vec![
            entry(&stable.to_string(), 2, 0x00, IFA_F_STABLE_PRIVACY),
            // Deprecated temporary
            entry(
                "2a02:4e0:2d8a:1b00:1::1",
                2,
                0x00,
                IFA_F_TEMPORARY | IFA_F_DEPRECATED,
            ),
            // Tentative temporary
            entry(
                "2a02:4e0:2d8a:1b00:1::2",
                2,
                0x00,
                IFA_F_TEMPORARY | IFA_F_TENTATIVE,
            ),
            // Optimistic + tentative temporary
            entry(
                "2a02:4e0:2d8a:1b00:1::3",
                2,
                0x00,
                IFA_F_TEMPORARY | IFA_F_OPTIMISTIC | IFA_F_TENTATIVE,
            ),
            // DAD-failed temporary
            entry(
                "2a02:4e0:2d8a:1b00:1::4",
                2,
                0x00,
                IFA_F_TEMPORARY | IFA_F_DADFAILED,
            ),
            // EUI-64 even if flagged temporary
            entry(
                "2a02:4e0:2d8a:1b00:21a:2bff:fe3c:4d5e",
                2,
                0x00,
                IFA_F_TEMPORARY,
            ),
            // Link-local even if flagged temporary
            entry("fe80::abcd:ef01:2345:6789", 2, 0x00, IFA_F_TEMPORARY),
            // Loopback / IPv4-mapped / IPv4-compatible / multicast even if flagged temporary
            entry("::1", 2, 0x00, IFA_F_TEMPORARY),
            entry("::ffff:192.0.2.1", 2, 0x00, IFA_F_TEMPORARY),
            entry("::192.0.2.1", 2, 0x00, IFA_F_TEMPORARY),
            entry("ff02::1", 2, 0x00, IFA_F_TEMPORARY),
            // Usable temporary
            entry(&good.to_string(), 2, 0x00, IFA_F_TEMPORARY),
        ];
        assert_eq!(select_temporary(&entries, stable), Some(good));

        // If sys_src itself is deprecated temporary, pick the fresh one.
        let dep: Ipv6Addr = "2a02:4e0:2d8a:1b00:1::1".parse().unwrap();
        assert_eq!(select_temporary(&entries, dep), Some(good));
    }

    #[test]
    fn select_temporary_enforces_ifindex_scope_and_ula_vs_gua_boundaries() {
        let gua_sys: Ipv6Addr = "2a02:4e0:2d8a:1::1".parse().unwrap();
        let ula_sys: Ipv6Addr = "fd12:3456:789a:1::1".parse().unwrap();
        let gua_other_if: Ipv6Addr = "2a02:4e0:2d8a:1::abcd".parse().unwrap();
        let ula_temp: Ipv6Addr = "fd12:3456:789a:1::beef".parse().unwrap();
        let gua_other_prefix: Ipv6Addr = "2a02:4e0:2d8a:2::cafe".parse().unwrap();
        let ula_other_prefix: Ipv6Addr = "fd12:3456:789a:2::feed".parse().unwrap();

        // Temporary on a different interface is never selected, nor is ULA for GUA.
        let cross_only = vec![
            entry(&gua_sys.to_string(), 2, 0x00, IFA_F_STABLE_PRIVACY),
            entry(&gua_other_if.to_string(), 3, 0x00, IFA_F_TEMPORARY),
            entry(&ula_temp.to_string(), 2, 0x00, IFA_F_TEMPORARY),
        ];
        assert_eq!(select_temporary(&cross_only, gua_sys), None);

        // GUA temporary is never selected for a ULA route.
        let gua_only_for_ula = vec![
            entry(&ula_sys.to_string(), 2, 0x00, IFA_F_STABLE_PRIVACY),
            entry(&gua_other_prefix.to_string(), 2, 0x00, IFA_F_TEMPORARY),
        ];
        assert_eq!(select_temporary(&gua_only_for_ula, ula_sys), None);

        // Global-scope fallback allows another /64 of the same class on the same interface.
        let same_class = vec![
            entry(&gua_sys.to_string(), 2, 0x00, IFA_F_STABLE_PRIVACY),
            entry(&ula_sys.to_string(), 2, 0x00, IFA_F_STABLE_PRIVACY),
            entry(&ula_other_prefix.to_string(), 2, 0x00, IFA_F_TEMPORARY),
            entry(&gua_other_prefix.to_string(), 2, 0x00, IFA_F_TEMPORARY),
        ];
        assert_eq!(
            select_temporary(&same_class, gua_sys),
            Some(gua_other_prefix)
        );
        assert_eq!(
            select_temporary(&same_class, ula_sys),
            Some(ula_other_prefix)
        );

        // Non-global scope does not fall back to global scope across prefixes.
        let site_sys: Ipv6Addr = "fec0::1".parse().unwrap();
        let site_entries = vec![
            entry(&site_sys.to_string(), 2, 0x40, IFA_F_PERMANENT),
            entry(&gua_other_prefix.to_string(), 2, 0x00, IFA_F_TEMPORARY),
        ];
        assert_eq!(select_temporary(&site_entries, site_sys), None);

        // Missing sys_src returns None.
        let missing: Ipv6Addr = "2001:db8::99".parse().unwrap();
        assert_eq!(select_temporary(&same_class, missing), None);
    }

    #[test]
    fn preferred_with_entries_falls_back_safely_without_leaking_eui64() {
        let stable: Ipv6Addr = "2a02:4e0:2d8a:1b00:5d1c:8f3e:91a2:6b47".parse().unwrap();
        let eui64: Ipv6Addr = "2a02:4e0:2d8a:1b00:21a:2bff:fe3c:4d5e".parse().unwrap();
        let temp: Ipv6Addr = "2a02:4e0:2d8a:1b00:abcd:ef01:2345:6789".parse().unwrap();

        let with_temp = vec![
            entry(&stable.to_string(), 2, 0x00, IFA_F_STABLE_PRIVACY),
            entry(&eui64.to_string(), 2, 0x00, IFA_F_PERMANENT),
            entry(&temp.to_string(), 2, 0x00, IFA_F_TEMPORARY),
        ];
        assert_eq!(preferred_with_entries(Some(&with_temp), stable), Some(temp));
        assert_eq!(preferred_with_entries(Some(&with_temp), eui64), Some(temp));

        let without_temp = vec![
            entry(&stable.to_string(), 2, 0x00, IFA_F_STABLE_PRIVACY),
            entry(&eui64.to_string(), 2, 0x00, IFA_F_PERMANENT),
        ];
        assert_eq!(
            preferred_with_entries(Some(&without_temp), stable),
            Some(stable)
        );
        assert_eq!(preferred_with_entries(Some(&without_temp), eui64), None);
        assert_eq!(preferred_with_entries(None, stable), Some(stable));
        assert_eq!(preferred_with_entries(None, eui64), None);
        assert_eq!(
            preferred_with_entries(Some(&with_temp), Ipv6Addr::LOCALHOST),
            None
        );
    }

    #[test]
    fn source_policy_with_entries_covers_all_outcomes() {
        let stable: Ipv6Addr = "2a02:4e0:2d8a:1b00:5d1c:8f3e:91a2:6b47".parse().unwrap();
        let eui64: Ipv6Addr = "2a02:4e0:2d8a:1b00:21a:2bff:fe3c:4d5e".parse().unwrap();
        let temp: Ipv6Addr = "2a02:4e0:2d8a:1b00:abcd:ef01:2345:6789".parse().unwrap();

        let with_temp = vec![
            entry(&stable.to_string(), 2, 0x00, IFA_F_STABLE_PRIVACY),
            entry(&eui64.to_string(), 2, 0x00, IFA_F_PERMANENT),
            entry(&temp.to_string(), 2, 0x00, IFA_F_TEMPORARY),
        ];
        assert_eq!(
            source_policy_with_entries(Some(&with_temp), stable),
            V6Source::Temporary {
                addr: temp,
                safe_default: true,
            }
        );
        assert_eq!(
            source_policy_with_entries(Some(&with_temp), eui64),
            V6Source::Temporary {
                addr: temp,
                safe_default: false,
            }
        );
        assert_eq!(
            source_policy_with_entries(Some(&with_temp), Ipv6Addr::LOCALHOST),
            V6Source::SystemDefault
        );

        let without_temp = vec![
            entry(&stable.to_string(), 2, 0x00, IFA_F_STABLE_PRIVACY),
            entry(&eui64.to_string(), 2, 0x00, IFA_F_PERMANENT),
        ];
        assert_eq!(
            source_policy_with_entries(Some(&without_temp), stable),
            V6Source::SystemDefault
        );
        assert_eq!(
            source_policy_with_entries(Some(&without_temp), eui64),
            V6Source::UnsafeDefault
        );
        assert_eq!(
            source_policy_with_entries(None, stable),
            V6Source::SystemDefault
        );
        assert_eq!(
            source_policy_with_entries(None, eui64),
            V6Source::UnsafeDefault
        );
    }

    #[test]
    fn entry_points_ignore_non_routable_targets_and_handle_doc_target() {
        let ignored: &[SocketAddr] = &[
            "192.0.2.1:9".parse().unwrap(),
            "127.0.0.1:9".parse().unwrap(),
            "[::1]:9".parse().unwrap(),
            "[::]:9".parse().unwrap(),
            "[::ffff:192.0.2.1]:9".parse().unwrap(),
            "[ff02::1]:9".parse().unwrap(),
            "[ff0e::1]:9".parse().unwrap(),
        ];
        for &target in ignored {
            assert_eq!(routable_v6_ip(target), None, "{target}");
            assert_eq!(temporary_source_v6(target), None, "{target}");
            assert_eq!(preferred_source_v6(target), None, "{target}");
            assert_eq!(
                source_policy_v6(target),
                V6Source::SystemDefault,
                "{target}"
            );
        }
        // Global documentation target: must never panic and any returned address
        // must be announceable.
        let doc: SocketAddr = "[2001:db8::1]:9".parse().unwrap();
        assert_eq!(
            routable_v6_ip(doc),
            Some("2001:db8::1".parse::<Ipv6Addr>().unwrap())
        );
        if let Some(ip) = temporary_source_v6(doc) {
            assert!(crate::transport::announceable_v6(ip));
        }
        if let Some(ip) = preferred_source_v6(doc) {
            assert!(crate::transport::announceable_v6(ip));
        }
        if let V6Source::Temporary { addr, .. } = source_policy_v6(doc) {
            assert!(crate::transport::announceable_v6(addr));
        }
        // Unscoped link-local is never safe for STUN on any OS, and counts as
        // an unsafe default whenever the OS resolves a link-local source for it.
        let link_local: SocketAddr = "[fe80::1]:3478".parse().unwrap();
        assert!(!is_safe_default_v6_for_stun(link_local));
        assert_eq!(
            is_unsafe_default_v6(link_local),
            crate::transport::source_for(link_local).is_some()
        );
        // IPv6 loopback is always safe for STUN when IPv6 loopback is available.
        let loopback: SocketAddr = "[::1]:3478".parse().unwrap();
        if crate::transport::source_for(loopback).is_some() {
            assert!(!is_unsafe_default_v6(loopback));
            assert!(is_safe_default_v6_for_stun(loopback));
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn read_if_inet6_reads_proc_on_linux() {
        if let Some(entries) = read_if_inet6() {
            assert!(
                entries.iter().any(|e| e.addr == Ipv6Addr::LOCALHOST),
                "expected ::1 in /proc/net/if_inet6: {entries:?}"
            );
        }
    }
}
