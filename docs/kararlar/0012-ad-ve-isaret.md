# 0012: The code is free; the name and the mark are only for TotalDrop's own releases

- **Status:** Accepted, 4 November 2026 (#234)
- **Context:** TotalDrop's code is openly licensed
  ([LICENSING.md](../../LICENSING.md)): the protocol libraries are MIT or
  Apache-2.0, the apps and the site GPL-3.0-or-later, the servers
  AGPL-3.0-or-later. These licences allow copying, changing and
  distributing the code. They do not give permission to use the TotalDrop
  name and mark. Apache-2.0 says so explicitly in its section 6. GPL-3.0
  gives no permission for the name and the mark; its section 7 lets a project
  write this into its licence notice as an additional term (e) and require
  modified versions to be marked as different (c). Until now, nothing said
  anywhere how others may use the name and the mark. The
  [/brand](https://totaldrop.app/brand) page (#220) offers the logo files
  for download but does not say who may use them, or on what terms.

  The reason to trust TotalDrop is its encryption. When a user installs an
  app that says "TotalDrop", they must know that they are installing an app
  whose code comes from this repository and that makes the same security
  promises. If a modified version carries the same name and mark, the user
  cannot tell it apart from the official app. Even if that version is not
  malicious, it may have weakened the encryption.

  Other open-source projects:
  - **Mozilla:** Anyone who distributes unmodified Firefox may use the
    name. Whoever changes the code and distributes it uses their own name.
    That is why Debian used the name "Iceweasel" for years.
  - **Rust Foundation:** Leaves the name widely free for promotion and
    community work. Forbids distributing a modified compiler under the name
    "Rust" and appearing official.
  - **Signal:** The code is AGPL, but forks cannot use the Signal name and
    logo; they are published under their own names.

- **Options:**
  1. **Forks are published under their own name.** Every version that
     changes the code uses its own name and mark. Only those who package
     without changing the code keep the name. Mozilla and Signal do this.
  2. **Allow small changes.** A version with bug fixes and local
     adaptations keeps the name; a version that adds features or affects
     security uses its own name. Drawing the line of a "small" change is
     hard. A change that affects security can look small.
  3. **Leave it free.** Forks may use the name and the mark. A user may
     mistake a weakened version for the official app.

- **Decision:** Option 1 (Ozan, 4 November 2026, decision card). Ozan
  approved the other points below and the store rule in the thread on the
  same day. The rules are in [docs/marka.md](../marka.md). In short:
  - Whatever the code licences allow is free. A separate rule applies to
    the name and the mark. No additional term under GPL section 7 is added
    to the licence texts: the rule belongs to the name and the mark, not to
    the licence, and puts no restriction on using the code.
  - The mark files (SVG, PNG, icons) are distributed under the licence of
    the folder they are in; changing a file is a right the licence gives.
    The rule only governs using the name and the mark to present a
    product.
  - Linking, and using the name and the unmodified mark in news, articles
    and teaching, is free.
  - Packagers who build without changing the code, with only the patches
    packaging needs (Linux distributions, the AUR, F-Droid), may use the
    name and the mark. The `app.totaldrop.TotalDrop` entry on Flathub is
    published or approved by the project.
  - Versions that change the code use their own name and mark. They may say
    "based on TotalDrop".
  - Other clients that implement the protocol may say "works with
    TotalDrop". Their names may not contain TotalDrop, and they may not use
    the mark.
  - Appearing official or endorsed, changing the mark, and using similar
    names and domains are forbidden. In stores where the publisher appears
    as the developer (Google Play, the App Store, the Microsoft Store, the
    Snap Store), only TotalDrop publishes TotalDrop.
  - Selling products that carry the name or the mark needs written
    permission.

- **Why:** The rule protects the user and is easy to apply. The answer to
  "did you change the code?" is clear; the answer to "is it small?" is not.
  It does not touch the freedom of the code: anyone can take the code and
  change it as they like; they just publish it under another name. The
  open-protocol goal is kept too: compatible clients can say that they work
  with TotalDrop.

- **Consequences:**
  - `docs/marka.md` is the full text of the rules. `LICENSING.md` links to
    it.
  - A "Use of the name and the mark" section is added to the /brand page,
    in 35 languages.
  - A fork is advised, not required, to use an application ID other than
    `app.totaldrop` on Android and `app.totaldrop.TotalDrop` on Linux. The
    reasons are technical: stores and Flathub do not accept the same ID
    twice, Android does not install an update signed with another key, and
    two apps with the same desktop ID replace each other.
  - The name is not registered. The texts do not say "registered
    trademark". If the name is registered, the documents are updated.
  - Permission requests go to hello@totaldrop.app.
  - Out of scope: forks and compatible clients using the totaldrop.app
    servers. That is a rule for using a service; if needed, it is written
    in a separate decision.

- **Review condition:** When the name is registered, when a packager or a
  fork reports a case where the rule cannot be applied, or when the rule for
  using the servers is written.
