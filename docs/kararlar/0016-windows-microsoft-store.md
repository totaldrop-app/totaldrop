# 0016: The Windows app is written with WinUI 3 and distributed through the Microsoft Store

- **Status:** Accepted, 10 November 2026 (Ozan, #33)
- **Context:** Until now, the Windows app (#33) was waiting on two things:
  choosing the UI toolkit and paying for a code signing certificate from
  donations. The donate page lists this as a cost "before Windows"
  (`don.b4`: Certum's open-source certificate, 69 EUR for the first year,
  then 49 EUR); T12 in the threat model also says "Windows and macOS
  signing are waiting on donations". An unsigned `.exe` or `.msi` opens on
  Windows behind SmartScreen and Defender warnings; for an app built on
  encryption, that undermines trust from the start.

  Two things changed:
  - Developer registration on the Microsoft Store is now free: since
    September 2025 for individual accounts, and since 2026 for company
    accounts. Registration goes through `storedeveloper.microsoft.com`;
    entering through other paths opens the old flow.
  - When an MSIX package is submitted to the Store, Microsoft re-signs the
    package with its own certificate after certification; we do not need to
    buy a certificate. This applies only to MSIX: if an `.exe` or `.msi` is
    submitted to the Store, we must sign it ourselves.

  That solves the signing problem at zero cost. The package managers
  decision proposal (#344, PR #347) leaves the Microsoft Store channel and
  Store signing to this decision. That proposal says company accounts are
  paid; according to Microsoft's February 2026 page, they no longer are.

  Sources (read on 7 November 2026):
  [opening an account](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account),
  [company account](https://learn.microsoft.com/en-us/windows/apps/publish/whats-new-company-developer),
  [code signing options](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options),
  [Store FAQ](https://learn.microsoft.com/en-us/windows/apps/publish/faq/get-started-with-the-microsoft-store).

## Interface

- **Options:**
  1. **Qt 6 and QML, sharing the Linux code.** Since Qt 6.8 there has been
     a "FluentWinUI3" style that imitates the Windows 11 look. This is the
     least work, but details such as the Mica backdrop, title bar
     integration, the system accent colour and high contrast remain an
     imitation. The design rules ([tasarim.md](../tasarim.md)) require one
     hundred percent fidelity to the system theme.
  2. **Flutter, using the Windows target of the Android app.** Code is
     shared, but the Material look is foreign to Windows; a Fluent look
     falls to third-party packages.
  3. **WinUI 3 (Windows App SDK) and C# (.NET, Native AOT), bound to the
     core over a C ABI.** This is Windows 11's own UI toolkit: Fluent
     controls, light/dark/high-contrast themes, the system accent colour,
     Segoe UI Variable and Fluent icons come natively. Mica (`MicaBackdrop`)
     and title bar integration (`ExtendsContentIntoTitleBar`, caption button
     colours) are native, though the app sets them up; caption buttons may
     not follow live theme changes automatically, which will be tested in
     step 3. With Native AOT, the .NET runtime is not installed separately
     and startup is fast; in return, XAML is written to be trim-safe
     (`x:Bind`, no reflection-based `{Binding}`).
- **Proposed:** Option 3. The criterion here is the same as when Qt was
  chosen on Linux ([0006](0006-linux-qt.md)): the platform's own toolkit
  and full fidelity to the system.

### Rules

- The core does not change: the protocol, cryptography and transfer stay in
  Rust (`crates/`) alone; the C# side performs no cryptography
  ([mimari.md](../mimari.md)). A thin `windows/core-ffi` crate exposes a C
  ABI for the core. The binding tool (uniffi with a C# generator, or
  csbindgen) is chosen in step 2; the criteria are maintenance of the
  dependency and thread safety of cancellation and progress events. The UI
  thread never blocks on the core.
- Whether the Windows App SDK runtime is bundled in the package
  (self-contained) or pulled as a Store dependency is chosen in step 2.
- Targets are x64 and ARM64 from day one (#86), both in a single
  `.msixbundle`.
- The minimum Windows version is chosen in step 2. The Windows App SDK
  supports Windows 10 1809 and later; Windows 10 left general support in
  October 2025. Windows 10 has no Mica: if Windows 10 is supported, the
  system background colour is used there and recorded in the Windows row of
  `tasarim.md`.
- Languages: 35 languages from a single source
  ([0008](0008-diller-ve-bolgeler.md)). The ARB source is converted to
  `.resw` during the build; plurals and numbers use ICU (Windows ships ICU
  itself). Right-to-left languages use `FlowDirection`; the transfer code
  and verification words stay left-to-right.
- During a transfer, the mark plays the same animation as on the website
  ([tasarim.md](../tasarim.md), "Motion"). The **Reduce motion** rule is
  wired on Windows to the "Animation effects" setting
  (`UISettings.AnimationsEnabled`): when off, the mark stays at rest.
- Every UI step is tested on Windows in the light, dark and high-contrast
  themes, with screenshots attached to the PR (like the rule on Linux,
  0006).
- Accessibility: every control has a UI Automation name; tested with
  Narrator and NVDA, with the keyboard alone and in high contrast; text
  scales up to 225% without overflow.

### System integration

- Right-click "Send with TotalDrop" in File Explorer: Windows 11's
  `IExplorerCommand` interface, using the MSIX package identity
  (`desktop4`/`desktop5:FileExplorerContextMenus` and `com:Extension`).
  This is a small COM server loaded inside `explorer.exe`; C# Native AOT is
  not suitable for it. It is written in Rust (`windows` crate), forwards
  only the selected paths to the app, and goes through its own security
  review: a bug crashes File Explorer. The legacy context menu is a
  fallback only.
- Acting as a Share Target in the Windows share sheet and sharing out from
  the app (`DataTransferManager`).
- Drag-and-drop onto the window and the taskbar icon, single-instance
  behaviour (when already open, a second launch hands the files to the open
  window), and actionable toast notifications.
- Preventing the computer from sleeping during an active transfer (#340)
  and what happens to a transfer when the window is closed follow the same
  rules as on Linux.
- Firewall: listening over UDP for QUIC triggers the Windows Defender
  Firewall prompt on first launch if no rule exists. The plan is an
  install-time rule declared in the MSIX manifest with
  `desktop2:Extension Category="windows.firewallRules"`, scoped only to the
  app's own binary and, if possible, only to the private network profile.
  **To verify:** whether this extension is accepted in a Store package and
  whether the rule is removed on uninstall.

## Distribution

- **Options:**
  1. Unsigned `.exe`/`.msi` on the release page. Free, but warns on every
     install; unacceptable for trust.
  2. Package signed with Certum's open-source certificate (today's plan,
     `don.b4`). Paid every year; the key lives on a smart card or in
     Certum's cloud key (SimplySign), and both make signing from CI hard.
     According to Microsoft, OV and EV certificates no longer grant
     immediate SmartScreen reputation; the warning persists until
     reputation builds up. Microsoft's own signing service (Artifact
     Signing) is open to individuals only in the US and Canada.
  3. SignPath Foundation's free open-source signing programme. The
     publisher appears as "SignPath Foundation", not TotalDrop (PR #347).
  4. **Microsoft Store with an MSIX package; Microsoft signs it.** winget
     installs the app from the `msstore` source using the same package (by
     Store product ID, `9N…`).
- **Proposed:** Option 4. There is no Windows package to download outside
  the Store. If option 2 or 3 is needed later, it is added in a separate
  decision.
- The `TotalDrop.TotalDrop` identifier in winget's community repository
  (PR #347) requires an installer URL; with no package outside the Store,
  that identifier cannot be published. Proposed: leave the identifier
  unclaimed and accept the risk of someone else taking it; if taken, report
  it to the `winget-pkgs` reviewers under the brand rule (0012).

### Rules

- The package is built only in CI, from a tagged commit, on GitHub's
  standard Windows runners (`windows-2025`, `windows-11-arm`)
  ([0007](0007-surec.md), [0009](0009-ios-macos.md) CI rules). Every PR
  builds the package and runs the tests.
- The Store does not rebuild our package; it only re-signs it. Because of
  that, the Store is not an exception to CI builds. However, the signature
  on the file delivered to users does not match our build provenance
  (attestation); provenance is issued for the `.msixupload` produced by CI
  and the binaries inside it.
- Like on Android, the release workflow (`windows-release.yml`) produces
  only an artifact; Ozan submits it to the Store through Partner Center
  ([surec.md](../surec.md) "Release checklist" and the header comment of
  `android-release.yml`: Ozan uploads the CI-built bundle to Play Console).
  **To verify:** whether the Store submission API or `msstore` CLI can be
  used on an individual account, which requires an Entra ID tenant and app
  registration. If it works, it is added later in a separate step behind an
  approved `windows-release` environment.
- Unsigned test package: for development and CI runs, the package is signed
  in CI with a throwaway self-signed certificate generated fresh on each
  run (or carries the unsigned-package publisher marker in the manifest and
  is installed with `Add-AppxPackage -AllowUnsigned`). This package is
  never distributed.
- The package identity (`Identity Name`, `Publisher`) comes from the name
  reservation in Partner Center and is committed to the repository; it is
  not a secret.
- **There is no sandbox.** A WinUI 3 desktop app runs with `runFullTrust`,
  carrying all of the user's privileges. The `internetClient`,
  `internetClientServer` and `privateNetworkClientServer` capabilities are
  enforced only inside an AppContainer; here they are declared but draw no
  boundary. There is no permission boundary on Windows comparable to
  Flatpak (T15); once accepted, this is recorded in the threat model.
  `runFullTrust` is a restricted capability; its justification is included
  in the Store submission.
- Updates are handled by the Store; the app has no updater of its own.

### Account

- **Individual account:** Free. Opened with a personal Microsoft account;
  verified with an identity document and a selfie. According to Microsoft,
  it is for those publishing "in their own name" and "not in connection
  with their business, trade, or profession". The name shown in the Store
  is the "publisher display name"; the account type, country and this name
  cannot be changed after the account is opened.
- **Company account:** Free. Requires a registered legal entity (a D-U-N-S
  number or official incorporation document) and a work email matching the
  domain. Team members can be assigned roles. TotalDrop has no legal entity
  today. The Open Collective application was declined; donations currently
  go through the project lead's personal page
  ([0017](0017-bagis-kisisel-sayfa.md)).
- An individual account cannot later be converted into a company account; a
  new account is opened for the company and the app is transferred to it.
- **Current state (7 November 2026, read from Partner Center):** Ozan has
  an existing account opened earlier. It is an individual account linked to
  Ozan's personal Microsoft account; the publisher display name and Windows
  publisher ID are his personal name, and the seller contact details are
  personal as well. As it stands, TotalDrop would appear in the Store under
  a personal name.
- **Ozan decides before the first submission** (picking the wrong name
  means opening a new account and repeating identity verification):
  - Whether the publisher display name on an individual account can be
    "TotalDrop". Microsoft's wording expects the person's own name;
    "TotalDrop" may be rejected or treated as against the terms.
  - Whether showing a personal name ("Ozan Özdil") as the publisher fits
    the brand rule ([0012](0012-ad-ve-isaret.md): only TotalDrop publishes
    TotalDrop). If not, the path is a company account once a legal entity
    exists, and the Store waits until then.
  - Whether the project fits the individual account definition
    (non-commercial): the app is free, ad-free and funded by donations.
- **Proposed:** Ozan tries changing the publisher display name on the
  existing account to "TotalDrop" and the seller contact email to
  `hello@totaldrop.app`. If Microsoft does not allow it, the Store waits
  for a company account opened under a legal entity. Ozan makes the
  changes himself; passwords and identity documents are never put in the
  repository or given to Claude. **To verify:** whether the publisher
  display name on an individual account can be changed after creation, and
  whether the Windows publisher ID (`Publisher` in the package) changes
  along with it.
- **Account protection:** The Store distributes whatever is submitted to it
  as an automatic update signed by Microsoft. Anyone who compromises the
  Partner Center account can push a malicious update that looks trusted to
  every user without touching any of our keys. Therefore: passwordless
  sign-in (passkey) or an authenticator app on the Microsoft account, with
  no SMS recovery; two-factor authentication and restricted access on the
  `hello@totaldrop.app` mailbox too, because it can recover the account.
  Who can recover the account is kept in writing. Because users cannot be
  added to an individual account, the account depends on one person; if
  Ozan is unreachable, releases stop.
- **To verify:**
  - The "trader" declaration Partner Center asks for under the European
    Union Digital Services Act. Declaring trader status makes Ozan's name,
    address, email and phone number public on the Store in the EU, which is
    a privacy risk for a one-person project; a false "non-trader"
    declaration is a legal risk. The declaration is Ozan's; how a project
    with a brand, website and donations falls under that test is read
    beforehand.
  - Whether users truly cannot be added to an individual account (secondary
    source).

### Name reservation

- According to Microsoft, a reserved name is held for at most three months
  without publishing; after that it expires. Steps 2–5 will very likely
  take longer than three months.
- Proposed: reserve the name "TotalDrop" roughly three months before the
  first submission, around step 4. Until then, the risk of someone else
  reserving the name is accepted; if taken, it is reported to Microsoft
  under the brand rule (0012).

### Store listing

- In 35 languages. The name, short description and description are
  generated from the same source that produces the AppStream file
  (`tools/desktop-entries.mjs`), never copied by hand. Windows-specific
  sentences (theme, File Explorer menu) are added to `l10n/` first.
  **To verify:** whether the Store accepts all 35 languages (especially
  `sw`, `ur`, `bn`, `fa`, `ms` and `zh-HK`/`zh-MO`/`zh-TW`) and under which
  locale codes.
- Privacy policy URL: `https://totaldrop.app/privacy/` (35 languages).
  Support: `hello@totaldrop.app`. Age rating via the IARC questionnaire.
- Screenshots in the light and dark themes, at least 1366×768, in several
  languages (including one right-to-left). No platform or manufacturer
  logos are used ([tasarim.md](../tasarim.md)).

## Order

1. Acceptance of this decision and Ozan's answers to the questions under
   "Account": either update the publisher name and contact details on the
   existing individual account to TotalDrop, or wait for a legal entity and
   a company account.
2. `windows/` folder: empty WinUI 3 window, core binding, x64 and ARM64
   builds in CI, test MSIX, unit tests; choose the binding tool, runtime
   distribution mode and minimum Windows version.
3. Send and receive screens, theme (Mica, title bar, high contrast), 35
   languages, accessibility; UI Automation tests in CI.
4. File Explorer menu (Rust COM server), share target, notifications,
   single-instance handling, firewall rule. Reserve the app name around
   this step.
5. Release workflow, Store listing copy, first submission to the Store
   (Ozan), Store package flights for closed testing.
6. Update the download page, README (35 languages), the `don.b4` item on
   the donate page ("Windows code signing certificate", 35 languages), the
   threat model (T12 and a new row for Windows) and the privacy policy (35
   languages: Microsoft Store collects install and usage data under its own
   responsibility).

## Risks

- Store certification may reject or delay a submission; according to
  Microsoft, each submission takes up to three business days. An urgent
  security fix waits through the same window.
- Compromise of the Partner Center account means a signed malicious update
  (see "Account protection" above).
- Users who want to install outside the Store (corporate machines with the
  Store disabled) cannot do so under this decision; if demand arises,
  option 2 or 3 is added in a separate decision.
- No local Windows machine: testing runs on CI runners with screenshots,
  supplemented by a Windows machine once one is available.
- If the publisher name is not settled before the first submission, the app
  ships under a personal name; fixing that later requires a new account and
  transferring the app.

## Consequences

- No budget is needed for a Windows code signing certificate; the `don.b4`
  item drops off the donate page (the Apple account for macOS remains).
- A Windows row is added to the platform table in
  [tasarim.md](../tasarim.md): colour source is the system accent colour
  with light/dark/high-contrast (`UISettings`, `ThemeSettings`); controls
  are WinUI 3 Fluent; window uses Mica and the system title bar. The
  Windows mark file joins the list of files sharing the same motion
  constants under "Motion".
- In the threat model: a note in T12 that "Microsoft re-signs the Store
  package, so users cannot verify our build provenance directly; the
  Partner Center account is a single point of trust", plus a row stating
  that Windows has no sandbox.
- The `windows/` folder and `windows.yml` / `windows-release.yml` workflows
  are created; `windows-app-engineering` is already listed for `windows/`
  in the area table in `CLAUDE.md`.
