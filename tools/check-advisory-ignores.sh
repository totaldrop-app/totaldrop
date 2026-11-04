#!/bin/sh
# cargo-deny (deny.toml) and OSV-Scanner (osv-scanner.toml, read by OpenSSF
# Scorecard) must ignore the same advisories, so that an advisory hidden
# from one is never silently hidden from only the other.
#
#   tools/check-advisory-ignores.sh
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
deny=$(sed -n '/^\[advisories\]/,/^\[/p' "$root/deny.toml" | grep -E '^\s*\{ *id *=' | grep -oE '(RUSTSEC|GHSA|CVE)-[0-9A-Za-z-]+' | sort -u)
osv=$(grep -E '^\s*id *=' "$root/osv-scanner.toml" | grep -oE '(RUSTSEC|GHSA|CVE)-[0-9A-Za-z-]+' | sort -u)
if [ "$deny" != "$osv" ]; then
  echo "deny.toml and osv-scanner.toml ignore different advisories:" >&2
  echo "deny.toml:" >&2; echo "$deny" >&2
  echo "osv-scanner.toml:" >&2; echo "$osv" >&2
  exit 1
fi
echo "advisory ignores agree: $(echo "$deny" | tr '\n' ' ')"
