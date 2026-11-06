// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Qt's own interface text in the app's language (#307).
#pragma once

#include <QtCore/QCoreApplication>
#include <QtCore/QHash>
#include <QtCore/QString>
#include <QtCore/QTranslator>

namespace totaldrop {

// Text that Qt or the desktop's Qt Quick style draws, not the app: the
// right-click menu of text fields (Qt 6.9+ Fusion, KDE's org.kde.desktop)
// marks its items with qsTr. Qt's own translation files have them in a few
// languages only, so this translator answers them from l10n/app instead,
// in the language the app chose. Anything it does not know falls through
// to the next translator, and then to Qt's English.
class QtTexts final : public QTranslator
{
public:
    using QTranslator::QTranslator;

    void add(const QString &context, const QString &source, const QString &text)
    {
        m_texts.insert(key(context, source), text);
    }

    QString translate(const char *context, const char *source, const char *disambiguation,
                      int n) const override
    {
        Q_UNUSED(disambiguation);
        Q_UNUSED(n);
        if (!context || !source)
            return {};
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

inline QtTexts *qt_texts()
{
    // Owned by the application object: deleted with it.
    static QtTexts *texts = new QtTexts(QCoreApplication::instance());
    return texts;
}

// Call after the application object exists and before QML loads.
inline void add_qt_text(const QString &context, const QString &source, const QString &text)
{
    qt_texts()->add(context, source, text);
}

inline void install_qt_texts()
{
    QCoreApplication::installTranslator(qt_texts());
}

} // namespace totaldrop
