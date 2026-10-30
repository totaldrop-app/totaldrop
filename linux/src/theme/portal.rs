//! xdg-desktop-portal appearance settings: colour scheme, accent colour and
//! reduced motion. Every desktop that ships a portal backend answers these
//! (GNOME, Hyprland through the GTK backend, KDE, COSMIC and others).

use ashpd::desktop::settings::{APPEARANCE_NAMESPACE, ColorScheme, ReducedMotion, Settings};
use futures_util::StreamExt;

use super::color::Rgb;

const GNOME_INTERFACE: &str = "org.gnome.desktop.interface";

/// GNOME's named accents, as libadwaita draws them.
pub fn gnome_accent(name: &str) -> Option<Rgb> {
    let hex = match name {
        "blue" => "#3584e4",
        "teal" => "#2190a4",
        "green" => "#3a944a",
        "yellow" => "#c88800",
        "orange" => "#ed5b00",
        "red" => "#e62d42",
        "pink" => "#d56199",
        "purple" => "#9141ac",
        "slate" => "#6f8396",
        _ => return None,
    };
    Rgb::parse(hex)
}

/// What the portal says.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Appearance {
    /// `Some(true)` for dark, `Some(false)` for light, `None` for no preference.
    pub dark: Option<bool>,
    /// The person's accent colour, if the desktop has one.
    pub accent: Option<Rgb>,
    /// Animations should stop.
    pub reduced_motion: bool,
}

async fn read(s: &Settings) -> Appearance {
    let dark = match s.color_scheme().await {
        Ok(ColorScheme::PreferDark) => Some(true),
        Ok(ColorScheme::PreferLight) => Some(false),
        _ => None,
    };
    let accent = match s.accent_color().await {
        Ok(c) => Rgb::from_unit(c.red(), c.green(), c.blue()),
        // Older GTK backends (as on Hyprland) only pass GNOME's own key.
        Err(_) => s
            .read::<String>(GNOME_INTERFACE, "accent-color")
            .await
            .ok()
            .and_then(|name| gnome_accent(&name)),
    };
    let reduced_motion = match s.reduced_motion().await {
        Ok(r) => r == ReducedMotion::ReducedMotion,
        Err(_) => s
            .read::<bool>(GNOME_INTERFACE, "enable-animations")
            .await
            .is_ok_and(|on| !on),
    };
    Appearance {
        dark,
        accent,
        reduced_motion,
    }
}

/// The GTK theme's name (`gtk-theme`), where the portal passes GNOME's
/// interface settings.
pub async fn gtk_theme() -> Option<String> {
    let s = Settings::new().await.ok()?;
    s.read::<String>(GNOME_INTERFACE, "gtk-theme").await.ok()
}

/// Call `on_change` with the GTK theme's name each time it changes, as
/// GNOME Tweaks, nwg-look or `gsettings` set it. Returns if the portal is
/// missing or goes away.
pub async fn watch_gtk_theme(mut on_change: impl FnMut(Option<String>)) {
    let Ok(s) = Settings::new().await else { return };
    let Ok(mut changes) = s.receive_setting_changed().await else {
        return;
    };
    while let Some(setting) = changes.next().await {
        if setting.namespace() == GNOME_INTERFACE && setting.key() == "gtk-theme" {
            on_change(s.read::<String>(GNOME_INTERFACE, "gtk-theme").await.ok());
        }
    }
}

/// Read the appearance once.
pub async fn current() -> Option<Appearance> {
    let s = Settings::new().await.ok()?;
    Some(read(&s).await)
}

/// Read the appearance now and call `on_change` with it, then again after
/// every change. Returns if the portal is missing or goes away.
pub async fn watch(mut on_change: impl FnMut(Appearance)) {
    let Ok(s) = Settings::new().await else { return };
    let mut last = read(&s).await;
    on_change(last);
    let Ok(mut changes) = s.receive_setting_changed().await else {
        return;
    };
    while let Some(setting) = changes.next().await {
        if setting.namespace() != APPEARANCE_NAMESPACE && setting.namespace() != GNOME_INTERFACE {
            continue;
        }
        let now = read(&s).await;
        if now != last {
            last = now;
            on_change(now);
        }
    }
}
