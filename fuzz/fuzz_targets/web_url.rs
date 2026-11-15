#![no_main]
// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Any input must validate cleanly without panicking; every accepted web URL
//! must be pure printable ASCII within [`MAX_WEB_URL_BYTES`], start with
//! `https://` or `http://`, survive re-parsing unchanged, and reject trailing
//! tokens.

use libfuzzer_sys::fuzz_target;
use totaldrop_proto::{MAX_WEB_URL_BYTES, single_web_url};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let parsed = single_web_url(s);
        if s.trim_ascii().len() > MAX_WEB_URL_BYTES {
            assert_eq!(parsed, None);
        }
        if let Some(url) = parsed {
            assert!(url.len() <= MAX_WEB_URL_BYTES);
            assert!(url.is_ascii());
            assert!(url.starts_with("https://") || url.starts_with("http://"));
            assert!(
                !url.chars()
                    .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
            );
            assert_eq!(single_web_url(&url).as_deref(), Some(url.as_str()));
            assert_eq!(single_web_url(&format!("{url} trailing")), None);
        }
    }
});
