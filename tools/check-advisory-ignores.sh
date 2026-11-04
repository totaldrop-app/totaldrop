#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# cargo-deny (deny.toml) and OSV-Scanner (osv-scanner.toml, read by OpenSSF
# Scorecard) must ignore the same advisories, so that an advisory hidden
# from one is never silently hidden from only the other.
#
#   tools/check-advisory-ignores.sh
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
# Only the value of each id = "…" key, so an ID named in a reason or a
# comment does not count.
id_values() { sed -nE 's/^[[:space:]]*\{?[[:space:]]*id[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' | sort -u; }
deny=$(sed -n '/^\[advisories\]/,/^\[/p' "$root/deny.toml" | id_values)
osv=$(id_values < "$root/osv-scanner.toml")
if [ "$deny" != "$osv" ]; then
  echo "deny.toml and osv-scanner.toml ignore different advisories:" >&2
  echo "deny.toml:" >&2; echo "$deny" >&2
  echo "osv-scanner.toml:" >&2; echo "$osv" >&2
  exit 1
fi
echo "advisory ignores agree: $(echo "$deny" | tr '\n' ' ')"
