// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtTest
import app.totaldrop

// Verifies SendPage's Flatpak unreadable notice and queued file hand-off in
// both files mode and text mode (#357).
TestCase {
    id: tc
    name: "SendHandoff"
    visible: true
    when: windowShown
    width: 480
    height: 720

    property int ticks: 0
    property var pages: []
    property var transfers: []

    Component {
        id: transferComponent
        Transfer {}
    }
    Component {
        id: sendComponent
        SendPage {
            width: 480
            height: 720
        }
    }

    function init() {
        failOnWarning(/.*/)
    }

    function cleanup() {
        for (let i = 0; i < pages.length; i++)
            pages[i].destroy()
        pages = []
        settle()
    }

    function cleanupTestCase() {
        for (let i = 0; i < transfers.length; i++)
            transfers[i].destroy()
        transfers = []
    }

    function settle() {
        const next = ticks + 1
        Qt.callLater(() => { ticks = next })
        tryCompare(tc, "ticks", next)
    }

    function findTextBox(item, seen) {
        if (!item) return null
        seen = seen || []
        if (seen.indexOf(item) >= 0) return null
        seen.push(item)
        if (item.Accessible && item.Accessible.name === Strings.s.sendText
                && typeof item.clear === "function")
            return item
        if (item.contentItem) {
            const hit = findTextBox(item.contentItem, seen)
            if (hit) return hit
        }
        const kids = item.children || []
        for (let i = 0; i < kids.length; i++) {
            const hit = findTextBox(kids[i], seen)
            if (hit) return hit
        }
        return null
    }

    function findResultPanel(item, seen) {
        if (!item) return null
        seen = seen || []
        if (seen.indexOf(item) >= 0) return null
        seen.push(item)
        if (typeof item.finished === "function" && typeof item.canRetry === "boolean")
            return item
        if (item.contentItem) {
            const hit = findResultPanel(item.contentItem, seen)
            if (hit) return hit
        }
        const kids = item.children || []
        for (let i = 0; i < kids.length; i++) {
            const hit = findResultPanel(kids[i], seen)
            if (hit) return hit
        }
        return null
    }

    function openSend(extra) {
        const xfer = transferComponent.createObject(tc)
        verify(xfer)
        transfers.push(xfer)
        const props = Object.assign({ transfer: xfer }, extra || {})
        const page = sendComponent.createObject(tc, props)
        verify(page)
        pages.push(page)
        settle()
        settle()
        return { xfer: xfer, page: page }
    }

    function test_unreadable_notice_in_files_mode_and_empty_text_mode() {
        const s = openSend()
        s.page.showFlatpakUnreadable()
        settle()
        compare(s.page.mode, "files")
        compare(s.page.fileNotice, Strings.s.sendFlatpakUnreadable)
        compare(s.page.textNotice, "")

        // Empty text mode also switches to files mode with fileNotice.
        s.page.mode = "text"
        s.page.fileNotice = ""
        s.page.showFlatpakUnreadable()
        settle()
        compare(s.page.mode, "files")
        compare(s.page.fileNotice, Strings.s.sendFlatpakUnreadable)
    }

    function test_unreadable_notice_keeps_typed_text_in_text_mode() {
        const s = openSend()
        s.page.mode = "text"
        const box = findTextBox(s.page)
        verify(box)
        box.text = "draft message"
        s.page.showFlatpakUnreadable()
        settle()
        compare(s.page.mode, "text")
        compare(s.page.outgoing, "draft message")
        compare(s.page.textNotice, Strings.s.sendFlatpakUnreadable)
        compare(s.page.fileNotice, "")
        compare(s.page.pendingFiles.length, 0)
    }

    function test_mixed_handoff_during_typed_text_queues_and_drains_on_clear() {
        const s = openSend()
        s.page.mode = "text"
        const box = findTextBox(s.page)
        verify(box)
        box.text = "keep this draft"

        s.page.add(["/tmp/readable.pdf"])
        s.page.showFlatpakUnreadable()
        settle()

        // Typed text stays intact, notice is visible immediately, and the
        // readable file is queued rather than silently dropped.
        compare(s.page.mode, "text")
        compare(s.page.outgoing, "keep this draft")
        compare(s.page.textNotice, Strings.s.sendFlatpakUnreadable)
        compare(s.page.paths(), [])
        compare(s.page.pendingFiles, ["/tmp/readable.pdf"])
        compare(s.page.pendingFlatpakUnreadable, true)

        // Clearing the draft drains the queued file and carries the notice over.
        box.clear()
        settle()
        settle()
        compare(s.page.mode, "files")
        compare(s.page.paths(), ["/tmp/readable.pdf"])
        compare(s.page.fileNotice, Strings.s.sendFlatpakUnreadable)
        compare(s.page.pendingFiles.length, 0)
        compare(s.page.pendingFlatpakUnreadable, false)
    }

    function test_queued_files_open_after_text_transfer_finishes() {
        const s = openSend()
        let closed = false
        s.page.close.connect(() => { closed = true })
        s.page.mode = "text"
        const box = findTextBox(s.page)
        verify(box)
        box.text = "send me first"

        s.page.add(["/tmp/queued.txt"])
        settle()
        compare(s.page.pendingFiles, ["/tmp/queued.txt"])

        // Simulate sending the text and finishing on ResultPanel.
        s.page.sendIt()
        s.xfer.state = "done"
        settle()
        const panel = findResultPanel(s.page)
        verify(panel)
        panel.finished()
        settle()
        settle()

        compare(closed, false)
        compare(s.page.mode, "files")
        compare(s.page.paths(), ["/tmp/queued.txt"])
        compare(s.page.pendingFiles.length, 0)
    }
}
