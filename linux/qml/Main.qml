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

    // Right-to-left languages mirror the whole layout. The code and the
    // verification words opt out where they are shown (ADR 0008).
    LayoutMirroring.enabled: Strings.rtl
    LayoutMirroring.childrenInherit: true

    // Outside Plasma (and qt6ct / hyprqt6engine) the controls take their
    // colours from the desktop theme through the window palette, re-applied
    // whenever the theme changes. Under those, the Qt palette already is the
    // desktop theme: it is left alone.
    function applyPalette() {
        if (Theme.system) return
        const p = win.palette
        p.window = Theme.background
        p.windowText = Theme.ink
        p.base = Theme.background
        p.alternateBase = Theme.raised
        p.text = Theme.ink
        p.button = Theme.raised
        p.buttonText = Theme.ink
        p.brightText = Theme.accentText
        p.highlight = Theme.accent
        p.highlightedText = Theme.accentText
        p.accent = Theme.accent
        p.placeholderText = Theme.muted
        p.link = Theme.accent
        p.mid = Theme.rule
        p.midlight = Qt.tint(Theme.raised, Qt.alpha(Theme.ink, 0.06))
        p.light = Qt.tint(Theme.raised, Qt.alpha(Theme.ink, 0.12))
        p.dark = Qt.tint(Theme.background, Qt.alpha(Theme.ink, 0.3))
        p.shadow = Qt.rgba(0, 0, 0, 0.5)
        p.toolTipBase = Theme.raised
        p.toolTipText = Theme.ink
        // Disabled controls read as disabled.
        p.disabled.windowText = Theme.muted
        p.disabled.text = Theme.muted
        p.disabled.buttonText = Theme.muted
        p.disabled.highlight = Theme.rule
        p.disabled.highlightedText = Theme.muted
    }
    readonly property string paletteKey: [Theme.system, Theme.background, Theme.raised, Theme.ink,
        Theme.muted, Theme.rule, Theme.accent, Theme.accentText].join()
    onPaletteKeyChanged: applyPalette()
    Component.onCompleted: applyPalette()

    Transfer { id: xfer }

    // What the command line asks for, and what a later launch hands this
    // window (#178): a TotalDrop link (from a QR, the browser or the file
    // manager) opens Receive with the code filled in; files ("Open with
    // TotalDrop") open Send with them listed; otherwise --screen=… opens
    // that screen. Launcher has already made paths absolute.
    Launcher {
        id: launcher
        onOpened: (args) => {
            win.raiseWindow()
            win.openArgs(args)
        }
    }

    function raiseWindow() {
        if (win.visibility === Window.Minimized || win.visibility === Window.Hidden) win.showNormal()
        win.raise()
        win.requestActivate()
    }

    // A transfer under way is not interrupted: the window only comes to
    // the front.
    readonly property bool busy: ["idle", "done", "failed", "declined"].indexOf(xfer.state) < 0

    function openArgs(args) {
        let code = ""
        for (const a of args) {
            if (a.startsWith("--screen=")) continue
            code = xfer.codeFromLink(a)
            if (code !== "") break
        }
        const files = args.filter(a => a.startsWith("/"))
        const screens = { "--screen=send": sendPage, "--screen=receive": receivePage, "--screen=settings": settingsPage }
        let page = null
        for (const a of args) if (screens[a]) page = screens[a]
        if (code !== "") show(receivePage, { initialCode: code })
        else if (files.length > 0) show(sendPage, { initialFiles: files })
        else if (page) show(page, {})
    }

    // Open `page` with `props`, unless a transfer is under way. Files for
    // a Send screen already open join its list.
    function show(page, props) {
        if (busy) return
        const top = stack.currentItem
        const idle = xfer.state === "idle"
        if (page === sendPage && top instanceof SendPage && idle) {
            if (props.initialFiles) top.add(props.initialFiles)
            return
        }
        if (page === receivePage && top instanceof ReceivePage && idle && !props.initialCode) return
        if (page === settingsPage && top instanceof SettingsPage) return
        if (xfer.state !== "idle") xfer.reset()
        stack.pop(null, StackView.Immediate)
        stack.push(page, props, StackView.Immediate)
    }

    StackView {
        id: stack
        anchors.fill: parent
        initialItem: HomePage {
            onSend: stack.push(sendPage)
            onReceive: stack.push(receivePage)
            onSettings: stack.push(settingsPage)
        }
        Component.onCompleted: Qt.callLater(() => win.openArgs(launcher.startArgs()))
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
        modal: true
        Component.onCompleted: if (Theme.fusion) Overlay.modal = Theme.dim
        title: Strings.s.cancelTransferTitle
        // A fixed text width: wrapped text in a self-sizing popup loops.
        contentWidth: Math.min(380, win.width - 4 * Theme.s24)
        Label {
            width: stopDialog.contentWidth
            text: Strings.s.cancelTransferBody
            wrapMode: Text.Wrap
            color: Theme.ink
        }
        footer: DialogButtonBox {
            SecondaryButton { text: Strings.s.keepGoing; DialogButtonBox.buttonRole: DialogButtonBox.RejectRole }
            SecondaryButton { text: Strings.s.stop; DialogButtonBox.buttonRole: DialogButtonBox.DestructiveRole
                     onClicked: { xfer.cancel(); stopDialog.close() } }
        }
    }

    // Tell the person when a transfer ends while the window is in the background.
    Connections {
        target: xfer
        function onStateChanged() {
            // The stop question is moot once the transfer has ended; left
            // open, Enter would go to the result screen under it.
            if (["done", "failed", "declined", "idle"].indexOf(xfer.state) >= 0) stopDialog.close()
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
