#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Runs the QML tests in this folder (tst_*.qml) with qmltestrunner,
# without a display. The components under test import app.totaldrop, which
# the app builds with Rust (linux/build.rs). Here a stand-in module (stub/)
# gives them fixed Theme/Strings/Transfer/Launcher stubs, and this script
# copies the components into it.
#
#   linux/tests/qml/run.sh
#   QMLTESTRUNNER=/path/to/qmltestrunner linux/tests/qml/run.sh
set -eu
here=$(cd "$(dirname "$0")" && pwd)
runner=${QMLTESTRUNNER:-}
if [ -z "$runner" ]; then
  for candidate in /usr/lib/qt6/bin/qmltestrunner qmltestrunner /usr/lib/x86_64-linux-gnu/qt6/bin/qmltestrunner /usr/lib/aarch64-linux-gnu/qt6/bin/qmltestrunner; do
    if command -v "$candidate" >/dev/null 2>&1; then
      runner=$candidate
      break
    fi
  done
fi
if [ -z "$runner" ]; then
  echo "qmltestrunner not found (install qt6-declarative-dev-tools or qt6-declarative)" >&2
  exit 1
fi
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mod="$tmp/app/totaldrop"
mkdir -p "$mod"
cp "$here/stub/qmldir" \
   "$here/stub/SystemTheme.qml" \
   "$here/stub/Theme.qml" \
   "$here/stub/Strings.qml" \
   "$here/stub/Launcher.qml" \
   "$here/stub/Transfer.qml" \
   "$mod/"
for f in ActionCard Divider Field FocusScroller IconButton Mark PageHeader \
         PrimaryButton QrCode ReceivePage ResultPanel SecondaryButton \
         SendPage SettingsPage StageHeading Toggle TransferProgress \
         VerificationWords; do
  cp "$here/../../qml/$f.qml" "$mod/"
done
# The platform theme and Controls style decide which keys press a button and
# how popups transition; pinning them keeps local runs identical to CI.
QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME= QT_QUICK_CONTROLS_STYLE=Basic \
  QT_QUICK_BACKEND=software "$runner" -import "$tmp" -input "$here"
