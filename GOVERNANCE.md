# Governance

This page says how decisions are made in TotalDrop, who holds which role, and what happens if a person with a key role is no longer available.

## Decision model

TotalDrop has a project lead, who makes the final call on every change, and a small team of maintainers who review and merge pull requests. Decisions are made in public and written down:

- **Everyday changes** start as a [GitHub issue](https://github.com/totaldrop-app/totaldrop/issues) that says what will change and when it is done. The work happens on a short-lived branch and reaches `main` only through a pull request that passes CI and an independent review; GitHub enforces CI for everyone ([docs/surec.md](docs/surec.md), in Turkish).
- **Architecture decisions** are recorded in [docs/kararlar/](docs/kararlar/README.md) (in Turkish): the options considered, the decision and its reasons. A decision is never deleted, only replaced by a newer record that says so.
- **Protocol decisions** are recorded in [docs/protocol-v0.1-notes.md](docs/protocol-v0.1-notes.md).
- **Security reports** follow [SECURITY.md](SECURITY.md) and are discussed privately until a fix is released.

Anyone can propose a change by opening an issue or a pull request. Proposals are judged by the rules in [CONTRIBUTING.md](CONTRIBUTING.md), the [threat model](docs/tehdit-modeli.md) and the design rules in [docs/tasarim.md](docs/tasarim.md). When a proposal is turned down, the reason is written on the issue or pull request.

## Roles

| Role | Responsibilities | Who |
| --- | --- | --- |
| **Project lead** | Sets the roadmap; reviews and merges pull requests; approves every release and deployment (the `linux-release`, `play-release` and `cloudflare` environments); pushes release tags; is an owner of the GitHub organization; owns the Cloudflare account, the Google Play account and the Play upload key; answers security reports | [@ozdil](https://github.com/ozdil) |
| **Maintainer** | Reviews and merges pull requests. The GitHub team `@totaldrop-app/maintainers` is the code owner of every file, so a pull request from anyone outside it needs a maintainer's approving review; maintainers merge others' pull requests by approving them, not by bypassing the rule. A maintainer may merge their own pull request without an approval; GitHub still enforces CI, but the independent review before such a merge is an agreement, not a setting | [@ozdil](https://github.com/ozdil), [@sadikkuzu](https://github.com/sadikkuzu) |
| **Developer** | Has write access: writes changes on branches in this repository and opens pull requests for them, which need a maintainer's approval to merge | [@zaforcan](https://github.com/zaforcan) |
| **Security contact** | Reads security@totaldrop.app and private vulnerability reports; replies within 72 hours; coordinates the fix and the advisory | [@ozdil](https://github.com/ozdil) |
| **Contributor** | Proposes changes through issues and pull requests, following [CONTRIBUTING.md](CONTRIBUTING.md) and the [code of conduct](CODE_OF_CONDUCT.md) | Anyone |
| **Translator** | Writes and reviews interface text in one of the 35 languages ([docs/ceviri.md](docs/ceviri.md), in Turkish) | Anyone; reviewed languages are listed in the README |
| **Tester** | Runs beta builds and reports problems; Android beta testers are invited by the project lead | [@mstfyrdmc](https://github.com/mstfyrdmc) (iOS and macOS), invited testers and anyone using the Linux beta |

AI coding assistants write much of the code and run the independent review of each pull request ([docs/surec.md](docs/surec.md), step 5). They hold no role here: every merge is made or approved by a maintainer, and every release, deployment and settings change is approved by the project lead. Both maintainers are organization owners, so the second half is an agreement, not a setting.

## Continuity

Releases, deployments and the accounts behind them still rest on one person, the project lead, so for those the bus factor is 1. This is the project's biggest organizational risk, and it is not solved yet. What exists now:

- Everything needed to rebuild and release the software is public: the source, the build and release workflows, the build images and the process documents. Releases are built only in GitHub Actions, never on a personal machine.
- The servers are deployed from this repository by CI, so a new account can run them from the same code.
- The secrets (the Google Play upload key and its password, the Cloudflare API token) live only in protected GitHub environments and with the project lead. Linux build provenance is signed without a long-lived key (Sigstore).

What is still missing, and needed before the project can say it continues without interruption if the project lead is gone:

- a second trusted person with owner access to the Cloudflare account and the Google Play account (the GitHub organization has a second owner, [@sadikkuzu](https://github.com/sadikkuzu));
- a copy of the Play upload key, its password and the account recovery codes that this person can reach.

When someone else takes on releases or deployments, this page will name them and the roles they take on.
