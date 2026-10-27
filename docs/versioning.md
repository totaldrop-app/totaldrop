# Versioning and releases

## Version numbers

TotalDrop follows [Semantic Versioning 2.0](https://semver.org/).

- One version number covers the whole project: the Rust crates, the apps
  and the servers are released together and carry the same number.
- **Beta until 1.0.** Every release is a pre-release, `0.MINOR.PATCH-beta.N`,
  until all five platforms (Linux, Windows, macOS, Android, iOS) ship and
  the independent security review is complete. Only then is `1.0.0`
  released. Apps show "beta" next to the version.
- During beta, `MINOR` goes up for new features or breaking changes, `PATCH`
  for fixes, and `beta.N` for each build of the same version.
- The **protocol** has its own version (`v0`, then `v0.1` drafts) because
  devices negotiate it with each other (spec section 11). An app version
  says which protocol versions it speaks in its release notes.

## Build numbers

- Android `versionCode` and iOS/macOS build numbers are one integer that
  goes up by one with every published build, across all versions. It lives
  after the `+` in `app/pubspec.yaml` (for example `0.1.0-beta.1+1`).
- A build number is never reused, so an older package can never install
  over a newer one.

## Git

- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/):
  `feat:`, `fix:`, `security:`, `docs:`, `refactor:`, `test:`, `build:`,
  `ci:`, `chore:`. A breaking change adds `!` (`feat!:`) or a
  `BREAKING CHANGE:` footer.
- Every release is a signed annotated tag `vX.Y.Z[-beta.N]` on `main`.
- [`CHANGELOG.md`](../CHANGELOG.md) follows [Keep a Changelog](https://keepachangelog.com/).
  Each release's notes are taken from it; security fixes are listed under
  "Security".

## Making a release

1. Update the version in `Cargo.toml` (`[workspace.package]`), in the
   `version` of every internal path dependency, and in `app/pubspec.yaml`,
   raising the build number.
2. Move the "Unreleased" changelog entries under the new version and date.
3. Commit as `chore(release): vX.Y.Z-beta.N`, tag, push.
4. CI builds every package from the tag. Nothing published is built on a
   personal machine once CI release builds exist.
