// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

pragma Singleton
import QtQuick

// Stand-in for the Rust SystemTheme singleton (linux/src/bridge/theme.rs),
// read directly by PrimaryButton.qml.
QtObject {
    readonly property string source: "system"
    readonly property bool dark: false
    readonly property color background: "#ffffff"
    readonly property color raised: "#f0f0f0"
    readonly property color ink: "#202020"
    readonly property color muted: "#707070"
    readonly property color rule: "#d0d0d0"
    readonly property color accent: "#3daee9"
    readonly property color accentText: "#ffffff"
    readonly property color success: "#2e7d32"
    readonly property color error: "#c62828"
    readonly property int radius: 8
    readonly property bool reducedMotion: true
    readonly property string iconSuffix: ""
    readonly property string controlStyle: ""
}
