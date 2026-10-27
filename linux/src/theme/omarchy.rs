//! Omarchy themes.
//!
//! Omarchy keeps the active theme as a folder of per-application files. Its
//! `colors.toml` is the source of truth; older themes only carry
//! `alacritty.toml`, so that is read when `colors.toml` is missing. When
//! the person picks another theme, Omarchy rewrites the folder; the caller
//! watches [`roots`] and reads again.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::color::{Palette, Rgb, best_text};

/// Folders that hold Omarchy's `current` state, newest layout first.
pub fn roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(state) = dirs::state_dir() {
        out.push(state.join("omarchy/current"));
    }
    if let Some(config) = dirs::config_dir() {
        out.push(config.join("omarchy/current"));
    }
    out
}

/// The active theme folder, if Omarchy is installed.
pub fn theme_dir() -> Option<PathBuf> {
    roots()
        .into_iter()
        .map(|r| r.join("theme"))
        .find(|d| d.join("colors.toml").is_file() || d.join("alacritty.toml").is_file())
}

#[derive(Deserialize, Default)]
struct Colors {
    mode: Option<String>,
    accent: Option<String>,
    selection: Option<String>,
    muted: Option<String>,
    background: Option<String>,
    lighter_background: Option<String>,
    foreground: Option<String>,
    dark_foreground: Option<String>,
    red: Option<String>,
    green: Option<String>,
    blue: Option<String>,
}

fn c(v: &Option<String>) -> Option<Rgb> {
    v.as_deref().and_then(Rgb::parse)
}

/// Read the theme in `dir`.
pub fn read(dir: &Path) -> Option<Palette> {
    if let Ok(text) = std::fs::read_to_string(dir.join("colors.toml"))
        && let Ok(colors) = toml::from_str::<Colors>(&text)
        && let Some(p) = palette(&colors)
    {
        return Some(p);
    }
    let text = std::fs::read_to_string(dir.join("alacritty.toml")).ok()?;
    palette(&from_alacritty(&text)?)
}

fn from_alacritty(text: &str) -> Option<Colors> {
    let v: toml::Value = toml::from_str(text).ok()?;
    let colors = v.get("colors")?;
    let get = |table: &str, key: &str| {
        colors
            .get(table)
            .and_then(|t| t.get(key))
            .and_then(|s| s.as_str())
            .map(str::to_owned)
    };
    Some(Colors {
        background: get("primary", "background"),
        foreground: get("primary", "foreground"),
        accent: get("normal", "blue"),
        selection: get("selection", "background"),
        red: get("normal", "red"),
        green: get("normal", "green"),
        ..Colors::default()
    })
}

fn palette(t: &Colors) -> Option<Palette> {
    let background = c(&t.background)?;
    let ink = c(&t.foreground)?;
    let dark = match t.mode.as_deref() {
        Some("dark") => true,
        Some("light") => false,
        _ => background.is_dark(),
    };
    let fallback = Palette::aurora(dark);
    let readable = |v: Option<Rgb>, min: f64| v.filter(|v| v.contrast(background) >= min);

    let raised = c(&t.lighter_background)
        .filter(|r| *r != background)
        .unwrap_or_else(|| background.mix(ink, 0.06));
    let muted = readable(c(&t.muted), 4.5)
        .or_else(|| readable(c(&t.dark_foreground), 4.5))
        .unwrap_or_else(|| ink.mix(background, 0.3));
    let rule = c(&t.selection)
        .filter(|r| *r != background)
        .unwrap_or_else(|| background.mix(ink, 0.14));
    let accent = c(&t.accent).or_else(|| c(&t.blue)).unwrap_or(ink);
    let accent_text = best_text(accent, &[background, ink, Rgb::WHITE, Rgb::BLACK]);

    Some(Palette {
        dark,
        background,
        raised,
        ink,
        muted,
        rule,
        accent,
        accent_text,
        success: readable(c(&t.green), 3.0).unwrap_or(fallback.success),
        error: readable(c(&t.red), 3.0).unwrap_or(fallback.error),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const COLORS: &str = r##"
mode = "dark"
accent = "#ff9f0a"
selection = "#3a3a3c"
muted = "#8e8e93"
background = "#1c1c1e"
lighter_background = "#2c2c2e"
foreground = "#f5f5f7"
dark_foreground = "#98989d"
red = "#ff5f4a"
green = "#ff9f0a"
"##;

    #[test]
    fn reads_colors_toml() {
        let p = palette(&toml::from_str(COLORS).unwrap()).unwrap();
        assert!(p.dark);
        assert_eq!(p.background.hex(), "#1c1c1e");
        assert_eq!(p.raised.hex(), "#2c2c2e");
        assert_eq!(p.accent.hex(), "#ff9f0a");
        assert_eq!(p.rule.hex(), "#3a3a3c");
        assert!(p.accent_text.contrast(p.accent) >= 4.5);
        assert!(p.muted.contrast(p.background) >= 4.5);
    }

    #[test]
    fn reads_light_theme_without_mode() {
        let p = palette(
            &toml::from_str(
                "background = \"#fafafa\"\nforeground = \"#202020\"\naccent = \"#0060c0\"",
            )
            .unwrap(),
        )
        .unwrap();
        assert!(!p.dark);
        assert!(p.muted.contrast(p.background) >= 4.5);
    }

    #[test]
    fn falls_back_to_alacritty() {
        let text = "[colors.primary]\nbackground = '0x1a1b26'\nforeground = '0xa9b1d6'\n[colors.normal]\nblue = '0x7aa2f7'\nred = '0xf7768e'\ngreen = '0x9ece6a'\n";
        let p = palette(&from_alacritty(text).unwrap()).unwrap();
        assert!(p.dark);
        assert_eq!(p.accent.hex(), "#7aa2f7");
        assert_eq!(p.success.hex(), "#9ece6a");
    }
}
