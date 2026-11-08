#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Tests for the Node part of check-versions.sh (docs/surec.md, "Node"): each
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

exit $failed
