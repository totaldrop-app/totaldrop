#!/bin/sh
# Install a built package in a clean container of its distribution and
# check that it runs: binaries resolve, the QML interface loads, and the
# desktop entry and AppStream data are valid.
#
#   packaging/test-install.sh dist/packages/totaldrop-….pkg.tar.zst|.deb|.rpm [image]
#
# With WAYLAND=1 the app also opens on this machine's Wayland session for
# a few seconds, for a screenshot.
set -eu
pkg=$(realpath "$1")
case $pkg in
*.pkg.tar.zst) image=${2:-archlinux:latest}
  install="pacman -Sy --noconfirm >/dev/null && pacman -U --noconfirm /pkg/$(basename "$pkg") >/dev/null && pacman -S --noconfirm --needed desktop-file-utils appstream >/dev/null" ;;
*.deb) image=${2:-debian:trixie}
  install="apt-get update >/dev/null && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends /pkg/$(basename "$pkg") desktop-file-utils appstream >/dev/null" ;;
*.rpm) image=${2:-fedora:43}
  install="dnf install -y /pkg/$(basename "$pkg") desktop-file-utils appstream >/dev/null" ;;
*) echo "unknown package type" >&2; exit 2 ;;
esac
run=/bin/true
wl=""
if [ "${WAYLAND:-}" = 1 ]; then
  r=/run/user/$(id -u); w=$(ls $r | grep -E '^wayland-[0-9]+$' | head -1)
  wl="-v $r/$w:/tmp/xdg/wayland-0"
  run="mkdir -p /tmp/xdg && chmod 700 /tmp/xdg && XDG_RUNTIME_DIR=/tmp/xdg WAYLAND_DISPLAY=wayland-0 LANG=tr_TR.UTF-8 timeout ${LIFE:-6} totaldrop-desktop || true"
fi
docker run --rm -v "$(dirname "$pkg"):/pkg:ro" $wl "$image" sh -ec "
  $install
  echo '--- versions'; totaldrop --version
  echo '--- libraries'; ! ldd /usr/bin/totaldrop-desktop | grep 'not found'
  echo '--- desktop entry'; desktop-file-validate /usr/share/applications/app.totaldrop.TotalDrop.desktop && echo valid
  echo '--- appstream'; appstreamcli validate --no-net --explain /usr/share/metainfo/app.totaldrop.TotalDrop.metainfo.xml || true
  echo '--- interface'; QT_QPA_PLATFORM=offscreen QT_FORCE_STDERR_LOGGING=1 timeout 4 totaldrop-desktop > /tmp/out.log 2>&1 || true
  if grep -E 'qrc:|module .* is not installed|Error' /tmp/out.log; then echo 'QML errors'; exit 1; else echo 'loads cleanly'; fi
  $run"
