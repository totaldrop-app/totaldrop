import QtQuick
import app.totaldrop

// A QR code from Transfer's module string. Always dark modules on a light
// tile with a four-module quiet zone, so every camera can read it.
Rectangle {
    id: qr

    property string modules
    property int count: 0
    property real side: 200

    width: side
    height: side
    radius: Theme.radius
    color: "#ffffff"
    Accessible.role: Accessible.Graphic
    Accessible.name: "QR"

    Canvas {
        id: canvas
        anchors.fill: parent
        renderStrategy: Canvas.Cooperative
        onPaint: {
            const ctx = getContext("2d")
            ctx.reset()
            const n = qr.count
            if (n <= 0 || qr.modules.length !== n * n) return
            const cell = Math.floor(width / (n + 8))
            const offset = Math.floor((width - cell * n) / 2)
            ctx.fillStyle = "#000000"
            for (let y = 0; y < n; y++)
                for (let x = 0; x < n; x++)
                    if (qr.modules.charCodeAt(y * n + x) === 49)
                        ctx.fillRect(offset + x * cell, offset + y * cell, cell, cell)
        }
    }
    onModulesChanged: canvas.requestPaint()
    onSideChanged: canvas.requestPaint()
}
