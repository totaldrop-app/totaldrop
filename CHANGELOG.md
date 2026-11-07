# Changelog

All notable changes to TotalDrop are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/) as described in
[docs/versioning.md](docs/versioning.md).

## [Unreleased]

Entries for the next release are in [changelog.d/](changelog.d/), one file
per change, and move here when it is released (docs/surec.md).

## [linux-0.3.0-beta.6] - 2026-11-07

### Added

- **Linux:** The release now also has a Flatpak package for x86_64 and
  ARM64, which installs without touching the system, also where the
  system is read-only. It takes the KDE runtime from Flathub and reaches
  the network, `~/Downloads/TotalDrop`, the files you pick, desktop
  notifications, Hyprland's socket and your desktop's theme files (read
  only), and nothing else of your home folder (#297).
- **Linux:** "Send with TotalDrop" in the right-click menu of Dolphin,
  Files (Nautilus, once nautilus-python is installed) and Nemo, for any
  selection of local files and folders; "Send files" and "Receive files"
  when you right-click TotalDrop in the taskbar or app launcher; and the
  app's description and search words in every language of the app (#303).
- **Linux:** Settings says, in every language, that codes and
  verification words stay in English so that two devices set to different
  languages show the same words (#305, #52).
- **Command line:** Send a text instead of files, such as a link:
  `totaldrop send --text "…"`, or `totaldrop send -` for the text on
  standard input (at most 1 MiB). The receiver prints it instead of
  saving it; an older receiver saves it as `text.txt` (#313).
- **Linux:** Software centres such as Discover, GNOME Software and
  Flathub describe TotalDrop in all 35 languages of the app, not only in
  English and Turkish (#324).
- **Linux:** Send a text instead of files: "Send text" on the Send
  screen, or "Send clipboard" in the app's launcher menu, which opens Send
  with the clipboard's text filled in. A pasted text is sent exactly as
  it was copied. A text that arrives is shown, not saved, with any hidden
  characters named, and "Copy text" copies it exactly as it came. A very
  long text shows only its beginning; all of it is sent and copied
  (#326).

### Fixed

- **Linux:** The right-click menu of text fields (Undo, Cut, Copy, Paste,
  Select All) is in the app's language where Qt draws it (Qt 6.11 and
  later), and Qt's own windows follow right-to-left languages. KDE's style
  keeps its own translation where it has one (#307, #52).

### Security

- **Command line:** When a send or receive fails, the detail of the error
  names control and formatting characters (`<U+001B>`) instead of passing
  them to the terminal. A reason the other device gives for cancelling or
  for closing the connection, or a message from a server, could otherwise
  move the cursor, rewrite earlier lines or set the window title
  (GHSA-c6jh-7p7q-8gg4).
- **Linux, CLI:** A sending device no longer connects to its relay
  when the receiving device uses a different relay or none, so that relay
  no longer sees the sender's IP address for a transfer it could never
  carry. A receiver on 0.1 set to another relay, which used to fall back
  to the sender's relay anyway, now needs a direct path (#301).

## [android-0.3.0-beta.7] - 2026-11-07

### Added

- **Site, README:** TotalDrop is on X as @totaldropapp. The site's page
  footers (all but the privacy policy's short one) and every README link
  to it, and the brand page lists the project's official accounts on the
  web and social media so that impostors are easier to spot (#284).
- **Site, README:** TotalDrop is on Instagram and Threads as
  @totaldropapp. The site's page footers (all but the privacy policy's
  short one), every README and the brand page's list of official
  accounts link to them (#286).
- **Site:** A team page at totaldrop.app/team shows who builds TotalDrop
  and, discipline by discipline, who worked on which part, with profile
  pictures and figures taken from the project's public history on GitHub.
  The AI assistant that writes much of the code is named openly. The page
  loads nothing from GitHub; the site's page footers (all but the privacy
  policy's short one) link to it (#288).
- **Android:** Settings says, in every language, that codes and
  verification words stay in English so that two devices set to different
  languages show the same words (#305, #52).
- **Android:** Share files to TotalDrop from any app, such as Gallery or
  Files: one or many, of any type. TotalDrop opens Send with them listed.
  Holding the app icon offers Send and Receive (#308).
- **Site:** The team page lists QA specialists too, with the platforms they
  test, starting with one for iOS and macOS (#309).
- **Android:** Send a text instead of files: "Send text" on the Send
  screen, a text shared from another app, "Send with TotalDrop" in the
  menu of text selected in any app, or "Send clipboard" when the app icon
  is held. The text opens in Send to be looked at first, with any hidden
  characters pointed out, and is sent as it is. A text that arrives is
  shown, not saved, with any hidden characters named, and "Copy text"
  copies it exactly as it came. Very long texts are best pasted: another
  app can hand over less than the 1 MiB limit (#326).

### Changed

- **Site, README:** The download page and the README point at Android
  0.3.0-beta.6 and list what is new: the send screen says at once that it
  is preparing a large file, says when the chosen files cannot be added,
  and TalkBack reads out each new step. The Linux app stays at 0.3.0-beta.5
  (#277).
- **Site:** The team page is easier to read: key figures at the top, people
  as cards with icon links to GitHub and their own sites, AI and automation
  in their own section, and a bar showing each discipline's share of work.
  Pages now ask for the stylesheet and scripts by version, so a browser
  never shows a new page with an old stylesheet after an update (#291).

### Fixed

- **Relay (Cloudflare):** The relay now bounds what it holds for a device
  that does not read: before, a modified client could make it keep any
  amount in memory. Devices from before 0.3.0-beta.4 lose a transfer
  through this relay after 32 MiB reaches them: a receiver after 32 MiB
  of the file, a sender later (an estimated 1 GiB of file, since only
  acknowledgements reach it). Update both devices (#190).
- **Site:** The language menu is readable in the dark theme. Its list
  opened on a white background with light text, so most languages could
  not be read (#293).
- **Android:** The heading above the verification words wraps onto a
  second line instead of running off a narrow screen, with larger text or
  in a language with a long heading (#315).

### Security

- **Android:** A sending device no longer connects to its relay
  when the receiving device uses a different relay or none, so that relay
  no longer sees the sender's IP address for a transfer it could never
  carry. A receiver on 0.1 set to another relay, which used to fall back
  to the sender's relay anyway, now needs a direct path (#301).
- **Android:** A share reads only files that another app serves
  (`content://`). Addresses served by TotalDrop itself, `file://`
  addresses and addresses with a user prefix are refused, so another app
  cannot make TotalDrop send its private data, such as its device key.
  Copies of shared files are kept out of backups and deleted when the
  selection is cleared, or at the latest when the app starts again
  (#308).

## [android-0.3.0-beta.6] - 2026-11-05

### Changed

- **Site, README:** The download page and the README point at Android and
  Linux 0.3.0-beta.5 and list what is new: on Android, transfers that
  survive a short network drop, Material 3 Expressive icon buttons and a
  smaller mark on the home screen icon; on Linux, packages that are easier
  to check. The brand page describes the smaller mark and the Expressive
  icon buttons (#267).

### Fixed

- **Android:** After you choose a large file, the send screen now says
  at once that it is preparing your files, instead of standing still for
  a few seconds while the app copies the file it was given (#269).
- **Android:** When the file picker cannot hand over the files you chose,
  for example because there is not enough free space for its copy, the
  send screen now says so instead of quietly going back to an empty
  selection. TalkBack now also reads out the main headline of each step
  of a send (preparing, waiting, sending, sent, failed) and of a receive
  (connecting, incoming files, receiving, received, declined, failed),
  and the selection once the picker has copied the files (#271).

## [android-0.3.0-beta.5] and [linux-0.3.0-beta.5] - 2026-11-05

### Added

- **Licensing:** Every file states its copyright and license, following
  REUSE 3.3: source files carry two SPDX lines, everything else
  (documents, build and configuration files, images, translations,
  generated and copied files) is covered by `REUSE.toml`, and CI checks
  both (#249).
- **CI:** A workflow builds each Linux package twice, from two clean
  checkouts, and fails if the two differ. It runs weekly, by hand, and on
  pull requests that change the packaging. The one known difference, the
  order cxx-qt links the Qt libraries, is reported as a warning
  (#252, #256).

### Changed

- **Licensing:** The translations in `l10n/` are now MIT OR Apache-2.0,
  like `totaldrop-l10n`, which compiles them into itself (#249).
- **Site, README:** The download page and the README point at Android and
  Linux 0.3.0-beta.4 and list what is new: the move to the relay when a
  direct connection goes quiet, relay transfers that reconnect, sender
  progress that shows what has arrived, and failure reasons in your
  language (#254).
- **Android:** Icon buttons (Settings, Paste, Remove and Back) follow
  Material 3 Expressive, like Android 16's own interface on Pixel: they
  turn into a rounded square while you press them. With "Remove
  animations" on, the change happens at once (#257).
- **Android:** The mark on the home screen icon is smaller, so its corner
  no longer seems to touch the edge of the icon's shape (#261).

### Fixed

- **Linux:** The Debian package's copyright file lists `totaldrop-l10n`
  and the fuzz targets under MIT OR Apache-2.0 and no longer repeats its
  license paragraphs (#231, #249).
- **Linux:** The desktop app's embedded Qt resources no longer carry the
  build time of the files cxx-qt generates, so they are the same on every
  build of the same source. One difference remains, the order in which
  cxx-qt links the Qt libraries (#252, #256).
- **Android:** A transfer no longer ends at once when the phone loses its
  network, for example in airplane mode or a tunnel: if the network comes
  back within about half a minute, the transfer carries on (#260).

## [android-0.3.0-beta.4] and [linux-0.3.0-beta.4] - 2026-11-04

### Added

- **Site:** The privacy policy is now in all 35 languages of the site, each
  at its own address (`/privacy/de`, `/privacy/ja`…). Every translation says
  that the English text is the authoritative one, and the site's Privacy
  links open the policy in the language you chose (#207).
- **Site:** A brand page at totaldrop.app/brand shows the mark and how it
  turns, the logo and app icon, the colours, type, buttons and other
  components, spacing, icons and voice, with what not to do, and offers
  the logo files to download, in all 35 languages (#220).
- **CI:** The OpenSSF Scorecard rates the repository's supply-chain
  practices on main and every week and publishes the result; the README
  shows its badge in every language. CodeQL code scanning, on in GitHub's
  default setup, reads the Rust, C++, JavaScript and TypeScript code and
  the workflows (#225).
- **README:** The README shows the OpenSSF Best Practices badge, at the
  passing level, in every language (#232).
- **Site:** The brand page at totaldrop.app/brand says how others may use
  the TotalDrop name and mark: writing about it, packaging it unchanged,
  building compatible software and publishing a changed version under its
  own name, with what is not allowed, in all 35 languages (#234).
- **Site:** The footer of the site's main pages links to the project's
  OpenSSF Best Practices page (#236).
- **Linux:** Each release also carries its signed build provenance as a
  `.sigstore.json` file, so the packages can be checked without GitHub's
  attestation API (#238).
- **Security:** A `SECURITY-INSIGHTS.yml` file describes, in a form tools
  can read, how to report a vulnerability, how releases are built and
  which security checks run (#238).
- **Docs:** GOVERNANCE.md describes how decisions are made, who holds
  which role and what is still missing for the project to carry on
  without its single maintainer (#241).
- **Docs:** CONTRIBUTING.md names the coding standard for each language
  and the tool that enforces it, the code review checklist, and where new
  contributors can start (#241).
- **Docs:** SECURITY.md describes how a vulnerability report is handled,
  step by step, and what TotalDrop does and does not protect (#241).
- **Docs:** docs/mimari.md describes the architecture, and the threat
  model gains an assurance case: why the security requirements hold
  (#241).
- **CI:** Line coverage of the protocol crates, the command line and the
  Rust servers is measured with cargo-llvm-cov, and CI fails below 90%.
  New tests run the `totaldrop` program itself: sending a folder,
  answering the question, declining, and its error exits (#245).

### Changed

- **Site:** The turning mark on the home page now has a desktop, a phone,
  a laptop and a tablet around it, and the one its corner points at
  lights up: every device to every device. With reduced motion everything
  stays still (#209).
- **Android, Linux, CLI:** When a transfer fails, the reason is now given in
  the device's language for many failures, for example that the connection
  stopped responding or that the disk is full; on Android and Linux it is
  under Details. The technical message stays in English for problem
  reports, and on Android and Linux it is labelled as such (#212).
- **Site, README:** The download page and the README point at Android and
  Linux 0.3.0-beta.3 and list what is new: one window on Linux, relay
  transfers that connect and reconnect, transfers that no longer stop right
  after the code, Stop that ends the transfer at once, and Android
  transfers that keep going in the background (#213).
- **Linux:** The licence texts moved to `LICENSES/`, named by their SPDX
  identifiers. The Arch and Fedora packages install them as
  `GPL-3.0-or-later.txt`, `MIT.txt` and `Apache-2.0.txt` under
  `/usr/share/licenses/totaldrop/` (#229).
- **Build:** The Linux build images check rustup-init against a fixed
  sha256 before running it, instead of piping an install script to the
  shell (#238).
- **CI:** OSV-Scanner, which OpenSSF Scorecard uses, ignores the same
  three unmaintained-crate advisories as cargo-deny, and CI checks that
  the two lists agree (#238).
- **Site:** The roadmap now describes the planned optional account as one
  the server never keeps, held by your own devices, instead of a random
  number (#243).

### Fixed

- **Android, Linux, CLI:** A transfer that started on a direct connection
  no longer stops with "timed out" when that path goes quiet, for example
  when a phone's Wi-Fi drops: after 8 seconds without anything from the
  other device, it moves onto the relay and goes on. It goes back to the
  direct connection only if the other device is still sending over it;
  otherwise it stays on the relay until it ends. Both devices must use
  the same relay (#185).
- **Android, Linux, CLI, relay:** A transfer through the relay survives a
  connection that dies without either end noticing, for example after
  switching Wi-Fi networks: the device notices within 10 seconds and
  connects again, and the relay now accepts the new connection at once
  instead of refusing it until the transfer times out. Both relays now end
  the other side in the same cases (#190).
- **Android, Linux, CLI:** The sending device's progress bar shows what has
  reached the other device, instead of running ahead and showing a file as
  sent while it is still on the way. When the other device runs an older
  version, the bar shows only what has surely arrived, so it may stay
  still until a file has arrived whole (#211).
- **Site:** totaldrop.app now has a site icon that search engines and
  older browsers can read, `/favicon.ico` beside the SVG, and an icon for
  the iPhone and iPad home screen. Google showed a plain globe next to the
  site in its results (#227).

## [android-0.3.0-beta.3] and [linux-0.3.0-beta.3] - 2026-11-04

### Changed

- **Site, README:** The download page and the README point at Android and
  Linux 0.3.0-beta.2 and list what is new: the GTK theme on GNOME and on
  Hyprland without a Qt theme, clearer code and server messages, pasted
  share messages and keyboard use on Linux (#187).

### Fixed

- **Linux:** Opening files with TotalDrop while it is already open, for
  example with "Open with" in a file manager, adds them to the open
  window instead of starting a second one; a link or `--screen` does the
  same, and the window comes to the front. During a transfer, or while
  its result is shown, the request waits until the window is back to
  idle; a screen with files listed, a code typed or a server address not
  saved stays under the new one. The window owns the D-Bus name
  `app.totaldrop.TotalDrop` and serves `org.freedesktop.Application`. A
  relative path given on the command line now opens too (#178).
- **Android, Linux, CLI:** A transfer through the relay connects even when
  the relay is slow to get going, instead of failing after 20 seconds, and
  the connection to the relay is made again when the relay closes it. It
  is also faster when the round trip to the relay is long, because more
  data may be on the way at once (#185).
- **Android, Linux, CLI:** A transfer no longer sometimes stops right after
  the code is entered, with "the other side left" on the sending device
  while the receiving device waits. The device that finished the code
  exchange first left the server too quickly, and its last message was
  sometimes lost (#191).
- **Android, Linux, CLI:** Stop on the sending device now cuts the transfer
  at once. Before, files whose data was already on the way could still
  arrive in full and be kept on the receiving device (#193).
- **Android:** A send or receive now keeps going when you switch to
  another app, for example to share the code. Before, Android cut the
  connection and the code stopped working. While a transfer runs, a
  notification shows it, with a Stop button (#194).

## [android-0.3.0-beta.2] and [linux-0.3.0-beta.2] - 2026-11-03

### Added

- **CI:** The site and the rendezvous and relay Workers deploy from CI, from
  main only. The commit's checks run again, then the owner approves the run
  in the `cloudflare` environment. Only what changed is deployed. A probe
  then opens a real WebSocket to each Worker, reaches the rendezvous
  directory without taking a number, and loads the site's pages.
  docs/surec.md describes the one-time setup and how to roll back (#94).

### Changed

- **Build:** A Linux release now checks the version of the command line it
  ships against every earlier Linux or `cli-v*` release. A version higher
  than all of them passes and a lower one stops the release. The same
  version stops it if the CLI's code has changed since a release that
  shipped it: `crates/` without its `tests/` and `benches/` directories,
  and the strings and translation status the CLI compiles in. `Cargo.lock`
  and the root `Cargo.toml` are not counted. The check is
  `packaging/check-cli-version.sh`, run by `check-versions.sh` on a
  release tag and tested in CI under bash and dash (#115).
- **Android, Linux:** The note in Settings for a language that is still a
  draft asks for help on GitHub instead of Weblate, which is not set up yet.
  The note is updated in all 37 language files (#131).
- **Site, README:** The download page and the README point at Android and
  Linux 0.3.0-beta.1 and list what is new: the Köşe mark and icon, 35
  languages, right-to-left screens and steadier QR reading on Linux. The
  apps are no longer described as English and Turkish only, and the
  roadmap adds "In your language" to Done (#132, #137).
- **Server:** The self-hosted rendezvous and relay servers carry their own
  version, starting at 0.1.0-beta.1, instead of the command line's; raising
  the command line's version no longer renames them (#146).
- **Android, Linux:** While connecting, the receive screen shows the code
  as it was read, so the code that is used can be seen even when the
  pasted text looks different, for example with a direction mark (#149).
- **CLI:** `totaldrop receive` says which code it connects with, for
  example "Connecting with 7-harbor-rocket…", so a code pasted with
  direction marks that show it in another order can be checked before it
  is used (#149).
- **Process:** Every pull request records its change in a file of its own
  in `changelog.d/` instead of editing CHANGELOG.md, so pull requests no
  longer conflict with each other there. `tools/changelog.mjs release`
  moves the files into CHANGELOG.md when a release is made, and CI checks
  every file and fails a pull request that adds none (#150).
- **Process:** A release of one platform takes only the changelog entries
  that name it or no platform; an entry for both platforms is split, so
  nothing has to be moved by hand before a Linux-only or Android-only
  release (#152).
- **Android (CI):** CI checks that the app's hand-written Dart code is
  formatted with `dart format` (#155).
- **Linux:** Photos, videos, music, archives and documents in the Send
  list get an icon for their kind instead of a generic one (#169).

### Fixed

- **Core, command line:** `totaldrop --version` printed 0.1.0-beta.2 in
  the Linux 0.2.0-beta.1 and 0.3.0-beta.1 packages, although the command
  line had since gained the relay, privacy mode (`--relay-only`),
  translations and a clean stop with Ctrl-C. The command line and the core
  libraries are now 0.2.0-beta.1; the self-hosted servers and the Android
  app's Rust library share that version and move with it (#115, #146).
- **Core, CLI (tests):** Two tests failed now and then on slow CI runners.
  The end-to-end test of a receiver stopped while it connects now stops it
  before any reply from the server can arrive, so it always ends in
  "stopped" and no longer sometimes reads the departed sender's mailbox
  and ends in "wrong code". The privacy-mode test now runs on a paused
  clock and checks that the relay connection comes up before the sender's
  own dial would start, instead of timing the connection on the wall clock
  (#126).
- **CLI, translations:** The translation check rejected only the English
  no-words (n, no, not) among a language's yes answers (`cliYes`), so a
  draft could make a word for no, such as нет, nein or لا, mean yes. Each
  language now lists its words for no in `l10n/status.json` (`noWords`),
  and the check rejects a yes answer that is one of them, or the answer
  for no in a save prompt it can be shown with (`[д/Н]`, `[ن/ل]`). A save
  prompt must show ASCII `[yes/no]` answers, and its answer for yes must
  be in `cliYes` or be y or yes. Answers are compared exactly as the
  command line compares them: `fold_answer`, `is_yes` and `ALWAYS_YES` are
  now public in `totaldrop-l10n`, and the command line uses them (#129).
- **Android, Linux, CLI:** In a right-to-left language the shared message
  could show the code out of order, with its number on the wrong side of
  its words; the code is now isolated, as the version and the save folder
  are. Both apps drop direction marks and other invisible characters, such
  as a zero-width space, from a pasted code, and Linux also from a
  totaldrop:// link; the command line does the same with a code given to
  it, so a code copied with them still connects (#130).
- **Android, Linux, CLI:** A code typed or pasted with the digits of an
  Arabic, Persian or Urdu keyboard, such as ٧ or ۷, was rejected as
  invalid; these digits are now read as their ASCII counterparts (#147).
- **Android, Linux:** Pasting the whole share message, such as "Get my
  files with TotalDrop. Code: 7-abandon-ability", into the receive field
  gave "invalid code" on Linux, and on Android when it was pasted with the
  system menu instead of the paste button; both apps now find the code in
  the message, in every language (#148).
- **Linux:** The connecting, preparing and empty send screens are
  centred in the window instead of sitting in a narrow column on the
  left (#159).
- **Android, Linux:** A code that no sender is using (mistyped, expired
  or already used) now says so, as the command line does, instead of
  telling the person to check their internet connection (#160).
- **Linux:** Escape goes back on Send, Receive and Settings, and on the
  result screen it is Done; it does not drop files not yet sent or skip
  past an unsaved server address. Enter presses the focused button on the
  result screen, Details included, and holding it presses only once.
  Going back from a finished or failed transfer no longer shows the same
  result the next time Send or Receive is opened (#162).
- **CLI:** When the answer to "save?" comes from a pipe or a file, the
  question ends its own line, so the progress no longer follows it on the
  same line (#163).
- **Android, Linux, CLI:** When nobody comes for a code before it
  expires, the sender is told so and asked to send again, instead of
  being told to check the internet connection or to ask the sender for a
  new code (#164).
- **Android, Linux:** A server address that is not valid is named as
  such, as the command line does, instead of "could not reach the
  server" (#164).
- **Linux:** In a wide window the home screen, Settings and the Send list
  no longer stretch across the whole width but stay centred like the other
  screens; the waiting screen's code card is centred too (#169).
- **Linux:** Settings shows its scroll bar while the page is taller than
  the window and scrolls to the control reached with Tab; the
  totaldrop.app link and "Add files" in the Send list can be reached with
  Tab, and a result screen puts the keyboard on Done (#169).
- **Linux:** Opening Details on a result no longer moves the button, the
  stop question dims the window behind it under the Fusion style, and it
  closes by itself when the transfer ends meanwhile (#169).
- **Linux:** With some Mesa drivers, seen on Intel graphics, the window
  was drawn with 16-bit colour, so the background and accent were a shade
  off from the desktop theme; the window now asks for full 24-bit colour
  (#170).
- **CLI:** `totaldrop --help` is now in the system language, like every
  other message of the command line (#171).
- **Linux:** `totaldrop-desktop --help` and `--version` print the usage or
  the version and exit instead of opening the window (#171).
- **Linux:** On GNOME and on Hyprland without a Qt theme, the window takes
  the GTK theme's colours (Adwaita, your GTK theme, or your own `gtk.css`)
  instead of TotalDrop's lavender, and follows a theme change without a
  restart. Outside Plasma (GNOME, Hyprland, Omarchy) buttons, fields and
  switches take GTK's shape and rounding, and icons stay visible in a dark
  theme. Omarchy keeps its own theme's colours (#174).
- **Linux:** In a short window the Send code screen shows its scroll bar
  and Tab brings Cancel into view. Under Fusion, "Add files" shows
  keyboard focus. Under Plasma, Accept is marked as the default button
  (#175).

## [android-0.3.0-beta.1] and [linux-0.3.0-beta.1] - 2026-10-30

### Added

- **GitHub:** The README is rewritten around the new mark, with a light and
  dark lockup, current screenshots and a section on languages, and it is
  translated into the site's 35 languages (docs/readme/), each listed under
  its own name. CI checks every translation's links and language list and
  reports translations older than the English. A code of conduct and a
  support page complete the community files (#135).
- **Android, Linux, CLI, site:** Draft translations in 31 more languages, 33
  with the Portuguese and Chinese variants (ADR 0008): Arabic, Bengali,
  Chinese (Simplified and Traditional), Czech, Danish, Dutch, Finnish,
  French, German, Greek, Hebrew, Hindi, Hungarian, Indonesian, Italian,
  Japanese, Korean, Malay, Norwegian Bokmål, Persian, Polish, Portuguese
  (Brazil and Portugal), Romanian, Russian, Spanish, Swahili, Swedish, Thai,
  Ukrainian, Urdu and Vietnamese. They are made with Claude and not yet
  checked by native speakers, so the apps say so in Settings and keep the
  security texts in English until a language is reviewed: verification,
  wrong code, invalid offer, accept and decline, and the command line's save
  prompt and yes answer. The site shows a short note at the top of the page
  in a draft language, lists every language under its own name, and picks
  Portuguese from Portugal outside Brazil, as the apps do; the picker is as
  wide as the chosen language. The comparison table now says no copy is
  stored on a company's server, instead of "files stay off company servers",
  which the relay made too broad (#127).

### Changed

- **Android, Linux, site:** New mark, "Köşe" (corner), replacing "Ağ" on
  Android and "Dalga" on Linux: a circle with one square corner. The circle is the
  whole file and the corner makes it a drop, pointing the way it goes. While
  a file is on its way the mark turns a quarter at a time, 3.2 s a loop, the
  same on the site, Android and Linux, and it stays still when the system
  asks for less motion. The app icons are the paper mark on violet; the done
  screen shows a check inside the mark (#133).
- **Site (download page, roadmap, home), README:** The download page and
  the README point at Linux 0.2.0-beta.1, with the ARM64 .deb and .rpm,
  and the page lists what is new in Android 0.2.0-beta.2 and Linux
  0.2.0-beta.1. The roadmap moves privacy mode in the apps and Linux on
  ARM to Done, and the home page's summary shows privacy mode in the apps
  as done (#122).
- **Site:** Privacy policy version 4, for Android and Linux 0.2.0-beta.1
  and later: devices contact the relay only when no direct connection is
  chosen about 1.2 seconds after they start connecting, or at once when
  there is no direct path to try; a receiver no longer uses a relay only
  the sender chose; privacy mode is in the apps' Settings, skips STUN and
  never connects directly; the apps also store the privacy mode setting,
  and Android keeps the files you pick in its cache until the send
  is closed or stopped. Android 0.1.0-beta.5 and Linux 0.1.0-beta.4, the
  first versions with the relay, are still described, with the advice to
  update (#124).
- **Linux, Android:** Dependencies: zbus 6.1, toml 1.1, dirs 7 and rqrr
  0.11 (QR codes with mixed block sizes now decode) in the Linux app; the
  Android Gradle plugin 9.5.

### Fixed

- **Android:** In right-to-left languages the home screen's mark, the
  "Add files" and "Use the default" buttons stayed on the left, the
  version showed as "beta.2-0.2.0", and the server address ran right to
  left. They now follow the language's direction, and the version and the
  save folder keep their own (#127).
- **Rendezvous (Cloudflare):** About 1 in 15 new codes failed with HTTP 500
  when Cloudflare returned a transient error on a new mailbox's first
  request. Opening a mailbox and looking up a code are now tried again on a
  fresh stub, and if every try fails the client gets a readable error
  instead of a 500 (#119).

## [android-0.2.0-beta.2] - 2026-10-29

### Changed

- **Android, Linux, site:** New mark, "Ağ" (network): three equal devices,
  each joined to the other two, and a drop on the way between them. While
  a transfer runs the drop visits each device in turn; with reduced motion
  it stays still. It replaces "Dalga", whose waves spread from one centre
  and did not show that every device sends to every other (#116). The
  Linux app gets it with its next release.

## [android-0.2.0-beta.1] and [linux-0.2.0-beta.1] - 2026-10-29

### Added

- **Android, Linux:** Privacy mode in Settings ("Hide my address from the
  other device"): every transfer goes through the relay, so the other device
  never learns your IP address. It is slower, and the app asks before
  turning it on. The command line has had it as `--relay-only` (#39).

- **Linux:** .deb and .rpm packages for ARM64 (Snapdragon X, NVIDIA RTX
  Spark). The command line is now also tested on ARM64 Linux and Windows
  (#86).

- **Command line:** Speaks the system's language (`LANGUAGE`, `LC_ALL`,
  `LC_MESSAGES`, `LANG`) and writes numbers and sizes in the region's format
  (`LC_NUMERIC`), for example "1,5 MB". English and Turkish today; more
  languages follow. Right-to-left languages stay English in the terminal,
  and `--help` and detailed error messages are still English.
- **Command line:** File names from the other side show hidden characters
  (controls, direction and other format characters) as `<U+202E>`, so a
  name cannot disguise its extension.
- **Linux:** The app takes its text from `l10n/app`, the same strings as
  Android, in the desktop's language, with sizes and counts in the region's
  format. The layout is ready for right-to-left languages (it mirrors, and
  the code and the verification words stay left to right); tested with a
  pseudo-locale until the first such translation lands. File names and
  cancel reasons from the other side show hidden direction characters.
  Settings says when a translation is still a draft. Packages recommend the
  Noto fonts for scripts beyond Latin.
- **Linux:** Sizes now read like on Android: one decimal below 10 and SI
  "kB" ("12 kB", "1.5 MB"), where they had one decimal below 100 and "KB".
- **Android:** Chooses the language like the Linux app and the command line
  (once those languages exist, `zh-HK` will get Traditional Chinese and
  `pt-AO` European Portuguese), and
  lists the app's languages in Android 13+ per-app language settings. Sizes
  follow the device's region ("1,5 MB"), counts the app language
  ("1.234 öğe"). The code and the
  verification words stay left to right and are read by screen readers with
  an English voice. Names from the other side show hidden direction
  characters. Safety-critical text stays English while its translation is
  an unreviewed draft, and Settings says when a translation is a draft.
- **Site:** A language picker replaces the EN/TR button; the language
  follows the browser (`navigator.languages`, with likely scripts, so
  `zh-HK` would find Traditional Chinese) and translations load from
  `/i18n/<lang>.json` on the same domain. Pages are ready for right-to-left
  translations (CSS logical properties; the direction comes from the
  language's script), with codes and commands kept left to right; tested
  with a temporary Arabic file until a real one lands. English text shown
  inside another language keeps its own language and direction. Receive links accept Turkish İ/ı and
  Arabic-Indic digits in the code.
- **Core:** `totaldrop-l10n`, the shared interface text for the Rust
  programs: all strings live in `l10n/`, one source with Android.

### Fixed

- **Relay (Cloudflare):** The 32 GiB limit per connection no longer starts
  again from zero when the relay object sleeps; the count is kept with the
  connection, losing at most 1 MiB each time the object wakes (#110).

- **Command line:** Failures are explained in the system's language, in the
  same words as the apps, with the technical detail on a second line. A
  code that is not in use and a bad server address get their own message
  instead of "check your internet connection" (#106).

- **Linux:** Adding a large folder no longer freezes the window: its size
  is added up in the background and shows "Counting…" until then (#103).

- **Android, Linux:** Stop works at every stage: while the files are
  prepared, while waiting for the receiver, while connecting and during the
  transfer. Before, stopping before the transfer began did nothing and the
  other side later saw a stale offer or a connection error; now it is told
  that the transfer was stopped, and leaving an unanswered offer declines
  it (#87).

- **Command line:** Ctrl-C stops a transfer cleanly: once connected, the
  other side is told and unfinished files are removed; a second Ctrl-C quits
  at once and may leave them. Ctrl-C at the "Save?" question declines.
  `receive` checks that the destination exists, is a folder and can be
  written before it uses up the code (#87).

- **Core:** Stopping a transfer now tells the other device, which reports
  that the other side stopped instead of a connection error. A stopped or
  failed receive no longer keeps writing in the background: unfinished
  files are removed with the empty folders the transfer created, and no
  file is placed afterwards. A sender waits at most 10 minutes for the
  receiver's answer (#87).

- **Relay (self-hosted):** The Caddy configuration now passes
  `/v0/relay/<keys>` to the relay; it matched only the exact path
  `/v0/relay`, so no relay connection could open. The server's name is now
  an argument of `install.sh` instead of `relay.totaldrop.app` (#84).
- **Relay:** A device signing in no longer pushes back the sign-in
  timeout of the other socket in the same pair, and a finished pair leaves
  no alarm behind (#75).
- **Core:** The apps now connect to the relay only when it is needed: when
  a device has no direct connection 1.2 seconds after pairing, or at once
  in privacy mode. A transfer whose direct path is up within that time no
  longer reaches the relay at all, so the relay no longer sees the IP
  addresses of devices it does not carry; when a direct path comes up later
  and wins, the relay connection is closed as soon as the direct path is
  chosen (#63).
- **Core, CLI:** A receiver now uses the relay the sender names only when
  it is also the receiver's own relay setting, so a sender running its own
  relay can no longer learn the receiver's IP address that way. An empty
  `--relay` turns the relay off when receiving too. When different relays
  leave no path (or in privacy mode), the receiver stops at once with an
  error that says so, and so does the sender when it can tell, instead of
  timing out. A sender with this version no longer pairs with an older
  receiver that would have used a relay it does not use itself (#68).
- **Site:** The privacy policy (version 3) now says that the apps connect
  to the relay at the start of every transfer, not only when no direct path
  works, as version 2 said, and that in privacy mode a receiver still uses
  the relay the sender names. The android-0.1.0-beta.5 notes above said the
  relay sees "only single-use keys, sizes and timing"; it also sees both IP
  addresses. A coming version will connect only when needed (#63, #68).
- **Android:** The privacy text in Settings said files always go directly
  between the devices and only the rendezvous server sees your IP address;
  it now names the relay, STUN and the other device too (#63).

### Security

- **Rendezvous (Cloudflare):** A connection to a mailbox that has not
  claimed a side within 10 seconds is closed, and at most two such
  connections wait at once (a newer one replaces the oldest); before,
  anyone who knew a nameplate in use could hold idle connections on it until
  it expired. The directory hands out
  nameplates from a pool instead of reading every nameplate on each request
  (GHSA-r9gc-m8c8-2xqw).

- **Servers (self-hosted):** The Rust relay accepts only the pair path and
  forwards datagrams only to the other key of the pair, as the Cloudflare
  relay does; a ping-only connection is closed after 60 seconds without
  datagrams. The Rust rendezvous server closes a connection that does not
  allocate or claim within 10 seconds, pings or not, and counts a failed
  code once instead of twice toward the attack-wave threshold
  (GHSA-57x6-rcg9-m954).

- **Core, command line:** In privacy mode (`--relay-only`, and the apps'
  new setting) a device refuses a direct connection even from the paired
  device, which could only dial an address it learned some other way; the
  transfer stays on the relay (#39).

- **Build:** A Linux release starts only from a tag on `main` whose version
  matches `linux/Cargo.toml`, the PKGBUILD, the RPM spec, the Debian
  changelog and the AppStream metainfo; every PR checks that the files
  agree. All packages are built with the pinned Rust (the Arch package used
  Arch's own), the Android bundle with a pinned Flutter, from base images
  pinned by digest, and the source archive carries the last commit's time
  (#90).

- **Build:** Linux packages are built only in CI from a release tag and
  carry signed build provenance, so anyone can check that a package came
  from this repository: `gh attestation verify <file> --repo
  totaldrop-app/totaldrop`. The signing workflow is
  `.github/workflows/linux-release.yml`. The Android Play bundle is also built and
  signed with the upload key in CI; Google Play re-signs it, so for
  Android the provenance is for the project's own records.
- **Build:** Every GitHub Action in CI is pinned to a full commit SHA, so
  a moved action tag cannot change which action code runs. Dependabot proposes the
  updates, which are reviewed like any other change. `actions/checkout`
  moved to v7 and `actions/setup-java` to v6.
- **Build:** One `CI` check now runs on every pull request and is required
  to merge into `main`. It runs the workflows of the areas that changed and
  fails if any of them fails. Releases build from their own workflows,
  which run the same checks first.

## [android-0.1.0-beta.5] and [linux-0.1.0-beta.4] - 2026-10-28

### Added

- **Core:** transfers across the internet. Each device asks a STUN server
  (Cloudflare's by default) for its public address and both try to reach
  each other directly (NAT traversal); when no direct path opens, the
  encrypted QUIC packets go through the sender's relay over WebSocket on TCP
  443, several to a message. The relay sees only single-use keys, sizes and
  timing, and logs nothing.
- **Core, command line:** privacy mode, `--relay-only`: no IP address is
  announced and everything goes through the relay.
- **Server:** the relay on Cloudflare Workers (`relay.totaldrop.app`), one
  Durable Object per transfer that admits only its two keys; and
  `totaldrop-relay`, the same relay for self-hosting, with a STUN responder.

## [linux-0.1.0-beta.3] - 2026-10-28

### Fixed

- **Site:** The Turkish download page showed last release's version numbers;
  CI now checks that versions in a translation match English.

- **Linux:** sending no longer fails when this computer's firewall blocks
  incoming connections (as Omarchy's ufw does by default). Built on the core
  fix released in android-0.1.0-beta.4.

## [android-0.1.0-beta.4] - 2026-10-28

### Fixed

- **Core:** a transfer no longer fails when a firewall blocks incoming
  connections on the sending device (as Omarchy's ufw does by default). Both
  sides now listen and dial; the first connection that comes up is used.

## [android-0.1.0-beta.3] - 2026-10-27

### Changed

- **Android:** while sending, the mark plays the website's animation: the
  file travels the channel and a ring spreads from the drop.

## [linux-0.1.0-beta.2] - 2026-10-27

### Added

- **Linux:** first desktop app, Qt 6 and QML, native Wayland. Send with the
  desktop's file chooser or by drag and drop, a code and a QR code; receive
  by typing the code with word completion, pasting it, reading a QR code from
  an image or opening a TotalDrop link. Desktop notifications when a
  transfer ends in the background.
- **Linux:** follows the desktop's look: the Omarchy theme (live), the
  portal's colour scheme and accent, the Plasma palette with Breeze controls
  and icons, and Hyprland's corner rounding.
- **Linux:** separate packages for Arch, Debian/Ubuntu (.deb) and
  Fedora (.rpm).
- **Site:** `totaldrop.app/r#code`, the page TotalDrop QR codes and links
  open: it shows the code and opens the app with it. The code stays in the
  browser. The download page offers the Linux packages.

## [0.1.0-beta.2] - 2026-10-27

### Changed

- New mark, "Dalga": a drop at the centre, waves spreading from it, and the
  file leaving through a channel in the waves. It replaces the orbit mark in
  the app, the launcher icon and the site. While sending, the file travels
  the channel; with reduced motion the mark stays still.

## [0.1.0-beta.1] - 2026-10-27

First beta. Protocol v0 with the clarifications in
[docs/protocol-v0.1-notes.md](docs/protocol-v0.1-notes.md).

### Added

- Transfer by code between two devices that can reach each other directly
  (same machine or same network), end to end encrypted:
  CPace over ristretto255, Noise XXpsk3 with an ML-KEM-768 hybrid step,
  QUIC pinned to single-use raw public keys, STREAM-encrypted chunks
  verified with BLAKE3.
- Command-line client `totaldrop send` / `totaldrop receive`.
- Rendezvous server in Rust (self-hosting) and on Cloudflare Workers,
  serving `rendezvous.totaldrop.app`.
- Android app (Flutter, Material 3, follows the system colours), in English
  and Turkish.

### Security

- Issues found by the first internal security review were fixed before
  this release; see the commit "Fix issues from the security review".
