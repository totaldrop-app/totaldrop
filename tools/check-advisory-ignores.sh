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
# on one line or many. What the check cannot compare fails instead of
# passing:
#
# - a value of the wrong type, such as `ignore = ""`;
# - in deny.toml, a crate spec in `ignore` ("crate", "crate@1.0" or
#   { crate = … }), which ignores a yanked version rather than an
#   advisory, so handle it by hand; a table field but `id` and `reason`;
#   and `ignore-expiry`, which narrows cargo-deny's ignores only;
# - in osv-scanner.toml, any key but `IgnoredVulns` (for example
#   `PackageOverrides`, which can also hide advisories), any spelling of it
#   but that one (OSV-Scanner matches keys without regard to case), and any
#   field of an entry but `id`, `reason` and `ignoreUntil`;
# - an osv-scanner.toml anywhere but the root: OSV-Scanner reads the one
#   next to each lock file, so one in fuzz/ or app/ would hide advisories
#   from Scorecard alone.
#
# Not compared, because the difference is never silent (cargo-deny or
# Scorecard still reports): deny.toml settings that hide a whole class of
# advisories from cargo-deny only, such as `unmaintained = "none"`; an
# `ignoreUntil` that has passed, after which OSV-Scanner reports again; and
# aliases, which OSV-Scanner follows and cargo-deny does not, so an alias
# such as GHSA-… listed in both hides the advisory from OSV-Scanner only.
# Use the RUSTSEC ID.
# The tests are in tools/test-advisory-ignores.sh.
set -eu
root=${1:-$(cd "$(dirname "$0")/.." && pwd)}
exec python3 -I - "$root" <<'EOF'
import os, pathlib, subprocess, sys, tomllib

root = pathlib.Path(sys.argv[1])


def load(name):
    with open(root / name, "rb") as f:
        return tomllib.load(f)


def of_type(value, kind, what):
    if not isinstance(value, kind):
        sys.exit(f"{what}: expected {kind.__name__}, found {type(value).__name__}")
    return value


def advisory_id(text):
    # cargo-deny's own rule: an advisory ID starts with an all-uppercase
    # prefix and a hyphen (RUSTSEC-, GHSA-, CVE-); anything else is a crate.
    prefix, dash, _ = text.partition("-")
    return bool(dash) and prefix.isascii() and prefix.isupper() and "@" not in text


advisories = of_type(load("deny.toml").get("advisories", {}), dict, "deny.toml: [advisories]")
if "ignore-expiry" in advisories:
    sys.exit("deny.toml: ignore-expiry narrows cargo-deny's ignores only")
deny = set()
for entry in of_type(advisories.get("ignore", []), list, "deny.toml: ignore"):
    table = entry if isinstance(entry, dict) else None
    if table is not None:
        entry = table.get("id")
    if not isinstance(entry, str) or not advisory_id(entry):
        sys.exit(f"deny.toml: ignore entry that is not an advisory id: {table or entry!r}")
    fields = sorted(set(table or {}) - {"id", "reason"})
    if fields:
        sys.exit(f"deny.toml: {entry}: ignore fields this check does not know: {', '.join(fields)}")
    deny.add(entry)

config = load("osv-scanner.toml")
other = sorted(set(config) - {"IgnoredVulns"})
if other:
    sys.exit(f"osv-scanner.toml: keys this check does not compare: {', '.join(other)}")
osv = set()
for entry in of_type(config.get("IgnoredVulns", []), list, "osv-scanner.toml: IgnoredVulns"):
    if not isinstance(entry, dict) or not isinstance(entry.get("id"), str):
        sys.exit(f"osv-scanner.toml: IgnoredVulns entry without an id: {entry!r}")
    fields = sorted(set(entry) - {"id", "reason", "ignoreUntil"})
    if fields:
        sys.exit(f"osv-scanner.toml: {entry['id']}: fields this check does not know: {', '.join(fields)}")
    osv.add(entry["id"])

# Tracked files when root is a git checkout, every file otherwise (tests).
if (root / ".git").exists():
    files = subprocess.run(["git", "-C", str(root), "ls-files", "-z"], check=True,
                           capture_output=True).stdout.decode().split("\0")
else:
    files = [os.path.relpath(os.path.join(d, f), root)
             for d, dirs, names in os.walk(root) for f in names]
nested = sorted(f for f in files
                if os.path.basename(f).lower() == "osv-scanner.toml" and f != "osv-scanner.toml")
if nested:
    sys.exit(f"osv-scanner.toml files outside the root: {', '.join(nested)}")

if deny != osv:
    print("deny.toml and osv-scanner.toml ignore different advisories:", file=sys.stderr)
    print("deny.toml:", " ".join(sorted(deny)) or "(none)", file=sys.stderr)
    print("osv-scanner.toml:", " ".join(sorted(osv)) or "(none)", file=sys.stderr)
    sys.exit(1)
print("advisory ignores agree:", " ".join(sorted(deny)) or "(none)")
EOF
