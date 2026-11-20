# Using the TotalDrop name and mark

This document explains how others may use the "TotalDrop" name and the
TotalDrop mark (Köşe). The reasoning behind the decision is in
[decision record 0012](kararlar/0012-ad-ve-isaret.md); the visual rules
are on the [/brand](https://totaldrop.app/brand) page and in the
[design rules](tasarim.md). The "Using the name and mark" section on that
page is the short version of this document in 35 languages.

## Code and brand are separate

TotalDrop's code is openly licensed ([LICENSING.md](../LICENSING.md)).
The licences allow you to copy, change and distribute the code, but they
do not give permission to use the TotalDrop name and mark. This document
governs the name and mark. It places no restriction on using the code and
is not an additional term on the licences.

The mark files (`site/public/brand/files/`, app icons) are also
distributed under the licence of the folder they are in: copying and
changing a file is a right the licence gives. This document only states
how the name and mark may be used to present a product or service.

Why: people trust TotalDrop for its encryption. Knowing that an app called
"TotalDrop" comes from the code in this repository and makes the same
security promises is part of how users protect themselves.

## What you may do without permission

- Link to totaldrop.app or to this repository.
- Mention TotalDrop, use the name and the unmodified mark, and share
  screenshots in news, articles, reviews, videos, lessons and books.
- **Packaging:** Build the code without changing it and distribute it
  under the TotalDrop name and mark in a Linux distribution, the AUR,
  F-Droid or a similar repository. Patches needed only for packaging do
  not count as changes: build paths, file locations, or using the
  distribution's same library or a compatible version of it in place of a
  bundled dependency. Patches that change the app's behaviour, its
  interface, the servers it connects to or protocol constants, or that
  replace a cryptographic library with another implementation, count as
  changes.
- **Flathub:** The entry on Flathub under the `app.totaldrop.TotalDrop` ID
  is published or approved by the project, because the publisher appears
  there as TotalDrop. If you want to package it on Flathub, write to
  hello@totaldrop.app first.
- **Compatible software:** If you write your own client, library or plugin
  that implements the protocol, you may say "works with TotalDrop" or
  "compatible with TotalDrop". Your software's name must not contain
  TotalDrop and you must not use the TotalDrop mark. Identifiers inside
  the protocol itself (e.g. `totaldrop/0`) are needed for compatibility
  and are not covered by this rule.
- Non-commercial community work: meetup talks, stickers, posters. The name
  and mark are used unmodified.

## If you changed the code (forks)

You may change and distribute the code; the licences allow it. Publish the
modified version under your own name:

- Use your own name and mark. The "TotalDrop" name and the Köşe mark do
  not appear as your product's name, icon or identity in the app, on the
  store page or on the website. The origin statement below is the
  exception.
- We recommend using your own application ID (something other than
  `app.totaldrop` on Android and `app.totaldrop.TotalDrop` on Linux). This
  is a practical need rather than a requirement: stores and Flathub do not
  accept the same ID twice, Android does not install an update signed with
  another key, and two apps with the same desktop ID replace each other.
- You may state your origin: "X is based on TotalDrop" or "X is a fork of
  TotalDrop". This statement must not imply that TotalDrop endorses or
  audits X.
- The copyright and licence notices required by the licences stay in
  place. Changing the name does not remove them.

## What not to do

- Appear to be an official release, partner or endorsed product of
  TotalDrop.
- Use a modified mark as TotalDrop's mark, combine the mark with another
  logo, or use it outside the visual rules
  ([/brand](https://totaldrop.app/brand), "What not to do").
- Use a name, domain, handle or app name that resembles TotalDrop and
  could be confused with it (e.g. "TotalDrop Pro", "totaldrop-app.net").
- Publish an app under the TotalDrop name or mark in stores where the
  publisher appears as the developer (such as Google Play, the App Store,
  the Microsoft Store or the Snap Store). In those stores, only TotalDrop
  publishes TotalDrop. The packaging permission above is for packages in
  distributions and community repositories.
- Use the name or mark in malware, fraud or misleading advertising.

## What needs permission

- Selling goods that carry the name or mark (such as T-shirts, stickers or
  mugs).
- Any other commercial use not covered by this document.

To ask for permission, write to hello@totaldrop.app: explain what you want
to do and where the name or mark will appear.

## Official accounts

TotalDrop's official accounts on the web and social media are:

- Website: [totaldrop.app](https://totaldrop.app)
- GitHub: [github.com/totaldrop-app](https://github.com/totaldrop-app)
- X: [x.com/totaldropapp](https://x.com/totaldropapp) (@totaldropapp)
- Instagram: [instagram.com/totaldropapp](https://www.instagram.com/totaldropapp/) (@totaldropapp)
- Threads: [threads.com/@totaldropapp](https://www.threads.com/@totaldropapp) (@totaldropapp)

Outside these, the project uses only the following channels: apps are
distributed through Google Play and GitHub releases, donations are taken
through GitHub Sponsors
([github.com/sponsors/ozdil](https://github.com/sponsors/ozdil)) and
Buy Me a Coffee ([buymeacoffee.com/ozdil](https://buymeacoffee.com/ozdil)),
and emails come only from `@totaldrop.app` addresses. The Buy Me a Coffee
page is linked from the coffee button on the project lead's card on the
[team page](https://totaldrop.app/team/) and is a second donation channel
under the same temporary exception as the Sponsors page (decision 0017,
#487): what arrives there is also a donation, is spent under the same rule
and is published on the same lines in the ledger on the
[donate page](https://totaldrop.app/donate/) (#486). Because it is a
personal page, it does not appear in the official brand accounts list
above, on the `/brand` page, in the site footer, in the READMEs or on the
org profile; it is mentioned only on the team card and on the `/donate`
page.

If an account is opened on a new platform, the handle `totaldropapp` is
used where possible (on GitHub the name is `totaldrop-app`). This list,
the list on [/brand](https://totaldrop.app/brand#use), the site footer,
the READMEs and the org profile (`totaldrop-app/.github`) are updated in
the same PRs. An account that uses this name but is not listed here does
not belong to the project; report any such account to hello@totaldrop.app.

## Notes

- The TotalDrop name is not registered. This document is not a claim of a
  registered trademark; it explains how the project wants its name and
  mark to be protected.
- If you see a use that does not follow these rules, report it to
  hello@totaldrop.app. If it is a security issue (such as a fake app), use
  the private channel in [SECURITY.md](../SECURITY.md).
- If this document changes, the change is made through a decision record
  and a PR.
