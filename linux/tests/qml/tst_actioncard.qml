// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtTest
import app.totaldrop

// The home screen's Send and Receive cards answer to the keys that press
// the app's other buttons (#182).
TestCase {
    id: tc
    name: "ActionCard"
    when: windowShown
    width: 400
    height: 200

    Component {
        id: cardComponent
        ActionCard {
            title: "Send"
            body: "Pick files and get a code."
            width: 300
        }
    }

    SignalSpy {
        id: clicks
        signalName: "clicked"
    }

    function focusedCard() {
        const card = createTemporaryObject(cardComponent, tc)
        verify(card)
        clicks.clear()
        clicks.target = card
        card.forceActiveFocus()
        verify(card.activeFocus)
        return card
    }

    function test_opens_data() {
        return [
            { tag: "Return", key: Qt.Key_Return },
            { tag: "Enter", key: Qt.Key_Enter },
            { tag: "Space", key: Qt.Key_Space },
        ]
    }

    // One press and release opens the card once, not again on release.
    function test_opens(data) {
        focusedCard()
        keyClick(data.key)
        compare(clicks.count, 1)
    }

    // The press opens the card; the release adds nothing.
    function test_opens_on_press_data() {
        return test_opens_data().slice(0, 2)
    }
    function test_opens_on_press(data) {
        focusedCard()
        keyPress(data.key)
        compare(clicks.count, 1)
        keyRelease(data.key)
        compare(clicks.count, 1)
    }

    // Held down, the key repeats: the repeats are ignored. QtTest cannot
    // send a repeated key, so the handler gets the event's fields.
    function test_held_enter_opens_once() {
        const card = focusedCard()
        card.enterPressed({ isAutoRepeat: false })
        card.enterPressed({ isAutoRepeat: true })
        card.enterPressed({ isAutoRepeat: true })
        compare(clicks.count, 1)
    }

    function test_disabled_does_not_open() {
        const card = focusedCard()
        card.enabled = false
        card.enterPressed({ isAutoRepeat: false })
        keyClick(Qt.Key_Return)
        compare(clicks.count, 0)
    }
}
