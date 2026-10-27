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

    Flickable {
        anchors.fill: parent
        contentHeight: col.implicitHeight + 2 * Theme.s24
        clip: true
        ScrollBar.vertical: ScrollBar {}

        ColumnLayout {
            id: col
            x: Theme.s24
            y: Theme.s24
            width: parent.width - 2 * Theme.s24
            spacing: Theme.s8

            Label { text: Strings.s.settingsServer; color: Theme.ink; font.pointSize: Qt.application.font.pointSize * 1.25; font.weight: Font.DemiBold }
            Label { text: Strings.s.settingsServerHelp; color: Theme.muted; wrapMode: Text.Wrap; Layout.fillWidth: true }
            TextField {
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
                Button {
                    text: Strings.s.settingsServerReset
                    flat: true
                    enabled: page.transfer.server !== page.transfer.defaultServer
                    onClicked: { page.transfer.resetServer(); server.text = page.transfer.server; page.invalid = false }
                }
            }

            MenuSeparator { Layout.fillWidth: true; Layout.topMargin: Theme.s16; Layout.bottomMargin: Theme.s16 }

            Label { text: Strings.s.settingsPrivacy; color: Theme.ink; font.pointSize: Qt.application.font.pointSize * 1.25; font.weight: Font.DemiBold }
            Label { text: Strings.s.settingsPrivacyBody; color: Theme.ink; wrapMode: Text.Wrap; Layout.fillWidth: true }

            MenuSeparator { Layout.fillWidth: true; Layout.topMargin: Theme.s16; Layout.bottomMargin: Theme.s16 }

            Label { text: Strings.s.settingsAbout; color: Theme.ink; font.pointSize: Qt.application.font.pointSize * 1.25; font.weight: Font.DemiBold }
            Label { text: Strings.s.settingsAboutBody; color: Theme.ink; wrapMode: Text.Wrap; Layout.fillWidth: true }
            Label { text: Strings.f("settingsVersion", { version: Qt.application.version }); color: Theme.muted; Layout.topMargin: Theme.s8 }
            Label {
                text: "<a href=\"https://totaldrop.app\">totaldrop.app</a>"
                textFormat: Text.StyledText
                linkColor: Theme.accent
                color: Theme.muted
                onLinkActivated: (l) => Qt.openUrlExternally(l)
                HoverHandler { cursorShape: Qt.PointingHandCursor }
            }
        }
    }
}
