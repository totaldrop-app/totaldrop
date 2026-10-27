import QtQuick
import QtQuick.Controls
import app.totaldrop

ApplicationWindow {
    id: win

    title: "TotalDrop"
    width: 480
    height: 720
    minimumWidth: 400
    minimumHeight: 600
    visible: true
    color: Theme.background

    // Outside Plasma (and qt6ct / hyprqt6engine) the controls take their
    // colours from the desktop theme through the window palette. Under
    // those, the Qt palette already is the desktop theme: leave it alone.
    Binding { target: win.palette; property: "window"; value: Theme.background; when: !Theme.system }
    Binding { target: win.palette; property: "windowText"; value: Theme.ink; when: !Theme.system }
    Binding { target: win.palette; property: "base"; value: Theme.background; when: !Theme.system }
    Binding { target: win.palette; property: "alternateBase"; value: Theme.raised; when: !Theme.system }
    Binding { target: win.palette; property: "text"; value: Theme.ink; when: !Theme.system }
    Binding { target: win.palette; property: "button"; value: Theme.raised; when: !Theme.system }
    Binding { target: win.palette; property: "buttonText"; value: Theme.ink; when: !Theme.system }
    Binding { target: win.palette; property: "brightText"; value: Theme.accentText; when: !Theme.system }
    Binding { target: win.palette; property: "highlight"; value: Theme.accent; when: !Theme.system }
    Binding { target: win.palette; property: "highlightedText"; value: Theme.accentText; when: !Theme.system }
    Binding { target: win.palette; property: "accent"; value: Theme.accent; when: !Theme.system }
    Binding { target: win.palette; property: "placeholderText"; value: Theme.muted; when: !Theme.system }
    Binding { target: win.palette; property: "link"; value: Theme.accent; when: !Theme.system }
    Binding { target: win.palette; property: "mid"; value: Theme.rule; when: !Theme.system }
    Binding { target: win.palette; property: "midlight"; value: Qt.tint(Theme.raised, Qt.alpha(Theme.ink, 0.06)); when: !Theme.system }
    Binding { target: win.palette; property: "light"; value: Qt.tint(Theme.raised, Qt.alpha(Theme.ink, 0.12)); when: !Theme.system }
    Binding { target: win.palette; property: "dark"; value: Qt.tint(Theme.background, Qt.alpha(Theme.ink, 0.3)); when: !Theme.system }
    Binding { target: win.palette; property: "shadow"; value: Qt.rgba(0, 0, 0, 0.5); when: !Theme.system }
    Binding { target: win.palette; property: "toolTipBase"; value: Theme.raised; when: !Theme.system }
    Binding { target: win.palette; property: "toolTipText"; value: Theme.ink; when: !Theme.system }

    Transfer { id: xfer }

    // A TotalDrop link on the command line (from a QR, the browser or the
    // file manager) opens Receive with the code filled in.
    function codeFromArgs() {
        for (const a of Array.from(Qt.application.arguments).slice(1)) {
            if (a === "--screen=send") continue
            const c = xfer.codeFromLink(a)
            if (c !== "") return c
        }
        return ""
    }

    // Files on the command line ("Open with TotalDrop" in a file manager)
    // open Send with them listed.
    function filesFromArgs() {
        return Array.from(Qt.application.arguments).slice(1)
            .filter(a => a.startsWith("/") || a.startsWith("file://"))
    }

    StackView {
        id: stack
        anchors.fill: parent
        initialItem: HomePage {
            onSend: stack.push(sendPage)
            onReceive: stack.push(receivePage)
            onSettings: stack.push(settingsPage)
        }
        Component.onCompleted: Qt.callLater(() => {
            const args = Array.from(Qt.application.arguments)
            const code = win.codeFromArgs()
            if (code !== "") stack.push(receivePage, { initialCode: code }, StackView.Immediate)
            else if (win.filesFromArgs().length > 0) stack.push(sendPage, { initialFiles: win.filesFromArgs() }, StackView.Immediate)
            else if (args.indexOf("--screen=send") >= 0) stack.push(sendPage, {}, StackView.Immediate)
            else if (args.indexOf("--screen=receive") >= 0) stack.push(receivePage, {}, StackView.Immediate)
            else if (args.indexOf("--screen=settings") >= 0) stack.push(settingsPage, {}, StackView.Immediate)
        })
    }

    Component {
        id: sendPage
        SendPage { transfer: xfer; onClose: stack.pop(); onAskCancel: stopDialog.open() }
    }
    Component {
        id: receivePage
        ReceivePage { transfer: xfer; onClose: stack.pop(); onAskCancel: stopDialog.open() }
    }
    Component {
        id: settingsPage
        SettingsPage { transfer: xfer; onClose: stack.pop() }
    }

    Dialog {
        id: stopDialog
        anchors.centerIn: parent
        width: Math.min(parent.width - 2 * Theme.s24, 420)
        modal: true
        title: Strings.s.cancelTransferTitle
        Label { text: Strings.s.cancelTransferBody; wrapMode: Text.Wrap; width: parent.width; color: Theme.ink }
        footer: DialogButtonBox {
            Button { text: Strings.s.keepGoing; DialogButtonBox.buttonRole: DialogButtonBox.RejectRole }
            Button { text: Strings.s.stop; DialogButtonBox.buttonRole: DialogButtonBox.DestructiveRole
                     onClicked: { xfer.cancel(); stopDialog.close() } }
        }
    }

    // Tell the person when a transfer ends while the window is in the background.
    Connections {
        target: xfer
        function onStateChanged() {
            if (win.active) return
            const send = xfer.role === "send"
            if (xfer.state === "done")
                xfer.notify(send ? Strings.s.sendDone : Strings.s.receiveDone,
                                send ? Strings.f("sendDoneBody", { count: xfer.filesOk })
                                     : Strings.f("receiveSaveTo", { folder: xfer.savedTo }))
            else if (xfer.state === "failed" || xfer.state === "declined")
                xfer.notify("TotalDrop", Strings.s["error" + (xfer.errorKind || "Other")])
            else if (xfer.state === "offer")
                xfer.notify(Strings.s.receiveOffer,
                                Strings.f("receiveOfferSummary", { files: xfer.offerFiles, size: Strings.size(xfer.size) }))
        }
    }
}
