// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtTest
import app.totaldrop

// When SendPage or ReceivePage moves to a new stage, the focused button goes
// away and a screen reader would otherwise say nothing (#273). Each stage's
// headline is an Accessible.Heading that announces itself once when shown;
// idle screens and progress ticks stay silent.
TestCase {
    id: tc
    name: "Announce"
    when: windowShown
    width: 480
    height: 720

    property int ticks: 0

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
    Component {
        id: receiveComponent
        ReceivePage {
            width: 480
            height: 720
        }
    }

    // Any QML warning (missing stub property, bad binding, destroyed target)
    // fails the test.
    function init() {
        failOnWarning(/.*/)
    }

    // Waits on a condition with TestCase's deadline until queued Qt.callLater
    // callbacks have drained, without a fixed sleep.
    function settle() {
        const next = ticks + 1
        Qt.callLater(() => { ticks = next })
        tryCompare(tc, "ticks", next)
    }

    function watchHeadings(root, spoken) {
        const kids = root.children
        for (let i = 0; i < kids.length; i++) {
            const child = kids[i]
            if (child.announced && typeof child.announced.connect === "function")
                child.announced.connect((msg) => spoken.push(msg))
            watchHeadings(child, spoken)
        }
    }

    function visibleHeadings(root, out) {
        out = out || []
        if (!root.visible) return out
        const kids = root.children
        for (let i = 0; i < kids.length; i++) {
            const child = kids[i]
            if (!child.visible) continue
            if (typeof child.text === "string"
                    && child.Accessible.role === Accessible.Heading)
                out.push(child.Accessible.name)
            visibleHeadings(child, out)
        }
        return out
    }

    function openPage(component) {
        const xfer = createTemporaryObject(transferComponent, tc)
        verify(xfer)
        const page = createTemporaryObject(component, tc, { transfer: xfer })
        verify(page)
        const spoken = []
        watchHeadings(page, spoken)
        // Drain initial 0-ms focus timer and any creation-time callLater.
        settle()
        settle()
        return { xfer: xfer, page: page, spoken: spoken }
    }

    function test_idle_screens_stay_quiet() {
        const s = openPage(sendComponent)
        compare(visibleHeadings(s.page), [])
        compare(s.spoken, [])
        s.page.mode = "text"
        settle()
        compare(visibleHeadings(s.page), [])
        compare(s.spoken, [])
        s.page.mode = "files"
        settle()
        compare(visibleHeadings(s.page), [])
        compare(s.spoken, [])

        const r = openPage(receiveComponent)
        compare(visibleHeadings(r.page), [])
        compare(r.spoken, [])
    }

    function test_send_announces_each_stage_and_skips_progress() {
        const ctx = openPage(sendComponent)
        const xfer = ctx.xfer

        xfer.role = "send"
        xfer.state = "preparing"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.sendPreparing])
        compare(ctx.spoken, [Strings.s.sendPreparing])

        xfer.code = "7-harbor-rocket"
        xfer.state = "waiting"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.sendWaiting])
        compare(ctx.spoken, [
            Strings.s.sendPreparing,
            Strings.s.sendWaiting,
        ])

        xfer.words = ["able", "baker", "coral", "dawn"]
        xfer.state = "connected"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.sendWaitingAccept])
        compare(ctx.spoken, [
            Strings.s.sendPreparing,
            Strings.s.sendWaiting,
            Strings.s.sendWaitingAccept,
        ])

        xfer.done = 0
        xfer.total = 4096
        xfer.state = "transferring"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.sendSending])
        compare(ctx.spoken.length, 4)
        compare(ctx.spoken[3], Strings.s.sendSending)

        // Byte counts and percentages must never trigger announcements.
        xfer.done = 1024
        settle()
        xfer.done = 2048
        settle()
        xfer.done = 4096
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.sendSending])
        compare(ctx.spoken.length, 4)

        xfer.filesOk = 1
        xfer.state = "done"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.sendDone])
        compare(ctx.spoken, [
            Strings.s.sendPreparing,
            Strings.s.sendWaiting,
            Strings.s.sendWaitingAccept,
            Strings.s.sendSending,
            Strings.s.sendDone,
        ])
    }

    function test_send_announces_errors_decline_and_retry() {
        const ctx = openPage(sendComponent)
        const xfer = ctx.xfer

        xfer.role = "send"
        xfer.state = "preparing"
        settle()
        xfer.errorKind = "Other"
        xfer.state = "failed"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.errorOther])

        // Failing again after a retry announces the same error headline again.
        xfer.errorKind = ""
        xfer.state = "preparing"
        settle()
        xfer.errorKind = "Other"
        xfer.state = "failed"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.errorOther])

        xfer.errorKind = ""
        xfer.state = "waiting"
        settle()
        xfer.errorKind = "Declined"
        xfer.state = "declined"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.errorDeclined])

        ctx.page.mode = "text"
        xfer.isText = true
        xfer.errorKind = ""
        xfer.state = "transferring"
        settle()
        xfer.filesFailed = 1
        xfer.state = "done"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.errorTextDamaged])

        compare(ctx.spoken, [
            Strings.s.sendPreparing,
            Strings.s.errorOther,
            Strings.s.sendPreparing,
            Strings.s.errorOther,
            Strings.s.sendWaiting,
            Strings.s.errorDeclined,
            Strings.s.sendSending,
            Strings.s.errorTextDamaged,
        ])
    }

    function test_receive_announces_each_stage_and_skips_progress() {
        const ctx = openPage(receiveComponent)
        const xfer = ctx.xfer

        xfer.role = "receive"
        xfer.state = "connecting"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.receiveConnecting])
        compare(ctx.spoken, [Strings.s.receiveConnecting])

        xfer.offerFiles = 2
        xfer.size = 2048
        xfer.offerNames = ["a.txt", "b.txt"]
        xfer.words = ["able", "baker", "coral", "dawn"]
        xfer.state = "offer"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.receiveOffer])
        compare(ctx.spoken, [
            Strings.s.receiveConnecting,
            Strings.s.receiveOffer,
        ])

        xfer.done = 0
        xfer.total = 2048
        xfer.state = "transferring"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.receiveReceiving])
        compare(ctx.spoken.length, 3)
        compare(ctx.spoken[2], Strings.s.receiveReceiving)

        // Progress ticks must never trigger announcements.
        xfer.done = 512
        settle()
        xfer.done = 2048
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.receiveReceiving])
        compare(ctx.spoken.length, 3)

        xfer.savedTo = "/tmp"
        xfer.savedToUrl = "file:///tmp"
        xfer.state = "done"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.receiveDone])
        compare(ctx.spoken, [
            Strings.s.receiveConnecting,
            Strings.s.receiveOffer,
            Strings.s.receiveReceiving,
            Strings.s.receiveDone,
        ])
    }

    function test_receive_announces_text_decline_and_failure() {
        const ctx = openPage(receiveComponent)
        const xfer = ctx.xfer

        xfer.role = "receive"
        xfer.state = "connecting"
        settle()

        xfer.isText = true
        xfer.size = 12
        xfer.words = ["able", "baker", "coral", "dawn"]
        xfer.state = "offer"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.receiveOfferText])

        xfer.state = "declined"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.receiveTextDeclined])

        xfer.isText = false
        xfer.state = "connecting"
        settle()
        xfer.state = "offer"
        settle()
        xfer.state = "declined"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.receiveDeclined])

        xfer.state = "connecting"
        settle()
        xfer.errorKind = "WrongCode"
        xfer.state = "failed"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.errorWrongCode])

        xfer.errorKind = ""
        xfer.isText = true
        xfer.state = "connecting"
        settle()
        xfer.state = "offer"
        settle()
        xfer.state = "transferring"
        settle()
        xfer.shownText = "hello"
        xfer.state = "done"
        settle()
        compare(visibleHeadings(ctx.page), [Strings.s.receiveTextDone])

        compare(ctx.spoken, [
            Strings.s.receiveConnecting,
            Strings.s.receiveOfferText,
            Strings.s.receiveTextDeclined,
            Strings.s.receiveConnecting,
            Strings.s.receiveOffer,
            Strings.s.receiveDeclined,
            Strings.s.receiveConnecting,
            Strings.s.errorWrongCode,
            Strings.s.receiveConnecting,
            Strings.s.receiveOfferText,
            Strings.s.receiveReceiving,
            Strings.s.receiveTextDone,
        ])
    }
}
