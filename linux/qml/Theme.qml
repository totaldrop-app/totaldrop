pragma Singleton
import QtQuick
import app.totaldrop

// The colour roles the screens use, resolved from the desktop.
// With source "system" (Plasma, qt6ct, hyprqt6engine) they are the Qt
// palette itself; otherwise they come from SystemTheme (Omarchy theme, or
// portal colour scheme and accent over Aurora).
QtObject {
    id: t

    readonly property SystemPalette sys: SystemPalette { colorGroup: SystemPalette.Active }
    readonly property string source: SystemTheme.source
    readonly property bool system: source === "system"

    function luma(c) { return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b }

    readonly property color background: system ? sys.window : SystemTheme.background
    readonly property color raised: system ? sys.button : SystemTheme.raised
    readonly property color ink: system ? sys.windowText : SystemTheme.ink
    readonly property color muted: system ? sys.placeholderText : SystemTheme.muted
    readonly property color rule: system ? Qt.tint(sys.window, Qt.alpha(sys.windowText, 0.16)) : SystemTheme.rule
    readonly property color accent: system ? (sys.accent !== undefined ? sys.accent : sys.highlight) : SystemTheme.accent
    readonly property color accentText: system ? sys.highlightedText : SystemTheme.accentText
    readonly property color success: SystemTheme.success
    readonly property color error: SystemTheme.error
    readonly property bool dark: system ? luma(sys.window) < 0.5 : SystemTheme.dark

    readonly property int radius: SystemTheme.radius
    readonly property bool reducedMotion: SystemTheme.reducedMotion

    // A freedesktop icon name in the form that takes the text colour:
    // `-symbolic` for GTK themes; under Plasma, Breeze's own name if it
    // differs (`kdeName`).
    readonly property bool breeze: SystemTheme.iconSuffix === ""
    function icon(name, kdeName) {
        if (breeze) return kdeName || name
        return name + SystemTheme.iconSuffix
    }

    // Spacing scale of the design system.
    readonly property int s8: 8
    readonly property int s16: 16
    readonly property int s24: 24
    readonly property int s40: 40

    // The widest a page's content grows; a wider window centres it.
    readonly property int contentWidth: 560
    // Width of a page's content in a window `w` wide, with the side margins.
    function columnWidth(w) { return Math.min(w - 2 * s24, contentWidth) }

    // Behind a modal dialog. Plasma's control style dims (and fades) on
    // its own; Fusion's shade is too faint to see, so it gets this one.
    readonly property bool fusion: SystemTheme.controlStyle === "Fusion"
    readonly property Component dim: Component { Rectangle { color: Qt.rgba(0, 0, 0, t.dark ? 0.5 : 0.3) } }

    // The mark keeps its own colours in every theme.
    readonly property color mark: dark ? "#A68BFF" : "#6D4AE0"
    // The check drawn on the mark.
    readonly property color markOn: dark ? "#22184F" : "#F4F2FB"
}
