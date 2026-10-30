import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import app.totaldrop

Page {
    id: page

    signal send()
    signal receive()
    signal settings()

    background: null

    ToolButton {
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.margins: Theme.s8
        icon.name: Theme.icon("preferences-system", "configure")
        display: AbstractButton.IconOnly
        text: Strings.s.settings
        ToolTip.visible: hovered
        ToolTip.text: text
        onClicked: page.settings()
    }

    ColumnLayout {
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.margins: Theme.s24
        width: Theme.columnWidth(parent.width)
        spacing: Theme.s16

        Item { Layout.fillHeight: true; Layout.maximumHeight: Theme.s40 }

        Mark { size: 88 }

        Label {
            text: Strings.s.appTagline
            color: Theme.ink
            wrapMode: Text.Wrap
            font.pointSize: Qt.application.font.pointSize * 2
            font.weight: Font.Bold
            Layout.fillWidth: true
            Layout.topMargin: Theme.s8
            Layout.bottomMargin: Theme.s16
        }

        ActionCard {
            primary: true
            title: Strings.s.homeSendTitle
            body: Strings.s.homeSendBody
            iconName: Theme.icon("document-send")
            Layout.fillWidth: true
            onClicked: page.send()
        }
        ActionCard {
            title: Strings.s.homeReceiveTitle
            body: Strings.s.homeReceiveBody
            iconName: Theme.icon("document-save", "download")
            Layout.fillWidth: true
            onClicked: page.receive()
        }

        Item { Layout.fillHeight: true }

        Label {
            text: Strings.s.homeFooter
            color: Theme.muted
            wrapMode: Text.Wrap
            font.pointSize: Qt.application.font.pointSize * 0.9
            Layout.fillWidth: true
        }
    }
}
