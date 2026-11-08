#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# cargo-deny (deny.toml) and OSV-Scanner (osv-scanner.toml, read by OpenSSF
# Scorecard) must ignore the same advisories, so that an advisory hidden
# from one is never silently hidden from only the other.
#
#   tools/check-advisory-ignores.sh [root]
#
# Both files are read as TOML (Python's tomllib), so every form cargo-deny
# accepts counts: bare IDs, inline tables, one line or many. An ignore by
# crate instead of by advisory has no OSV-Scanner equivalent and fails.
# The tests are in tools/test-advisory-ignores.sh.
set -eu
root=${1:-$(cd "$(dirname "$0")/.." && pwd)}
exec python3 -I - "$root" <<'EOF'
import pathlib, sys, tomllib

root = pathlib.Path(sys.argv[1])


def load(name):
    with open(root / name, "rb") as f:
        return tomllib.load(f)


deny = set()
for entry in load("deny.toml").get("advisories", {}).get("ignore", []):
    if isinstance(entry, str):
        deny.add(entry)
    elif isinstance(entry, dict) and isinstance(entry.get("id"), str):
        deny.add(entry["id"])
    else:
        sys.exit(f"deny.toml: ignore entry without an advisory id: {entry!r}")

osv = set()
for entry in load("osv-scanner.toml").get("IgnoredVulns", []):
    if not isinstance(entry.get("id"), str):
        sys.exit(f"osv-scanner.toml: IgnoredVulns entry without an id: {entry!r}")
    osv.add(entry["id"])

if deny != osv:
    print("deny.toml and osv-scanner.toml ignore different advisories:", file=sys.stderr)
    print("deny.toml:", " ".join(sorted(deny)) or "(none)", file=sys.stderr)
    print("osv-scanner.toml:", " ".join(sorted(osv)) or "(none)", file=sys.stderr)
    sys.exit(1)
print("advisory ignores agree:", " ".join(sorted(deny)) or "(none)")
EOF
