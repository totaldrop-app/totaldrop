#!/bin/sh
# Every component has its own version (docs/versioning.md). Check that the
# places naming one component's version agree, and that the servers and the
# Android app's Rust library do not inherit the workspace version shared by
# the CLI and the core libraries:
#
# - servers: server/totaldrop-rendezvous, server/totaldrop-relay, both
#   Workers' package.json and package-lock.json, and the CLI's
#   dev-dependencies on the two Rust servers name the same version;
# - Android: app/rust/Cargo.toml names app/pubspec.yaml's version without
#   the build number.
#
# The Linux app and its packages are checked by packaging/check-versions.sh.
#
#   tools/check-component-versions.sh
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
status=0
fail() {
  echo "$1" >&2
  status=1
}
# The version line of a Cargo.toml's [package] table, or "workspace".
cargo_version() {
  awk '
    /^\[/ { in_pkg = ($0 == "[package]") }
    in_pkg && /^version\.workspace[[:space:]]*=[[:space:]]*true/ { print "workspace"; exit }
    in_pkg && /^version[[:space:]]*=/ { sub(/^version[[:space:]]*=[[:space:]]*"/, ""); sub(/".*/, ""); print; exit }
  ' "$1"
}
# The first "version" at the top level of a package.json or
# package-lock.json (two spaces in, as npm writes them), and the root
# package's in a package-lock.json (six spaces in).
json_version() {
  sed -n 's/^  "version":[[:space:]]*"\([^"]*\)".*/\1/p' "$1" | head -1
}
lock_root_version() {
  sed -n 's/^      "version":[[:space:]]*"\([^"]*\)".*/\1/p' "$1" | head -1
}
# The version a dependency line in a Cargo.toml asks for.
dep_version() {
  sed -n "s/^$2[[:space:]]*=.*version[[:space:]]*=[[:space:]]*\"\([^\"]*\)\".*/\\1/p" "$1" | head -1
}
same() {
  [ "$2" = "$3" ] || fail "mismatch: $1 says '$2', expected '$3'"
}

server=$(cargo_version "$root/server/totaldrop-rendezvous/Cargo.toml")
case "$server" in
  "" | workspace)
    fail "server/totaldrop-rendezvous/Cargo.toml must name its own version, not the workspace's"
    exit 1
    ;;
esac
same server/totaldrop-relay/Cargo.toml "$(cargo_version "$root/server/totaldrop-relay/Cargo.toml")" "$server"
for worker in rendezvous-worker relay-worker; do
  same "server/$worker/package.json" "$(json_version "$root/server/$worker/package.json")" "$server"
  same "server/$worker/package-lock.json" "$(json_version "$root/server/$worker/package-lock.json")" "$server"
  same "server/$worker/package-lock.json (root package)" \
    "$(lock_root_version "$root/server/$worker/package-lock.json")" "$server"
done
for crate in totaldrop-rendezvous totaldrop-relay; do
  same "crates/totaldrop-cli/Cargo.toml ($crate)" \
    "$(dep_version "$root/crates/totaldrop-cli/Cargo.toml" "$crate")" "$server"
done

android=$(sed -n 's/^version:[[:space:]]*\([^+[:space:]]*\).*/\1/p' "$root/app/pubspec.yaml" | head -1)
[ -n "$android" ] || fail "no version in app/pubspec.yaml"
same app/rust/Cargo.toml "$(cargo_version "$root/app/rust/Cargo.toml")" "$android"

[ "$status" -eq 0 ] && echo "component versions ok: server $server, android $android"
exit "$status"
