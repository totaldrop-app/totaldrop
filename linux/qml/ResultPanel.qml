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
    // Normalized single http(s) URL when the received text is one web
    // link (#54); empty otherwise. Validated in Rust, never in QML.
    property string webUrl: ""
    property var launcher: null
    property bool linkOpenFailed: false

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
    onVisibleChanged: {
        if (visible) {
            more.checked = false
            r.linkOpenFailed = false
            Qt.callLater(() => {
                if (!r || !r.visible) return
                const target = r.shownText !== "" ? copyButton : doneButton
                if (target) target.forceActiveFocus()
            })
        } else {
            openLinkDialog.close()
            r.linkOpenFailed = false
        }
    }
    onWebUrlChanged: {
        r.linkOpenFailed = false
        if (r.webUrl === "") openLinkDialog.close()
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
    Flow {
        visible: r.shownText !== ""
        spacing: Theme.s8
        readonly property real singleRowWidth: copyButton.implicitWidth
            + (openLinkButton.visible ? spacing + openLinkButton.implicitWidth : 0)
        Layout.preferredWidth: r.width > 0 ? Math.min(r.width, singleRowWidth) : singleRowWidth
        Layout.alignment: Qt.AlignHCenter

        SecondaryButton {
            id: copyButton
            objectName: "copyButton"
            text: Strings.s.copyText
            icon.name: Theme.icon("edit-copy")
            // Straight to the clipboard: a text box would change the text.
            onClicked: { r.copyText(); textCopied.start() }
            Timer { id: textCopied; interval: 2000 }
            ToolTip.visible: textCopied.running
            ToolTip.text: Strings.s.textCopied
        }
        SecondaryButton {
            id: openLinkButton
            objectName: "openLinkButton"
            visible: r.webUrl !== ""
            text: Strings.s.openLink || ""
            icon.name: Theme.icon("insert-link")
            onClicked: {
                r.linkOpenFailed = false
                openLinkDialog.open()
            }
        }
    }
    Label {
        id: openLinkFailedLabel
        objectName: "openLinkFailedLabel"
        visible: r.linkOpenFailed && r.webUrl !== ""
        text: Strings.s.openLinkFailed || ""
        color: Theme.error
        wrapMode: Text.Wrap
        horizontalAlignment: Text.AlignHCenter
        Layout.fillWidth: true
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

    Dialog {
        id: openLinkDialog
        objectName: "openLinkDialog"
        parent: r.Overlay.overlay ? r.Overlay.overlay : r
        anchors.centerIn: parent
        modal: true
        focus: true
        Component.onCompleted: if (Theme.fusion) Overlay.modal = Theme.dim
        title: Strings.s.openLinkTitle || ""
        // A fixed text width: wrapped text in a self-sizing popup loops.
        readonly property int boxWidth: Math.max(200, Math.min(380,
            (r.Overlay.overlay ? r.Overlay.overlay.width : r.width + 2 * Theme.s24) - 4 * Theme.s24))
        contentWidth: boxWidth
        onOpened: Qt.callLater(() => {
            if (openLinkDialog && openLinkDialog.opened && cancelLinkButton)
                cancelLinkButton.forceActiveFocus()
        })
        onClosed: if (r.visible && openLinkButton.visible) {
            Qt.callLater(() => {
                if (r && r.visible && openLinkButton && openLinkButton.visible)
                    openLinkButton.forceActiveFocus()
            })
        }
        onAccepted: {
            const ok = r.launcher !== null && r.launcher.openWebUrl(r.webUrl)
            r.linkOpenFailed = !ok
        }

        Column {
            width: openLinkDialog.boxWidth
            spacing: Theme.s16

            Label {
                width: openLinkDialog.boxWidth
                text: Strings.s.openLinkBody || ""
                wrapMode: Text.Wrap
                color: Theme.ink
            }
            ScrollView {
                id: linkScroll
                width: openLinkDialog.boxWidth
                contentWidth: availableWidth
                height: Math.min(linkAddress.implicitHeight + linkScroll.topPadding + linkScroll.bottomPadding, 160)
                clip: true
                TextArea {
                    id: linkAddress
                    objectName: "linkAddress"
                    width: linkScroll.availableWidth
                    // Normalized ASCII/Punycode URL, always left to right so
                    // right-to-left locales never reorder domain labels (T17, #509, #54).
                    LayoutMirroring.enabled: false
                    LayoutMirroring.childrenInherit: true
                    horizontalAlignment: TextEdit.AlignLeft
                    text: r.webUrl
                    textFormat: TextEdit.PlainText
                    readOnly: true
                    selectByMouse: true
                    wrapMode: TextEdit.WrapAnywhere
                    font.family: "monospace"
                    Accessible.name: r.webUrl
                }
            }
        }

        footer: DialogButtonBox {
            SecondaryButton {
                id: cancelLinkButton
                objectName: "cancelLinkButton"
                text: Strings.s.cancel || ""
                DialogButtonBox.buttonRole: DialogButtonBox.RejectRole
            }
            PrimaryButton {
                id: confirmLinkButton
                objectName: "confirmLinkButton"
                text: Strings.s.openLinkConfirm || ""
                DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole
            }
        }
    }
}
