// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls
import app.totaldrop

// A tool button showing only its icon, with its text as the tooltip and
// accessible name. With Fusion the icon takes the text colour; Breeze
// icons already do.
ToolButton {
    id: b
    display: AbstractButton.IconOnly
    icon.color: !Theme.fusion ? "transparent" : enabled ? Theme.ink : Theme.muted
    ToolTip.visible: hovered
    ToolTip.text: text
    Keys.onReturnPressed: (e) => { if (!e.isAutoRepeat) click() }
    Keys.onEnterPressed: (e) => { if (!e.isAutoRepeat) click() }
}
