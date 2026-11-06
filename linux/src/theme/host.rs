// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Where the desktop keeps its theme files.
//!
//! Inside Flatpak the XDG base directories point into the app's own
//! `~/.var/app/app.totaldrop.TotalDrop/`, but the desktop's theme files
//! (`kdeglobals`, Omarchy's theme folder, GTK's settings) stay in the
//! person's own folders. The Flatpak manifest (`packaging/flatpak`) exposes
//! exactly those read-only, at the same paths as on the host. Flatpak passes
//! the host's own `XDG_*_HOME` on as `HOST_XDG_*_HOME` when the host set
//! them; otherwise they are the XDG defaults under `$HOME`.
//!
//! Outside Flatpak these are the usual XDG directories.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Whether TotalDrop runs inside a Flatpak sandbox.
pub fn sandboxed() -> bool {
    Path::new("/.flatpak-info").exists()
}

/// The desktop's configuration folder, usually `~/.config`.
pub fn config_dir() -> Option<PathBuf> {
    pick(
        sandboxed(),
        std::env::var_os("HOST_XDG_CONFIG_HOME"),
        dirs::home_dir(),
        ".config",
        dirs::config_dir,
    )
}

/// The desktop's data folder, usually `~/.local/share`.
pub fn data_dir() -> Option<PathBuf> {
    pick(
        sandboxed(),
        std::env::var_os("HOST_XDG_DATA_HOME"),
        dirs::home_dir(),
        ".local/share",
        dirs::data_dir,
    )
}

/// The desktop's state folder, usually `~/.local/state`.
pub fn state_dir() -> Option<PathBuf> {
    pick(
        sandboxed(),
        std::env::var_os("HOST_XDG_STATE_HOME"),
        dirs::home_dir(),
        ".local/state",
        dirs::state_dir,
    )
}

/// Outside the sandbox, `outside`. Inside, the host's value when it is an
/// absolute path (the XDG specification ignores relative ones), otherwise
/// `default` under `home`.
fn pick(
    sandboxed: bool,
    host: Option<OsString>,
    home: Option<PathBuf>,
    default: &str,
    outside: impl FnOnce() -> Option<PathBuf>,
) -> Option<PathBuf> {
    if !sandboxed {
        return outside();
    }
    host.map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| home.map(|h| h.join(default)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home() -> Option<PathBuf> {
        Some(PathBuf::from("/home/deck"))
    }

    #[test]
    fn outside_the_sandbox_the_usual_folder() {
        let got = pick(false, Some("/elsewhere".into()), home(), ".config", || {
            Some(PathBuf::from("/home/deck/.var/app/x/config"))
        });
        assert_eq!(got, Some(PathBuf::from("/home/deck/.var/app/x/config")));
    }

    #[test]
    fn inside_the_sandbox_the_default_under_home() {
        let got = pick(true, None, home(), ".config", || {
            Some(PathBuf::from("/home/deck/.var/app/x/config"))
        });
        assert_eq!(got, Some(PathBuf::from("/home/deck/.config")));
    }

    #[test]
    fn inside_the_sandbox_the_hosts_own_value() {
        let got = pick(true, Some("/data/conf".into()), home(), ".config", || None);
        assert_eq!(got, Some(PathBuf::from("/data/conf")));
    }

    #[test]
    fn a_relative_host_value_is_ignored() {
        let got = pick(true, Some("conf".into()), home(), ".local/state", || None);
        assert_eq!(got, Some(PathBuf::from("/home/deck/.local/state")));
        let got = pick(true, Some("".into()), home(), ".local/state", || None);
        assert_eq!(got, Some(PathBuf::from("/home/deck/.local/state")));
    }

    #[test]
    fn no_home_no_folder() {
        assert_eq!(pick(true, None, None, ".config", || None), None);
    }
}
