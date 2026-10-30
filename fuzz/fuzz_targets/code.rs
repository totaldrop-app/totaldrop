#![no_main]
//! Any input must parse or fail cleanly, and every parsed code must survive a
//! round trip through both its canonical and its English form. Invisible
//! format characters and Arabic-Indic digits never change the result. A
//! code found in a longer text is a code the text holds.

use libfuzzer_sys::fuzz_target;
use totaldrop_proto::{TransferCode, ascii_digits, strip_invisible_format};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        assert_eq!(
            TransferCode::parse(s),
            TransferCode::parse(&strip_invisible_format(s))
        );
        assert_eq!(
            TransferCode::parse(s),
            TransferCode::parse(&ascii_digits(s))
        );
        let found = TransferCode::find(s);
        if let Ok(code) = TransferCode::parse(s) {
            assert_eq!(found.as_ref(), Some(&code));
            assert_eq!(
                TransferCode::parse(&code.canonical()).ok(),
                Some(code.clone())
            );
            assert_eq!(
                TransferCode::parse(&code.to_english()).ok(),
                Some(code.clone())
            );
            // What goes to CPace is ASCII, whatever digits were typed.
            assert!(code.canonical().is_ascii());
            assert!(code.to_english().is_ascii());
        }
        if let Some(code) = found {
            assert_eq!(TransferCode::parse(&code.canonical()).ok(), Some(code));
        }
    }
});
