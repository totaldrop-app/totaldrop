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
  `fixed`, `security`, as in
  [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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

Some changes mean nothing to someone using or contributing to TotalDrop: a
comment, a typo in a document, a test-only refactor. Say so in
`<issue>-<slug>.none.md`, one sentence on why, so the choice is visible in
review. A release deletes these files without copying them. A correction
to an old release section of CHANGELOG.md also comes with one.
Dependabot's pull requests need no file.

## A release for one platform

A heading for one platform, `"[linux-…]"` or `"[android-…]"`, takes only
what that platform ships (#152):

- an entry whose bold area names that platform, or no platform at all
  (Core, Site, CI, Build, translations…), moves into the release;
- an entry that names only the other platform stays in its file;
- an entry that names both moves in under every area except the other
  platform's, and stays in its file under the other platform's area only,
  for that platform's release. For a Linux release, `**Android, Linux,
  site:**` is released as `**Linux, site:**` and left as `**Android:**`.

The command line ships with the Linux packages, so `CLI` and `Command line`
count as Linux. Any other heading, such as `"[android-…] and [linux-…]"`,
takes every entry. `.none.md` files go with any release.

## Checks

`node tools/changelog.mjs check` checks every file here, and that the
Unreleased section of CHANGELOG.md holds only its pointer. CI runs it, and
fails a pull request that adds no file here.
