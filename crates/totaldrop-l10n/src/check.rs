//! The rules every translation must follow (ADR 0008, "CI denetimleri").
//! `cargo test -p totaldrop-l10n` runs them on the compiled-in files, so CI
//! fails on a broken translation; missing translations are only reported.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::{
    ALWAYS_YES, File, Message, TEMPLATE, catalog, fold_answer, is_hidden, is_yes, layer_tags,
    status,
};

/// Placeholders a translation must insert exactly as the English text does,
/// in every case, and never use to choose a case: dropping, repeating or
/// replacing them could mislead someone on a security screen (a fake code,
/// a different folder).
pub const CRITICAL_PLACEHOLDERS: [&str; 4] = ["code", "words", "folder", "name"];

/// English answers that mean no; a translation's yes-words may not be one.
/// Each language's own come from `noWords` in `l10n/status.json` and from
/// its save prompt (see [`prompt_answers`]).
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
    check_no_words(all, &status.no_words, &mut report);
    check_answers(all, &status.no_words, &mut report);
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

/// The answers a save prompt offers for yes and for no: the two sides of
/// the last ASCII `[yes/no]` in it (`[y/N]` → `y`, `N`; `[ن/ل]` → `ن`,
/// `ل`).
fn prompt_answers(raw: &str) -> Option<(&str, &str)> {
    let open = raw.rfind('[')?;
    let inside = raw.get(open.saturating_add(1)..)?;
    let close = inside.find(']')?;
    let (yes, no) = inside.get(..close)?.split_once('/')?;
    Some((yes.trim(), no.trim()))
}

fn check_prompt(raw: &str, at: &str, report: &mut Report) {
    let Some((yes, no)) = prompt_answers(raw) else {
        report.errors.push(format!(
            "{at}: shows no ASCII `[yes/no]` answers (square brackets around two \
             answers split by `/`), so `cliYes` cannot be checked against them"
        ));
        return;
    };
    let mut fault = |why: String| report.errors.push(format!("{at}: {why}"));
    if fold_answer(yes).is_empty() {
        fault("the answer for yes in `[yes/no]` is empty".to_owned());
    }
    let folded = fold_answer(no);
    if folded.is_empty() {
        fault("the answer for no in `[yes/no]` is empty".to_owned());
    } else if folded.contains('/') {
        fault(format!(
            "the answer for no, `{no}`, contains `/`: `[yes/no]` takes two answers"
        ));
    } else if ALWAYS_YES.contains(&folded.as_str()) {
        fault(format!(
            "the answer for no, `{no}`, always means yes on the command line (y, yes)"
        ));
    }
}

/// Checks each language's `noWords` in `l10n/status.json`: every language
/// with command-line strings has some, and none is empty or always means
/// yes.
fn check_no_words(all: &[File], no_words: &HashMap<String, Vec<String>>, report: &mut Report) {
    let tags: BTreeSet<&str> = all
        .iter()
        .filter(|f| f.domain == "cli" && f.locale != TEMPLATE)
        .map(|f| f.locale.as_str())
        .collect();
    for tag in tags {
        let words = no_words.get(tag).map(Vec::as_slice).unwrap_or_default();
        if words.is_empty() {
            report.errors.push(format!(
                "status.json: `{tag}` has no `noWords` (the answers that mean no)"
            ));
        }
        for word in words {
            let w = fold_answer(word);
            if w.is_empty() {
                report
                    .errors
                    .push(format!("status.json: `{tag}` `noWords` has an empty word"));
            } else if ALWAYS_YES.contains(&w.as_str()) {
                report.errors.push(format!(
                    "status.json: `{tag}` `noWords`: `{word}` always means yes on the command line"
                ));
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

/// Checks the answers in `cliYes` against what means no for the person
/// answering, all folded as the command line folds answers
/// ([`fold_answer`]):
///
/// - no yes answer is one of the language's `noWords`
///   (`l10n/status.json`), or a fallback language's;
/// - no yes answer is the answer for no in a save prompt shown with it;
/// - the prompt's answer for yes is accepted, by `cliYes` or
///   [`ALWAYS_YES`].
///
/// The first two are deliberately conservative: for each language they
/// pair every `cliYes` layer (the language, its fallbacks and English, in
/// [`CLI_DOMAINS`]) with every `cliPromptSave` layer, because an
/// unreviewed critical key is skipped on its own, and they may reject a
/// combination the command line cannot actually show. The third pairs only
/// the layers the command line shows once everything is reviewed: across
/// layers it would fail every language (English `cliYes` does not take
/// `e`), and a mismatch there only declines a transfer.
///
/// The command line takes only the active language's answers and
/// [`ALWAYS_YES`], so another language's no-words do not matter (Arabic
/// yes `ن` is Urdu's no).
fn check_answers(all: &[File], no_words: &HashMap<String, Vec<String>>, report: &mut Report) {
    let tags: BTreeSet<&str> = all.iter().map(|f| f.locale.as_str()).collect();
    let mut errors = BTreeSet::new();
    // (file, word) → the first language that reported it, so a base
    // language's word is reported once, not once per variant.
    let mut means_no: BTreeMap<(String, String), String> = BTreeMap::new();
    for tag in tags {
        let language_no: Vec<(String, String)> = layer_tags(tag)
            .into_iter()
            .filter_map(|t| no_words.get(&t).map(|w| (t, w)))
            .flat_map(|(t, words)| words.iter().map(move |w| (fold_answer(w), t.clone())))
            .collect();
        let prompts = layers(all, tag, "cliPromptSave");
        let yeses = layers(all, tag, "cliYes");
        for (yes_file, yes) in &yeses {
            let at = format!("{}/{} `cliYes`", yes_file.domain, yes_file.locale);
            for word in yes.split('|') {
                let w = fold_answer(word);
                // check_yes reports these already.
                if w.is_empty() || NO_WORDS.contains(&w.as_str()) {
                    continue;
                }
                if let Some((_, t)) = language_no.iter().find(|(no, _)| *no == w) {
                    means_no
                        .entry((at.clone(), word.to_owned()))
                        .or_insert_with(|| {
                            format!(
                                "{at}: `{word}` means no in {t} (`noWords` in l10n/status.json)"
                            )
                        });
                }
                for (prompt_file, prompt) in &prompts {
                    let Some((_, no)) = prompt_answers(prompt) else {
                        continue;
                    };
                    if w == fold_answer(no) {
                        errors.insert(format!(
                            "{at}: `{word}` is the answer for no in {}/{} `cliPromptSave`",
                            prompt_file.domain, prompt_file.locale
                        ));
                    }
                }
            }
        }
        if let (Some((yes_file, yes)), Some((prompt_file, prompt))) =
            (yeses.first(), prompts.first())
            && let Some((offered, _)) = prompt_answers(prompt)
            && !fold_answer(offered).is_empty()
            && !is_yes(offered, yes)
        {
            errors.insert(format!(
                "{}/{} `cliPromptSave`: offers `{offered}` for yes, which {}/{} `cliYes` does not accept",
                prompt_file.domain, prompt_file.locale, yes_file.domain, yes_file.locale
            ));
        }
    }
    errors.extend(means_no.into_values());
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
    fn prompt_answers_and_faults() {
        let parse = prompt_answers;
        assert_eq!(parse("Save into {folder}? [y/N] "), Some(("y", "N")));
        assert_eq!(parse("هل تريد الحفظ في {folder}؟ [ن/ل] "), Some(("ن", "ل")));
        assert_eq!(parse("[a] b [ d / Н ]"), Some(("d", "Н")));
        assert_eq!(parse("Save into {folder}? "), None);
        assert_eq!(parse("Save [y/N"), None);
        assert_eq!(parse("保存到 {folder}？［y／N］"), None);
        let errors = |raw: &str| {
            let mut r = Report::default();
            check_prompt(raw, "x", &mut r);
            r.errors
        };
        assert!(errors("Сохранить в {folder}? [д/Н] ").is_empty());
        let why = |raw: &str, part: &str| {
            let e = errors(raw);
            assert_eq!(e.len(), 1, "{raw}: {e:?}");
            assert!(e.iter().all(|e| e.contains(part)), "{raw}: {e:?}");
        };
        why("Save into {folder}? ", "no ASCII `[yes/no]` answers");
        why("[j/]", "the answer for no in `[yes/no]` is empty");
        why("[ /n]", "the answer for yes in `[yes/no]` is empty");
        why("[j/a/n]", "`a/n`, contains `/`");
        // English y and yes always mean yes.
        why("[j/Y]", "`Y`, always means yes");
        why("[j/ｙｅｓ]", "always means yes");
    }

    /// An ARB file for the tests: (domain, locale, `cliYes`,
    /// `cliPromptSave`); an empty text leaves the key out.
    type Arb<'a> = (&'static str, &'a str, &'a str, &'a str);

    fn files(arbs: &[Arb<'_>]) -> Vec<File> {
        let all: Vec<File> = arbs
            .iter()
            .filter_map(|(domain, locale, yes, prompt)| {
                let mut json = serde_json::Map::new();
                json.insert("@@locale".into(), (*locale).into());
                if !yes.is_empty() {
                    json.insert("cliYes".into(), (*yes).into());
                }
                if !prompt.is_empty() {
                    json.insert("cliPromptSave".into(), (*prompt).into());
                }
                let text = serde_json::Value::Object(json).to_string();
                crate::parse_file(domain, &text).ok()
            })
            .collect();
        assert_eq!(all.len(), arbs.len());
        all
    }

    fn no_words(list: &[(&str, &str)]) -> HashMap<String, Vec<String>> {
        list.iter()
            .map(|(tag, words)| {
                let words = words.split('|').map(str::to_owned).collect();
                ((*tag).to_owned(), words)
            })
            .collect()
    }

    /// Errors from [`check_answers`].
    fn answers(arbs: &[Arb<'_>], no_words: &HashMap<String, Vec<String>>) -> Vec<String> {
        let mut report = Report::default();
        check_answers(&files(arbs), no_words, &mut report);
        report.errors
    }

    /// Errors from [`check_answers`] without `noWords`, for the prompt's
    /// answers alone.
    fn prompt_errors(arbs: &[Arb<'_>]) -> Vec<String> {
        answers(arbs, &HashMap::new())
    }

    const EN: Arb<'static> = ("cli", "en", "y|yes", "Save into {folder}? [y/N] ");

    #[test]
    fn no_words_in_status() {
        let all = files(&[
            ("cli", "ru", "д|да", ""),
            ("cli", "de", "j|ja", ""),
            ("app", "fr", "", ""),
            EN,
        ]);
        let mut r = Report::default();
        check_no_words(
            &all,
            &no_words(&[("ru", "нет|не|н"), ("de", "nein||Y")]),
            &mut r,
        );
        assert_eq!(
            r.errors,
            [
                "status.json: `de` `noWords` has an empty word",
                "status.json: `de` `noWords`: `Y` always means yes on the command line",
            ]
        );
        let mut r = Report::default();
        check_no_words(&all, &no_words(&[("ru", "нет")]), &mut r);
        assert_eq!(
            r.errors,
            ["status.json: `de` has no `noWords` (the answers that mean no)"]
        );
        // Every language has them, and every current file passes.
        assert!(run().errors.is_empty());
    }

    #[test]
    fn yes_is_not_a_no_word() {
        // The issue's examples, against the no-words in l10n/status.json.
        let seeded = &status().no_words;
        for (tag, yes, prompt, word) in [
            ("ru", "д|да|нет", "[д/Н]", "нет"),
            ("de", "j|ja|nein", "[j/N]", "nein"),
            ("ar", "ن|نعم|لا", "[ن/ل]", "لا"),
            ("fa", "ب|بله|نه", "[ب/خ]", "نه"),
            ("el", "ν|ναι|όχι", "[ν/Ο]", "όχι"),
            // Folded: HAYIR is hayır.
            ("tr", "e|evet|HAYIR", "[e/H]", "HAYIR"),
        ] {
            let arb: Arb<'_> = ("cli", tag, yes, prompt);
            assert_eq!(
                answers(&[arb], seeded),
                [format!(
                    "cli/{tag} `cliYes`: `{word}` means no in {tag} (`noWords` in l10n/status.json)"
                )]
            );
            // The same file without the no-word passes.
            let good = yes.rsplit_once('|').map(|(g, _)| g).unwrap_or_default();
            assert!(answers(&[("cli", tag, good, prompt)], seeded).is_empty());
        }
        // A fallback's no-words count too.
        let words = no_words(&[("pt", "não|nao")]);
        assert_eq!(
            answers(
                &[
                    ("cli", "pt", "s|sim", "[s/N]"),
                    ("cli", "pt-BR", "s|sim|nao", "[s/N]")
                ],
                &words
            ),
            ["cli/pt-BR `cliYes`: `nao` means no in pt (`noWords` in l10n/status.json)"]
        );
        // A base language's word is reported once, not once per variant.
        let words = no_words(&[("pt", "não"), ("pt-BR", "não"), ("pt-PT", "não")]);
        assert_eq!(
            answers(
                &[
                    ("cli", "pt", "s|sim|não", "[s/N]"),
                    ("cli", "pt-BR", "", ""),
                    ("cli", "pt-PT", "", ""),
                ],
                &words
            ),
            ["cli/pt `cliYes`: `não` means no in pt (`noWords` in l10n/status.json)"]
        );
    }

    #[test]
    fn yes_is_not_the_prompts_no() {
        // Good drafts.
        assert!(
            prompt_errors(&[
                EN,
                ("cli", "ru", "д|да", "Сохранить в {folder}? [д/Н] "),
                ("cli", "ar", "ن|نعم", "هل تريد الحفظ في {folder}؟ [ن/ل] "),
                // Arabic yes is Urdu's no, but the command line takes only
                // the active language's answers.
                ("cli", "ur", "ہ|ہاں|جی", "{folder} میں محفوظ کریں؟ [ہ/ن] "),
                (
                    "cli",
                    "tr",
                    "e|evet",
                    "{folder} klasörüne kaydedilsin mi? [e/H] "
                ),
            ])
            .is_empty()
        );
        // Russian: the prompt's no letter, in either case.
        assert_eq!(
            prompt_errors(&[EN, ("cli", "ru", "д|да|н", "Сохранить в {folder}? [д/Н] ")]),
            ["cli/ru `cliYes`: `н` is the answer for no in cli/ru `cliPromptSave`"]
        );
        // Russian: a no word the prompt offers.
        assert_eq!(
            prompt_errors(&[
                EN,
                ("cli", "ru", "да|нет", "Сохранить в {folder}? [да/нет] ")
            ]),
            ["cli/ru `cliYes`: `нет` is the answer for no in cli/ru `cliPromptSave`"]
        );
        // Arabic.
        assert_eq!(
            prompt_errors(&[
                EN,
                ("cli", "ar", "ن|نعم|ل", "هل تريد الحفظ في {folder}؟ [ن/ل] ")
            ]),
            ["cli/ar `cliYes`: `ل` is the answer for no in cli/ar `cliPromptSave`"]
        );
        // Folded as the command line folds: full-width, case, Turkish i.
        assert_eq!(
            prompt_errors(&[("cli", "tr", "e|ｈ", "[e/H]")]),
            ["cli/tr `cliYes`: `ｈ` is the answer for no in cli/tr `cliPromptSave`"]
        );
        assert_eq!(
            prompt_errors(&[("cli", "xx", "e|ı", "[e/İ]")]),
            ["cli/xx `cliYes`: `ı` is the answer for no in cli/xx `cliPromptSave`"]
        );
        // Empty answers and English no-words are check_yes's to report.
        assert!(prompt_errors(&[("cli", "xx", "x||n", "[x/N]")]).is_empty());
        // A translated yes is checked against the English prompt too, which
        // is shown while the translated prompt is an unreviewed draft.
        assert_eq!(
            prompt_errors(&[("cli", "en", "", "[j/X]"), ("cli", "xx", "x", "[x/N]")]),
            ["cli/xx `cliYes`: `x` is the answer for no in cli/en `cliPromptSave`"]
        );
        // A regional variant pairs with its base language's prompt.
        assert_eq!(
            prompt_errors(&[
                EN,
                ("cli", "pt", "s|sim", "Salvar em {folder}? [s/X] "),
                ("cli", "pt-BR", "s|sim|x", ""),
            ]),
            ["cli/pt-BR `cliYes`: `x` is the answer for no in cli/pt `cliPromptSave`"]
        );
        // The command line falls back to the app strings.
        assert_eq!(
            prompt_errors(&[("app", "xx", "", "[x/K]"), ("cli", "xx", "x|k", "")]),
            ["cli/xx `cliYes`: `k` is the answer for no in app/xx `cliPromptSave`"]
        );
        // One error per file pair, however many languages fall back to it.
        assert_eq!(
            prompt_errors(&[
                ("cli", "pt", "s|x", "[s/X]"),
                ("cli", "pt-BR", "", ""),
                ("cli", "pt-PT", "", ""),
            ]),
            ["cli/pt `cliYes`: `x` is the answer for no in cli/pt `cliPromptSave`"]
        );
    }

    #[test]
    fn prompts_yes_is_accepted() {
        assert_eq!(
            prompt_errors(&[("cli", "ru", "да", "[д/Н]")]),
            ["cli/ru `cliPromptSave`: offers `д` for yes, which cli/ru `cliYes` does not accept"]
        );
        // Folded, and English y always works.
        assert!(prompt_errors(&[("cli", "ru", "Д|да", "[д/Н]")]).is_empty());
        assert!(prompt_errors(&[("cli", "xx", "ja", "[y/N]")]).is_empty());
        // Only the layers shown once everything is reviewed: English yes
        // does not take a translated prompt's `s`.
        assert!(
            prompt_errors(&[
                EN,
                ("cli", "pt", "s|sim", "[s/N]"),
                ("cli", "pt-BR", "", "[s/N]"),
            ])
            .is_empty()
        );
        // The base language's yes, for a variant's prompt.
        assert_eq!(
            prompt_errors(&[
                ("cli", "pt", "s|sim", "[s/N]"),
                ("cli", "pt-BR", "", "[x/N]")
            ]),
            [
                "cli/pt-BR `cliPromptSave`: offers `x` for yes, which cli/pt `cliYes` does not accept"
            ]
        );
    }
}
