#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Check that every place naming a version names the same one.
#
# The Linux app: linux/Cargo.toml, the Arch PKGBUILD, the RPM spec, the
# Debian changelog and the newest release in the AppStream metainfo. RPM,
# Debian and AppStream must write a pre-release with "~" (0.1.0~beta.4), so
# that it sorts before the final release. With a tag (linux-v0.1.0-beta.5),
# the tag must match too, and the command line the packages ship must have
# a version higher than any released one, or the highest with the same
# code (check-cli-version.sh; it needs the full history, which the release
# workflow fetches).
#
# The tools: every pinned Rust version (the Flatpak's too) equals
# rust-toolchain.toml, every pinned Flutter version is the same, every
# Node in the workflows and @types/node have the same major, and the Arch
# check container uses the same base image as the Arch package (Dependabot
# updates only the latter).
#
#   packaging/check-versions.sh [linux-vX.Y.Z[-pre]]
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
status=0
fail() {
  echo "$1" >&2
  status=1
}
same() {
  [ "$2" = "$3" ] || fail "mismatch: $1 says '$2', expected '$3'"
}

cargo=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/linux/Cargo.toml" | head -1)
[ -n "$cargo" ] || { echo "no version in linux/Cargo.toml" >&2; exit 1; }
tilde=$(printf %s "$cargo" | sed 's/-/~/')

same packaging/arch/PKGBUILD "$(sed -n 's/^_ver=\(.*\)$/\1/p' "$root/packaging/arch/PKGBUILD" | head -1)" "$cargo"
same "packaging/rpm/totaldrop.spec (Version)" \
  "$(sed -n 's/^Version:[[:space:]]*\(.*\)$/\1/p' "$root/packaging/rpm/totaldrop.spec" | head -1)" "$tilde"
same "packaging/rpm/totaldrop.spec (upstream_version)" \
  "$(sed -n 's/^%global upstream_version[[:space:]]*\(.*\)$/\1/p' "$root/packaging/rpm/totaldrop.spec" | head -1)" "$cargo"
same packaging/debian/changelog \
  "$(sed -n '1s/^[^(]*(\([^)]*\)).*/\1/p' "$root/packaging/debian/changelog" | sed 's/-[0-9][0-9]*$//')" "$tilde"
same packaging/linux/app.totaldrop.TotalDrop.metainfo.xml \
  "$(sed -n 's/.*<release version="\([^"]*\)".*/\1/p' "$root/packaging/linux/app.totaldrop.TotalDrop.metainfo.xml" | head -1)" "$tilde"
if [ $# -ge 1 ]; then
  same "tag $1" "${1#linux-v}" "$cargo"
  "$root/packaging/check-cli-version.sh" || status=1
fi

rust=$(sed -n 's/^channel = "\(.*\)"/\1/p' "$root/rust-toolchain.toml" | head -1)
[ -n "$rust" ] || fail "no channel in rust-toolchain.toml"
for f in .github/workflows/core.yml .github/workflows/linux.yml packaging/build.sh \
  packaging/containers/arch.Dockerfile packaging/containers/debian.Dockerfile \
  packaging/containers/fedora.Dockerfile; do
  [ -f "$root/$f" ] || { fail "$f: missing"; continue; }
  # A file on this list with no pin left is a pin that was lost.
  vs=$(grep -oE "(RUSTUP_TOOLCHAIN[=:] *[\"']?|TOTALDROP_RUST=|rust-version: *[\"']?|--default-toolchain |toolchain install )[0-9]+\.[0-9]+\.[0-9]+" "$root/$f" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+$' || true)
  [ -n "$vs" ] || fail "$f: no Rust pin found"
  for v in $vs; do
    same "$f (Rust)" "$v" "$rust"
  done
done

# The Flatpak builds the same Rust from its release archive.
fp_rust=$(grep -oE 'static\.rust-lang\.org/dist/rust-[0-9]+\.[0-9]+\.[0-9]+-' "$root/packaging/flatpak/app.totaldrop.TotalDrop.yml" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | sort -u || true)
[ -n "$fp_rust" ] || fail "packaging/flatpak/app.totaldrop.TotalDrop.yml: no Rust pin found"
for v in $fp_rust; do
  same "packaging/flatpak/app.totaldrop.TotalDrop.yml (Rust)" "$v" "$rust"
done

flutter=$(grep -hoE "flutter-version: *[\"']?[0-9.]+" "$root"/.github/workflows/*.yml | grep -oE '[0-9.]+$' | sort -u || true)
[ "$(printf '%s\n' "$flutter" | grep -c . || true)" -eq 1 ] || fail "Flutter must be pinned to exactly one version, found: '$(echo $flutter)'"

# Node: every setup-node step in the workflows names its version, each one
# a fixed major ("24") or release ("24.11.1"), never a moving one (lts/*,
# 24.x), all with the same major; and @types/node has that major in every
# package.json (docs/surec.md, "Node"). The Flatpak job runs the Node in
# Flathub's image, pinned by digest, and is not checked here.
node_majors=
for f in "$root"/.github/workflows/*.yml "$root"/.github/workflows/*.yaml; do
  [ -f "$f" ] || continue
  rel=${f#"$root"/}
  steps=$(grep -cE '^[[:space:]]*(-[[:space:]]+)?uses:[[:space:]]*actions/setup-node@' "$f" || true)
  vs=$(sed -nE "s/^[[:space:]]*node-version:[[:space:]]*//p" "$f" | sed -E "s/[[:space:]]+#.*$//; s/^[\"']//; s/[\"'][[:space:]]*$//")
  [ "$steps" -eq "$(printf '%s' "$vs" | grep -c . || true)" ] ||
    fail "$rel: $steps setup-node steps, but not one node-version line for each"
  # Read line by line, so a value like lts/* is never expanded as a glob.
  while IFS= read -r v; do
    [ -n "$v" ] || continue
    if printf '%s' "$v" | grep -qE '^[0-9]+(\.[0-9]+\.[0-9]+)?$'; then
      node_majors="$node_majors ${v%%.*}"
    else
      fail "$rel: node-version '$v' is not a fixed Node major or release"
    fi
  done <<EOF
$vs
EOF
done
node=$(printf '%s\n' $node_majors | sort -u | grep . || true)
if [ "$(printf '%s\n' "$node" | grep -c . || true)" -ne 1 ]; then
  fail "Node must be pinned to exactly one major, found: '$(echo $node)'"
  node=
fi
# With no single major there is nothing to compare @types/node with.
packages=
[ -z "$node" ] || packages=$(find "$root" \( -name node_modules -o -name .git -o -name target -o -name build -o -name .dart_tool \) -prune -o -name package.json -print)
while IFS= read -r f; do
  [ -n "$f" ] || continue
  t=$(sed -nE 's/.*"@types\/node"[[:space:]]*:[[:space:]]*"[^0-9"]*([0-9]+)[^"]*".*/\1/p' "$f" | head -1)
  [ -z "$t" ] || same "${f#"$root"/} (@types/node major)" "$t" "$node"
done <<EOF
$packages
EOF

base=$(sed -n 's/^FROM \(archlinux:[^ ]*\).*/\1/p' "$root/packaging/containers/arch.Dockerfile" | head -1)
check=$(sed -n 's/^ *container: *\(archlinux:[^ ]*\).*/\1/p' "$root/.github/workflows/linux.yml" | head -1)
[ -n "$base" ] || fail "no archlinux FROM in packaging/containers/arch.Dockerfile"
same ".github/workflows/linux.yml (container)" "$check" "$base"

# The deb and rpm images pin the same rustup-init release and checksums.
rustup_args() { grep -E '^ARG RUSTUP_(VERSION|SHA256_[A-Z0-9_]+)=' "$root/packaging/containers/$1.Dockerfile" | sort; }
rustup_deb=$(rustup_args debian)
[ -n "$rustup_deb" ] || fail "packaging/containers/debian.Dockerfile: no RUSTUP_* ARGs"
same "packaging/containers/fedora.Dockerfile (RUSTUP_* ARGs)" "$(rustup_args fedora)" "$rustup_deb"

[ $status -eq 0 ] && echo "versions agree: $cargo; Rust $rust; Flutter $flutter; Node $node"
exit $status
