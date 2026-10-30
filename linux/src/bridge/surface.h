// The surface format Qt Quick asks the windowing system for.
#pragma once

#include <QtGui/QSurfaceFormat>

namespace totaldrop {

// Ask for at least 8 bits per colour channel. Left unset, Qt takes the
// first EGL config the driver lists, and Mesa on some Intel GPUs lists
// RGB565 first: every colour is then rounded to 5 or 6 bits, so the
// desktop's #1c1c1e is drawn as #181c21 (#170). Alpha is left as Qt sets
// it, so the window stays opaque to the compositor.
inline void use_true_colour()
{
    QSurfaceFormat format = QSurfaceFormat::defaultFormat();
    format.setRedBufferSize(8);
    format.setGreenBufferSize(8);
    format.setBlueBufferSize(8);
    QSurfaceFormat::setDefaultFormat(format);
}

} // namespace totaldrop
