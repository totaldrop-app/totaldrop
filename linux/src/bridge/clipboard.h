// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The clipboard's text, exactly. A QML TextEdit keeps text in a
// QTextDocument, which turns a no-break space into a space and \r\n,
// U+2028 and some noncharacters into \n; a text to send or a received
// text to copy goes through here instead (#326).
#pragma once

#include <QtCore/QString>
#include <QtGui/QClipboard>
#include <QtGui/QGuiApplication>

namespace totaldrop {

// The clipboard's text, or an empty string. On Wayland a window gets the
// clipboard only while it has the keyboard focus.
inline QString clipboard_text()
{
    return QGuiApplication::clipboard()->text(QClipboard::Clipboard);
}

inline void set_clipboard_text(const QString &text)
{
    QGuiApplication::clipboard()->setText(text, QClipboard::Clipboard);
}

} // namespace totaldrop
