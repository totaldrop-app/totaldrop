// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! `Launcher`, what the command line and later launches ask the window to
//! show (#178), and opening a confirmed web link in the browser (#54). See
//! [`crate::instance`].

use std::pin::Pin;

use cxx_qt::Threading;
use cxx_qt_lib::{QString, QStringList};

use crate::instance::{self, Request};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        /// Qt string list.
        type QStringList = cxx_qt_lib::QStringList;
    }

    #[namespace = "totaldrop"]
    unsafe extern "C++" {
        include!("totaldrop-linux/src/bridge/launcher.h");

        /// Give Qt the token to activate the window with; see `launcher.h`.
        fn set_activation_token(token: &QString);

        /// Open a validated http(s) URL in the desktop's browser; see
        /// `launcher.h`.
        fn open_web_url_cpp(url: &QString) -> bool;
    }

    extern "RustQt" {
        /// Files, links and screens to open.
        #[qobject]
        #[qml_element]
        type Launcher = super::LauncherRust;

        /// This process's command line, as a list of items and
        /// `--screen=…` (the same list `opened` carries).
        #[qinvokable]
        #[cxx_name = "startArgs"]
        fn start_args(self: &Launcher) -> QStringList;

        /// Open a confirmed single `http://` or `https://` web link in the
        /// desktop's browser (#54). Returns `false` if `url` is not a valid
        /// single web URL or no handler opened it.
        #[qinvokable]
        #[cxx_name = "openWebUrl"]
        fn open_web_url(self: &Launcher, url: &QString) -> bool;

        /// A later launch asks for `args`; the window comes to the front.
        #[qsignal]
        fn opened(self: Pin<&mut Launcher>, args: QStringList);
    }

    impl cxx_qt::Threading for Launcher {}
    impl cxx_qt::Initialize for Launcher {}
}

/// Backing state of [`qobject::Launcher`].
#[derive(Default)]
pub struct LauncherRust {}

fn list(items: Vec<String>) -> QStringList {
    items.iter().map(|s| QString::from(s.as_str())).collect()
}

/// Maximum encoded length accepted when passing an RFC 3986 web URL to Qt.
const MAX_STRICT_WEB_URL_BYTES: usize = 8192;

/// Whether `b` may appear unencoded in the path, query or fragment of an
/// RFC 3986 URI (`pchar / "/" / "?"`, excluding `%` and `#` which need
/// context checks).
fn is_rfc3986_tail_char(b: u8) -> bool {
    b.is_ascii_alphanumeric()
        || matches!(
            b,
            b'-' | b'.'
                | b'_'
                | b'~'
                | b'!'
                | b'$'
                | b'&'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b';'
                | b'='
                | b':'
                | b'@'
                | b'/'
                | b'?'
        )
}

/// Encodes characters that WHATWG `url::Url` leaves unencoded in path, query
/// or fragment (`[`, `]`, `|`, `^`, `` ` ``, `{`, `}`, `"`, `<`, `>`, bare `%`,
/// or a second `#`) so `QUrl(url, QUrl::StrictMode)` accepts the URI while
/// preserving `scheme://authority` (including IPv6 brackets and port) and
/// existing `%XX` escapes (#54).
fn rfc3986_encode_normalized(clean: &str) -> Option<String> {
    use std::fmt::Write as _;

    let scheme_len = if clean.starts_with("https://") {
        8
    } else if clean.starts_with("http://") {
        7
    } else {
        return None;
    };
    let after_scheme = clean.get(scheme_len..)?;
    let slash_offset = after_scheme.find('/')?;
    let split_at = scheme_len.checked_add(slash_offset)?;
    let origin = clean.get(..split_at)?;
    let rest = clean.get(split_at..)?.as_bytes();
    if origin.len() > MAX_STRICT_WEB_URL_BYTES {
        return None;
    }

    let mut out = String::with_capacity(clean.len());
    out.push_str(origin);
    let mut seen_hash = false;
    for (i, &b) in rest.iter().enumerate() {
        let next1 = i.checked_add(1).and_then(|j| rest.get(j));
        let next2 = i.checked_add(2).and_then(|j| rest.get(j));
        if b == b'%'
            && next1.is_some_and(u8::is_ascii_hexdigit)
            && next2.is_some_and(u8::is_ascii_hexdigit)
        {
            out.push('%');
        } else if b == b'#' && !seen_hash {
            seen_hash = true;
            out.push('#');
        } else if is_rfc3986_tail_char(b) {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
        if out.len() > MAX_STRICT_WEB_URL_BYTES {
            return None;
        }
    }
    Some(out)
}

/// Validates `raw` with [`totaldrop_proto::single_web_url`] and encodes its
/// path, query and fragment to strict RFC 3986 form for `QUrl::StrictMode`
/// (#54).
fn rfc3986_web_url(raw: &str) -> Option<String> {
    let clean = totaldrop_proto::single_web_url(raw)?;
    rfc3986_encode_normalized(&clean)
}

/// Re-validates and normalizes `raw` with [`rfc3986_web_url`] before handing
/// it to `open`, so non-web schemes, credentials, hidden format marks or
/// malformed addresses never reach `QDesktopServices::openUrl` (#54).
fn open_validated_web_url(raw: &str, open: impl FnOnce(&str) -> bool) -> bool {
    let Some(clean) = rfc3986_web_url(raw) else {
        return false;
    };
    open(&clean)
}

impl qobject::Launcher {
    fn start_args(&self) -> QStringList {
        let cwd = std::env::current_dir().unwrap_or_default();
        list(Request::from_args(std::env::args_os().skip(1), &cwd).args())
    }

    fn open_web_url(&self, url: &QString) -> bool {
        open_validated_web_url(&url.to_string(), |clean| {
            qobject::open_web_url_cpp(&QString::from(clean))
        })
    }
}

impl cxx_qt::Initialize for qobject::Launcher {
    fn initialize(self: Pin<&mut Self>) {
        let thread = self.qt_thread();
        instance::listen(move |o| {
            // Gone with the window: nothing is left to show it in.
            let _ = thread.queue(move |obj| {
                if let Some(t) = &o.token {
                    qobject::set_activation_token(&QString::from(t.as_str()));
                }
                obj.opened(list(o.request.args()));
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{open_validated_web_url, rfc3986_encode_normalized, rfc3986_web_url};

    #[test]
    fn dangerous_or_malformed_urls_never_reach_desktop_services() {
        for bad in [
            "",
            "   ",
            "file:///etc/passwd",
            "javascript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
            "totaldrop://r/7-harbor-rocket",
            "https://user:pass@evil.example.com/",
            "https://google.com@evil.example.com/",
            "https://example.com/\u{202E}fdp.exe",
            "http://2130706433/",
            "http://0x7f.0.0.1/",
            "See https://totaldrop.app",
        ] {
            let mut called = false;
            let ok = open_validated_web_url(bad, |_| {
                called = true;
                true
            });
            assert!(!ok, "{bad:?} should return false");
            assert!(!called, "{bad:?} must not reach open_web_url_cpp");
        }
    }

    #[test]
    fn valid_web_urls_pass_normalized_punycode_and_propagate_result() {
        let mut seen = String::new();
        let ok = open_validated_web_url("  HTTPS://аррӏе.com/login\r\n", |clean| {
            seen = clean.to_owned();
            true
        });
        assert!(ok);
        assert_eq!(seen, "https://xn--80ak6aa92e.com/login");

        let failed = open_validated_web_url("https://totaldrop.app/docs", |_| false);
        assert!(!failed);
    }

    #[test]
    fn encodes_whatwg_characters_to_rfc3986_for_qurl_strict_mode() {
        let cases = [
            (
                "https://en.wikipedia.org/w/api.php?action=query&titles=A|B",
                "https://en.wikipedia.org/w/api.php?action=query&titles=A%7CB",
            ),
            (
                "https://example.com/items?filter[status]=open",
                "https://example.com/items?filter%5Bstatus%5D=open",
            ),
            (
                "https://example.com/p^1`2{3}\"4<5>6?q=^`{}\"<>#f^`{}\"<>[|]",
                "https://example.com/p%5E1%602%7B3%7D%224%3C5%3E6?q=%5E%60%7B%7D%22%3C%3E#f%5E%60%7B%7D%22%3C%3E%5B%7C%5D",
            ),
            (
                "https://example.com/100%?a=%20&b=%7c&c=%2G&d=%%20",
                "https://example.com/100%25?a=%20&b=%7c&c=%252G&d=%25%20",
            ),
            (
                "https://example.com/docs#sec#sub#third",
                "https://example.com/docs#sec%23sub%23third",
            ),
            (
                "http://[::1]:8080/path[1]|2",
                "http://[::1]:8080/path%5B1%5D%7C2",
            ),
            ("https://münchen.de/a|b", "https://xn--mnchen-3ya.de/a%7Cb"),
        ];
        for (raw, expected) in cases {
            let got = rfc3986_web_url(raw);
            assert_eq!(got.as_deref(), Some(expected), "input: {raw}");
            // Re-validating an already RFC 3986-encoded URL never double-encodes.
            assert_eq!(
                rfc3986_web_url(expected).as_deref(),
                Some(expected),
                "idempotence: {expected}"
            );
        }
    }

    #[test]
    fn rejects_urls_exceeding_strict_length_bound() {
        let prefix = "https://example.com/";
        let exact = format!("{prefix}{}", "|".repeat(2724));
        let encoded = rfc3986_encode_normalized(&exact);
        assert_eq!(encoded.as_ref().map(String::len), Some(8192));

        let too_long = format!("{prefix}{}", "|".repeat(2725));
        assert_eq!(rfc3986_encode_normalized(&too_long), None);

        let over_proto = format!("{prefix}{}", "a".repeat(2040));
        assert_eq!(rfc3986_web_url(&over_proto), None);
    }
}
