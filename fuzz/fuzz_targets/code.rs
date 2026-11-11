#![no_main]
// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Any input must parse or fail cleanly, and every parsed code must survive a
//! round trip through both its canonical and its English form. Invisible
//! format characters and Arabic-Indic digits never change the result. A
//! code found in a longer text is a code the text holds. A link is read
//! strictly: a bare code reads as `parse` reads it, a code behind either
//! prefix reads as itself with its escapes decoded once (not twice), text
//! after the code is refused, and nothing longer than the limit is read.

use libfuzzer_sys::fuzz_target;
use totaldrop_proto::{MAX_LINK_BYTES, TransferCode, ascii_digits, strip_invisible_format};

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
        let linked = TransferCode::from_link(s);
        if s.len() > MAX_LINK_BYTES {
            assert_eq!(linked, None);
        } else {
            if let Ok(code) = TransferCode::parse(s) {
                assert_eq!(linked.as_ref(), Some(&code));
            }
            assert_eq!(TransferCode::from_link(&format!("{s}?x")), None, "{s:?}");
        }
        if let Ok(code) = TransferCode::parse(s) {
            let english = code.to_english();
            let link = |l: String| TransferCode::from_link(&l);
            assert_eq!(link(format!("totaldrop://r/{english}")), Some(code.clone()));
            assert_eq!(
                link(format!("https://totaldrop.app/r#{}", escaped(&english))),
                Some(code.clone())
            );
            assert_eq!(
                link(format!("totaldrop://r/{}", escaped(&escaped(&english)))),
                None
            );
        }
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
            // The text holds the code: its number and each word, as a
            // word or as its index, appear in the text as read.
            let read = ascii_digits(&strip_invisible_format(s)).to_lowercase();
            let english = code.to_english();
            let canonical = code.canonical();
            for (word, index) in english.split('-').zip(canonical.split('-')) {
                assert!(
                    read.contains(word) || read.contains(index),
                    "{s:?} -> {english}"
                );
            }
        }
    }
});

/// Every byte as `%XX`.
fn escaped(s: &str) -> String {
    s.bytes().map(|b| format!("%{b:02X}")).collect()
}
