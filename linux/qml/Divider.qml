import QtQuick
import QtQuick.Controls
import app.totaldrop

// A dividing line. Fusion draws it from the Qt palette, which GTK desktops
// do not set, so in a dark theme it vanished; it takes the theme's rule
// colour there. Plasma keeps Breeze's.
MenuSeparator {
    id: d
    Component.onCompleted: {
        if (!Theme.fusion) return
        d.contentItem = line.createObject(d)
    }
    Component {
        id: line
        Rectangle {
            implicitHeight: 1
            color: Theme.rule
        }
    }
}
