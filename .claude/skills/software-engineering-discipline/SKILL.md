---
name: software-engineering-discipline
description: Use for every task in the TotalDrop repository, whatever the folder. The working method from issue to release, the definition of done, independent review, approvals and honest reporting. Load it first; the area skills build on it.
---

# Software engineering discipline (TotalDrop)

Work like a senior engineer on a team that ships security-sensitive software.
The method is applied in full every time; speed never justifies skipping a
step. When a step is impossible, say so, record why and open a follow-up issue.

## Read first (binding, and stricter than this skill)

- `CLAUDE.md`: which area skills to load for the folders you change.
- `docs/surec.md`: flow, definition of done ("Bitti tanımı"), the review
  rules ("İncelemeden çıkan kurallar"), release list, deployment, secrets.
- `docs/gelistirme.md`: folders, branch names, local checks (`pre-commit`).
- `docs/kararlar/` (decisions), `docs/tehdit-modeli.md` (threat model),
  `docs/protocol-v0.1-notes.md` (protocol), `docs/tasarim.md` (design).
- `changelog.d/README.md`: one entry file per PR, `.none.md` when nothing
  changes for users.

Do not restate these documents in code comments or PRs; link them.

## Before writing code

- Restate the goal and the acceptance criteria of the issue. Ask only when a
  wrong guess is expensive; otherwise state the assumption and go on.
- Find the existing pattern and follow it. Prefer small, reversible steps.
- An architectural choice gets a decision record in `docs/kararlar/`
  (context, options, decision, reason, consequences). Old records are
  superseded, never deleted.
- A security-relevant change updates `docs/tehdit-modeli.md` in the same PR.

## While writing code

- Correctness, then clarity, then performance (measured, not guessed).
- Make invalid states unrepresentable: a stage is an enum, not a string or a
  pair of bools; after every `await`, check that generation and stage are
  still the same.
- No panic or `unwrap` on outside input; bound every size and allocation
  driven by input; every network call has a timeout.
- Never swallow an error; messages are actionable and hold no secrets.
- A new dependency needs a reason, its licence, maintenance status and the
  alternatives in the PR (`deny.toml` must still pass).
- A TODO carries an issue number, or it is not written.

## Tests

- A bug fix starts with a test that fails without the fix; say in the PR
  that it failed before and passes after.
- A test that expects an error checks its kind or code, not just "an error".
- Timing tests wait for a condition with a deadline, never a fixed `sleep`.
- Every new parser of untrusted input gets a fuzz target in `fuzz/`.
- When you doubt a behaviour, write a small test and report the result
  (passed/failed) instead of assuming.

## Review and approval

- Nobody approves their own work. Before asking a maintainer, run a separate
  review agent that sees only the diff and the code, not your reasoning; ask
  it for correctness, security, contradictions with the docs, overclaims, and
  the six error types in `docs/surec.md`. Every finding is fixed or answered
  in the PR with a reason and a follow-up issue.
- Merging, tagging, publishing, deploying, changing repository, DNS or
  Cloudflare settings and anything that spends money need the maintainer's
  explicit approval. `main` is merged by rebase only.

## Reporting

- Reply in the user's language (Turkish for the maintainers); code, commits
  and identifiers stay in English.
- Say what was done, what was verified and how, what failed and what
  remains. Mark every unverified or assumed point as such.
- Cite code as `path/file.ext:line`.
- Point out when the project breaks its own rules, and propose the fix.
