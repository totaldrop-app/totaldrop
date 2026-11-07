#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Check that a built package carries the file-manager menus (#303) and
# only suggests the Nautilus Python bindings, never requires them (#319).
#
#   packaging/check-package-contents.sh dist/packages/totaldrop-….deb|.rpm|.pkg.tar.zst …
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
  # Every real package has hard dependencies (Qt at least): none read means
  # the fields were not read, and the check below would pass on nothing.
  if ! grep -Eqi "^($hard):" "$deps"; then
    echo "no $hard field was read from the package" >&2; bad=1
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
  # Only absence is checked below, so first make sure this is TotalDrop's
  # Flatpak, not an empty or wrong folder that would pass.
  [ -x "$2/bin/totaldrop-desktop" ] || { echo "$2 is not TotalDrop's Flatpak files/ folder" >&2; exit 2; }
  (cd "$2" && find . -type f -o -type l) > "$tmp/files"
  : > "$tmp/deps"
  check flatpak "$tmp/files" "$tmp/deps"; exit ;;
esac

# run COMMAND: COMMAND in the image build.sh built (never a same-named one
# from a registry), with the package as "$1", never pasted into its text.
run() {
  docker run --rm --pull=never \
    --mount "type=bind,src=$(dirname "$pkg"),dst=/pkg,readonly" \
    "$image" sh -ec "$1" sh "/pkg/$name"
}

[ $# -ge 1 ] || { echo "one or more .deb, .rpm or .pkg.tar.zst packages" >&2; exit 2; }
failed=0
for arg in "$@"; do
  [ -f "$arg" ] || { echo "not a file: $arg" >&2; exit 2; }
  pkg=$(realpath "$arg")
  name=$(basename "$pkg")
  # The commands are single-quoted on purpose: "$1" is the container's.
  # shellcheck disable=SC2016
  case $name in
  *.deb) format=deb image=totaldrop-build-deb
    list='dpkg-deb --fsys-tarfile "$1" | tar -t'
    # Prints "Field: value" for each field present, nothing for the rest.
    fields='dpkg-deb -f "$1" Depends Pre-Depends Recommends Suggests | sed "s/^Pre-Depends:/Depends:/"' ;;
  *.rpm) format=rpm image=totaldrop-build-rpm
    list='rpm -qlp "$1"'
    fields='r=$(rpm -qp --requires "$1"); m=$(rpm -qp --recommends "$1"); s=$(rpm -qp --suggests "$1")
      printf "%s\n" "$r" | sed "s/^/depends: /"
      [ -z "$m" ] || printf "%s\n" "$m" | sed "s/^/recommends: /"
      [ -z "$s" ] || printf "%s\n" "$s" | sed "s/^/suggests: /"' ;;
  *.pkg.tar.zst) format=arch image=totaldrop-build-arch
    list='bsdtar -tf "$1"'
    # .PKGINFO says "depend = …" and "optdepend = …: why".
    fields='bsdtar -xOf "$1" .PKGINFO | sed -n "s/^\(depend\|optdepend\) = /\1: /p"' ;;
  *) echo "unknown package type: $name" >&2; exit 2 ;;
  esac
  run "$list" > "$tmp/files"
  run "$fields" > "$tmp/deps"
  echo "--- $name"
  check "$format" "$tmp/files" "$tmp/deps" || failed=1
done
exit "$failed"
