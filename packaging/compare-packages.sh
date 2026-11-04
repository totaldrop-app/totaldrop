#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Compares the packages of two builds of the same commit (#252).
#
#   packaging/compare-packages.sh FIRST_DIR SECOND_DIR
#
# Identical packages pass. Otherwise the files inside the packages are
# compared one by one, and every difference fails, with one exception: the
# desktop binary may differ only in the order cxx-qt links the Qt libraries
# (#256). For it, the read-only data (which holds the Qt resources), the set
# of linked libraries and the size of the code must still be the same; the
# difference is then reported as a warning. The package metadata records the
# binary's checksum, so it is not compared once the binary differs.
set -eu

a=$1 b=$2
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

(cd "$a" && sha256sum -- *) > "$tmp/first.sums"
(cd "$b" && sha256sum -- *) > "$tmp/second.sums"
cat "$tmp/first.sums"
if cmp -s "$tmp/first.sums" "$tmp/second.sums"; then
  echo "identical"
  exit 0
fi

names() { sed 's/^[0-9a-f]*  //' "$1"; }
if [ "$(names "$tmp/first.sums")" != "$(names "$tmp/second.sums")" ]; then
  echo "::error::the two builds made different packages"
  diff "$tmp/first.sums" "$tmp/second.sums" || true
  exit 1
fi

extract() { # package, folder
  mkdir -p "$2"
  case $1 in
  *.deb) dpkg-deb -x "$1" "$2" ;;
  *.rpm | *.pkg.tar.zst) bsdtar -xf "$1" -C "$2" ;;
  *) echo "::error::unknown package type: $1"; exit 1 ;;
  esac
  # Arch's own metadata (.PKGINFO, .BUILDINFO, .MTREE) records checksums.
  rm -f "$2/.PKGINFO" "$2/.BUILDINFO" "$2/.MTREE"
}

section() { # binary, section, output
  objcopy -O binary --only-section="$2" "$1" "$3"
}

needed() {
  readelf -dW "$1" | sed -n 's/.*(NEEDED).*\[\(.*\)\]/\1/p' | sort
}

text_size() {
  readelf -SW "$1" | awk '$2 == ".text" { print $6 }'
}

status=0
for pkg in $(names "$tmp/first.sums"); do
  cmp -s "$a/$pkg" "$b/$pkg" && continue
  extract "$a/$pkg" "$tmp/1/$pkg"
  extract "$b/$pkg" "$tmp/2/$pkg"
  (cd "$tmp/1/$pkg" && find . | sort) > "$tmp/files1"
  (cd "$tmp/2/$pkg" && find . | sort) > "$tmp/files2"
  if ! diff "$tmp/files1" "$tmp/files2"; then
    echo "::error::$pkg: the two builds hold different files"
    status=1
    continue
  fi
  while read -r f; do
    one=$tmp/1/$pkg/$f two=$tmp/2/$pkg/$f
    [ -f "$one" ] && [ ! -L "$one" ] || continue
    cmp -s "$one" "$two" && continue
    case $f in
    */usr/bin/totaldrop-desktop)
      section "$one" .rodata "$tmp/ro1"
      section "$two" .rodata "$tmp/ro2"
      if cmp -s "$tmp/ro1" "$tmp/ro2" &&
        [ "$(needed "$one")" = "$(needed "$two")" ] &&
        [ "$(text_size "$one")" = "$(text_size "$two")" ]; then
        echo "::warning::$pkg: $f differs only in the Qt link order (#256)"
      else
        echo "::error::$pkg: $f differs beyond the Qt link order (#256)"
        status=1
      fi ;;
    *)
      echo "::error::$pkg: $f differs"
      status=1 ;;
    esac
  done < "$tmp/files1"
done
exit $status
