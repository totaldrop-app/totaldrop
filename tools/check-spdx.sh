#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Every hand-written source file states its copyright and license in its
# first lines (LICENSING.md). REUSE.toml gives every other file a license,
# so `reuse lint` alone would not notice a source file without them.
# Generated files and Cargokit are listed in REUSE.toml instead.
set -eu
cd "$(dirname "$0")/.."
missing=$(git ls-files \
    '*.rs' '*.ts' '*.js' '*.mjs' '*.dart' '*.qml' '*.h' '*.cpp' \
    '*.kt' '*.kts' '*.gradle' '*.sh' '*.py' '*.css' |
  grep -vE '^app/lib/l10n/|^app/lib/src/rust/|frb_generated|^app/rust_builder/cargokit/' |
  # REUSE-IgnoreStart (the patterns below are not this file's tags)
  while IFS= read -r f; do
    head -n 5 "$f" | grep -q 'SPDX-FileCopyrightText:' &&
      head -n 5 "$f" | grep -q 'SPDX-License-Identifier:' ||
      echo "$f"
  done)
# REUSE-IgnoreEnd
if [ -n "$missing" ]; then
  echo "These files have no SPDX lines at the top (LICENSING.md):"
  echo "$missing"
  exit 1
fi
echo "SPDX lines ok"
