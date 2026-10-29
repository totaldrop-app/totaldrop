import QtQuick
import QtQuick.Shapes
import app.totaldrop

// The Ağ mark: three devices, each joined to the other two, and a drop on
// the bottom link. At rest it matches the SVG. While `sending`, it plays the
// website's animation exactly (docs/tasarim.md): a 4.8 s loop in which the
// drop goes to the right device, the top one, the left one and back to its
// place, each leg on cubic-bezier(0.65, 0, 0.35, 1), resting briefly at
// every device. The links and devices never move.
Item {
    id: root

    property real size: 96
    property bool sending: false

    implicitWidth: size
    implicitHeight: size
    width: size
    height: size

    Accessible.role: Accessible.Graphic
    Accessible.name: "TotalDrop"

    readonly property real unit: size / 112
    readonly property bool moving: sending && !Theme.reducedMotion && visible

    // 0..1 along the motion path; 0 is the resting place.
    property real along: 0

    Shape {
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer
        antialiasing: true

        ShapePath {
            strokeColor: Theme.markWave
            strokeWidth: 7 * root.unit
            fillColor: "transparent"
            startX: 56 * root.unit; startY: 24 * root.unit
            PathArc { x: 88.91 * root.unit; y: 81 * root.unit; radiusX: 60 * root.unit; radiusY: 60 * root.unit }
            PathArc { x: 23.09 * root.unit; y: 81 * root.unit; radiusX: 60 * root.unit; radiusY: 60 * root.unit }
            PathArc { x: 56 * root.unit; y: 24 * root.unit; radiusX: 60 * root.unit; radiusY: 60 * root.unit }
        }
    }

    // The devices.
    Repeater {
        model: [[56, 24], [88.91, 81], [23.09, 81]]
        Rectangle {
            required property var modelData
            x: (modelData[0] - 11) * root.unit
            y: (modelData[1] - 11) * root.unit
            width: 22 * root.unit
            height: width
            radius: width / 2
            color: Theme.markFile
            antialiasing: true
        }
    }

    // The drop's way round, counter-clockwise from the middle of the bottom
    // link: right device, top, left, and back.
    PathInterpolator {
        id: way
        progress: root.along
        path: Path {
            startX: 56 * root.unit; startY: 90.83 * root.unit
            PathArc { x: 88.91 * root.unit; y: 81 * root.unit; radiusX: 60 * root.unit; radiusY: 60 * root.unit; direction: PathArc.Counterclockwise }
            PathArc { x: 56 * root.unit; y: 24 * root.unit; radiusX: 60 * root.unit; radiusY: 60 * root.unit; direction: PathArc.Counterclockwise }
            PathArc { x: 23.09 * root.unit; y: 81 * root.unit; radiusX: 60 * root.unit; radiusY: 60 * root.unit; direction: PathArc.Counterclockwise }
            PathArc { x: 56 * root.unit; y: 90.83 * root.unit; radiusX: 60 * root.unit; radiusY: 60 * root.unit; direction: PathArc.Counterclockwise }
        }
    }

    // The drop, body centred on the path and pointing the way it goes.
    Item {
        x: way.x
        y: way.y
        rotation: way.angle
        Shape {
            preferredRendererType: Shape.CurveRenderer
            antialiasing: true
            ShapePath {
                fillColor: Theme.markFile
                strokeColor: "transparent"
                startX: 11 * root.unit; startY: 0
                PathLine { x: 3.84 * root.unit; y: -5.24 * root.unit }
                PathArc {
                    x: 3.84 * root.unit; y: 5.24 * root.unit
                    radiusX: 6.5 * root.unit; radiusY: 6.5 * root.unit
                    useLargeArc: true; direction: PathArc.Counterclockwise
                }
            }
        }
    }

    // Legs and rests of the 4.8 s loop, as in mark.dart's MarkMotion.legs.
    component Leg: NumberAnimation {
        target: root; property: "along"
        easing.type: Easing.BezierSpline
        easing.bezierCurve: [0.65, 0, 0.35, 1, 1, 1]
    }
    SequentialAnimation {
        running: root.moving
        loops: Animation.Infinite
        Leg { from: 0; to: 1 / 6; duration: 600 }
        PauseAnimation { duration: 360 }
        Leg { from: 1 / 6; to: 0.5; duration: 1200 }
        PauseAnimation { duration: 360 }
        Leg { from: 0.5; to: 5 / 6; duration: 1200 }
        PauseAnimation { duration: 360 }
        Leg { from: 5 / 6; to: 1; duration: 600 }
        PauseAnimation { duration: 120 }
        onStopped: root.along = 0
    }

    onMovingChanged: if (!moving) along = 0
}
