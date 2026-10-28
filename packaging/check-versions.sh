#!/bin/sh
# Check that every place naming the Linux app's version names the same one:
# linux/Cargo.toml, the Arch PKGBUILD, the RPM spec, the Debian changelog
# and the newest release in the AppStream metainfo. With a tag (linux-v0.1.0-beta.5), the tag must match too.
#
#   packaging/check-versions.sh [linux-vX.Y.Z[-pre]]
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)

cargo=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/linux/Cargo.toml" | head -1)
pkgbuild=$(sed -n "s/^_ver=\(.*\)$/\1/p" "$root/packaging/arch/PKGBUILD" | head -1)
# RPM and Debian write a pre-release as 0.1.0~beta.4; Debian adds -<revision>.
spec=$(sed -n 's/^Version:[[:space:]]*\(.*\)$/\1/p' "$root/packaging/rpm/totaldrop.spec" | head -1 | tr '~' '-')
debian=$(sed -n '1s/^[^(]*(\([^)]*\)).*/\1/p' "$root/packaging/debian/changelog" | sed 's/-[0-9][0-9]*$//' | tr '~' '-')
metainfo=$(sed -n 's/.*<release version="\([^"]*\)".*/\1/p' "$root/packaging/linux/app.totaldrop.TotalDrop.metainfo.xml" | head -1 | tr '~' '-')

status=0
check() {
  if [ "$2" != "$cargo" ]; then
    echo "version mismatch: $1 says '$2', linux/Cargo.toml says '$cargo'" >&2
    status=1
  fi
}
[ -n "$cargo" ] || { echo "no version in linux/Cargo.toml" >&2; exit 1; }
check packaging/arch/PKGBUILD "$pkgbuild"
check packaging/rpm/totaldrop.spec "$spec"
check packaging/debian/changelog "$debian"
check packaging/linux/app.totaldrop.TotalDrop.metainfo.xml "$metainfo"
if [ $# -ge 1 ]; then
  check "tag $1" "${1#linux-v}"
fi
[ $status -eq 0 ] && echo "versions agree: $cargo"
exit $status
