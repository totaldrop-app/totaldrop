#![no_main]
// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Router answers to port mapping requests come from the local network.
//! Every parser must accept or refuse any input without panicking, and an
//! accepted mapping must answer the request it was matched against.
//! The first 14 bytes choose that request (nonce and internal port); the
//! rest is the answer.

use libfuzzer_sys::fuzz_target;
use totaldrop_proto::portmap::{self, Answer};

fuzz_target!(|data: &[u8]| {
    let Some((head, m)) = data.split_at_checked(14) else {
        return;
    };
    let nonce: [u8; 12] = head[..12].try_into().unwrap();
    let port = u16::from_be_bytes([head[12], head[13]]);
    match portmap::parse_pcp_map(m, &nonce, port) {
        Some(Answer::Mapped { external, lifetime }) => {
            assert_eq!(m.get(24..36), Some(&nonce[..]));
            assert_eq!(m.get(40..42), Some(&port.to_be_bytes()[..]));
            assert!(lifetime == 0 || (external.1 != 0 && !external.0.is_unspecified()));
        }
        Some(Answer::Refused(_)) => assert_eq!(m.get(24..36), Some(&nonce[..])),
        _ => {}
    }
    if let Some(Ok(ip)) = portmap::parse_pmp_address(m) {
        assert!(!ip.is_unspecified());
    }
    if let Some(r) = portmap::parse_pmp_map(m, port) {
        assert_eq!((m.len(), m.get(8..10)), (16, Some(&port.to_be_bytes()[..])));
        if let Ok((external, lifetime)) = r {
            assert!(lifetime == 0 || external != 0);
        }
    }
});
