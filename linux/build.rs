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

fn pkg_config_libs(pkg: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let out = Command::new("pkg-config")
        .args(["--libs", pkg])
        .output()
        .map_err(|e| format!("failed to run pkg-config --libs {pkg}: {e}"))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!(
            "pkg-config --libs {pkg} failed ({}): {}",
            out.status,
            stderr.trim()
        )
        .into());
    }
    let flags = String::from_utf8(out.stdout)
        .map_err(|e| format!("pkg-config --libs {pkg} returned non-UTF-8 output: {e}"))?;
    Ok(flags.split_whitespace().map(str::to_owned).collect())
}

fn wayland_scanner_program() -> std::ffi::OsString {
    if let Some(env_scanner) = env::var_os("WAYLAND_SCANNER")
        && !env_scanner.is_empty()
    {
        return env_scanner;
    }
    if let Some(pc_scanner) = pkg_config_var("wayland-scanner", "wayland_scanner") {
        return pc_scanner.into();
    }
    "wayland-scanner".into()
}

fn qmake_query(var: &str) -> Option<String> {
    let candidates: Vec<std::ffi::OsString> = match env::var_os("QMAKE") {
        Some(v) if !v.is_empty() => vec![v],
        _ => vec!["qmake6".into(), "qmake".into()],
    };
    for qmake in candidates {
        let Ok(out) = Command::new(qmake).args(["-query", var]).output() else {
            continue;
        };
        if !out.status.success() {
            continue;
        }
        let Ok(val) = String::from_utf8(out.stdout) else {
            continue;
        };
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_owned());
        }
    }
    None
}

/// Returns the QtGui private-headers include directory for Qt 6.5+ so
/// `wayland_parent.cpp` always resolves `QPlatformNativeInterface` across
/// every supported Qt 6 version (#631). Fails the build loudly if private
/// headers are missing on Qt 6.5+ instead of silently disabling parenting.
fn qt6_qpa_include_dir() -> Result<Option<PathBuf>, Box<dyn Error>> {
    let Some(version) = qmake_query("QT_VERSION") else {
        return Err(
            "failed to query QT_VERSION via QMAKE/qmake6/qmake (install Qt 6 dev tools)".into(),
        );
    };
    let mut parts = version.split('.');
    let major: u32 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("unexpected QT_VERSION format: {version}"))?;
    let minor: u32 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("unexpected QT_VERSION format: {version}"))?;
    if major != 6 || minor < 5 {
        return Ok(None);
    }
    let headers = PathBuf::from(
        qmake_query("QT_INSTALL_HEADERS")
            .ok_or("failed to query QT_INSTALL_HEADERS via QMAKE/qmake6/qmake")?,
    );
    let qpa_root = headers.join("QtGui").join(&version).join("QtGui");
    if qpa_root
        .join("qpa")
        .join("qplatformnativeinterface.h")
        .is_file()
    {
        Ok(Some(qpa_root))
    } else {
        Err(format!(
            "Qt {version} private Gui headers not found at {} \
             (install qt6-base-private-dev on Debian/Ubuntu or qt6-qtbase-private-devel on Fedora)",
            qpa_root.display()
        )
        .into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-env-changed=QMAKE");
    println!("cargo::rerun-if-env-changed=WAYLAND_SCANNER");
    println!("cargo::rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo::rerun-if-env-changed=PKG_CONFIG_LIBDIR");
    println!("cargo::rerun-if-changed=src/bridge/wayland_parent.h");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR not set")?);
    let wl_dir = out_dir.join("wayland");
    fs::create_dir_all(&wl_dir)?;

    let protocols_dir = wayland_protocols_dir()?;
    let scanner = wayland_scanner_program();
    let wayland_libs = pkg_config_libs("wayland-client")?;
    let qpa_dir = qt6_qpa_include_dir()?;

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
    if let Some(qpa_dir) = qpa_dir {
        builder = builder.include_dir(qpa_dir);
    }
    builder.build();
    for flag in wayland_libs {
        if let Some(dir) = flag.strip_prefix("-L")
            && !dir.is_empty()
        {
            println!("cargo::rustc-link-search=native={dir}");
        } else if let Some(lib) = flag.strip_prefix("-l")
            && !lib.is_empty()
        {
            println!("cargo::rustc-link-lib={lib}");
        }
    }

    Ok(())
}
