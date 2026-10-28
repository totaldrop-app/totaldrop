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
    if let Some((k, datagrams)) = relay::parse_frame(data) {
        let mut b = relay::Batch::new(&k);
        for d in &datagrams {
            assert!(b.push(d));
        }
        assert_eq!(b.finish(), data);
    }
    if let Ok(s) = std::str::from_utf8(data) {
        if let Some((a, b)) = relay::parse_pair_name(s) {
            assert_eq!(relay::pair_name(&a, &b), s);
        }
    }
    let _ = relay::parse_stun_request(data);
    let _ = relay::parse_stun_response(data, &[0u8; 12]);
    let _ = relay::is_ok(data);
});
