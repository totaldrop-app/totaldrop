import QtQuick
import QtQuick.Controls
import QtQuick.Controls.impl
import QtQuick.Layouts
import app.totaldrop

// The end of a transfer: success, a decline or an error. Errors say what
// happened and what to do; the technical detail sits under Details as
// selectable text.
ColumnLayout {
    id: r

    property bool success: false
    property string title
    property string body
    property string detail
    property string folder
    property bool canRetry: false

    signal finished()
    signal retry()

    spacing: Theme.s16

    Item {
        Layout.alignment: Qt.AlignHCenter
        Layout.preferredWidth: 140
        Layout.preferredHeight: 140
        Layout.bottomMargin: Theme.s16
        // The done badge: the resting mark with a check inside its circle.
        Mark { size: 140; done: true; visible: r.success }
        IconImage {
            visible: !r.success
            anchors.centerIn: parent
            name: Theme.icon("dialog-error", "dialog-error")
            color: Theme.error
            sourceSize: Qt.size(56, 56)
        }
    }

    Label {
        text: r.title
        color: Theme.ink
        font.pointSize: Qt.application.font.pointSize * 1.5
        font.weight: Font.DemiBold
        wrapMode: Text.Wrap
        horizontalAlignment: Text.AlignHCenter
        Layout.fillWidth: true
    }
    Label {
        visible: text !== ""
        text: r.body
        color: Theme.ink
        wrapMode: Text.Wrap
        horizontalAlignment: Text.AlignHCenter
        Layout.fillWidth: true
    }
    Button {
        visible: r.folder !== ""
        text: Strings.s.openFolder
        icon.name: Theme.icon("folder-open", "folder-open")
        onClicked: Qt.openUrlExternally("file://" + r.folder)
        Layout.alignment: Qt.AlignHCenter
    }

    ColumnLayout {
        visible: r.detail !== ""
        Layout.fillWidth: true
        spacing: Theme.s8
        Button {
            id: more
            flat: true
            checkable: true
            text: Strings.s.moreInfo
            icon.name: Theme.icon(checked ? "go-up" : "go-down")
            Layout.alignment: Qt.AlignHCenter
        }
        TextArea {
            visible: more.checked
            // May hold the other side's cancel reason: hidden direction
            // characters are shown, not obeyed.
            text: Strings.name(r.detail)
            readOnly: true
            selectByMouse: true
            wrapMode: Text.Wrap
            font.family: "monospace"
            Layout.fillWidth: true
        }
    }

    Item { Layout.fillHeight: true }

    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.s8
        Item { Layout.fillWidth: true }
        Button {
            visible: r.canRetry
            text: Strings.s.tryAgain
            onClicked: r.retry()
        }
        PrimaryButton {
            text: Strings.s.done
            onClicked: r.finished()
        }
    }
}
