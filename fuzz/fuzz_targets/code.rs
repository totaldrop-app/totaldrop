#![no_main]
//! Any input must parse or fail cleanly, and every parsed code must survive a
//! round trip through both its canonical and its English form.

use libfuzzer_sys::fuzz_target;
use totaldrop_proto::TransferCode;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(code) = TransferCode::parse(s) {
            assert_eq!(TransferCode::parse(&code.canonical()).ok(), Some(code.clone()));
            assert_eq!(TransferCode::parse(&code.to_english()).ok(), Some(code));
        }
    }
});
