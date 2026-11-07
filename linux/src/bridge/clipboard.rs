// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! The clipboard's text, exactly as it is (`clipboard.h`).

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;
    }

    #[namespace = "totaldrop"]
    unsafe extern "C++" {
        include!("totaldrop-linux/src/bridge/clipboard.h");

        /// The clipboard's text, or empty. Call on the GUI thread.
        fn clipboard_text() -> QString;

        /// Put `text` on the clipboard as it is. Call on the GUI thread.
        fn set_clipboard_text(text: &QString);
    }
}

pub use ffi::{clipboard_text, set_clipboard_text};
