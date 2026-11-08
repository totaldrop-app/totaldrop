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
# accepts for an advisory ignore counts: bare IDs or tables with an `id`,
# on one line or many. Anything the check cannot compare fails instead of
# passing:
#
# - in deny.toml, a crate spec in `ignore` ("crate@1.0" or { crate = … }),
#   which ignores a yanked version rather than an advisory; handle such a
#   case by hand;
# - in osv-scanner.toml, any key but `IgnoredVulns` (for example
#   `PackageOverrides`, which can also hide advisories), any spelling of it
#   but that one (OSV-Scanner matches keys without regard to case), and any
#   field of an entry but `id`, `reason` and `ignoreUntil`.
#
# Not compared: deny.toml settings that hide a whole class of advisories
# from cargo-deny only, such as `unmaintained = "none"` or `[graph]
# exclude`. They hide nothing from OSV-Scanner, so Scorecard still reports.
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
    if isinstance(entry, dict):
        entry = entry.get("id")
    if not isinstance(entry, str) or "@" in entry:
        sys.exit(f"deny.toml: ignore entry that is not an advisory id: {entry!r}")
    deny.add(entry)

config = load("osv-scanner.toml")
other = sorted(set(config) - {"IgnoredVulns"})
if other:
    sys.exit(f"osv-scanner.toml: keys this check does not compare: {', '.join(other)}")
osv = set()
for entry in config.get("IgnoredVulns", []):
    if not isinstance(entry, dict) or not isinstance(entry.get("id"), str):
        sys.exit(f"osv-scanner.toml: IgnoredVulns entry without an id: {entry!r}")
    fields = sorted(set(entry) - {"id", "reason", "ignoreUntil"})
    if fields:
        sys.exit(f"osv-scanner.toml: {entry['id']}: fields this check does not know: {', '.join(fields)}")
    osv.add(entry["id"])

if deny != osv:
    print("deny.toml and osv-scanner.toml ignore different advisories:", file=sys.stderr)
    print("deny.toml:", " ".join(sorted(deny)) or "(none)", file=sys.stderr)
    print("osv-scanner.toml:", " ".join(sorted(osv)) or "(none)", file=sys.stderr)
    sys.exit(1)
print("advisory ignores agree:", " ".join(sorted(deny)) or "(none)")
EOF
