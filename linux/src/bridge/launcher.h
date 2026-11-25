// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The activation token a later launch hands the window (#178), and opening
// a confirmed web link in the desktop's browser (#54).
#pragma once

#include <cstdint>

#include <QtCore/QByteArray>
#include <QtCore/QString>
#include <QtCore/QUrl>
#include <QtCore/QtGlobal>
#include <QtGui/QDesktopServices>
#include <QtGui/QGuiApplication>
#include <QtGui/QWindow>

namespace totaldrop {

// Qt's Wayland plugin reads XDG_ACTIVATION_TOKEN when a window asks to be
// activated, and then unsets it: with the token of the launch that asked,
// the compositor lets the window come to the front. Set through qputenv,
// which takes Qt's environment lock, on the GUI thread just before the
// window asks. Only on Wayland: elsewhere nothing takes it back out, and
// programs started later would inherit it.
inline void set_activation_token(const QString &token)
{
    if (QGuiApplication::platformName().startsWith(QLatin1String("wayland")))
        qputenv("XDG_ACTIVATION_TOKEN", token.toUtf8());
}

// X11 window ID of the focused (or first visible) TotalDrop window, or 0 on
// Wayland / headless, so GlobalShortcuts portal dialogs can parent to it
// (#54). Wayland xdg-foreign-v2 parenting and fresh xdg-activation-v1 tokens
// live in `wayland_parent.h` (#631).
inline std::uint64_t parent_x11_xid()
{
    if (QGuiApplication::platformName() != QLatin1String("xcb"))
        return 0;
    QWindow *window = QGuiApplication::focusWindow();
    if (!window) {
        for (QWindow *top : QGuiApplication::topLevelWindows()) {
            if (top && top->isVisible()) {
                window = top;
                break;
            }
        }
    }
    return window ? static_cast<std::uint64_t>(window->winId()) : 0;
}

// Checks with QUrl::StrictMode (RFC 3986) that `url` is a valid http(s)
// web address with a non-empty host and no userinfo (#54).
inline bool valid_strict_web_url(const QString &url)
{
    const QUrl qurl(url, QUrl::StrictMode);
    return qurl.isValid()
        && (qurl.scheme() == QStringLiteral("https") || qurl.scheme() == QStringLiteral("http"))
        && !qurl.host().isEmpty()
        && qurl.userInfo().isEmpty();
}

// Opens a single http(s) URL that Rust (`rfc3986_web_url`) has already
// validated and RFC 3986-encoded (#54).
inline bool open_web_url_cpp(const QString &url)
{
    if (!valid_strict_web_url(url))
        return false;
    return QDesktopServices::openUrl(QUrl(url, QUrl::StrictMode));
}

} // namespace totaldrop
