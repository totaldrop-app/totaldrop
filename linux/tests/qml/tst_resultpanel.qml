// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtTest
import app.totaldrop

// ResultPanel: confirmed single web link flow (#54) — plain text hides Open
// link, a single http(s) URL shows the full normalized address left to right
// before opening a browser, cancelling does nothing, and handler failure
// surfaces openLinkFailed until the panel resets.
TestCase {
    id: tc
    name: "ResultPanel"
    visible: true
    when: windowShown
    width: 480
    height: 600

    function init() {
        failOnWarning(/.*/)
    }

    Component {
        id: panelComponent
        ResultPanel {
            visible: false
            width: 400
            title: Strings.s.receiveTextDone
        }
    }

    function makePanel(props) {
        const p = createTemporaryObject(panelComponent, tc, props || {})
        verify(p !== null)
        p.visible = true
        return p
    }

    function makeLauncher(ok) {
        const calls = []
        return {
            calls: calls,
            openWebUrl: function(url) {
                calls.push(url)
                return ok
            }
        }
    }

    function test_plain_text_hides_open_link_button() {
        const p = makePanel({ shownText: "hello note", webUrl: "" })
        const copyBtn = findChild(p, "copyButton")
        const openBtn = findChild(p, "openLinkButton")
        verify(copyBtn !== null)
        verify(openBtn !== null)
        compare(copyBtn.visible, true)
        compare(openBtn.visible, false)
        tryCompare(copyBtn, "activeFocus", true)
    }

    function test_single_link_shows_ltr_punycode_dialog_and_cancel_does_not_open() {
        const url = "https://xn--80ak6aa92e.com/login"
        const launcher = makeLauncher(true)
        const p = makePanel({
            shownText: url,
            webUrl: url,
            launcher: launcher
        })
        const copyBtn = findChild(p, "copyButton")
        const openBtn = findChild(p, "openLinkButton")
        const dialog = findChild(p, "openLinkDialog")
        verify(copyBtn !== null)
        verify(openBtn !== null)
        verify(dialog !== null)
        compare(openBtn.visible, true)
        compare(dialog.visible, false)
        // Initial focus stays on Copy text even when Open link is shown.
        tryCompare(copyBtn, "activeFocus", true)
        compare(openBtn.activeFocus, false)

        openBtn.click()
        tryCompare(dialog, "opened", true)
        compare(launcher.calls.length, 0)

        const addr = findChild(dialog, "linkAddress")
        const cancelBtn = findChild(dialog, "cancelLinkButton")
        const confirmBtn = findChild(dialog, "confirmLinkButton")
        verify(addr !== null)
        verify(cancelBtn !== null)
        verify(confirmBtn !== null)
        compare(addr.text, url)
        compare(addr.readOnly, true)
        compare(addr.LayoutMirroring.enabled, false)
        compare(addr.horizontalAlignment, TextEdit.AlignLeft)
        // Safe default inside the confirmation dialog: focus lands on Cancel.
        tryCompare(cancelBtn, "activeFocus", true)
        compare(confirmBtn.activeFocus, false)

        cancelBtn.click()
        tryCompare(dialog, "visible", false)
        tryCompare(openBtn, "activeFocus", true)
        compare(launcher.calls.length, 0)
        compare(p.linkOpenFailed, false)
    }

    function test_confirm_calls_launcher_once_with_normalized_url() {
        const url = "https://totaldrop.app/docs"
        const launcher = makeLauncher(true)
        const p = makePanel({
            shownText: url,
            webUrl: url,
            launcher: launcher
        })
        const copyBtn = findChild(p, "copyButton")
        const openBtn = findChild(p, "openLinkButton")
        const dialog = findChild(p, "openLinkDialog")
        const failedLabel = findChild(p, "openLinkFailedLabel")
        verify(copyBtn !== null)
        verify(openBtn !== null)
        verify(dialog !== null)
        verify(failedLabel !== null)
        tryCompare(copyBtn, "activeFocus", true)

        openBtn.click()
        tryCompare(dialog, "opened", true)
        const confirmBtn = findChild(dialog, "confirmLinkButton")
        verify(confirmBtn !== null)
        confirmBtn.click()
        tryCompare(dialog, "visible", false)
        tryCompare(openBtn, "activeFocus", true)

        compare(launcher.calls, [url])
        compare(p.linkOpenFailed, false)
        compare(failedLabel.visible, false)
    }

    function test_launcher_failure_shows_error_and_clears_on_url_reset() {
        const url = "https://totaldrop.app/docs"
        const launcher = makeLauncher(false)
        const p = makePanel({
            shownText: url,
            webUrl: url,
            launcher: launcher
        })
        const copyBtn = findChild(p, "copyButton")
        const openBtn = findChild(p, "openLinkButton")
        const dialog = findChild(p, "openLinkDialog")
        const failedLabel = findChild(p, "openLinkFailedLabel")
        verify(copyBtn !== null)
        verify(openBtn !== null)
        verify(dialog !== null)
        verify(failedLabel !== null)
        tryCompare(copyBtn, "activeFocus", true)

        openBtn.click()
        tryCompare(dialog, "opened", true)
        const confirmBtn = findChild(dialog, "confirmLinkButton")
        verify(confirmBtn !== null)
        confirmBtn.click()
        tryCompare(dialog, "visible", false)
        tryCompare(openBtn, "activeFocus", true)

        compare(launcher.calls, [url])
        compare(p.linkOpenFailed, true)
        compare(failedLabel.visible, true)
        compare(failedLabel.text, Strings.s.openLinkFailed)

        p.webUrl = "https://totaldrop.app/security"
        compare(p.linkOpenFailed, false)
        compare(failedLabel.visible, false)

        openBtn.click()
        tryCompare(dialog, "opened", true)
        p.webUrl = ""
        tryCompare(dialog, "visible", false)
        compare(openBtn.visible, false)
    }
}
