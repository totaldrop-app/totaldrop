import QtQuick
import QtQuick.Controls
import QtQuick.Controls.impl
import QtQuick.Layouts
import app.totaldrop

// A large tappable unit on the home screen. `primary` carries the screen's
// one accent; the other sits calmly on the raised surface.
AbstractButton {
    id: card

    property string title
    property string body
    property string iconName
    property bool primary: false

    readonly property color fg: primary ? Theme.accentText : Theme.ink

    implicitHeight: Math.max(96, content.implicitHeight + 2 * Theme.s24)
    hoverEnabled: true
    focusPolicy: Qt.StrongFocus
    Accessible.role: Accessible.Button
    Accessible.name: title
    Accessible.description: body

    background: Rectangle {
        radius: Theme.radius
        color: {
            const base = card.primary ? Theme.accent : Theme.raised
            if (card.down) return Qt.tint(base, Qt.alpha(card.fg, 0.16))
            if (card.hovered) return Qt.tint(base, Qt.alpha(card.fg, 0.08))
            return base
        }
        // Focus ring: 3 px of the accent, set off by the background.
        Rectangle {
            visible: card.visualFocus
            anchors.fill: parent
            anchors.margins: -4
            radius: Theme.radius > 0 ? Theme.radius + 4 : 0
            color: "transparent"
            border.width: 3
            border.color: card.primary ? Theme.ink : Theme.accent
        }
    }

    contentItem: RowLayout {
        id: content
        spacing: Theme.s16
        anchors.fill: parent
        anchors.margins: Theme.s24

        IconImage {
            name: card.iconName
            color: card.fg
            sourceSize: Qt.size(32, 32)
            Layout.preferredWidth: 32
            Layout.preferredHeight: 32
            Layout.alignment: Qt.AlignVCenter
        }
        ColumnLayout {
            spacing: 2
            Layout.fillWidth: true
            Label {
                text: card.title
                color: card.fg
                font.pointSize: Qt.application.font.pointSize * 1.35
                font.weight: Font.DemiBold
                Layout.fillWidth: true
            }
            Label {
                text: card.body
                color: card.fg
                opacity: card.primary ? 0.9 : 1
                wrapMode: Text.Wrap
                Layout.fillWidth: true
            }
        }
    }
}
