pragma Singleton
import QtQuick
import app.totaldrop

// Interface text in the desktop's language and region. The strings live in
// l10n/app, shared with the Android app; Texts (Rust, totaldrop-l10n)
// chooses the language, plurals and number formats (ADR 0008).
QtObject {
    // Every plain string by key: Strings.s.sendTitle.
    readonly property var s: JSON.parse(Texts.strings)
    // Right-to-left language: Main.qml mirrors the layout.
    readonly property bool rtl: Texts.rtl
    // Some text in this language is an unreviewed draft translation.
    readonly property bool draft: Texts.draft

    // A message with arguments: numbers are counts (they choose the plural
    // and are written in the region's digits), strings are inserted as text.
    function f(key, args) {
        return Texts.format(key, JSON.stringify(args || {}))
    }

    // "1,5 MB" in the region's format.
    function size(bytes) {
        return Texts.bytes(bytes)
    }

    // A name from the other side, with hidden direction characters shown.
    function name(text) {
        return Texts.reveal(text)
    }
}
