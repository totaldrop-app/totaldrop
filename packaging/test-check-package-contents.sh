#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Tests for check-package-contents.sh: the check on package listings and
# dependency fields written the way each distribution's tools print them.
#
#   packaging/test-check-package-contents.sh
set -eu
here=$(cd "$(dirname "$0")" && pwd)
check=$here/check-package-contents.sh
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
failed=0

# expect pass|fail NAME FORMAT FILES DEPS
expect() {
  want=$1 name=$2
  shift 2
  printf '%s\n' "$2" > "$tmp/files"
  printf '%s\n' "$3" > "$tmp/deps"
  if sh "$check" --check "$1" "$tmp/files" "$tmp/deps" > "$tmp/out" 2>&1; then got=pass; else got=fail; fi
  if [ "$got" = "$want" ]; then
    echo "ok   $name"
  else
    echo "FAIL $name: wanted $want, got $got"; sed 's/^/     /' "$tmp/out"; failed=1
  fi
}

# As dpkg-deb --fsys-tarfile | tar -t, rpm -qlp and bsdtar -tf print them.
deb_files='./
./usr/bin/totaldrop
./usr/share/kio/servicemenus/app.totaldrop.TotalDrop.send.desktop
./usr/share/nemo/actions/app.totaldrop.TotalDrop.nemo_action
./usr/share/nautilus-python/extensions/totaldrop_nautilus.py'
rpm_files=$(printf '%s\n' "$deb_files" | sed -n 's,^\./usr,/usr,p')
arch_files='.BUILDINFO
.MTREE
.PKGINFO
usr/
usr/bin/totaldrop
usr/share/kio/servicemenus/app.totaldrop.TotalDrop.send.desktop
usr/share/nemo/actions/app.totaldrop.TotalDrop.nemo_action
usr/share/nautilus-python/extensions/totaldrop_nautilus.py'

deb_deps='Depends: libc6 (>= 2.38), qt6-base-dev (>= 6.8) | libqt6core6t64, hicolor-icon-theme
Recommends: xdg-desktop-portal-gtk | xdg-desktop-portal-kde, fonts-noto-core
Suggests: fonts-noto-cjk, qml6-module-org-kde-desktop, python3-nautilus'
rpm_deps='depends: libc.so.6()(64bit)
depends: qt6-qtbase >= 6.8
recommends: (kf6-qqc2-desktop-style if plasma-workspace)
recommends: (nautilus-python if nautilus)
suggests: google-noto-sans-cjk-fonts'
arch_deps='depend: qt6-base
depend: qt6-declarative
optdepend: qqc2-desktop-style: Breeze controls under KDE Plasma
optdepend: python-nautilus: "Send with TotalDrop" in the Nautilus menu'

# Each format as it is built today.
expect pass "deb as built" deb "$deb_files" "$deb_deps"
expect pass "rpm as built" rpm "$rpm_files" "$rpm_deps"
expect pass "arch as built" arch "$arch_files" "$arch_deps"

# A menu dropped from the package.
for m in kio/servicemenus/app.totaldrop.TotalDrop.send.desktop \
  nemo/actions/app.totaldrop.TotalDrop.nemo_action \
  nautilus-python/extensions/totaldrop_nautilus.py; do
  expect fail "deb without $m" deb "$(printf '%s\n' "$deb_files" | grep -vF "$m")" "$deb_deps"
  expect fail "rpm without $m" rpm "$(printf '%s\n' "$rpm_files" | grep -vF "$m")" "$rpm_deps"
  expect fail "arch without $m" arch "$(printf '%s\n' "$arch_files" | grep -vF "$m")" "$arch_deps"
done
# A menu only in a folder of the same name somewhere else.
expect fail "menu in the wrong folder" rpm \
  "$(printf '%s\n' "$rpm_files" | sed 's,/usr/share/nemo/,/usr/lib/nemo/,')" "$rpm_deps"

# The bindings no longer suggested.
expect fail "deb without the suggestion" deb "$deb_files" \
  "$(printf '%s\n' "$deb_deps" | sed 's/, python3-nautilus//')"
expect fail "rpm without the suggestion" rpm "$rpm_files" \
  "$(printf '%s\n' "$rpm_deps" | grep -v nautilus-python)"
expect fail "arch without the suggestion" arch "$arch_files" \
  "$(printf '%s\n' "$arch_deps" | grep -v python-nautilus)"
# A longer name that only starts with the bindings' name is not them.
expect fail "deb suggests only a longer name" deb "$deb_files" \
  "$(printf '%s\n' "$deb_deps" | sed 's/python3-nautilus/python3-nautilus-extra/')"
expect fail "rpm recommends only a longer name" rpm "$rpm_files" \
  "$(printf '%s\n' "$rpm_deps" | sed 's/(nautilus-python if/(nautilus-python3 if/')"

# The bindings required: Dolphin and Nemo users would have to install
# Nautilus's Python bindings.
expect fail "deb depends on the bindings" deb "$deb_files" \
  "$(printf '%s\n' "$deb_deps" | sed 's/^Depends: /Depends: python3-nautilus, /')"
expect fail "rpm requires the bindings" rpm "$rpm_files" \
  "$(printf '%s\ndepends: nautilus-python\n' "$rpm_deps")"
expect fail "arch depends on the bindings" arch "$arch_files" \
  "$(printf '%s\ndepend: python-nautilus\n' "$arch_deps")"

# No hard dependency read: the fields were not read at all, and the
# "required" check would pass on nothing.
expect fail "deb with no Depends read" deb "$deb_files" \
  "$(printf '%s\n' "$deb_deps" | grep -v '^Depends:')"
expect fail "rpm with no requires read" rpm "$rpm_files" \
  "$(printf '%s\n' "$rpm_deps" | grep -v '^depends:')"
expect fail "arch with no depend read" arch "$arch_files" \
  "$(printf '%s\n' "$arch_deps" | grep -v '^depend:')"

# The Flatpak: its files/ folder is /app, and the menus must not be in it.
flatpak_files='./bin/totaldrop
./share/applications/app.totaldrop.TotalDrop.desktop
./share/metainfo/app.totaldrop.TotalDrop.metainfo.xml'
expect pass "flatpak as built" flatpak "$flatpak_files" ""
expect fail "flatpak with a service menu" flatpak \
  "$(printf '%s\n./share/kio/servicemenus/app.totaldrop.TotalDrop.send.desktop\n' "$flatpak_files")" ""
expect fail "flatpak with the Nautilus extension" flatpak \
  "$(printf '%s\n./share/nautilus-python/extensions/totaldrop_nautilus.py\n' "$flatpak_files")" ""

# --flatpak reads a real folder, and only TotalDrop's: an empty or wrong
# folder has no menus either, and must not pass.
mkdir -p "$tmp/app/bin" "$tmp/app/share/nemo/actions"
if sh "$check" --flatpak "$tmp/app" > /dev/null 2>&1; then
  echo "FAIL --flatpak on a folder without TotalDrop passed"; failed=1
else
  echo "ok   --flatpak on a folder without TotalDrop"
fi
: > "$tmp/app/bin/totaldrop-desktop"
chmod +x "$tmp/app/bin/totaldrop-desktop"
if sh "$check" --flatpak "$tmp/app" > /dev/null 2>&1; then
  echo "ok   --flatpak without menus"
else
  echo "FAIL --flatpak without menus"; failed=1
fi
: > "$tmp/app/share/nemo/actions/app.totaldrop.TotalDrop.nemo_action"
if sh "$check" --flatpak "$tmp/app" > /dev/null 2>&1; then
  echo "FAIL --flatpak with the Nemo action passed"; failed=1
else
  echo "ok   --flatpak with the Nemo action"
fi

# Wrong use stops with an explanation, not a pass.
if sh "$check" --check deb "$tmp/missing" "$tmp/missing" > /dev/null 2>&1; then
  echo "FAIL missing listing passed"; failed=1
else
  echo "ok   missing listing"
fi
if sh "$check" > /dev/null 2>&1; then
  echo "FAIL no argument passed"; failed=1
else
  echo "ok   no argument"
fi

exit "$failed"
