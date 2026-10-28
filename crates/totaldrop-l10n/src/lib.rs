//! TotalDrop's interface text for the Rust programs (the Linux app and the
//! command line): the strings in `l10n/`, the language the system asks
//! for, and numbers, sizes and plurals in that language and region.
//!
//! The rules are in `docs/kararlar/0008-diller-ve-bolgeler.md`. The ARB
//! files are compiled in, so a program never reads translations from disk.

pub mod check;
pub mod locale;
pub mod message;

use std::collections::{BTreeSet, HashMap};

use icu_decimal::DecimalFormatter;
use icu_decimal::input::Decimal;
use icu_locale::Locale;
use icu_plurals::{PluralCategory, PluralRules};

pub use message::{Arg, Message};

mod files {
    include!(concat!(env!("OUT_DIR"), "/files.rs"));
}

/// The template language.
pub const TEMPLATE: &str = "en";

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

pub(crate) fn files() -> Vec<File> {
    files::FILES
        .iter()
        .filter_map(|(domain, json)| {
            let value: serde_json::Value = serde_json::from_str(json).ok()?;
            let object = value.as_object()?;
            let locale = object.get("@@locale")?.as_str()?.replace('_', "-");
            let mut messages = HashMap::new();
            let mut raw = HashMap::new();
            for (key, v) in object {
                if key.starts_with('@') {
                    continue;
                }
                if let Some(s) = v.as_str() {
                    messages.insert(key.clone(), Message::parse(s));
                    raw.insert(key.clone(), s.to_owned());
                }
            }
            Some(File {
                domain,
                locale,
                messages,
                raw,
            })
        })
        .collect()
}

/// `l10n/status.json`: which languages were reviewed, which keys are still
/// drafts, and which keys must never be shown as a draft.
#[derive(Debug, Default)]
pub(crate) struct Status {
    pub(crate) critical: BTreeSet<String>,
    pub(crate) reviewed: HashMap<String, bool>,
    pub(crate) draft: HashMap<String, BTreeSet<String>>,
}

pub(crate) fn status() -> Status {
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
        }
    }
    status
}

/// The strings and formats for one language and region.
pub struct Strings {
    tag: String,
    draft: bool,
    /// Lookup order: this language, then the fallbacks, then English.
    layers: Vec<HashMap<String, Message>>,
    formats: Formats,
}

/// Plurals, numbers and direction for one language and region, with or
/// without translations for it.
pub struct Formats {
    rtl: bool,
    plurals: Option<PluralRules>,
    decimal: Option<DecimalFormatter>,
}

impl Formats {
    /// Plural rules and direction of `language`, numbers of `numeric` (or
    /// of `language` when `None`).
    #[must_use]
    pub fn new(language: &Locale, numeric: Option<&Locale>) -> Self {
        Self {
            rtl: locale::is_rtl(language),
            plurals: PluralRules::try_new_cardinal(language.into()).ok(),
            decimal: DecimalFormatter::try_new(
                numeric.unwrap_or(language).into(),
                Default::default(),
            )
            .ok(),
        }
    }
}

impl std::fmt::Debug for Strings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Strings")
            .field("tag", &self.tag)
            .finish_non_exhaustive()
    }
}

/// Every language with strings in `domain`.
#[must_use]
pub fn available(domain: Domain) -> Vec<String> {
    let mut tags: Vec<String> = files()
        .into_iter()
        .filter(|f| f.domain == domain.name())
        .map(|f| f.locale)
        .collect();
    tags.sort();
    tags
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
        let all = files();
        let tags = available(Domain::App);
        let own = available(domain);
        let candidates: Vec<&str> = own.iter().chain(tags.iter()).map(String::as_str).collect();
        let mut tag = locale::negotiate(preferences, &candidates)
            .unwrap_or(TEMPLATE)
            .to_owned();
        let mut chosen = locale::parse(&tag).unwrap_or(Locale::UNKNOWN);
        if domain == Domain::Cli && locale::is_rtl(&chosen) {
            tag = TEMPLATE.to_owned();
            chosen = locale::parse(TEMPLATE).unwrap_or(Locale::UNKNOWN);
        }
        let status = status();
        let drafts = status.draft.get(&tag).cloned().unwrap_or_default();
        let reviewed = tag == TEMPLATE || status.reviewed.get(&tag).copied().unwrap_or(false);
        // Safety-critical text is never shown as a draft: those keys stay
        // English until a person has reviewed them.
        let hidden: BTreeSet<&String> = status
            .critical
            .iter()
            .filter(|k| !reviewed || drafts.contains(*k))
            .collect();

        // This language and its fallbacks (pt-BR → pt), then English; for
        // the command line, its own strings before the app's at each step.
        let mut chain: Vec<String> = Vec::new();
        let mut parent = tag.clone();
        loop {
            chain.push(parent.clone());
            match parent.rsplit_once('-') {
                Some((p, _)) => parent = p.to_owned(),
                None => break,
            }
        }
        if !chain.iter().any(|t| t == TEMPLATE) {
            chain.push(TEMPLATE.to_owned());
        }
        let domains: &[&str] = match domain {
            Domain::App => &["app"],
            Domain::Cli => &["cli", "app"],
        };
        let mut layers = Vec::new();
        for t in &chain {
            for d in domains {
                if let Some(file) = all.iter().find(|f| f.domain == *d && f.locale == *t) {
                    let layer: HashMap<String, Message> = file
                        .messages
                        .iter()
                        .filter(|(k, _)| t == TEMPLATE || !hidden.contains(k))
                        .filter_map(|(k, m)| m.as_ref().ok().map(|m| (k.clone(), m.clone())))
                        .collect();
                    layers.push(layer);
                }
            }
        }
        Self {
            draft: !reviewed || !drafts.is_empty(),
            formats: Formats::new(&chosen, numeric),
            layers,
            tag,
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

    /// Whether some of the text is a translation nobody has reviewed yet.
    #[must_use]
    pub fn is_draft(&self) -> bool {
        self.draft
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

    /// A size in bytes, in SI units ("1.5 MB", "1,5 MB", "١٫٥ MB").
    #[must_use]
    pub fn bytes(&self, n: u64) -> String {
        const KEYS: [&str; 5] = ["sizeKB", "sizeMB", "sizeGB", "sizeTB", "sizePB"];
        if n < 1000 {
            return self.get("sizeBytes", &[("value", Arg::Text(&self.number(n)))]);
        }
        // Tenths of the unit, rounded half up: 1 549 999 B → 15 tenths of MB.
        let mut divisor: u128 = 1000;
        let mut unit = 0usize;
        let n = u128::from(n);
        while n >= divisor.saturating_mul(1000) && unit.saturating_add(1) < KEYS.len() {
            divisor = divisor.saturating_mul(1000);
            unit = unit.saturating_add(1);
        }
        let tenths = n
            .saturating_mul(10)
            .saturating_add(divisor / 2)
            .checked_div(divisor)
            .unwrap_or(0);
        let tenths = u64::try_from(tenths).unwrap_or(u64::MAX);
        let value = if tenths < 100 {
            let mut d = Decimal::from(tenths);
            d.multiply_pow10(-1);
            self.formats.decimal.as_ref().map_or_else(
                || format!("{}.{}", tenths / 10, tenths % 10),
                |f| f.format(&d).to_string(),
            )
        } else {
            self.number(tenths.saturating_add(5) / 10)
        };
        let key = KEYS.get(unit).copied().unwrap_or("sizePB");
        self.get(key, &[("value", Arg::Text(&value))])
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

#[cfg(test)]
mod tests {
    use super::*;

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

    fn strings(tag: &str) -> Strings {
        let prefs: Vec<Locale> = locale::parse(tag).into_iter().collect();
        Strings::new(Domain::App, &prefs, None)
    }

    #[test]
    fn english_and_turkish() {
        assert_eq!(strings("en-US").tag(), "en");
        assert_eq!(strings("tr-TR").tag(), "tr");
        assert_eq!(strings("xx").tag(), "en");
        assert!(!strings("tr").is_draft());
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
        assert_eq!(en.bytes(12_345_678), "12 MB");
        assert_eq!(en.bytes(999_999_999), "1,000 MB");
        assert_eq!(en.bytes(u64::MAX), "18,447 PB");
        let tr = strings("tr");
        assert_eq!(tr.bytes(1_500_000), "1,5 MB");
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
}
