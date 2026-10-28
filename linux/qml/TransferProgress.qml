import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import app.totaldrop

// The moving mark, a bar and the numbers, while data flows.
ColumnLayout {
    id: p

    property string title
    property real done: 0
    property real total: 0

    readonly property real fraction: total > 0 ? Math.min(1, done / total) : 0

    spacing: Theme.s16

    Mark { size: 140; sending: true; Layout.alignment: Qt.AlignHCenter; Layout.bottomMargin: Theme.s16 }

    Label {
        text: p.title
        color: Theme.ink
        font.pointSize: Qt.application.font.pointSize * 1.5
        font.weight: Font.DemiBold
        Layout.alignment: Qt.AlignHCenter
    }
    ProgressBar {
        from: 0
        to: 1
        value: p.fraction
        Layout.fillWidth: true
        Accessible.name: p.title
    }
    RowLayout {
        Layout.fillWidth: true
        Label {
            text: Strings.f("progressPercent", { percent: Math.floor(p.fraction * 100) })
            color: Theme.ink
            font.features: { "tnum": 1 }
        }
        Item { Layout.fillWidth: true }
        Label {
            text: Strings.size(p.done) + " / " + Strings.size(p.total)
            color: Theme.muted
            font.features: { "tnum": 1 }
        }
    }
}
