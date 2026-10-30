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

/// A colour as a theme writes it. A translucent one is drawn over what
/// lies under it, which is only known once the whole palette is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colour {
    /// The colour itself.
    pub rgb: Rgb,
    /// Its opacity, 0 to 1.
    pub alpha: f64,
}

impl Colour {
    /// The colour as seen over `under`.
    pub fn over(self, under: Rgb) -> Rgb {
        under.mix(self.rgb, self.alpha)
    }
}

/// Named colours from a `gtk.css` for the `dark` or light scheme:
/// `@define-color name value;` at the top level, and the CSS variables
/// libadwaita 1.6 reads from `:root` (`--accent-bg-color: value;`, kept as
/// `accent_bg_color`), including those in a `@media
/// (prefers-color-scheme: …)` block for that scheme. Variables set for one
/// kind of widget are not the palette and are skipped.
///
/// Values are hex (`#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`), `rgb()` or
/// `rgba()` with commas or spaces, or a reference to a colour defined
/// earlier (`@name`, `var(--name)`). Anything else (`shade()`, `mix()`,
/// `color-mix()` and the like) is skipped.
pub fn define_colors(css: &str, dark: bool) -> HashMap<String, Colour> {
    let mut out: HashMap<String, Colour> = HashMap::new();
    let css = strip_comments(css);
    // The headers of the blocks the current statement is in.
    let mut blocks: Vec<String> = Vec::new();
    let mut start = 0;
    for (i, ch) in css.char_indices() {
        if !matches!(ch, ';' | '{' | '}') {
            continue;
        }
        let s = css[start..i].trim();
        start = i + ch.len_utf8();
        match ch {
            '{' => {
                blocks.push(s.to_owned());
                continue;
            }
            '}' if s.is_empty() => {
                blocks.pop();
                continue;
            }
            _ => {}
        }
        if let Some((name, value)) = declaration(s, &blocks, dark) {
            let reference = value.strip_prefix('@').or_else(|| {
                value
                    .strip_prefix("var(--")
                    .and_then(|v| v.strip_suffix(')'))
            });
            let colour = match reference {
                Some(other) => out.get(&other.trim().replace('-', "_")).copied(),
                None => css_colour(value),
            };
            if let Some(c) = colour {
                out.insert(name, c);
            }
        }
        // A block's last declaration may lack its `;`.
        if ch == '}' {
            blocks.pop();
        }
    }
    out
}

/// A colour statement `s` inside `blocks`, as its name and value, if it
/// sets the palette for the scheme.
fn declaration<'a>(s: &'a str, blocks: &[String], dark: bool) -> Option<(String, &'a str)> {
    if let Some(rest) = s.strip_prefix("@define-color") {
        if !blocks.is_empty() || !rest.starts_with(char::is_whitespace) {
            return None;
        }
        let mut parts = rest.trim().splitn(2, char::is_whitespace);
        let (name, value) = (parts.next()?, parts.next()?);
        return Some((name.to_owned(), value.trim()));
    }
    let rest = s.strip_prefix("--")?;
    let root = |b: &str| b == ":root" || b == "*";
    let scheme = |b: &str| {
        let b: String = b.chars().filter(|c| !c.is_whitespace()).collect();
        b == "@media(prefers-color-scheme:dark)" && dark
            || b == "@media(prefers-color-scheme:light)" && !dark
    };
    let for_palette = match blocks {
        [r] => root(r),
        [m, r] => scheme(m) && root(r),
        _ => false,
    };
    if !for_palette {
        return None;
    }
    let (name, value) = rest.split_once(':')?;
    Some((name.trim().replace('-', "_"), value.trim()))
}

/// Whether `colors` names an accent of its own.
pub fn defines_accent(colors: &HashMap<String, Colour>) -> bool {
    ACCENT.iter().any(|n| colors.contains_key(*n))
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

/// One CSS colour value.
fn css_colour(value: &str) -> Option<Colour> {
    let v = value.trim();
    if let Some(hex) = v.strip_prefix('#') {
        let full: String = match hex.len() {
            3 | 4 => hex.chars().flat_map(|c| [c, c]).collect(),
            6 | 8 => hex.to_owned(),
            _ => return None,
        };
        let rgb = Rgb::parse(&format!("#{}", full.get(..6)?))?;
        let alpha = match full.get(6..8) {
            Some(a) => f64::from(u8::from_str_radix(a, 16).ok()?) / 255.0,
            None => 1.0,
        };
        return Some(Colour { rgb, alpha });
    }
    let inner = v
        .strip_prefix("rgba(")
        .or_else(|| v.strip_prefix("rgb("))?
        .strip_suffix(')')?;
    // `rgb(r, g, b, a)` or `rgb(r g b / a)`.
    let (channels, alpha): (Vec<&str>, Option<&str>) = if inner.contains(',') {
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        match parts.as_slice() {
            [r, g, b] => (vec![*r, *g, *b], None),
            [r, g, b, a] => (vec![*r, *g, *b], Some(*a)),
            _ => return None,
        }
    } else {
        let (c, a) = match inner.split_once('/') {
            Some((c, a)) => (c, Some(a.trim())),
            None => (inner, None),
        };
        (c.split_whitespace().collect(), a)
    };
    let channel = |s: &str| -> Option<f64> {
        match s.strip_suffix('%') {
            Some(p) => p.parse::<f64>().ok().map(|p| p / 100.0),
            None => s.parse::<f64>().ok().map(|n| n / 255.0),
        }
    };
    let [r, g, b] = channels.as_slice() else {
        return None;
    };
    let alpha = match alpha {
        Some(a) => match a.strip_suffix('%') {
            Some(p) => p.parse::<f64>().ok()? / 100.0,
            None => a.parse::<f64>().ok()?,
        }
        .clamp(0.0, 1.0),
        None => 1.0,
    };
    Some(Colour {
        rgb: Rgb::from_unit(channel(r)?, channel(g)?, channel(b)?)?,
        alpha,
    })
}

const BACKGROUND: &[&str] = &["window_bg_color", "theme_bg_color"];
const INK: &[&str] = &["window_fg_color", "theme_fg_color"];
const CARD: &[&str] = &["card_bg_color", "view_bg_color", "theme_base_color"];
const ACCENT: &[&str] = &["accent_bg_color", "accent_color", "theme_selected_bg_color"];
const ACCENT_TEXT: &[&str] = &["accent_fg_color", "theme_selected_fg_color"];
const SUCCESS: &[&str] = &["success_color", "success_bg_color"];
const ERROR: &[&str] = &["error_color", "destructive_color", "error_bg_color"];

/// Overlay the named colours in `colors` on `base`. Both the GTK 4 names
/// (`window_bg_color`) and the GTK 3 ones (`theme_bg_color`) count. The
/// background is laid over the base's, everything else over the new
/// background.
pub fn apply(base: Palette, colors: &HashMap<String, Colour>) -> Palette {
    let get = |names: &[&str]| names.iter().find_map(|n| colors.get(*n).copied());
    let mut p = base;
    let bg = get(BACKGROUND);
    let ink = get(INK);
    if let Some(bg) = bg {
        p.background = bg.over(base.background);
        p.dark = p.background.is_dark();
    }
    if let Some(ink) = ink {
        p.ink = ink.over(p.background);
    }
    // The roles GTK does not name follow the base colours, and only them.
    let base_changed = bg.is_some() || ink.is_some();
    match get(CARD).map(|c| c.over(p.background)) {
        Some(card) if card != p.background => p.raised = card,
        _ if base_changed => p.raised = p.background.mix(p.ink, 0.06),
        _ => {}
    }
    if let Some(accent) = get(ACCENT) {
        p.accent = accent.over(p.background);
    }
    p.accent_text = match get(ACCENT_TEXT).map(|t| t.over(p.accent)) {
        Some(t) if t.contrast(p.accent) >= ACCENT_TEXT_CONTRAST => t,
        _ => accent_text(p.accent, &p),
    };
    if let Some(s) = get(SUCCESS) {
        p.success = s.over(p.background);
    }
    if let Some(e) = get(ERROR) {
        p.error = e.over(p.background);
    }
    // Derived roles keep their contrast whatever the theme gave.
    if base_changed || p.muted.contrast(p.background) < 4.5 {
        p.muted = p.ink.mix(p.background, 0.35);
        if p.muted.contrast(p.background) < 4.5 {
            p.muted = p.ink.mix(p.background, 0.2);
        }
    }
    if base_changed {
        p.rule = p.background.mix(p.ink, 0.14);
    }
    p
}

/// How deep `@import`s are followed.
const IMPORT_DEPTH: u8 = 3;

/// The text of a `gtk.css`, with the files it imports from its own folder
/// put in place of the `@import`, as many themes keep their colours in
/// `gtk-contained.css`. Imports of GResources, absolute paths and paths
/// out of the folder are left out.
pub fn read_css(path: &Path) -> String {
    read_css_at(path, IMPORT_DEPTH)
}

fn read_css_at(path: &Path, depth: u8) -> String {
    let Ok(text) = std::fs::read_to_string(path) else {
        return String::new();
    };
    let text = strip_comments(&text);
    let mut out = String::new();
    for statement in text.split(';') {
        let import = statement
            .trim()
            .strip_prefix("@import")
            .and_then(import_target)
            .filter(|_| depth > 0)
            .and_then(|t| path.parent().map(|d| d.join(t)));
        match import {
            Some(child) => out.push_str(&read_css_at(&child, depth.saturating_sub(1))),
            None => out.push_str(statement),
        }
        out.push(';');
    }
    out
}

/// The file an `@import` names, if it is a plain relative path inside the
/// importing file's folder.
fn import_target(rest: &str) -> Option<PathBuf> {
    let r = rest.trim();
    let r = r
        .strip_prefix("url(")
        .and_then(|u| u.strip_suffix(')'))
        .unwrap_or(r)
        .trim()
        .trim_matches(|c| c == '"' || c == '\'');
    if r.is_empty() || r.contains(':') || r.contains('\0') {
        return None;
    }
    let path = PathBuf::from(r);
    path.components()
        .all(|c| matches!(c, std::path::Component::Normal(_)))
        .then_some(path)
}

/// `~/.config/gtk-4.0/gtk.css`: the person's own overrides, which
/// libadwaita applies too.
pub fn user_css() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("gtk-4.0/gtk.css"))
}

/// The `gtk.css` of the GTK theme called `name`, GTK 4 first, in the
/// places GTK looks.
pub fn theme_css(name: &str, dark: bool) -> Option<PathBuf> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains(['/', '\0'])
        || name.starts_with("Adwaita")
    {
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

/// A GTK palette, and whether its files named an accent: one they name
/// wins over the portal's, as it does in GTK.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gtk {
    /// The colours.
    pub palette: Palette,
    /// The theme or the person's `gtk.css` named an accent.
    pub own_accent: bool,
}

/// The palette: Adwaita in the given scheme, then the GTK theme `theme`
/// (outside GNOME, where libadwaita ignores it), then the person's own
/// overrides.
pub fn palette(dark: bool, theme: Option<&str>, gnome: bool) -> Gtk {
    let theme_file = if gnome {
        None
    } else {
        theme.and_then(|t| theme_css(t, dark))
    };
    palette_from(dark, theme_file.as_deref(), user_css().as_deref())
}

/// [`palette`] from the given files.
pub fn palette_from(dark: bool, theme_file: Option<&Path>, user_file: Option<&Path>) -> Gtk {
    let mut g = Gtk {
        palette: adwaita(dark),
        own_accent: false,
    };
    for file in [theme_file, user_file].into_iter().flatten() {
        let colors = define_colors(&read_css(file), dark);
        g.own_accent |= defines_accent(&colors);
        g.palette = apply(g.palette, &colors);
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh folder for one test's files.
    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("totaldrop-gtk-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap_or_default();
        d
    }

    fn rgb(c: &HashMap<String, Colour>, name: &str, under: Rgb) -> String {
        c.get(name).map(|c| c.over(under).hex()).unwrap_or_default()
    }

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
        let c = define_colors(css, false);
        let under = Rgb(0x24, 0x24, 0x24);
        assert_eq!(rgb(&c, "window_bg_color", under), "#242424");
        assert_eq!(rgb(&c, "window_fg_color", under), "#ffffff");
        assert_eq!(rgb(&c, "card_bg_color", under), "#363636");
        // Defined later: not known yet when referenced.
        assert!(!c.contains_key("accent_bg_color"));
        assert!(!c.contains_key("headerbar_bg_color"));
        assert_eq!(rgb(&c, "theme_selected_bg_color", under), "#ff00ff");
    }

    #[test]
    fn comments_and_rules_do_not_hide_colours() {
        let css = "/* @define-color accent_bg_color #f00; */\n\
                   @define-color window_bg_color #101010;\n\
                   window { color: red; }\n\
                   @define-color accent_bg_color #00ff00;";
        let c = define_colors(css, false);
        assert_eq!(rgb(&c, "window_bg_color", Rgb::WHITE), "#101010");
        assert_eq!(rgb(&c, "accent_bg_color", Rgb::WHITE), "#00ff00");
    }

    #[test]
    fn reads_css_variables() {
        let css = ":root { --accent-bg-color: #e01b24; --accent-color: var(--accent-bg-color); }";
        let c = define_colors(css, false);
        assert_eq!(rgb(&c, "accent_bg_color", Rgb::WHITE), "#e01b24");
        assert_eq!(rgb(&c, "accent_color", Rgb::WHITE), "#e01b24");
        assert!(defines_accent(&c));
    }

    #[test]
    fn variables_follow_the_scheme_and_skip_widgets() {
        let css = ":root { --window-bg-color: #fafafa }\n\
                   @media (prefers-color-scheme: dark) { :root { --window-bg-color: #111111; } }\n\
                   .destructive-action { --accent-bg-color: #ff0000; }\n\
                   button { @define-color nested #00ff00; }";
        let light = define_colors(css, false);
        let dark = define_colors(css, true);
        assert_eq!(rgb(&light, "window_bg_color", Rgb::WHITE), "#fafafa");
        assert_eq!(rgb(&dark, "window_bg_color", Rgb::WHITE), "#111111");
        assert!(!defines_accent(&light) && !defines_accent(&dark));
        assert!(!light.contains_key("nested"));
    }

    #[test]
    fn reads_modern_colour_syntax() {
        let css = ":root { --window-fg-color: rgb(0 0 6 / 80%); --view-bg-color: #ffffff80; --a: #f008; }";
        let c = define_colors(css, false);
        assert_eq!(c["window_fg_color"].rgb.hex(), "#000006");
        assert!((c["window_fg_color"].alpha - 0.8).abs() < 1e-9);
        assert_eq!(rgb(&c, "view_bg_color", Rgb::BLACK), "#808080");
        assert_eq!(c["a"].rgb.hex(), "#ff0000");
    }

    #[test]
    fn applies_a_gtk3_theme() {
        let css = "@define-color theme_bg_color #2b2e37;\n\
                   @define-color theme_fg_color #d3dae3;\n\
                   @define-color theme_base_color #383c4a;\n\
                   @define-color theme_selected_bg_color #5294e2;";
        let p = apply(adwaita(false), &define_colors(css, false));
        assert!(p.dark);
        assert_eq!(p.background.hex(), "#2b2e37");
        assert_eq!(p.raised.hex(), "#383c4a");
        assert_eq!(p.accent.hex(), "#5294e2");
        assert!(p.muted.contrast(p.background) >= 4.5);
        assert!(p.accent_text.contrast(p.accent) >= ACCENT_TEXT_CONTRAST);
    }

    #[test]
    fn translucent_colours_lie_on_the_files_own_background() {
        // A dark file read for the light scheme (#174 review).
        let css = "@define-color window_bg_color #242424;\n\
                   @define-color window_fg_color #ffffff;\n\
                   @define-color card_bg_color rgba(255, 255, 255, 0.08);";
        let p = apply(adwaita(false), &define_colors(css, false));
        assert_eq!(p.raised.hex(), "#363636");
        assert!(p.ink.contrast(p.raised) >= 4.5);
    }

    #[test]
    fn an_accent_alone_changes_only_the_accent() {
        let css = "@define-color accent_bg_color #e01b24;";
        let p = apply(adwaita(false), &define_colors(css, false));
        let a = adwaita(false);
        assert_eq!(p.accent.hex(), "#e01b24");
        assert_eq!(p.raised, a.raised);
        assert_eq!(p.muted, a.muted);
        assert_eq!(p.rule, a.rule);
    }

    #[test]
    fn success_and_error_are_not_their_text() {
        let css = "@define-color error_bg_color #c01c28;\n\
                   @define-color error_fg_color #ffffff;\n\
                   @define-color success_bg_color #2ec27e;\n\
                   @define-color success_fg_color #ffffff;";
        let p = apply(adwaita(false), &define_colors(css, false));
        assert_eq!(p.error.hex(), "#c01c28");
        assert_eq!(p.success.hex(), "#2ec27e");
    }

    #[test]
    fn nothing_defined_keeps_adwaita() {
        assert_eq!(apply(adwaita(true), &HashMap::new()), adwaita(true));
    }

    #[test]
    fn follows_imports_inside_the_theme_only() {
        let d = scratch("import");
        let write = |name: &str, text: &str| std::fs::write(d.join(name), text).unwrap_or_default();
        write(
            "gtk.css",
            "@import url(\"gtk-contained.css\");\n@import url(\"../outside.css\");\n@import url(\"resource:///org/gtk/x.css\");",
        );
        write("gtk-contained.css", "@define-color theme_bg_color #2b2e37;");
        let colors = define_colors(&read_css(&d.join("gtk.css")), false);
        assert_eq!(rgb(&colors, "theme_bg_color", Rgb::WHITE), "#2b2e37");
        assert_eq!(import_target("url(\"../outside.css\")"), None);
        assert_eq!(import_target("\"/etc/passwd\""), None);
        // A file that imports itself stops.
        write(
            "loop.css",
            "@import \"loop.css\"; @define-color theme_bg_color #000000;",
        );
        let looped = define_colors(&read_css(&d.join("loop.css")), false);
        assert_eq!(rgb(&looped, "theme_bg_color", Rgb::WHITE), "#000000");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn the_persons_css_comes_after_the_theme() {
        let d = scratch("layers");
        let theme = d.join("theme.css");
        let user = d.join("user.css");
        std::fs::write(
            &theme,
            "@define-color theme_bg_color #2b2e37; @define-color theme_fg_color #d3dae3;",
        )
        .unwrap_or_default();
        std::fs::write(&user, "@define-color accent_bg_color #e01b24;").unwrap_or_default();
        let g = palette_from(false, Some(&theme), Some(&user));
        assert_eq!(g.palette.background.hex(), "#2b2e37");
        assert_eq!(g.palette.accent.hex(), "#e01b24");
        assert!(g.own_accent);
        let plain = palette_from(true, Some(&d.join("missing.css")), None);
        assert_eq!(
            plain,
            Gtk {
                palette: adwaita(true),
                own_accent: false
            }
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn adwaita_named_themes_are_libadwaita() {
        assert_eq!(theme_css("Adwaita", false), None);
        assert_eq!(theme_css("Adwaita-dark", true), None);
        assert_eq!(theme_css("../etc", true), None);
        assert_eq!(theme_css("..", true), None);
        assert_eq!(theme_css(".", true), None);
        assert_eq!(theme_css("a\0b", true), None);
    }
}
