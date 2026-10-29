import QtQuick
import QtQuick.Shapes
import app.totaldrop

// The Köşe mark: a circle with one square corner, at top right when at rest.
// It matches the SVG. While `sending`, it plays the website's animation
// exactly (docs/tasarim.md, "Hareket"): four quarter turns clockwise around
// the circle's centre, each 0.6 s on cubic-bezier(0.65, 0, 0.35, 1) and
// followed by a 0.2 s rest, 3.2 s in all. `done` draws a check inside.
Item {
    id: root

    property real size: 96
    property bool sending: false
    property bool done: false

    implicitWidth: size
    implicitHeight: size
    width: size
    height: size

    Accessible.role: Accessible.Graphic
    Accessible.name: "TotalDrop"

    readonly property real unit: size / 112
    readonly property bool moving: sending && !Theme.reducedMotion && visible

    // Degrees turned from rest, clockwise.
    property real angle: 0

    Shape {
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer
        antialiasing: true
        transform: Rotation { origin.x: 55 * root.unit; origin.y: 57 * root.unit; angle: root.angle }

        ShapePath {
            fillColor: Theme.mark
            strokeColor: "transparent"
            startX: 55 * root.unit; startY: 15 * root.unit
            PathLine { x: 97 * root.unit; y: 15 * root.unit }
            PathLine { x: 97 * root.unit; y: 57 * root.unit }
            PathArc {
                x: 55 * root.unit; y: 15 * root.unit
                radiusX: 42 * root.unit; radiusY: 42 * root.unit
                useLargeArc: true
            }
        }
    }

    // The check, when the file has arrived.
    Shape {
        anchors.fill: parent
        visible: root.done
        preferredRendererType: Shape.CurveRenderer
        antialiasing: true
        ShapePath {
            strokeColor: Theme.markOn
            strokeWidth: 9 * root.unit
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            startX: 37 * root.unit; startY: 58 * root.unit
            PathLine { x: 50 * root.unit; y: 71 * root.unit }
            PathLine { x: 74 * root.unit; y: 45 * root.unit }
        }
    }

    // A quarter turn and its rest, as in mark.dart's MarkMotion.
    component Quarter: NumberAnimation {
        target: root; property: "angle"
        duration: 600
        easing.type: Easing.BezierSpline
        easing.bezierCurve: [0.65, 0, 0.35, 1, 1, 1]
    }
    SequentialAnimation {
        running: root.moving
        loops: Animation.Infinite
        Quarter { from: 0; to: 90 }
        PauseAnimation { duration: 200 }
        Quarter { from: 90; to: 180 }
        PauseAnimation { duration: 200 }
        Quarter { from: 180; to: 270 }
        PauseAnimation { duration: 200 }
        Quarter { from: 270; to: 360 }
        PauseAnimation { duration: 200 }
        onStopped: root.angle = 0
    }

    onMovingChanged: if (!moving) angle = 0
}
