#![no_main]
//! Every relay message parser must accept or refuse any input without
//! panicking, and what it accepts must encode back to the same bytes.

use libfuzzer_sys::fuzz_target;
use totaldrop_proto::relay;

fuzz_target!(|data: &[u8]| {
    if let Some(c) = relay::parse_challenge(data) {
        assert_eq!(relay::challenge(&c), data);
    }
    if let Some((k, s)) = relay::parse_hello(data) {
        assert_eq!(relay::hello(&k, &s), data);
    }
    if let Some((k, d)) = relay::parse_frame(data) {
        assert_eq!(relay::frame(&k, d), data);
    }
    if let Some(n) = relay::parse_probe(data) {
        assert_eq!(relay::parse_probe(&relay::probe(&n)), Some(n));
    }
    let _ = relay::parse_answer(data, &[0u8; 12]);
    let _ = relay::is_ok(data);
});
