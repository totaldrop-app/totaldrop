// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

pragma Singleton
import QtQuick

// The roles the components under test read from Theme, with fixed values.
// The real Theme reads the desktop through SystemTheme, a Rust object the
// test runner does not have.
QtObject {
    readonly property string source: "system"
    readonly property bool system: true
    readonly property color background: "#ffffff"
    readonly property color raised: "#f0f0f0"
    readonly property color ink: "#202020"
    readonly property color muted: "#707070"
    readonly property color rule: "#d0d0d0"
    readonly property color accent: "#3daee9"
    readonly property color accentText: "#ffffff"
    readonly property color success: "#2e7d32"
    readonly property color error: "#c62828"
    readonly property bool dark: false
    readonly property int radius: 8
    readonly property int controlRadius: 6
    readonly property bool reducedMotion: true
    readonly property bool breeze: true
    // Empty icon name avoids missing-icon-theme lookups under QT_QPA_PLATFORM=offscreen.
    function icon(name, kdeName) { return "" }
    readonly property int s8: 8
    readonly property int s16: 16
    readonly property int s24: 24
    readonly property int s40: 40
    readonly property int contentWidth: 560
    function columnWidth(w) { return Math.min(w - 2 * s24, contentWidth) }
    readonly property bool fusion: false
    readonly property Component dim: Component { Rectangle { color: "#4d000000" } }
    readonly property color mark: "#6D4AE0"
    readonly property color markOn: "#F4F2FB"
}
