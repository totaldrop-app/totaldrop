import QtQuick
import QtQuick.Controls
import app.totaldrop

// A text field. With Fusion it takes the buttons' shape on the raised
// surface, with the accent as its focus outline, as GTK draws entries.
TextField {
    id: f

    Component.onCompleted: {
        if (!Theme.fusion) return
        f.background = surface.createObject(f)
        f.topPadding = 8
        f.bottomPadding = 8
        f.leftPadding = 12
        f.rightPadding = 12
    }

    Component {
        id: surface
        Rectangle {
            implicitHeight: 36
            radius: Math.min(Theme.radius, height / 2)
            color: Theme.raised
            border.width: f.activeFocus ? 2 : 1
            border.color: f.activeFocus ? Theme.accent : Theme.rule
        }
    }
}
