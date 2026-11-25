// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick

// Stand-in for the Rust Launcher QObject (linux/src/bridge/launcher.rs) so
// typed `property Launcher launcher` in ReceivePage resolves in QML tests.
QtObject {
    signal opened(var args)
    function startArgs() { return [] }
    function openWebUrl(url) { return false }
}
