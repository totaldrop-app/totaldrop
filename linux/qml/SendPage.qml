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
    Component.onCompleted: if (initialFiles.length > 0) add(initialFiles)
    // With files listed, Enter creates the code.
    onPhaseChanged: if (phase === "idle") Qt.callLater(() => createButton.forceActiveFocus())
    Timer { running: true; interval: 0; onTriggered: (files.count > 0 ? createButton : addButton).forceActiveFocus() }

    background: null

    header: PageHeader {
        title: Strings.s.sendTitle
        onBack: page.running ? page.askCancel() : page.close()
    }

    ListModel { id: files }

    function add(paths) {
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

    // Hidden editor for the clipboard.
    TextEdit { id: clip; visible: false }
    function copy(text) { clip.text = text; clip.selectAll(); clip.copy() }

    StackLayout {
        anchors.fill: parent
        anchors.margins: Theme.s24
        currentIndex: {
            switch (page.phase) {
            case "idle": return 0
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
                Item { Layout.fillHeight: true }
            }

            ListView {
                id: list
                visible: files.count > 0
                model: files
                clip: true
                Layout.fillWidth: true
                Layout.fillHeight: true
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
                            name: row.dir ? Theme.icon("folder") : Theme.icon("text-x-generic", "text-x-generic")
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
                        ToolButton {
                            icon.name: Theme.icon("list-remove")
                            display: AbstractButton.IconOnly
                            text: Strings.s.sendRemove
                            ToolTip.visible: hovered
                            ToolTip.text: text
                            onClicked: { files.remove(row.index); page.recalc() }
                        }
                    }
                }
                footer: ItemDelegate {
                    width: ListView.view.width
                    text: Strings.s.sendAddFiles
                    icon.name: Theme.icon("list-add")
                    onClicked: page.transfer.pickFiles(Strings.s.pickTitle)
                }
            }

            MenuSeparator { visible: files.count > 0; Layout.fillWidth: true; padding: 0 }

            ColumnLayout {
                visible: files.count > 0
                spacing: Theme.s8
                Layout.fillWidth: true
                Label {
                    text: Strings.f("itemsAndSize", { count: files.count, size: page.counting ? Strings.s.sendCounting : Strings.size(page.totalBytes) })
                    color: Theme.muted
                    font.features: { "tnum": 1 }
                }
                Switch {
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
                    onClicked: page.transfer.send(page.paths(), page.strong)
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
            contentHeight: waiting.implicitHeight
            clip: true
            ScrollBar.vertical: ScrollBar {}
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
                        Label { text: Strings.s.sendYourCode; color: Theme.muted; font.weight: Font.DemiBold }
                        TextEdit {
                            // The code is English words, left to right.
                            LayoutMirroring.enabled: false
                            LayoutMirroring.childrenInherit: true
                            horizontalAlignment: TextEdit.AlignLeft
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
                            Layout.fillWidth: true
                        }
                        Button {
                            id: copyButton
                            text: Strings.s.sendCopy
                            icon.name: Theme.icon("edit-copy")
                            onClicked: { page.copy(page.transfer.code); copied.start() }
                            Timer { id: copied; interval: 2000 }
                            ToolTip.visible: copied.running
                            ToolTip.text: Strings.s.sendCopied
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
                    font.pointSize: Qt.application.font.pointSize * 0.9
                    Layout.fillWidth: true
                    Layout.topMargin: Theme.s8
                }
                Button {
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
            Button { text: Strings.s.cancel; onClicked: page.askCancel(); Layout.alignment: Qt.AlignRight }
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
            Button { text: Strings.s.stop; onClicked: page.askCancel(); Layout.alignment: Qt.AlignRight }
        }

        // 5: result
        ResultPanel {
            success: page.phase === "done"
            title: page.phase === "done" ? Strings.s.sendDone
                 : Strings.s["error" + (page.transfer.errorKind || "Other")]
            body: page.phase === "done"
                  ? Strings.f("sendDoneBody", { count: page.transfer.filesOk })
                    + (page.transfer.filesFailed > 0 ? " " + Strings.f("sendSomeFailed", { count: page.transfer.filesFailed }) : "")
                  : ""
            detail: page.phase === "done" ? "" : page.transfer.errorDetail
            canRetry: page.phase === "failed"
            onRetry: page.transfer.send(page.paths(), page.strong)
            onFinished: { page.transfer.reset(); files.clear(); page.recalc(); page.close() }
        }
    }

    // Drag files onto the window.
    DropArea {
        id: drop
        anchors.fill: parent
        enabled: page.phase === "idle"
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
