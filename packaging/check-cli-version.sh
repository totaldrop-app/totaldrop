#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

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
# Every 0.x version has the beta form 0.MINOR.PATCH-beta.N; from 1.0.0 full
# SemVer 2.0 (MAJOR.MINOR.PATCH and -{alpha,beta,rc}.N) is ordered too
# (docs/kararlar/0026-automated-component-versioning.md). Needs the full
# history and the tags (actions/checkout with fetch-depth: 0).
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
# "MAJOR MINOR PATCH RANK PRE" of a valid version (RANK: alpha=1, beta=2,
# rc=3, final=4), or nothing. Before 1.0 only 0.MINOR.PATCH-beta.N is valid.
parts() {
  if printf '%s\n' "$1" | grep -Eq '^0\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)-beta\.[1-9][0-9]*$'; then
    rest=${1#0.}
    minor=${rest%%.*}
    rest=${rest#*.}
    echo "0 $minor ${rest%%-*} 2 ${rest##*.}"
    return 0
  fi
  printf '%s\n' "$1" |
    grep -Eq '^[1-9][0-9]*\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-(alpha|beta|rc)\.[1-9][0-9]*)?$' || return 0
  base=${1%%-*}
  major=${base%%.*}
  rest=${base#*.}
  minor=${rest%%.*}
  patch=${rest#*.}
  if [ "$base" = "$1" ]; then
    echo "$major $minor $patch 4 0"
    return 0
  fi
  suf=${1#*-}
  chan=${suf%%.*}
  pre=${suf#*.}
  case $chan in
    alpha) rank=1 ;;
    beta) rank=2 ;;
    rc) rank=3 ;;
    *) return 0 ;;
  esac
  echo "$major $minor $patch $rank $pre"
}
# Whether $1 > $2, both "MAJOR MINOR PATCH RANK PRE".
newer() {
  # shellcheck disable=SC2086
  set -- $1 $2
  [ "$1" -gt "$6" ] && return 0
  [ "$1" -lt "$6" ] && return 1
  [ "$2" -gt "$7" ] && return 0
  [ "$2" -lt "$7" ] && return 1
  [ "$3" -gt "$8" ] && return 0
  [ "$3" -lt "$8" ] && return 1
  [ "$4" -gt "$9" ] && return 0
  [ "$4" -lt "$9" ] && return 1
  [ "$5" -gt "${10}" ]
}
# The checked version's parts at REV, or at a release tag; fails closed.
checked_parts() {
  ver=$(version_at "$1")
  [ -n "$ver" ] || err "no [workspace.package] version in Cargo.toml at $1"
  p=$(parts "$ver")
  [ -n "$p" ] || err "version '$ver' at $1 is not of the form 0.MINOR.PATCH-beta.N or MAJOR.MINOR.PATCH[-{alpha,beta,rc}.N]"
  echo "$p"
}

version=$(version_at "$rev")
v=$(checked_parts "$rev") || exit 1

# The releases before REV across all merge parents; a tag on REV itself is the
# one being made.
git rev-parse --verify -q "$rev^" >/dev/null || err "'$rev' has no parent, so no release before it"
rev_commit=$(git rev-parse "$rev^{commit}")
tags=
for cand in $(git tag --merged "$rev_commit" -l 'linux-v*' 'cli-v*' 'core-v*'); do
  [ "$(git rev-parse "$cand^{commit}")" = "$rev_commit" ] && continue
  tags="$tags $cand"
done
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
