//! KDE Plasma.
//!
//! Plasma's Qt platform theme already gives every Qt application its
//! palette, accent included, and updates it live. Two things it does not
//! carry are read here from `kdeglobals`: the positive and negative text
//! colours (for success and error) and whether animations are turned off.

use std::path::PathBuf;

use super::color::Rgb;

/// `~/.config/kdeglobals`.
pub fn kdeglobals() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("kdeglobals"))
}

/// What TotalDrop takes from `kdeglobals`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Globals {
    /// `[Colors:Window] ForegroundPositive`.
    pub positive: Option<Rgb>,
    /// `[Colors:Window] ForegroundNegative`.
    pub negative: Option<Rgb>,
    /// `[KDE] AnimationDurationFactor` is 0.
    pub animations_off: bool,
}

/// Read `kdeglobals`, or nothing if it is missing.
pub fn read() -> Globals {
    kdeglobals()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|t| parse(&t))
        .unwrap_or_default()
}

/// Parse the INI text of `kdeglobals`.
pub fn parse(text: &str) -> Globals {
    let mut out = Globals::default();
    let mut section = "";
    for line in text.lines().map(str::trim) {
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = name;
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        match (section, key) {
            ("Colors:Window", "ForegroundPositive") => out.positive = Rgb::parse_kde(value),
            ("Colors:Window", "ForegroundNegative") => out.negative = Rgb::parse_kde(value),
            ("KDE", "AnimationDurationFactor") => {
                out.animations_off = value.parse::<f64>().is_ok_and(|f| f <= 0.0);
            }
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_breeze_dark() {
        let g = parse(
            "[Colors:View]\nForegroundPositive=1,1,1\n\n[Colors:Window]\nBackgroundNormal=32,35,38\nForegroundNegative=218,68,83\nForegroundPositive=39,174,96\n\n[KDE]\nAnimationDurationFactor=0\n",
        );
        assert_eq!(g.positive, Some(Rgb(39, 174, 96)));
        assert_eq!(g.negative, Some(Rgb(218, 68, 83)));
        assert!(g.animations_off);
    }

    #[test]
    fn animations_default_on() {
        assert!(!parse("[KDE]\nAnimationDurationFactor=0.5\n").animations_off);
        assert!(!parse("").animations_off);
    }
}
