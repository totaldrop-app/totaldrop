//! The rules every translation must follow (ADR 0008, "CI denetimleri").
//! `cargo test -p totaldrop-l10n` runs them on the compiled-in files, so CI
//! fails on a broken translation; missing translations are only reported.

use std::collections::BTreeSet;

use crate::{TEMPLATE, files, status};

/// Placeholders that must appear exactly once in every translation, because
/// dropping or repeating them could mislead someone on a security screen.
pub const CRITICAL_PLACEHOLDERS: [&str; 4] = ["code", "words", "folder", "name"];

/// Characters no translation may contain: bidi embeddings, overrides and
/// isolates, and invisible zero-width characters (U+200B, U+2060, U+FEFF).
/// Left-to-right and right-to-left marks (U+200E, U+200F) are allowed.
fn forbidden(c: char) -> bool {
    matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200B}' | '\u{2060}' | '\u{FEFF}')
}

/// What [`run`] found.
#[derive(Debug, Default)]
pub struct Report {
    /// Rule violations: CI fails on any of these.
    pub errors: Vec<String>,
    /// Keys in the template that a language has not translated yet.
    pub missing: Vec<(String, String, usize)>,
}

/// Checks every compiled-in file against its template.
#[must_use]
pub fn run() -> Report {
    let all = files();
    let status = status();
    let mut report = Report::default();
    for file in &all {
        let where_ = |key: &str| format!("{}/{} `{key}`", file.domain, file.locale);
        let Some(template) = all
            .iter()
            .find(|f| f.domain == file.domain && f.locale == TEMPLATE)
        else {
            report
                .errors
                .push(format!("{}: no {TEMPLATE} template", file.domain));
            continue;
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
                    report.errors.push(format!("{}: {e}", where_(key)));
                    continue;
                }
            };
            let raw = file.raw.get(key).map(String::as_str).unwrap_or_default();
            if raw.chars().any(forbidden) {
                report.errors.push(format!(
                    "{}: invisible or direction control character",
                    where_(key)
                ));
            }
            let lower = raw.to_lowercase();
            if lower.contains("http://") || lower.contains("https://") || lower.contains("www.") {
                report
                    .errors
                    .push(format!("{}: contains a link", where_(key)));
            }
            if raw.contains('<') && raw.contains('>') {
                report
                    .errors
                    .push(format!("{}: contains markup", where_(key)));
            }
            if file.locale == TEMPLATE {
                continue;
            }
            let Some(Ok(expected)) = template.messages.get(key) else {
                report
                    .errors
                    .push(format!("{}: not in the template", where_(key)));
                continue;
            };
            let want = expected.arguments();
            let got = message.arguments();
            let unknown: BTreeSet<_> = got.difference(&want).collect();
            if !unknown.is_empty() {
                report
                    .errors
                    .push(format!("{}: unknown placeholders {unknown:?}", where_(key)));
            }
            for p in CRITICAL_PLACEHOLDERS {
                let needle = format!("{{{p}}}");
                let in_template = template
                    .raw
                    .get(key)
                    .map_or(0, |t| t.matches(&needle).count());
                if in_template > 0 && raw.matches(&needle).count() != in_template {
                    report.errors.push(format!(
                        "{}: `{{{p}}}` must appear exactly as in English",
                        where_(key)
                    ));
                }
            }
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

#[cfg(test)]
mod tests {
    #[test]
    fn every_translation_follows_the_rules() {
        let report = super::run();
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
        assert!(super::forbidden('\u{202E}'));
        assert!(super::forbidden('\u{2068}'));
        assert!(!super::forbidden('\u{200F}'));
        assert!(!super::forbidden('ı'));
    }
}
