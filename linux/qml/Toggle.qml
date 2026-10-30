import QtQuick
import QtQuick.Controls
import app.totaldrop

// A switch. With Fusion it is drawn as GTK draws one: a pill, filled with
// the accent when on, with a round knob. Plasma keeps Breeze's.
Switch {
    id: s

    Component.onCompleted: {
        if (!Theme.fusion) return
        s.indicator = pill.createObject(s)
    }

    Component {
        id: pill
        Rectangle {
            implicitWidth: 46
            implicitHeight: 26
            x: s.mirrored ? s.width - width - s.rightPadding : s.leftPadding
            y: s.topPadding + (s.availableHeight - height) / 2
            radius: height / 2
            color: s.checked ? Theme.accent : Qt.tint(Theme.raised, Qt.alpha(Theme.ink, 0.16))
            opacity: s.enabled ? 1 : 0.6
            Rectangle {
                width: 20
                height: 20
                radius: 10
                y: 3
                x: s.checked !== s.mirrored ? parent.width - width - 3 : 3
                // GTK draws the knob white, on or off.
                color: "#ffffff"
                // GTK shades its edge so it shows on a light track.
                border.width: 1
                border.color: Qt.rgba(0, 0, 0, 0.15)
                Behavior on x { enabled: !Theme.reducedMotion; NumberAnimation { duration: 120 } }
            }
            Rectangle {
                visible: s.visualFocus
                anchors.fill: parent
                anchors.margins: -3
                radius: height / 2
                color: "transparent"
                border.width: 2
                border.color: Theme.accent
            }
        }
    }
}
