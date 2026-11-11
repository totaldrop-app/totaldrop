<!-- Title in Conventional Commits form: feat(linux): ..., fix(core): ... -->

## What changed

Closes #

## Platform

- [ ] Core (crates/)
- [ ] Linux (linux/, packaging/)
- [ ] Android (app/)
- [ ] Server (server/)
- [ ] Site (site/)
- [ ] Docs

## Definition of done (docs/surec.md)

- [ ] Tests exercise the behaviour; a bug fix starts with a test that shows the bug
- [ ] `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` and the platform checks are clean
- [ ] A new parser of outside data has a fuzz target
- [ ] A protocol decision is in `docs/protocol-v0.1-notes.md`, an architectural decision under `docs/kararlar/`
- [ ] Every text a user sees is in English and Turkish, and in the other languages as docs/ceviri.md describes; the documents are up to date and in English (docs/kararlar/0019-english-by-default.md)
- [ ] If the interface changed: screenshots in the light and dark theme, and on Linux in KDE Plasma, Hyprland and Omarchy (docs/tasarim.md)
- [ ] If what the servers see changed, the privacy policy is updated
- [ ] Every promise made in a document, a policy or on the site (a duration, a limit, a cost) is backed by code and a test; changed text is updated in every language file
- [ ] Tests that expect an error check its kind; timing tests use no fixed wait (docs/surec.md, the rules learned from review)
- [ ] Every check is green, optional ones included; a red one has its cause written down and an issue opened
- [ ] A new dependency says why it is needed and who maintains it
- [ ] An entry file is added under `changelog.d/` (or a `.none.md` that says why not)

## Independent review

<!-- The reviewer's findings and what was done about each -->
