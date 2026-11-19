// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Window activation tokens (#178) and X11 startup notification (#184).
#pragma once

#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>

#include "rust/cxx.h"

#include <QtCore/QByteArray>
#include <QtCore/QList>
#include <QtCore/QString>
#include <QtCore/QtGlobal>
#include <QtGui/QGuiApplication>
#include <QtGui/QWindow>

#if QT_CONFIG(xcb)
#include <xcb/xcb.h>
#endif

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

#if QT_CONFIG(xcb)
inline xcb_connection_t *x11_connection()
{
    auto *app = qGuiApp;
    if (!app || QGuiApplication::platformName() != QLatin1String("xcb"))
        return nullptr;
    auto *x11 = app->nativeInterface<QNativeInterface::QX11Application>();
    xcb_connection_t *conn = x11 ? x11->connection() : nullptr;
    return (conn && !xcb_connection_has_error(conn)) ? conn : nullptr;
}

template <std::size_t N>
inline xcb_intern_atom_cookie_t x11_intern(xcb_connection_t *conn, const char (&name)[N])
{
    static_assert(N > 1 && N - 1 <= 0xffff);
    return xcb_intern_atom(conn, 0, static_cast<std::uint16_t>(N - 1), name);
}

inline xcb_atom_t x11_atom_reply(xcb_connection_t *conn, xcb_intern_atom_cookie_t cookie)
{
    xcb_intern_atom_reply_t *reply = xcb_intern_atom_reply(conn, cookie, nullptr);
    if (!reply)
        return XCB_ATOM_NONE;
    const xcb_atom_t atom = reply->atom;
    std::free(reply);
    return atom;
}

inline QList<xcb_window_t> x11_top_windows(bool include_hidden = false)
{
    QList<xcb_window_t> out;
    const QWindowList windows = QGuiApplication::topLevelWindows();
    for (QWindow *w : windows) {
        if (!w || w->handle() == nullptr || (!include_hidden && !w->isVisible())
            || w->transientParent() != nullptr || w->type() != Qt::Window) {
            continue;
        }
        const auto wid = static_cast<xcb_window_t>(w->winId());
        if (wid != XCB_WINDOW_NONE)
            out.append(wid);
    }
    return out;
}

inline xcb_window_t x11_root_window(xcb_connection_t *conn, const QList<xcb_window_t> &windows)
{
    if (!windows.isEmpty()) {
        const xcb_query_tree_cookie_t tree_cookie = xcb_query_tree(conn, windows.first());
        if (xcb_query_tree_reply_t *tree = xcb_query_tree_reply(conn, tree_cookie, nullptr)) {
            const xcb_window_t root = tree->root;
            std::free(tree);
            if (root != XCB_WINDOW_NONE)
                return root;
        }
    }
    const xcb_setup_t *setup = xcb_get_setup(conn);
    if (!setup)
        return XCB_WINDOW_NONE;
    const xcb_screen_iterator_t iter = xcb_setup_roots_iterator(setup);
    if (iter.rem > 0 && iter.data)
        return iter.data->root;
    return XCB_WINDOW_NONE;
}

// EWMH: when Qt's xcb plugin sets `_NET_WM_USER_TIME_WINDOW` on a top-level
// window, window managers (KWin X11, Mutter Xorg) read `_NET_WM_USER_TIME`
// from that helper window instead of the outer window.
inline void x11_set_user_time_on_windows(xcb_connection_t *conn,
                                         const QList<xcb_window_t> &windows,
                                         xcb_atom_t user_time_atom,
                                         xcb_atom_t user_time_win_atom,
                                         std::uint32_t timestamp)
{
    if (timestamp == 0 || user_time_atom == XCB_ATOM_NONE)
        return;
    for (const xcb_window_t wid : windows) {
        xcb_window_t target = wid;
        if (user_time_win_atom != XCB_ATOM_NONE) {
            const xcb_get_property_cookie_t prop_cookie =
                xcb_get_property(conn, 0, wid, user_time_win_atom, XCB_ATOM_WINDOW, 0, 1);
            if (xcb_get_property_reply_t *prop =
                    xcb_get_property_reply(conn, prop_cookie, nullptr)) {
                if (prop->type == XCB_ATOM_WINDOW && prop->format == 32
                    && xcb_get_property_value_length(prop)
                        >= static_cast<int>(sizeof(xcb_window_t))) {
                    xcb_window_t helper = XCB_WINDOW_NONE;
                    std::memcpy(&helper, xcb_get_property_value(prop), sizeof(helper));
                    if (helper != XCB_WINDOW_NONE)
                        target = helper;
                }
                std::free(prop);
            }
        }
        xcb_change_property(conn, XCB_PROP_MODE_REPLACE, target, user_time_atom,
                            XCB_ATOM_CARDINAL, 32, 1, &timestamp);
    }
}
#endif

// Update `_NET_WM_USER_TIME` before QML unminimizes and raises the window,
// using Qt 6's public `QNativeInterface::QX11Application` and `libxcb` rather
// than private QPA headers (`qplatformnativeinterface.h`, #184).
inline void prepare_x11_activation(std::uint32_t timestamp)
{
#if QT_CONFIG(xcb)
    if (timestamp == 0)
        return;
    xcb_connection_t *conn = x11_connection();
    if (!conn)
        return;
    // Include hidden windows that already own a native handle so
    // `_NET_WM_USER_TIME` is in place before `Main.qml:raiseWindow()` calls
    // `showNormal()` (#184).
    const QList<xcb_window_t> windows = x11_top_windows(true);
    if (windows.isEmpty())
        return;
    const xcb_intern_atom_cookie_t ut_cookie = x11_intern(conn, "_NET_WM_USER_TIME");
    const xcb_intern_atom_cookie_t utw_cookie = x11_intern(conn, "_NET_WM_USER_TIME_WINDOW");
    const xcb_atom_t user_time_atom = x11_atom_reply(conn, ut_cookie);
    const xcb_atom_t user_time_win_atom = x11_atom_reply(conn, utw_cookie);
    x11_set_user_time_on_windows(conn, windows, user_time_atom, user_time_win_atom, timestamp);
    xcb_flush(conn);
#else
    Q_UNUSED(timestamp);
#endif
}

// After QML raises the window, send `_NET_ACTIVE_WINDOW` carrying the
// launcher's `_TIME` timestamp (Qt's own `requestActivate()` sends its last
// local input timestamp, which focus-stealing prevention rejects when stale)
// and broadcast the freedesktop Startup Notification `remove: ID="…"`
// 20-byte `ClientMessage` chunks so the desktop ends its busy cursor (#184).
inline void finish_x11_startup(std::uint32_t timestamp,
                               rust::Slice<const std::uint8_t> remove_chunks)
{
#if QT_CONFIG(xcb)
    const bool has_remove =
        remove_chunks.size() >= 20 && (remove_chunks.size() % 20) == 0;
    if (timestamp == 0 && !has_remove)
        return;
    xcb_connection_t *conn = x11_connection();
    if (!conn)
        return;
    const QList<xcb_window_t> windows = x11_top_windows();
    const xcb_window_t root = x11_root_window(conn, windows);
    if (root == XCB_WINDOW_NONE)
        return;

    const bool activate = timestamp != 0 && !windows.isEmpty();
    xcb_intern_atom_cookie_t ut_cookie{};
    xcb_intern_atom_cookie_t utw_cookie{};
    xcb_intern_atom_cookie_t active_cookie{};
    if (activate) {
        ut_cookie = x11_intern(conn, "_NET_WM_USER_TIME");
        utw_cookie = x11_intern(conn, "_NET_WM_USER_TIME_WINDOW");
        active_cookie = x11_intern(conn, "_NET_ACTIVE_WINDOW");
    }
    xcb_intern_atom_cookie_t begin_cookie{};
    xcb_intern_atom_cookie_t info_cookie{};
    if (has_remove) {
        begin_cookie = x11_intern(conn, "_NET_STARTUP_INFO_BEGIN");
        info_cookie = x11_intern(conn, "_NET_STARTUP_INFO");
    }

    if (activate) {
        const xcb_atom_t user_time_atom = x11_atom_reply(conn, ut_cookie);
        const xcb_atom_t user_time_win_atom = x11_atom_reply(conn, utw_cookie);
        const xcb_atom_t active_win_atom = x11_atom_reply(conn, active_cookie);
        x11_set_user_time_on_windows(conn, windows, user_time_atom, user_time_win_atom, timestamp);
        if (active_win_atom != XCB_ATOM_NONE) {
            for (const xcb_window_t wid : windows) {
                xcb_client_message_event_t ev{};
                ev.response_type = XCB_CLIENT_MESSAGE;
                ev.format = 32;
                ev.window = wid;
                ev.type = active_win_atom;
                ev.data.data32[0] = 1; // EWMH source indication: application
                ev.data.data32[1] = timestamp;
                xcb_send_event(
                    conn, 0, root,
                    XCB_EVENT_MASK_SUBSTRUCTURE_REDIRECT | XCB_EVENT_MASK_SUBSTRUCTURE_NOTIFY,
                    reinterpret_cast<const char *>(&ev));
            }
        }
    }

    if (has_remove) {
        const xcb_atom_t begin_atom = x11_atom_reply(conn, begin_cookie);
        const xcb_atom_t info_atom = x11_atom_reply(conn, info_cookie);
        if (begin_atom != XCB_ATOM_NONE && info_atom != XCB_ATOM_NONE) {
            const xcb_window_t xid = xcb_generate_id(conn);
            if (xid != XCB_WINDOW_NONE && xid != static_cast<xcb_window_t>(-1)) {
                const std::uint32_t override_redirect = 1;
                xcb_create_window(conn, 0, xid, root, -100, -100, 1, 1, 0,
                                  XCB_WINDOW_CLASS_INPUT_ONLY, XCB_COPY_FROM_PARENT,
                                  XCB_CW_OVERRIDE_REDIRECT, &override_redirect);
                const std::uint8_t *bytes = remove_chunks.data();
                const std::size_t total = remove_chunks.size();
                for (std::size_t offset = 0; offset < total; offset += 20) {
                    xcb_client_message_event_t ev{};
                    ev.response_type = XCB_CLIENT_MESSAGE;
                    ev.format = 8;
                    ev.window = xid;
                    ev.type = (offset == 0) ? begin_atom : info_atom;
                    std::memcpy(ev.data.data8, bytes + offset, 20);
                    xcb_send_event(conn, 0, root, XCB_EVENT_MASK_PROPERTY_CHANGE,
                                   reinterpret_cast<const char *>(&ev));
                }
                xcb_destroy_window(conn, xid);
            }
        }
    }

    xcb_flush(conn);
#else
    Q_UNUSED(timestamp);
    Q_UNUSED(remove_chunks);
#endif
}

} // namespace totaldrop
