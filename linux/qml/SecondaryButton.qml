import QtQuick
import QtQuick.Controls
import app.totaldrop

// Every button that is not the screen's primary action. Under Plasma it is
// Breeze's own button. With Fusion (Omarchy, Hyprland, GNOME) it takes the
// same shape as PrimaryButton, on the raised surface, as GTK draws its
// buttons; flat ones show the surface only on hover. Fusion leaves icons
// in their file's colour, which a dark theme cannot show: they take the
// text colour.
// Held down, Enter repeats; only the first press counts.
Button {
    id: b

    readonly property bool styled: Theme.fusion

    topPadding: styled ? 8 : undefined
    bottomPadding: styled ? 8 : undefined
    leftPadding: styled ? 16 : undefined
    rightPadding: styled ? 16 : undefined

    Keys.onReturnPressed: (e) => { if (enabled && !e.isAutoRepeat) click() }
    Keys.onEnterPressed: (e) => { if (enabled && !e.isAutoRepeat) click() }

    Component.onCompleted: {
        if (!styled) return
        b.background = surface.createObject(b)
        b.icon.color = Qt.binding(() => b.enabled ? Theme.ink : Theme.muted)
    }

    Component {
        id: surface
        Rectangle {
            implicitHeight: 34
            radius: Math.min(Theme.radius, height / 2)
            color: b.down || b.checked ? Qt.tint(Theme.raised, Qt.alpha(Theme.ink, 0.16))
                 : b.hovered ? Qt.tint(Theme.raised, Qt.alpha(Theme.ink, 0.08))
                 : b.flat ? "transparent"
                 : Theme.raised
            border.width: b.flat ? 0 : 1
            border.color: Theme.rule
            opacity: b.enabled ? 1 : 0.6
            Rectangle {
                visible: b.visualFocus
                anchors.fill: parent
                anchors.margins: -3
                radius: parent.radius > 0 ? parent.radius + 3 : 0
                color: "transparent"
                border.width: 2
                border.color: Theme.accent
            }
        }
    }
}
