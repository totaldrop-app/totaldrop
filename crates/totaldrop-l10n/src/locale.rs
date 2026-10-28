//! Which language to show, from the system's preferences.
//!
//! Matching walks CLDR fallback chains (ICU4X `LocaleFallbacker`, so
//! `pt-AO` → `pt-PT` → `pt`) for each preference, first as written and then
//! with likely subtags added (`zh-HK` → `zh-Hant-HK`). It is a fallback
//! chain, not a full CLDR language matcher.

use icu_locale::fallback::LocaleFallbacker;
use icu_locale::{DataLocale, Locale, LocaleExpander};

/// Languages written right to left.
const RTL: [&str; 6] = ["ar", "fa", "he", "ur", "ps", "yi"];

/// Parses a BCP 47 tag, accepting `_` as a separator and mapping
/// Norwegian (`no`, `nn`) to Bokmål, which is what we translate.
#[must_use]
pub fn parse(tag: &str) -> Option<Locale> {
    let tag = tag.trim().replace('_', "-");
    let mut locale = Locale::try_from_str(&tag).ok()?;
    if matches!(locale.id.language.as_str(), "no" | "nn") {
        locale.id.language = "nb".parse().ok()?;
    }
    if locale.id.language.is_unknown() {
        return None;
    }
    Some(locale)
}

/// The fallback chain of a locale, most specific first, without `und`.
fn chain(locale: &Locale) -> Vec<String> {
    let fallbacker = LocaleFallbacker::new().for_config(Default::default());
    let mut it = fallbacker.fallback_for(DataLocale::from(locale));
    let mut out = Vec::new();
    // A chain is a handful of steps; the bound only guards against a loop.
    for _ in 0..16 {
        let step = it.get();
        if step.is_unknown() {
            break;
        }
        out.push(step.to_string());
        it.step();
    }
    out
}

/// A locale in the form chains use, so `zh-Hans` and `zh` compare equal.
fn normal(locale: &Locale) -> String {
    chain(locale).into_iter().next().unwrap_or_default()
}

/// The best of `available` for the ordered `preferences`, or `None` when
/// nothing fits and the caller should use its default (English).
#[must_use]
pub fn negotiate<'a>(preferences: &[Locale], available: &[&'a str]) -> Option<&'a str> {
    let normals: Vec<(String, &'a str)> = available
        .iter()
        .filter_map(|a| parse(a).map(|l| (normal(&l), *a)))
        .collect();
    let expander = LocaleExpander::new_common();
    for pref in preferences {
        let mut maximized = pref.clone();
        expander.maximize(&mut maximized.id);
        for step in chain(pref).into_iter().chain(chain(&maximized)) {
            if let Some((_, a)) = normals.iter().find(|(n, _)| *n == step) {
                return Some(a);
            }
        }
    }
    None
}

/// Whether a language is written right to left.
#[must_use]
pub fn is_rtl(locale: &Locale) -> bool {
    RTL.contains(&locale.id.language.as_str())
}

/// The locale preferences of a POSIX system, as gettext reads them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct System {
    /// Languages for messages, most preferred first.
    pub messages: Vec<Locale>,
    /// The locale for numbers, if it differs from the first language.
    pub numeric: Option<Locale>,
}

impl System {
    /// Reads `LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, `LC_NUMERIC` and `LANG`.
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_vars(|k| std::env::var(k).ok())
    }

    /// As [`System::from_env`], with variables from `get` (for tests).
    pub fn from_vars(get: impl Fn(&str) -> Option<String>) -> Self {
        let first = |keys: &[&str]| {
            keys.iter()
                .filter_map(|k| get(k))
                .find(|v| !v.trim().is_empty())
        };
        let messages_locale = first(&["LC_ALL", "LC_MESSAGES", "LANG"]);
        let c_locale = messages_locale.as_deref().is_none_or(is_c);
        let mut messages = Vec::new();
        // gettext ignores LANGUAGE when the locale is C: the program then
        // shows its untranslated (English) text.
        if !c_locale && let Some(list) = get("LANGUAGE") {
            messages.extend(list.split(':').filter_map(posix));
        }
        messages.extend(messages_locale.as_deref().and_then(posix));
        let numeric = first(&["LC_ALL", "LC_NUMERIC", "LANG"])
            .as_deref()
            .and_then(posix);
        Self { messages, numeric }
    }
}

fn is_c(value: &str) -> bool {
    let base = value.split(['.', '@']).next().unwrap_or_default();
    base == "C" || base == "POSIX"
}

/// `pt_BR.UTF-8`, `sr_RS@latin` → BCP 47.
fn posix(value: &str) -> Option<Locale> {
    if value.is_empty() || is_c(value) {
        return None;
    }
    let (base, modifier) = match value.split_once('@') {
        Some((b, m)) => (b, Some(m)),
        None => (value, None),
    };
    let base = base.split('.').next().unwrap_or_default();
    let mut locale = parse(base)?;
    match modifier {
        Some("latin") => locale.id.script = "Latn".parse().ok(),
        Some("cyrillic") => locale.id.script = "Cyrl".parse().ok(),
        _ => {}
    }
    Some(locale)
}

#[cfg(test)]
mod tests {
    use super::*;

    const AVAILABLE: [&str; 9] = [
        "en", "tr", "pt", "pt-BR", "pt-PT", "zh", "zh-Hant", "nb", "ar",
    ];

    fn pick(prefs: &[&str]) -> Option<&'static str> {
        let prefs: Vec<Locale> = prefs.iter().filter_map(|p| parse(p)).collect();
        negotiate(&prefs, &AVAILABLE)
    }

    #[test]
    fn exact_and_regional_fallback() {
        assert_eq!(pick(&["tr-TR"]), Some("tr"));
        assert_eq!(pick(&["pt-BR"]), Some("pt-BR"));
        assert_eq!(pick(&["pt-AO"]), Some("pt-PT"));
        assert_eq!(pick(&["ar-EG"]), Some("ar"));
    }

    #[test]
    fn scripts_come_from_likely_subtags() {
        assert_eq!(pick(&["zh-TW"]), Some("zh-Hant"));
        assert_eq!(pick(&["zh-HK"]), Some("zh-Hant"));
        assert_eq!(pick(&["zh-CN"]), Some("zh"));
        assert_eq!(pick(&["zh-Hans-SG"]), Some("zh"));
    }

    #[test]
    fn norwegian_and_order_of_preferences() {
        assert_eq!(pick(&["no-NO"]), Some("nb"));
        assert_eq!(pick(&["nn"]), Some("nb"));
        assert_eq!(pick(&["xx", "de-AT", "tr"]), Some("tr"));
        assert_eq!(pick(&["de", "fr"]), None);
    }

    #[test]
    fn right_to_left() {
        assert!(parse("ar-EG").is_some_and(|l| is_rtl(&l)));
        assert!(parse("fa").is_some_and(|l| is_rtl(&l)));
        assert!(!parse("tr").is_some_and(|l| is_rtl(&l)));
    }

    fn env(vars: &[(&str, &str)]) -> System {
        System::from_vars(|k| {
            vars.iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_owned())
        })
    }

    fn tags(s: &System) -> Vec<String> {
        s.messages.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn gettext_order() {
        let s = env(&[("LANG", "tr_TR.UTF-8")]);
        assert_eq!(tags(&s), ["tr-TR"]);
        let s = env(&[("LANGUAGE", "de:fr"), ("LANG", "tr_TR.UTF-8")]);
        assert_eq!(tags(&s), ["de", "fr", "tr-TR"]);
        let s = env(&[
            ("LC_ALL", "pt_BR.UTF-8"),
            ("LC_MESSAGES", "de_DE"),
            ("LANG", "tr_TR"),
        ]);
        assert_eq!(tags(&s), ["pt-BR"]);
        let s = env(&[("LC_MESSAGES", "sr_RS@latin")]);
        assert_eq!(tags(&s), ["sr-Latn-RS"]);
    }

    #[test]
    fn c_locale_ignores_language() {
        let s = env(&[("LANGUAGE", "de"), ("LANG", "C.UTF-8")]);
        assert!(s.messages.is_empty());
        assert!(env(&[]).messages.is_empty());
    }

    #[test]
    fn numbers_follow_lc_numeric() {
        let s = env(&[("LANG", "en_US.UTF-8"), ("LC_NUMERIC", "de_DE.UTF-8")]);
        assert_eq!(s.numeric.map(|l| l.to_string()).as_deref(), Some("de-DE"));
        let s = env(&[("LC_ALL", "fr_FR"), ("LC_NUMERIC", "de_DE")]);
        assert_eq!(s.numeric.map(|l| l.to_string()).as_deref(), Some("fr-FR"));
    }
}
