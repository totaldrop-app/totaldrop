import QtQuick
import QtQuick.Controls
import QtQuick.Controls.impl
import QtQuick.Layouts
import app.totaldrop

// The four words both sides see. Framed, because comparing them is what
// keeps the transfer safe.
Rectangle {
    id: box

    property var words: []

    implicitHeight: col.implicitHeight + 2 * 20
    radius: Theme.radius
    color: "transparent"
    border.width: 1
    border.color: Theme.rule

    Accessible.role: Accessible.StaticText
    Accessible.name: Strings.s.verificationTitle + ": " + words.join(" ")

    ColumnLayout {
        id: col
        anchors.fill: parent
        anchors.margins: 20
        spacing: Theme.s8

        RowLayout {
            spacing: Theme.s8
            IconImage {
                name: Theme.icon("security-high", "security-high")
                color: Theme.ink
                sourceSize: Qt.size(20, 20)
                Layout.preferredWidth: 20
                Layout.preferredHeight: 20
            }
            Label {
                text: Strings.s.verificationTitle
                color: Theme.ink
                font.weight: Font.DemiBold
            }
        }
        Label {
            text: box.words.join("  ·  ")
            color: Theme.ink
            font.pointSize: Qt.application.font.pointSize * 1.35
            font.weight: Font.DemiBold
            wrapMode: Text.Wrap
            textFormat: Text.PlainText
            Layout.fillWidth: true
        }
        Label {
            text: Strings.s.verificationHint
            color: Theme.muted
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }
    }
}
