import QtQuick
import QtQuick.Controls
import app.totaldrop

// The screen's one primary action. Under Plasma it is Breeze's default
// button. With Fusion (Omarchy, Hyprland, GNOME) it is filled with the
// desktop's accent, the way GTK desktops draw their suggested action, with
// the desktop's corner rounding.
Button {
    id: b

    highlighted: true
    // Breeze marks the default button from this (qqc2-desktop-style) or
    // from `highlighted` (qqc2-breeze-style, a blue frame).
    Accessible.defaultButton: true
    readonly property bool filled: SystemTheme.controlStyle === "Fusion"

    topPadding: filled ? 10 : undefined
    bottomPadding: filled ? 10 : undefined
    leftPadding: filled ? 20 : undefined
    rightPadding: filled ? 20 : undefined

    // Held down, Enter repeats; only the first press counts.
    Keys.onReturnPressed: (e) => { if (enabled && !e.isAutoRepeat) click() }
    Keys.onEnterPressed: (e) => { if (enabled && !e.isAutoRepeat) click() }

    Component.onCompleted: {
        if (!filled) return
        b.background = fill.createObject(b)
        b.palette.buttonText = Qt.binding(() => Theme.accentText)
        b.icon.color = Qt.binding(() => Theme.accentText)
        b.opacity = Qt.binding(() => b.enabled ? 1 : 0.5)
    }

    Component {
        id: fill
        Rectangle {
            implicitHeight: 36
            radius: Theme.controlRadius
            // Disabled, the whole button fades, text too, as GTK draws it:
            // a faint fill under full-strength text read as active.
            color: b.down ? Qt.tint(Theme.accent, Qt.alpha(Theme.accentText, 0.2))
                 : b.hovered ? Qt.tint(Theme.accent, Qt.alpha(Theme.accentText, 0.1))
                 : Theme.accent
            Rectangle {
                visible: b.visualFocus
                anchors.fill: parent
                anchors.margins: -3
                radius: parent.radius > 0 ? parent.radius + 3 : 0
                color: "transparent"
                border.width: 2
                border.color: Theme.ink
            }
        }
    }
}
