import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import app.totaldrop

// A flat header: back button and title, drawn by the control style.
ToolBar {
    id: bar

    property string title
    signal back()

    background: null
    padding: Theme.s8

    RowLayout {
        anchors.fill: parent
        spacing: Theme.s8

        ToolButton {
            // Qt does not pick -rtl icon variants: flip the arrow itself.
            icon.name: Theme.icon(Strings.rtl ? "go-next" : "go-previous")
            display: AbstractButton.IconOnly
            text: Strings.s.back
            ToolTip.visible: hovered
            ToolTip.text: text
            onClicked: bar.back()
        }
        Label {
            text: bar.title
            color: Theme.ink
            font.pointSize: Qt.application.font.pointSize * 1.25
            font.weight: Font.DemiBold
            elide: Text.ElideRight
            Layout.fillWidth: true
        }
    }
}
