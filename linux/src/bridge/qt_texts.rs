// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Qt's own interface text in the app's language (#307): a translator that
//! answers the strings Qt and the desktop's Qt Quick style draw themselves
//! (the right-click menu of text fields) from `l10n/app`.

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

        /// Install the translator. Call once the application object exists
        /// and before QML loads.
        fn install_qt_texts();
    }
}

/// Qt contexts whose text-editing menu items are marked with `qsTr`:
/// Qt 6.9+ Quick Controls (`QtQuick/Controls/impl/*Action.qml`, used by
/// Fusion and Basic) and KDE's `org.kde.desktop` style
/// (`qqc2-desktop-style`, `private/TextFieldContextMenu.qml`). Sources are
/// Qt's English exactly, as `qsTr` looks them up.
const QT_TEXTS: [(&str, &str, &str); 14] = [
    ("UndoAction", "Undo", "editUndo"),
    ("RedoAction", "Redo", "editRedo"),
    ("CutAction", "Cut", "editCut"),
    ("CopyAction", "Copy", "editCopy"),
    ("PasteAction", "Paste", "editPaste"),
    ("DeleteAction", "Delete", "editDelete"),
    ("SelectAllAction", "Select All", "editSelectAll"),
    ("TextFieldContextMenu", "Undo", "editUndo"),
    ("TextFieldContextMenu", "Redo", "editRedo"),
    ("TextFieldContextMenu", "Cut", "editCut"),
    ("TextFieldContextMenu", "Copy", "editCopy"),
    ("TextFieldContextMenu", "Paste", "editPaste"),
    ("TextFieldContextMenu", "Delete", "editDelete"),
    ("TextFieldContextMenu", "Select All", "editSelectAll"),
];

/// Qt's `(context, source, text)` triples in the language of `strings`.
fn entries(strings: &Strings) -> Vec<(&'static str, &'static str, String)> {
    QT_TEXTS
        .iter()
        .map(|&(context, source, key)| (context, source, strings.get(key, &[])))
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
    ffi::install_qt_texts();
}

#[cfg(test)]
mod tests {
    use super::*;
    use totaldrop_l10n::{Domain, locale};

    fn strings(tag: &str) -> Strings {
        Strings::new(Domain::App, &[locale::parse(tag).unwrap()], None)
    }

    #[test]
    fn every_key_is_in_the_template() {
        let en = strings("en");
        let keys = en.keys();
        for (_, _, key) in QT_TEXTS {
            assert!(keys.contains(&key), "{key} is not in l10n/app/app_en.arb");
        }
    }

    #[test]
    fn both_styles_get_every_command() {
        // Fusion's actions and KDE's menu name the same seven commands.
        let commands = |context: &str| -> Vec<&str> {
            QT_TEXTS
                .iter()
                .filter(|(c, _, _)| *c == context)
                .map(|(_, source, _)| *source)
                .collect()
        };
        let actions: Vec<&str> = QT_TEXTS
            .iter()
            .filter(|(c, _, _)| c.ends_with("Action"))
            .map(|(_, source, _)| *source)
            .collect();
        assert_eq!(actions, commands("TextFieldContextMenu"));
        assert_eq!(actions.len(), 7);
    }

    #[test]
    fn the_menu_follows_the_language() {
        let tr = entries(&strings("tr"));
        assert!(tr.contains(&("PasteAction", "Paste", "Yapıştır".to_owned())));
        assert!(tr.contains(&(
            "TextFieldContextMenu",
            "Select All",
            "Tümünü seç".to_owned()
        )));
        let ja = entries(&strings("ja"));
        assert!(ja.contains(&("CopyAction", "Copy", "コピー".to_owned())));
        let en = entries(&strings("en"));
        assert!(en.contains(&("SelectAllAction", "Select All", "Select all".to_owned())));
    }
}
