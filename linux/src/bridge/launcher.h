// The activation token a later launch hands the window (#178).
#pragma once

#include <QtCore/QByteArray>
#include <QtCore/QString>
#include <QtCore/QtGlobal>

namespace totaldrop {

// Qt's Wayland plugin reads XDG_ACTIVATION_TOKEN when a window asks to be
// activated, and then unsets it: with the token of the launch that asked,
// the compositor lets the window come to the front. Set through qputenv,
// which takes Qt's environment lock, on the GUI thread just before the
// window asks.
inline void set_activation_token(const QString &token)
{
    qputenv("XDG_ACTIVATION_TOKEN", token.toUtf8());
}

} // namespace totaldrop
