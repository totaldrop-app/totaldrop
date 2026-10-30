//! GTK colours, for desktops whose look is GTK's: GNOME, and Hyprland or
//! others without a Qt platform theme.
//!
//! libadwaita draws GNOME with a fixed palette, light or dark, and takes
//! the accent from the portal; a person may override any named colour in
//! `~/.config/gtk-4.0/gtk.css` with `@define-color`. Other desktops pick a
//! GTK theme by name (`gtk-theme`), whose `gtk.css` names its colours the
//! same way (GTK 4) or with the older `theme_*` names (GTK 3). This module
//! turns those into a [`Palette`]; Adwaita stands in for anything missing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::color::{Palette, Rgb, best_text};

/// The least contrast GTK accepts for text on the accent: libadwaita puts
/// white on its blue (#3584e4) at 4.1:1, as buttons with large text.
pub const ACCENT_TEXT_CONTRAST: f64 = 3.0;

/// Text on `accent`: white, as libadwaita draws it, unless that does not
/// read; then whichever of the palette's colours reads best.
pub fn accent_text(accent: Rgb, p: &Palette) -> Rgb {
    if Rgb::WHITE.contrast(accent) >= ACCENT_TEXT_CONTRAST {
        Rgb::WHITE
    } else {
        best_text(accent, &[p.background, p.ink, Rgb::WHITE, Rgb::BLACK])
    }
}

/// libadwaita's own palette (libadwaita 1.6 and later).
pub fn adwaita(dark: bool) -> Palette {
    let c = |s: &str| Rgb::parse(s).unwrap_or(Rgb::BLACK);
    if dark {
        Palette {
            dark,
            background: c("#222226"),
            raised: c("#343437"),
            ink: c("#ffffff"),
            muted: c("#a6a6a8"),
            rule: c("#3e3e42"),
            accent: c("#3584e4"),
            accent_text: c("#ffffff"),
            success: c("#78e9ab"),
            error: c("#ff938c"),
        }
    } else {
        Palette {
            dark,
            background: c("#fafafb"),
            raised: c("#ffffff"),
            ink: c("#323235"),
            muted: c("#6e6e71"),
            rule: c("#dcdcde"),
            accent: c("#3584e4"),
            accent_text: c("#ffffff"),
            success: c("#1b8553"),
            error: c("#c01c28"),
        }
    }
}

/// Named colours from a `gtk.css`: `@define-color name value;`.
///
/// Values are `#rgb`, `#rrggbb`, `rgb()`, `rgba()` or a reference to a
/// colour defined earlier (`@name`). A translucent colour is laid over
/// `under`, as GTK draws it on the window. Anything else (`shade()`,
/// `mix()`, `alpha()` and the like) is skipped.
pub fn define_colors(css: &str, under: Rgb) -> HashMap<String, Rgb> {
    let mut out: HashMap<String, Rgb> = HashMap::new();
    for statement in css.split(';') {
        let s = strip_comments(statement);
        let Some(rest) = s.trim().strip_prefix("@define-color") else {
            continue;
        };
        let mut parts = rest.trim().splitn(2, char::is_whitespace);
        let (Some(name), Some(value)) = (parts.next(), parts.next()) else {
            continue;
        };
        let value = value.trim();
        let colour = match value.strip_prefix('@') {
            Some(other) => out.get(other).copied(),
            None => css_colour(value, under),
        };
        if let Some(c) = colour {
            out.insert(name.to_owned(), c);
        }
    }
    out
}

fn strip_comments(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = match rest[start..].find("*/") {
            Some(end) => &rest[start + end + 2..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

/// One CSS colour value, laid over `under` if translucent.
fn css_colour(value: &str, under: Rgb) -> Option<Rgb> {
    let v = value.trim();
    if let Some(hex) = v.strip_prefix('#') {
        let full: String = match hex.len() {
            3 => hex.chars().flat_map(|c| [c, c]).collect(),
            6 => hex.to_owned(),
            _ => return None,
        };
        return Rgb::parse(&format!("#{full}"));
    }
    let inner = v
        .strip_prefix("rgba(")
        .or_else(|| v.strip_prefix("rgb("))?
        .strip_suffix(')')?;
    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
    let channel = |s: &str| -> Option<f64> {
        match s.strip_suffix('%') {
            Some(p) => p.parse::<f64>().ok().map(|p| p / 100.0),
            None => s.parse::<f64>().ok().map(|n| n / 255.0),
        }
    };
    let (r, g, b) = match parts.as_slice() {
        [r, g, b] | [r, g, b, _] => (channel(r)?, channel(g)?, channel(b)?),
        _ => return None,
    };
    let alpha = match parts.get(3) {
        Some(a) => a.parse::<f64>().ok()?.clamp(0.0, 1.0),
        None => 1.0,
    };
    let solid = Rgb::from_unit(r, g, b)?;
    Some(under.mix(solid, alpha))
}

/// Overlay the named colours in `colors` on `base`. Both the GTK 4 names
/// (`window_bg_color`) and the GTK 3 ones (`theme_bg_color`) count.
pub fn apply(base: Palette, colors: &HashMap<String, Rgb>) -> Palette {
    let get = |names: &[&str]| names.iter().find_map(|n| colors.get(*n).copied());
    let mut p = base;
    if let Some(bg) = get(&["window_bg_color", "theme_bg_color"]) {
        p.background = bg;
        p.dark = bg.is_dark();
    }
    if let Some(ink) = get(&["window_fg_color", "theme_fg_color"]) {
        p.ink = ink;
    }
    match get(&["card_bg_color", "view_bg_color", "theme_base_color"]) {
        Some(card) if card != p.background => p.raised = card,
        _ if colors.is_empty() => {}
        _ => p.raised = p.background.mix(p.ink, 0.06),
    }
    if let Some(accent) = get(&["accent_bg_color", "accent_color", "theme_selected_bg_color"]) {
        p.accent = accent;
    }
    p.accent_text = match get(&["accent_fg_color", "theme_selected_fg_color"]) {
        Some(t) if t.contrast(p.accent) >= ACCENT_TEXT_CONTRAST => t,
        _ => accent_text(p.accent, &p),
    };
    if let Some(s) = get(&["success_color", "success_fg_color"]) {
        p.success = s;
    }
    if let Some(e) = get(&["error_color", "destructive_color", "error_fg_color"]) {
        p.error = e;
    }
    // Derived roles keep their contrast whatever the theme gave.
    if p.muted.contrast(p.background) < 4.5 || !colors.is_empty() {
        p.muted = p.ink.mix(p.background, 0.35);
        if p.muted.contrast(p.background) < 4.5 {
            p.muted = p.ink.mix(p.background, 0.2);
        }
    }
    if !colors.is_empty() {
        p.rule = p.background.mix(p.ink, 0.14);
    }
    p
}

/// `~/.config/gtk-4.0/gtk.css`: the person's own overrides, which
/// libadwaita applies too.
pub fn user_css() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("gtk-4.0/gtk.css"))
}

/// The `gtk.css` of the GTK theme called `name`, GTK 4 first, in the
/// places GTK looks.
pub fn theme_css(name: &str, dark: bool) -> Option<PathBuf> {
    if name.is_empty() || name.contains('/') || name.starts_with("Adwaita") {
        return None;
    }
    let mut roots = Vec::new();
    if let Some(d) = dirs::data_dir() {
        roots.push(d.join("themes"));
    }
    if let Some(h) = dirs::home_dir() {
        roots.push(h.join(".themes"));
    }
    roots.push(PathBuf::from("/usr/share/themes"));
    let files: &[&str] = if dark {
        &[
            "gtk-4.0/gtk-dark.css",
            "gtk-4.0/gtk.css",
            "gtk-3.0/gtk-dark.css",
            "gtk-3.0/gtk.css",
        ]
    } else {
        &["gtk-4.0/gtk.css", "gtk-3.0/gtk.css"]
    };
    roots
        .iter()
        .flat_map(|r| files.iter().map(move |f| r.join(name).join(f)))
        .find(|p| p.is_file())
}

/// `~/.config/gtk-3.0/settings.ini`, where GTK settings tools write the
/// theme's name.
pub fn settings_ini() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("gtk-3.0/settings.ini"))
}

/// The GTK theme's name from [`settings_ini`], for when the portal does
/// not say.
pub fn settings_theme() -> Option<String> {
    let path = settings_ini()?;
    let text = std::fs::read_to_string(path).ok()?;
    text.lines().find_map(|l| {
        let (k, v) = l.split_once('=')?;
        (k.trim() == "gtk-theme-name").then(|| v.trim().trim_matches('"').to_owned())
    })
}

/// The palette: Adwaita in the given scheme, then the GTK theme `theme`
/// (outside GNOME, where libadwaita ignores it), then the person's own
/// overrides.
pub fn palette(dark: bool, theme: Option<&str>, gnome: bool) -> Palette {
    let mut p = adwaita(dark);
    let read = |path: &Path, under: Rgb| {
        std::fs::read_to_string(path)
            .map(|css| define_colors(&css, under))
            .unwrap_or_default()
    };
    if !gnome && let Some(path) = theme.and_then(|t| theme_css(t, dark)) {
        p = apply(p, &read(&path, p.background));
    }
    if let Some(path) = user_css() {
        p = apply(p, &read(&path, p.background));
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adwaita_meets_contrast() {
        for dark in [false, true] {
            let p = adwaita(dark);
            assert!(p.ink.contrast(p.background) >= 4.5);
            assert!(p.muted.contrast(p.background) >= 4.5);
            assert!(p.accent_text.contrast(p.accent) >= ACCENT_TEXT_CONTRAST);
        }
    }

    #[test]
    fn reads_define_color() {
        let css = "/* a theme */\n@define-color window_bg_color #242424;\n\
                   @define-color window_fg_color rgba(255, 255, 255, 1);\n\
                   @define-color card_bg_color rgba(255,255,255,0.08);\n\
                   @define-color accent_bg_color @blue_3;\n\
                   @define-color blue_3 #3584e4;\n\
                   @define-color headerbar_bg_color shade(#303030, 1.1);\n\
                   @define-color theme_selected_bg_color #f0f;";
        let c = define_colors(css, Rgb(0x24, 0x24, 0x24));
        assert_eq!(c["window_bg_color"].hex(), "#242424");
        assert_eq!(c["window_fg_color"].hex(), "#ffffff");
        assert_eq!(c["card_bg_color"].hex(), "#363636");
        // Defined later: not known yet when referenced.
        assert!(!c.contains_key("accent_bg_color"));
        assert!(!c.contains_key("headerbar_bg_color"));
        assert_eq!(c["theme_selected_bg_color"].hex(), "#ff00ff");
    }

    #[test]
    fn applies_a_gtk3_theme() {
        let css = "@define-color theme_bg_color #2b2e37;\n\
                   @define-color theme_fg_color #d3dae3;\n\
                   @define-color theme_base_color #383c4a;\n\
                   @define-color theme_selected_bg_color #5294e2;";
        let p = apply(adwaita(false), &define_colors(css, Rgb::WHITE));
        assert!(p.dark);
        assert_eq!(p.background.hex(), "#2b2e37");
        assert_eq!(p.raised.hex(), "#383c4a");
        assert_eq!(p.accent.hex(), "#5294e2");
        assert!(p.muted.contrast(p.background) >= 4.5);
        assert!(p.accent_text.contrast(p.accent) >= ACCENT_TEXT_CONTRAST);
    }

    #[test]
    fn nothing_defined_keeps_adwaita() {
        assert_eq!(apply(adwaita(true), &HashMap::new()), adwaita(true));
    }

    #[test]
    fn adwaita_named_themes_are_libadwaita() {
        assert_eq!(theme_css("Adwaita", false), None);
        assert_eq!(theme_css("Adwaita-dark", true), None);
        assert_eq!(theme_css("../etc", true), None);
    }
}
