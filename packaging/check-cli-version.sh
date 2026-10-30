#!/bin/sh
# Check that the command line's version names the code it is built from.
#
# The CLI and the core libraries take their version from [workspace.package]
# in Cargo.toml (docs/versioning.md). The Linux packages are the CLI's
# release channel, so every linux-v* tag (and a cli-v* tag, should there be
# one) before REV is a release of the version in Cargo.toml at that tag.
# Compared with the highest version released so far:
#
# - a higher version passes: it has not shipped yet;
# - a lower version fails;
# - the same version fails if the code that goes into the CLI differs from
#   any release that shipped it.
#
# The code that goes into the CLI: crates/ without the tests/ and benches/
# directories (unit tests inside src/ count), and l10n/cli,
# l10n/status.json and l10n/app, which totaldrop-l10n compiles in; a CLI
# string missing from l10n/cli falls back to the app's.
#
# Not counted: Cargo.lock and the root Cargo.toml, that is dependency
# versions, lints and the build profile. Counting the lock file would make
# every Linux release, whose own dependencies move, raise the CLI version
# too. A dependency update that matters to the CLI, a security fix say,
# still calls for a new CLI version; that is the releaser's call.
#
# Every version must have the beta form 0.MINOR.PATCH-beta.N; from 1.0 this
# check has to learn the final form. Needs the full history and the tags
# (actions/checkout with fetch-depth: 0).
#
#   packaging/check-cli-version.sh [REV]     (default: HEAD)
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
rev=${1:-HEAD}
paths="crates l10n/app l10n/cli l10n/status.json :(exclude,glob)crates/*/tests/** :(exclude,glob)crates/*/benches/**"
err() {
  echo "check-cli-version: $1" >&2
  exit 1
}

cd "$root"
[ "$(git rev-parse --is-shallow-repository)" = false ] || err "shallow clone; fetch the full history and tags"
git rev-parse --verify -q "$rev^{commit}" >/dev/null || err "no commit '$rev'"

# The [workspace.package] version at a revision.
version_at() {
  git show "$1:Cargo.toml" 2>/dev/null |
    sed -n '/^\[workspace\.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' | head -1
}
# "MINOR PATCH N" of a 0.MINOR.PATCH-beta.N version, or nothing.
parts() {
  printf '%s\n' "$1" | grep -Eq '^0\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)-beta\.[1-9][0-9]*$' || return 0
  rest=${1#0.}
  minor=${rest%%.*}
  rest=${rest#*.}
  echo "$minor ${rest%%-*} ${rest##*.}"
}
# Whether $1 > $2, both "MINOR PATCH N".
newer() {
  # shellcheck disable=SC2086
  set -- $1 $2
  [ "$1" -gt "$4" ] && return 0
  [ "$1" -lt "$4" ] && return 1
  [ "$2" -gt "$5" ] && return 0
  [ "$2" -lt "$5" ] && return 1
  [ "$3" -gt "$6" ]
}
# The checked version's parts at REV, or at a release tag; fails closed.
checked_parts() {
  ver=$(version_at "$1")
  [ -n "$ver" ] || err "no [workspace.package] version in Cargo.toml at $1"
  p=$(parts "$ver")
  [ -n "$p" ] || err "version '$ver' at $1 is not of the form 0.MINOR.PATCH-beta.N"
  echo "$p"
}

version=$(version_at "$rev")
v=$(checked_parts "$rev") || exit 1

# The releases before REV; a tag on REV itself is the one being made.
git rev-parse --verify -q "$rev^" >/dev/null || err "'$rev' has no parent, so no release before it"
tags=$(git tag --merged "$rev^" -l 'linux-v*' 'cli-v*')
[ -n "$tags" ] || err "no linux-v* or cli-v* tag before '$rev'"

top="" top_ver="" top_tag=""
for t in $tags; do
  p=$(checked_parts "$t") || exit 1
  if [ -z "$top" ] || newer "$p" "$top"; then
    top=$p top_ver=$(version_at "$t") top_tag=$t
  fi
done

if [ "$v" != "$top" ]; then
  if newer "$v" "$top"; then
    echo "CLI version $version is new (the highest released is $top_ver, in $top_tag)"
    exit 0
  fi
  err "CLI version $version is lower than $top_ver, released in $top_tag"
fi

# The same version: every release that shipped it must have this code.
for t in $tags; do
  [ "$(checked_parts "$t")" = "$v" ] || continue
  # $paths is split into its pathspecs on purpose; git, not the shell,
  # expands their patterns.
  set -f
  # shellcheck disable=SC2086
  changed=$(git -c core.quotePath=false diff --name-only "$t" "$rev" -- $paths)
  set +f
  [ -n "$changed" ] || continue
  {
    echo "CLI version $version shipped in $t, but the CLI's code changed since:"
    printf '%s\n' "$changed" | head -20 | sed 's/^/  /'
    [ "$(printf '%s\n' "$changed" | wc -l)" -le 20 ] || echo "  ..."
    echo "Raise [workspace.package] version in Cargo.toml (docs/versioning.md)."
  } >&2
  exit 1
done
echo "CLI version $version is the one released in $top_tag, and its code is unchanged"
