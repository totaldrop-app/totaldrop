import QtQuick
import QtQuick.Shapes
import app.totaldrop

// The Dalga mark. At rest it matches the SVG. While `sending`, it plays the
// website's animation exactly (styles.css, `travel` and `ripple`): a 3.2 s
// loop; the file goes from the drop to the end of the channel in the first
// 55 % on cubic-bezier(0.65, 0, 0.35, 1); a 2 px ring in the file colour
// grows from 1x to 4.6x the drop and fades from 0.7 to 0 in the first 70 %
// on cubic-bezier(0.2, 0.6, 0.3, 1). The waves never move.
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

    // 0..1 along the channel, and 0..1 through the ring's growth.
    property real travel: 1
    property real ripple: 1

    function half(r) { return Math.asin(9 / r) * 180 / Math.PI }

    Shape {
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer
        antialiasing: true

        ShapePath {
            strokeColor: Theme.markWave
            strokeWidth: 8 * root.unit
            fillColor: "transparent"
            capStyle: ShapePath.FlatCap
            PathAngleArc {
                centerX: 56 * root.unit; centerY: 56 * root.unit
                radiusX: 26 * root.unit; radiusY: 26 * root.unit
                startAngle: -45 + root.half(26)
                sweepAngle: 360 - 2 * root.half(26)
            }
        }
        ShapePath {
            strokeColor: Theme.markWave
            strokeWidth: 6 * root.unit
            fillColor: "transparent"
            capStyle: ShapePath.FlatCap
            PathAngleArc {
                centerX: 56 * root.unit; centerY: 56 * root.unit
                radiusX: 40 * root.unit; radiusY: 40 * root.unit
                startAngle: -45 + root.half(40)
                sweepAngle: 360 - 2 * root.half(40)
            }
        }
    }

    // The ring: 2 px whatever the size, like the web's non-scaling stroke.
    Rectangle {
        readonly property real r: 12 * root.unit * (1 + 3.6 * root.ripple)
        visible: root.moving && root.ripple < 1
        x: 56 * root.unit - r
        y: 56 * root.unit - r
        width: 2 * r
        height: 2 * r
        radius: r
        color: "transparent"
        border.width: 2
        border.color: Theme.markFile
        opacity: 0.7 * (1 - root.ripple)
        antialiasing: true
    }

    // The drop.
    Rectangle {
        x: (56 - 12) * root.unit
        y: (56 - 12) * root.unit
        width: 24 * root.unit
        height: width
        radius: width / 2
        color: Theme.markFile
        antialiasing: true
    }

    // The file, at distance 12..52 from the centre along the north-east axis.
    Rectangle {
        readonly property real d: 12 + 40 * root.travel
        x: (56 + d * Math.SQRT1_2 - 8) * root.unit
        y: (56 - d * Math.SQRT1_2 - 8) * root.unit
        width: 16 * root.unit
        height: width
        radius: width / 2
        color: Theme.markFile
        antialiasing: true
    }

    SequentialAnimation {
        id: loop
        running: root.moving
        // The web's 0.4 s delay before the first loop.
        PropertyAction { target: root; properties: "travel,ripple"; value: 0 }
        PauseAnimation { duration: 400 }
        SequentialAnimation {
            loops: Animation.Infinite
            PropertyAction { target: root; properties: "travel,ripple"; value: 0 }
            ParallelAnimation {
                NumberAnimation {
                    target: root; property: "travel"; from: 0; to: 1; duration: 1760
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: [0.65, 0, 0.35, 1, 1, 1]
                }
                NumberAnimation {
                    target: root; property: "ripple"; from: 0; to: 1; duration: 2240
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: [0.2, 0.6, 0.3, 1, 1, 1]
                }
            }
            PauseAnimation { duration: 960 }
        }
        onStopped: { root.travel = 1; root.ripple = 1 }
    }

    onMovingChanged: if (!moving) { travel = 1; ripple = 1 }
}
