// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick

// Stand-in for the Rust Transfer element (linux/src/bridge/transfer.rs)
// so QML component tests can instantiate SettingsPage without a Rust build.
QtObject {
    property string server: "wss://rendezvous.totaldrop.app"
    property string defaultServer: "wss://rendezvous.totaldrop.app"
    property bool privacyMode: false

    function saveServer(url) {
        if (!url.startsWith("wss://")) return false
        server = url
        return true
    }

    function resetServer() {
        server = defaultServer
    }

    function savePrivacyMode(on) {
        privacyMode = on
        return true
    }
}
