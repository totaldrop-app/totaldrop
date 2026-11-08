// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

pragma Singleton
import QtQuick

// The roles the components under test read from Theme, with fixed values.
// The real Theme reads the desktop through SystemTheme, a Rust object the
// test runner does not have.
QtObject {
    readonly property color background: "#ffffff"
    readonly property color raised: "#f0f0f0"
    readonly property color ink: "#202020"
    readonly property color muted: "#707070"
    readonly property color accent: "#3daee9"
    readonly property color accentText: "#ffffff"
    readonly property int radius: 8
    readonly property int s8: 8
    readonly property int s16: 16
    readonly property int s24: 24
    readonly property int s40: 40
}
