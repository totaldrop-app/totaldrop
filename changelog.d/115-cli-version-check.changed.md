- **Build:** A Linux release now checks the version of the command line it
  ships against every earlier Linux or `cli-v*` release. A version higher
  than all of them passes and a lower one stops the release. The same
  version stops it if the CLI's code has changed since a release that
  shipped it: `crates/` without its `tests/` and `benches/` directories,
  and the strings and translation status the CLI compiles in. `Cargo.lock`
  and the root `Cargo.toml` are not counted. The check is
  `packaging/check-cli-version.sh`, run by `check-versions.sh` on a
  release tag and tested in CI under bash and dash (#115).
