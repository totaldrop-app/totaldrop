//! TotalDrop's interface text for the Rust programs (the Linux app and the
//! command line): the strings in `l10n/`, the language the system asks
//! for, and numbers, sizes and plurals in that language and region.
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
/// drafts, and which keys must never be shown as a draft. One status per
/// language covers its app and command-line strings.
#[derive(Debug, Default)]
pub(crate) struct Status {
    pub(crate) critical: BTreeSet<String>,
    pub(crate) reviewed: HashMap<String, bool>,
    pub(crate) draft: HashMap<String, BTreeSet<String>>,
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
            }
        }
        status
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

    /// `tenths / 10` with one decimal in the region's format.
    fn tenths(&self, tenths: u64) -> String {
        let mut d = Decimal::from(tenths);
        d.multiply_pow10(-1);
        self.decimal.as_ref().map_or_else(
            || format!("{}.{}", tenths / 10, tenths % 10),
            |f| f.format(&d).to_string(),
        )
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
/// separators (Zl, Zp). Zero-width joiner and non-joiner, which Persian and
/// Indic scripts need, are allowed.
#[must_use]
pub fn is_hidden(c: char) -> bool {
    if matches!(c, '\u{200C}' | '\u{200D}') {
        return false;
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn drafts_are_per_language_and_key() {
        let status = Status {
            critical: BTreeSet::from(["verificationTitle".to_owned()]),
            reviewed: HashMap::from([("pt-PT".to_owned(), true), ("pt".to_owned(), false)]),
            draft: HashMap::from([("pt-PT".to_owned(), BTreeSet::from(["newKey".to_owned()]))]),
        };
        // A reviewed variant does not make its unreviewed base reviewed.
        assert!(status.is_draft("pt", "verificationTitle"));
        assert!(!status.is_draft("pt-PT", "verificationTitle"));
        assert!(status.is_draft("pt-PT", "newKey"));
        assert!(!status.is_draft("en", "anything"));
        assert!(status.is_draft("xx", "anything"));
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
