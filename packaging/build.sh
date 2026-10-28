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
"$root/packaging/check-versions.sh" >/dev/null
ver=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/linux/Cargo.toml" | head -1)
# One timestamp for the source tarball and every packager: the last commit's.
SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-$(git -C "$root" log -1 --format=%ct 2>/dev/null || date +%s)}
export SOURCE_DATE_EPOCH
src=totaldrop-linux-v$ver
out=$root/dist/packages; mkdir -p "$out"
work=$root/dist/work-$format; rm -rf "$work"; mkdir -p "$work"
image=totaldrop-build-$format
case $format in deb) df=debian ;; rpm) df=fedora ;; arch) df=arch ;; *) echo "deb, rpm or arch" >&2; exit 2 ;; esac

docker build -q -t "$image" -f "$root/packaging/containers/$df.Dockerfile" "$root/packaging/containers" >/dev/null
tar -C "$root" --exclude=./target --exclude=./dist --exclude=./.git --exclude=./app/build \
    --exclude=./app/.dart_tool --exclude=./fuzz/target \
    --sort=name --mtime="@$SOURCE_DATE_EPOCH" --owner=0 --group=0 --numeric-owner \
    --mode=u+rwX,go=rX --transform "s,^\.,$src," -cf "$work/$src.tar" .
gzip -n "$work/$src.tar"

case $format in
deb)
  docker run --rm -u "$(id -u):$(id -g)" -e HOME=/tmp -e SOURCE_DATE_EPOCH -v "$work:/w" "$image" sh -ec "
    cd /w && tar xzf $src.tar.gz && cd $src && cp -r packaging/debian debian
    export RUSTUP_TOOLCHAIN=1.99.1
    dpkg-buildpackage -b -us -uc
    lintian --no-tag-display-limit ../*.deb || true"
  cp "$work"/*.deb "$out/" ;;
rpm)
  docker run --rm -u "$(id -u):$(id -g)" -e HOME=/tmp -e SOURCE_DATE_EPOCH -v "$work:/w" "$image" sh -ec "
    mkdir -p /tmp/rpm/SOURCES && cp /w/$src.tar.gz /tmp/rpm/SOURCES/
    tar xzf /w/$src.tar.gz -C /tmp $src/packaging/rpm/totaldrop.spec
    export RUSTUP_TOOLCHAIN=1.99.1
    rpmbuild --define '_topdir /tmp/rpm' --define 'use_source_date_epoch_as_buildtime 1' \
      --define '_buildhost reproducible' \
      --define 'clamp_mtime_to_source_date_epoch 1' -bb /tmp/$src/packaging/rpm/totaldrop.spec
    cp /tmp/rpm/RPMS/*/*.rpm /w/"
  cp "$work"/*.rpm "$out/" ;;
arch)
  cp "$root/packaging/arch/PKGBUILD" "$work/"
  mv "$work/$src.tar.gz" "$work/totaldrop-$ver.tar.gz"
  docker run --rm -e SOURCE_DATE_EPOCH -v "$work:/w" "$image" sh -ec "
    chown -R builder /w && cd /w
    sudo -u builder env TOTALDROP_RUST=1.99.1 RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo \
      PATH=/opt/cargo/bin:\$PATH SOURCE_DATE_EPOCH=\$SOURCE_DATE_EPOCH makepkg -sf --noconfirm
    chown -R $(id -u):$(id -g) /w"
  cp "$work"/*.pkg.tar.zst "$out/" ;;
esac
ls -la "$out"
