//! `SystemTheme`, the QML singleton that carries the desktop's look.
//!
//! It is filled once without waiting and then kept current by watchers on a
//! background runtime: the portal, the Omarchy theme folder, `kdeglobals`
//! and Hyprland's event socket. Every change is queued onto the Qt thread.

use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QColor, QString};

use crate::runtime::RUNTIME;
use crate::theme::color::Rgb;
use crate::theme::{Inputs, Look, Session, Source, hyprland, kde, omarchy, portal};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qcolor.h");
        /// Qt colour.
        type QColor = cxx_qt_lib::QColor;
    }

    extern "RustQt" {
        /// The desktop's look, for QML.
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, source)]
        #[qproperty(bool, dark)]
        #[qproperty(QColor, background)]
        #[qproperty(QColor, raised)]
        #[qproperty(QColor, ink)]
        #[qproperty(QColor, muted)]
        #[qproperty(QColor, rule)]
        #[qproperty(QColor, accent)]
        #[qproperty(QColor, accent_text, cxx_name = "accentText")]
        #[qproperty(QColor, success)]
        #[qproperty(QColor, error)]
        #[qproperty(i32, radius)]
        #[qproperty(bool, reduced_motion, cxx_name = "reducedMotion")]
        #[qproperty(QString, icon_suffix, cxx_name = "iconSuffix")]
        #[qproperty(QString, control_style, cxx_name = "controlStyle")]
        type SystemTheme = super::SystemThemeRust;
    }

    impl cxx_qt::Threading for SystemTheme {}
    impl cxx_qt::Initialize for SystemTheme {}
}

/// Backing state of [`qobject::SystemTheme`].
#[derive(Default)]
pub struct SystemThemeRust {
    source: QString,
    dark: bool,
    background: QColor,
    raised: QColor,
    ink: QColor,
    muted: QColor,
    rule: QColor,
    accent: QColor,
    accent_text: QColor,
    success: QColor,
    error: QColor,
    radius: i32,
    reduced_motion: bool,
    icon_suffix: QString,
    control_style: QString,
    last: Option<Look>,
}

fn q(c: Rgb) -> QColor {
    QColor::from_rgb(i32::from(c.0), i32::from(c.1), i32::from(c.2))
}

impl qobject::SystemTheme {
    fn apply(mut self: Pin<&mut Self>, look: Look) {
        if self.rust().last == Some(look) {
            return;
        }
        self.as_mut().rust_mut().last = Some(look);
        let p = look.palette;
        self.as_mut().set_source(QString::from(look.source.name()));
        self.as_mut().set_dark(p.dark);
        self.as_mut().set_background(q(p.background));
        self.as_mut().set_raised(q(p.raised));
        self.as_mut().set_ink(q(p.ink));
        self.as_mut().set_muted(q(p.muted));
        self.as_mut().set_rule(q(p.rule));
        self.as_mut().set_accent(q(p.accent));
        self.as_mut().set_accent_text(q(p.accent_text));
        self.as_mut().set_success(q(p.success));
        self.as_mut().set_error(q(p.error));
        self.as_mut().set_radius(look.radius);
        self.as_mut().set_reduced_motion(look.reduced_motion);
        self.as_mut()
            .set_icon_suffix(QString::from(if look.symbolic_icons {
                "-symbolic"
            } else {
                ""
            }));
    }
}

type Shared = Arc<Mutex<Inputs>>;
type Thread = cxx_qt::CxxQtThread<qobject::SystemTheme>;

/// Recompute the look and hand it to the Qt thread.
fn push(inputs: &Shared, thread: &Thread, change: impl FnOnce(&mut Inputs)) {
    let look = match inputs.lock() {
        Ok(mut i) => {
            change(&mut i);
            i.look()
        }
        Err(_) => return,
    };
    let _ = thread.queue(move |obj| obj.apply(look));
}

/// Wait briefly for a first answer, so the window opens in the right colours.
fn first<T>(fut: impl std::future::Future<Output = Option<T>>) -> Option<T> {
    RUNTIME
        .block_on(async { tokio::time::timeout(Duration::from_millis(300), fut).await })
        .ok()
        .flatten()
}

impl cxx_qt::Initialize for qobject::SystemTheme {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .set_control_style(cxx_qt_lib::QQuickStyle::name());
        let session = Session::current();
        let mut inputs = Inputs::new(&session);
        if let Some(a) = first(portal::current()) {
            inputs.portal = a;
        }
        let hypr = hyprland::running();
        if hypr {
            inputs.hyprland_rounding = first(hyprland::rounding());
        }
        self.as_mut().apply(inputs.look());

        let source = inputs.source;
        let shared: Shared = Arc::new(Mutex::new(inputs));
        let thread = self.qt_thread();

        // Portal: colour scheme, accent and reduced motion, everywhere.
        {
            let (s, t) = (shared.clone(), thread.clone());
            RUNTIME.spawn(portal::watch(move |a| push(&s, &t, |i| i.portal = a)));
        }

        // Hyprland: rounding, after every configuration reload.
        if hypr {
            let (s, t) = (shared.clone(), thread.clone());
            RUNTIME.spawn(async move {
                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                tokio::spawn(hyprland::on_config_reload(move || {
                    let _ = tx.send(());
                }));
                while rx.recv().await.is_some() {
                    let r = hyprland::rounding().await;
                    push(&s, &t, |i| i.hyprland_rounding = r);
                }
            });
        }

        // Omarchy: the theme folder; KDE: kdeglobals.
        let watched = match source {
            Source::Omarchy => omarchy::roots()
                .into_iter()
                .filter(|r| r.exists())
                .collect(),
            Source::System if session.is_kde() => kde::kdeglobals()
                .and_then(|p| p.parent().map(|d| vec![d.to_path_buf()]))
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        if !watched.is_empty() {
            watch_files(watched, source, shared, thread);
        }
    }
}

/// Watch `dirs` and read the theme again after changes settle.
fn watch_files(dirs: Vec<std::path::PathBuf>, source: Source, shared: Shared, thread: Thread) {
    use notify::{RecursiveMode, Watcher};

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            let relevant = match source {
                Source::Omarchy => true,
                _ => event
                    .paths
                    .iter()
                    .any(|p| p.file_name().is_some_and(|n| n == "kdeglobals")),
            };
            if relevant {
                let _ = tx.send(());
            }
        }
    });
    let Ok(mut watcher) = watcher else { return };
    let mode = match source {
        Source::Omarchy => RecursiveMode::Recursive,
        _ => RecursiveMode::NonRecursive,
    };
    for d in &dirs {
        let _ = watcher.watch(d, mode);
    }

    RUNTIME.spawn(async move {
        let _keep = watcher;
        while rx.recv().await.is_some() {
            // A theme switch rewrites many files; read once it is done.
            tokio::time::sleep(Duration::from_millis(150)).await;
            while rx.try_recv().is_ok() {}
            match source {
                Source::Omarchy => {
                    let p = omarchy::theme_dir().and_then(|d| omarchy::read(&d));
                    if p.is_some() {
                        push(&shared, &thread, |i| i.omarchy = p);
                    }
                }
                _ => {
                    let g = kde::read();
                    push(&shared, &thread, |i| i.kde_globals = g);
                }
            }
        }
    });
}
