#!/bin/sh
# Render the Android launcher icons from the mark's geometry (docs/tasarim.md,
# "İşaret"). Run after changing the mark; commit the PNGs it writes.
#
#   packaging/render-icons.sh
#
# Needs rsvg-convert (librsvg).
set -eu
cd "$(dirname "$0")/.."
res=app/android/app/src/main/res
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# The Köşe mark in its 112-unit box; $1 = colour.
mark() {
  printf '<path fill="%s" d="M55 15H97V57A42 42 0 1 1 55 15Z"/>' "$1"
}

# Adaptive icon layers: a 108-unit canvas on the violet background
# (values/ic_launcher_background.xml). The corner is the point furthest from
# the centre (58 units), so at 0.55 it stays inside the 66-unit safe circle.
layer() {
  printf '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 108 108"><g transform="translate(23.2 23.2) scale(0.55)">%s</g></svg>' "$(mark "$1")"
}
layer '#F4F2FB' > "$tmp/foreground.svg"
layer '#FFFFFF' > "$tmp/monochrome.svg"

# Legacy icon for launchers without adaptive icons: the paper mark on violet.
printf '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 112 112"><rect width="112" height="112" rx="22" fill="#6D4AE0"/><g transform="translate(21.28 21.28) scale(0.62)">%s</g></svg>' \
  "$(mark '#F4F2FB')" > "$tmp/legacy.svg"

for d in mdpi:1 hdpi:1.5 xhdpi:2 xxhdpi:3 xxxhdpi:4; do
  name=${d%%:*}; k=${d#*:}
  layer=$(awk "BEGIN { print int(108 * $k) }")
  icon=$(awk "BEGIN { print int(48 * $k) }")
  rsvg-convert -w "$layer" -h "$layer" "$tmp/foreground.svg" -o "$res/mipmap-$name/ic_launcher_foreground.png"
  rsvg-convert -w "$layer" -h "$layer" "$tmp/monochrome.svg" -o "$res/mipmap-$name/ic_launcher_monochrome.png"
  rsvg-convert -w "$icon" -h "$icon" "$tmp/legacy.svg" -o "$res/mipmap-$name/ic_launcher.png"
done
echo "icons written to $res"
