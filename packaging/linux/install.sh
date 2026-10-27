#!/bin/sh
# Install TotalDrop for Linux into a package root. Shared by the Arch,
# Debian and RPM recipes so every package ships the same files.
#
#   packaging/linux/install.sh DESTDIR [PREFIX] [TARGET_DIR]
#
# Expects `cargo build --release --locked -p totaldrop-linux -p totaldrop-cli`
# to have run.
set -eu
dest=$1
prefix=${2:-/usr}
target=${3:-target/release}
here=$(dirname "$0")
id=app.totaldrop.TotalDrop

install -Dm755 "$target/totaldrop-desktop" "$dest$prefix/bin/totaldrop-desktop"
install -Dm755 "$target/totaldrop" "$dest$prefix/bin/totaldrop"
install -Dm644 "$here/$id.desktop" "$dest$prefix/share/applications/$id.desktop"
install -Dm644 "$here/$id.metainfo.xml" "$dest$prefix/share/metainfo/$id.metainfo.xml"
install -Dm644 "$here/$id.svg" "$dest$prefix/share/icons/hicolor/scalable/apps/$id.svg"
install -Dm644 "$here/$id-symbolic.svg" "$dest$prefix/share/icons/hicolor/symbolic/apps/$id-symbolic.svg"
