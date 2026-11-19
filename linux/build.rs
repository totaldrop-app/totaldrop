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
        QmlFile::from("qml/StageHeading.qml"),
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
    let xcb_lib_dirs: Vec<String> = {
        println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
        println!("cargo:rerun-if-env-changed=PKG_CONFIG_LIBDIR");
        let dirs = match pkg_config_xcb("--cflags-only-I", "-I") {
            Ok(dirs) => dirs,
            Err(msg) => {
                println!("cargo::error={msg}");
                std::process::exit(1);
            }
        };
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
        match pkg_config_xcb("--libs-only-L", "-L") {
            Ok(dirs) => dirs,
            Err(msg) => {
                println!("cargo::error={msg}");
                std::process::exit(1);
            }
        }
    };

    builder.build();

    #[cfg(target_os = "linux")]
    {
        for dir in xcb_lib_dirs {
            println!("cargo:rustc-link-search=native={dir}");
        }
        println!("cargo:rustc-link-lib=xcb");
    }
}

#[cfg(target_os = "linux")]
fn pkg_config_xcb(arg: &str, prefix: &str) -> Result<Vec<String>, String> {
    let out = std::process::Command::new("pkg-config")
        .args([arg, "xcb"])
        .output()
        .map_err(|e| format!("libxcb development files missing (could not run pkg-config: {e})"))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!(
            "libxcb development files missing (pkg-config {arg} xcb failed: {})",
            stderr.trim()
        ));
    }
    let text = std::str::from_utf8(&out.stdout)
        .map_err(|e| format!("pkg-config {arg} xcb output was not UTF-8: {e}"))?;
    Ok(text
        .split_whitespace()
        .filter_map(|flag| flag.strip_prefix(prefix).map(str::to_owned))
        .collect())
}
