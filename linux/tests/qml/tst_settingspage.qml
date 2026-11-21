// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtTest
import app.totaldrop

Item {
    id: stage
    width: 480
    height: 960
    LayoutMirroring.enabled: Strings.rtl
    LayoutMirroring.childrenInherit: true

    Transfer {
        id: sharedTransfer
    }

    Component {
        id: launcherBox
        Launcher {}
    }

    Component {
        id: pageBox
        SettingsPage {
            width: 480
            height: 960
        }
    }

    TestCase {
        name: "SettingsPageGlobalShortcut"
        when: windowShown

        function init() {
            Strings.rtl = false
            sharedTransfer.resetServer()
            sharedTransfer.savePrivacyMode(false)
            failOnWarning(/.*/)
        }

        function cleanup() {
            Strings.rtl = false
        }

        function make(launcherProps) {
            const lnch = createTemporaryObject(launcherBox, stage, launcherProps || {})
            verify(lnch !== null)
            const page = createTemporaryObject(pageBox, stage, {
                transfer: sharedTransfer,
                launcher: lnch
            })
            verify(page !== null)
            waitForRendering(page)
            return { page: page, launcher: lnch, transfer: sharedTransfer }
        }

        function test_portalSupportedUnsetOffersSetShortcutAndConfigures() {
            const ctx = make({
                shortcutSupported: true,
                shortcutTrigger: ""
            })
            const row = findChild(ctx.page, "shortcutStatusRow")
            const trig = findChild(ctx.page, "shortcutTriggerLabel")
            const btn = findChild(ctx.page, "configureShortcutButton")
            const manual = findChild(ctx.page, "shortcutManualLabel")
            const cmdBox = findChild(ctx.page, "shortcutCommandBox")

            verify(row !== null && row.visible)
            verify(trig !== null && trig.visible)
            compare(trig.text, "Not set")
            verify(btn !== null && btn.visible)
            compare(btn.text, "Set shortcut")
            verify(manual !== null && !manual.visible)
            verify(cmdBox !== null && !cmdBox.visible)

            mouseClick(btn)
            compare(ctx.launcher.configureCalls, 1)
        }

        function test_portalSupportedBoundShowsLtrTriggerAndUpdatesLive() {
            const ctx = make({
                shortcutSupported: true,
                shortcutTrigger: "Meta+Shift+V"
            })
            const trig = findChild(ctx.page, "shortcutTriggerLabel")
            const btn = findChild(ctx.page, "configureShortcutButton")

            verify(trig !== null && trig.visible)
            compare(trig.text, "Meta+Shift+V")
            compare(trig.LayoutMirroring.enabled, false)
            compare(trig.horizontalAlignment, Text.AlignLeft)
            verify(btn !== null && btn.visible)
            compare(btn.text, "Change shortcut")

            mouseClick(btn)
            compare(ctx.launcher.configureCalls, 1)

            // ShortcutsChanged from the desktop updates Settings live.
            ctx.launcher.shortcutTrigger = "Ctrl+Alt+V"
            compare(trig.text, "Ctrl+Alt+V")
            compare(btn.text, "Change shortcut")

            ctx.launcher.shortcutTrigger = ""
            compare(trig.text, "Not set")
            compare(btn.text, "Set shortcut")
        }

        function test_portalUnavailableShowsManualCommandFallback() {
            const ctx = make({
                shortcutSupported: false,
                shortcutCommand: "flatpak run app.totaldrop.TotalDrop --screen=clipboard"
            })
            const row = findChild(ctx.page, "shortcutStatusRow")
            const btn = findChild(ctx.page, "configureShortcutButton")
            const manual = findChild(ctx.page, "shortcutManualLabel")
            const cmdBox = findChild(ctx.page, "shortcutCommandBox")

            verify(row !== null && !row.visible)
            verify(btn !== null && !btn.visible)
            verify(manual !== null && manual.visible)
            verify(cmdBox !== null && cmdBox.visible)
            compare(cmdBox.text, "flatpak run app.totaldrop.TotalDrop --screen=clipboard")
            compare(cmdBox.readOnly, true)
            compare(cmdBox.LayoutMirroring.enabled, false)
            compare(cmdBox.horizontalAlignment, TextEdit.AlignLeft)
        }

        function test_rtlKeepsTriggerAndCommandLtrWhileUnsetMirrors() {
            Strings.rtl = true
            const ctx = make({
                shortcutSupported: true,
                shortcutTrigger: "",
                shortcutCommand: "totaldrop-desktop --screen=clipboard"
            })
            const trig = findChild(ctx.page, "shortcutTriggerLabel")
            const cmdBox = findChild(ctx.page, "shortcutCommandBox")

            verify(trig !== null && trig.visible)
            compare(trig.text, "Not set")
            compare(trig.LayoutMirroring.enabled, true)
            compare(trig.horizontalAlignment, Text.AlignRight)

            ctx.launcher.shortcutTrigger = "Meta+Shift+V"
            compare(trig.text, "Meta+Shift+V")
            compare(trig.LayoutMirroring.enabled, false)
            compare(trig.horizontalAlignment, Text.AlignLeft)

            ctx.launcher.shortcutSupported = false
            verify(cmdBox !== null && cmdBox.visible)
            compare(cmdBox.LayoutMirroring.enabled, false)
            compare(cmdBox.horizontalAlignment, TextEdit.AlignLeft)

            Strings.rtl = false
        }

        function test_hidesShortcutRowsUntilPortalReady() {
            const ctx = make({
                shortcutReady: false,
                shortcutSupported: false,
                shortcutCommand: "totaldrop-desktop --screen=clipboard"
            })
            const row = findChild(ctx.page, "shortcutStatusRow")
            const btn = findChild(ctx.page, "configureShortcutButton")
            const manual = findChild(ctx.page, "shortcutManualLabel")
            const cmdBox = findChild(ctx.page, "shortcutCommandBox")

            verify(row !== null && !row.visible)
            verify(btn !== null && !btn.visible)
            verify(manual !== null && !manual.visible)
            verify(cmdBox !== null && !cmdBox.visible)

            // First portal state callback sets shortcutReady = true and reveals
            // the matching row without flashing the manual command first.
            ctx.launcher.shortcutSupported = true
            ctx.launcher.shortcutReady = true
            verify(row.visible)
            verify(btn.visible)
            verify(!manual.visible)
            verify(!cmdBox.visible)

            // When launcher is absent (and no stub command), shortcutCommand is "".
            const bare = createTemporaryObject(pageBox, stage, {
                transfer: sharedTransfer,
                launcher: null
            })
            verify(bare !== null)
            compare(bare.shortcutReady, false)
            compare(bare.shortcutCommand, "")
        }
    }
}
