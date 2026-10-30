- **Process:** Every pull request records its change in a file of its own
  in `changelog.d/` instead of editing CHANGELOG.md, so pull requests no
  longer conflict with each other there. `tools/changelog.mjs release`
  moves the files into CHANGELOG.md when a release is made, and CI checks
  every file and fails a pull request that adds none (#150).
