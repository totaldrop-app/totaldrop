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
        assert_eq!(negotiate_locale(vec!["tr-TR".into()], avail.clone()).as_deref(), Some("tr"));
        assert_eq!(negotiate_locale(vec!["de".into()], avail), None);
    }

    #[test]
    fn critical_strings() {
        assert!(!needs_english("tr".into(), "verificationTitle".into()));
        assert!(!needs_english("xx".into(), "settings".into()));
        // No file for xx: the text is English already.
        assert!(!needs_english("xx".into(), "verificationTitle".into()));
    }
}
