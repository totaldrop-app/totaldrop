// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls
import QtQuick.Controls.impl
import QtQuick.Layouts
import app.totaldrop

Page {
    id: page

    required property Transfer transfer
    signal close()
    signal askCancel()

    readonly property string phase: transfer.role === "send" ? transfer.state : "idle"
    readonly property bool running: ["preparing", "waiting", "connected", "transferring"].indexOf(phase) >= 0
    property bool strong: false
    property var initialFiles: []
    // Opened by "Send clipboard": the text on the clipboard, filled in.
    property bool pasteOnOpen: false
    // What is sent: the files listed, or a text the receiver shows (#326).
    property string mode: "files"
    readonly property bool textMode: mode === "text"
    // Files listed or a text typed and not yet sent: another launch does
    // not replace them.
    readonly property bool hasInput: files.count > 0 || textBox.text !== "" || bigText !== ""
    // A text box holds text in a document that changes it on the way back
    // (a no-break space becomes a space, \r\n becomes \n), and lays out a
    // long line very slowly. A pasted text is therefore kept as it came:
    // `pasted` is sent while the box still shows exactly what the paste
    // put there. A paste longer than the box shows is `bigText`: the box
    // shows its beginning, read-only, and all of it is sent.
    property string pasted: ""
    property string pastedShown: ""
    property string bigText: ""
    readonly property string outgoing: bigText !== "" ? bigText
        : (pasted !== "" && textBox.text === pastedShown ? pasted : textBox.text)
    // The text's size as the limit counts it: UTF-8 bytes.
    readonly property real textBytes: transfer.textBytes(outgoing)
    readonly property bool textTooLong: textBytes > transfer.maxText()
    property string textNotice: ""
    // "Send clipboard" asked while the window had no keyboard focus: on
    // Wayland the clipboard is readable only with it.
    property bool pasteWhenActive: false
    Component.onCompleted: {
        if (initialFiles.length > 0) add(initialFiles)
        if (pasteOnOpen) Qt.callLater(page.pasteClipboard)
    }
    Connections {
        target: page.Window.window
        function onActiveChanged() {
            if (page.pasteWhenActive && page.Window.window.active) {
                page.pasteWhenActive = false
                // The click that brought the window up may have pasted
                // already (the Paste button); then there is nothing to say.
                Qt.callLater(() => { if (!page.hasInput) page.pasteClipboard() })
            }
        }
    }
    // With files listed, Enter creates the code.
    onPhaseChanged: if (phase === "idle") Qt.callLater(() => (page.textMode ? textBox : createButton).forceActiveFocus())
    Timer { running: true; interval: 0; onTriggered: (page.textMode ? textBox : files.count > 0 ? createButton : addButton).forceActiveFocus() }

    // "Send clipboard": the clipboard's text goes into the text box for
    // the person to look at; the code is made only when they ask. A text
    // already typed is kept.
    function pasteClipboard() {
        page.mode = "text"
        page.textNotice = ""
        const w = page.Window.window
        if (w && !w.active) { page.pasteWhenActive = true; return }
        if (textBox.text === "" && page.bigText === "") page.pasteText()
        else page.textNotice = Strings.s.sendClipboardKept
        if (textBox.text === "" && page.bigText === "") page.textNotice = Strings.s.sendClipboardEmpty
        textBox.forceActiveFocus()
    }
    // Paste, as the button, Ctrl+V and "Send clipboard" do. Into an empty
    // box the text is kept exactly; a long one replaces what is there.
    function pasteText() {
        const t = page.transfer.clipboardText()
        if (t === "") return
        const max = page.transfer.shownChars()
        if (t.length > max) {
            // Not cut inside a character.
            const end = (t.charCodeAt(max - 1) & 0xFC00) === 0xD800 ? max - 1 : max
            page.bigText = t
            textBox.text = t.slice(0, end)
            page.pasted = ""
        } else if (textBox.text === "") {
            textBox.text = t
            page.pasted = t
            page.pastedShown = textBox.text
        } else {
            textBox.insert(textBox.cursorPosition, t)
        }
    }
    function clearText() {
        page.bigText = ""
        page.pasted = ""
        textBox.clear()
    }
    function sendIt() {
        if (page.textMode) page.transfer.sendText(page.outgoing, page.strong)
        else page.transfer.send(page.paths(), page.strong)
    }

    background: null

    header: PageHeader {
        title: page.textMode ? Strings.s.sendText : Strings.s.sendTitle
        onBack: page.goBack()
    }

    // Back, from the header or Escape. A running transfer asks first; an
    // ended one is cleared, so the page opens fresh the next time.
    function goBack() {
        if (page.running) page.askCancel()
        else {
            if (page.phase !== "idle") page.transfer.reset()
            page.close()
        }
    }

    // Escape does what the back button does, but does not throw away a
    // list of files or a text not yet sent: that takes the back button.
    Shortcut {
        sequences: [StandardKey.Cancel]
        enabled: page.StackView.status === StackView.Active
                 && !(page.phase === "idle" && page.hasInput)
        onActivated: page.goBack()
    }

    ListModel { id: files }

    function add(paths) {
        // Files asked to be sent show the file list.
        if (paths.length > 0) page.mode = "files"
        for (const p of paths) {
            let known = false
            for (let i = 0; i < files.count; i++) if (files.get(i).path === p) known = true
            if (known) continue
            const [name, bytes, dir] = page.transfer.describe(p).split("\t")
            files.append({ path: p, name: name, bytes: Number(bytes), dir: dir === "1" })
            // A folder is -1 bytes until measured off the GUI thread.
            if (Number(bytes) < 0) page.transfer.measure(p)
        }
        recalc()
    }
    function paths() {
        const out = []
        for (let i = 0; i < files.count; i++) out.push(files.get(i).path)
        return out
    }
    // Kept as properties: a binding that reads ListModel.get() is not
    // re-evaluated when a row's size arrives, so every change calls recalc().
    property real totalBytes: 0
    property bool counting: false
    function recalc() {
        let t = 0
        let c = false
        for (let i = 0; i < files.count; i++) {
            const b = files.get(i).bytes
            if (b < 0) c = true
            else t += b
        }
        totalBytes = t
        counting = c
    }

    Connections {
        target: page.transfer
        function onPicked(paths) { page.add(paths) }
        function onMeasured(path, bytes) {
            for (let i = 0; i < files.count; i++)
                if (files.get(i).path === path) files.setProperty(i, "bytes", bytes)
            page.recalc()
        }
    }

    // A generic icon for the kind of file the name says, as file managers
    // show it.
    function fileIcon(name) {
        const ext = name.lastIndexOf(".") > 0 ? name.slice(name.lastIndexOf(".") + 1).toLowerCase() : ""
        const kinds = {
            "image-x-generic": ["jpg", "jpeg", "png", "gif", "webp", "heic", "heif", "avif", "bmp", "tif", "tiff", "svg", "raw", "dng"],
            "video-x-generic": ["mp4", "mkv", "mov", "webm", "avi", "m4v", "3gp"],
            "audio-x-generic": ["mp3", "flac", "ogg", "opus", "wav", "m4a", "aac"],
            "package-x-generic": ["zip", "tar", "gz", "xz", "zst", "bz2", "7z", "rar", "deb", "rpm"],
            "x-office-document": ["pdf", "doc", "docx", "odt", "rtf"],
        }
        for (const icon in kinds) if (kinds[icon].indexOf(ext) >= 0) return Theme.icon(icon)
        return Theme.icon("text-x-generic")
    }

    // Hidden editor for the clipboard.
    TextEdit { id: clip; visible: false }
    function copy(text) { clip.text = text; clip.selectAll(); clip.copy() }

    StackLayout {
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.margins: Theme.s24
        width: Theme.columnWidth(parent.width)
        currentIndex: {
            switch (page.phase) {
            case "idle": return page.textMode ? 6 : 0
            case "preparing": return 1
            case "waiting": return 2
            case "connected": return 3
            case "transferring": return 4
            default: return 5
            }
        }

        // 0: choose files
        ColumnLayout {
            spacing: Theme.s16

            ColumnLayout {
                visible: files.count === 0
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: Theme.s8
                // Fills the width too, so a wide window centres this.
                Item { Layout.fillHeight: true; Layout.fillWidth: true }
                IconImage {
                    name: Theme.icon("document-send")
                    color: Theme.muted
                    sourceSize: Qt.size(56, 56)
                    Layout.alignment: Qt.AlignHCenter
                }
                Label {
                    text: Strings.s.sendEmpty
                    color: Theme.ink
                    font.pointSize: Qt.application.font.pointSize * 1.25
                    font.weight: Font.DemiBold
                    Layout.alignment: Qt.AlignHCenter
                }
                Label {
                    text: Strings.s.sendEmptyHint + " " + Strings.s.sendDropHint
                    color: Theme.muted
                    wrapMode: Text.Wrap
                    horizontalAlignment: Text.AlignHCenter
                    Layout.fillWidth: true
                    Layout.maximumWidth: 380
                    Layout.alignment: Qt.AlignHCenter
                }
                PrimaryButton {
                    id: addButton
                    text: Strings.s.sendAddFiles
                    icon.name: Theme.icon("list-add")
                    onClicked: page.transfer.pickFiles(Strings.s.pickTitle)
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: Theme.s16
                }
                SecondaryButton {
                    text: Strings.s.sendText
                    icon.name: Theme.icon("insert-text")
                    onClicked: { page.mode = "text"; Qt.callLater(() => textBox.forceActiveFocus()) }
                    Layout.alignment: Qt.AlignHCenter
                }
                Item { Layout.fillHeight: true }
            }

            ListView {
                id: list
                visible: files.count > 0
                model: files
                clip: true
                Layout.fillWidth: true
                Layout.fillHeight: true
                // No taller than its rows, so "Add files" follows the last.
                Layout.maximumHeight: contentHeight
                ScrollBar.vertical: ScrollBar {}
                delegate: ItemDelegate {
                    id: row
                    required property int index
                    required property string name
                    required property real bytes
                    required property bool dir
                    width: ListView.view.width
                    contentItem: RowLayout {
                        spacing: Theme.s16
                        IconImage {
                            name: row.dir ? Theme.icon("folder") : page.fileIcon(row.name)
                            color: Theme.muted
                            sourceSize: Qt.size(24, 24)
                            Layout.preferredWidth: 24
                            Layout.preferredHeight: 24
                        }
                        ColumnLayout {
                            spacing: 0
                            Layout.fillWidth: true
                            Label { text: row.name; color: Theme.ink; elide: Text.ElideMiddle; Layout.fillWidth: true }
                            Label { text: row.bytes < 0 ? Strings.s.sendCounting : Strings.size(row.bytes); color: Theme.muted; font.features: { "tnum": 1 } }
                        }
                        IconButton {
                            icon.name: Theme.icon("list-remove")
                            text: Strings.s.sendRemove
                            onClicked: { files.remove(row.index); page.recalc() }
                        }
                    }
                }
            }
            // Under the list rather than its footer: a footer comes before
            // the rows in the Tab order.
            ItemDelegate {
                visible: files.count > 0
                text: Strings.s.sendAddFiles
                icon.name: Theme.icon("list-add")
                // Fusion leaves symbolic icons black; Breeze's already follow.
                icon.color: Theme.fusion ? Theme.ink : "transparent"
                // Delegates take no keyboard focus by default.
                focusPolicy: Qt.StrongFocus
                onClicked: page.transfer.pickFiles(Strings.s.pickTitle)
                Keys.onReturnPressed: (e) => { if (!e.isAutoRepeat) click() }
                Keys.onEnterPressed: (e) => { if (!e.isAutoRepeat) click() }
                Layout.fillWidth: true
                Layout.topMargin: -Theme.s16
                // Fusion shows no keyboard focus on a delegate.
                Rectangle {
                    visible: Theme.fusion && parent.visualFocus
                    anchors.fill: parent
                    radius: Theme.controlRadius
                    color: "transparent"
                    border.width: 2
                    border.color: Theme.accent
                }
            }
            Item { visible: files.count > 0; Layout.fillHeight: true }

            Divider { visible: files.count > 0; Layout.fillWidth: true; padding: 0 }

            ColumnLayout {
                visible: files.count > 0
                spacing: Theme.s8
                Layout.fillWidth: true
                Label {
                    text: Strings.f("itemsAndSize", { count: files.count, size: page.counting ? Strings.s.sendCounting : Strings.size(page.totalBytes) })
                    color: Theme.muted
                    font.features: { "tnum": 1 }
                }
                Toggle {
                    id: strongSwitch
                    text: Strings.s.sendStrongCode
                    checked: page.strong
                    onToggled: page.strong = checked
                    Layout.fillWidth: true
                }
                Label {
                    text: Strings.s.sendStrongCodeHint
                    color: Theme.muted
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                    Layout.leftMargin: strongSwitch.indicator ? strongSwitch.indicator.width + strongSwitch.spacing : 0
                }
                PrimaryButton {
                    id: createButton
                    text: Strings.s.sendCreateCode
                    enabled: files.count > 0
                    onClicked: page.sendIt()
                    Layout.fillWidth: true
                    Layout.topMargin: Theme.s8
                }
            }
        }

        // 1: preparing
        ColumnLayout {
            spacing: Theme.s24
            // Fills the width too: a layout with no child that does keeps
            // its implicit width, and the stack leaves it at the left.
            Item { Layout.fillHeight: true; Layout.fillWidth: true }
            Mark { size: 140; sending: page.phase === "preparing"; Layout.alignment: Qt.AlignHCenter }
            Label { text: Strings.s.sendPreparing; color: Theme.muted; Layout.alignment: Qt.AlignHCenter }
            Item { Layout.fillHeight: true }
        }

        // 2: the code, waiting for the receiver
        Flickable {
            id: waitFlick
            contentHeight: waiting.implicitHeight
            clip: true
            // In a short window the QR code pushes Cancel out of view: show
            // that the page scrolls, and bring Cancel into view on Tab.
            ScrollBar.vertical: ScrollBar {
                policy: waitFlick.contentHeight > waitFlick.height ? ScrollBar.AlwaysOn : ScrollBar.AsNeeded
            }
            FocusScroller { view: waitFlick; area: waiting }
            ColumnLayout {
                id: waiting
                width: parent.width
                spacing: Theme.s16

                Mark { size: 96; sending: page.phase === "waiting"; Layout.alignment: Qt.AlignHCenter }
                Label {
                    text: Strings.s.sendWaiting
                    color: Theme.muted
                    Layout.alignment: Qt.AlignHCenter
                }
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: codeCol.implicitHeight + 2 * Theme.s24
                    radius: Theme.radius
                    color: "transparent"
                    border.width: 1
                    border.color: Theme.rule
                    ColumnLayout {
                        id: codeCol
                        anchors.fill: parent
                        anchors.margins: Theme.s24
                        spacing: Theme.s8
                        Label { text: Strings.s.sendYourCode; color: Theme.muted; font.weight: Font.DemiBold; Layout.alignment: Qt.AlignHCenter }
                        TextEdit {
                            // The code is English words, left to right.
                            LayoutMirroring.enabled: false
                            LayoutMirroring.childrenInherit: true
                            horizontalAlignment: TextEdit.AlignHCenter
                            text: page.transfer.code
                            readOnly: true
                            selectByMouse: true
                            color: Theme.ink
                            selectionColor: Theme.accent
                            selectedTextColor: Theme.accentText
                            font.pointSize: Qt.application.font.pointSize * 2
                            font.weight: Font.Bold
                            wrapMode: TextEdit.WrapAnywhere
                            Layout.fillWidth: true
                            Accessible.role: Accessible.StaticText
                            Accessible.name: Strings.s.sendYourCode + " " + text
                        }
                        Label {
                            text: Strings.s.sendCodeHint
                            color: Theme.muted
                            wrapMode: Text.Wrap
                            horizontalAlignment: Text.AlignHCenter
                            Layout.fillWidth: true
                        }
                        SecondaryButton {
                            id: copyButton
                            text: Strings.s.sendCopy
                            icon.name: Theme.icon("edit-copy")
                            onClicked: { page.copy(page.transfer.code); copied.start() }
                            Timer { id: copied; interval: 2000 }
                            ToolTip.visible: copied.running
                            ToolTip.text: Strings.s.sendCopied
                            Layout.alignment: Qt.AlignHCenter
                        }
                    }
                }
                QrCode {
                    modules: page.transfer.qr
                    count: page.transfer.qrSize
                    side: 208
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: Theme.s8
                }
                Label {
                    text: Strings.s.sendQrHint
                    color: Theme.muted
                    wrapMode: Text.Wrap
                    horizontalAlignment: Text.AlignHCenter
                    Layout.fillWidth: true
                }
                Label {
                    text: Strings.s.sameNetworkNote
                    color: Theme.muted
                    wrapMode: Text.Wrap
                    horizontalAlignment: Text.AlignHCenter
                    font.pointSize: Qt.application.font.pointSize * 0.9
                    Layout.fillWidth: true
                    Layout.topMargin: Theme.s8
                }
                SecondaryButton {
                    text: Strings.s.cancel
                    onClicked: page.askCancel()
                    Layout.alignment: Qt.AlignHCenter
                }
            }
        }

        // 3: connected, compare the words
        ColumnLayout {
            spacing: Theme.s24
            Item { Layout.fillHeight: true }
            Mark { size: 96; sending: page.phase === "connected"; Layout.alignment: Qt.AlignHCenter }
            VerificationWords { words: page.transfer.words; Layout.fillWidth: true }
            Label {
                text: Strings.s.sendWaitingAccept
                color: Theme.muted
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
                Layout.fillWidth: true
            }
            Item { Layout.fillHeight: true }
            SecondaryButton { text: Strings.s.cancel; onClicked: page.askCancel(); Layout.alignment: Qt.AlignRight }
        }

        // 4: sending
        ColumnLayout {
            spacing: Theme.s24
            Item { Layout.fillHeight: true }
            TransferProgress {
                title: Strings.s.sendSending
                done: page.transfer.done
                total: page.transfer.total
                Layout.fillWidth: true
            }
            VerificationWords { words: page.transfer.words; Layout.fillWidth: true }
            Item { Layout.fillHeight: true }
            SecondaryButton { text: Strings.s.stop; onClicked: page.askCancel(); Layout.alignment: Qt.AlignRight }
        }

        // 5: result
        ResultPanel {
            // A text the receiver found damaged is not shown there: it did
            // not arrive (#326).
            readonly property bool textFailed: page.phase === "done" && page.textMode && page.transfer.filesFailed > 0
            success: page.phase === "done" && !textFailed
            title: textFailed ? Strings.s.errorTextDamaged
                 : page.phase === "done" ? Strings.s.sendDone
                 : Strings.s["error" + (page.transfer.errorKind || "Other")]
            body: textFailed ? ""
                  : page.phase === "done" && page.textMode ? Strings.s.sendTextDone
                  : page.phase === "done"
                  ? Strings.f("sendDoneBody", { count: page.transfer.filesOk })
                    + (page.transfer.filesFailed > 0 ? " " + Strings.f("sendSomeFailed", { count: page.transfer.filesFailed }) : "")
                  : ""
            detail: page.phase === "done" ? "" : page.transfer.errorDetail
            cause: page.phase === "done" ? "" : page.transfer.errorCause
            canRetry: page.phase === "failed" || textFailed
            onRetry: page.sendIt()
            onFinished: {
                page.transfer.reset(); files.clear(); page.recalc()
                page.clearText(); page.mode = "files"; page.close()
            }
        }

        // 6: write or paste a text (#326)
        ColumnLayout {
            spacing: Theme.s16

            ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                TextArea {
                    id: textBox
                    placeholderText: Strings.s.sendTextHint
                    // Plain text: pasted formatting is dropped. A long
                    // paste is shown in part and cannot be edited.
                    textFormat: TextEdit.PlainText
                    readOnly: page.bigText !== ""
                    wrapMode: TextEdit.Wrap
                    selectByMouse: true
                    Accessible.name: Strings.s.sendText
                    onTextChanged: page.textNotice = ""
                    // Ctrl+V pastes as the Paste button does.
                    Keys.onPressed: (e) => {
                        if (e.matches(StandardKey.Paste)) { page.pasteText(); e.accepted = true }
                    }
                }
            }
            Label {
                visible: page.bigText !== ""
                text: Strings.s.sendTextShortened
                color: Theme.muted
                wrapMode: Text.Wrap
                Layout.fillWidth: true
            }
            Label {
                visible: text !== ""
                text: page.textTooLong ? Strings.f("sendTextTooLong", { size: Strings.size(page.transfer.maxText()) })
                                       : page.textNotice
                color: Theme.error
                wrapMode: Text.Wrap
                Layout.fillWidth: true
            }
            RowLayout {
                spacing: Theme.s8
                Layout.fillWidth: true
                SecondaryButton {
                    visible: page.bigText === ""
                    text: Strings.s.receivePaste
                    icon.name: Theme.icon("edit-paste")
                    onClicked: { page.pasteText(); textBox.forceActiveFocus() }
                }
                SecondaryButton {
                    // A long paste cannot be edited, only taken away.
                    visible: page.bigText !== ""
                    text: Strings.s.sendRemove
                    icon.name: Theme.icon("edit-clear")
                    onClicked: { page.clearText(); textBox.forceActiveFocus() }
                }
                SecondaryButton {
                    // Back to files, while nothing is typed.
                    visible: textBox.text === ""
                    text: Strings.s.sendAddFiles
                    icon.name: Theme.icon("list-add")
                    onClicked: { page.mode = "files"; page.transfer.pickFiles(Strings.s.pickTitle) }
                }
                Item { Layout.fillWidth: true }
                Label {
                    visible: page.outgoing !== ""
                    text: Strings.size(page.textBytes)
                    color: page.textTooLong ? Theme.error : Theme.muted
                    font.features: { "tnum": 1 }
                }
            }
            Divider { Layout.fillWidth: true; padding: 0 }
            Toggle {
                id: textStrongSwitch
                text: Strings.s.sendStrongCode
                checked: page.strong
                onToggled: page.strong = checked
                Layout.fillWidth: true
            }
            Label {
                text: Strings.s.sendStrongCodeHint
                color: Theme.muted
                wrapMode: Text.Wrap
                Layout.fillWidth: true
                Layout.leftMargin: textStrongSwitch.indicator ? textStrongSwitch.indicator.width + textStrongSwitch.spacing : 0
            }
            PrimaryButton {
                text: Strings.s.sendCreateCode
                enabled: page.outgoing !== "" && !page.textTooLong
                onClicked: page.sendIt()
                Layout.fillWidth: true
                Layout.topMargin: Theme.s8
            }
        }
    }

    // Drag files onto the window.
    DropArea {
        id: drop
        anchors.fill: parent
        // Files dropped while nothing is typed go back to the file list.
        enabled: page.phase === "idle" && (!page.textMode || !page.hasInput)
        keys: ["text/uri-list"]
        onDropped: (d) => { if (d.hasUrls) { page.add(page.transfer.localPaths(d.urls)); d.accept(Qt.CopyAction) } }
    }
    Rectangle {
        visible: drop.containsDrag
        anchors.fill: parent
        anchors.margins: Theme.s16
        radius: Theme.radius
        color: Qt.alpha(Theme.background, 0.92)
        border.width: 2
        border.color: Theme.accent
        Label {
            anchors.centerIn: parent
            text: Strings.s.sendDropHere
            color: Theme.ink
            font.pointSize: Qt.application.font.pointSize * 1.35
            font.weight: Font.DemiBold
        }
    }
}
