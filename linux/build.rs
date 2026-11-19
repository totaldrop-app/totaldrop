// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Builds the QML module and the Rust QObjects with CXX-Qt.

use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

fn main() {
    let qml = [
        QmlFile::from("qml/Main.qml"),
        QmlFile::from("qml/Theme.qml").singleton(true),
        QmlFile::from("qml/Strings.qml").singleton(true),
        QmlFile::from("qml/Mark.qml"),
        QmlFile::from("qml/HomePage.qml"),
        QmlFile::from("qml/ActionCard.qml"),
        QmlFile::from("qml/PageHeader.qml"),
        QmlFile::from("qml/PrimaryButton.qml"),
        QmlFile::from("qml/SecondaryButton.qml"),
        QmlFile::from("qml/IconButton.qml"),
        QmlFile::from("qml/Field.qml"),
        QmlFile::from("qml/Toggle.qml"),
        QmlFile::from("qml/Divider.qml"),
        QmlFile::from("qml/FocusScroller.qml"),
        QmlFile::from("qml/VerificationWords.qml"),
        QmlFile::from("qml/QrCode.qml"),
        QmlFile::from("qml/TransferProgress.qml"),
        QmlFile::from("qml/ResultPanel.qml"),
        QmlFile::from("qml/SendPage.qml"),
        QmlFile::from("qml/ReceivePage.qml"),
        QmlFile::from("qml/SettingsPage.qml"),
    ];
    let mut builder = CxxQtBuilder::new_qml_module(QmlModule::new("app.totaldrop").qml_files(qml))
        .qt_module("Gui")
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .files([
            "src/bridge/clipboard.rs",
            "src/bridge/launcher.rs",
            "src/bridge/qt_texts.rs",
            "src/bridge/surface.rs",
            "src/bridge/texts.rs",
            "src/bridge/theme.rs",
            "src/bridge/transfer.rs",
        ]);

    #[cfg(target_os = "linux")]
    if let Ok(out) = std::process::Command::new("pkg-config")
        .args(["--cflags-only-I", "xcb"])
        .output()
        && out.status.success()
        && let Ok(text) = std::str::from_utf8(&out.stdout)
    {
        let dirs: Vec<&str> = text
            .split_whitespace()
            .filter_map(|flag| flag.strip_prefix("-I"))
            .collect();
        if !dirs.is_empty() {
            // SAFETY: only adds `libxcb` include directories from `pkg-config`
            // without altering CXX-Qt's own compiler flags or sources.
            builder = unsafe {
                builder.cc_builder(|cc| {
                    for dir in &dirs {
                        cc.include(dir);
                    }
                })
            };
        }
    }

    builder.build();

    #[cfg(target_os = "linux")]
    {
        if let Ok(out) = std::process::Command::new("pkg-config")
            .args(["--libs-only-L", "xcb"])
            .output()
            && out.status.success()
            && let Ok(text) = std::str::from_utf8(&out.stdout)
        {
            for flag in text.split_whitespace() {
                if let Some(dir) = flag.strip_prefix("-L") {
                    println!("cargo:rustc-link-search=native={dir}");
                }
            }
        }
        println!("cargo:rustc-link-lib=xcb");
    }
}
