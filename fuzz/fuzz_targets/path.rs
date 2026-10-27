#![no_main]
//! Arbitrary bytes, split into segments, must never panic the path validator,
//! and anything it accepts must satisfy the rules again after normalization.

use libfuzzer_sys::fuzz_target;
use totaldrop_proto::validate_path;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let segments: Vec<&str> = s.split('\u{1f}').collect();
        if let Ok(safe) = validate_path(&segments) {
            let again = validate_path(safe.segments()).expect("accepted path must stay valid");
            assert_eq!(again, safe);
        }
    }
});
