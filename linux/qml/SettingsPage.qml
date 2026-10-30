import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import app.totaldrop

Page {
    id: page

    required property Transfer transfer
    signal close()

    property bool invalid: false

    background: null

    header: PageHeader {
        title: Strings.s.settings
        onBack: page.close()
    }

    // Escape first drops an unsaved server address, then does what the
    // back button does.
    Shortcut {
        sequence: StandardKey.Cancel
        enabled: page.StackView.status === StackView.Active
        onActivated: {
            if (server.text !== page.transfer.server) {
                server.text = page.transfer.server
                page.invalid = false
            } else page.close()
        }
    }

    Flickable {
        id: flick
        anchors.fill: parent
        contentHeight: col.implicitHeight + 2 * Theme.s24
        clip: true
        // Shown whenever the page is taller than the window, so it is
        // clear that there is more below.
        ScrollBar.vertical: ScrollBar {
            policy: flick.contentHeight > flick.height ? ScrollBar.AlwaysOn : ScrollBar.AsNeeded
        }

        // Tab to a control out of view scrolls it into view.
        Connections {
            target: page.Window.window
            function onActiveFocusItemChanged() {
                const item = page.Window.activeFocusItem
                let inside = false
                for (let p = item; p && !inside; p = p.parent) inside = p === col
                if (!inside) return
                const top = item.mapToItem(flick.contentItem, 0, 0).y
                const bottom = top + item.height
                if (top < flick.contentY + Theme.s24)
                    flick.contentY = Math.max(0, top - Theme.s24)
                else if (bottom > flick.contentY + flick.height - Theme.s24)
                    flick.contentY = Math.min(flick.contentHeight - flick.height, bottom - flick.height + Theme.s24)
            }
        }

        ColumnLayout {
            id: col
            x: (parent.width - width) / 2
            y: Theme.s24
            width: Theme.columnWidth(parent.width)
            spacing: Theme.s8

            Label { text: Strings.s.settingsServer; color: Theme.ink; font.pointSize: Qt.application.font.pointSize * 1.25; font.weight: Font.DemiBold }
            Label { text: Strings.s.settingsServerHelp; color: Theme.muted; wrapMode: Text.Wrap; Layout.fillWidth: true }
            Field {
                id: server
                text: page.transfer.server
                inputMethodHints: Qt.ImhUrlCharactersOnly | Qt.ImhNoAutoUppercase
                Layout.fillWidth: true
                Layout.topMargin: Theme.s8
                Accessible.name: Strings.s.settingsServer
                onTextEdited: page.invalid = false
                onAccepted: save()
                function save() { page.invalid = !page.transfer.saveServer(text.trim()) }
            }
            Label {
                visible: page.invalid
                text: Strings.s.settingsServerInvalid
                color: Theme.error
            }
            RowLayout {
                spacing: Theme.s8
                Layout.topMargin: Theme.s8
                PrimaryButton {
                    text: Strings.s.settingsSave
                    enabled: server.text.trim() !== page.transfer.server
                    onClicked: server.save()
                }
                SecondaryButton {
                    text: Strings.s.settingsServerReset
                    flat: true
                    enabled: page.transfer.server !== page.transfer.defaultServer
                    onClicked: { page.transfer.resetServer(); server.text = page.transfer.server; page.invalid = false }
                }
            }

            Divider { Layout.fillWidth: true; Layout.topMargin: Theme.s16; Layout.bottomMargin: Theme.s16 }

            Label { text: Strings.s.settingsPrivacy; color: Theme.ink; font.pointSize: Qt.application.font.pointSize * 1.25; font.weight: Font.DemiBold }
            Label { text: Strings.s.settingsPrivacyBody; color: Theme.ink; wrapMode: Text.Wrap; Layout.fillWidth: true }
            Toggle {
                id: privacy
                text: Strings.s.privacyMode
                checked: page.transfer.privacyMode
                Layout.fillWidth: true
                Layout.topMargin: Theme.s8
                // Turning it on is confirmed first: it is slower. The switch
                // always shows what is saved, so a failed save shows too.
                onToggled: {
                    const on = checked
                    checked = Qt.binding(() => page.transfer.privacyMode)
                    if (on) privacyDialog.open()
                    else page.transfer.savePrivacyMode(false)
                }
            }
            Label {
                text: Strings.s.privacyModeHint
                color: Theme.muted
                wrapMode: Text.Wrap
                Layout.fillWidth: true
                Layout.leftMargin: privacy.indicator ? privacy.indicator.width + privacy.spacing : 0
            }

            Divider { Layout.fillWidth: true; Layout.topMargin: Theme.s16; Layout.bottomMargin: Theme.s16 }

            Label { text: Strings.s.settingsAbout; color: Theme.ink; font.pointSize: Qt.application.font.pointSize * 1.25; font.weight: Font.DemiBold }
            Label { text: Strings.s.settingsAboutBody; color: Theme.ink; wrapMode: Text.Wrap; Layout.fillWidth: true }
            Label {
                visible: Strings.draft
                text: Strings.s.draftTranslation
                color: Theme.muted
                wrapMode: Text.Wrap
                Layout.fillWidth: true
                Layout.topMargin: Theme.s8
            }
            Label { text: Strings.f("settingsVersion", { version: Qt.application.version }); color: Theme.muted; Layout.topMargin: Theme.s8 }
            Label {
                id: site
                readonly property string url: "https://totaldrop.app"
                text: "<a href=\"" + url + "\">totaldrop.app</a>"
                textFormat: Text.StyledText
                linkColor: Theme.accent
                color: Theme.muted
                onLinkActivated: (l) => Qt.openUrlExternally(l)
                HoverHandler { cursorShape: Qt.PointingHandCursor }
                // Reached with Tab and opened with Enter, like a link in a
                // browser, or Space, like the buttons; the focus ring is the
                // accent outline.
                activeFocusOnTab: true
                Accessible.role: Accessible.Link
                Accessible.name: "totaldrop.app"
                Accessible.onPressAction: Qt.openUrlExternally(url)
                Keys.onReturnPressed: (e) => { if (!e.isAutoRepeat) Qt.openUrlExternally(url) }
                Keys.onEnterPressed: (e) => { if (!e.isAutoRepeat) Qt.openUrlExternally(url) }
                Keys.onSpacePressed: (e) => { if (!e.isAutoRepeat) Qt.openUrlExternally(url) }
                Rectangle {
                    visible: site.activeFocus
                    anchors.fill: parent
                    anchors.margins: -2
                    radius: 2
                    color: "transparent"
                    border.width: 1
                    border.color: Theme.accent
                }
            }
        }
    }

    Dialog {
        id: privacyDialog
        anchors.centerIn: parent
        modal: true
        Component.onCompleted: if (Theme.fusion) Overlay.modal = Theme.dim
        title: Strings.s.privacyModeConfirmTitle
        // A fixed text width: wrapped text in a self-sizing popup loops.
        contentWidth: Math.min(380, page.width - 4 * Theme.s24)
        Label {
            width: privacyDialog.contentWidth
            text: Strings.s.privacyModeConfirmBody
            wrapMode: Text.Wrap
            color: Theme.ink
        }
        footer: DialogButtonBox {
            SecondaryButton { text: Strings.s.cancel; DialogButtonBox.buttonRole: DialogButtonBox.RejectRole }
            SecondaryButton { text: Strings.s.privacyModeTurnOn; DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole }
        }
        onAccepted: page.transfer.savePrivacyMode(true)
    }
}
