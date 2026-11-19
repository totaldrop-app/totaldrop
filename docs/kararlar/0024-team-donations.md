# 0024: Take donations in the TotalDrop team's name through Open Collective

- **Status:** Proposed (#352); pending Open Source Collective fiscal-host
  approval (supersedes the channel of [0005](0005-bagis.md) and
  [0017](0017-bagis-kisisel-sayfa.md) once approved and accepted before merge).
  The two-person admin requirement was met on 10 November 2026
  (`totaldrop-team` alongside two personal admin accounts); if that condition
  ceases to hold before or after approval, this decision cannot become or
  remain `Accepted` and is reopened.
- **Context:** [0005](0005-bagis.md) chose GitHub Sponsors for donations, and
  [0017](0017-bagis-kisisel-sayfa.md) set a time-limited interim arrangement
  using the project lead's personal GitHub Sponsors page
  (`github.com/sponsors/ozdil`) and Buy Me a Coffee page
  (`buymeacoffee.com/ozdil`) after Open Source Collective declined the initial
  application on 11 November 2026 while the contributor base was still small.

  TotalDrop is run by a team ([/team](https://totaldrop.app/team/)), and
  [0017](0017-bagis-kisisel-sayfa.md) specifies that once the project grows
  and a fiscal host approves an account in TotalDrop's name, donations move
  off personal accounts so donors see clearly that money goes to the project
  rather than an individual, without attaching personal bank, card, or tax
  details directly to the GitHub organization.

## Decision

1. **Single team channel:** Donations are taken on TotalDrop's Open
   Collective page at
   [opencollective.com/totaldrop](https://opencollective.com/totaldrop).
   The single-channel principle ([0005](0005-bagis.md)) continues; once
   active, this channel replaces both personal channels permitted under
   [0017](0017-bagis-kisisel-sayfa.md).
2. **Fiscal host and fees:** Funds are held by the non-profit fiscal host
   Open Source Collective, which retains 10% of each donation; the card or
   PayPal processor deducts its own payment-processing fee on top (varying by
   country and payment method). Every incoming donation and outgoing expense
   appears on Open Collective's public ledger as well as in the monthly table
   on `/donate` and in [docs/bagis-dokumu.md](../bagis-dokumu.md).
3. **Two-person administration:** At least two separate people administer the
   Open Collective page through their own personal accounts. The collective's
   shared contact address and team account (`totaldrop-team`,
   `hello@totaldrop.app`) serves only as the common contact address and a
   backup login alongside the personal accounts, and does not count toward
   the two-person requirement.
4. **Spending rules:**
   - Every expense request is approved by an admin other than the person who
     submitted it. Nobody approves their own request, and Open Collective's
     expense-policy setting preventing self-approval stays enabled. Any
     payment to a maintainer follows the same rule (the maintainer being paid
     submits it, another admin approves it, and its invoice or receipt is
     published in the ledger, carrying forward
     [0017](0017-bagis-kisisel-sayfa.md) rule 4).
   - Donations pay only for the costs listed on `/donate`. When funds are
     short, the priority order is: servers (the site, the rendezvous server,
     the relay server, and the `totaldrop.app` domain), signing and developer
     certificates, and the independent security audit before 1.0. Any expense
     outside those three is discussed in a public issue before an expense
     request is opened, and the request links to that issue.
   - The team account (`totaldrop-team`, `hello@totaldrop.app`) never submits
     or approves an expense request. Because that mailbox currently reaches
     only the project lead, allowing it to approve alongside a personal
     account would defeat the two-person rule; even if multiple people read
     that mailbox later, approvals stay strictly on named personal accounts.
     Any change in who has access to `hello@totaldrop.app` updates this
     clause immediately in a pull request.
5. **Cutover from [0017](0017-bagis-kisisel-sayfa.md):**
   - Donation copy across the repository and site names "TotalDrop" or "the
     TotalDrop team", never an individual.
   - Existing sponsors on `github.com/sponsors/ozdil` are notified of the new
     Open Collective address, monthly sponsorships on the personal GitHub
     Sponsors page are closed, and `buymeacoffee.com/ozdil` (along with any
     link to it on `/team`) is closed.
   - Any unspent donation balance held under
     [0017](0017-bagis-kisisel-sayfa.md) is transferred to
     `opencollective.com/totaldrop` and recorded as a line in
     [docs/bagis-dokumu.md](../bagis-dokumu.md).

## Why

Donors give out of trust in the project; sending them to a personal profile
blurs whether money supports the project or an individual. Open Collective
shows the team, lets a non-profit fiscal host hold the funds without linking
personal banking or tax credentials to the GitHub organization, and backs the
`/donate` transparency promise with an independent public ledger.

## Options considered

- **GitHub Sponsors for the organization:** An application was started, but
  even with a fiscal host GitHub requires linking an organization owner's
  personal payment identity for verification, which violates the rule against
  tying personal banking or payment records to the organization.
- **Direct bank account under the organization:** Zero host fee, but attaches
  banking and tax obligations directly to the organization.
- **Keep personal pages ([0017](0017-bagis-kisisel-sayfa.md)) permanently:**
  Suitable as a temporary bridge, not once a team collective is approved.

## Consequences

- Once Open Source Collective approves the collective and Ozan marks this
  decision `Accepted` before merging PR #368, `README.md` (and its 34
  translations), the site (`/` and `/donate` across 35 languages), the Linux
  AppStream metadata, `.github/FUNDING.yml`,
  [docs/bagis-dokumu.md](../bagis-dokumu.md), and
  [docs/marka.md](../marka.md) switch to `opencollective.com/totaldrop`, and
  the channel of [0005](0005-bagis.md) and
  [0017](0017-bagis-kisisel-sayfa.md) become `Superseded by 0024`.
- Until fiscal-host approval arrives, PR #368 stays a draft and
  [0017](0017-bagis-kisisel-sayfa.md) remains in effect so live `/donate`
  buttons never point to an inactive collective.
- The `/donate` cost table adds the "Donation fees" row (10% fiscal-host fee
  plus payment-processor fee) in all 35 languages in the same pull request,
  keeping the promise that `/donate` changes first whenever a cost is added.
