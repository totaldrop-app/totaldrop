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
# The recipes: every Linux package recipe and the Flatpak keep build times
# out of the Qt resources in the same way, and the AppStream metainfo names
# the same license as the RPM spec and the PKGBUILD.
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

# Node (docs/surec.md, "Node"). In the workflows: every setup-node step
# names its version, each one a fixed major ("24") or release ("24.11.1"),
# never a moving one (lts/*, 24.x), all with the same major; and every job
# that runs node, npm or npx has a setup-node step, so none runs whatever
# Node the runner image ships. The one exception is the Flatpak job, which
# runs the Node in Flathub's image, pinned by digest. For @types/node:
# every tracked package.json that names it pins that major, and its
# package-lock.json installs that major.
#
# One pass over a workflow prints "V value" for each node-version,
# "M line" for a setup-node step with none, and "J job" for a job that runs
# Node with no setup-node step. A step ends at the next list item. A job
# ends at the next key under jobs: with the first job's indent; Node run
# before any job key counts as a job named "?". A line runs Node when it
# names node, npm or npx as a word, or node_modules/.bin/; step names do
# not count.
node_scan() {
  awk -v q="'" '
    function endstep() { if (pending) print "M " pline; pending = 0 }
    function endjob() {
      endstep()
      if (runs && !setup) print "J " (job == "" ? "?" : job)
      job = ""; runs = 0; setup = 0
    }
    /^[[:space:]]*(#.*)?$/ { next }
    /^jobs:[[:space:]]*(#.*)?$/ { injobs = 1; jind = -1; next }
    /^[^[:space:]]/ { if (injobs) { endjob(); injobs = 0 } }
    injobs {
      match($0, /^ */); ind = RLENGTH
      if (jind < 0) jind = ind
      if (ind == jind && $0 ~ "^ *[\"" q "]?[A-Za-z0-9_-]+[\"" q "]?:[[:space:]]*(#.*)?$") {
        endjob(); job = $0
        sub(/^ */, "", job); sub(/:.*$/, "", job); gsub("[\"" q "]", "", job)
        next
      }
    }
    /^[[:space:]]*-[[:space:]]/ { endstep() }
    $0 ~ "^[[:space:]]*(-[[:space:]]+)?uses:[[:space:]]*[\"" q "]?actions/setup-node@" {
      pending = 1; pline = NR; setup = 1; next
    }
    /^[[:space:]]*node-version:/ {
      v = $0
      sub(/^[[:space:]]*node-version:[[:space:]]*/, "", v)
      sub(/[[:space:]]+#.*$/, "", v)
      sub(/[[:space:]]+$/, "", v)
      if (v ~ "^[\"" q "]") v = substr(v, 2)
      if (v ~ "[\"" q "]$") v = substr(v, 1, length(v) - 1)
      print "V " v; pending = 0; next
    }
    /^[[:space:]]*(-[[:space:]]+)?name:/ { next }
    /(^|[^A-Za-z0-9_.\/-])(node|npm|npx)([^A-Za-z0-9_.-]|$)/ || /node_modules\/\.bin\// { runs = 1 }
    END { endjob() }
  ' "$1"
}
node_majors=
for f in "$root"/.github/workflows/*.yml "$root"/.github/workflows/*.yaml; do
  [ -f "$f" ] || continue
  rel=${f#"$root"/}
  # Read line by line, so a value like lts/* is never expanded as a glob.
  while IFS=" " read -r kind v; do
    case $kind in
      V)
        if printf '%s' "$v" | grep -qE '^[0-9]+(\.[0-9]+\.[0-9]+)?$'; then
          node_majors="$node_majors ${v%%.*}"
        else
          fail "$rel: node-version '$v' is not a fixed Node major or release"
        fi ;;
      M) fail "$rel:$v: setup-node step with no node-version" ;;
      J) [ "$rel $v" = ".github/workflows/linux.yml flatpak" ] ||
        fail "$rel: job '$v' runs Node with no setup-node step" ;;
    esac
  done <<EOF
$(node_scan "$f")
EOF
done
node=$(printf '%s\n' $node_majors | sort -u | grep . || true)
if [ "$(printf '%s\n' "$node" | grep -c . || true)" -ne 1 ]; then
  fail "Node must be pinned to exactly one major, found: '$(echo $node)'"
  node=
fi
# With no single major there is nothing to compare @types/node with. Only
# tracked files count; a copy with no .git (a source archive) is searched.
packages=
if [ -n "$node" ]; then
  # git can refuse a checkout owned by another user (a container); then
  # the search is the fallback too.
  if ! { [ -e "$root/.git" ] && packages=$(git -C "$root" ls-files -- package.json '*/package.json' 2>/dev/null); }; then
    packages=$(cd "$root" && find . \( -name node_modules -o -name .git -o -name target -o -name build -o -name .dart_tool \) -prune -o -type f -name package.json -print | sed 's|^\./||')
  fi
fi
while IFS= read -r pkg; do
  [ -n "$pkg" ] || continue
  grep -q '"@types/node"' "$root/$pkg" || continue
  t=$(sed -nE 's/.*"@types\/node"[[:space:]]*:[[:space:]]*"([^"]*)".*/\1/p' "$root/$pkg" | head -1)
  if printf '%s' "$t" | grep -qE '^[~^]?[0-9]+(\.[0-9]+){0,2}$'; then
    t=${t#[~^]}
    same "$pkg (@types/node major)" "${t%%.*}" "$node"
  else
    fail "$pkg: @types/node '$t' does not pin one major"
  fi
  lock=$(dirname "$pkg")/package-lock.json
  if [ -f "$root/$lock" ]; then
    l=$(awk '/"node_modules\/@types\/node":[[:space:]]*\{/ { f = 1; next }
      f && /"version":/ { sub(/.*"version":[[:space:]]*"/, ""); sub(/".*/, ""); print; exit }' "$root/$lock")
    same "$lock (@types/node major)" "${l%%.*}" "$node"
  else
    fail "$pkg: names @types/node but has no package-lock.json"
  fi
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

# Qt's resource compiler stores each file's modification time, and the
# qmldir that cxx-qt writes during the build carries the build's own time
# into the binary (#252). cxx-qt starts rcc with an empty environment, so
# neither SOURCE_DATE_EPOCH nor QT_RCC_SOURCE_DATE_OVERRIDE reaches it;
# format 1 has no timestamps (and no zstd). The QML files are compiled
# ahead of time by qmlcachegen, so nothing reads these timestamps at run
# time. Each recipe sets CXX_QT_AUTORCC_OPTIONS itself, so a build straight
# from dpkg-buildpackage, rpmbuild or makepkg is reproducible too (#266);
# build.sh does not set it, so the reproducible builds check the recipes.
# The RPM spec and the PKGBUILD set it in the test step as well, so the
# tests build the same Qt resources; and where the tests run before the
# install step (the PKGBUILD, like debian/rules), cargo test may relink the
# binaries that step copies.
#
# One pass prints "SECTION VALUE" for each line that sets it, outside
# comments: SECTION is the RPM section (%build) or the shell function
# (build()) the line is in, or "-" before any. Only an export counts: a
# plain variable never reaches cargo. In debian/rules, a make export at
# the top, with = or := (not ?=, which lets the environment win); in the
# spec and the PKGBUILD, a shell export (make's spaced form would fail
# there and set nothing). In the Flatpak, only a key of the env: mapping
# under build-options: of the totaldrop module itself, not of a module
# nested in it (a module nested before build-options: makes the check
# fail, loudly).
rcc=--format-version:1:--no-zstd
rcc_values() {
  yml=0
  case $1 in
    *.yml) yml=1 set_re='^[[:space:]]+CXX_QT_AUTORCC_OPTIONS:' ;;
    */rules) set_re='^export[[:space:]]+CXX_QT_AUTORCC_OPTIONS[[:space:]]*:?=' ;;
    *) set_re='^[[:space:]]*export[[:space:]]+CXX_QT_AUTORCC_OPTIONS=' ;;
  esac
  awk -v q="'" -v envind=-1 -v bind=-1 -v set_re="$set_re" -v yml="$yml" '
    /^[[:space:]]*#/ { next }
    /^%(prep|build|install|check|files|description|package|changelog|pre|post|preun|postun)([[:space:]]|$)/ { sec = $1 }
    /^[A-Za-z_][A-Za-z0-9_]*\(\)/ { sec = $1 }
    yml && !/^[[:space:]]*$/ {
      match($0, /^ */); ind = RLENGTH
      if (/^ *- name:/) { mod = (ind == 2 ? $3 : "") }
      if (envind >= 0 && ind <= envind) envind = -1
      if (bind >= 0 && ind <= bind) bind = -1
      if (/^ +build-options:[[:space:]]*$/) { bind = ind; next }
      if (bind >= 0 && /^ +env:[[:space:]]*$/) { envind = ind; next }
    }
    $0 ~ set_re && (!yml || (mod == "totaldrop" && envind >= 0)) {
      v = $0
      sub(/.*CXX_QT_AUTORCC_OPTIONS[[:space:]]*(:|[:?]?=)[[:space:]]*/, "", v)
      sub(/[[:space:]]+#.*$/, "", v)
      gsub("[\"" q "]", "", v)
      sub(/[[:space:]]+$/, "", v)
      print (sec == "" ? "-" : sec) " " v
    }
  ' "$1"
}
# check_rcc FILE SECTIONS: FILE sets it to $rcc, in exactly SECTIONS.
check_rcc() {
  f=$1 want=$2
  got=$(rcc_values "$root/$f")
  [ -n "$got" ] || { fail "$f: CXX_QT_AUTORCC_OPTIONS not set"; return; }
  secs=$(printf '%s\n' "$got" | cut -d' ' -f1 | sort -u | tr '\n' ' ' | sed 's/ $//')
  same "$f (CXX_QT_AUTORCC_OPTIONS set in)" "$secs" "$want"
  while read -r _ v; do
    same "$f (CXX_QT_AUTORCC_OPTIONS)" "$v" "$rcc"
  done <<EOF
$got
EOF
}
check_rcc packaging/debian/rules "-"
check_rcc packaging/rpm/totaldrop.spec "%build %check"
check_rcc packaging/arch/PKGBUILD "build() check()"
check_rcc packaging/flatpak/app.totaldrop.TotalDrop.yml "-"

# One license, written the SPDX way: the PKGBUILD's list joined with AND.
license=$(sed -n 's/^License:[[:space:]]*//p' "$root/packaging/rpm/totaldrop.spec" | head -1)
[ -n "$license" ] || fail "packaging/rpm/totaldrop.spec: no License"
meta_license=$(sed -n 's/.*<project_license>\(.*\)<\/project_license>.*/\1/p' \
  "$root/packaging/linux/app.totaldrop.TotalDrop.metainfo.xml" | head -1)
same "packaging/linux/app.totaldrop.TotalDrop.metainfo.xml (project_license)" "$meta_license" "$license"
arch_license=$(sed -n 's/^license=(\(.*\))[[:space:]]*$/\1/p' "$root/packaging/arch/PKGBUILD" | head -1 |
  awk -v q="'" '{
    n = split($0, a, q)
    for (i = 2; i <= n; i += 2) {
      it = a[i]
      if (it ~ / /) it = "(" it ")"
      out = out (out == "" ? "" : " AND ") it
    }
    print out
  }')
same "packaging/arch/PKGBUILD (license)" "$arch_license" "$license"

[ $status -eq 0 ] && echo "versions agree: $cargo; Rust $rust; Flutter $flutter; Node $node"
exit $status
