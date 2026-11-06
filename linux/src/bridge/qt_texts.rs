// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Qt's own interface text in the app's language (#307): a translator that
//! answers the strings Qt and the desktop's Qt Quick style draw themselves
//! (the right-click menu of text fields) from `l10n/app`, and tells Qt the
//! layout direction of the chosen language.

use cxx_qt_lib::QString;
use totaldrop_l10n::Strings;

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;
    }

    #[namespace = "totaldrop"]
    unsafe extern "C++" {
        include!("totaldrop-linux/src/bridge/qt_texts.h");

        /// Answer `source` in `context` with `text`; see `qt_texts.h`.
        fn add_qt_text(context: &QString, source: &QString, text: &QString);

        /// Install the translator; false if Qt refused it. Call once the
        /// application object exists and before QML loads.
        fn install_qt_texts() -> bool;

        /// What Qt shows for `source` in `context` with every installed
        /// translator. Only the tests call it.
        #[allow(dead_code)]
        fn qt_translate(context: &QString, source: &QString) -> QString;
    }
}

/// The text-editing commands: Qt's English source and the `l10n/app` key.
const UNDO: (&str, &str) = ("Undo", "editUndo");
const REDO: (&str, &str) = ("Redo", "editRedo");
const CUT: (&str, &str) = ("Cut", "editCut");
const COPY: (&str, &str) = ("Copy", "editCopy");
const PASTE: (&str, &str) = ("Paste", "editPaste");
const DELETE: (&str, &str) = ("Delete", "editDelete");
const SELECT_ALL: (&str, &str) = ("Select All", "editSelectAll");

/// Qt contexts that mark text-editing commands with `qsTr`, and the
/// commands each one shows. `qsTr` uses the QML file's base name as the
/// context:
/// - Qt Quick Controls 6.9.0 and 6.11+ (`QtQuick/Controls/impl/*Action.qml`,
///   the menu of Fusion and Basic; 6.9.1 to 6.10 have no menu);
/// - KDE's `org.kde.desktop` style (`qqc2-desktop-style`,
///   `private/TextFieldContextMenu.qml`, and the touch toolbar
///   `private/MobileTextActionsToolBarImpl.qml`).
const QT_TEXTS: [(&str, &[(&str, &str)]); 9] = [
    ("UndoAction", &[UNDO]),
    ("RedoAction", &[REDO]),
    ("CutAction", &[CUT]),
    ("CopyAction", &[COPY]),
    ("PasteAction", &[PASTE]),
    ("DeleteAction", &[DELETE]),
    ("SelectAllAction", &[SELECT_ALL]),
    (
        "TextFieldContextMenu",
        &[UNDO, REDO, CUT, COPY, PASTE, DELETE, SELECT_ALL],
    ),
    ("MobileTextActionsToolBarImpl", &[CUT, COPY, PASTE]),
];

/// Qt's `(context, source, text)` triples in the language of `strings`.
/// Qt reads `QGuiApplication`/`QT_LAYOUT_DIRECTION` when a translator is
/// installed and lays out its own windows (such as the menu, a window of
/// its own in Qt 6.11) right to left on `RTL`.
fn entries(strings: &Strings) -> Vec<(&'static str, &'static str, String)> {
    let direction = if strings.is_rtl() { "RTL" } else { "LTR" };
    QT_TEXTS
        .iter()
        .flat_map(|&(context, commands)| {
            commands
                .iter()
                .map(move |&(source, key)| (context, source, strings.get(key, &[])))
        })
        .chain(std::iter::once((
            "QGuiApplication",
            "QT_LAYOUT_DIRECTION",
            direction.to_owned(),
        )))
        .collect()
}

/// Translate Qt's own text into the app's language. Call once the
/// application object exists and before QML loads.
pub fn install(strings: &Strings) {
    for (context, source, text) in entries(strings) {
        ffi::add_qt_text(
            &QString::from(context),
            &QString::from(source),
            &QString::from(text.as_str()),
        );
    }
    if !ffi::install_qt_texts() {
        eprintln!("totaldrop: Qt did not take the interface-text translator");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cxx_qt_lib::QCoreApplication;
    use totaldrop_l10n::{Domain, locale};

    fn strings(tag: &str) -> Strings {
        Strings::new(Domain::App, &[locale::parse(tag).unwrap()], None)
    }

    fn find<'a>(e: &'a [(&str, &str, String)], context: &str, source: &str) -> Option<&'a str> {
        e.iter()
            .find(|(c, s, _)| *c == context && *s == source)
            .map(|(_, _, t)| t.as_str())
    }

    #[test]
    fn every_key_is_in_the_template() {
        let en = strings("en");
        let keys = en.keys();
        for (_, commands) in QT_TEXTS {
            for (_, key) in commands {
                assert!(keys.contains(key), "{key} is not in l10n/app/app_en.arb");
            }
        }
    }

    #[test]
    fn the_menu_and_direction_follow_the_language() {
        let tr = entries(&strings("tr"));
        assert_eq!(find(&tr, "PasteAction", "Paste"), Some("Yapıştır"));
        assert_eq!(
            find(&tr, "TextFieldContextMenu", "Select All"),
            Some("Tümünü seç")
        );
        assert_eq!(
            find(&tr, "QGuiApplication", "QT_LAYOUT_DIRECTION"),
            Some("LTR")
        );
        let ar = entries(&strings("ar"));
        assert_eq!(
            find(&ar, "MobileTextActionsToolBarImpl", "Copy"),
            Some("نسخ")
        );
        assert_eq!(
            find(&ar, "QGuiApplication", "QT_LAYOUT_DIRECTION"),
            Some("RTL")
        );
        // English keeps Qt's own words, in the platform's title case.
        let en = entries(&strings("en"));
        for (context, source, text) in &en {
            if *context != "QGuiApplication" {
                assert_eq!(source, text, "{context}");
            }
        }
    }

    // Through Qt itself: the translator answers its texts and lets every
    // other lookup fall through to Qt's English. One test, so the process
    // has one application object.
    #[test]
    fn qt_asks_the_translator() {
        let _app = QCoreApplication::new();
        install(&strings("ja"));
        let tr =
            |c: &str, s: &str| ffi::qt_translate(&QString::from(c), &QString::from(s)).to_string();
        assert_eq!(tr("CopyAction", "Copy"), "コピー");
        assert_eq!(tr("TextFieldContextMenu", "Select All"), "すべて選択");
        assert_eq!(tr("QGuiApplication", "QT_LAYOUT_DIRECTION"), "LTR");
        assert_eq!(tr("CopyAction", "Copy all"), "Copy all");
        assert_eq!(tr("SomethingElse", "Paste"), "Paste");
    }
}
