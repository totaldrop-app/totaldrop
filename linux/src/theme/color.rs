//! Colours and the palette the interface is drawn with.

/// An opaque sRGB colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    /// White.
    pub const WHITE: Rgb = Rgb(255, 255, 255);
    /// Black.
    pub const BLACK: Rgb = Rgb(0, 0, 0);

    /// Parse `#rrggbb`, `rrggbb` or `0xrrggbb`, as theme files write them.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().trim_matches('"').trim_matches('\'');
        let hex = s
            .strip_prefix('#')
            .or_else(|| s.strip_prefix("0x"))
            .unwrap_or(s);
        let hex = hex.get(..6)?; // ignore an alpha suffix
        if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let v = u32::from_str_radix(hex, 16).ok()?;
        Some(Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8))
    }

    /// Parse KDE's `r,g,b` form.
    pub fn parse_kde(s: &str) -> Option<Self> {
        let mut it = s.split(',').map(|p| p.trim().parse::<u8>());
        let (r, g, b) = (it.next()?.ok()?, it.next()?.ok()?, it.next()?.ok()?);
        Some(Rgb(r, g, b))
    }

    /// `#rrggbb`.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }

    fn linear(c: u8) -> f64 {
        let c = f64::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// WCAG relative luminance, 0 to 1.
    pub fn luminance(self) -> f64 {
        0.2126 * Self::linear(self.0)
            + 0.7152 * Self::linear(self.1)
            + 0.0722 * Self::linear(self.2)
    }

    /// WCAG contrast ratio, 1 to 21.
    pub fn contrast(self, other: Rgb) -> f64 {
        let (a, b) = (self.luminance() + 0.05, other.luminance() + 0.05);
        if a > b { a / b } else { b / a }
    }

    /// Whether text on this colour should be light.
    pub fn is_dark(self) -> bool {
        self.contrast(Rgb::WHITE) > self.contrast(Rgb::BLACK)
    }

    /// `self` moved towards `other` by `t` (0 to 1).
    pub fn mix(self, other: Rgb, t: f64) -> Rgb {
        let m = |a: u8, b: u8| (f64::from(a) + (f64::from(b) - f64::from(a)) * t).round() as u8;
        Rgb(m(self.0, other.0), m(self.1, other.1), m(self.2, other.2))
    }

    /// From the portal's 0 to 1 channels.
    pub fn from_unit(r: f64, g: f64, b: f64) -> Option<Self> {
        let c = |v: f64| (0.0..=1.0).contains(&v).then(|| (v * 255.0).round() as u8);
        Some(Rgb(c(r)?, c(g)?, c(b)?))
    }
}

/// The candidate that reads best on `on`.
pub fn best_text(on: Rgb, candidates: &[Rgb]) -> Rgb {
    candidates
        .iter()
        .copied()
        .max_by(|a, b| a.contrast(on).total_cmp(&b.contrast(on)))
        .unwrap_or(Rgb::WHITE)
}

/// The colour roles of the interface (see the design system's tokens).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    /// Dark theme.
    pub dark: bool,
    /// Window background.
    pub background: Rgb,
    /// A calm secondary surface.
    pub raised: Rgb,
    /// Text.
    pub ink: Rgb,
    /// Secondary text, at least 4.5:1 on the background.
    pub muted: Rgb,
    /// Hairlines.
    pub rule: Rgb,
    /// The one accent per screen.
    pub accent: Rgb,
    /// Text on the accent.
    pub accent_text: Rgb,
    /// Success state.
    pub success: Rgb,
    /// Error state.
    pub error: Rgb,
}

impl Palette {
    /// Aurora, TotalDrop's own palette, used when the system gives none.
    pub fn aurora(dark: bool) -> Self {
        let c = |s: &str| Rgb::parse(s).unwrap_or(Rgb::BLACK);
        if dark {
            Self {
                dark,
                background: c("#140E33"),
                raised: c("#1E1648"),
                ink: c("#F4F2FB"),
                muted: c("#B9B1E0"),
                rule: c("#2E2566"),
                accent: c("#A68BFF"),
                accent_text: c("#140E33"),
                success: c("#7ED9B5"),
                error: c("#FFB4AB"),
            }
        } else {
            Self {
                dark,
                background: c("#F4F2FB"),
                raised: c("#E2DCF6"),
                ink: c("#22184F"),
                muted: c("#463D72"),
                rule: c("#E2DCF6"),
                accent: c("#6D4AE0"),
                accent_text: c("#FFFFFF"),
                success: c("#0A7357"),
                error: c("#BA1A1A"),
            }
        }
    }

    /// The same palette with the system's accent colour.
    pub fn with_accent(mut self, accent: Rgb) -> Self {
        self.accent = accent;
        self.accent_text = best_text(accent, &[self.background, self.ink, Rgb::WHITE, Rgb::BLACK]);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_theme_forms() {
        assert_eq!(Rgb::parse("#ff9f0a"), Some(Rgb(255, 159, 10)));
        assert_eq!(Rgb::parse("0x1C1C1E"), Some(Rgb(28, 28, 30)));
        assert_eq!(Rgb::parse("\"#a68bffcc\""), Some(Rgb(166, 139, 255)));
        assert_eq!(Rgb::parse("nope"), None);
        assert_eq!(Rgb::parse_kde("61,174,233"), Some(Rgb(61, 174, 233)));
        assert_eq!(Rgb(1, 2, 255).hex(), "#0102ff");
    }

    #[test]
    fn aurora_meets_contrast() {
        for dark in [false, true] {
            let p = Palette::aurora(dark);
            assert!(p.ink.contrast(p.background) >= 4.5);
            assert!(p.muted.contrast(p.background) >= 4.5);
            assert!(p.accent_text.contrast(p.accent) >= 4.5);
        }
    }

    #[test]
    fn accent_text_follows_accent() {
        let p = Palette::aurora(true).with_accent(Rgb(255, 159, 10));
        assert!(p.accent_text.contrast(p.accent) >= 4.5);
    }
}
