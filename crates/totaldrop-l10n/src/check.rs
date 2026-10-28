//! The rules every translation must follow (ADR 0008, "CI denetimleri").
//! `cargo test -p totaldrop-l10n` runs them on the compiled-in files, so CI
//! fails on a broken translation; missing translations are only reported.

use std::collections::BTreeSet;

use crate::{File, Message, TEMPLATE, catalog, is_hidden, status};

/// Placeholders a translation must insert exactly as the English text does,
/// in every case, and never use to choose a case: dropping, repeating or
/// replacing them could mislead someone on a security screen (a fake code,
/// a different folder).
pub const CRITICAL_PLACEHOLDERS: [&str; 4] = ["code", "words", "folder", "name"];

/// English answers that mean no; a translation's yes-words may not be one.
const NO_WORDS: [&str; 3] = ["n", "no", "not"];

/// Top-level domains that would make a bare word a link.
const TLDS: [&str; 16] = [
    "com", "org", "net", "app", "io", "co", "info", "xyz", "link", "site", "online", "me", "ly",
    "gl", "to", "dev",
];

/// Hidden characters, except the left-to-right and right-to-left marks a
/// reviewed translation may need.
fn forbidden(c: char) -> bool {
    is_hidden(c) && !matches!(c, '\u{200E}' | '\u{200F}')
}

/// Whether text contains a link: a scheme with a host, `www.`, or `word.tld`.
fn has_link(text: &str) -> bool {
    let lower = text.to_lowercase();
    // A scheme followed by a host is a link; naming a scheme ("must start
    // with wss://") is not.
    let scheme_host = lower.match_indices("://").any(|(i, _)| {
        lower
            .get(i.saturating_add(3)..)
            .and_then(|r| r.chars().next())
            .is_some_and(char::is_alphanumeric)
    });
    if scheme_host || lower.contains("www.") {
        return true;
    }
    let chars: Vec<char> = lower.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if *c != '.' || i == 0 {
            continue;
        }
        let before = chars
            .get(i.saturating_sub(1))
            .is_some_and(|b| b.is_alphanumeric());
        let tld: String = chars
            .iter()
            .skip(i.saturating_add(1))
            .take_while(|c| c.is_ascii_alphabetic())
            .collect();
        if before && TLDS.contains(&tld.as_str()) {
            return true;
        }
    }
    false
}

/// Whether text contains an HTML or XML tag (`<b>`, `</a>`, `<!--`), not
/// merely arrows or comparisons.
fn has_markup(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    chars.windows(2).any(|w| match w {
        [a, b] => *a == '<' && (b.is_ascii_alphabetic() || *b == '/' || *b == '!'),
        _ => false,
    })
}

/// What [`run`] found.
#[derive(Debug, Default)]
pub struct Report {
    /// Rule violations: CI fails on any of these.
    pub errors: Vec<String>,
    /// (domain, language, how many) template keys not translated yet.
    pub missing: Vec<(String, String, usize)>,
}

/// Checks every compiled-in file against its template.
#[must_use]
pub fn run() -> Report {
    let all = &catalog().files;
    let status = status();
    let mut report = Report::default();
    report.errors.extend(catalog().broken.iter().cloned());
    for file in all {
        check_file(file, all, &mut report);
    }
    for key in &status.critical {
        if !all
            .iter()
            .any(|f| f.locale == TEMPLATE && f.messages.contains_key(key))
        {
            report.errors.push(format!(
                "status.json: critical key `{key}` is in no template"
            ));
        }
    }
    report
}

fn check_file(file: &File, all: &[File], report: &mut Report) {
    let status = status();
    let at = |key: &str| format!("{}/{} `{key}`", file.domain, file.locale);
    let Some(template) = all
        .iter()
        .find(|f| f.domain == file.domain && f.locale == TEMPLATE)
    else {
        report
            .errors
            .push(format!("{}: no {TEMPLATE} template", file.domain));
        return;
    };
    // A regional or script variant needs its base language too (pt-BR
    // needs pt), as Flutter's gen_l10n does.
    if let Some((base, _)) = file.locale.split_once('-')
        && !all
            .iter()
            .any(|f| f.domain == file.domain && f.locale == base)
    {
        report.errors.push(format!(
            "{}/{}: no base file for {base}",
            file.domain, file.locale
        ));
    }
    if file.locale != TEMPLATE && !status.reviewed.contains_key(&file.locale) {
        report.errors.push(format!(
            "{}/{}: not listed in l10n/status.json",
            file.domain, file.locale
        ));
    }
    for (key, parsed) in &file.messages {
        let message = match parsed {
            Ok(m) => m,
            Err(e) => {
                report.errors.push(format!("{}: {e}", at(key)));
                continue;
            }
        };
        let raw = file.raw.get(key).map(String::as_str).unwrap_or_default();
        if let Some(c) = raw.chars().find(|c| forbidden(*c)) {
            report.errors.push(format!(
                "{}: invisible or control character U+{:04X}",
                at(key),
                u32::from(c)
            ));
        }
        if has_link(raw) {
            report.errors.push(format!("{}: contains a link", at(key)));
        }
        if has_markup(raw) {
            report.errors.push(format!("{}: contains markup", at(key)));
        }
        if key == "cliYes" {
            check_yes(raw, &at(key), report);
        }
        if file.locale == TEMPLATE {
            continue;
        }
        let Some(Ok(expected)) = template.messages.get(key) else {
            report
                .errors
                .push(format!("{}: not in the template", at(key)));
            continue;
        };
        check_placeholders(message, expected, &at(key), report);
    }
    if file.locale != TEMPLATE {
        let missing = template
            .messages
            .keys()
            .filter(|k| !file.messages.contains_key(*k))
            .count();
        if missing > 0 {
            report
                .missing
                .push((file.domain.to_owned(), file.locale.clone(), missing));
        }
    }
}

fn check_placeholders(message: &Message, expected: &Message, at: &str, report: &mut Report) {
    let want = expected.arguments();
    let got = message.arguments();
    let unknown: BTreeSet<_> = got.difference(&want).collect();
    if !unknown.is_empty() {
        report
            .errors
            .push(format!("{at}: unknown placeholders {unknown:?}"));
    }
    let selectors = message.selectors();
    for p in CRITICAL_PLACEHOLDERS {
        if !want.contains(p) {
            continue;
        }
        if selectors.contains(p) {
            report
                .errors
                .push(format!("{at}: `{{{p}}}` may not choose a case"));
        }
        // However the cases fall, the placeholder appears as often as in
        // English (usually exactly once).
        if message.counts(p) != expected.counts(p) {
            report.errors.push(format!(
                "{at}: `{{{p}}}` must appear in every case as often as in English"
            ));
        }
    }
}

fn check_yes(raw: &str, at: &str, report: &mut Report) {
    for word in raw.split('|') {
        let w = word.trim().to_lowercase();
        if w.is_empty() || NO_WORDS.contains(&w.as_str()) {
            report
                .errors
                .push(format!("{at}: `{word}` cannot mean yes"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_translation_follows_the_rules() {
        let report = run();
        for (domain, locale, n) in &report.missing {
            println!("{domain}/{locale}: {n} strings not translated yet (shown in English)");
        }
        assert!(
            report.errors.is_empty(),
            "translation errors:\n{}",
            report.errors.join("\n")
        );
    }

    #[test]
    fn forbidden_characters() {
        for c in [
            '\u{202E}',
            '\u{2068}',
            '\u{061C}',
            '\u{2028}',
            '\u{00AD}',
            '\u{E0041}',
            '\r',
            '\u{1B}',
            '\u{FEFF}',
        ] {
            assert!(forbidden(c), "U+{:04X}", u32::from(c));
        }
        for c in ['\u{200F}', '\u{200E}', '\u{200C}', '\u{200D}', 'ı', 'ش'] {
            assert!(!forbidden(c), "U+{:04X}", u32::from(c));
        }
    }

    #[test]
    fn links_and_markup() {
        assert!(has_link("see https://x.example"));
        assert!(!has_link("Must start with wss://"));
        assert!(has_link("visit totaldrop-help.com now"));
        assert!(has_link("www.example.org"));
        assert!(!has_link("e.g. this, or U.S. that, 1.5 MB"));
        assert!(has_markup("<b>bold</b>"));
        assert!(has_markup("a <!-- b"));
        assert!(!has_markup("a -> b <- c"));
        assert!(!has_markup("[y/N] 3 < 4"));
    }

    fn placeholder_errors(translation: &str, english: &str) -> Vec<String> {
        let mut report = Report::default();
        if let (Ok(t), Ok(e)) = (Message::parse(translation), Message::parse(english)) {
            check_placeholders(&t, &e, "x", &mut report);
        }
        report.errors
    }

    #[test]
    fn critical_placeholders_cannot_be_faked() {
        let en = "Code: {code}";
        assert!(placeholder_errors("Kod: {code}", en).is_empty());
        assert!(!placeholder_errors("Kod: 1-fake-word", en).is_empty());
        assert!(!placeholder_errors("Kod: {code} {code}", en).is_empty());
        assert!(
            !placeholder_errors("Code: {code, select, zzzz{{code}} other{1-fake-word}}", en)
                .is_empty()
        );
        // A plural with more cases is fine when every case has it once.
        let en = "{n, plural, one{{n} file in {folder}} other{{n} files in {folder}}}";
        let pl = "{n, plural, one{{n} plik w {folder}} few{{n} pliki w {folder}} many{{n} plików w {folder}} other{{n} pliku w {folder}}}";
        assert!(placeholder_errors(pl, en).is_empty());
        let bad = "{n, plural, one{{n} plik w {folder}} other{{n} pliku}}";
        assert!(!placeholder_errors(bad, en).is_empty());
    }

    #[test]
    fn yes_words() {
        let errors = |raw: &str| {
            let mut r = Report::default();
            check_yes(raw, "x", &mut r);
            r.errors.len()
        };
        assert_eq!(errors("e|evet"), 0);
        assert_eq!(errors("n|no|j|ja"), 2);
        assert_eq!(errors("ja||j"), 1);
    }
}
