//! Where the interface takes its look from.
//!
//! TotalDrop has no look of its own on Linux: colours, light or dark,
//! accent, corner rounding, fonts, icons and motion all come from the
//! desktop (docs/tasarim.md, "Sisteme yüzde yüz uyum"). This module decides
//! which desktop facility to follow and combines what they report into one
//! [`Look`]. Only the mark keeps its own colours.

pub mod color;
pub mod hyprland;
pub mod kde;
pub mod omarchy;
pub mod portal;

use color::Palette;

/// Which facility gives the colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// The active Omarchy theme's `colors.toml`.
    Omarchy,
    /// The Qt palette the platform theme hands every application: Plasma,
    /// qt6ct or hyprqt6engine. The interface reads it directly.
    System,
    /// The portal's colour scheme and accent over the Aurora palette.
    Portal,
    /// Aurora only (forced with `TOTALDROP_THEME=aurora`).
    Aurora,
}

impl Source {
    /// The name QML sees.
    pub fn name(self) -> &'static str {
        match self {
            Source::Omarchy => "omarchy",
            Source::System => "system",
            Source::Portal => "portal",
            Source::Aurora => "aurora",
        }
    }
}

/// Facts about the session that decide the [`Source`].
#[derive(Clone, Debug, Default)]
pub struct Session {
    /// `TOTALDROP_THEME`, for testing a given path.
    pub forced: Option<String>,
    /// `XDG_CURRENT_DESKTOP`.
    pub desktop: String,
    /// `QT_QPA_PLATFORMTHEME`.
    pub platform_theme: String,
    /// An Omarchy theme folder exists.
    pub omarchy: bool,
}

impl Session {
    /// Read the facts from the environment.
    pub fn current() -> Self {
        let var = |k| std::env::var(k).unwrap_or_default();
        Self {
            forced: std::env::var("TOTALDROP_THEME")
                .ok()
                .filter(|s| !s.is_empty()),
            desktop: var("XDG_CURRENT_DESKTOP"),
            platform_theme: var("QT_QPA_PLATFORMTHEME"),
            omarchy: omarchy::theme_dir().is_some(),
        }
    }

    /// A KDE Plasma session.
    pub fn is_kde(&self) -> bool {
        self.desktop
            .split(':')
            .any(|d| d.eq_ignore_ascii_case("kde"))
    }

    /// A platform theme that sets the Qt palette for every application.
    pub fn has_qt_palette(&self) -> bool {
        let t = self.platform_theme.to_ascii_lowercase();
        ["kde", "qt6ct", "qt5ct", "hyprqt6engine"]
            .iter()
            .any(|known| t.split(';').any(|p| p.trim() == *known))
    }

    /// Decide the source.
    pub fn source(&self) -> Source {
        match self.forced.as_deref() {
            Some("omarchy") => return Source::Omarchy,
            Some("system" | "kde") => return Source::System,
            Some("portal" | "hyprland") => return Source::Portal,
            Some("aurora") => return Source::Aurora,
            _ => {}
        }
        if self.is_kde() {
            Source::System
        } else if self.omarchy {
            Source::Omarchy
        } else if self.has_qt_palette() {
            Source::System
        } else {
            Source::Portal
        }
    }
}

/// Everything the interface needs, combined from all inputs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// Where the colours come from.
    pub source: Source,
    /// The colours. With [`Source::System`] the interface takes the Qt
    /// palette instead; success and error still come from here.
    pub palette: Palette,
    /// Corner radius for cards and fields, in pixels.
    pub radius: i32,
    /// Animations should stop.
    pub reduced_motion: bool,
    /// Ask for `-symbolic` icons: freedesktop themes (Adwaita, Yaru,
    /// Papirus) keep their single-colour icons under that suffix. Breeze
    /// icons are single-colour under their plain names.
    pub symbolic_icons: bool,
}

/// What each input last reported.
#[derive(Clone, Debug)]
pub struct Inputs {
    /// The decided source.
    pub source: Source,
    /// The session is KDE Plasma.
    pub kde: bool,
    /// Last Omarchy theme read.
    pub omarchy: Option<Palette>,
    /// Last portal reading.
    pub portal: portal::Appearance,
    /// Last `kdeglobals` reading.
    pub kde_globals: kde::Globals,
    /// Hyprland's rounding, if Hyprland runs.
    pub hyprland_rounding: Option<i32>,
}

/// Breeze's frame radius.
const BREEZE_RADIUS: i32 = 5;
/// Rounding used where the desktop says nothing (GNOME's cards).
const DEFAULT_RADIUS: i32 = 12;

impl Inputs {
    /// Start from what can be read without waiting.
    pub fn new(session: &Session) -> Self {
        let source = session.source();
        Self {
            source,
            kde: session.is_kde(),
            omarchy: if source == Source::Omarchy {
                omarchy::theme_dir().and_then(|d| omarchy::read(&d))
            } else {
                None
            },
            portal: portal::Appearance::default(),
            kde_globals: if session.is_kde() {
                kde::read()
            } else {
                kde::Globals::default()
            },
            hyprland_rounding: None,
        }
    }

    /// Combine the inputs.
    pub fn look(&self) -> Look {
        let portal_dark = self.portal.dark;
        let mut palette = match self.source {
            Source::Omarchy => self
                .omarchy
                .unwrap_or_else(|| Palette::aurora(portal_dark.unwrap_or(true))),
            Source::Portal => {
                let p = Palette::aurora(portal_dark.unwrap_or(false));
                match self.portal.accent {
                    Some(a) => p.with_accent(a),
                    None => p,
                }
            }
            Source::System | Source::Aurora => Palette::aurora(portal_dark.unwrap_or(false)),
        };
        if self.source == Source::System {
            if let Some(p) = self.kde_globals.positive {
                palette.success = p;
            }
            if let Some(n) = self.kde_globals.negative {
                palette.error = n;
            }
        }
        let radius = match (self.hyprland_rounding, self.kde) {
            (_, true) => BREEZE_RADIUS,
            (Some(r), false) => r,
            (None, false) => DEFAULT_RADIUS,
        };
        Look {
            source: self.source,
            palette,
            radius,
            reduced_motion: self.portal.reduced_motion || self.kde_globals.animations_off,
            symbolic_icons: !self.kde,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(desktop: &str, qpa: &str, omarchy: bool) -> Session {
        Session {
            forced: None,
            desktop: desktop.into(),
            platform_theme: qpa.into(),
            omarchy,
        }
    }

    #[test]
    fn picks_the_source() {
        assert_eq!(session("KDE", "", true).source(), Source::System);
        assert_eq!(session("Hyprland", "", true).source(), Source::Omarchy);
        assert_eq!(
            session("Hyprland", "hyprqt6engine", false).source(),
            Source::System
        );
        assert_eq!(session("Hyprland", "", false).source(), Source::Portal);
        assert_eq!(session("GNOME", "", false).source(), Source::Portal);
        let mut s = session("Hyprland", "", true);
        s.forced = Some("portal".into());
        assert_eq!(s.source(), Source::Portal);
    }

    #[test]
    fn radius_follows_the_desktop() {
        let mut i = Inputs::new(&session("Hyprland", "", false));
        assert_eq!(i.look().radius, DEFAULT_RADIUS);
        i.hyprland_rounding = Some(0);
        assert_eq!(i.look().radius, 0);
        i.kde = true;
        assert_eq!(i.look().radius, BREEZE_RADIUS);
    }

    #[test]
    fn portal_accent_and_scheme() {
        let mut i = Inputs::new(&session("GNOME", "", false));
        i.portal = portal::Appearance {
            dark: Some(true),
            accent: Some(color::Rgb(53, 132, 228)),
            reduced_motion: true,
        };
        let look = i.look();
        assert!(look.palette.dark);
        assert_eq!(look.palette.accent.hex(), "#3584e4");
        assert!(look.reduced_motion);
    }
}
