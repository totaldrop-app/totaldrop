# Governance

This page says how decisions are made in TotalDrop, who holds which role, and what happens if a person with a key role is no longer available.

## Decision model

TotalDrop has a project lead, who makes the final call on every change, and a small team of maintainers who review and merge pull requests. Decisions are made in public and written down:

- **Everyday changes** start as a [GitHub issue](https://github.com/totaldrop-app/totaldrop/issues) that says what will change and when it is done. The work happens on a short-lived branch and reaches `main` only through a pull request that passes CI and an independent review; GitHub enforces CI and a maintainer's approving review for everyone, with no bypass ([docs/surec.md](docs/surec.md), in Turkish).
- **Architecture decisions** are recorded in [docs/kararlar/](docs/kararlar/README.md) (in Turkish): the options considered, the decision and its reasons. A decision is never deleted, only replaced by a newer record that says so.
- **Protocol decisions** are recorded in [docs/protocol-v0.1-notes.md](docs/protocol-v0.1-notes.md).
- **Security reports** follow [SECURITY.md](SECURITY.md) and are discussed privately until a fix is released.

Anyone can propose a change by opening an issue or a pull request. Proposals are judged by the rules in [CONTRIBUTING.md](CONTRIBUTING.md), the [threat model](docs/tehdit-modeli.md) and the design rules in [docs/tasarim.md](docs/tasarim.md). When a proposal is turned down, the reason is written on the issue or pull request.

## Roles

| Role | Responsibilities | Who |
| --- | --- | --- |
| **Project lead** | Sets the roadmap; reviews and merges pull requests; approves every release and deployment (the `linux-release`, `play-release` and `cloudflare` environments); pushes release tags; is an owner of the GitHub organization; owns the Cloudflare account, the Google Play account and the Play upload key; answers security reports | [@ozdil](https://github.com/ozdil) |
| **Maintainer** | Reviews and merges pull requests. The GitHub team `@totaldrop-app/maintainers` is the code owner of every file ([.github/CODEOWNERS](.github/CODEOWNERS)), and the `main review` ruleset requires one code-owner approval with no bypass: every pull request needs an approving review on GitHub from a maintainer who did not open it (so a maintainer's own pull request needs the other maintainer's approval) before it can merge | [@ozdil](https://github.com/ozdil), [@sadikkuzu](https://github.com/sadikkuzu) |
| **Developer** | Has write access: writes changes on branches in this repository and opens pull requests for them, which need a maintainer's approval to merge | [@zaforcan](https://github.com/zaforcan) |
| **Security contact** | Reads security@totaldrop.app and private vulnerability reports; replies within 72 hours; coordinates the fix and the advisory | [@ozdil](https://github.com/ozdil) |
| **Contributor** | Proposes changes through issues and pull requests, following [CONTRIBUTING.md](CONTRIBUTING.md) and the [code of conduct](CODE_OF_CONDUCT.md) | Anyone |
| **Translator** | Writes and reviews interface text in one of the 35 languages ([docs/ceviri.md](docs/ceviri.md)) | Anyone; reviewed languages are listed in the README |
| **Tester** | Runs beta builds and reports problems; Android beta testers are invited by the project lead | Invited testers and anyone using the Linux beta |

AI coding assistants write much of the code and run the independent review of each pull request ([docs/surec.md](docs/surec.md), step 5). They hold no role here: every merge is made or approved by a maintainer, and every release, deployment and settings change is approved by the project lead. Both maintainers are organization owners, so the second half is an agreement, not a setting.

## Continuity

Releases, deployments and the accounts behind them still rest on one person, the project lead, so for those the bus factor is 1. This is the project's biggest organizational risk, and it is not solved yet. What exists now:

- Everything needed to rebuild and release the software is public: the source, the build and release workflows, the build images and the process documents. Releases are built only in GitHub Actions, never on a personal machine.
- The servers are deployed from this repository by CI, so a new account can run them from the same code.
- The secrets (the Google Play upload key and its password, the Cloudflare API token, the addresses project mail is forwarded to) live only in protected GitHub environments and with the project lead. Linux build provenance is signed without a long-lived key (Sigstore).

What is still missing, and needed before the project can say it continues without interruption if the project lead is gone:

- a second trusted person with owner access to the Cloudflare account and the Google Play account (the GitHub organization has a second owner, [@sadikkuzu](https://github.com/sadikkuzu));
- a copy of the Play upload key, its password and the account recovery codes that this person can reach.

When someone else takes on releases or deployments, this page will name them and the roles they take on.

## Branch protection and Scorecard

Two active repository rulesets protect `main`, and neither allows any actor — including maintainers and organization owners — to bypass it:

- **`main`:** blocks branch deletion and force pushes, requires linear history, and requires the `CI` status check to pass.
- **`main review`:** requires a pull request before merging, rebase-only merges, 1 approving review, a Code Owners review ([.github/CODEOWNERS](.github/CODEOWNERS) assigns `*` to `@totaldrop-app/maintainers`), dismissal of stale approvals when new commits are pushed, an extra approval if unattributed changes are present (`require_extra_approval_for_unattributed_changes: true`), and resolution of all review threads.

Those two rulesets are the sole branch-protection rules on `main`; the earlier classic branch protection rule was removed so rules are maintained in one place. A third active ruleset, **`release tags`**, protects release tags (`linux-v*`, `android-v*`, `cli-v*`, and `server-v*`) against creation, update, deletion, and force-push; only repository admins can bypass it.

There is no everyday bypass button. If [@sadikkuzu](https://github.com/sadikkuzu) cannot be reached during an urgent security fix or production outage, the project lead may temporarily relax `main review` in repository settings (an action recorded in the organization audit log), record the independent review's findings on the pull request, and restore the empty bypass list immediately after merging; if the project lead cannot be reached during an active emergency, the other maintainer may do the same and notify the project lead for post-incident review as soon as they are reachable.

[OpenSSF Scorecard](.github/workflows/scorecard.yml) runs on `main` and weekly. How its branch, review and maintenance checks apply to this repository:

- **`Branch-Protection` — 1 required approval instead of 2:** TotalDrop has two maintainers, and GitHub does not let a pull request's author approve their own pull request, so requiring two maintainer approvals would deadlock every maintainer-authored pull request.
- **`Branch-Protection` — last-push approval off (`require_last_push_approval: false`):** right before merging, the maintainer who merges rebases the branch onto current `main` ([docs/surec.md](docs/surec.md), step 6). Because `dismiss_stale_reviews_on_push` is on, that rebase dismisses the earlier approval. Requiring the most recent push to be approved by someone other than who pushed it would stop the reviewing maintainer from re-approving after rebasing the other maintainer's pull request, while the author cannot approve their own — deadlocking the merge until the author pushes again. With `dismiss_stale_reviews_on_push` on, `require_extra_approval_for_unattributed_changes: true`, and no bypass, any new commit pushed by the author still dismisses approvals and requires a fresh review from the other maintainer (and any unattributed commit requires an extra approval, whereas attributed maintainer rebases do not).
- **`Branch-Protection` — up-to-date branches off (`strict_required_status_checks_policy: false`):** turning strict status checks on in GitHub would mark every open pull request out of date on each merge to `main`, forcing a rebase, a new `CI` run, and — because `dismiss_stale_reviews_on_push` is on — a fresh approval before merging. Instead, the merging maintainer rebases onto `main` immediately before merging, waits for `CI` on that commit, and approves (or re-approves) that rebased commit ([docs/surec.md](docs/surec.md), step 6).
- **`Code-Review`:** maintainers review and approve each other's pull requests on GitHub; since `main review` removed maintainer bypass on 10 November 2026, every merged pull request carries a maintainer approval and `Code-Review` will reach `10/10` once 30 reviewed commits have landed in Scorecard's 30-commit window (earlier unapproved merges from before that change weigh on the score until they age out of that window).
- **`Maintained`:** the repository opened on 27 October 2026, and Scorecard scores every repository younger than 90 days as `0`; this check clears on its own once 90 days have passed (around 25 January 2027).

After uploading the SARIF report on `main`, a separate `reconcile` job in [scorecard.yml](.github/workflows/scorecard.yml) (without `id-token: write`) runs [tools/dismiss-scorecard-alerts.mjs](tools/dismiss-scorecard-alerts.mjs) to dismiss only alerts matching those exact `Branch-Protection` (or initial-90-day `Maintained`) warnings and to reopen any previously dismissed alert if an undocumented warning appears.
