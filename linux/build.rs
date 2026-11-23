// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Builds the QML module, the Rust QObjects with CXX-Qt, and the Wayland
//! protocol bindings used for `GlobalShortcuts` dialog parenting (#631).

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

fn pkg_config_var(pkg: &str, var: &str) -> Option<String> {
    let out = Command::new("pkg-config")
        .args([&format!("--variable={var}"), pkg])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let val = String::from_utf8(out.stdout).ok()?;
    let trimmed = val.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn wayland_protocols_dir() -> Result<PathBuf, Box<dyn Error>> {
    if let Some(dir) = pkg_config_var("wayland-protocols", "pkgdatadir") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return Ok(p);
        }
    }
    let fallback = PathBuf::from("/usr/share/wayland-protocols");
    if fallback.is_dir() {
        return Ok(fallback);
    }
    Err("wayland-protocols XML directory not found (install wayland-protocols)".into())
}

fn run_wayland_scanner(
    scanner: &std::ffi::OsStr,
    mode: &str,
    xml: &Path,
    out: &Path,
) -> Result<(), Box<dyn Error>> {
    let status = Command::new(scanner)
        .arg(mode)
        .arg(xml)
        .arg(out)
        .status()
        .map_err(|e| format!("failed to run wayland-scanner ({mode}): {e}"))?;
    if !status.success() {
        return Err(format!(
            "wayland-scanner {mode} failed for {} with status {status}",
            xml.display()
        )
        .into());
    }
    Ok(())
}

fn qmake_query(var: &str) -> Option<String> {
    let qmake = env::var_os("QMAKE").unwrap_or_else(|| "qmake".into());
    let out = Command::new(qmake).args(["-query", var]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let val = String::from_utf8(out.stdout).ok()?;
    let trimmed = val.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// Returns the QtGui private-headers include directory ONLY on Qt 6.5..6.8
/// where `QWaylandWindow::winId()` did not yet return `wl_surface*` (#631).
/// Qt 6.9+ uses public `QWindow::winId()` with zero private headers.
fn qt68_qpa_include_dir() -> Option<PathBuf> {
    let version = qmake_query("QT_VERSION")?;
    let mut parts = version.split('.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;
    if major != 6 || !(5..9).contains(&minor) {
        return None;
    }
    let headers = PathBuf::from(qmake_query("QT_INSTALL_HEADERS")?);
    let qpa_root = headers.join("QtGui").join(&version).join("QtGui");
    qpa_root
        .join("qpa")
        .join("qplatformnativeinterface.h")
        .is_file()
        .then_some(qpa_root)
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-env-changed=QMAKE");
    println!("cargo::rerun-if-env-changed=WAYLAND_SCANNER");
    println!("cargo::rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo::rerun-if-changed=src/bridge/wayland_parent.h");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR not set")?);
    let wl_dir = out_dir.join("wayland");
    fs::create_dir_all(&wl_dir)?;

    let protocols_dir = wayland_protocols_dir()?;
    let scanner = env::var_os("WAYLAND_SCANNER").unwrap_or_else(|| "wayland-scanner".into());

    let foreign_xml = protocols_dir
        .join("unstable")
        .join("xdg-foreign")
        .join("xdg-foreign-unstable-v2.xml");
    let activation_xml = protocols_dir
        .join("staging")
        .join("xdg-activation")
        .join("xdg-activation-v1.xml");
    println!("cargo::rerun-if-changed={}", foreign_xml.display());
    println!("cargo::rerun-if-changed={}", activation_xml.display());

    let foreign_h = wl_dir.join("xdg-foreign-unstable-v2-client-protocol.h");
    let foreign_code = wl_dir.join("xdg-foreign-unstable-v2-protocol-code.h");
    let activation_h = wl_dir.join("xdg-activation-v1-client-protocol.h");
    let activation_code = wl_dir.join("xdg-activation-v1-protocol-code.h");

    run_wayland_scanner(&scanner, "client-header", &foreign_xml, &foreign_h)?;
    run_wayland_scanner(&scanner, "private-code", &foreign_xml, &foreign_code)?;
    run_wayland_scanner(&scanner, "client-header", &activation_xml, &activation_h)?;
    run_wayland_scanner(&scanner, "private-code", &activation_xml, &activation_code)?;

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
        .include_dir(&wl_dir)
        .cpp_file("src/bridge/wayland_parent.cpp")
        .files([
            "src/bridge/clipboard.rs",
            "src/bridge/launcher.rs",
            "src/bridge/qt_texts.rs",
            "src/bridge/surface.rs",
            "src/bridge/texts.rs",
            "src/bridge/theme.rs",
            "src/bridge/transfer.rs",
        ]);
    if let Some(qpa_dir) = qt68_qpa_include_dir() {
        builder = builder.include_dir(qpa_dir);
    }
    builder.build();
    println!("cargo::rustc-link-lib=wayland-client");

    Ok(())
}
