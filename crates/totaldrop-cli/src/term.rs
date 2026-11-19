// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Sanitised terminal output for the command-line client (#399).
//!
//! Every line written to standard output or standard error outside a piped
//! text transfer goes through [`Clean`], whose inner `String` is private so
//! external text (paths, environment values, peer strings, error details,
//! CLI arguments) cannot reach the terminal without [`reveal_controls`] or
//! [`reveal_text`]. `crates/totaldrop-cli/src/main.rs` denies
//! `clippy::disallowed_macros` and `clippy::disallowed_methods`, and only
//! the minimal sink functions in this module allow them.

use std::io::{BufRead, IsTerminal as _, Write};
use std::path::Path;
use std::process::ExitCode;

use totaldrop_l10n::{Arg, Strings, reveal_controls, reveal_text};
use totaldrop_proto::{CodeError, TransferCode};

/// A single-line string whose untrusted control, formatting, separator and
/// noncharacter code points have been made visible with [`reveal_controls`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clean(String);

/// An argument to [`Clean::msg`]: a numeric count, or text already sanitised
/// into a [`Clean`] value.
#[derive(Debug, Clone, Copy)]
pub enum CleanArg<'a> {
    /// A number formatted by `totaldrop-l10n` for plural selection and display.
    Count(u64),
    /// Sanitised text inserted into a placeholder.
    Text(&'a Clean),
}

impl Clean {
    /// View the sanitised string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Sanitise an arbitrary string slice with [`reveal_controls`].
    #[must_use]
    pub fn visible(raw: &str) -> Self {
        Self(reveal_controls(raw))
    }

    /// Sanitise a filesystem path's display representation.
    #[must_use]
    pub fn path(path: &Path) -> Self {
        Self(reveal_controls(&path.display().to_string()))
    }

    /// Sanitise a `Display` error value.
    #[must_use]
    pub fn error(err: &(impl std::fmt::Display + ?Sized)) -> Self {
        Self(reveal_controls(&err.to_string()))
    }

    /// Sanitise a parsed [`TransferCode`]'s word form (defence in depth).
    #[must_use]
    pub fn code(code: &TransferCode) -> Self {
        Self(reveal_controls(&code.to_string()))
    }

    /// Sanitise a parsed [`TransferCode`]'s canonical numeric form.
    #[must_use]
    pub fn canonical_code(code: &TransferCode) -> Self {
        Self(reveal_controls(&code.canonical()))
    }

    /// Format a byte count with `s` (`totaldrop-l10n`'s compiled-in catalog).
    #[must_use]
    pub fn bytes(s: &Strings, n: u64) -> Self {
        Self(s.bytes(n))
    }

    /// Format a compiled-in message `key` in `s` using only [`CleanArg`]
    /// arguments. Untrusted text in [`CleanArg::Text`] has already passed
    /// through [`reveal_controls`], so right-to-left `U+2068`…`U+2069`
    /// isolates added by `Strings::get` remain intact while untrusted Bidi
    /// controls inside the argument stay neutralised.
    #[must_use]
    pub fn msg(s: &Strings, key: &str, args: &[(&str, CleanArg<'_>)]) -> Self {
        let safe_key = reveal_controls(key);
        let raw_args: Vec<(&str, Arg<'_>)> = args
            .iter()
            .map(|(name, arg)| {
                let a = match arg {
                    CleanArg::Count(n) => Arg::Count(*n),
                    CleanArg::Text(clean) => Arg::Text(clean.as_str()),
                };
                (*name, a)
            })
            .collect();
        Self(s.get(&safe_key, &raw_args))
    }

    /// Format the indented English error detail line (`  (…)`), naming every
    /// control and hidden character the peer or server included (T17).
    #[must_use]
    pub fn detail(err: &totaldrop_core::Error) -> Self {
        Self(format!("  ({})", reveal_controls(&err.to_string())))
    }
}

/// Write a sanitised line to standard error without panicking if the terminal
/// has hung up (`EIO` on `SIGHUP`) or the pipe has closed.
#[allow(clippy::disallowed_macros, clippy::disallowed_methods)]
pub fn errln(msg: &Clean) {
    let _ = writeln!(std::io::stderr(), "{}", msg.as_str());
}

/// Write an empty line to standard error.
#[allow(clippy::disallowed_macros, clippy::disallowed_methods)]
pub fn err_blank() {
    let _ = writeln!(std::io::stderr());
}

/// Write `totaldrop: <msg>` to standard error.
#[allow(clippy::disallowed_macros, clippy::disallowed_methods)]
pub fn err_totaldrop(msg: &Clean) {
    let _ = writeln!(std::io::stderr(), "totaldrop: {}", msg.as_str());
}

/// Write two-space-indented `  <msg>` to standard error.
#[allow(clippy::disallowed_macros, clippy::disallowed_methods)]
pub fn err_indented(msg: &Clean) {
    let _ = writeln!(std::io::stderr(), "  {}", msg.as_str());
}

/// Write `totaldrop: invalid code: <err>` to standard error.
#[allow(clippy::disallowed_macros, clippy::disallowed_methods)]
pub fn err_invalid_code(err: &CodeError) {
    let safe = Clean::error(err);
    let _ = writeln!(
        std::io::stderr(),
        "totaldrop: invalid code: {}",
        safe.as_str()
    );
}

/// Update the in-place progress line on standard error, clearing any trailing
/// characters on a terminal (`ESC [ K`) and ending the line when `done`.
#[allow(clippy::disallowed_macros, clippy::disallowed_methods)]
pub fn progress(line: &Clean, done: bool) {
    // On a terminal, clear the rest of the line: a shorter translation must
    // not leave old characters behind.
    let clear = if std::io::stderr().is_terminal() {
        "\x1b[K"
    } else {
        ""
    };
    let mut err = std::io::stderr().lock();
    let _ = write!(err, "\r{}{clear}", line.as_str());
    if done {
        let _ = writeln!(err);
    }
    let _ = err.flush();
}

/// Write `totaldrop receive <code>` to standard output and flush immediately.
#[allow(clippy::disallowed_macros, clippy::disallowed_methods)]
pub fn out_receive_command(code: &TransferCode) {
    let safe = Clean::code(code);
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "totaldrop receive {}", safe.as_str());
    let _ = out.flush();
}

/// Write the `totaldrop code` summary lines (`code:` and `canonical:`) to
/// standard output.
#[allow(clippy::disallowed_macros, clippy::disallowed_methods)]
pub fn out_code(code: &TransferCode) {
    let words = Clean::code(code);
    let canonical = Clean::canonical_code(code);
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "code:      {}", words.as_str());
    let _ = writeln!(out, "canonical: {}", canonical.as_str());
    let _ = out.flush();
}

/// A received text as the terminal should show it ([`reveal_text`]); into a
/// pipe or a file it goes as it came (#313).
#[must_use]
pub fn shown_text(text: &str, terminal: bool) -> String {
    if terminal {
        reveal_text(text)
    } else {
        text.to_owned()
    }
}

/// Write a received text to `out`, sanitising hidden characters and ensuring a
/// final newline when `terminal` is `true`, or writing the exact UTF-8 bytes
/// unchanged when piped (#313).
#[allow(clippy::disallowed_macros, clippy::disallowed_methods)]
fn write_received_text(out: &mut impl Write, text: &str, terminal: bool) -> std::io::Result<()> {
    write!(out, "{}", shown_text(text, terminal))?;
    if terminal && !text.ends_with('\n') {
        writeln!(out)?;
    }
    out.flush()
}

/// Write a received text to standard output, sanitising it if standard output
/// is a terminal (#313).
#[allow(clippy::disallowed_methods)]
pub fn write_received_stdout(text: &str) -> std::io::Result<()> {
    let terminal = std::io::stdout().is_terminal();
    let mut out = std::io::stdout().lock();
    write_received_text(&mut out, text, terminal)
}

/// Ask `prompt` on `out` and read one line of answer from `input`: `None` on
/// a read error, empty at the end of input. A terminal echoes what is
/// `typed`, Enter included; an answer from a pipe or a file is not echoed,
/// and Ctrl-D echoes no Enter, so then the line is ended here and what comes
/// next does not follow the question (#163).
#[allow(clippy::disallowed_macros, clippy::disallowed_methods)]
pub fn ask(
    prompt: &Clean,
    input: &mut impl BufRead,
    out: &mut impl Write,
    typed: bool,
) -> Option<String> {
    let _ = write!(out, "{}", prompt.as_str());
    let _ = out.flush();
    let mut line = String::new();
    let read = input.read_line(&mut line);
    if !typed || !line.ends_with('\n') {
        let _ = writeln!(out);
    }
    read.ok().map(|_| line)
}

/// Ask `prompt` on standard error and read one line from standard input.
#[allow(clippy::disallowed_methods)]
pub fn ask_stderr(prompt: &Clean) -> Option<String> {
    // The answer shows in the log only when both are terminals.
    let typed = std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
    ask(
        prompt,
        &mut std::io::stdin().lock(),
        &mut std::io::stderr(),
        typed,
    )
}

/// Render a Clap help, version or parse error with [`reveal_text`] so
/// untrusted CLI tokens, `argv[0]` names and environment values cannot send
/// control or Bidi override characters to the terminal (#399), then return
/// Clap's exit code.
#[allow(clippy::disallowed_macros, clippy::disallowed_methods)]
pub fn exit_clap(err: &clap::Error) -> ExitCode {
    // `help()` sets `Styles::plain()` so Clap emits no ANSI sequences of its
    // own; `.ansi()` bypasses `anstream`'s stripper so untrusted 7-bit ESC
    // sequences in arguments or `argv[0]` reach `reveal_text` intact (#399).
    let safe = reveal_text(&err.render().ansi().to_string());
    if err.use_stderr() {
        let mut out = std::io::stderr().lock();
        let _ = write!(out, "{safe}");
        if !safe.ends_with('\n') {
            let _ = writeln!(out);
        }
        let _ = out.flush();
    } else {
        let mut out = std::io::stdout().lock();
        let _ = write!(out, "{safe}");
        if !safe.ends_with('\n') {
            let _ = writeln!(out);
        }
        let _ = out.flush();
    }
    ExitCode::from(u8::try_from(err.exit_code()).unwrap_or(2))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::path::Path;

    use super::{Clean, CleanArg, shown_text, write_received_text};
    use totaldrop_l10n::{Domain, Strings, is_hidden};
    use totaldrop_proto::{CodeError, TransferCode};

    #[test]
    fn clean_constructors_name_every_class_of_hidden_character() {
        let hostile = concat!(
            "\x00\x07\x08\n\r\x1b[31m\x7f",
            "\u{0085}\u{009b}2J",
            "\u{202a}\u{202b}\u{202c}\u{202d}\u{202e}",
            "\u{2066}\u{2067}\u{2068}\u{2069}",
            "\u{200e}\u{200f}\u{061c}",
            "\u{2028}\u{2029}\u{fdd0}\u{ffff}",
        );

        let vis = Clean::visible(hostile);
        assert!(!vis.as_str().chars().any(is_hidden), "{:?}", vis.as_str());
        for expected in [
            "<U+0000>", "<U+0007>", "<U+0008>", "<U+000A>", "<U+000D>", "<U+001B>", "<U+007F>",
            "<U+0085>", "<U+009B>", "<U+202E>", "<U+2066>", "<U+2068>", "<U+2069>", "<U+200E>",
            "<U+200F>", "<U+061C>", "<U+2028>", "<U+2029>", "<U+FDD0>", "<U+FFFF>",
        ] {
            assert!(
                vis.as_str().contains(expected),
                "missing {expected} in {}",
                vis.as_str()
            );
        }

        let path = Clean::path(Path::new("/tmp/invoice\u{202e}\x1b[2Jfdp.exe"));
        assert!(!path.as_str().chars().any(is_hidden));
        assert!(
            path.as_str().ends_with("invoice<U+202E><U+001B>[2Jfdp.exe"),
            "{}",
            path.as_str()
        );

        let io_err = std::io::Error::other(hostile);
        let clean_io = Clean::error(&io_err);
        assert!(!clean_io.as_str().chars().any(is_hidden));
        assert!(clean_io.as_str().contains("<U+202E>"));

        let code_err = CodeError::UnknownWord(format!("bad{hostile}"));
        let clean_code_err = Clean::error(&code_err);
        assert!(!clean_code_err.as_str().chars().any(is_hidden));

        let parsed = TransferCode::parse("7-harbor-rocket").unwrap();
        assert_eq!(Clean::code(&parsed).as_str(), "7-harbor-rocket");
        assert_eq!(Clean::canonical_code(&parsed).as_str(), "7-840-1499");
    }

    #[test]
    fn clean_msg_neutralises_arguments_while_keeping_rtl_isolates() {
        let strings = |tag: &str| {
            Strings::new(
                Domain::Cli,
                &[totaldrop_l10n::locale::parse(tag).unwrap()],
                None,
            )
        };
        let folder = Clean::path(Path::new("/tmp/dir-\x1b[31m-\u{202e}\u{2068}evil"));

        let en = strings("en");
        let en_line = Clean::msg(&en, "cliNoFolder", &[("folder", CleanArg::Text(&folder))]);
        assert!(!en_line.as_str().chars().any(is_hidden), "{en_line:?}");
        assert!(
            en_line
                .as_str()
                .contains("/tmp/dir-<U+001B>[31m-<U+202E><U+2068>evil")
        );

        // In the RTL pseudo-locale `ar-XB`, `Strings::get` wraps inserted
        // `Arg::Text` in trusted U+2068…U+2069 isolates, while any untrusted
        // controls or isolates inside `folder` have already been turned into
        // literal `<U+…>` tags by `Clean::path`.
        let rtl = strings("ar-XB");
        assert!(rtl.is_rtl());
        let rtl_line = Clean::msg(&rtl, "cliNoFolder", &[("folder", CleanArg::Text(&folder))]);
        assert!(
            rtl_line
                .as_str()
                .contains("\u{2068}/tmp/dir-<U+001B>[31m-<U+202E><U+2068>evil\u{2069}"),
            "{:?}",
            rtl_line.as_str()
        );
        let without_trusted_isolates: String = rtl_line
            .as_str()
            .chars()
            .filter(|c| !matches!(c, '\u{2068}' | '\u{2069}'))
            .collect();
        assert!(!without_trusted_isolates.chars().any(is_hidden));

        let unknown = Clean::msg(&en, "missing-\x1b-\u{202e}", &[]);
        assert_eq!(unknown.as_str(), "missing-<U+001B>-<U+202E>");
    }

    #[test]
    fn write_received_text_sanitises_on_a_terminal_and_passes_raw_to_a_pipe() {
        let raw = "hello \x1b[31m\u{202e}world";
        let mut tty = Vec::new();
        write_received_text(&mut tty, raw, true).unwrap();
        assert_eq!(
            String::from_utf8(tty).unwrap(),
            "hello <U+001B>[31m<U+202E>world\n"
        );
        assert_eq!(shown_text(raw, true), "hello <U+001B>[31m<U+202E>world");

        let mut pipe = Vec::new();
        write_received_text(&mut pipe, raw, false).unwrap();
        assert_eq!(pipe, raw.as_bytes());
    }

    #[test]
    fn compile_time_terminal_gate_stays_armed() {
        fn collect_rs(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    collect_rs(&path, out);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    out.push(path);
                }
            }
        }

        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let term_rs = src_dir.join("term.rs");
        let main_rs = src_dir.join("main.rs");
        let mut files = Vec::new();
        collect_rs(&src_dir, &mut files);
        files.sort();
        assert!(files.contains(&main_rs), "main.rs not found in {src_dir:?}");
        assert!(files.contains(&term_rs), "term.rs not found in {src_dir:?}");

        let deny_line = concat!(
            "#![deny(clippy::disallowed_macros, ",
            "clippy::disallowed_methods)]"
        );
        let disallowed_prefix = concat!("clippy::disallowed_", "");
        for file in &files {
            if file == &term_rs {
                continue;
            }
            let text = std::fs::read_to_string(file).unwrap();
            let mut saw_root_deny = false;
            for (idx, line) in text.lines().enumerate() {
                if file == &main_rs && line.trim() == deny_line {
                    assert!(!saw_root_deny, "duplicate root deny in main.rs");
                    saw_root_deny = true;
                    continue;
                }
                assert!(
                    !line.contains(disallowed_prefix),
                    "{}:{} mentions {disallowed_prefix}: {line}",
                    file.display(),
                    idx + 1,
                );
            }
            if file == &main_rs {
                assert!(saw_root_deny, "crate-root deny missing from main.rs");
            }
        }

        let clippy_toml = include_str!("../../../clippy.toml");
        for required in [
            "std::print",
            "std::println",
            "std::eprint",
            "std::eprintln",
            "std::dbg",
            "core::write",
            "core::writeln",
            "std::io::stdout",
            "std::io::stderr",
            "std::io::Write::write",
            "std::io::Write::write_vectored",
            "std::io::Write::write_all",
            "std::io::Write::write_fmt",
            "clap::Command::get_matches",
            "clap::Command::get_matches_from",
            "clap::Command::get_matches_mut",
            "clap::Command::print_help",
            "clap::Command::print_long_help",
            "clap::Parser::parse",
            "clap::Parser::parse_from",
            "clap::Parser::update_from",
            "clap::error::Error::exit",
            "clap::error::Error::print",
        ] {
            assert!(
                clippy_toml.contains(&format!("\"{required}\"")),
                "clippy.toml missing {required}"
            );
        }

        for (manifest, toml_src) in [
            ("Cargo.toml", include_str!("../../../Cargo.toml")),
            (
                "linux/Cargo.toml",
                include_str!("../../../linux/Cargo.toml"),
            ),
            (
                "app/rust/Cargo.toml",
                include_str!("../../../app/rust/Cargo.toml"),
            ),
            ("fuzz/Cargo.toml", include_str!("../../../fuzz/Cargo.toml")),
        ] {
            assert!(
                toml_src.contains("disallowed_macros = \"allow\"")
                    && toml_src.contains("disallowed_methods = \"allow\""),
                "{manifest} must allow disallowed_macros and disallowed_methods outside totaldrop-cli"
            );
        }
    }
}
