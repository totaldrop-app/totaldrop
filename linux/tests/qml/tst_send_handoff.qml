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

    function findButtonByText(item, label, seen) {
        if (!item) return null
        seen = seen || []
        if (seen.indexOf(item) >= 0) return null
        seen.push(item)
        if (item.visible && typeof item.text === "string" && item.text === label
                && typeof item.click === "function")
            return item
        if (item.contentItem) {
            const hit = findButtonByText(item.contentItem, label, seen)
            if (hit) return hit
        }
        const kids = item.children || []
        for (let i = 0; i < kids.length; i++) {
            const hit = findButtonByText(kids[i], label, seen)
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

    function test_unreadable_only_handoff_in_text_mode_returns_after_send() {
        const s = openSend()
        let closed = false
        s.page.close.connect(() => { closed = true })
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
        compare(s.page.pendingFlatpakUnreadable, true)

        // Continuing to type clears the transient textNotice, but keeps the
        // pending flag so the notice returns in file mode next to Add files.
        box.text = "draft message edited"
        settle()
        compare(s.page.textNotice, "")
        compare(s.page.pendingFlatpakUnreadable, true)

        // Finishing the text transfer opens files mode with the notice restored.
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
        compare(s.page.paths(), [])
        compare(s.page.fileNotice, Strings.s.sendFlatpakUnreadable)
        compare(s.page.pendingFlatpakUnreadable, false)
    }

    function test_backspacing_to_empty_keeps_text_mode_and_clear_text_drains() {
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

        // Backspacing the draft to empty must not yank the person out of text
        // mode while they retype.
        box.text = ""
        settle()
        settle()
        compare(s.page.mode, "text")
        compare(s.page.paths(), [])
        compare(s.page.pendingFiles, ["/tmp/readable.pdf"])
        compare(s.page.pendingFlatpakUnreadable, true)

        // Explicitly clearing text drains the queued file and carries the notice.
        s.page.clearText()
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

    function test_empty_text_add_files_button_drains_queued_files_without_picker() {
        const s = openSend()
        s.page.mode = "text"
        const box = findTextBox(s.page)
        verify(box)
        box.text = "draft"

        s.page.add(["/tmp/queued.pdf"])
        s.page.showFlatpakUnreadable()
        settle()

        box.text = ""
        settle()
        compare(s.page.mode, "text")
        compare(s.page.hasInput, true)

        const btn = findButtonByText(s.page, Strings.s.sendAddFiles)
        verify(btn)
        btn.click()
        settle()
        settle()

        compare(s.page.mode, "files")
        compare(s.page.paths(), ["/tmp/queued.pdf"])
        compare(s.page.fileNotice, Strings.s.sendFlatpakUnreadable)
        compare(s.page.pendingFiles.length, 0)
        compare(s.page.pendingFlatpakUnreadable, false)
        compare(s.xfer.pickFilesCalls, 0)
    }

    function test_empty_text_add_files_button_drains_unreadable_only_and_opens_picker() {
        const s = openSend()
        s.page.mode = "text"
        const box = findTextBox(s.page)
        verify(box)
        box.text = "draft"

        s.page.showFlatpakUnreadable()
        settle()

        box.text = ""
        settle()
        compare(s.page.mode, "text")

        const btn = findButtonByText(s.page, Strings.s.sendAddFiles)
        verify(btn)
        btn.click()
        settle()
        settle()

        compare(s.page.mode, "files")
        compare(s.page.paths(), [])
        compare(s.page.fileNotice, Strings.s.sendFlatpakUnreadable)
        compare(s.page.pendingFiles.length, 0)
        compare(s.page.pendingFlatpakUnreadable, false)
        compare(s.xfer.pickFilesCalls, 1)
    }

    function test_add_after_backspace_merges_queued_files_and_notice() {
        const s = openSend()
        s.page.mode = "text"
        const box = findTextBox(s.page)
        verify(box)
        box.text = "draft"

        s.page.add(["/tmp/first.pdf"])
        s.page.showFlatpakUnreadable()
        settle()

        box.text = ""
        settle()
        compare(s.page.mode, "text")
        compare(s.page.hasInput, true)

        s.page.add(["/tmp/second.pdf"])
        settle()

        compare(s.page.mode, "files")
        compare(s.page.paths(), ["/tmp/first.pdf", "/tmp/second.pdf"])
        compare(s.page.fileNotice, Strings.s.sendFlatpakUnreadable)
        compare(s.page.pendingFiles.length, 0)
        compare(s.page.pendingFlatpakUnreadable, false)
    }

    function test_pending_files_capped_at_max_pending_files_and_deduplicated() {
        const s = openSend()
        s.page.mode = "text"
        const box = findTextBox(s.page)
        verify(box)
        box.text = "draft"

        const limit = s.page.maxPendingFiles
        const many = ["/tmp/dup.txt", "/tmp/dup.txt"]
        for (let i = 0; i < limit + 5; i++)
            many.push("/tmp/file-" + i + ".txt")

        s.page.add(many)
        settle()

        compare(s.page.mode, "text")
        compare(s.page.pendingFiles.length, limit)
        compare(s.page.pendingFiles[0], "/tmp/dup.txt")
        compare(s.page.pendingFiles[1], "/tmp/file-0.txt")

        // Additional paths once full do not exceed maxPendingFiles.
        s.page.add(["/tmp/overflow.txt"])
        settle()
        compare(s.page.pendingFiles.length, limit)
    }
}
