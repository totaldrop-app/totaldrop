//! TotalDrop for Linux: a Qt 6 Quick interface over `totaldrop-core`.
//!
//! The window is a native Wayland client (X11 still works through Qt's xcb
//! backend). The compositor draws the frame. Colours, fonts, icons, control
//! style and motion come from the desktop; see [`theme`].

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod bridge;
mod desktop;
mod runtime;
mod settings;
mod theme;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QQuickStyle, QString, QUrl};

/// The desktop entry and Wayland app id.
pub const APP_ID: &str = "app.totaldrop.TotalDrop";

/// Where distributions install Qt 6 QML modules.
const QML_DIRS: [&str; 4] = [
    "/usr/lib/qt6/qml",
    "/usr/lib64/qt6/qml",
    "/usr/lib/x86_64-linux-gnu/qt6/qml",
    "/usr/lib/aarch64-linux-gnu/qt6/qml",
];

fn has_qml_module(rel: &str) -> bool {
    QML_DIRS
        .iter()
        .any(|d| std::path::Path::new(d).join(rel).join("qmldir").is_file())
}

/// Pick the Qt platform theme and control style before Qt starts.
///
/// - Plasma: its own platform theme (Qt loads it by itself) and the
///   `org.kde.desktop` controls, so buttons and fields are Breeze.
/// - Everywhere else, unless the person chose a platform theme: GTK's, so
///   Qt follows the desktop's icon theme, font and colour scheme. Controls
///   are Fusion, coloured from the desktop theme by the QML side.
fn choose_style(session: &theme::Session) -> &'static str {
    if session.is_kde() {
        if has_qml_module("org/kde/desktop") {
            return "org.kde.desktop";
        }
        return "Fusion";
    }
    if std::env::var_os("QT_QPA_PLATFORMTHEME").is_none() {
        // SAFETY: no other thread exists yet.
        unsafe { std::env::set_var("QT_QPA_PLATFORMTHEME", "gtk3") };
    }
    "Fusion"
}

fn main() {
    let session = theme::Session::current();
    let style = choose_style(&session);
    if std::env::var_os("QT_QUICK_CONTROLS_STYLE").is_none() {
        QQuickStyle::set_style(&QString::from(style));
    }
    QQuickStyle::set_fallback_style(&QString::from("Fusion"));

    let mut app = QGuiApplication::new();
    QGuiApplication::set_desktop_file_name(&QString::from(APP_ID));
    if let Some(mut app) = app.as_mut() {
        app.as_mut()
            .set_application_name(&QString::from("totaldrop"));
        app.as_mut()
            .set_application_display_name(&QString::from("TotalDrop"));
        app.as_mut()
            .set_application_version(&QString::from(env!("CARGO_PKG_VERSION")));
        app.as_mut()
            .set_organization_domain(&QString::from("totaldrop.app"));
    }

    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/app/totaldrop/qml/Main.qml"));
    }
    if let Some(app) = app.as_mut() {
        std::process::exit(app.exec());
    }
}
