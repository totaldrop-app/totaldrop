#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Check that a built package carries the file-manager menus (#303) and
# only suggests the Nautilus Python bindings, never requires them (#319).
#
#   packaging/check-package-contents.sh dist/packages/totaldrop-….deb|.rpm|.pkg.tar.zst
#   packaging/check-package-contents.sh --flatpak DIR
#   packaging/check-package-contents.sh --check deb|rpm|arch|flatpak FILES DEPS
#
# The first form reads the package with its own distribution's tools, in
# the image packaging/build.sh built it in (totaldrop-build-…), so it runs
# right after build.sh on any machine with Docker. --flatpak checks an
# installed or built Flatpak's files/ folder: the sandbox cannot add to
# another program's menu, so the menus must not be there. --check is the
# check itself on a list of files (one path per line) and the package's
# dependency fields, for packaging/test-check-package-contents.sh.
set -eu

menus="usr/share/kio/servicemenus/app.totaldrop.TotalDrop.send.desktop
usr/share/nemo/actions/app.totaldrop.TotalDrop.nemo_action
usr/share/nautilus-python/extensions/totaldrop_nautilus.py"

# check FORMAT FILES DEPS: FILES lists the package's paths, DEPS holds its
# dependency fields, one "field: value" per line, field names lowercase:
# depends, recommends, suggests (deb, rpm) or depend, optdepend (arch).
check() {
  format=$1 files=$2 deps=$3 bad=0
  # Paths without a leading "/" or "./", so every format reads the same.
  paths=$(sed -e 's,^\./,,' -e 's,^/,,' "$files")
  for m in $menus; do
    if [ "$format" = flatpak ]; then
      # A Flatpak's files/ folder is its /app; the menus go nowhere in it.
      rel=${m#usr/}
      if printf '%s\n' "$paths" | grep -qxF "$rel"; then
        echo "the Flatpak must not carry /app/$rel" >&2; bad=1
      fi
    elif ! printf '%s\n' "$paths" | grep -qxF "$m"; then
      echo "missing /$m" >&2; bad=1
    fi
  done
  case $format in
  deb) bindings=python3-nautilus; weak='recommends|suggests'; hard=depends ;;
  rpm) bindings=nautilus-python; weak='recommends|suggests'; hard=depends ;;
  arch) bindings=python-nautilus; weak=optdepend; hard=depend ;;
  flatpak) [ "$bad" = 0 ] && echo "no file-manager menus, as expected"; return "$bad" ;;
  *) echo "unknown format $format" >&2; return 2 ;;
  esac
  # The package name as a whole word: python3-nautilus must not match
  # python3-nautilus-extra, nor nautilus-python match nautilus-python3.
  word="(^|[^A-Za-z0-9_.+-])$bindings([^A-Za-z0-9_.+-]|\$)"
  if ! grep -Ei "^($weak):" "$deps" | grep -Eq "$word"; then
    echo "$bindings is not suggested ($(echo "$weak" | tr '|' '/'))" >&2; bad=1
  fi
  if grep -Ei "^($hard):" "$deps" | grep -Eq "$word"; then
    echo "$bindings is required ($hard); it must only be suggested" >&2; bad=1
  fi
  [ "$bad" = 0 ] && echo "file-manager menus installed, $bindings suggested"
  return "$bad"
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

case ${1:-} in
--check)
  [ $# = 4 ] || { echo "--check FORMAT FILES DEPS" >&2; exit 2; }
  check "$2" "$3" "$4"; exit ;;
--flatpak)
  [ $# = 2 ] && [ -d "$2" ] || { echo "--flatpak DIR (a Flatpak's files/ folder)" >&2; exit 2; }
  (cd "$2" && find . -type f -o -type l) > "$tmp/files"
  : > "$tmp/deps"
  check flatpak "$tmp/files" "$tmp/deps"; exit ;;
esac

[ $# = 1 ] && [ -f "$1" ] || { echo "a .deb, .rpm or .pkg.tar.zst package" >&2; exit 2; }
pkg=$(realpath "$1")
name=$(basename "$pkg")
case $name in
*.deb) format=deb
  list="dpkg-deb --fsys-tarfile /pkg/$name | tar -t"
  # Recommends and Suggests may be missing; dpkg-deb then prints nothing.
  fields="for f in Depends Pre-Depends Recommends Suggests; do v=\$(dpkg-deb -f /pkg/$name \$f); [ -z \"\$v\" ] || echo \"\$f: \$v\"; done | sed 's/^Pre-Depends:/Depends:/'" ;;
*.rpm) format=rpm
  list="rpm -qlp /pkg/$name"
  fields="rpm -qp --requires /pkg/$name | sed 's/^/depends: /'
    rpm -qp --recommends /pkg/$name | sed 's/^/recommends: /'
    rpm -qp --suggests /pkg/$name | sed 's/^/suggests: /'" ;;
*.pkg.tar.zst) format=arch
  list="bsdtar -tf /pkg/$name"
  # .PKGINFO says "depend = …" and "optdepend = …: why".
  fields="bsdtar -xOf /pkg/$name .PKGINFO | sed -n 's/^\(depend\|optdepend\) = /\1: /p'" ;;
*) echo "unknown package type: $name" >&2; exit 2 ;;
esac
case $format in deb) image=totaldrop-build-deb ;; rpm) image=totaldrop-build-rpm ;; arch) image=totaldrop-build-arch ;; esac
docker run --rm -v "$(dirname "$pkg"):/pkg:ro" "$image" sh -ec "$list" > "$tmp/files"
docker run --rm -v "$(dirname "$pkg"):/pkg:ro" "$image" sh -ec "$fields" > "$tmp/deps"
echo "--- $name"
check "$format" "$tmp/files" "$tmp/deps"
