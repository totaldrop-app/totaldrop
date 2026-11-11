// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls
import QtQuick.Controls.impl
import QtQuick.Layouts
import app.totaldrop

// The end of a transfer: success, a decline or an error. Errors say what
// happened and what to do; the technical detail sits under Details as
// selectable text. Enter presses the focused button, as Space does: Qt
// Quick buttons take only Space, and GTK desktops press with Enter too.
// Held down, Enter repeats; only the first press counts.
// Escape is Done, through the page's back.
ColumnLayout {
    id: r

    property bool success: false
    property string title
    property string body
    property string detail
    // Why, as the core names it (TimedOut, DiskFull, …); its text is
    // Strings "cause" + it, in the person's language (#212).
    property string cause
    // The folder the files went to, as a file:// URL encoded in Rust:
    // "file://" + a path would cut a name at "#" or "?" and decode "%"
    // (#427). Empty when there is no folder to open.
    property string folderUrl
    property bool canRetry: false
    // A text received (#326), with hidden characters named. Copy text
    // copies the text exactly as it came, not this.
    property string shownText
    // shownText is only the text's beginning.
    property bool shortened: false
    // The text holds characters shown by their code.
    property bool hidden: false

    signal finished()
    signal retry()
    // Copy text: the page puts the received text on the clipboard.
    signal copyText()

    spacing: Theme.s16

    // Shown, the panel hands Done the keyboard: focus would otherwise stay
    // on the hidden button that started the transfer. Details closes again
    // for the next result.
    // A received text hands it to Copy instead: one Enter too many on
    // Done would throw the text away.
    onVisibleChanged: if (visible) {
        more.checked = false
        Qt.callLater(() => (r.shownText !== "" ? copyButton : doneButton).forceActiveFocus())
    }

    Item {
        Layout.alignment: Qt.AlignHCenter
        Layout.preferredWidth: 140
        Layout.preferredHeight: 140
        Layout.bottomMargin: Theme.s16
        // The done badge: the resting mark with a check inside its circle.
        Mark { size: 140; done: true; visible: r.success }
        IconImage {
            visible: !r.success
            anchors.centerIn: parent
            name: Theme.icon("dialog-error", "dialog-error")
            color: Theme.error
            sourceSize: Qt.size(56, 56)
        }
    }

    Label {
        text: r.title
        color: Theme.ink
        font.pointSize: Qt.application.font.pointSize * 1.5
        font.weight: Font.DemiBold
        wrapMode: Text.Wrap
        horizontalAlignment: Text.AlignHCenter
        Layout.fillWidth: true
    }
    Label {
        visible: text !== ""
        text: r.body
        color: Theme.ink
        wrapMode: Text.Wrap
        horizontalAlignment: Text.AlignHCenter
        Layout.fillWidth: true
    }
    // The text, as data: plain text, never rich text or a link that
    // opens by itself.
    ScrollView {
        visible: r.shownText !== ""
        Layout.fillWidth: true
        Layout.fillHeight: true
        Layout.minimumHeight: 96
        TextArea {
            text: r.shownText
            textFormat: TextEdit.PlainText
            readOnly: true
            selectByMouse: true
            wrapMode: TextEdit.Wrap
            Accessible.name: Strings.s.receiveOfferText
        }
    }
    Label {
        visible: r.shortened
        text: Strings.s.receiveTextShortened
        color: Theme.muted
        wrapMode: Text.Wrap
        Layout.fillWidth: true
    }
    Label {
        visible: r.shownText !== "" && r.hidden
        text: Strings.s.receiveTextHidden
        color: Theme.muted
        wrapMode: Text.Wrap
        Layout.fillWidth: true
    }
    SecondaryButton {
        id: copyButton
        visible: r.shownText !== ""
        text: Strings.s.copyText
        icon.name: Theme.icon("edit-copy")
        // Straight to the clipboard: a text box would change the text.
        onClicked: { r.copyText(); textCopied.start() }
        Timer { id: textCopied; interval: 2000 }
        ToolTip.visible: textCopied.running
        ToolTip.text: Strings.s.textCopied
        Layout.alignment: Qt.AlignHCenter
    }
    SecondaryButton {
        visible: r.folderUrl !== ""
        text: Strings.s.openFolder
        icon.name: Theme.icon("folder-open", "folder-open")
        onClicked: Qt.openUrlExternally(r.folderUrl)
        Layout.alignment: Qt.AlignHCenter
    }

    // Details and its text sit directly in the column: nested in a layout
    // of their own, the button moved when the text opened.
    SecondaryButton {
        id: more
        visible: r.detail !== ""
        flat: true
        checkable: true
        text: Strings.s.moreInfo
        icon.name: Theme.icon(checked ? "go-up" : "go-down")
        Layout.alignment: Qt.AlignHCenter
    }
    Label {
        visible: more.visible && more.checked && text !== ""
        text: r.cause !== "" ? (Strings.s["cause" + r.cause] || "") : ""
        color: Theme.ink
        wrapMode: Text.Wrap
        Layout.fillWidth: true
    }
    Label {
        visible: more.visible && more.checked
        text: Strings.s.technicalDetail
        color: Theme.muted
        wrapMode: Text.Wrap
        Layout.fillWidth: true
        // The label belongs to the text below it: closer than the gap
        // between sections.
        Layout.bottomMargin: Theme.s8 - r.spacing
    }
    TextArea {
        visible: more.visible && more.checked
        // May hold the other side's cancel reason: hidden direction
        // characters are shown, not obeyed.
        text: Strings.name(r.detail)
        readOnly: true
        selectByMouse: true
        wrapMode: Text.Wrap
        font.family: "monospace"
        Layout.fillWidth: true
    }

    Item { visible: r.shownText === ""; Layout.fillHeight: true }

    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.s8
        Item { Layout.fillWidth: true }
        SecondaryButton {
            visible: r.canRetry
            text: Strings.s.tryAgain
            onClicked: r.retry()
        }
        PrimaryButton {
            id: doneButton
            text: Strings.s.done
            onClicked: r.finished()
        }
    }
}
