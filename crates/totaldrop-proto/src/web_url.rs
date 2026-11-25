// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Strict validation and normalization for a received text that holds a single
//! `http://` or `https://` web link (#53).
//!
//! A link opened from a received text must show the exact destination address
//! before the browser is launched: userinfo (`user@host`), non-web schemes,
//! obfuscated IPv4 literals (`0x7f000001`, `2130706433`), backslashes, hidden
//! format/direction characters, and texts containing extra words or lines are
//! rejected, while internationalized domain names are normalized to ASCII
//! Punycode (`xn--...`) so homograph spoofing is visible in the confirmation
//! dialog.

use url::{Host, Url};

use crate::strip_invisible_format;

/// Maximum length in UTF-8 bytes accepted for a single web URL (#53).
pub const MAX_WEB_URL_BYTES: usize = 2048;

/// Returns a normalized ASCII `http://` or `https://` URL if `text` consists
/// solely of one safe web URL (ignoring surrounding ASCII whitespace), or
/// `None` otherwise.
pub fn single_web_url(text: &str) -> Option<String> {
    let trimmed = text.trim_ascii();
    if trimmed.is_empty() || trimmed.len() > MAX_WEB_URL_BYTES {
        return None;
    }
    // Reject any internal whitespace, control characters, backslashes (which
    // WHATWG URL turns into forward slashes), or invisible format/bidi marks.
    if trimmed
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
    {
        return None;
    }
    if strip_invisible_format(trimmed) != trimmed {
        return None;
    }

    let rest = if trimmed
        .as_bytes()
        .get(..8)
        .is_some_and(|p| p.eq_ignore_ascii_case(b"https://"))
    {
        trimmed.get(8..)?
    } else if trimmed
        .as_bytes()
        .get(..7)
        .is_some_and(|p| p.eq_ignore_ascii_case(b"http://"))
    {
        trimmed.get(7..)?
    } else {
        return None;
    };

    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty() || authority.contains('@') {
        return None;
    }

    let url = Url::parse(trimmed).ok()?;
    if !matches!(url.scheme(), "https" | "http") {
        return None;
    }
    if !url.username().is_empty() || url.password().is_some() {
        return None;
    }

    match url.host()? {
        Host::Domain(domain) => {
            if !valid_domain(domain) {
                return None;
            }
        }
        Host::Ipv4(ip) => {
            // Require canonical dotted-quad decimal in the raw authority so
            // integer, hex, or octal IPv4 forms cannot disguise an address.
            let raw_host = authority
                .split_once(':')
                .map_or(authority, |(host, _)| host);
            if raw_host != ip.to_string() {
                return None;
            }
        }
        Host::Ipv6(ip) => {
            // Reject IPv4-mapped IPv6 literals (`[::ffff:127.0.0.1]`,
            // `[::ffff:7f00:1]`) consistently with the IPv4 obfuscation rule.
            if ip.to_ipv4_mapped().is_some()
                || !authority.starts_with('[')
                || !authority.contains(']')
            {
                return None;
            }
        }
    }

    let normalized: String = url.into();
    if normalized.len() > MAX_WEB_URL_BYTES
        || !normalized.is_ascii()
        || normalized
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
    {
        return None;
    }
    Some(normalized)
}

fn valid_domain(domain: &str) -> bool {
    if domain.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
        || domain.contains("..")
        || domain.len() > 253
    {
        return false;
    }
    let mut last_label = "";
    for label in domain.split('.') {
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return false;
        }
        last_label = label;
    }
    !last_label.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_clean_single_http_and_https_urls() {
        assert_eq!(
            single_web_url("https://totaldrop.app/docs"),
            Some("https://totaldrop.app/docs".into())
        );
        assert_eq!(
            single_web_url("  HTTPS://Example.COM/path?q=1#top\r\n"),
            Some("https://example.com/path?q=1#top".into())
        );
        assert_eq!(
            single_web_url("http://192.168.1.10:8080/index.html"),
            Some("http://192.168.1.10:8080/index.html".into())
        );
        assert_eq!(
            single_web_url("http://[::1]:8080/status"),
            Some("http://[::1]:8080/status".into())
        );
        assert_eq!(
            single_web_url("http://localhost:3000/"),
            Some("http://localhost:3000/".into())
        );
        // '@' in path (such as profile handles) is allowed; only authority '@' is refused.
        assert_eq!(
            single_web_url("https://mastodon.social/@alice"),
            Some("https://mastodon.social/@alice".into())
        );
    }

    #[test]
    fn normalizes_idn_domains_to_visible_punycode() {
        assert_eq!(
            single_web_url("https://münchen.de/rathaus"),
            Some("https://xn--mnchen-3ya.de/rathaus".into())
        );
        // Cyrillic homograph domain becomes explicit xn-- Punycode for confirmation.
        assert_eq!(
            single_web_url("https://аррӏе.com/login"),
            Some("https://xn--80ak6aa92e.com/login".into())
        );
    }

    #[test]
    fn rejects_non_web_schemes_prose_and_spoofing_tricks() {
        for bad in [
            "",
            "   ",
            "totaldrop.app",
            "http:/é",
            "http:/éx",
            "https:/é",
            "https:/éx",
            "javascript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
            "file:///etc/passwd",
            "intent://scan/#Intent;scheme=zxing;end",
            "content://media/external/images/media/1",
            "totaldrop://r/7-harbor-rocket",
            "ftp://files.example.com/pub",
            "See https://totaldrop.app",
            "https://a.example.com https://b.example.com",
            "https://a.example.com\nhttps://b.example.com",
            "https://evil.\tcom/",
            "https://google.com@evil.example.com/",
            "https://user:pass@evil.example.com/",
            "https://@evil.example.com/",
            "https://good.example.com\\@evil.example.com/",
            "https://example.com/\u{202E}fdp.exe",
            "https://ex\u{200B}ample.com/",
            "\u{2068}https://example.com/\u{2069}",
            "http://2130706433/",
            "http://213070643%33/",
            "http://0x7f.0.0.1/",
            "http://0x7f00000%31/",
            "http://0177.0.0.1/",
            "http://0177.0.0.%31/",
            "https://127.%31/",
            "https://１２７.１/",
            "https://127。1/",
            "https://127.0.0.1./",
            "http://[::ffff:127.0.0.1]/",
            "https://[::ffff:7f00:1]/",
            "http://[0:0:0:0:0:ffff:127.0.0.1]/",
            "https://intranet/",
            "https://.",
            "https://example..com/",
            "https://-example.com/",
            "https:///",
        ] {
            assert_eq!(single_web_url(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn enforces_max_length_before_and_after_encoding() {
        let base = "https://example.com/";
        let exact = format!("{base}{}", "a".repeat(MAX_WEB_URL_BYTES - base.len()));
        assert_eq!(single_web_url(&exact), Some(exact));

        let over = format!("{base}{}", "a".repeat(MAX_WEB_URL_BYTES - base.len() + 1));
        assert_eq!(single_web_url(&over), None);

        // Raw UTF-8 under 2048 bytes that expands past 2048 bytes when percent-encoded.
        let non_ascii = format!("{base}{}", "ç".repeat(800));
        assert!(non_ascii.len() < MAX_WEB_URL_BYTES);
        assert_eq!(single_web_url(&non_ascii), None);
    }
}
