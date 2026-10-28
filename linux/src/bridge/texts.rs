//! `Texts`, the QML singleton with the interface text in the desktop's
//! language and region, from `totaldrop-l10n` (ADR 0008).
//!
//! The language is read once at start from `LANGUAGE`, `LC_ALL`,
//! `LC_MESSAGES` and `LANG`; numbers follow `LC_NUMERIC`. QML reads plain
//! strings from `strings` (JSON) and formats the rest through `format`.

use std::sync::OnceLock;

use cxx_qt_lib::QString;
use totaldrop_l10n::{Arg, Domain, Strings};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        /// Interface text for QML.
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, tag)]
        #[qproperty(bool, rtl)]
        #[qproperty(bool, draft)]
        #[qproperty(QString, strings)]
        type Texts = super::TextsRust;

        /// The message `key` with `args`, a JSON object (numbers are
        /// counts, strings are text).
        #[qinvokable]
        fn format(self: &Texts, key: &QString, args: &QString) -> QString;

        /// A size in bytes in the region's format ("1,5 MB").
        #[qinvokable]
        fn bytes(self: &Texts, n: f64) -> QString;

        /// A name from the other side with hidden direction and control
        /// characters made visible.
        #[qinvokable]
        fn reveal(self: &Texts, name: &QString) -> QString;
    }

    impl cxx_qt::Initialize for Texts {}
}

/// Backing state of [`qobject::Texts`].
#[derive(Default)]
pub struct TextsRust {
    tag: QString,
    rtl: bool,
    draft: bool,
    strings: QString,
}

fn strings() -> &'static Strings {
    static STRINGS: OnceLock<Strings> = OnceLock::new();
    STRINGS.get_or_init(|| Strings::system(Domain::App))
}

/// A QML number as a count: negative, fractional or huge values clamp.
fn count(n: f64) -> u64 {
    if n.is_finite() && n > 0.0 {
        // Saturating float-to-int conversion; sizes fit easily.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let v = n.round() as u64;
        v
    } else {
        0
    }
}

impl cxx_qt::Initialize for qobject::Texts {
    fn initialize(mut self: std::pin::Pin<&mut Self>) {
        let s = strings();
        let plain: serde_json::Map<String, serde_json::Value> = s
            .keys()
            .into_iter()
            .map(|k| (k.to_owned(), serde_json::Value::String(s.get(k, &[]))))
            .collect();
        let json = serde_json::Value::Object(plain).to_string();
        self.as_mut().set_tag(QString::from(s.tag()));
        self.as_mut().set_rtl(s.is_rtl());
        self.as_mut().set_draft(s.is_draft());
        self.as_mut().set_strings(QString::from(json.as_str()));
    }
}

impl qobject::Texts {
    fn format(&self, key: &QString, args: &QString) -> QString {
        let args: serde_json::Value =
            serde_json::from_str(&args.to_string()).unwrap_or(serde_json::Value::Null);
        let owned: Vec<(String, serde_json::Value)> = args
            .as_object()
            .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        let list: Vec<(&str, Arg<'_>)> = owned
            .iter()
            .map(|(k, v)| match v {
                serde_json::Value::Number(n) => {
                    (k.as_str(), Arg::Count(count(n.as_f64().unwrap_or(0.0))))
                }
                serde_json::Value::String(t) => (k.as_str(), Arg::Text(t.as_str())),
                _ => (k.as_str(), Arg::Text("")),
            })
            .collect();
        QString::from(strings().get(&key.to_string(), &list).as_str())
    }

    fn bytes(&self, n: f64) -> QString {
        QString::from(strings().bytes(count(n)).as_str())
    }

    fn reveal(&self, name: &QString) -> QString {
        QString::from(totaldrop_l10n::reveal_controls(&name.to_string()).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::count;

    #[test]
    fn counts_clamp() {
        assert_eq!(count(-1.0), 0);
        assert_eq!(count(f64::NAN), 0);
        assert_eq!(count(2.6), 3);
        assert_eq!(count(1.5e6), 1_500_000);
        assert_eq!(count(f64::INFINITY), 0);
    }
}
