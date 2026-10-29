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

# The mark in its 112-unit box; $1 = link colour, $2 = node and drop colour.
mark() {
  printf '<path fill="none" stroke="%s" stroke-width="7" d="M56 24A60 60 0 0 1 88.91 81A60 60 0 0 1 23.09 81A60 60 0 0 1 56 24Z"/>' "$1"
  printf '<g fill="%s"><circle cx="56" cy="24" r="11"/><circle cx="88.91" cy="81" r="11"/><circle cx="23.09" cy="81" r="11"/>' "$2"
  printf '<path d="M67 90.83L59.84 85.59A6.5 6.5 0 1 0 59.84 96.07Z"/></g>'
}

# Adaptive icon layers: a 108-unit canvas, the mark inside the 66-unit safe zone.
layer() {
  printf '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 108 108"><g transform="translate(23.2 23.2) scale(0.55)">%s</g></svg>' "$(mark "$1" "$2")"
}
layer '#22184F' '#6D4AE0' > "$tmp/foreground.svg"
layer '#FFFFFF' '#FFFFFF' > "$tmp/monochrome.svg"

# Legacy icon for launchers without adaptive icons: the mark on its paper square.
printf '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 112 112"><rect width="112" height="112" rx="22" fill="#F4F2FB"/><g transform="translate(21.28 21.28) scale(0.62)">%s</g></svg>' \
  "$(mark '#22184F' '#6D4AE0')" > "$tmp/legacy.svg"

for d in mdpi:1 hdpi:1.5 xhdpi:2 xxhdpi:3 xxxhdpi:4; do
  name=${d%%:*}; k=${d#*:}
  layer=$(awk "BEGIN { print int(108 * $k) }")
  icon=$(awk "BEGIN { print int(48 * $k) }")
  rsvg-convert -w "$layer" -h "$layer" "$tmp/foreground.svg" -o "$res/mipmap-$name/ic_launcher_foreground.png"
  rsvg-convert -w "$layer" -h "$layer" "$tmp/monochrome.svg" -o "$res/mipmap-$name/ic_launcher_monochrome.png"
  rsvg-convert -w "$icon" -h "$icon" "$tmp/legacy.svg" -o "$res/mipmap-$name/ic_launcher.png"
done
echo "icons written to $res"
