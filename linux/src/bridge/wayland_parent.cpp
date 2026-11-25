// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Wayland xdg-foreign-v2 parent export and xdg-activation-v1 fresh token
// acquisition for GlobalShortcuts portal dialogs (#631).

#include "wayland_parent.h"

#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <string>
#include <unordered_map>
#include <wayland-util.h>

#include <QtCore/QByteArray>
#include <QtCore/QMetaObject>
#include <QtCore/QObject>
#include <QtCore/QThread>
#include <QtCore/QtGlobal>
#include <QtGui/QGuiApplication>
#include <QtGui/QWindow>
#include <QtGui/qguiapplication_platform.h>

#if QT_VERSION >= QT_VERSION_CHECK(6, 5, 0)
#if QT_CONFIG(wayland)
#define TOTALDROP_HAS_QT_WAYLAND_NATIVE 1
#endif
#endif
#ifndef TOTALDROP_HAS_QT_WAYLAND_NATIVE
#define TOTALDROP_HAS_QT_WAYLAND_NATIVE 0
#endif

#if TOTALDROP_HAS_QT_WAYLAND_NATIVE
#include <qpa/qplatformnativeinterface.h>
#endif

extern "C" {
#include <wayland-client.h>
#include "xdg-activation-v1-client-protocol.h"
#include "xdg-foreign-unstable-v2-client-protocol.h"
#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wattributes"
#endif
#include "xdg-activation-v1-protocol-code.h"
#undef WL_PRIVATE
#include "xdg-foreign-unstable-v2-protocol-code.h"
#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic pop
#endif
}

namespace totaldrop {
namespace {

constexpr std::size_t kMaxCompositorStringBytes = 1024;
constexpr std::size_t kMaxActiveExports = 16;

struct ActiveExport {
    struct zxdg_exported_v2 *exported = nullptr;
    struct wl_display *display = nullptr;
    QMetaObject::Connection destroyed_conn;
};

std::unordered_map<std::uint64_t, ActiveExport> &active_exports()
{
    static std::unordered_map<std::uint64_t, ActiveExport> map;
    return map;
}

std::uint64_t next_export_id()
{
    static std::uint64_t counter = 0;
    ++counter;
    if (counter == 0)
        ++counter;
    return counter;
}

void destroy_export_entry(ActiveExport &entry)
{
    QObject::disconnect(entry.destroyed_conn);
    if (entry.exported) {
        zxdg_exported_v2_destroy(entry.exported);
        entry.exported = nullptr;
    }
    if (entry.display) {
        wl_display_flush(entry.display);
        entry.display = nullptr;
    }
}

void evict_oldest_if_full()
{
    auto &map = active_exports();
    while (map.size() >= kMaxActiveExports) {
        auto oldest = map.begin();
        for (auto it = map.begin(); it != map.end(); ++it) {
            if (it->first < oldest->first)
                oldest = it;
        }
        destroy_export_entry(oldest->second);
        map.erase(oldest);
    }
}

std::string bounded_cstr(const char *raw)
{
    if (!raw)
        return {};
    const std::size_t len = ::strnlen(raw, kMaxCompositorStringBytes + 1);
    if (len == 0 || len > kMaxCompositorStringBytes)
        return {};
    return std::string(raw, len);
}

struct RegistryGlobals {
    struct zxdg_exporter_v2 *exporter = nullptr;
    struct xdg_activation_v1 *activation = nullptr;
};

void on_registry_global(
    void *data,
    struct wl_registry *registry,
    std::uint32_t name,
    const char *interface,
    std::uint32_t version)
{
    if (!data || !interface || version < 1)
        return;
    auto *globals = static_cast<RegistryGlobals *>(data);
    if (!globals->exporter
        && std::strcmp(interface, zxdg_exporter_v2_interface.name) == 0) {
        globals->exporter = static_cast<struct zxdg_exporter_v2 *>(
            wl_registry_bind(registry, name, &zxdg_exporter_v2_interface, 1));
    } else if (!globals->activation
        && std::strcmp(interface, xdg_activation_v1_interface.name) == 0) {
        globals->activation = static_cast<struct xdg_activation_v1 *>(
            wl_registry_bind(registry, name, &xdg_activation_v1_interface, 1));
    }
}

void on_registry_global_remove(void *, struct wl_registry *, std::uint32_t) {}

const struct wl_registry_listener kRegistryListener = {
    on_registry_global,
    on_registry_global_remove,
};

struct ExportCapture {
    std::string handle;
};

void on_exported_handle(
    void *data,
    struct zxdg_exported_v2 *,
    const char *handle)
{
    if (!data)
        return;
    auto *capture = static_cast<ExportCapture *>(data);
    if (capture->handle.empty())
        capture->handle = bounded_cstr(handle);
}

const struct zxdg_exported_v2_listener kExportedListener = {
    on_exported_handle,
};

struct TokenCapture {
    std::string token;
};

void on_activation_token_done(
    void *data,
    struct xdg_activation_token_v1 *,
    const char *token)
{
    if (!data)
        return;
    auto *capture = static_cast<TokenCapture *>(data);
    if (capture->token.empty())
        capture->token = bounded_cstr(token);
}

const struct xdg_activation_token_v1_listener kActivationTokenListener = {
    on_activation_token_done,
};

// Runs a private-queue Wayland exchange on `display` so Qt's own Wayland event
// queue is never dispatched re-entrantly while inside a QML/Qt slot.
std::uint64_t acquire_on_wayland_display(
    QWindow *window,
    struct wl_display *display,
    struct wl_surface *surface,
    struct wl_seat *seat,
    std::uint32_t serial,
    QString &handle_out,
    QString &token_out)
{
    if (!display)
        return 0;

    struct wl_event_queue *queue = wl_display_create_queue(display);
    if (!queue)
        return 0;

    auto *display_wrapper = static_cast<struct wl_display *>(wl_proxy_create_wrapper(display));
    if (!display_wrapper) {
        wl_event_queue_destroy(queue);
        return 0;
    }
    wl_proxy_set_queue(reinterpret_cast<struct wl_proxy *>(display_wrapper), queue);

    struct wl_registry *registry = wl_display_get_registry(display_wrapper);
    if (!registry) {
        wl_proxy_wrapper_destroy(display_wrapper);
        wl_event_queue_destroy(queue);
        return 0;
    }

    RegistryGlobals globals;
    wl_registry_add_listener(registry, &kRegistryListener, &globals);

    std::uint64_t export_id = 0;
    struct zxdg_exported_v2 *exported = nullptr;
    struct xdg_activation_token_v1 *token_obj = nullptr;
    ExportCapture export_capture;
    TokenCapture token_capture;

    if (wl_display_roundtrip_queue(display, queue) >= 0) {
        if (globals.exporter && surface) {
            exported = zxdg_exporter_v2_export_toplevel(globals.exporter, surface);
            if (exported)
                zxdg_exported_v2_add_listener(exported, &kExportedListener, &export_capture);
        }
        if (globals.activation) {
            token_obj = xdg_activation_v1_get_activation_token(globals.activation);
            if (token_obj) {
                xdg_activation_token_v1_add_listener(
                    token_obj, &kActivationTokenListener, &token_capture);
                if (seat && serial != 0)
                    xdg_activation_token_v1_set_serial(token_obj, serial, seat);
                if (surface)
                    xdg_activation_token_v1_set_surface(token_obj, surface);
                xdg_activation_token_v1_commit(token_obj);
            }
        }

        if ((exported || token_obj) && wl_display_roundtrip_queue(display, queue) >= 0) {
            if (!token_capture.token.empty())
                token_out = QString::fromUtf8(
                    token_capture.token.data(),
                    static_cast<qsizetype>(token_capture.token.size()));

            if (exported && !export_capture.handle.empty()) {
                // Detach listener user_data and event queue before our local
                // capture struct and temporary queue go out of scope.
                wl_proxy_set_user_data(reinterpret_cast<struct wl_proxy *>(exported), nullptr);
                wl_proxy_set_queue(reinterpret_cast<struct wl_proxy *>(exported), nullptr);

                evict_oldest_if_full();
                export_id = next_export_id();
                ActiveExport entry;
                entry.exported = exported;
                entry.display = display;
                if (window) {
                    entry.destroyed_conn = QObject::connect(
                        window, &QObject::destroyed, [export_id]() {
                            release_wayland_parent(export_id);
                        });
                }
                active_exports().emplace(export_id, std::move(entry));
                exported = nullptr;
                handle_out = QString::fromUtf8(
                    export_capture.handle.data(),
                    static_cast<qsizetype>(export_capture.handle.size()));
            }
        }
    }

    if (exported)
        zxdg_exported_v2_destroy(exported);
    if (token_obj)
        xdg_activation_token_v1_destroy(token_obj);
    if (globals.activation)
        xdg_activation_v1_destroy(globals.activation);
    if (globals.exporter)
        zxdg_exporter_v2_destroy(globals.exporter);
    wl_registry_destroy(registry);
    wl_proxy_wrapper_destroy(display_wrapper);
    wl_event_queue_destroy(queue);
    wl_display_flush(display);
    return export_id;
}

#if TOTALDROP_HAS_QT_WAYLAND_NATIVE
bool is_exportable_toplevel(const QWindow *window)
{
    return window
        && window->isTopLevel()
        && !window->transientParent()
        && (window->flags() & Qt::WindowType_Mask) == Qt::Window
        && window->isVisible()
        && window->isExposed();
}

QWindow *focused_toplevel_window()
{
    QWindow *window = QGuiApplication::focusWindow();
    while (window && window->transientParent())
        window = window->transientParent();
    if (is_exportable_toplevel(window))
        return window;
    for (QWindow *top : QGuiApplication::topLevelWindows()) {
        if (is_exportable_toplevel(top))
            return top;
    }
    return nullptr;
}

struct wl_surface *wayland_surface_for(QWindow *window)
{
    if (!window)
        return nullptr;
    // Query QPlatformNativeInterface::nativeResourceForWindow("surface") on
    // every Qt 6.5+ version rather than assuming QWaylandWindow::winId()
    // returns a wl_surface*. Checking QPlatformNativeInterface::staticMetaObject
    // also records an explicit Qt_6_PRIVATE_API ELF reference so dpkg-shlibdeps
    // (Debian qt6-base-private-abi) and RPM find-requires automatically pin the
    // Qt private ABI across upgrades.
    if (QPlatformNativeInterface *pni = QGuiApplication::platformNativeInterface()) {
        if (QPlatformNativeInterface::staticMetaObject.cast(pni)) {
            if (void *surface = pni->nativeResourceForWindow(QByteArrayLiteral("surface"), window))
                return static_cast<struct wl_surface *>(surface);
        }
    }
    return nullptr;
}
#endif

} // namespace

std::uint64_t acquire_wayland_dialog_context(QString &handle_out, QString &token_out)
{
    handle_out.clear();
    token_out.clear();
    if (!qGuiApp || QThread::currentThread() != qGuiApp->thread())
        return 0;
    if (!QGuiApplication::platformName().startsWith(QLatin1String("wayland")))
        return 0;

#if TOTALDROP_HAS_QT_WAYLAND_NATIVE
    auto *wl = qGuiApp->nativeInterface<QNativeInterface::QWaylandApplication>();
    if (!wl)
        return 0;
    struct wl_display *display = wl->display();
    if (!display)
        return 0;
    QWindow *window = focused_toplevel_window();
    struct wl_surface *surface = wayland_surface_for(window);
    struct wl_seat *seat = wl->lastInputSeat();
    if (!seat)
        seat = wl->seat();
    const std::uint32_t serial = wl->lastInputSerial();
    return acquire_on_wayland_display(
        window, display, surface, seat, serial, handle_out, token_out);
#else
    (void)&acquire_on_wayland_display;
    return 0;
#endif
}

void release_wayland_parent(std::uint64_t export_id)
{
    if (export_id == 0 || !qGuiApp || QThread::currentThread() != qGuiApp->thread())
        return;
    auto &map = active_exports();
    auto it = map.find(export_id);
    if (it == map.end())
        return;
    ActiveExport entry = std::move(it->second);
    map.erase(it);
    destroy_export_entry(entry);
}

} // namespace totaldrop
