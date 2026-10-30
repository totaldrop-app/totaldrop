#!/bin/sh
# Tests for check-cli-version.sh, each in a throwaway git repository, and
# for check-versions.sh calling it on a release tag.
#
#   packaging/test-check-cli-version.sh
set -eu
# The tests must not depend on, or sign with, the caller's git settings,
# nor act on the caller's repository.
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_OBJECT_DIRECTORY GIT_COMMON_DIR
GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
GIT_AUTHOR_NAME=test GIT_AUTHOR_EMAIL=test@example.invalid
GIT_COMMITTER_NAME=test GIT_COMMITTER_EMAIL=test@example.invalid
export GIT_CONFIG_GLOBAL GIT_CONFIG_NOSYSTEM GIT_AUTHOR_NAME GIT_AUTHOR_EMAIL \
  GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL
here=$(cd "$(dirname "$0")" && pwd)
src=$(cd "$here/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
failed=0
n=0
edits=0

# repo VERSION: a new repository with a workspace version and some CLI code.
repo() {
  n=$((n + 1))
  r="$tmp/r$n"
  mkdir -p "$r/packaging" "$r/crates/totaldrop-cli/src" "$r/linux" \
    "$r/l10n/app" "$r/l10n/cli"
  cp "$here/check-cli-version.sh" "$here/check-versions.sh" "$r/packaging/"
  git -C "$r" init -q -b main
  set_version "$1"
  echo 'fn main() {}' > "$r/crates/totaldrop-cli/src/main.rs"
  echo '{}' > "$r/l10n/app/en.arb"
  echo '{}' > "$r/l10n/cli/en.arb"
  echo '{}' > "$r/l10n/status.json"
  echo 'fn main() {}' > "$r/linux/main.rs"
  commit "start"
}
set_version() {
  printf '[workspace]\nmembers = []\n\n[workspace.package]\nversion = "%s"\nrust-version = "1.99"\n' "$1" > "$r/Cargo.toml"
}
commit() {
  git -C "$r" add -A
  git -C "$r" commit -q -m "$1"
}
# change FILE: a commit that changes FILE.
change() {
  mkdir -p "$(dirname "$r/$1")"
  edits=$((edits + 1))
  echo "// $edits" >> "$r/$1"
  commit "change $1"
}
# bump VERSION: a commit that sets the workspace version.
bump() {
  set_version "$1"
  commit "version $1"
}
tag() {
  git -C "$r" tag -a -m "$1" "$1"
}
# release TAG: tag HEAD, then a commit outside the CLI, as the next release
# would have.
release() {
  tag "$1"
  change linux/main.rs
}
# expect pass DESCRIPTION [ARGS...]
# expect fail DESCRIPTION MESSAGE [ARGS...]: fails, saying MESSAGE.
expect() {
  want=$1 what=$2
  shift 2
  if [ "$want" = fail ]; then
    msg=$1
    shift
  else
    msg=
  fi
  check "$want" "$what" "$msg" "$r/packaging/check-cli-version.sh" "$@"
}
check() {
  want=$1 what=$2 msg=$3
  shift 3
  if out=$("$@" 2>&1 </dev/null); then got=pass; else got=fail; fi
  if [ "$got" != "$want" ]; then
    echo "FAILED: $what: expected $want, got $got" >&2
  elif [ -n "$msg" ] && ! printf '%s\n' "$out" | grep -qF -- "$msg"; then
    echo "FAILED: $what: the output does not say '$msg'" >&2
  else
    echo "ok: $what"
    return 0
  fi
  printf '%s\n' "$out" | sed 's/^/  | /' >&2
  failed=1
}

repo 0.1.0-beta.2
change crates/totaldrop-cli/src/main.rs
expect fail "no release tag at all" "no linux-v* or cli-v* tag before"
expect fail "a commit with no parent" "has no parent" HEAD~1
expect fail "unknown revision" "no commit 'no-such-rev'" no-such-rev

repo 0.1.0-beta.2
release linux-v0.1.0-beta.1
expect pass "released, nothing in the CLI changed"
change crates/totaldrop-cli/tests/end_to_end.rs
change crates/totaldrop-core/benches/speed.rs
expect pass "released, only tests/ and benches/ changed"
change crates/totaldrop-cli/src/main.rs
expect fail "released, then the CLI changed" \
  "CLI version 0.1.0-beta.2 shipped in linux-v0.1.0-beta.1"
expect pass "checking the commit before the change" HEAD~1
bump 0.2.0-beta.1
expect pass "released, changed, then the version raised"
change crates/totaldrop-core/src/lib.rs
expect pass "raised ahead of time, then more changes"

for f in l10n/status.json l10n/app/en.arb l10n/cli/en.arb crates/totaldrop-l10n/build.rs; do
  repo 0.1.0-beta.2
  release linux-v0.1.0-beta.1
  change "$f"
  expect fail "released, then $f changed" "  $f"
done

repo 0.1.0-beta.2
git -C "$r" tag linux-v0.1.0-beta.1
change linux/main.rs
change crates/totaldrop-cli/src/main.rs
expect fail "a lightweight tag is a release too" "shipped in linux-v0.1.0-beta.1"

# old new: a downgrade, or a version not of the beta form.
while read -r old new msg; do
  repo "$old"
  release linux-v0.1.0-beta.1
  bump "$new"
  expect fail "$old -> $new" "$msg"
done <<EOF
0.2.0-beta.1 0.1.9-beta.9 is lower than 0.2.0-beta.1
0.1.0-beta.10 0.1.0-beta.9 is lower than 0.1.0-beta.10
0.1.1-beta.1 0.1.0-beta.5 is lower than 0.1.1-beta.1
0.1.0-beta.2 0.1.0 is not of the form
0.1.0-beta.2 0.2.0-rc.1 is not of the form
0.1.0-beta.2 0.02.0-beta.1 is not of the form
0.1.0-beta.2 0.1.0-beta.0 is not of the form
EOF
while read -r old new; do
  repo "$old"
  release linux-v0.1.0-beta.1
  bump "$new"
  expect pass "$old -> $new"
done <<EOF
0.1.0-beta.9 0.1.0-beta.10
0.1.0-beta.5 0.1.1-beta.1
0.9.0-beta.1 0.10.0-beta.1
0.1.9-beta.9 0.2.0-beta.1
EOF

repo 0.1.0
release linux-v0.1.0-beta.1
bump 0.2.0-beta.1
expect fail "a malformed version at a release tag" "version '0.1.0' at linux-v0.1.0-beta.1"

repo 0.1.0-beta.2
printf '[workspace]\nmembers = []\n' > "$r/Cargo.toml"
commit "no version"
release linux-v0.1.0-beta.1
bump 0.2.0-beta.1
expect fail "no version at a release tag" "no [workspace.package] version in Cargo.toml at linux-v0.1.0-beta.1"

repo 0.1.0-beta.2
release linux-v0.3.0-beta.1
bump 0.2.0-beta.1
expect pass "the version comes from Cargo.toml at the tag, not the tag's name"

repo 0.1.0-beta.2
release linux-v0.1.0-beta.1
bump 0.2.0-beta.1
release cli-v0.2.0-beta.1
change crates/totaldrop-cli/src/main.rs
expect fail "a cli-v* tag is a release too" "shipped in cli-v0.2.0-beta.1"

repo 0.1.0-beta.2
release linux-v0.1.0-beta.1
git -C "$r" checkout -q -b side
bump 0.2.0-beta.1
release linux-v0.2.0-beta.1
git -C "$r" checkout -q main
change linux/main.rs
# Were the side branch's tag used, 0.1.0-beta.2 would be a downgrade.
expect pass "a tag on a branch not merged does not count"

# The highest release counts, not the nearest tag: an older release on a
# side branch merged later is closer to HEAD.
repo 0.1.0-beta.2
git -C "$r" branch side
bump 0.2.0-beta.1
release linux-v0.3.0-beta.1
git -C "$r" checkout -q side
change linux/side.rs
tag linux-v0.2.0-beta.9
git -C "$r" checkout -q main
git -C "$r" merge -q --no-edit side
change crates/totaldrop-cli/src/main.rs
expect fail "the highest release counts, not the nearest" \
  "CLI version 0.2.0-beta.1 shipped in linux-v0.3.0-beta.1"

# Several releases with the same version: the code must match every one.
repo 0.1.0-beta.2
release linux-v0.1.0-beta.1
change crates/totaldrop-cli/src/main.rs
release linux-v0.1.0-beta.2
expect fail "the same version shipped with different code" \
  "shipped in linux-v0.1.0-beta.1"

# A shallow clone that would pass with the full history.
repo 0.1.0-beta.2
change linux/main.rs
release linux-v0.1.0-beta.1
expect pass "the history the shallow clone is cut from"
git clone -q --depth 2 "file://$r" "$tmp/shallow"
r="$tmp/shallow"
git -C "$r" rev-parse -q --verify linux-v0.1.0-beta.1 >/dev/null ||
  { echo "FAILED: the shallow clone lacks the release tag" >&2; failed=1; }
expect fail "shallow clone" "shallow clone"

# check-versions.sh on a release tag: the tag on the commit being released
# is skipped, so the release before it is the baseline. The packaging files
# check-versions.sh reads come from this checkout.
repo 0.1.0-beta.2
for f in linux/Cargo.toml rust-toolchain.toml packaging/build.sh \
  packaging/arch/PKGBUILD packaging/rpm/totaldrop.spec \
  packaging/debian/changelog packaging/containers \
  packaging/linux/app.totaldrop.TotalDrop.metainfo.xml .github/workflows; do
  mkdir -p "$(dirname "$r/$f")"
  cp -R "$src/$f" "$r/$f"
done
commit "packaging"
linux=$(sed -n 's/^version = "\(.*\)"/\1/p' "$r/linux/Cargo.toml" | head -1)
release linux-v0.0.1-beta.1
change crates/totaldrop-cli/src/main.rs
tag "linux-v$linux"
check fail "check-versions.sh: release with a changed CLI and the same version" \
  "shipped in linux-v0.0.1-beta.1" "$r/packaging/check-versions.sh" "linux-v$linux"
git -C "$r" tag -d "linux-v$linux" >/dev/null
bump 0.2.0-beta.1
tag "linux-v$linux"
check pass "check-versions.sh: release with the CLI version raised" "" \
  "$r/packaging/check-versions.sh" "linux-v$linux"

exit $failed
