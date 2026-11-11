# 0007: Working process

- **Status:** Accepted, 28 October 2026
- **Context:** In the first two days, for speed, pull requests were merged
  by their authors and the Android packages were built on a personal
  computer. This went against the rule in our own documents.
- **Decision:** Every piece of work starts with an issue; every pull request
  is read by a reviewer who did not write the code; nothing is merged until
  every CI check is green; `main` is closed to direct pushes; every
  published package is built in CI. The Linux packages carry build
  provenance that the user can verify; because Google re-signs the Play
  package, the provenance on Android is only the project's own record.
- **Why:** In software that makes security claims, the author approving
  their own work is not enough; the user must also be able to verify that
  the package they downloaded came from the code in the repository.
- **Consequences:** The process is in [surec.md](../surec.md); the pull
  request template contains the definition of done.
