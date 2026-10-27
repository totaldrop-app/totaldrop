#!/bin/sh
# Build one Linux package in its own container, from the working tree.
#
#   packaging/build.sh deb|rpm|arch      → dist/packages/
#
# Each format has its own recipe (packaging/debian, packaging/rpm,
# packaging/arch) and its own build image (packaging/containers), so they
# never affect each other.
set -eu
format=$1
root=$(cd "$(dirname "$0")/.." && pwd)
ver=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -1)
src=totaldrop-linux-v$ver
out=$root/dist/packages; mkdir -p "$out"
work=$root/dist/work-$format; rm -rf "$work"; mkdir -p "$work"
image=totaldrop-build-$format
case $format in deb) df=debian ;; rpm) df=fedora ;; arch) df=arch ;; *) echo "deb, rpm or arch" >&2; exit 2 ;; esac

docker build -q -t "$image" -f "$root/packaging/containers/$df.Dockerfile" "$root/packaging/containers" >/dev/null
tar -C "$root" --exclude=./target --exclude=./dist --exclude=./.git --exclude=./app/build \
    --exclude=./app/.dart_tool --exclude=./fuzz/target \
    --transform "s,^\.,$src," -czf "$work/$src.tar.gz" .

case $format in
deb)
  docker run --rm -u "$(id -u):$(id -g)" -e HOME=/tmp -v "$work:/w" "$image" sh -ec "
    cd /w && tar xzf $src.tar.gz && cd $src && cp -r packaging/debian debian
    export RUSTUP_TOOLCHAIN=1.99.1
    dpkg-buildpackage -b -us -uc
    lintian --no-tag-display-limit ../*.deb || true"
  cp "$work"/*.deb "$out/" ;;
rpm)
  docker run --rm -u "$(id -u):$(id -g)" -e HOME=/tmp -v "$work:/w" "$image" sh -ec "
    mkdir -p /tmp/rpm/SOURCES && cp /w/$src.tar.gz /tmp/rpm/SOURCES/
    tar xzf /w/$src.tar.gz -C /tmp $src/packaging/rpm/totaldrop.spec
    export RUSTUP_TOOLCHAIN=1.99.1
    rpmbuild --define '_topdir /tmp/rpm' -bb /tmp/$src/packaging/rpm/totaldrop.spec
    cp /tmp/rpm/RPMS/*/*.rpm /w/"
  cp "$work"/*.rpm "$out/" ;;
arch)
  cp "$root/packaging/arch/PKGBUILD" "$work/"
  mv "$work/$src.tar.gz" "$work/totaldrop-$ver.tar.gz"
  docker run --rm -v "$work:/w" "$image" sh -ec "
    chown -R builder /w && cd /w && sudo -u builder makepkg -sf --noconfirm
    chown -R $(id -u):$(id -g) /w"
  cp "$work"/*.pkg.tar.zst "$out/" ;;
esac
ls -la "$out"
