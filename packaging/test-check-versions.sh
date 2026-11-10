#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Tests for check-versions.sh: its Node part (docs/surec.md, "Node"), the
# recipes' Qt resource options and their one license (#266). Each
# case copies the files the check reads, changes one thing and runs it. A
# case that should fail must also print the message it names, so a failure
# for another reason does not count. The copy sits under a path with a
# space, as a checkout might.
#
#   packaging/test-check-versions.sh
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
failed=0

base="$tmp/base copy"
mkdir -p "$base"
(cd "$root" && tar -cf - linux/Cargo.toml rust-toolchain.toml packaging .github \
  server/rendezvous-worker/package.json server/rendezvous-worker/package-lock.json \
  server/relay-worker/package.json server/relay-worker/package-lock.json) |
  (cd "$base" && tar -xf -)

# edit FILE SED-SCRIPT: sed -i, the same under GNU and BSD sed.
edit() {
  sed "$2" "$1" > "$1.new"
  mv "$1.new" "$1"
}

# expect pass NAME [COMMANDS]
# expect fail NAME MESSAGE [COMMANDS]
# COMMANDS run in the copy; MESSAGE is a fixed string the check must print.
expect() {
  want=$1 name=$2
  shift 2
  msg=
  if [ "$want" = fail ]; then msg=$1; shift; fi
  rm -rf "$tmp/w"
  cp -R "$base" "$tmp/w"
  (cd "$tmp/w" && eval "${1:-:}")
  if sh "$tmp/w/packaging/check-versions.sh" > "$tmp/out" 2>&1; then got=pass; else got=fail; fi
  if [ "$got" = "$want" ] && { [ -z "$msg" ] || grep -qF -- "$msg" "$tmp/out"; }; then
    echo "ok   $name"
  else
    echo "FAIL $name: wanted $want${msg:+ with \"$msg\"}, got $got"; sed 's/^/     /' "$tmp/out"; failed=1
  fi
}

wf=.github/workflows
pkg=server/rendezvous-worker/package.json
lock=server/rendezvous-worker/package-lock.json
major=$(sed -n 's/^ *node-version: *"\([0-9]*\)".*/\1/p' "$root/$wf/ci.yml" | head -1)
[ -n "$major" ] || { echo "no node-version in $wf/ci.yml" >&2; exit 1; }
next=$((major + 2))
nv="node-version: \"$major\""

expect pass "the repository as it is"

# node-version values.
expect fail "lts/* in one workflow" "node-version 'lts/*'" \
  "edit $wf/site.yml 's|$nv|node-version: lts/*|'"
expect fail "a range (24.x)" "node-version '$major.x'" \
  "edit $wf/site.yml 's|$nv|node-version: $major.x|'"
expect fail "a bare *" "node-version '*'" \
  "edit $wf/site.yml 's|$nv|node-version: \"*\"|'"
expect fail "another major in one workflow" "exactly one major" \
  "edit $wf/deploy.yml 's|$nv|node-version: \"$next\"|'"
expect pass "a full release of the same major, with a comment" \
  "edit $wf/deploy.yml 's|$nv|node-version: $major.11.1 # pinned|'"
expect pass "single quotes" \
  "edit $wf/deploy.yml \"s|$(printf '%s' "$nv" | sed 's/"/\\"/g')|node-version: '$major'|\""
expect pass "trailing spaces" \
  "edit $wf/deploy.yml 's|$nv|node-version: $major   |'"

# setup-node steps and the jobs that run Node.
expect fail "a setup-node step with no node-version" "setup-node step with no node-version" \
  "edit $wf/linux.yml '/$nv/d'"
expect fail "node-version-file instead of node-version" "setup-node step with no node-version" \
  "edit $wf/ci.yml 's|$nv|node-version-file: .nvmrc|'"
expect fail "a quoted uses: with no node-version" "setup-node step with no node-version" \
  "edit $wf/site.yml 's|uses: actions/setup-node@|uses: \"actions/setup-node@|; /$nv/d'"
expect fail "a node-version in another step does not cover a step with none" \
  "setup-node step with no node-version" \
  "edit $wf/site.yml '/$nv/d; s|^\\(      - name: Scripts parse\\)\$|\\1\\
        env:\\
          $nv|'"
expect fail "a job that runs node with no setup-node step" "job 'check' runs Node" \
  "edit $wf/site.yml '/uses: actions\\/setup-node@/d; /^        with:\$/d; /$nv/d'"

# workflow NAME JOB-INDENT STEP-LINE...: a new workflow whose second job,
# after one with setup-node, runs the given steps.
workflow() {
  name=$1 i=$2
  shift 2
  {
    printf 'on: push\njobs:\n'
    printf '%sa:\n%s  runs-on: x\n%s  steps:\n' "$i" "$i" "$i"
    printf '%s    - uses: actions/setup-node@0 # v0\n%s      with:\n%s        %s\n' "$i" "$i" "$i" "$nv"
    printf '%s    - run: node --version\n' "$i"
    printf '%sb: # the second job\n%s  runs-on: x\n%s  steps:\n' "$i" "$i" "$i"
    for l in "$@"; do printf '%s    %s\n' "$i" "$l"; done
  } > "$wf/$name.yml"
}
expect pass "a second job that does not run Node" \
  "workflow x '  ' '- name: check node modules' '  run: echo nodes npmrc'"
expect fail "a second job, after a comment on its key, that runs node" "job 'b' runs Node" \
  "workflow x '  ' '- run: node'"
expect fail "jobs indented by four spaces" "job 'b' runs Node" \
  "workflow x '    ' '- run: npm ci'"
expect fail "node quoted" "job 'b' runs Node" \
  "workflow x '  ' '- run: \"node tools/x.mjs\"'"
expect fail "npm at the end of a line" "job 'b' runs Node" \
  "workflow x '  ' '- run: cd server && npm'"
expect fail "a tool from node_modules/.bin" "job 'b' runs Node" \
  "workflow x '  ' '- run: ./server/relay-worker/node_modules/.bin/wrangler deploy'"
expect fail "a quoted job key" "job 'b' runs Node" \
  "workflow x '  ' '- run: npx wrangler' && edit $wf/x.yml 's|^  b:|  \"b\":|'"

# @types/node.
expect fail "@types/node a major ahead" "package.json (@types/node major)" \
  "edit $pkg 's|\"@types/node\": \"[^0-9\"]*$major\\.|\"@types/node\": \"^$next.|'"
expect fail "@types/node latest" "does not pin one major" \
  "edit $pkg 's|\"@types/node\": \"[^\"]*\"|\"@types/node\": \"latest\"|'"
expect fail "@types/node as an open range" "does not pin one major" \
  "edit $pkg 's|\"@types/node\": \"[^\"]*\"|\"@types/node\": \">=$major\"|'"
expect pass "@types/node with a caret on the same major" \
  "edit $pkg 's|\"@types/node\": \"\\([0-9]\\)|\"@types/node\": \"^\\1|'"
expect fail "the lock file installs another major" "package-lock.json (@types/node major)" \
  "edit $lock '/\"node_modules\\/@types\\/node\": {/,/\"version\"/s|\"version\": \"$major\\.|\"version\": \"$next.|'"
expect fail "@types/node with no lock file" "has no package-lock.json" \
  "rm $lock"
expect pass "an untracked package.json is not read" \
  "git init -q . && git add -A && mkdir -p scratch && printf '{\"devDependencies\":{\"@types/node\":\"^1\"}}\\n' > scratch/package.json"
expect fail "a tracked package.json is read" "scratch/package.json (@types/node major)" \
  "git init -q . && mkdir -p scratch && printf '{\"devDependencies\":{\"@types/node\":\"^1\"}}\\n' > scratch/package.json && git add -A"

expect pass "every node-version and @types/node upgraded together" \
  "for f in $wf/*.yml; do edit \"\$f\" 's|$nv|node-version: \"$next\"|'; done
   edit $pkg 's|\"@types/node\": \"$major\\.|\"@types/node\": \"$next.|'
   edit $lock 's|\"@types/node\": \"$major\\.|\"@types/node\": \"$next.|; /\"node_modules\\/@types\\/node\": {/,/\"version\"/s|\"version\": \"$major\\.|\"version\": \"$next.|'"

# The recipes' Qt resource options (#266).
deb=packaging/debian/rules
spec=packaging/rpm/totaldrop.spec
arch=packaging/arch/PKGBUILD
fp=packaging/flatpak/app.totaldrop.TotalDrop.yml
meta=packaging/linux/app.totaldrop.TotalDrop.metainfo.xml
expect fail "debian/rules without them" "$deb: CXX_QT_AUTORCC_OPTIONS not set" \
  "edit $deb '/CXX_QT_AUTORCC_OPTIONS/d'"
expect fail "only a comment names them" "$deb: CXX_QT_AUTORCC_OPTIONS not set" \
  "edit $deb 's|^export CXX_QT_AUTORCC_OPTIONS|# export CXX_QT_AUTORCC_OPTIONS|'"
expect fail "the Flatpak without them" "$fp: CXX_QT_AUTORCC_OPTIONS not set" \
  "edit $fp '/CXX_QT_AUTORCC_OPTIONS/d'"
expect fail "the spec's %check without them" "$spec (CXX_QT_AUTORCC_OPTIONS set in)" \
  "edit $spec '/^%check/,/^%files/{/CXX_QT_AUTORCC_OPTIONS/d;}'"
expect fail "the PKGBUILD's build() without them" "$arch (CXX_QT_AUTORCC_OPTIONS set in)" \
  "edit $arch '/^build()/,/^}/{/CXX_QT_AUTORCC_OPTIONS/d;}'"
expect fail "the PKGBUILD sets them in package() as well" "$arch (CXX_QT_AUTORCC_OPTIONS set in)" \
  "edit $arch 's|^package() {|&\\
  export CXX_QT_AUTORCC_OPTIONS=--format-version:1:--no-zstd|'"
expect fail "another value in one place" "$spec (CXX_QT_AUTORCC_OPTIONS)" \
  "edit $spec '/^%check/,/^%files/s|--no-zstd|--no-compress|'"
expect fail "a PKGBUILD variable that is not exported" "$arch: CXX_QT_AUTORCC_OPTIONS not set" \
  "edit $arch 's|export CXX_QT_AUTORCC_OPTIONS|CXX_QT_AUTORCC_OPTIONS|'"
expect fail "a make variable that is not exported" "$deb: CXX_QT_AUTORCC_OPTIONS not set" \
  "edit $deb 's|^export CXX_QT_AUTORCC_OPTIONS|CXX_QT_AUTORCC_OPTIONS|'"
expect fail "the spec's %build sets a variable it does not export" "$spec (CXX_QT_AUTORCC_OPTIONS set in)" \
  "edit $spec '/^%build/,/^%install/s|export CXX_QT_AUTORCC_OPTIONS|CXX_QT_AUTORCC_OPTIONS|'"
expect fail "the Flatpak sets them for the rust module only" "$fp: CXX_QT_AUTORCC_OPTIONS not set" \
  "edit $fp '/CXX_QT_AUTORCC_OPTIONS/d'
   edit $fp 's|^  - name: rust$|&\\
    build-options:\\
      env:\\
        CXX_QT_AUTORCC_OPTIONS: --format-version:1:--no-zstd|'"
expect fail "another value in the Flatpak" "$fp (CXX_QT_AUTORCC_OPTIONS)" \
  "edit $fp 's|CXX_QT_AUTORCC_OPTIONS: .*|CXX_QT_AUTORCC_OPTIONS: --no-zstd|'"
expect pass "a quoted value" \
  "edit $arch 's|=--format-version:1:--no-zstd|=\"--format-version:1:--no-zstd\"|'"
expect pass "a single-quoted value with a comment after it" \
  "edit $arch \"s|=--format-version:1:--no-zstd|='--format-version:1:--no-zstd' # #252|\""
expect pass "make's :=" \
  "edit $deb 's|^export CXX_QT_AUTORCC_OPTIONS =|export CXX_QT_AUTORCC_OPTIONS :=|'"
expect fail "make's ?=, which lets the environment win" "$deb: CXX_QT_AUTORCC_OPTIONS not set" \
  "edit $deb 's|^export CXX_QT_AUTORCC_OPTIONS =|export CXX_QT_AUTORCC_OPTIONS ?=|'"
expect fail "make's spaced form in the PKGBUILD" "$arch: CXX_QT_AUTORCC_OPTIONS not set" \
  "edit $arch 's|export CXX_QT_AUTORCC_OPTIONS=|export CXX_QT_AUTORCC_OPTIONS = |'"
expect fail "the Flatpak key under build-options, not env" "$fp: CXX_QT_AUTORCC_OPTIONS not set" \
  "edit $fp 's|^        CXX_QT_AUTORCC_OPTIONS:|      CXX_QT_AUTORCC_OPTIONS:|'"
expect fail "the Flatpak key in an env: on the module, not under build-options" "$fp: CXX_QT_AUTORCC_OPTIONS not set" \
  "edit $fp '/CXX_QT_AUTORCC_OPTIONS/d'
   edit $fp 's|^    build-commands:|    env:\\
      CXX_QT_AUTORCC_OPTIONS: --format-version:1:--no-zstd\\
&|'"
expect fail "the Flatpak key for one architecture only" "$fp: CXX_QT_AUTORCC_OPTIONS not set" \
  "edit $fp '/CXX_QT_AUTORCC_OPTIONS/d'
   edit $fp 's|^      env:$|      arch:\\
        x86_64:\\
          env:\\
            CXX_QT_AUTORCC_OPTIONS: --format-version:1:--no-zstd\\
&|'"
expect fail "a per-arch Flatpak value that overrides the common one" "$fp (CXX_QT_AUTORCC_OPTIONS)" \
  "edit $fp 's|^      env:$|      arch:\\
        x86_64:\\
          env:\\
            CXX_QT_AUTORCC_OPTIONS: --no-zstd\\
&|'"
expect fail "a plain variable with another value next to the export" "$arch (CXX_QT_AUTORCC_OPTIONS)" \
  "edit $arch 's|^  cargo build|  CXX_QT_AUTORCC_OPTIONS=--no-zstd\\
&|'"
expect fail "the Flatpak key in a module nested in totaldrop" "$fp: CXX_QT_AUTORCC_OPTIONS not set" \
  "edit $fp '/CXX_QT_AUTORCC_OPTIONS/d'
   edit $fp 's|^    build-commands:|    modules:\\
      - name: nested\\
        build-options:\\
          env:\\
            CXX_QT_AUTORCC_OPTIONS: --format-version:1:--no-zstd\\
&|'"

# One license.
expect fail "the metainfo with only the GPL" "$meta (project_license)" \
  "edit $meta 's|<project_license>.*</project_license>|<project_license>GPL-3.0-or-later</project_license>|'"
expect fail "the PKGBUILD with another license" "$arch (license)" \
  "edit $arch \"s|'MIT OR Apache-2.0'|'MIT'|\""
expect pass "a license added everywhere" \
  "edit $spec 's|^License:.*|& AND OFL-1.1|'
   edit $meta 's|</project_license>| AND OFL-1.1&|'
   edit $arch \"s|'MIT OR Apache-2.0')|'MIT OR Apache-2.0' 'OFL-1.1')|\""

exit $failed
