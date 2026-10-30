//! Desktop services: the portal file chooser, notifications, QR codes,
//! TotalDrop links and code entry helpers.
//!
//! Files are chosen through xdg-desktop-portal, so the desktop's own dialog
//! opens (KDE's under Plasma, GTK's under Hyprland and GNOME) and the app
//! works unchanged inside Flatpak.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};

/// Where received files go: `~/Downloads/TotalDrop` (localized by XDG).
pub fn downloads() -> PathBuf {
    dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("TotalDrop")
}

/// Decode `%XX` escapes of a file URL path.
pub fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Some(Ok(v)) = s.get(i + 1..i + 3).map(|h| u8::from_str_radix(h, 16))
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Total size of the regular files under `dir`.
pub fn folder_size(dir: &Path) -> u64 {
    let mut total = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            match e.file_type() {
                Ok(t) if t.is_dir() => stack.push(e.path()),
                Ok(t) if t.is_file() => total += e.metadata().map(|m| m.len()).unwrap_or(0),
                _ => {}
            }
        }
    }
    total
}

/// Normalize a code as typed: lowercase, spaces to dashes, invisible format
/// characters such as direction marks dropped. `None` if it cannot be a
/// code.
pub fn normalize_code(input: &str) -> Option<String> {
    let cleaned = totaldrop_proto::strip_invisible_format(input);
    let cleaned = cleaned
        .trim()
        .to_lowercase()
        .split(|c: char| c.is_whitespace() || c == '-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    totaldrop_proto::TransferCode::parse(&cleaned)
        .ok()
        .map(|c| c.to_string())
}

/// Code words that start with `prefix`.
pub fn completions(prefix: &str, limit: usize) -> Vec<&'static str> {
    let prefix = prefix.trim().to_lowercase();
    if prefix.is_empty() || prefix.chars().any(|c| !c.is_ascii_lowercase()) {
        return Vec::new();
    }
    (0..2048u16)
        .filter_map(totaldrop_proto::english_word)
        .filter(|w| w.starts_with(&prefix))
        .take(limit)
        .collect()
}

/// The link a QR code carries. The code sits after `#`, so it never
/// reaches a web server.
pub fn link_for(code: &str) -> String {
    format!("https://totaldrop.app/r#{code}")
}

/// The code in a TotalDrop link (`https://totaldrop.app/r#…`,
/// `totaldrop://r/…`) or a bare code, normalized.
pub fn code_from_link(text: &str) -> Option<String> {
    // Before the prefixes: a mark in front of the link hides them.
    let t = totaldrop_proto::strip_invisible_format(text);
    let t = t.trim();
    let raw = t
        .strip_prefix("https://totaldrop.app/r#")
        .or_else(|| t.strip_prefix("http://totaldrop.app/r#"))
        .or_else(|| t.strip_prefix("totaldrop://r/"))
        .or_else(|| t.strip_prefix("totaldrop:"))
        .unwrap_or(t);
    normalize_code(&percent_decode(raw))
}

/// QR modules for `text` as a row-major string of `1` (dark) and `0`, and
/// the side length.
pub fn qr_modules(text: &str) -> (String, i32) {
    use qrcode::{Color, EcLevel, QrCode};
    let Ok(code) = QrCode::with_error_correction_level(text, EcLevel::M) else {
        return (String::new(), 0);
    };
    let s: String = code
        .to_colors()
        .iter()
        .map(|c| if *c == Color::Dark { '1' } else { '0' })
        .collect();
    (s, i32::try_from(code.width()).unwrap_or(0))
}

/// Read the first QR code in the image at `path`, dark on light or light
/// on dark.
pub fn read_qr(path: &Path) -> Option<String> {
    let img = image::open(path).ok()?.to_luma8();
    let try_decode = |img: image::GrayImage| {
        let mut prepared = rqrr::PreparedImage::prepare(img);
        prepared
            .detect_grids()
            .into_iter()
            .find_map(|g| g.decode().ok().map(|(_, content)| content))
    };
    if let Some(c) = try_decode(img.clone()) {
        return Some(c);
    }
    let mut inverted = img;
    image::imageops::invert(&mut inverted);
    try_decode(inverted)
}

fn file_path(u: &ashpd::Uri) -> Option<PathBuf> {
    u.as_str()
        .strip_prefix("file://")
        .map(|p| PathBuf::from(percent_decode(p)))
}

/// Ask the desktop's file chooser for files to send.
pub async fn pick_files(title: &str) -> Vec<String> {
    let request = SelectedFiles::open_file()
        .title(title)
        .modal(true)
        .multiple(true)
        .send()
        .await;
    let Ok(request) = request else {
        return Vec::new();
    };
    let Ok(files) = request.response() else {
        return Vec::new();
    };
    files
        .uris()
        .iter()
        .filter_map(file_path)
        .map(|p| p.display().to_string())
        .collect()
}

/// Ask the desktop's file chooser for one image.
pub async fn pick_image(title: &str) -> Option<PathBuf> {
    let images = FileFilter::new("Images")
        .mimetype("image/png")
        .mimetype("image/jpeg")
        .mimetype("image/webp");
    let request = SelectedFiles::open_file()
        .title(title)
        .modal(true)
        .multiple(false)
        .filter(images)
        .send()
        .await
        .ok()?;
    let files = request.response().ok()?;
    files.uris().first().and_then(file_path)
}

/// A desktop notification through `org.freedesktop.Notifications`, which
/// every desktop answers (Plasma, mako and swaync on Hyprland, GNOME).
pub async fn notify(title: String, body: String) {
    let Ok(conn) = zbus::Connection::session().await else {
        return;
    };
    let hints: HashMap<&str, zbus::Variant<'_>> =
        HashMap::from([("desktop-entry", zbus::Variant::from(crate::APP_ID))]);
    let _ = conn
        .call_method(
            Some("org.freedesktop.Notifications"),
            "/org/freedesktop/Notifications",
            Some("org.freedesktop.Notifications"),
            "Notify",
            &(
                "TotalDrop",
                0u32,
                crate::APP_ID,
                title.as_str(),
                body.as_str(),
                Vec::<&str>::new(),
                hints,
                -1i32,
            ),
        )
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_from_links_and_typing() {
        assert_eq!(
            normalize_code(" 7 Abandon  ability "),
            Some("7-abandon-ability".into())
        );
        assert_eq!(normalize_code("seven words"), None);
        assert_eq!(
            code_from_link("https://totaldrop.app/r#7-abandon-ability"),
            Some("7-abandon-ability".into())
        );
        assert_eq!(
            code_from_link("totaldrop://r/12-zoo-zone-able"),
            Some("12-zoo-zone-able".into())
        );
        assert_eq!(code_from_link("https://example.com"), None);
    }

    #[test]
    fn codes_typed_with_arabic_indic_digits() {
        // Arabic, Persian and Urdu keyboards; the field then shows ASCII.
        for code in ["\u{0667}-abandon-ability", "\u{06F7} abandon ability"] {
            assert_eq!(
                normalize_code(code),
                Some("7-abandon-ability".into()),
                "{code:?}"
            );
        }
    }

    #[test]
    fn codes_pasted_from_a_right_to_left_message() {
        // The Android share text isolates the code (U+2068 … U+2069); a copy
        // of the code alone can carry the isolates or other direction marks.
        for code in [
            "\u{2068}7-abandon-ability\u{2069}",
            "\u{2066}7 abandon ability\u{2069}",
            "\u{2067}7-abandon-ability\u{2069}\u{200F}",
            "\u{200E}7-\u{202D}abandon\u{202C}-ability\u{061C}",
        ] {
            assert_eq!(
                normalize_code(code),
                Some("7-abandon-ability".into()),
                "{code:?}"
            );
        }
        assert_eq!(normalize_code("\u{2068}\u{2069}"), None);
        for link in [
            "\u{2068}totaldrop://r/7-abandon-ability\u{2069}",
            "\u{2068}https://totaldrop.app/r#7-abandon-ability\u{2069}",
            "totaldrop://r/\u{2068}7-abandon-ability\u{2069}",
            "totaldrop://r/%E2%81%A87-abandon-ability%E2%81%A9",
            "\u{FEFF}totaldrop://r/7-aban\u{200B}don-ability",
        ] {
            assert_eq!(
                code_from_link(link),
                Some("7-abandon-ability".into()),
                "{link:?}"
            );
        }
    }

    #[test]
    fn completes_words() {
        assert_eq!(completions("abo", 5), vec!["about", "above"]);
        assert!(completions("", 5).is_empty());
        assert!(completions("Ä", 5).is_empty());
    }

    #[test]
    fn qr_round_trip() {
        let (m, n) = qr_modules(&link_for("7-abandon-ability"));
        assert!(n >= 21);
        assert_eq!(m.len(), (n * n) as usize);
    }

    #[test]
    fn decodes_file_urls() {
        assert_eq!(percent_decode("/home/a%20b/%C3%A7.txt"), "/home/a b/ç.txt");
        assert_eq!(percent_decode("100%"), "100%");
    }
}
