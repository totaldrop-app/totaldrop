// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Qt's own interface text in the app's language (#307).
#pragma once

#include <QtCore/QCoreApplication>
#include <QtCore/QHash>
#include <QtCore/QString>
#include <QtCore/QTranslator>
#include <QtCore/QtGlobal>

namespace totaldrop {

// Text that Qt or the desktop's Qt Quick style draws, not the app, and
// marks with qsTr: the right-click menu of text fields (Qt 6.9.0 and 6.11+
// Quick Controls; KDE's org.kde.desktop style) and Qt's layout direction.
// Qt's own translation files carry the menu in a few languages only, so
// this translator answers it from l10n/app, in the language the app chose.
// A translator installed later answers first: on KDE, the style installs
// its own catalog when QML loads, and its text (in the system's language)
// wins where it has one. Anything this translator does not know falls
// through to Qt's English.
class QtTexts final : public QTranslator
{
public:
    using QTranslator::QTranslator;

    void add(const QString &context, const QString &source, const QString &text)
    {
        // An empty text would blank the item: leave it to Qt's English.
        if (!text.isEmpty())
            m_texts.insert(key(context, source), text);
    }

    QString translate(const char *context, const char *source, const char *disambiguation,
                      int n) const override
    {
        Q_UNUSED(disambiguation);
        Q_UNUSED(n);
        if (!context || !source)
            return {};
        // A null string, not an empty one, tells Qt there is no answer.
        return m_texts.value(key(QString::fromUtf8(context), QString::fromUtf8(source)));
    }

    bool isEmpty() const override { return m_texts.isEmpty(); }

private:
    static QString key(const QString &context, const QString &source)
    {
        return context + QChar(0x1f) + source;
    }

    QHash<QString, QString> m_texts;
};

// The translator, owned by the application object. Null before it exists.
inline QtTexts *qt_texts()
{
    static QtTexts *texts = nullptr;
    QCoreApplication *app = QCoreApplication::instance();
    Q_ASSERT(app);
    if (!texts && app)
        texts = new QtTexts(app);
    return texts;
}

// Call after the application object exists and before QML loads.
inline void add_qt_text(const QString &context, const QString &source, const QString &text)
{
    if (QtTexts *texts = qt_texts())
        texts->add(context, source, text);
}

inline bool install_qt_texts()
{
    QtTexts *texts = qt_texts();
    return texts && QCoreApplication::installTranslator(texts);
}

// What Qt shows for `source` in `context` with every translator installed.
inline QString qt_translate(const QString &context, const QString &source)
{
    return QCoreApplication::translate(context.toUtf8().constData(),
                                       source.toUtf8().constData());
}

} // namespace totaldrop
