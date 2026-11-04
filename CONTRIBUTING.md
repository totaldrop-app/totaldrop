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

CI also measures line coverage and fails below 90%. To measure it yourself (needs `cargo install cargo-llvm-cov` and `rustup component add llvm-tools-preview`):

```bash
cargo llvm-cov --summary-only -p totaldrop-crypto -p totaldrop-proto -p totaldrop-core -p totaldrop-cli -p totaldrop-l10n -p totaldrop-rendezvous -p totaldrop-relay
```

If you changed the Android app's Dart code, also run the format check CI runs (it leaves out the generated `lib/l10n` and `lib/src/rust`):

```bash
cd app && find lib test tool -name '*.dart' -not -path 'lib/l10n/*' -not -path 'lib/src/rust/*' -print0 | xargs -0 dart format
```

## Coding standards

Each language follows one published style. The last column says what CI checks; where it says "review only", the reviewer checks the style by hand:

| Language | Style | Enforced by |
| --- | --- | --- |
| Rust (`crates/`, `server/`, `linux/`) | [The Rust Style Guide](https://doc.rust-lang.org/style-guide/) and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) | `cargo fmt --check` and `cargo clippy -- -D warnings`; `unsafe_code = "forbid"` in `crates/` and `server/` |
| Rust (`app/rust`, `fuzz/`) | The same | Review only, for now; `app/rust` contains generated bridge code |
| Dart (`app/`) | [Effective Dart](https://dart.dev/effective-dart) | `dart format --set-exit-if-changed`; `flutter analyze` with `flutter_lints` |
| TypeScript (`server/*-worker`) | The TypeScript compiler's `strict` mode | Types only: `tsc --noEmit`; no formatter yet |
| QML (`linux/qml`) | [Qt's QML coding conventions](https://doc.qt.io/qt-6/qml-codingconventions.html) | Review only, for now |
| C++ headers (`linux/src/bridge`) | [Qt's coding style](https://wiki.qt.io/Qt_Coding_Style) | Review only, for now |
| JavaScript (`site/`, `tools/`) | Plain modern JavaScript, no build step | Syntax only: `node --check` on the site's scripts; the tools run in CI |

Names, comments and commit messages are in English. Process and design documents in `docs/` are in Turkish; the protocol notes are in English.

## Code review

Every pull request is reviewed before it is merged, by someone who did not write it ([docs/surec.md](docs/surec.md), step 5). The reviewer reads the diff and the code around it, and checks that:

- it does what its issue asks, and nothing else;
- it is correct at the edges: empty, maximum and malformed input, errors, timeouts, concurrency;
- it keeps the security properties in the [threat model](docs/tehdit-modeli.md) and adds no new cryptography or `unsafe`;
- behaviour is covered by tests, a bug fix comes with a test that failed before it, and a new parser has a fuzz target;
- documents, the changelog entry and user-facing text match the change, and nothing claims more than the code does. User-facing text is written in all 35 languages.

A pull request is acceptable when CI is green, every finding is fixed or answered in writing on the pull request (with a follow-up issue where one is needed), and the maintainer approves the merge. The pull request records the review and what was done about each finding.

## Small tasks for new contributors

Issues labelled [good first issue](https://github.com/totaldrop-app/totaldrop/labels/good%20first%20issue) are small and self-contained, and say where to start. Translation reviews in your own language ([docs/ceviri.md](docs/ceviri.md)) are another good first step.

Which branch to use, how platforms are versioned and released, and the Linux packages: see [docs/gelistirme.md](docs/gelistirme.md). Interface changes follow [docs/tasarim.md](docs/tasarim.md).

Security problems go to [SECURITY.md](SECURITY.md), never to public issues. How decisions are made and who holds which role: [GOVERNANCE.md](GOVERNANCE.md).
