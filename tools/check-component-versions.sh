#!/bin/sh
# Every component has its own version (docs/versioning.md). Check that the
# places naming one component's version agree, and that the servers and the
# Android app's Rust library do not inherit the workspace version shared by
# the CLI and the core libraries:
#
# - servers: server/totaldrop-rendezvous, server/totaldrop-relay and both
#   Workers' package.json name the same version;
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
json_version() {
  sed -n 's/^  "version":[[:space:]]*"\([^"]*\)".*/\1/p' "$1" | head -1
}
same() {
  [ "$2" = "$3" ] || fail "mismatch: $1 says '$2', expected '$3'"
}

server=$(cargo_version "$root/server/totaldrop-rendezvous/Cargo.toml")
case "$server" in
  "" | workspace) fail "server/totaldrop-rendezvous/Cargo.toml must name its own version, not the workspace's" ;;
esac
same server/totaldrop-relay/Cargo.toml "$(cargo_version "$root/server/totaldrop-relay/Cargo.toml")" "$server"
same server/rendezvous-worker/package.json "$(json_version "$root/server/rendezvous-worker/package.json")" "$server"
same server/relay-worker/package.json "$(json_version "$root/server/relay-worker/package.json")" "$server"

android=$(sed -n 's/^version:[[:space:]]*\([^+[:space:]]*\).*/\1/p' "$root/app/pubspec.yaml" | head -1)
[ -n "$android" ] || fail "no version in app/pubspec.yaml"
same app/rust/Cargo.toml "$(cargo_version "$root/app/rust/Cargo.toml")" "$android"

[ "$status" -eq 0 ] && echo "component versions ok: server $server, android $android"
exit "$status"
