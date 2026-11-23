// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Wayland xdg-foreign-v2 parent export and xdg-activation-v1 fresh token
// acquisition for GlobalShortcuts portal dialogs (#631).
#pragma once

#include <cstdint>

#include <QtCore/QString>

namespace totaldrop {

// Must be called on Qt's GUI thread. On Wayland (Qt 6.5+), exports the
// focused visible toplevel window via zxdg_exporter_v2 (writing the handle to
// `handle_out` and returning a non-zero export ID) and requests a fresh
// xdg_activation_v1 token (writing it to `token_out`). Returns 0 when no
// Wayland handle was exported (for example on X11, headless, or compositors
// without zxdg_exporter_v2); `token_out` may still be populated independently.
std::uint64_t acquire_wayland_dialog_context(QString &handle_out, QString &token_out);

// Must be called on Qt's GUI thread. Destroys the zxdg_exported_v2 object
// identified by `export_id` if still active. Safe to call with 0 or an ID
// already released (for example when the window closed first).
void release_wayland_parent(std::uint64_t export_id);

} // namespace totaldrop
