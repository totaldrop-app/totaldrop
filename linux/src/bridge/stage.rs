// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! What the transfer screen shows, as types (#343).
//!
//! QML reads the stage and the role as the `state` and `role` strings of
//! `Transfer`. Those strings come only from [`Stage::name`] and
//! [`Role::name`], and a test checks that every one QML compares with is
//! one of them, so a misspelt state fails CI instead of failing silently.
//!
//! The stages match Android's (`SendStage` and `ReceiveStage` in
//! `app/lib/ui`): idle is Android's picking or entering, offer its
//! offered.

/// One stage of a send or a receive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// Nothing runs: the page to choose files or type a code.
    #[default]
    Idle,
    /// Send: the files are scanned and a code is reserved.
    Preparing,
    /// Send: the code is shown, waiting for the receiver.
    Waiting,
    /// Send: the receiver came and decides; compare the words.
    Connected,
    /// Bytes are moving.
    Transferring,
    /// Receive: connecting with the code.
    Connecting,
    /// Receive: the offer waits for an answer.
    Offer,
    /// It ended well.
    Done,
    /// It ended with an error.
    Failed,
    /// The other side, or the person, said no.
    Declined,
}

impl Stage {
    /// Every stage, for the tests.
    #[cfg(test)]
    pub const ALL: [Self; 10] = [
        Self::Idle,
        Self::Preparing,
        Self::Waiting,
        Self::Connected,
        Self::Transferring,
        Self::Connecting,
        Self::Offer,
        Self::Done,
        Self::Failed,
        Self::Declined,
    ];

    /// The name QML compares with.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Preparing => "preparing",
            Self::Waiting => "waiting",
            Self::Connected => "connected",
            Self::Transferring => "transferring",
            Self::Connecting => "connecting",
            Self::Offer => "offer",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Declined => "declined",
        }
    }
}

/// Which side this window is on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Role {
    /// Nothing has run yet.
    #[default]
    None,
    Send,
    Receive,
}

impl Role {
    /// The name QML compares with.
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Send => "send",
            Self::Receive => "receive",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Role, Stage};

    /// The QML files that read `state` or `role`.
    const QML: [(&str, &str); 3] = [
        ("Main.qml", include_str!("../../qml/Main.qml")),
        ("SendPage.qml", include_str!("../../qml/SendPage.qml")),
        ("ReceivePage.qml", include_str!("../../qml/ReceivePage.qml")),
    ];

    /// The string literals on a line, between double quotes.
    fn literals(line: &str) -> impl Iterator<Item = &str> {
        line.split('"').skip(1).step_by(2)
    }

    /// Does `line` compare with the stage or the role? The pages read the
    /// stage as `phase`; the `switch (page.phase)` there has `case` lines.
    fn reads_stage(line: &str) -> bool {
        let line = line.trim_start();
        line.contains("phase")
            || line.contains(".state")
            || line.contains(".role ")
            || line.starts_with("case \"")
    }

    #[test]
    fn names_are_distinct() {
        for a in Stage::ALL {
            for b in Stage::ALL {
                assert_eq!(a == b, a.name() == b.name(), "{a:?} {b:?}");
            }
        }
    }

    /// A state or role QML compares with that the bridge never sets would
    /// make a screen never show, with no error anywhere (#343).
    #[test]
    fn qml_compares_only_with_known_names() {
        let known: Vec<&str> = Stage::ALL
            .iter()
            .map(|s| s.name())
            .chain([Role::Send.name(), Role::Receive.name()])
            .collect();
        let mut checked = 0;
        for (file, text) in QML {
            for (n, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap_or_default();
                if !reads_stage(code) {
                    continue;
                }
                for lit in literals(code) {
                    // Only the names: words in a sentence are not states.
                    if lit.is_empty() || !lit.bytes().all(|b| b.is_ascii_lowercase()) {
                        continue;
                    }
                    assert!(
                        known.contains(&lit),
                        "{file}:{}: \"{lit}\" is not a stage or role",
                        n + 1
                    );
                    checked += 1;
                }
            }
        }
        // The check must see the comparisons, or it proves nothing.
        assert!(checked > 40, "only {checked} names found");
    }

    /// Every stage a page lays out is one the bridge sets, and the reverse:
    /// a stage with no page would leave the window on the last one.
    #[test]
    fn every_stage_is_shown_by_a_page() {
        let all: String = QML.iter().map(|(_, t)| *t).collect();
        for s in Stage::ALL {
            assert!(
                all.contains(&format!("\"{}\"", s.name())),
                "QML never names {s:?}"
            );
        }
    }

    /// `state` and `role` are set in one place each, from the types.
    #[test]
    fn the_strings_are_set_only_from_the_types() {
        let bridge = include_str!("transfer.rs");
        assert_eq!(bridge.matches(".set_state(").count(), 1);
        assert_eq!(bridge.matches(".set_role(").count(), 1);
    }

    #[test]
    fn the_check_catches_a_misspelling() {
        let line = r#"if (xfer.state === "trasnferring") stopDialog.close()"#;
        assert!(reads_stage(line));
        let lits: Vec<&str> = literals(line).collect();
        assert_eq!(lits, ["trasnferring"]);
        assert!(!Stage::ALL.iter().any(|s| s.name() == lits[0]));
    }
}
