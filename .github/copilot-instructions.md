# Instructions for Copilot code review

TotalDrop is an open-source app that sends files end-to-end encrypted
from any platform to any other. The binding rules are in `CLAUDE.md`,
`docs/surec.md`, `docs/tasarim.md`, `docs/ceviri.md`, `docs/kararlar/`
and `.claude/skills/`; this file sums up what a review needs most often.

## How to write the review

- Write the review and its comments in English.
- Every finding about code gives `path/file.ext:line`.
- Mark a finding you are not sure of as unverified and say how to test it.
- Do not report formatting or style; the formatters run in CI.
- Your review is an extra reading; it never replaces a maintainer's
  approval.

## Language

The repository is written in English (decision 0019, #490).

- **English:** code and comments, commit messages, PR titles
  (Conventional Commits: `fix(core): …`) and descriptions, issues,
  `changelog.d/` entries (moved as they are into `CHANGELOG.md`), and
  every new or translated document.
- **Turkish, until translated:** documents in `docs/` not yet translated
  (#490) stay binding. Editing one in Turkish, or not translating it in
  an unrelated PR, is not a finding.
- **35 languages:** every text users read. The English file is the
  template (`l10n/app/app_en.arb`, `l10n/cli/cli_en.arb`,
  `site/public/i18n/en.json`, `site/privacy/en.html`). New or changed text
  comes in all 35 languages in the same PR (`CLAUDE.md`). A missing
  language falls back to English and does not fail CI, but it is a
  finding.
- `CHANGELOG.md` changes only at a release; a PR that leaves it alone is
  right.

## Check on every PR

- **Definition of done** (`docs/surec.md`): a bug fix starts with a test
  that shows the bug; `changelog.d/<issue>-<slug>.<section>.md`, or a
  `.none.md` saying why; the PR template's list filled in.
- **Rules from past reviews** (`docs/surec.md`):
  - A test that expects an error checks its kind or code.
  - No fixed `sleep` in timing tests; wait for a condition with a deadline.
  - A state is a named type, not a string or a pair of bools; after an
    `await`, the generation and stage are checked again.
  - When a document, the privacy policy, the site or a README promises a
    time, a limit or a cost, the PR shows the code and test behind it.
  - Outside text has its control characters made visible before it
    reaches a terminal or the UI; no error is swallowed.
  - A tool or base version has one source; a check compares the copies.
- **Security and cryptography** (`.claude/skills/`): secrets are compared
  in constant time; lengths and counts from the network are bounded
  before they are read; a new parser has a target in `fuzz/`. A protocol
  change updates `docs/protocol-v0.1-notes.md`, an architecture change
  `docs/kararlar/`.
- **Secrets:** `key.properties`, signing keys, passwords and API keys
  never enter the repository. If you see one, do not repeat any part of
  it: give only `path:line` and ask for it to be reported through the
  private channel in `SECURITY.md`.
- **UI** (`docs/tasarim.md`): follows the system theme (light and dark);
  on Linux, KDE Plasma, Hyprland and Omarchy. A fixed colour or size that
  ignores the system theme or text scale, or a layout that breaks in
  right-to-left languages, is a finding. Exceptions: the mark's (Köşe)
  colours and the Aurora palette used when the system gives no colour.
- **CI** (`.github/workflows/`): third-party actions pinned to a commit
  SHA, least permissions; packages are built only in CI.
