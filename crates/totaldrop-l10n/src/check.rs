//! The rules every translation must follow (ADR 0008, "CI denetimleri").
//! `cargo test -p totaldrop-l10n` runs them on the compiled-in files, so CI
//! fails on a broken translation; missing translations are only reported.

use std::collections::BTreeSet;

use crate::{
    ALWAYS_YES, File, Message, TEMPLATE, catalog, fold_answer, is_hidden, layer_tags, status,
};

/// Placeholders a translation must insert exactly as the English text does,
/// in every case, and never use to choose a case: dropping, repeating or
/// replacing them could mislead someone on a security screen (a fake code,
/// a different folder).
pub const CRITICAL_PLACEHOLDERS: [&str; 4] = ["code", "words", "folder", "name"];

/// English answers that mean no; a translation's yes-words may not be one.
/// Each language's own no answer comes from its save prompt (see
/// [`prompt_no`]).
const NO_WORDS: [&str; 3] = ["n", "no", "not"];

/// The domains the command line looks a key up in, in order.
const CLI_DOMAINS: [&str; 2] = ["cli", "app"];

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
    check_answers(all, &mut report);
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
        if key == "cliPromptSave" {
            check_prompt(raw, &at(key), report);
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
        let w = fold_answer(word);
        if w.is_empty() || NO_WORDS.contains(&w.as_str()) {
            report
                .errors
                .push(format!("{at}: `{word}` cannot mean yes"));
        }
    }
}

/// The answer a save prompt offers for no: the second of the last
/// `[yes/no]` in it (`[y/N]` → `N`, `[ن/ل]` → `ل`).
fn prompt_no(raw: &str) -> Option<&str> {
    let open = raw.rfind('[')?;
    let inside = raw.get(open.saturating_add(1)..)?;
    let close = inside.find(']')?;
    let (_, no) = inside.get(..close)?.split_once('/')?;
    Some(no.trim())
}

fn check_prompt(raw: &str, at: &str, report: &mut Report) {
    match prompt_no(raw) {
        None => report.errors.push(format!(
            "{at}: shows no `[yes/no]` answers to check `cliYes` against"
        )),
        Some(no) => {
            let folded = fold_answer(no);
            if folded.is_empty() || folded.contains('/') || ALWAYS_YES.contains(&folded.as_str()) {
                report
                    .errors
                    .push(format!("{at}: `{no}` cannot be the answer for no"));
            }
        }
    }
}

/// The files the command line looks `key` up in for language `tag`, in
/// order, with their text: the language, its fallbacks and English, each
/// in [`CLI_DOMAINS`].
fn layers<'a>(all: &'a [File], tag: &str, key: &str) -> Vec<(&'a File, &'a str)> {
    layer_tags(tag)
        .iter()
        .flat_map(|t| {
            CLI_DOMAINS
                .iter()
                .filter_map(move |d| all.iter().find(|f| f.domain == *d && f.locale == *t))
        })
        .filter_map(|f| f.raw.get(key).map(|raw| (f, raw.as_str())))
        .collect()
}

/// Checks that no answer in `cliYes` is the answer the save prompt shown
/// with it offers for no, folded as the command line folds answers
/// ([`fold_answer`]). For each language this pairs every `cliYes` with
/// every `cliPromptSave` it could be shown with: a key can come from the
/// language, a fallback or English, since an unreviewed critical key is
/// skipped. The command line takes only the active language's answers and
/// [`ALWAYS_YES`], so another language's no-words do not matter (Arabic
/// yes `ن` is Urdu's no).
fn check_answers(all: &[File], report: &mut Report) {
    let tags: BTreeSet<&str> = all.iter().map(|f| f.locale.as_str()).collect();
    let mut errors = BTreeSet::new();
    for tag in tags {
        let prompts = layers(all, tag, "cliPromptSave");
        for (yes_file, yes) in layers(all, tag, "cliYes") {
            for (prompt_file, prompt) in &prompts {
                let Some(no) = prompt_no(prompt).map(fold_answer) else {
                    continue;
                };
                for word in yes.split('|') {
                    let w = fold_answer(word);
                    // check_yes reports these already.
                    if w.is_empty() || NO_WORDS.contains(&w.as_str()) {
                        continue;
                    }
                    if w == no {
                        errors.insert(format!(
                            "{}/{} `cliYes`: `{word}` is the answer for no in {}/{} `cliPromptSave`",
                            yes_file.domain, yes_file.locale, prompt_file.domain, prompt_file.locale
                        ));
                    }
                }
            }
        }
    }
    report.errors.extend(errors);
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
        // Folded as the command line folds answers.
        assert_eq!(errors("ＮＯ| N |j"), 2);
    }

    #[test]
    fn prompt_answers() {
        assert_eq!(prompt_no("Save into {folder}? [y/N] "), Some("N"));
        assert_eq!(prompt_no("هل تريد الحفظ في {folder}؟ [ن/ل] "), Some("ل"));
        assert_eq!(prompt_no("[a] b [ d / Н ]"), Some("Н"));
        assert_eq!(prompt_no("Save into {folder}? "), None);
        assert_eq!(prompt_no("Save [y/N"), None);
        let errors = |raw: &str| {
            let mut r = Report::default();
            check_prompt(raw, "x", &mut r);
            r.errors.len()
        };
        assert_eq!(errors("Сохранить в {folder}? [д/Н] "), 0);
        assert_eq!(errors("Save into {folder}? "), 1);
        assert_eq!(errors("[j/]"), 1);
        assert_eq!(errors("[j/a/n]"), 1);
        // English y and yes always mean yes.
        assert_eq!(errors("[j/Y]"), 1);
        assert_eq!(errors("[j/ｙｅｓ]"), 1);
    }

    /// Errors from [`check_answers`] for ARB files given as
    /// (locale, `cliYes`, `cliPromptSave`); an empty text leaves the key
    /// out.
    fn answer_errors(files: &[(&str, &str, &str)]) -> Vec<String> {
        let all: Vec<File> = files
            .iter()
            .filter_map(|(locale, yes, prompt)| {
                let mut json = serde_json::Map::new();
                json.insert("@@locale".into(), (*locale).into());
                if !yes.is_empty() {
                    json.insert("cliYes".into(), (*yes).into());
                }
                if !prompt.is_empty() {
                    json.insert("cliPromptSave".into(), (*prompt).into());
                }
                let text = serde_json::Value::Object(json).to_string();
                crate::parse_file("cli", &text).ok()
            })
            .collect();
        assert_eq!(all.len(), files.len());
        let mut report = Report::default();
        check_answers(&all, &mut report);
        report.errors
    }

    const EN: (&str, &str, &str) = ("en", "y|yes", "Save into {folder}? [y/N] ");

    #[test]
    fn yes_is_not_the_prompts_no() {
        // Good drafts, and the files as they are.
        assert!(
            answer_errors(&[
                EN,
                ("ru", "д|да", "Сохранить в {folder}? [д/Н] "),
                ("ar", "ن|نعم", "هل تريد الحفظ في {folder}؟ [ن/ل] "),
                // Arabic yes is Urdu's no, but the command line takes only
                // the active language's answers.
                ("ur", "ہ|ہاں|جی", "{folder} میں محفوظ کریں؟ [ہ/ن] "),
                ("tr", "e|evet", "{folder} klasörüne kaydedilsin mi? [e/H] "),
            ])
            .is_empty()
        );
        assert!(run().errors.iter().all(|e| !e.contains("answer for no")));

        // Russian: the prompt's no letter, in either case.
        let errors = answer_errors(&[EN, ("ru", "д|да|н", "Сохранить в {folder}? [д/Н] ")]);
        assert_eq!(
            errors,
            ["cli/ru `cliYes`: `н` is the answer for no in cli/ru `cliPromptSave`"]
        );
        // Russian: a no word the prompt offers.
        let errors = answer_errors(&[EN, ("ru", "да|нет", "Сохранить в {folder}? [да/нет] ")]);
        assert_eq!(errors.len(), 1);
        assert!(errors.iter().all(|e| e.contains("`нет`")));
        // Arabic.
        let errors = answer_errors(&[EN, ("ar", "ن|نعم|ل", "هل تريد الحفظ في {folder}؟ [ن/ل] ")]);
        assert_eq!(errors.len(), 1);
        // Folded as the command line folds: full-width, case, Turkish i.
        assert_eq!(answer_errors(&[("tr", "e|ｈ", "[e/H]")]).len(), 1);
        assert_eq!(answer_errors(&[("xx", "ı", "[e/İ]")]).len(), 1);
        // A translated yes is checked against the English prompt too, which
        // is shown while the translated prompt is an unreviewed draft.
        assert_eq!(
            answer_errors(&[("en", "", "[j/X]"), ("xx", "x", "[x/N]")]),
            ["cli/xx `cliYes`: `x` is the answer for no in cli/en `cliPromptSave`"]
        );
        // A regional variant pairs with its base language's prompt.
        let errors = answer_errors(&[
            EN,
            ("pt", "s|sim", "Salvar em {folder}? [s/X] "),
            ("pt-BR", "s|sim|x", ""),
        ]);
        assert_eq!(
            errors,
            ["cli/pt-BR `cliYes`: `x` is the answer for no in cli/pt `cliPromptSave`"]
        );
    }
}
