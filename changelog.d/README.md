# Changelog entries

Every pull request records its change here, in a file of its own, instead
of editing CHANGELOG.md. Pull requests then never touch the same lines and
can be merged one after the other without a rebase. A release moves every
file into CHANGELOG.md and deletes it (docs/surec.md, "Sürüm listesi").

## One file per change

Name the file `<issue>-<slug>.<section>.md`, for example
`129-cli-yes-words.fixed.md`:

- `<issue>` is the issue the pull request closes or refers to;
- `<slug>` is a few lowercase words joined by hyphens;
- `<section>` is one of `added`, `changed`, `deprecated`, `removed`,
  `fixed`, `security` ([Keep a Changelog](https://keepachangelog.com/en/1.1.0/)).

Write the entry exactly as it will appear in CHANGELOG.md:

```markdown
- **Android, Linux:** What changed, said for someone who uses TotalDrop,
  wrapped at 76 columns, with continuation lines indented by two spaces
  (#129).
```

- It starts with the areas in bold, followed by a colon.
- It ends with the issue numbers in parentheses and a full stop, and names
  the file's issue.
- A file may hold several entries of the same section, one after the
  other, with no blank line between them. A change that belongs to two
  sections gets two files.

## A change that needs no entry

Some changes mean nothing to someone using TotalDrop: a comment, a CI
setting, a test-only refactor. Say so in `<issue>-<slug>.none.md`, one
sentence on why, so the choice is visible in review. A release deletes
these files without copying them. Dependabot's pull requests need no file.

## Checks

`node tools/changelog.mjs check` checks every file here, and that the
Unreleased section of CHANGELOG.md holds only its pointer. CI runs it, and
fails a pull request that adds no file here.
