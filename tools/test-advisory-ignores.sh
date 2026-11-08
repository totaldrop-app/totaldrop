#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Tests for tools/check-advisory-ignores.sh: every form cargo-deny accepts
# for an ignored advisory must count, so that one hidden from cargo-deny
# but not from OSV-Scanner (or the other way round) fails the check.
#
#   tools/test-advisory-ignores.sh
set -eu
check="$(cd "$(dirname "$0")" && pwd)/check-advisory-ignores.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
failed=0

osv_none='# no ignores'
osv_one='[[IgnoredVulns]]
id = "RUSTSEC-2026-0001"
reason = "test"'

# case <name> <expected: pass|fail> <deny.toml [advisories] body> <osv-scanner.toml>
expect() {
  printf '[advisories]\nversion = 2\n%s\n\n[licenses]\nversion = 2\n' "$3" > "$tmp/deny.toml"
  printf '%s\n' "$4" > "$tmp/osv-scanner.toml"
  if sh "$check" "$tmp" > /dev/null 2>&1; then got=pass; else got=fail; fi
  if [ "$got" = "$2" ]; then
    echo "ok: $1"
  else
    echo "FAIL: $1 (expected $2, got $got)" >&2
    failed=1
  fi
}

expect "both empty" pass 'ignore = []' "$osv_none"
expect "no ignore key" pass '' "$osv_none"
expect "same ID, multi-line table" pass 'ignore = [
    { id = "RUSTSEC-2026-0001", reason = "test" },
]' "$osv_one"
expect "same ID, bare string" pass 'ignore = ["RUSTSEC-2026-0001"]' "$osv_one"
expect "bare string only in deny.toml" fail 'ignore = ["RUSTSEC-2026-0001"]' "$osv_none"
expect "multi-line bare strings only in deny.toml" fail 'ignore = [
    "RUSTSEC-2026-0001",
]' "$osv_none"
expect "one-line inline table only in deny.toml" fail \
  'ignore = [{ id = "RUSTSEC-2026-0001", reason = "test" }]' "$osv_none"
expect "only in osv-scanner.toml" fail 'ignore = []' "$osv_one"
expect "ID only in a reason does not count" fail \
  'ignore = [{ id = "RUSTSEC-2026-0002", reason = "not RUSTSEC-2026-0001" }]' "$osv_one"
expect "ignore by crate has no OSV equivalent" fail \
  'ignore = [{ crate = "zvariant", reason = "test" }]' "$osv_none"

exit $failed
