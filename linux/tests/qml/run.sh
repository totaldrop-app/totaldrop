#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Runs the QML tests in this folder (tst_*.qml) with qmltestrunner,
# without a display. The components under test import app.totaldrop, which
# the app builds with Rust (linux/build.rs). Here a stand-in module (stub/)
# gives them a fixed Theme, and this script copies the components into it.
#
#   linux/tests/qml/run.sh
#   QMLTESTRUNNER=/path/to/qmltestrunner linux/tests/qml/run.sh
set -eu
here=$(cd "$(dirname "$0")" && pwd)
runner=${QMLTESTRUNNER:-/usr/lib/qt6/bin/qmltestrunner}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mod="$tmp/app/totaldrop"
mkdir -p "$mod"
cp "$here/stub/qmldir" \
   "$here/stub/SystemTheme.qml" \
   "$here/stub/Theme.qml" \
   "$here/stub/Strings.qml" \
   "$here/stub/Transfer.qml" \
   "$mod/"
for f in ActionCard Divider Field FocusScroller IconButton Mark PageHeader \
         PrimaryButton QrCode ReceivePage ResultPanel SecondaryButton \
         SendPage StageHeading Toggle TransferProgress VerificationWords; do
  cp "$here/../../qml/$f.qml" "$mod/"
done
# The platform theme decides which keys press a button; the caller's
# (QT_QPA_PLATFORMTHEME) would make the results differ from CI's.
QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME= QT_QUICK_BACKEND=software \
  "$runner" -import "$tmp" -input "$here"
