// The surface format Qt Quick asks the windowing system for.
#pragma once

#include <QtGui/QSurfaceFormat>

namespace totaldrop {

// Ask for at least 8 bits per colour channel. With the sizes unset, Qt
// asks EGL for the smallest config, which on Mesa can be RGB565 (seen on
// Intel Arc): every colour is then rounded to 5 or 6 bits, so the
// desktop's #1c1c1e is drawn as #181c21 (#170). On EGL this is a hard
// minimum; every desktop driver offers 8-bit configs. Alpha is left to
// Qt; asking for it here would give the window a translucent buffer.
inline void use_true_colour()
{
    QSurfaceFormat format = QSurfaceFormat::defaultFormat();
    format.setRedBufferSize(8);
    format.setGreenBufferSize(8);
    format.setBlueBufferSize(8);
    QSurfaceFormat::setDefaultFormat(format);
}

} // namespace totaldrop
