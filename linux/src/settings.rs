//! Settings kept in `~/.config/totaldrop/settings.toml`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// What the person changed. Unset means the default.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    /// Rendezvous server, a `wss://` URL.
    pub server: Option<String>,
}

fn path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("totaldrop/settings.toml"))
}

impl Settings {
    /// Read the settings, or the defaults.
    pub fn load() -> Self {
        path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| toml::from_str(&t).ok())
            .unwrap_or_default()
    }

    /// Write the settings; failures leave the old file.
    pub fn save(&self) {
        let Some(p) = path() else { return };
        let Ok(text) = toml::to_string(self) else {
            return;
        };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let tmp = p.with_extension("toml.tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, &p);
        }
    }

    /// A server URL the app accepts.
    pub fn valid_server(url: &str) -> bool {
        url.strip_prefix("wss://")
            .is_some_and(|rest| !rest.is_empty() && !rest.contains(char::is_whitespace))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_servers() {
        assert!(Settings::valid_server("wss://rendezvous.totaldrop.app"));
        assert!(!Settings::valid_server("ws://insecure.example"));
        assert!(!Settings::valid_server("wss://"));
    }
}
