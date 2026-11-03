//! `Launcher`, what the command line and later launches ask the window to
//! show (#178). See [`crate::instance`].

use std::pin::Pin;

use cxx_qt::Threading;
use cxx_qt_lib::{QString, QStringList};

use crate::instance::{self, Request};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        /// Qt string list.
        type QStringList = cxx_qt_lib::QStringList;
    }

    #[namespace = "totaldrop"]
    unsafe extern "C++" {
        include!("totaldrop-linux/src/bridge/launcher.h");

        /// Give Qt the token to activate the window with; see `launcher.h`.
        fn set_activation_token(token: &QString);
    }

    extern "RustQt" {
        /// Files, links and screens to open.
        #[qobject]
        #[qml_element]
        type Launcher = super::LauncherRust;

        /// This process's command line, as a list of items and
        /// `--screen=…` (the same list `opened` carries).
        #[qinvokable]
        #[cxx_name = "startArgs"]
        fn start_args(self: &Launcher) -> QStringList;

        /// A later launch asks for `args`; the window comes to the front.
        #[qsignal]
        fn opened(self: Pin<&mut Launcher>, args: QStringList);
    }

    impl cxx_qt::Threading for Launcher {}
    impl cxx_qt::Initialize for Launcher {}
}

/// Backing state of [`qobject::Launcher`].
#[derive(Default)]
pub struct LauncherRust {}

fn list(items: Vec<String>) -> QStringList {
    items.iter().map(|s| QString::from(s.as_str())).collect()
}

impl qobject::Launcher {
    fn start_args(&self) -> QStringList {
        let cwd = std::env::current_dir().unwrap_or_default();
        list(Request::from_args(std::env::args_os().skip(1), &cwd).args())
    }
}

impl cxx_qt::Initialize for qobject::Launcher {
    fn initialize(self: Pin<&mut Self>) {
        let thread = self.qt_thread();
        instance::listen(move |o| {
            // Gone with the window: nothing is left to show it in.
            let _ = thread.queue(move |obj| {
                if let Some(t) = &o.token {
                    qobject::set_activation_token(&QString::from(t.as_str()));
                }
                obj.opened(list(o.request.args()));
            });
        });
    }
}
