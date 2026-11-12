# 0017: Donations go through the project lead's GitHub Sponsors page for now

- **Status:** Accepted, 11 November 2026 (proposed by Sadık, accepted by
  Ozan; #487). Amended 12 November 2026 (Ozan): the Buy Me a Coffee page,
  payments to a maintainer, and the balance on the move. Sets in whose
  name the donation channel of [0005](0005-bagis.md) is opened, and when
  that changes.

## Context

[0005](0005-bagis.md) tied donations to GitHub Sponsors. The page used for
it is the personal page of the project lead, Ozan
(github.com/sponsors/ozdil). As the project is run by a team, taking
donations in the team's name was the better choice, so we applied to Open
Collective (#352, PR #368). Open Source Collective declined the
application on 11 November 2026. We can apply again once there are more
contributors.

Sadık proposed that donations go through Ozan's personal page for now and
move to an account in TotalDrop's name when the project grows. Ozan
accepted.

## Decision

1. Donations are taken once or every month on the project lead's
   personal GitHub Sponsors page. This is a deliberate, time-limited
   exception to the rule that the project acts in the team's name (#352).
2. The project lead's Buy Me a Coffee page (buymeacoffee.com/ozdil),
   linked from the project lead's card on the team page (#486), is a
   second channel under the same exception. What comes in there is a
   donation like any other: it is spent under rule 4 and published in
   the same ledger.
3. The /donate page says plainly, in 35 languages, that the money arrives
   in the project lead's name and moves to an account in TotalDrop's name
   when the project grows.
4. Donations pay only for the costs on the /donate page. This decision
   sets the rule: Ozan proposes each expense and Sadık approves it, and
   nobody approves their own proposal. A payment to a maintainer is the
   exception: the maintainer who is paid proposes it, the other
   maintainer approves it, and its invoice or receipt is shown in the
   ledger. What came in and what went out are published every month, in
   the same lines, in the table on /donate and in the
   [donation ledger](../bagis-dokumu.md) in the repository.
5. The obligations that donations create as personal income belong to
   the account holder.

## When this changes

A new decision is opened to move to an account in TotalDrop's name (Open
Collective through a fiscal host, or the Sponsors page of an organization
on GitHub) when one of these happens:

- the number of regular contributors grows and applying to a fiscal host
  again makes sense;
- monthly donations exceed the project's monthly costs three months in a
  row;
- someone on the team, or Ozan, says the money should no longer sit in a
  personal account.

On the move, the old page's sponsors are told about the new account, the
monthly sponsorships on the personal page are closed, and so is the Buy
Me a Coffee page. Donations not yet spent move to the new account, and
the transfer is a line in the ledger.

## Consequences

- The donation address in `.github/FUNDING.yml`, the README badge, the
  site buttons and the Linux metainfo stays github.com/sponsors/ozdil.
- The move to Open Collective in PR #368 is not applied while this
  decision stands.
- The profile text, tiers and tax details of the GitHub Sponsors and Buy
  Me a Coffee pages are kept in Ozan's own accounts, not in the
  repository.
