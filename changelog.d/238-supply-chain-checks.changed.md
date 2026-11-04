- **Build:** The Linux build images check rustup-init against a fixed
  sha256 before running it, instead of piping an install script to the
  shell (#238).
- **CI:** OSV-Scanner, which OpenSSF Scorecard uses, ignores the same
  three unmaintained-crate advisories as cargo-deny, and CI checks that
  the two lists agree (#238).
