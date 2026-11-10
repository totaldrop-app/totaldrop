#![no_main]
// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Router answers to port mapping requests come from the local network.
//! Every parser must accept or refuse any input without panicking, and an
//! accepted mapping must answer the request it was matched against.

use libfuzzer_sys::fuzz_target;
use totaldrop_proto::portmap::{self, Answer};

fuzz_target!(|data: &[u8]| {
    let nonce: [u8; 12] = data.get(..12).and_then(|n| n.try_into().ok()).unwrap_or([0; 12]);
    let port = data.get(40..42).map_or(0, |p| u16::from_be_bytes([p[0], p[1]]));
    if let Some(Answer::Mapped { external, lifetime }) = portmap::parse_pcp_map(data, &nonce, port) {
        assert_eq!(data.get(24..36), Some(&nonce[..]));
        assert!(lifetime == 0 || (external.1 != 0 && !external.0.is_unspecified()));
    }
    let _ = portmap::parse_pcp_map(data, &[0; 12], 0);
    if let Some(Ok(ip)) = portmap::parse_pmp_address(data) {
        assert!(!ip.is_unspecified());
    }
    let internal = data.get(8..10).map_or(0, |p| u16::from_be_bytes([p[0], p[1]]));
    if let Some(Ok((external, lifetime))) = portmap::parse_pmp_map(data, internal) {
        assert!(lifetime == 0 || external != 0);
    }
});
