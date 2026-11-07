// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! The parts of `totaldrop-l10n` that Flutter's generated localizations do
//! not cover: choosing the language the same way as the Rust programs,
//! revealing hidden characters in names from the other side, and which
//! strings are unreviewed drafts (ADR 0008).

/// The best of `available` for the device's ordered `preferences` (BCP 47
/// tags), or `None` for the default (English).
#[flutter_rust_bridge::frb(sync)]
pub fn negotiate_locale(preferences: Vec<String>, available: Vec<String>) -> Option<String> {
    let prefs: Vec<_> = preferences
        .iter()
        .filter_map(|p| totaldrop_l10n::locale::parse(p))
        .collect();
    let avail: Vec<&str> = available.iter().map(String::as_str).collect();
    totaldrop_l10n::locale::negotiate(&prefs, &avail).map(str::to_owned)
}

/// A name from the other side with hidden direction and control characters
/// shown as `<U+202E>`.
#[flutter_rust_bridge::frb(sync)]
pub fn reveal_controls(name: String) -> String {
    totaldrop_l10n::reveal_controls(&name)
}

/// The most characters of a received text shown on screen (#326). Laying
/// out one very long line is slow (in Qt 16 Ki characters take some 40 ms,
/// 1 Mi nearly a minute); Copy text still copies the whole text.
pub const SHOWN_CHARS: usize = 16 * 1024;

/// How a received text is shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShownText {
    /// Hidden characters named (`reveal_text`), then cut to
    /// [`SHOWN_CHARS`] characters.
    pub text: String,
    /// It was cut: only its beginning is shown.
    pub short: bool,
    /// Some characters are shown by their code (`U+202E`).
    pub hidden: bool,
}

/// A received text as it is shown: hidden direction, control and format
/// characters named, the rest as it came, at most [`SHOWN_CHARS`].
#[flutter_rust_bridge::frb(sync)]
pub fn shown_text(text: String) -> ShownText {
    let revealed = totaldrop_l10n::reveal_text(&text);
    // Only named characters make it differ; \r\n is shown as \n.
    let hidden = revealed != text.replace("\r\n", "\n");
    match revealed.char_indices().nth(SHOWN_CHARS) {
        Some((at, _)) => ShownText {
            text: revealed[..at].to_owned(),
            short: true,
            hidden,
        },
        None => ShownText {
            text: revealed,
            short: false,
            hidden,
        },
    }
}

/// Whether `key` must be shown in English because, in language `tag`, it
/// is a safety-critical string nobody has reviewed yet.
#[flutter_rust_bridge::frb(sync)]
pub fn needs_english(tag: String, key: String) -> bool {
    totaldrop_l10n::needs_english(&tag, &key)
}

/// Whether some text in language `tag` is an unreviewed draft.
#[flutter_rust_bridge::frb(sync)]
pub fn has_draft_translation(tag: String) -> bool {
    totaldrop_l10n::has_drafts(&tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negotiation() {
        let avail = vec!["en".to_owned(), "tr".to_owned()];
        assert_eq!(
            negotiate_locale(vec!["tr-TR".into()], avail.clone()).as_deref(),
            Some("tr")
        );
        assert_eq!(negotiate_locale(vec!["de".into()], avail), None);
    }

    #[test]
    fn a_text_is_shown_with_hidden_characters_named() {
        let s = shown_text("a\r\nb\tc".into());
        assert_eq!(s.text, "a\nb\tc");
        assert!(!s.hidden && !s.short);
        let s = shown_text("x\u{202E}y".into());
        assert_eq!(s.text, "x<U+202E>y");
        assert!(s.hidden);
    }

    #[test]
    fn a_long_text_is_shown_in_part() {
        let s = shown_text("é".repeat(SHOWN_CHARS + 5));
        assert!(s.short);
        assert_eq!(s.text.chars().count(), SHOWN_CHARS);
        let s = shown_text("é".repeat(SHOWN_CHARS));
        assert!(!s.short);
    }

    #[test]
    fn critical_strings() {
        assert!(!needs_english("tr".into(), "verificationTitle".into()));
        assert!(!needs_english("xx".into(), "settings".into()));
        // No file for xx: the text is English already.
        assert!(!needs_english("xx".into(), "verificationTitle".into()));
    }
}
