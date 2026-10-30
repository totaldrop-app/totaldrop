# Contributing

Thank you for helping. A few rules keep a security project safe:

1. **The specification comes first.** A change to protocol behaviour starts as a proposal against the v0.1 draft, not as code.
2. **No new cryptography.** Use only the primitives listed in the specification, through audited libraries. No hand-written cryptographic code.
3. **No `unsafe`.** The workspace forbids it. The one exception is `linux/`, whose CXX-Qt bridge to Qt is `unsafe extern "C++"` by nature; keep `unsafe` there to the bridge declarations and to setting environment variables before Qt starts.
4. **Every parser gets a fuzz target**, and every protocol rule gets a test.
5. **Every pull request must pass CI**: formatting, Clippy with warnings as errors, tests on Linux, macOS and Windows, and `cargo deny`.
6. **New dependencies need a reason.** Say in the pull request why it is needed, who maintains it, and what it pulls in.
7. **Every pull request records its change** in a file of its own under [changelog.d/](changelog.d/README.md), never in CHANGELOG.md. CI fails a pull request without one.

Before opening a pull request:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
node tools/changelog.mjs check
```

If you changed the Android app's Dart code, also run the format check CI runs (it leaves out the generated `lib/l10n` and `lib/src/rust`):

```bash
cd app && find lib test tool -name '*.dart' -not -path 'lib/l10n/*' -not -path 'lib/src/rust/*' -print0 | xargs -0 dart format
```

Which branch to use, how platforms are versioned and released, and the Linux packages: see [docs/gelistirme.md](docs/gelistirme.md). Interface changes follow [docs/tasarim.md](docs/tasarim.md).

Security problems go to [SECURITY.md](SECURITY.md), never to public issues.
