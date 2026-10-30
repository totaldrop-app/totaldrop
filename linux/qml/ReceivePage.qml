import QtQuick
import QtQuick.Controls
import QtQuick.Controls.impl
import QtQuick.Layouts
import app.totaldrop

Page {
    id: page

    required property Transfer transfer
    property string initialCode: ""
    signal close()
    signal askCancel()

    readonly property string phase: transfer.role === "receive" ? transfer.state : "idle"
    readonly property bool running: ["connecting", "offer", "transferring"].indexOf(phase) >= 0
    property bool tried: false
    property string notice: ""
    // The code as read, shown while connecting: what is typed or pasted can
    // look different (direction marks, Arabic-Indic digits, a whole message).
    property string connectingCode: ""

    background: null

    header: PageHeader {
        title: Strings.s.receiveTitle
        onBack: page.running ? page.askCancel() : page.close()
    }

    Component.onCompleted: {
        if (initialCode !== "") field.text = initialCode
        Qt.callLater(() => field.forceActiveFocus())
    }

    Connections {
        target: page.transfer
        function onCodeFound(code) { field.text = code; page.notice = ""; page.tried = false }
        function onCodeNotFound() { page.notice = Strings.s.receiveQrNotFound }
    }

    onPhaseChanged: if (phase === "offer") Qt.callLater(() => acceptButton.forceActiveFocus())

    readonly property string normalized: transfer.normalizeCode(field.text)
    function connect() {
        page.tried = true
        if (page.normalized === "") return
        page.connectingCode = page.normalized
        page.transfer.receive(page.normalized)
    }

    // The word being typed, for completion: the last part, letters only,
    // after the number.
    readonly property string partial: {
        const parts = field.text.toLowerCase().split(/[\s-]+/)
        if (parts.length < 2) return ""
        const last = parts[parts.length - 1]
        return /^[a-z]{2,}$/.test(last) ? last : ""
    }
    readonly property var suggestions: {
        if (partial === "") return []
        const s = transfer.completions(partial, 4)
        return s.length === 1 && s[0] === partial ? [] : s
    }
    function complete(word) {
        const t = field.text.replace(/[a-zA-Z]+$/, "")
        field.text = t + word + "-"
        field.forceActiveFocus()
        field.cursorPosition = field.text.length
    }

    StackLayout {
        anchors.fill: parent
        anchors.margins: Theme.s24
        currentIndex: {
            switch (page.phase) {
            case "idle": return 0
            case "connecting": return 1
            case "offer": return 2
            case "transferring": return 3
            default: return 4
            }
        }

        // 0: enter the code
        ColumnLayout {
            spacing: Theme.s16

            Label { text: Strings.s.receiveCodeLabel; color: Theme.ink; font.weight: Font.DemiBold }
            TextField {
                id: field
                // The code is always English words, left to right.
                LayoutMirroring.enabled: false
                LayoutMirroring.childrenInherit: true
                horizontalAlignment: TextInput.AlignLeft
                placeholderText: Strings.s.receiveCodeHint
                inputMethodHints: Qt.ImhNoAutoUppercase | Qt.ImhNoPredictiveText | Qt.ImhPreferLowercase
                font.pointSize: Qt.application.font.pointSize * 1.35
                Layout.fillWidth: true
                Accessible.name: Strings.s.receiveCodeLabel
                focus: true
                onTextEdited: { page.tried = false; page.notice = "" }
                onAccepted: page.connect()
                Keys.onTabPressed: (e) => {
                    if (page.suggestions.length > 0) { page.complete(page.suggestions[0]); e.accepted = true }
                    else e.accepted = false
                }
            }

            // Word completion: the code words come from a fixed list.
            Flow {
                visible: page.suggestions.length > 0
                spacing: Theme.s8
                Layout.fillWidth: true
                Accessible.name: Strings.s.codeSuggestions
                Repeater {
                    model: page.suggestions
                    delegate: Button {
                        required property string modelData
                        text: modelData
                        flat: true
                        onClicked: page.complete(modelData)
                    }
                }
            }

            Label {
                text: page.tried && page.normalized === "" ? Strings.s.receiveCodeInvalid
                    : (page.notice !== "" ? page.notice : Strings.s.receiveCodeHelp)
                color: (page.tried && page.normalized === "") || page.notice !== "" ? Theme.error : Theme.muted
                wrapMode: Text.Wrap
                Layout.fillWidth: true
            }

            Flow {
                spacing: Theme.s8
                Layout.fillWidth: true
                Button {
                    text: Strings.s.receivePaste
                    icon.name: Theme.icon("edit-paste")
                    onClicked: {
                        field.selectAll(); field.paste()
                        const fromLink = page.transfer.codeFromLink(field.text)
                        if (fromLink !== "") field.text = fromLink
                        page.tried = false
                    }
                }
                Button {
                    text: Strings.s.receiveFromImage
                    icon.name: Theme.icon("view-barcode-qr", "view-barcode-qr")
                    onClicked: page.transfer.readQrImage(Strings.s.pickImageTitle)
                }
            }

            Item { Layout.fillHeight: true }

            Label {
                text: Strings.s.sameNetworkNote
                color: Theme.muted
                wrapMode: Text.Wrap
                font.pointSize: Qt.application.font.pointSize * 0.9
                Layout.fillWidth: true
            }
            PrimaryButton {
                text: Strings.s.receiveConnect
                enabled: field.text.trim() !== ""
                onClicked: page.connect()
                Layout.fillWidth: true
            }
        }

        // 1: connecting
        ColumnLayout {
            spacing: Theme.s24
            Item { Layout.fillHeight: true }
            Mark { size: 140; sending: page.phase === "connecting"; Layout.alignment: Qt.AlignHCenter }
            Label { text: Strings.s.receiveConnecting; color: Theme.muted; Layout.alignment: Qt.AlignHCenter }
            Label {
                text: page.connectingCode
                visible: text !== ""
                // The code is English words, left to right in every language.
                LayoutMirroring.enabled: false
                textFormat: Text.PlainText
                color: Theme.ink
                font.pointSize: Qt.application.font.pointSize * 1.35
                Layout.alignment: Qt.AlignHCenter
            }
            Item { Layout.fillHeight: true }
            Button { text: Strings.s.cancel; onClicked: page.transfer.cancel(); Layout.alignment: Qt.AlignRight }
        }

        // 2: the offer
        ColumnLayout {
            spacing: Theme.s16

            Label {
                text: Strings.s.receiveOffer
                color: Theme.ink
                font.pointSize: Qt.application.font.pointSize * 1.5
                font.weight: Font.DemiBold
            }
            Label {
                text: Strings.f("receiveOfferSummary", { files: page.transfer.offerFiles, size: Strings.size(page.transfer.size) })
                color: Theme.muted
                font.features: { "tnum": 1 }
            }
            ColumnLayout {
                spacing: 0
                Layout.fillWidth: true
                Repeater {
                    model: page.transfer.offerNames.slice(0, 5)
                    delegate: ColumnLayout {
                        required property string modelData
                        spacing: 0
                        Layout.fillWidth: true
                        MenuSeparator { Layout.fillWidth: true; padding: 0 }
                        Label {
                            // Names come from the other side: hidden direction
                            // characters are shown, not obeyed.
                            text: Strings.name(modelData)
                            color: Theme.ink
                            elide: Text.ElideMiddle
                            textFormat: Text.PlainText
                            Layout.fillWidth: true
                            Layout.topMargin: Theme.s8
                            Layout.bottomMargin: Theme.s8
                        }
                    }
                }
                Label {
                    visible: page.transfer.offerNames.length > 5
                    text: Strings.f("receiveAndMore", { count: page.transfer.offerNames.length - 5 })
                    color: Theme.muted
                }
            }
            VerificationWords { words: page.transfer.words; Layout.fillWidth: true; Layout.topMargin: Theme.s8 }
            Label {
                text: page.transfer.destination
                color: Theme.muted
                elide: Text.ElideMiddle
                Layout.fillWidth: true
            }
            Item { Layout.fillHeight: true }
            RowLayout {
                spacing: Theme.s8
                Layout.fillWidth: true
                Item { Layout.fillWidth: true }
                Button { text: Strings.s.receiveDecline; onClicked: page.transfer.decline() }
                PrimaryButton {
                    id: acceptButton
                    text: Strings.s.receiveAccept
                                        // The default button, as in a dialog: Enter accepts.
                    onClicked: page.transfer.accept()
                }
            }
        }

        // 3: receiving
        ColumnLayout {
            spacing: Theme.s24
            Item { Layout.fillHeight: true }
            TransferProgress {
                title: Strings.s.receiveReceiving
                done: page.transfer.done
                total: page.transfer.total
                Layout.fillWidth: true
            }
            Item { Layout.fillHeight: true }
            Button { text: Strings.s.stop; onClicked: page.askCancel(); Layout.alignment: Qt.AlignRight }
        }

        // 4: result
        ResultPanel {
            success: page.phase === "done"
            title: page.phase === "done" ? Strings.s.receiveDone
                 : page.phase === "declined" ? Strings.s.receiveDeclined
                 : Strings.s["error" + (page.transfer.errorKind || "Other")]
            body: page.phase === "done"
                  ? Strings.f("receiveSaveTo", { folder: page.transfer.savedTo })
                    + (page.transfer.filesFailed > 0 ? " " + Strings.f("sendSomeFailed", { count: page.transfer.filesFailed }) : "")
                  : ""
            folder: page.phase === "done" ? page.transfer.savedTo : ""
            detail: page.phase === "failed" ? page.transfer.errorDetail : ""
            onFinished: { page.transfer.reset(); page.close() }
        }
    }
}
