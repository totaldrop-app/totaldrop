//! `--help` and `--version`, answered on the terminal before Qt starts.
//!
//! Everything else on the command line goes to the window: files to send,
//! a TotalDrop link, `--screen=…`, and Qt's own options. Unknown options
//! are left to Qt, which ignores what it does not know.

use totaldrop_l10n::Strings;

/// A request the terminal answers without opening the window.
#[derive(Debug, PartialEq, Eq)]
pub enum Flag {
    Help,
    Version,
}

/// The first `--help` or `--version` in `args` (without the program
/// name), up to a `--` that ends the options.
pub fn early(args: impl IntoIterator<Item = String>) -> Option<Flag> {
    args.into_iter()
        .take_while(|a| a != "--")
        .find_map(|a| match a.as_str() {
            "--help" | "-h" => Some(Flag::Help),
            "--version" | "-V" => Some(Flag::Version),
            _ => None,
        })
}

/// The usage text in the language of `s` (the command line's strings).
pub fn help(s: &Strings) -> String {
    let t = |key: &str| s.get(key, &[]);
    let rows = [
        ("[FILES]... | [LINK]", t("cliDesktopArgs")),
        ("--screen=<send|receive|settings>", t("cliDesktopScreen")),
        ("-h, --help", t("cliHelpHelp")),
        ("-V, --version", t("cliHelpVersion")),
    ];
    let width = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let row = |(k, v): &(&str, String)| format!("  {k:width$}  {v}\n");
    format!(
        "{}\n\n{}: totaldrop-desktop [OPTIONS] [FILES]... | [LINK]\n\n{}:\n{}\n{}:\n{}{}{}",
        t("cliDesktopAbout"),
        t("cliHelpUsage"),
        t("cliHelpArguments"),
        row(&rows[0]),
        t("cliHelpOptions"),
        row(&rows[1]),
        row(&rows[2]),
        row(&rows[3]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use totaldrop_l10n::Domain;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| (*s).to_owned()).collect()
    }

    /// `totaldrop-desktop --help` printed nothing and opened the window
    /// (#171).
    #[test]
    fn help_and_version_are_answered() {
        assert_eq!(early(args(&["--help"])), Some(Flag::Help));
        assert_eq!(early(args(&["-h"])), Some(Flag::Help));
        assert_eq!(
            early(args(&["/tmp/a.jpg", "--version"])),
            Some(Flag::Version)
        );
        assert_eq!(early(args(&["-V"])), Some(Flag::Version));
    }

    #[test]
    fn everything_else_opens_the_window() {
        assert_eq!(early(args(&[])), None);
        assert_eq!(early(args(&["--screen=send", "/tmp/a.jpg"])), None);
        assert_eq!(early(args(&["--foo", "-platform", "wayland"])), None);
        // After `--`, a file may be called `--help`.
        assert_eq!(early(args(&["--", "--help"])), None);
    }

    #[test]
    fn help_is_in_the_persons_language() {
        let tr = Strings::new(
            Domain::Cli,
            &[totaldrop_l10n::locale::parse("tr").unwrap()],
            None,
        );
        let text = help(&tr);
        for key in [
            "cliDesktopAbout",
            "cliDesktopArgs",
            "cliDesktopScreen",
            "cliHelpUsage",
            "cliHelpOptions",
            "cliHelpHelp",
        ] {
            assert!(text.contains(&tr.get(key, &[])), "{key} missing:\n{text}");
        }
        assert!(!text.contains("Usage"), "{text}");
    }
}
