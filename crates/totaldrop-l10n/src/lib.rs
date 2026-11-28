// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! TotalDrop's interface text for the Rust programs (the Linux app and the
//! command line): the strings in `l10n/`, the language the system asks
//! for, and numbers, sizes, percentages and plurals in that language and
//! region.
//!
//! The rules are in ADR 0008 (`docs/kararlar/0008-diller-ve-bolgeler.md`).
//! The ARB files are compiled in, so a program never reads translations
//! from disk.

pub mod check;
pub mod locale;
pub mod message;

use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

use icu_decimal::DecimalFormatter;
use icu_decimal::input::Decimal;
use icu_locale::Locale;
use icu_plurals::{PluralCategory, PluralRules};
use icu_properties::CodePointMapData;
use icu_properties::props::GeneralCategory;
use unicode_normalization::UnicodeNormalization as _;

pub use message::{Arg, Message};

mod files {
    include!(concat!(env!("OUT_DIR"), "/files.rs"));
}

/// The template language.
pub const TEMPLATE: &str = "en";

/// Pseudo-locales for testing layouts (ADR 0008), used only when asked for
/// by name: `en-XA` accents every letter and makes text about 40% longer;
/// `ar-XB` is English laid out right to left.
pub const PSEUDO: [&str; 2] = ["en-XA", "ar-XB"];

/// Which set of strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    /// The apps (`l10n/app`), shared with Android.
    App,
    /// The command line (`l10n/cli`); falls back to the app strings.
    Cli,
}

impl Domain {
    fn name(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::Cli => "cli",
        }
    }
}

/// One parsed ARB file.
#[derive(Debug)]
pub(crate) struct File {
    pub(crate) domain: &'static str,
    pub(crate) locale: String,
    pub(crate) messages: HashMap<String, Result<Message, message::ParseError>>,
    pub(crate) raw: HashMap<String, String>,
}

/// Every compiled-in ARB file, parsed once, and the files that could not
/// be read at all (bad JSON, no `@@locale`, a value that is not text, a
/// key twice), which [`check::run`] reports.
pub(crate) struct Catalog {
    pub(crate) files: Vec<File>,
    pub(crate) broken: Vec<String>,
}

pub(crate) fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut files = Vec::new();
        let mut broken = Vec::new();
        for (i, (domain, json)) in files::FILES.iter().enumerate() {
            match parse_file(domain, json) {
                Ok(f) => files.push(f),
                Err(e) => broken.push(format!("{domain} file #{i}: {e}")),
            }
        }
        Catalog { files, broken }
    })
}

fn parse_file(domain: &'static str, json: &str) -> Result<File, String> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let object = value.as_object().ok_or("not a JSON object")?;
    let locale = object
        .get("@@locale")
        .and_then(serde_json::Value::as_str)
        .ok_or("no @@locale")?
        .replace('_', "-");
    let mut messages = HashMap::new();
    let mut raw = HashMap::new();
    for (key, v) in object {
        if key.starts_with('@') {
            continue;
        }
        let s = v
            .as_str()
            .ok_or_else(|| format!("{locale} `{key}` is not text"))?;
        // serde_json keeps the last of two equal keys; refuse instead.
        if json.matches(&format!("\"{key}\":")).count() > 1 {
            return Err(format!("{locale} `{key}` appears twice"));
        }
        messages.insert(key.clone(), Message::parse(s));
        raw.insert(key.clone(), s.to_owned());
    }
    Ok(File {
        domain,
        locale,
        messages,
        raw,
    })
}

/// `l10n/status.json`: which languages were reviewed, which keys are still
/// drafts, which keys must never be shown as a draft, and the answers that
/// mean no in each language. One status per language covers its app and
/// command-line strings.
#[derive(Debug, Default)]
pub(crate) struct Status {
    pub(crate) critical: BTreeSet<String>,
    pub(crate) reviewed: HashMap<String, bool>,
    pub(crate) draft: HashMap<String, BTreeSet<String>>,
    /// Per language, the answers that mean no (`noWords`, separated by
    /// `|`), as written; a translation's `cliYes` may not be one.
    pub(crate) no_words: HashMap<String, Vec<String>>,
}

impl Status {
    /// Whether `key` in language `tag` is a draft nobody has reviewed.
    fn is_draft(&self, tag: &str, key: &str) -> bool {
        tag != TEMPLATE
            && (!self.reviewed.get(tag).copied().unwrap_or(false)
                || self.draft.get(tag).is_some_and(|d| d.contains(key)))
    }
}

pub(crate) fn status() -> &'static Status {
    static STATUS: OnceLock<Status> = OnceLock::new();
    STATUS.get_or_init(|| {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(files::STATUS) else {
            return Status::default();
        };
        let strings = |v: Option<&serde_json::Value>| -> BTreeSet<String> {
            v.and_then(serde_json::Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|s| s.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default()
        };
        let mut status = Status {
            critical: strings(value.get("critical")),
            ..Status::default()
        };
        if let Some(languages) = value
            .get("languages")
            .and_then(serde_json::Value::as_object)
        {
            for (tag, info) in languages {
                let reviewed = info
                    .get("reviewed")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                status.reviewed.insert(tag.clone(), reviewed);
                status.draft.insert(tag.clone(), strings(info.get("draft")));
                if let Some(words) = info.get("noWords").and_then(serde_json::Value::as_str) {
                    status
                        .no_words
                        .insert(tag.clone(), words.split('|').map(str::to_owned).collect());
                }
            }
        }
        status
    })
}

/// Whether `key` must never be shown as an unreviewed draft (for apps
/// that format messages themselves, such as Android's generated classes).
#[must_use]
pub fn is_critical(key: &str) -> bool {
    status().critical.contains(key)
}

/// Whether `key` in language `tag` (`pt-BR`) is a draft nobody has
/// reviewed.
#[must_use]
pub fn is_draft(tag: &str, key: &str) -> bool {
    status().is_draft(tag, key)
}

/// Whether an app that formats messages itself (Android) must show `key`
/// in English for language `tag`: the layer the text would come from (the
/// language or the first fallback that has the key, as generated classes
/// inherit) holds an unreviewed draft of a safety-critical string.
#[must_use]
pub fn needs_english(tag: &str, key: &str) -> bool {
    if !is_critical(key) {
        return false;
    }
    for t in layer_tags(tag) {
        if t == TEMPLATE {
            return false;
        }
        let has = catalog()
            .files
            .iter()
            .any(|f| f.domain == "app" && f.locale == t && f.messages.contains_key(key));
        if has {
            return status().is_draft(&t, key);
        }
    }
    false
}

/// Whether any text in language `tag`, or in a language it falls back to,
/// is an unreviewed draft.
#[must_use]
pub fn has_drafts(tag: &str) -> bool {
    layer_tags(tag).iter().any(|t| {
        t != TEMPLATE
            && (!status().reviewed.get(t).copied().unwrap_or(false)
                || status().draft.get(t).is_some_and(|d| !d.is_empty()))
            && catalog().files.iter().any(|f| f.locale == *t)
    })
}

/// Every language with strings in `domain`.
#[must_use]
pub fn available(domain: Domain) -> Vec<String> {
    let mut tags: Vec<String> = catalog()
        .files
        .iter()
        .filter(|f| f.domain == domain.name())
        .map(|f| f.locale.clone())
        .collect();
    tags.sort();
    tags
}

/// Plurals, numbers and direction for one language and region, with or
/// without translations for it.
pub struct Formats {
    rtl: bool,
    numeric: Locale,
    plurals: Option<PluralRules>,
    decimal: Option<DecimalFormatter>,
}

impl Formats {
    /// Plural rules and direction of `language`, numbers of `numeric` (or
    /// of `language` when `None`; `Locale::UNKNOWN` keeps ungrouped POSIX
    /// `C` numbers).
    #[must_use]
    pub fn new(language: &Locale, numeric: Option<&Locale>) -> Self {
        let numeric = numeric.unwrap_or(language);
        let decimal = (*numeric != Locale::UNKNOWN)
            .then(|| DecimalFormatter::try_new(numeric.into(), Default::default()).ok())
            .flatten();
        Self {
            rtl: locale::is_rtl(language),
            numeric: numeric.clone(),
            plurals: PluralRules::try_new_cardinal(language.into()).ok(),
            decimal,
        }
    }

    /// `tenths / 10` with one decimal in the region's format.
    fn tenths(&self, tenths: u64) -> String {
        let mut d = Decimal::from(tenths);
        d.multiply_pow10(-1);
        self.decimal.as_ref().map_or_else(
            || format!("{}.{}", tenths / 10, tenths % 10),
            |f| f.format(&d).to_string(),
        )
    }

    /// `n` as a whole percent in `numeric`'s CLDR standard percent pattern
    /// and digit system ("50%", "%50", "50\u{a0}%", "٥٠٪؜"; #491).
    fn percent(&self, n: u64) -> String {
        let digits = message::Locale::number(self, n);
        let mut steps = locale::chain(&self.numeric);
        // Non-default scripts (be_BY@latin -> be-Latn-BY, ku-Arab, zgh-Tfng)
        // stop before <lang> in ICU4X's fallbacker; append <lang> last so
        // regional overrides (ku-IQ, de-CH, es-419) still win first.
        if !self.numeric.id.language.is_unknown() {
            let lang = self.numeric.id.language.to_string();
            if !steps.contains(&lang) {
                steps.push(lang);
            }
        }
        match message::Locale::number(self, 0).chars().next() {
            // Eastern Arabic-Indic digits (arab: ar-EG, ar-SA, ckb, sd, ...).
            Some('\u{0660}') => {
                if steps.iter().any(|s| s == "ckb") {
                    format!("{digits}\u{00A0}\u{066A}")
                } else {
                    format!("{digits}\u{066A}\u{061C}")
                }
            }
            // Extended Arabic-Indic digits (arabext: fa, ps, ks, ur-IN,
            // *-u-nu-arabext); CLDR's arabext root uses ٪, overridden in ur.
            Some('\u{06F0}') => {
                if steps.iter().any(|s| s == "ur") {
                    format!("{digits}%")
                } else {
                    format!("{digits}\u{066A}")
                }
            }
            _ => {
                for step in &steps {
                    match step.as_str() {
                        "blo" | "eu" => return format!("%\u{00A0}{digits}"),
                        "ku-IQ" | "de-CH" | "de-LI" | "fr-CH" | "es-419" | "es-US" => {
                            return format!("{digits}%");
                        }
                        "tr" | "ku" => return format!("%{digits}"),
                        "ar" => return format!("{digits}\u{200E}%\u{200E}"),
                        "oc" => return format!("{digits}\u{202F}%"),
                        "bas" | "be" | "br" | "bua" | "ca" | "ce" | "cs" | "cv" | "da" | "de"
                        | "dsb" | "dua" | "dz" | "en-AT" | "en-DE" | "en-DK" | "en-FI"
                        | "en-NO" | "en-SE" | "es" | "fi" | "fo" | "fr" | "gl" | "gsw" | "hr"
                        | "hsb" | "kl" | "ksh" | "lb" | "lt" | "mk" | "nb" | "nds" | "nn"
                        | "no" | "qu" | "rn" | "ro" | "ru" | "se" | "sk" | "sl" | "smn" | "sv"
                        | "tk" | "tt" | "zgh" => {
                            return format!("{digits}\u{00A0}%");
                        }
                        _ => {}
                    }
                }
                format!("{digits}%")
            }
        }
    }
}

impl message::Locale for Formats {
    fn plural(&self, n: u64) -> &'static str {
        match self.plurals.as_ref().map(|p| p.category_for(n)) {
            Some(PluralCategory::Zero) => "zero",
            Some(PluralCategory::One) => "one",
            Some(PluralCategory::Two) => "two",
            Some(PluralCategory::Few) => "few",
            Some(PluralCategory::Many) => "many",
            _ => "other",
        }
    }

    fn number(&self, n: u64) -> String {
        self.decimal.as_ref().map_or_else(
            || n.to_string(),
            |f| f.format(&Decimal::from(n)).to_string(),
        )
    }

    fn text(&self, s: &str, out: &mut String) {
        // In a right-to-left sentence, isolate inserted text (a code, a
        // file name) so its own direction cannot reorder the sentence.
        if self.rtl {
            out.push('\u{2068}');
            out.push_str(s);
            out.push('\u{2069}');
        } else {
            out.push_str(s);
        }
    }
}

/// The strings and formats for one language and region.
pub struct Strings {
    tag: String,
    draft: bool,
    /// Lookup order: this language, then the fallbacks, then English.
    layers: Vec<HashMap<String, Message>>,
    formats: Formats,
}

impl std::fmt::Debug for Strings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Strings")
            .field("tag", &self.tag)
            .finish_non_exhaustive()
    }
}

/// The tags to look in for `tag`: itself, then shorter forms (`pt-BR` →
/// `pt`), but never across a script (`zh-Hant` does not fall back to
/// Simplified `zh`, nor `sr-Latn` to Cyrillic `sr`), then English. This is
/// how Flutter's generated classes inherit too.
fn layer_tags(tag: &str) -> Vec<String> {
    let mut out = vec![tag.to_owned()];
    let mut current = tag.to_owned();
    while let Some((parent, last)) = current.rsplit_once('-') {
        let is_script = last.len() == 4 && last.chars().all(|c| c.is_ascii_alphabetic());
        if is_script {
            break;
        }
        current = parent.to_owned();
        out.push(current.clone());
    }
    if !out.iter().any(|t| t == TEMPLATE) {
        out.push(TEMPLATE.to_owned());
    }
    out
}

impl Strings {
    /// The strings for the system's language and region (see
    /// [`locale::System`]).
    #[must_use]
    pub fn system(domain: Domain) -> Self {
        let system = locale::System::from_env();
        Self::new(domain, &system.messages, system.numeric.as_ref())
    }

    /// The strings for the best match of `preferences`, with numbers in
    /// `numeric` (or the chosen language when `None`). The command line
    /// keeps right-to-left languages in English, since terminals do not
    /// lay out bidirectional text reliably.
    #[must_use]
    pub fn new(domain: Domain, preferences: &[Locale], numeric: Option<&Locale>) -> Self {
        if let Some(first) = preferences.first() {
            let tag = first.to_string();
            if PSEUDO.contains(&tag.as_str()) {
                return Self::pseudo(domain, &tag, numeric);
            }
        }
        let own = available(domain);
        let app = available(Domain::App);
        let candidates: Vec<&str> = own.iter().chain(app.iter()).map(String::as_str).collect();
        let mut tag = locale::negotiate(preferences, &candidates)
            .unwrap_or(TEMPLATE)
            .to_owned();
        let mut chosen = locale::parse(&tag).unwrap_or(Locale::UNKNOWN);
        if domain == Domain::Cli && locale::is_rtl(&chosen) {
            tag = TEMPLATE.to_owned();
            chosen = locale::parse(TEMPLATE).unwrap_or(Locale::UNKNOWN);
        }
        let status = status();
        let domains: &[&str] = match domain {
            Domain::App => &["app"],
            Domain::Cli => &["cli", "app"],
        };
        let mut layers = Vec::new();
        let mut draft = false;
        for t in layer_tags(&tag) {
            for d in domains {
                let Some(file) = catalog()
                    .files
                    .iter()
                    .find(|f| f.domain == *d && f.locale == t)
                else {
                    continue;
                };
                let mut layer = HashMap::new();
                for (k, m) in &file.messages {
                    let Ok(m) = m else { continue };
                    if status.is_draft(&t, k) {
                        // Safety-critical text is never shown as a draft:
                        // it stays English until a person has reviewed it.
                        if status.critical.contains(k) {
                            continue;
                        }
                        draft = true;
                    }
                    layer.insert(k.clone(), m.clone());
                }
                layers.push(layer);
            }
        }
        Self {
            draft,
            formats: Formats::new(&chosen, numeric),
            layers,
            tag,
        }
    }

    fn pseudo(domain: Domain, tag: &str, numeric: Option<&Locale>) -> Self {
        let en: Vec<Locale> = locale::parse(TEMPLATE).into_iter().collect();
        let base = Self::new(domain, &en, numeric);
        let rtl = tag == "ar-XB";
        let layers = base
            .layers
            .iter()
            .map(|layer| {
                layer
                    .iter()
                    .map(|(k, m)| {
                        let m = if rtl {
                            m.clone()
                        } else {
                            m.map_text(&accented)
                        };
                        (k.clone(), m)
                    })
                    .collect()
            })
            .collect();
        let mut formats = base.formats;
        formats.rtl = rtl;
        Self {
            tag: tag.to_owned(),
            draft: false,
            layers,
            formats,
        }
    }

    /// The chosen language, as a BCP 47 tag (`pt-BR`).
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Whether the language is written right to left.
    #[must_use]
    pub fn is_rtl(&self) -> bool {
        self.formats.rtl
    }

    /// Whether some of the text shown, in this language or a fallback, is a
    /// translation nobody has reviewed yet.
    #[must_use]
    pub fn is_draft(&self) -> bool {
        self.draft
    }

    /// Every key this language can show, translated or from English.
    #[must_use]
    pub fn keys(&self) -> BTreeSet<&str> {
        self.layers
            .iter()
            .flat_map(|l| l.keys().map(String::as_str))
            .collect()
    }

    /// The message `key` with `args`. An unknown key comes back as itself,
    /// so a mistake is visible.
    #[must_use]
    pub fn get(&self, key: &str, args: &[(&str, Arg<'_>)]) -> String {
        self.layers
            .iter()
            .find_map(|l| l.get(key))
            .map_or_else(|| key.to_owned(), |m| m.format(args, &self.formats))
    }

    /// `n` with the region's digits and grouping.
    #[must_use]
    pub fn number(&self, n: u64) -> String {
        message::Locale::number(&self.formats, n)
    }

    /// The whole percent of `done` in `total`, rounded down so 100 means
    /// finished, in the region's percent format ("50%", "%50", "50\u{a0}%";
    /// #491).
    #[must_use]
    pub fn percent(&self, done: u64, total: u64) -> String {
        self.formats.percent(whole_percent(done, total))
    }

    /// A size in bytes, in SI units ("1.5 MB", "1,5 MB", "12 MB"): one
    /// decimal below 10 of a unit, whole numbers above, and the next unit
    /// once the rounded value reaches 1000.
    #[must_use]
    pub fn bytes(&self, n: u64) -> String {
        const KEYS: [&str; 5] = ["sizeKB", "sizeMB", "sizeGB", "sizeTB", "sizePB"];
        if n < 1000 {
            return self.get("sizeBytes", &[("value", Arg::Text(&self.number(n)))]);
        }
        let n = u128::from(n);
        let mut unit = 0usize;
        let mut divisor: u128 = 1000;
        loop {
            // Tenths and whole units, each rounded once, half up.
            let tenths = n
                .saturating_mul(10)
                .saturating_add(divisor / 2)
                .checked_div(divisor)
                .unwrap_or(0);
            let whole = n
                .saturating_add(divisor / 2)
                .checked_div(divisor)
                .unwrap_or(0);
            let last = unit.saturating_add(1) >= KEYS.len();
            if whole < 1000 || last {
                let key = KEYS.get(unit).copied().unwrap_or("sizePB");
                let value = if tenths < 100 {
                    self.formats
                        .tenths(u64::try_from(tenths).unwrap_or(u64::MAX))
                } else {
                    self.number(u64::try_from(whole).unwrap_or(u64::MAX))
                };
                return self.get(key, &[("value", Arg::Text(&value))]);
            }
            unit = unit.saturating_add(1);
            divisor = divisor.saturating_mul(1000);
        }
    }
}

/// The whole percent of `done` in `total`, rounded down with integer
/// arithmetic so that 100 means finished (#491, matching Android's
/// `wholePercent` in #483).
#[must_use]
pub fn whole_percent(done: u64, total: u64) -> u64 {
    if total == 0 {
        return 0;
    }
    let pct = u128::from(done)
        .saturating_mul(100)
        .checked_div(u128::from(total))
        .unwrap_or(0)
        .min(100);
    u64::try_from(pct).unwrap_or(100)
}

fn accented(text: &str) -> String {
    let mut out = String::new();
    let mut letters = 0usize;
    for c in text.chars() {
        let a = match c {
            'a' => 'á',
            'e' => 'é',
            'i' => 'î',
            'o' => 'ö',
            'u' => 'ü',
            'c' => 'ç',
            'n' => 'ñ',
            's' => 'š',
            'y' => 'ý',
            'A' => 'Å',
            'E' => 'É',
            'I' => 'Î',
            'O' => 'Ö',
            'U' => 'Ü',
            'C' => 'Ç',
            'N' => 'Ñ',
            'S' => 'Š',
            other => other,
        };
        if c.is_alphabetic() {
            letters = letters.saturating_add(1);
        }
        out.push(a);
    }
    // About 40% longer, the most a translation usually grows.
    for _ in 0..letters.saturating_mul(2) / 5 {
        out.push('·');
    }
    out
}

/// Whether a character is invisible or changes how text around it is laid
/// out: controls (Cc), format characters (Cf) and line or paragraph
/// separators (Zl, Zp), and noncharacters (U+FDD0–U+FDEF and the last two
/// of every plane), which Qt's text box shows as a line break (#326).
/// Zero-width joiner and non-joiner, which Persian and Indic scripts need,
/// are allowed.
#[must_use]
pub fn is_hidden(c: char) -> bool {
    if matches!(c, '\u{200C}' | '\u{200D}') {
        return false;
    }
    let n = u32::from(c);
    if (0xFDD0..=0xFDEF).contains(&n) || n & 0xFFFE == 0xFFFE {
        return true;
    }
    matches!(
        CodePointMapData::<GeneralCategory>::new().get(c),
        GeneralCategory::Control
            | GeneralCategory::Format
            | GeneralCategory::LineSeparator
            | GeneralCategory::ParagraphSeparator
    )
}

/// A name from the other side with every [hidden](is_hidden) character
/// shown as `<U+202E>`, so a name cannot pretend to have another extension
/// (`invoice` U+202E `fdp.exe` reads as "invoiceexe.pdf").
#[must_use]
pub fn reveal_controls(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if is_hidden(c) {
            out.push_str(&format!("<U+{:04X}>", u32::from(c)));
        } else {
            out.push(c);
        }
    }
    out
}

/// A text from the other side (#313) as it is shown, on a terminal or in
/// a window: [hidden](is_hidden) characters are named as
/// [`reveal_controls`] names them, so the text cannot move a terminal's
/// cursor, colour it, or hide its words behind direction marks. Line
/// breaks (`\n` or `\r\n`, which becomes `\n`) and tabs stay. A `\r`
/// anywhere else is named too: it would send the cursor back over the
/// line.
#[must_use]
pub fn reveal_text(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let last = lines.len().saturating_sub(1);
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            // Only a \r that a \n follows ends a line.
            let line = if i < last {
                line.strip_suffix('\r').unwrap_or(line)
            } else {
                line
            };
            line.split('\t')
                .map(reveal_controls)
                .collect::<Vec<_>>()
                .join("\t")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Answers the command line takes as yes in every language, besides the
/// translation's own (`cliYes`).
pub const ALWAYS_YES: [&str; 2] = ["y", "yes"];

/// An answer as the command line compares it: trimmed, with compatibility
/// forms folded (NFKC: a full-width "ｙ", a decomposed "sí") and case
/// folded without regard to language, the dotted and dotless i included,
/// so a Turkish `E` or `EVET` works on any system. [`check`] folds a
/// translation's answers the same way.
#[must_use]
pub fn fold_answer(answer: &str) -> String {
    answer
        .trim()
        .nfkc()
        .collect::<String>()
        .replace(['İ', 'I'], "i")
        .replace('ı', "i")
        .to_lowercase()
}

/// Whether an answer means yes: one of [`ALWAYS_YES`], or one of the
/// translation's `words` (`cliYes`, separated by `|`), compared after
/// [`fold_answer`].
#[must_use]
pub fn is_yes(answer: &str, words: &str) -> bool {
    let answer = fold_answer(answer);
    ALWAYS_YES
        .into_iter()
        .chain(words.split('|'))
        .map(fold_answer)
        .any(|w| !w.is_empty() && w == answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yes_answers() {
        assert!(is_yes("y\n", "e|evet"));
        assert!(is_yes("YES", "e|evet"));
        assert!(is_yes("EVET", "e|evet"));
        assert!(is_yes("E", "e|evet"));
        assert!(!is_yes("", "e|evet"));
        assert!(!is_yes("n", "e|evet"));
        assert!(!is_yes("hayır", "e|evet"));
        assert!(is_yes("ｙ", ""));
        assert!(is_yes("si\u{301}", "sí|s"));
        assert!(is_yes("İ", "i|igen"));
        assert!(!is_yes(" ", "|"));
    }

    fn strings(tag: &str) -> Strings {
        let prefs: Vec<Locale> = locale::parse(tag).into_iter().collect();
        Strings::new(Domain::App, &prefs, None)
    }

    /// The shared vectors in `l10n/tests/messages.json`, which the site's
    /// formatter must pass too.
    #[test]
    // Indexing a serde_json::Value yields Null when absent; it cannot panic.
    #[allow(clippy::indexing_slicing)]
    fn shared_vectors() {
        let text = include_str!("../../../l10n/tests/messages.json");
        let v: serde_json::Value = serde_json::from_str(text).unwrap_or_default();
        let cases = v["cases"].as_array().cloned().unwrap_or_default();
        assert!(cases.len() > 10);
        for case in cases {
            let tag = case["locale"].as_str().unwrap_or_default();
            let lang = locale::parse(tag).unwrap_or(Locale::UNKNOWN);
            let formats = Formats::new(&lang, None);
            let owned: Vec<(String, serde_json::Value)> = case["args"]
                .as_object()
                .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default();
            let args: Vec<(&str, Arg<'_>)> = owned
                .iter()
                .map(|(k, v)| match v.as_u64() {
                    Some(n) => (k.as_str(), Arg::Count(n)),
                    None => (k.as_str(), Arg::Text(v.as_str().unwrap_or_default())),
                })
                .collect();
            let src = case["message"].as_str().unwrap_or_default();
            let got = Message::parse(src).map(|m| m.format(&args, &formats));
            assert_eq!(
                got.as_deref(),
                Ok(case["expect"].as_str().unwrap_or_default()),
                "{tag}: {src}"
            );
        }
        for bad in v["invalid"].as_array().cloned().unwrap_or_default() {
            let src = bad.as_str().unwrap_or_default();
            assert!(Message::parse(src).is_err(), "{src} should be rejected");
        }
    }

    #[test]
    fn english_and_turkish() {
        assert_eq!(strings("en-US").tag(), "en");
        assert_eq!(strings("tr-TR").tag(), "tr");
        assert_eq!(strings("xx").tag(), "en");
        assert!(!strings("tr").is_draft());
        assert!(!strings("en").is_draft());
    }

    #[test]
    fn plurals_follow_the_language() {
        let en = strings("en");
        assert_eq!(
            en.get("sendDoneBody", &[("count", Arg::Count(1))]),
            "The file arrived intact."
        );
        assert_eq!(
            en.get("sendDoneBody", &[("count", Arg::Count(3))]),
            "All 3 files arrived intact."
        );
    }

    #[test]
    fn sizes() {
        let en = strings("en");
        assert_eq!(en.bytes(0), "0 B");
        assert_eq!(en.bytes(999), "999 B");
        assert_eq!(en.bytes(1000), "1.0 kB");
        assert_eq!(en.bytes(1_549_999), "1.5 MB");
        assert_eq!(en.bytes(9_960_000), "10 MB");
        assert_eq!(en.bytes(12_345_678), "12 MB");
        assert_eq!(en.bytes(12_450_000), "12 MB");
        assert_eq!(en.bytes(12_500_000), "13 MB");
        assert_eq!(en.bytes(999_499), "999 kB");
        assert_eq!(en.bytes(999_950), "1.0 MB");
        assert_eq!(en.bytes(999_999_999), "1.0 GB");
        assert_eq!(en.bytes(u64::MAX), "18,447 PB");
        let tr = strings("tr");
        assert_eq!(tr.bytes(1_500_000), "1,5 MB");
    }

    #[test]
    fn layers_do_not_cross_scripts() {
        assert_eq!(layer_tags("pt-BR"), ["pt-BR", "pt", "en"]);
        assert_eq!(layer_tags("zh-Hant"), ["zh-Hant", "en"]);
        assert_eq!(layer_tags("zh-Hant-HK"), ["zh-Hant-HK", "zh-Hant", "en"]);
        assert_eq!(layer_tags("sr-Latn"), ["sr-Latn", "en"]);
        assert_eq!(layer_tags("en"), ["en"]);
    }

    #[test]
    fn numbers_follow_the_region() {
        let prefs: Vec<Locale> = locale::parse("en").into_iter().collect();
        let de = locale::parse("de-DE");
        let s = Strings::new(Domain::App, &prefs, de.as_ref());
        assert_eq!(s.number(1_234_567), "1.234.567");
        assert_eq!(s.bytes(1_500_000), "1,5 MB");
    }

    #[test]
    fn percents_follow_the_region_and_floor() {
        let pair = |ui: &str, region: &str| {
            let prefs: Vec<Locale> = locale::parse(ui).into_iter().collect();
            let numeric = locale::parse(region);
            Strings::new(Domain::App, &prefs, numeric.as_ref())
        };
        // Language and region separated (#491, matching Android's #483):
        assert_eq!(pair("en", "en-US").percent(1, 2), "50%");
        assert_eq!(pair("tr", "en-US").percent(1, 2), "50%");
        assert_eq!(pair("en", "tr-TR").percent(1, 2), "%50");
        assert_eq!(pair("en", "de-DE").percent(1, 2), "50\u{a0}%");
        assert_eq!(pair("de", "de-CH").percent(1, 2), "50%");
        assert_eq!(pair("fr", "fr-FR").percent(1, 2), "50\u{a0}%");
        assert_eq!(pair("fr", "fr-CH").percent(1, 2), "50%");
        assert_eq!(pair("es", "es-ES").percent(1, 2), "50\u{a0}%");
        assert_eq!(pair("es", "es-MX").percent(1, 2), "50%");
        assert_eq!(pair("es", "es-BR").percent(1, 2), "50%");
        assert_eq!(pair("en", "en-DE").percent(1, 2), "50\u{a0}%");
        assert_eq!(pair("en", "en-150").percent(1, 2), "50%");
        assert_eq!(pair("ja", "ja-JP").percent(1, 2), "50%");
        // Right-to-left and non-Latin digit systems, matching CLDR and #489:
        assert_eq!(pair("ar", "ar").percent(1, 2), "50\u{200e}%\u{200e}");
        assert_eq!(pair("ar", "ar-AE").percent(1, 2), "50\u{200e}%\u{200e}");
        assert_eq!(
            pair("ar", "ar-EG").percent(1, 2),
            "\u{0665}\u{0660}\u{066a}\u{061c}"
        );
        assert_eq!(
            pair("en", "ar-EG").percent(1, 2),
            "\u{0665}\u{0660}\u{066a}\u{061c}"
        );
        assert_eq!(pair("ar", "en-US").percent(1, 2), "50%");
        assert_eq!(
            pair("fa", "fa-IR").percent(1, 2),
            "\u{06f5}\u{06f0}\u{066a}"
        );
        assert_eq!(pair("fa", "en-US").percent(1, 2), "50%");
        assert_eq!(
            pair("en", "ps-AF").percent(1, 2),
            "\u{06f5}\u{06f0}\u{066a}"
        );
        assert_eq!(
            pair("en", "ks-IN").percent(1, 2),
            "\u{06f5}\u{06f0}\u{066a}"
        );
        assert_eq!(pair("en", "ur-PK").percent(1, 2), "50%");
        assert_eq!(pair("en", "ur-IN").percent(1, 2), "\u{06f5}\u{06f0}%");
        assert_eq!(
            pair("en", "ur-u-nu-arabext").percent(1, 2),
            "\u{06f5}\u{06f0}%"
        );
        assert_eq!(pair("bn", "bn-BD").percent(1, 2), "\u{09eb}\u{09e6}%");
        assert_eq!(pair("en", "eu-ES").percent(1, 2), "%\u{a0}50");
        assert_eq!(pair("en", "blo").percent(1, 2), "%\u{a0}50");
        assert_eq!(pair("en", "oc-FR").percent(1, 2), "50\u{202f}%");
        assert_eq!(pair("en", "bas").percent(1, 2), "50\u{a0}%");
        assert_eq!(pair("en", "bua").percent(1, 2), "50\u{a0}%");
        // Non-default scripts keep their base language's CLDR pattern while
        // still respecting regional overrides:
        assert_eq!(pair("en", "be-Latn-BY").percent(1, 2), "50\u{a0}%");
        assert_eq!(pair("en", "ku-Arab-IR").percent(1, 2), "%50");
        assert_eq!(pair("en", "ku-IQ").percent(1, 2), "50%");
        assert_eq!(pair("en", "zgh-Tfng-MA").percent(1, 2), "50\u{a0}%");
        // POSIX LC_NUMERIC=C / C.UTF-8 / POSIX or unset LANG keeps C/ASCII
        // numbers and "50%" even when the UI language is Turkish or Persian:
        let from_env = |vars: &[(&str, &str)]| {
            let sys = locale::System::from_vars(|k| {
                vars.iter()
                    .find(|(n, _)| *n == k)
                    .map(|(_, v)| (*v).to_owned())
            });
            Strings::new(Domain::App, &sys.messages, sys.numeric.as_ref())
        };
        let tr_c = from_env(&[("LANG", "tr_TR.UTF-8"), ("LC_NUMERIC", "C.UTF-8")]);
        assert_eq!(tr_c.tag(), "tr");
        assert_eq!(tr_c.percent(1, 2), "50%");
        assert_eq!(tr_c.bytes(1_500_000), "1.5 MB");
        assert_eq!(tr_c.number(1_234_567), "1234567");
        let fa_c = from_env(&[("LANG", "fa_IR.UTF-8"), ("LC_NUMERIC", "POSIX")]);
        assert_eq!(fa_c.tag(), "fa");
        assert_eq!(fa_c.percent(1, 2), "50%");
        let tr_msg_only = from_env(&[("LC_MESSAGES", "tr_TR.UTF-8")]);
        assert_eq!(tr_msg_only.tag(), "tr");
        assert_eq!(tr_msg_only.percent(1, 2), "50%");
        assert_eq!(
            from_env(&[("LANG", "be_BY.UTF-8@latin")]).percent(1, 2),
            "50\u{a0}%"
        );
        // When numeric=None is passed directly in Rust, formats follow the chosen language:
        assert_eq!(strings("tr").percent(1, 2), "%50");
        assert_eq!(strings("de").percent(1, 2), "50\u{a0}%");
        // Integer flooring: 29/100 in f64*100 is 28.999..., and 995/1000 must stay 99%:
        let en = pair("en", "en-US");
        assert_eq!(en.percent(29, 100), "29%");
        assert_eq!(en.percent(995, 1000), "99%");
        assert_eq!(en.percent(0, 0), "0%");
        assert_eq!(en.percent(5, 0), "0%");
        assert_eq!(en.percent(1000, 1000), "100%");
        assert_eq!(en.percent(1200, 1000), "100%");
        assert_eq!(en.percent(u64::MAX - 1, u64::MAX), "99%");
        assert_eq!(en.percent(u64::MAX, u64::MAX), "100%");
    }

    #[test]
    fn unknown_keys_and_missing_arguments_show() {
        let en = strings("en");
        assert_eq!(en.get("noSuchKey", &[]), "noSuchKey");
        assert_eq!(
            en.get("sendShareText", &[]),
            "Get my files with TotalDrop. Code: {code}"
        );
    }

    #[test]
    fn command_line_falls_back_to_app_strings() {
        let prefs: Vec<Locale> = locale::parse("tr").into_iter().collect();
        let s = Strings::new(Domain::Cli, &prefs, None);
        assert_eq!(s.tag(), "tr");
        assert_eq!(s.bytes(1_500_000), "1,5 MB");
    }

    #[test]
    fn command_line_keeps_right_to_left_languages_in_english() {
        let prefs: Vec<Locale> = locale::parse("ar").into_iter().collect();
        assert_eq!(Strings::new(Domain::Cli, &prefs, None).tag(), "en");
    }

    #[test]
    fn share_text_gives_its_code_back_in_every_language() {
        // Pasting the whole share message must connect (#148). The app
        // isolates the code (U+2068 … U+2069) so it keeps its order; a chat
        // app may drop the isolates, and the hyphens still find it.
        let tags = available(Domain::App);
        assert!(tags.len() > 30, "{tags:?}");
        for tag in tags {
            let s = strings(&tag);
            for code in ["7-abandon-ability", "12-zoo-zone-able"] {
                // As shared, and with the isolates lost on the way.
                let isolated = format!("\u{2068}{code}\u{2069}");
                for arg in [isolated.as_str(), code] {
                    let text = s.get("sendShareText", &[("code", Arg::Text(arg))]);
                    assert_eq!(
                        totaldrop_proto::TransferCode::find(&text).map(|c| c.to_string()),
                        Some(code.to_string()),
                        "{tag}: {text}"
                    );
                }
            }
        }
    }

    #[test]
    fn pseudo_locales() {
        let xa = strings("en-XA");
        assert_eq!(xa.tag(), "en-XA");
        let text = xa.get("sendShareText", &[("code", Arg::Text("7-harbor-rocket"))]);
        assert!(text.starts_with("Gét mý fîléš"), "{text}");
        assert!(
            text.contains("7-harbor-rocket"),
            "placeholders stay: {text}"
        );
        let xb = strings("ar-XB");
        assert!(xb.is_rtl());
        assert_eq!(xb.get("settings", &[]), "Settings");
        assert!(!strings("en").is_rtl());
    }

    #[test]
    fn hidden_characters_become_visible() {
        assert_eq!(
            reveal_controls("invoice\u{202E}fdp.exe"),
            "invoice<U+202E>fdp.exe"
        );
        assert_eq!(
            reveal_controls("a\u{2068}b\u{061C}c\u{00AD}d\u{2028}e\u{E0041}"),
            "a<U+2068>b<U+061C>c<U+00AD>d<U+2028>e<U+E0041>"
        );
        assert_eq!(reveal_controls("a\nb\u{85}"), "a<U+000A>b<U+0085>");
        assert_eq!(reveal_controls("fotoğraf 1.jpg"), "fotoğraf 1.jpg");
        assert_eq!(reveal_controls("صورة.jpg"), "صورة.jpg");
        assert_eq!(reveal_controls("می\u{200C}خواهم"), "می\u{200C}خواهم");
    }

    /// A received text cannot move the cursor or hide its words; its line
    /// breaks and tabs stay (#313).
    #[test]
    fn a_text_shows_its_hidden_characters() {
        let text = "one\ttwo\r\nred\u{1b}[31m \u{202e}evil\nfake\rreal\r\t\n";
        let shown = reveal_text(text);
        assert!(!shown.contains('\u{1b}') && !shown.contains('\u{202e}'));
        assert!(shown.contains("<U+001B>") && shown.contains("<U+202E>"));
        assert!(shown.starts_with("one\ttwo\nred"));
        // Only a \r that ends a line goes; one inside a line is named.
        assert!(
            shown.ends_with("\nfake<U+000D>real<U+000D>\t\n"),
            "{shown:?}"
        );
        assert_eq!(reveal_text("end\r"), "end<U+000D>");
        assert_eq!(reveal_text("می\u{200C}خواهم\n"), "می\u{200C}خواهم\n");
        assert_eq!(reveal_text(""), "");
        // Qt shows a noncharacter as a line break.
        assert_eq!(reveal_text("a\u{FDD0}b\u{FFFF}"), "a<U+FDD0>b<U+FFFF>");
    }

    #[test]
    fn drafts_are_per_language_and_key() {
        let status = Status {
            critical: BTreeSet::from(["verificationTitle".to_owned()]),
            reviewed: HashMap::from([("pt-PT".to_owned(), true), ("pt".to_owned(), false)]),
            draft: HashMap::from([("pt-PT".to_owned(), BTreeSet::from(["newKey".to_owned()]))]),
            no_words: HashMap::new(),
        };
        // A reviewed variant does not make its unreviewed base reviewed.
        assert!(status.is_draft("pt", "verificationTitle"));
        assert!(!status.is_draft("pt-PT", "verificationTitle"));
        assert!(status.is_draft("pt-PT", "newKey"));
        assert!(!status.is_draft("en", "anything"));
        assert!(status.is_draft("xx", "anything"));
    }

    #[test]
    fn needs_english_follows_the_layer_the_text_comes_from() {
        assert!(!needs_english("tr", "verificationTitle"));
        assert!(!needs_english("en", "verificationTitle"));
        assert!(!needs_english("xx", "verificationTitle"));
        assert!(!needs_english("tr", "settings"));
    }

    #[test]
    fn status_queries() {
        assert!(is_critical("verificationTitle"));
        assert!(!is_critical("settings"));
        assert!(!is_draft("tr", "verificationTitle"));
        assert!(!has_drafts("tr"));
        assert!(!has_drafts("en"));
        assert!(!has_drafts("xx"));
    }

    #[test]
    fn the_catalog_parses() {
        assert!(catalog().broken.is_empty(), "{:?}", catalog().broken);
        assert!(parse_file("app", "{\"@@locale\": \"xx\", \"a\": \"1\",}").is_err());
        assert!(parse_file("app", "{\"a\": \"1\"}").is_err());
        assert!(parse_file("app", "{\"@@locale\": \"xx\", \"a\": 1}").is_err());
        assert!(parse_file("app", "{\"@@locale\": \"xx\", \"a\": \"1\", \"a\": \"2\"}").is_err());
    }
}
