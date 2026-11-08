#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Tests for tools/check-advisory-ignores.sh: every form cargo-deny accepts
# for an ignored advisory must count, so that one hidden from cargo-deny
# but not from OSV-Scanner (or the other way round) fails the check, and
# anything the check cannot compare must fail with its own message.
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

# expect <name> <pass|fail> <deny.toml [advisories] body> <osv-scanner.toml> [stderr text]
# A failing case must print the given text, so that a crash or a different
# error does not count as the failure the case is about. An osv-scanner.toml
# of "-" means the file is missing.
expect() {
  printf '[advisories]\nversion = 2\n%s\n\n[licenses]\nversion = 2\n' "$3" > "$tmp/deny.toml"
  rm -f "$tmp/osv-scanner.toml"
  [ "$4" = - ] || printf '%s\n' "$4" > "$tmp/osv-scanner.toml"
  if sh "$check" "$tmp" > "$tmp/out" 2>&1; then got=pass; else got=fail; fi
  if [ "$got" != "$2" ]; then
    echo "FAIL: $1 (expected $2, got $got)" >&2
    sed 's/^/  /' "$tmp/out" >&2
    failed=1
  elif [ $# -ge 5 ] && ! grep -qF -- "$5" "$tmp/out"; then
    echo "FAIL: $1 (no \"$5\" in the output)" >&2
    sed 's/^/  /' "$tmp/out" >&2
    failed=1
  else
    echo "ok: $1"
  fi
}

differ='ignore different advisories'

# The same advisories, written in each form cargo-deny accepts.
expect "both empty" pass 'ignore = []' "$osv_none"
expect "no ignore key" pass '' "$osv_none"
expect "same ID, multi-line table" pass 'ignore = [
    { id = "RUSTSEC-2026-0001", reason = "test" },
]' "$osv_one"
expect "same ID, bare string" pass 'ignore = ["RUSTSEC-2026-0001"]' "$osv_one"
expect "IDs in a reason or a comment do not count" pass \
  '# RUSTSEC-2026-0002
ignore = [{ id = "RUSTSEC-2026-0001", reason = "not RUSTSEC-2026-0002" }]' "$osv_one"
expect "IgnoredVulns with ignoreUntil" pass 'ignore = ["RUSTSEC-2026-0001"]' \
  "$osv_one
ignoreUntil = 2027-01-01"

# An advisory ignored by one file only.
expect "bare string only in deny.toml" fail 'ignore = ["RUSTSEC-2026-0001"]' "$osv_none" "$differ"
expect "multi-line bare strings only in deny.toml" fail 'ignore = [
    "RUSTSEC-2026-0001",
]' "$osv_none" "$differ"
expect "one-line inline table only in deny.toml" fail \
  'ignore = [{ id = "RUSTSEC-2026-0001", reason = "test" }]' "$osv_none" "$differ"
expect "only in osv-scanner.toml" fail 'ignore = []' "$osv_one" "$differ"

# What the check cannot compare.
expect "crate spec as a table" fail 'ignore = [{ crate = "zvariant", reason = "test" }]' \
  "$osv_none" "not an advisory id"
expect "crate spec as a string" fail 'ignore = ["zvariant@5.0.0"]' "$osv_none" "not an advisory id"
expect "PackageOverrides in osv-scanner.toml" fail 'ignore = []' '[[PackageOverrides]]
name = "zvariant"
ecosystem = "crates.io"
vulnerability.ignore = true' "keys this check does not compare: PackageOverrides"
expect "IgnoredVulns in another case" fail 'ignore = []' '[[ignoredvulns]]
id = "RUSTSEC-2026-0001"' "keys this check does not compare: ignoredvulns"
expect "unknown field in an IgnoredVulns entry" fail 'ignore = ["RUSTSEC-2026-0001"]' \
  "$osv_one
aliases = true" "fields this check does not know: aliases"
expect "IgnoredVulns entry without an id" fail 'ignore = []' '[[IgnoredVulns]]
reason = "test"' "IgnoredVulns entry without an id"

# Broken input fails instead of passing.
expect "invalid TOML" fail 'ignore = [' "$osv_none" "TOMLDecodeError"
expect "missing osv-scanner.toml" fail 'ignore = []' - "FileNotFoundError"

exit $failed
