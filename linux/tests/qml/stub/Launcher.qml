// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick

// Stand-in for the Rust Launcher QObject (linux/src/bridge/launcher.rs) so
// QML component tests can resolve ReceivePage and SettingsPage bindings
// without a Rust build.
QtObject {
    property bool shortcutSupported: false
    property string shortcutTrigger: ""
    property string shortcutCommand: "totaldrop-desktop --screen=clipboard"
    property int configureCalls: 0

    signal opened(var args)

    function startArgs() { return [] }
    function openWebUrl(url) { return false }
    function configureShortcut() { configureCalls += 1 }
}
