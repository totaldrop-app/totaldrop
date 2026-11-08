#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Tests for the Node part of check-versions.sh (docs/surec.md, "Node"): each
# case copies the files the check reads, changes one thing and runs it. The
# copy sits under a path with a space, as a checkout might.
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
  server/rendezvous-worker/package.json server/relay-worker/package.json) |
  (cd "$base" && tar -xf -)

# edit FILE SED-SCRIPT: sed -i, the same under GNU and BSD sed.
edit() {
  sed "$2" "$1" > "$1.new"
  mv "$1.new" "$1"
}

# expect pass|fail NAME [COMMANDS run in the copy]
expect() {
  want=$1 name=$2
  shift 2
  rm -rf "$tmp/w"
  cp -R "$base" "$tmp/w"
  (cd "$tmp/w" && eval "${1:-:}")
  if sh "$tmp/w/packaging/check-versions.sh" > "$tmp/out" 2>&1; then got=pass; else got=fail; fi
  if [ "$got" = "$want" ]; then
    echo "ok   $name"
  else
    echo "FAIL $name: wanted $want, got $got"; sed 's/^/     /' "$tmp/out"; failed=1
  fi
}

wf=.github/workflows
types=server/rendezvous-worker/package.json
major=$(sed -n 's/^ *node-version: *"\([0-9]*\)".*/\1/p' "$root/$wf/ci.yml" | head -1)
[ -n "$major" ] || { echo "no node-version in $wf/ci.yml" >&2; exit 1; }
next=$((major + 2))

expect pass "the repository as it is"
expect fail "lts/* in one workflow" \
  "edit $wf/site.yml 's|node-version: \"$major\"|node-version: lts/*|'"
expect fail "a range (24.x)" \
  "edit $wf/site.yml 's|node-version: \"$major\"|node-version: $major.x|'"
expect fail "a bare *" \
  "edit $wf/site.yml 's|node-version: \"$major\"|node-version: \"*\"|'"
expect fail "another major in one workflow" \
  "edit $wf/deploy.yml 's|node-version: \"$major\"|node-version: \"$next\"|'"
expect pass "a full release of the same major, with a comment" \
  "edit $wf/deploy.yml 's|node-version: \"$major\"|node-version: $major.11.1 # pinned|'"
expect pass "single quotes" \
  "edit $wf/deploy.yml \"s|node-version: \\\"$major\\\"|node-version: '$major'|\""
expect fail "a setup-node step with no node-version" \
  "edit $wf/linux.yml '/node-version: \"$major\"/d'"
expect fail "node-version-file instead of node-version" \
  "edit $wf/ci.yml 's|node-version: \"$major\"|node-version-file: .nvmrc|'"
expect fail "@types/node a major ahead" \
  "edit $types 's|\"@types/node\": \"[^0-9\"]*$major\\.|\"@types/node\": \"^$next.|'"
expect pass "every node-version and @types/node upgraded together" \
  "for f in $wf/*.yml; do edit \"\$f\" 's|node-version: \"$major\"|node-version: \"$next\"|'; done
   edit $types 's|\"@types/node\": \"$major\\.|\"@types/node\": \"$next.|'"

exit $failed
