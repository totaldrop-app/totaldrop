// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls

// A stage headline that a screen reader reads out when its step appears
// (#273). When a screen moves to a new stage, the focused control goes
// away; announcing the new headline keeps Orca from falling silent. Only
// for text that changes with the stage, never for progress counters.
Label {
    id: heading

    Accessible.role: Accessible.Heading
    Accessible.name: text

    // Emitted whenever the headline is announced, so QML tests can check
    // announcements under the offscreen platform too.
    signal announced(string message)

    property bool queued: false
    property string spoken: ""

    function flush() {
        queued = false
        if (!visible || text === "" || text === spoken) return
        spoken = text
        if (typeof heading.Accessible.announce === "function")
            heading.Accessible.announce(text, Accessible.Polite)
        announced(text)
    }

    function schedule() {
        if (queued || !visible || text === "") return
        queued = true
        Qt.callLater(flush)
    }

    onVisibleChanged: {
        if (visible) schedule()
        else spoken = ""
    }
    onTextChanged: if (visible) schedule()
    Component.onCompleted: if (visible) schedule()
}
