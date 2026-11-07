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
//! The stages follow Android's (`SendStage` and `ReceiveStage` in
//! `app/lib/ui`): idle is Android's picking or entering, offer its
//! offered. One differs: a send the receiver declines ends as declined
//! here and as failed there.

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

impl Stage {
    /// A send or a receive is under way, from preparing to the last byte:
    /// the computer must not go to sleep (#340). A finished, failed or
    /// declined transfer, and the empty page, let it.
    ///
    /// The stages that wait for a person (the shown code, comparing the
    /// words, an offer to answer) hold it too: sleep there drops the code
    /// or the connection, and the transfer is lost anyway. They end on
    /// their own or when the person cancels: an unclaimed code after 15
    /// minutes, a mailbox after an hour (`server/totaldrop-rendezvous`).
    pub const fn keeps_awake(self) -> bool {
        // Exhaustive, so a new stage has to say whether it holds sleep.
        match self {
            Self::Preparing
            | Self::Waiting
            | Self::Connected
            | Self::Transferring
            | Self::Connecting
            | Self::Offer => true,
            Self::Idle | Self::Done | Self::Failed | Self::Declined => false,
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

    /// The QML files that read `state` or `role`; the next test makes
    /// sure no other one does.
    const QML: [(&str, &str); 3] = [
        ("Main.qml", include_str!("../../qml/Main.qml")),
        ("SendPage.qml", include_str!("../../qml/SendPage.qml")),
        ("ReceivePage.qml", include_str!("../../qml/ReceivePage.qml")),
    ];

    /// Sleep is held off from preparing to the last byte, and only then
    /// (#340).
    #[test]
    fn a_running_transfer_keeps_the_computer_awake() {
        let awake: Vec<_> = Stage::ALL.iter().filter(|s| s.keeps_awake()).collect();
        assert_eq!(
            awake,
            [
                &Stage::Preparing,
                &Stage::Waiting,
                &Stage::Connected,
                &Stage::Transferring,
                &Stage::Connecting,
                &Stage::Offer,
            ]
        );
    }

    #[test]
    fn no_other_qml_file_reads_the_stage() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/qml");
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if !name.ends_with(".qml") || QML.iter().any(|(f, _)| *f == name) {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for needle in ["transfer.state", "xfer.state", ".phase", "transfer.role"] {
                assert!(
                    !text.contains(needle),
                    "{name} reads {needle}: add it to QML"
                );
            }
        }
    }

    /// The string literals on a line, between double or single quotes.
    fn literals(line: &str) -> Vec<&str> {
        let mut out = Vec::new();
        let mut rest = line;
        while let Some(at) = rest.find(['"', '\'']) {
            let quote = &rest[at..=at];
            let after = &rest[at + 1..];
            let Some(end) = after.find(quote) else { break };
            out.push(&after[..end]);
            rest = &after[end + 1..];
        }
        out
    }

    /// Could `lit` be a stage or role name, rightly or misspelt? Names are
    /// one word of letters; a sentence, a key or a path is not one.
    fn looks_like_a_name(lit: &str) -> bool {
        !lit.is_empty() && lit.bytes().all(|b| b.is_ascii_alphabetic())
    }

    /// Does `line` compare with the stage or the role? The pages read the
    /// stage as `phase`; the `switch (page.phase)` there has `case` lines.
    fn reads_stage(line: &str) -> bool {
        let line = line.trim_start();
        line.contains("phase")
            || line.contains(".state")
            || line.contains(".role")
            || line.starts_with("case \"")
            || line.starts_with("case '")
    }

    /// [`Stage::ALL`] has every stage: a new one fails to compile here.
    #[test]
    fn all_has_every_stage() {
        for s in Stage::ALL {
            match s {
                Stage::Idle
                | Stage::Preparing
                | Stage::Waiting
                | Stage::Connected
                | Stage::Transferring
                | Stage::Connecting
                | Stage::Offer
                | Stage::Done
                | Stage::Failed
                | Stage::Declined => {}
            }
        }
        // Ten arms above, ten entries in ALL.
        assert_eq!(Stage::ALL.len(), 10);
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
                    if !looks_like_a_name(lit) {
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

    /// Every stage is named somewhere in the QML: a stage no page or
    /// handler names is likely one a page forgot.
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
        assert_eq!(bridge.matches(".state = ").count(), 1);
        assert_eq!(bridge.matches(".role = ").count(), 1);
        assert_eq!(bridge.matches("state_changed()").count(), 1);
        assert_eq!(bridge.matches("role_changed()").count(), 1);
    }

    #[test]
    fn the_check_catches_a_misspelling() {
        for line in [
            r#"if (xfer.state === "trasnferring") stopDialog.close()"#,
            "case 'ofer': return 2",
            r#"xfer.role==="sned""#,
            r#"page.phase === "Done""#,
        ] {
            assert!(reads_stage(line), "{line}");
            let lits = literals(line);
            assert_eq!(lits.len(), 1, "{line}");
            assert!(looks_like_a_name(lits[0]), "{line}");
            assert!(!Stage::ALL.iter().any(|s| s.name() == lits[0]), "{line}");
        }
        assert!(!looks_like_a_name("Send a text"));
        assert!(!looks_like_a_name(""));
    }
}
