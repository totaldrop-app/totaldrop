# 0014: Steam Deck, Steam Machine and Steam Frame are supported as part of the Linux release

- **Status:** Accepted, 6 November 2026 (Ozan, #295)
- **Context:** Steam Deck users often move screenshots, game videos, mods
  and games' save files between the Deck, a PC and a phone; the same holds
  for Valve's other devices that run SteamOS (Steam Machine, Steam Frame).
  Today they mostly do it with a USB stick, cloud storage, or tools such as
  Warpinator and KDE Connect. That TotalDrop can do this end-to-end
  encrypted and without an account makes these devices a natural target.
  This document first records what works on the Deck today, what is
  missing and in which order it will be completed, then how the other two
  devices differ. It contains no code change.

  ### Platform (6 November 2026)

  The version information in this section was taken from press reports; it
  will be verified against Valve's own release notes in step 3.

  - SteamOS 3 is based on Arch Linux. The latest stable release is 3.9.5
    (October 2026): Linux kernel 7.2, KDE Plasma 6.7 in Desktop Mode.
  - The Deck has two faces:
    - **Desktop Mode:** an ordinary KDE Plasma session. Dolphin, Discover,
      xdg-desktop-portal-kde and notifications work.
    - **Game Mode:** Steam's full-screen interface, under Valve's Gamescope
      compositor. Apps open full screen, one at a time; input comes from
      the controller, and Steam Input can turn it into keyboard and mouse.
  - The root file system is read-only, and every SteamOS update replaces
    the whole image. Using `pacman` after `steamos-readonly disable` is
    possible, but the installed packages are removed at the first update;
    Valve does not support it. The supported route for user software is
    Flatpak: Discover comes connected to Flathub.
  - The home folder is `/home/deck`; the microSD card is mounted under
    `/run/media/…`.
  - The processor is x86_64 (AMD Zen 2); today's x86_64 build fits the
    architecture.

  ### Today's Linux app on the Deck

  - The app is Qt 6 and QML ([0006](0006-linux-qt.md)). On KDE it follows
    the system palette and Breeze. When installed from a system package,
    the theme rule ([tasarim.md](../tasarim.md)) is met in the Deck's
    Desktop Mode (Plasma) with no extra work; this is an expectation, not
    tried on a Deck. **Inside a Flatpak it is not met:** the theme code
    reads files directly, and those files are not visible in the sandbox.
    `kde.rs` reads `~/.config/kdeglobals`, `omarchy.rs` Omarchy's folder
    under `~/.config`, `gtk.rs` `~/.config/gtk-3.0`, `gtk-4.0` and
    `/usr/share/themes`, `hyprland.rs` `$XDG_RUNTIME_DIR/hypr` (all in
    `linux/src/theme/`). In a Flatpak, `~/.config` turns into the app's own
    `~/.var/app/…/config` folder. For the Breeze controls, the
    `org.kde.desktop` Qt style must also be found at run time. This is the
    biggest piece of work in step 1.
  - Files are chosen through xdg-desktop-portal (`linux/src/desktop.rs`).
    Notifications go through `org.freedesktop.Notifications`. The single
    window is ensured by a session bus name (`linux/src/instance.rs`).
  - Received files are written to the `~/Downloads/TotalDrop` folder; this
    cannot be changed today.
  - There is keyboard navigation: focus moves with Tab, the focused control
    is scrolled into view (`linux/qml/FocusScroller.qml`); on the send,
    receive and settings pages, Esc goes back (`Shortcut`). There is no
    navigation with the arrow keys and no controller support.
  - On Linux the QR code is only shown on the send page. The receive page
    reads a QR from an image file, not from a camera (`readQrImage`); the
    Deck has no camera anyway.
  - There is no code that stops the system from going to sleep during a
    transfer.
  - The published packages are `.deb`, `.rpm` and an Arch package. None of
    them can be installed on SteamOS in a supported way: `.deb`/`.rpm` are
    a foreign package manager, and the Arch package writes to the read-only
    root and is removed at the next update. **So today there is no permanent,
    supported way to install on the Deck.** The application ID for Flathub
    (`app.totaldrop.TotalDrop`) and the brand rule
    ([0012](0012-ad-ve-isaret.md)) are ready, but there is no Flatpak
    recipe yet.

  ### Questions that Game Mode raises

  1. **Adding to Steam:** In Desktop Mode, the Flatpak's desktop entry is
     chosen in Steam with "Add a Non-Steam Game" ("Steam Dışı Bir Oyun
     Ekle" in the Turkish interface). Steam keeps the `flatpak run` command
     as a shortcut. The user can supply the cover, logo and icon images in
     the library by hand; the app itself cannot write them into Steam.
  2. **Window, focus and theme:** Gamescope shows windows one at a time,
     full screen. The main window is no problem. Unclear: whether the
     portal file chooser opens in Game Mode (which portal back end runs in
     the Game Mode session) and, if it opens, whether it takes the focus;
     and where notifications appear. Gamescope usually gives apps only
     Xwayland; that is why `fallback-x11` is needed in the Flatpak. Because
     `XDG_CURRENT_DESKTOP` is not KDE in Game Mode, today's code picks
     Fusion as the style and the GTK/portal colours. What the "system
     theme" is in Game Mode (Steam's dark interface, or the Deck's Plasma
     settings) is decided in step 6. Until tried on a Deck, these remain
     assumptions.

     **Blocking risk:** If there is no file chooser portal in Game Mode,
     sending cannot be done at all. In that case only receiving is
     supported in Game Mode, and for sending the user is directed to
     Desktop Mode. No separate file chooser is written into the app: it
     would need a broad file permission (see below).
  3. **Controller:** Steam Input's desktop layout (right touchpad as mouse,
     triggers as clicks) makes the app usable with no extra work, but not
     comfortable. Comfortable use needs the app to be navigable with the
     d-pad and A/B. Steam Input can offer a layout that turns these into the
     keyboard's arrow, Enter and Esc keys; the work on the app side is that
     every page can be used completely with the keyboard alone (arrows,
     Enter, Esc, Tab). The focused control must also be clearly visible
     from a distance (on a television) and on a 7-inch screen.
  4. **On-screen keyboard:** In Game Mode, the Steam keyboard does not open
     by itself when a text field in a non-Steam app is tapped; the user
     opens it with Steam + X (that it does not open by itself will be
     verified in step 6). When sending from a phone to the Deck, the code is
     typed on the Deck; because the Deck has no camera, there is no way to
     receive with a QR. To ease this, when the code field gets focus, a
     short "Open the keyboard with Steam + X" hint is shown (only under
     Gamescope, in 35 languages). Sending from the Deck needs no typing of
     the code: the Deck shows the code and the QR, and the phone scans it.
  5. **Sleep:** The Deck sleeps when the screen turns off or the power
     button is pressed, and the transfer is cut. While a transfer runs,
     sleep must be postponed with the portal's `Inhibit` interface (or with
     logind). This is not specific to the Deck; it holds for every laptop
     too. Whether Steam's power-button path respects this inhibition will
     be verified; if not, a warning remains that says not to put the device
     to sleep during a transfer.

  ### Use cases and file paths

  | Use case | Direction | Location |
  | --- | --- | --- |
  | Steam screenshots | Deck → phone/PC | `~/.local/share/Steam/userdata/<account>/760/remote/<app>/screenshots` |
  | Steam game videos | Deck → PC | The folder in Steam's "Game Recording" setting (that the default is `userdata/<account>/gamerecordings` will be verified); to share, first exported as a video from Steam |
  | Mods, maps | PC → Deck | Depends on the game; mostly `~/.local/share/Steam/steamapps/common/<game>` or the Proton prefix |
  | Save files (games without Steam Cloud, emulators) | both ways | The Proton prefix `steamapps/compatdata/<app>/pfx/drive_c/users/steamuser/…`, the emulators' own folders |
  | Any file | both ways | `~/Downloads`, microSD `/run/media/…` |

  The screenshot and game video folders are deep and have numeric names;
  users find them hard to locate in the file chooser. But having the app
  scan these folders itself would need a broad file permission in the
  Flatpak, and would tie it to Steam's internal layout.

  ### File and system access in the Flatpak

  - Sending: the file chooser portal grants access to the chosen file; no
    broad permission is needed. Today's code uses this route.
  - Receiving: `--filesystem=xdg-download` opens only the Downloads folder;
    `~/Downloads/TotalDrop` is enough.
  - Theme: only read access to the desktop's theme files is needed
    (`--filesystem=xdg-config/kdeglobals:ro`, the Omarchy and GTK theme
    folders `:ro`, `--filesystem=xdg-run/hypr:ro`), or all theme reading is
    moved to the portal. Mounting a single file misses live theme changes,
    because KConfig renames the file when it saves; verified in step 1, and
    to be dealt with together with the system GTK themes in
    [#299](https://github.com/totaldrop-app/totaldrop/issues/299).
  - Asking for permanent access to the Steam folders or to `/run/media`
    (`--filesystem=home`, `--filesystem=/run/media`) would be convenient,
    but needs a justification in the Flathub review and goes against the
    principle of least privilege ([threat model](../tehdit-modeli.md)). Not
    recommended.
  - Settings: the system package's `~/.config/totaldrop/settings.toml` file
    is not visible in the Flatpak; the first launch starts with the
    defaults.
  - Command line: the `totaldrop` tool in the package runs in the Flatpak
    with `flatpak run --command=totaldrop app.totaldrop.TotalDrop`; this is written into the installation guide.
  - Single window: taking the D-Bus name that matches the application ID is
    allowed by default in a Flatpak; `instance.rs` does not change.

  ### The other SteamOS devices: Steam Machine and Steam Frame

  According to reports, Valve released two more devices that run SteamOS in
  2026. All the hardware information below comes from press reports; it
  will be verified against Valve's own documentation or the device itself
  (steps 3, 6 and 8).

  - **Steam Machine:** a small PC that connects to a television (to be
    verified: AMD Zen 4, x86_64; SteamOS 3, KDE Plasma in Desktop Mode). It
    uses the same x86_64 Flatpak as the Deck. The difference is in use:
    there is no touch screen, and the user mostly uses it in Game Mode,
    with a controller, from a few metres away. That is why step 5 (full
    keyboard use) and step 6 (Game Mode) are mandatory on the Machine, and
    a convenience on the Deck. Text and the focus frame must be readable
    from television distance. It is reported to have Ethernet and a microSD
    slot (to be verified).
  - **Steam Frame:** a standalone VR headset (to be verified: Snapdragon 8
    Gen 3, ARM, aarch64; SteamOS). According to reports, native
    `linuxarm64` apps run, and x86_64 Linux apps run translated with FEX.
    Today only our `.deb` and `.rpm` packages have an arm64 build; the
    Flatpak's aarch64 build comes in step 1, and Flathub builds aarch64
    too. Unclear: whether the Frame has Flatpak and Discover, how a 2D Qt
    window is shown inside VR, how input (VR controllers, hand tracking,
    the on-screen keyboard) reaches the app, whether the file chooser
    portal works, and what the "system theme" is in the VR shell (the
    [tasarim.md](../tasarim.md) rule is binding here too). If the Frame's
    cameras are not open to apps, there is no receiving by QR here either.

  The three devices use the same package; no separate app is written.

  ### Network

  The Deck connects only over Wi-Fi (the dock has Ethernet). Wi-Fi going
  silent for a short while, or the network changing, is handled by the
  move to the relay that came with 0.3.0-beta.4 (#185). The device going
  to sleep is a different case: nothing runs while asleep, and the
  transfer stops. The answer to that is not the protocol but postponing
  sleep in step 4.

- **Options:**
  1. **Do nothing.** The user installs with `pacman`; the app is removed at
     the first update. It amounts to recommending an unsupported route.
  2. **A separate Deck app** (e.g. a full-screen interface just for Steam).
     The maintenance of a second interface, a second set of texts in 35
     languages; little gain. Rejected.
  3. **AppImage.** Runs on the read-only root, but updates, the desktop
     entry and portal integration are left to the user; it does not show
     in Discover; the Qt plugins and theme matching become a separate
     headache.
  4. **A Flatpak on Flathub, and Deck support in the Linux app.** The
     Deck's only supported installation route; the same package is useful
     on every other distribution too. For Game Mode, not a separate app
     but keyboard/controller, sleep and Gamescope details in the existing
     interface.

- **Decision:** Option 4. Steam Deck, Steam Machine and Steam Frame are not
  separate platforms; they are targets of the Linux release and are
  supported in the order below. Each step is a separate issue and PR; the
  steps proceed alongside the 0.3 beta releases.

  | Step | Work | Rough effort |
  | --- | --- | --- |
  | 1 | **Flatpak recipe** (`packaging/flatpak/`): a KDE runtime whose Qt version suits CXX-Qt and the Rust SDK extension, offline cargo sources with `flatpak-cargo-generator`, x86_64 and aarch64. `finish-args`: `--share=network`, `--socket=wayland`, `--socket=fallback-x11`, `--device=dri`, `--filesystem=xdg-download`, `--talk-name=org.freedesktop.Notifications` (or a move to the notification portal) and the read-only theme permissions above; the file chooser and sleep portals need no permission. A build in CI and a smoke test with `flatpak-builder --run`; the Flatpak build is tried on KDE, Hyprland and Omarchy, in the light and the dark theme. The build in our CI gets the same Sigstore provenance as the other packages; the build that Flathub distributes is signed with Flathub's key and does not carry our provenance. | 4–6 days (uncertain) |
  | 2 | **Flathub submission:** the metainfo passing Flathub's checks (`flatpak-builder-lint`), screenshots; the metainfo summary and description are only in English and Turkish today, and are translated into 35 languages. Maintaining the Flathub repository. The review usually takes a few rounds. | 2 days + waiting |
  | 3 | **Trial in Desktop Mode** (separately on each device; needs the device or a tester who uses it): installation, theme, file chooser, notification, receive folder, transfer speed; verifying the platform information. Results with screenshots in the issue. | 0.5 day |
  | 4 | **Postponing sleep during a transfer** (portal `Inhibit`), for every Linux desktop. New text in 35 languages. | 1 day |
  | 5 | **Full use with the keyboard alone:** Tab and Esc exist; what is missing is navigation with the arrow keys and a focus frame clearly visible in every theme. Also tried on KDE, Hyprland and Omarchy (an accessibility gain there too). | 2 days |
  | 6 | **Game Mode trial and fixes** (on the Deck and on the Machine separately): adding to Steam, the window under Gamescope, the file chooser (blocking risk), notification, sleep, the theme target and the on-screen keyboard; if needed, a keyboard hint in the code field when Gamescope is detected (35 languages). Separate issues according to the findings. | 1–2 days + findings |
  | 7 | **Documentation and site:** a separate page and installation guide for each device (Deck, Machine, Frame; each once that device's name applies) (installing from Discover, adding to Steam, the recommended Steam Input layout, the command line), a Flathub link in the Linux download area of the site. 35 languages. | 1–2 days |
  | 8 | **Trials specific to Steam Machine and Steam Frame:** on the Machine, readability at television distance and use with the controller alone; on the Frame, installing the aarch64 Flatpak, the window inside VR, input, the file chooser and the theme target. Separate issues according to the findings; if the Frame has no Flatpak, this decision is reopened for the Frame. | Machine 1 day, Frame 2–3 days + findings |

  In total, roughly 12–16 working days for the Deck, 15–20 together with
  the Machine and the Frame; not counting Flathub's approval time and
  access to the devices. Without a device, 1, 2, 4 and 5 can be completed;
  3, 6 and 8 need the device (or a tester who uses it). Going to Steam with
  our own app page (Steamworks) is outside this decision: it needs a paid
  application and company details, and adds nothing to the donation model
  ([0005](0005-bagis.md)) or the account-free design.

  **Name: "TotalDrop for Steam Deck", "TotalDrop for Steam Machine",
  "TotalDrop for Steam Frame"** (Ozan, 6 November 2026). The release on
  each device is presented to users under its own name: the device page on
  the site, the installation guide, and the title shown in the Steam
  library. Technically it is not a separate app: the same Flatpak, the same
  application ID (`app.totaldrop.TotalDrop`), the same version number. The
  name of the Flathub entry stays "TotalDrop", because the same package is
  for every distribution.

  - "Steam Deck", "Steam Machine" and "Steam Frame" are Valve's trademarks
    and are used only to describe compatibility. No endorsement by, or
    partnership with, Valve is implied. Valve's names and logos do not
    appear in the logo, icon and cover images; the images show only
    TotalDrop's mark and name. Every device page has a line that names the
    trademark owner and the sentence "TotalDrop is not affiliated with
    Valve". Before step 7, Valve's own trademark rules are read and the
    result is added to this document; if the names conflict with them, the
    names change.
  - A device's name is used on the site and in the store text only after
    the trials on that device have passed: for the Deck steps 3 and 6, for
    the Machine steps 3 and 6 done on the Machine and the Machine part of
    step 8, for the Frame step 3 and the Frame part of step 8.
  - Translation: "TotalDrop" and the device name in the product name are
    not translated. The name may be written naturally for each language,
    such as "Steam Deck için TotalDrop"; how it is written in which
    language is settled in step 7 by the rules of
    [0008](0008-diller-ve-bolgeler.md) and [ceviri.md](../ceviri.md).
  - Added to [marka.md](../marka.md) in the PR of step 7: these names are
    only for TotalDrop's own releases; packages that others distribute may
    not use the name "TotalDrop for …" ([0012](0012-ad-ve-isaret.md)).

  **Switch 2 is out of scope.** Nintendo's developer programme needs a
  company application, a non-disclosure agreement and a closed SDK; an
  open-source file transfer app is not expected to be accepted, nor its
  source to be publishable. The system also does not give apps free access
  to the save and media folders.

- **Why:** Flatpak is the only supported permanent installation route on
  SteamOS devices, and the same package also solves installation on every
  distribution. Because the app is already written to use portals, file
  choosing and the single window do not change; the real work is reading
  the theme inside the sandbox. For Game Mode, the aim is full keyboard
  use instead of a separate interface: Steam Input turns the controller
  into a keyboard, and the same work improves accessibility for every Linux
  user.

- **Consequences:**
  - Flatpak becomes the fourth package format for Linux; the release list
    ([surec.md](../surec.md)) and the brand rule cover the Flathub entry.
  - **Rule change (approved by Ozan, 6 November 2026):** an exception is
    added to the "packages are only built in CI" rule in CLAUDE.md. Flathub
    builds the app on its own infrastructure, from the recipe in our
    repository and the tagged source; we do not make or sign that build.
    The same recipe is also built in our CI on every PR, so that if the
    recipe breaks, we see it first. The exception is written into CLAUDE.md
    and [surec.md](../surec.md) in the PR of step 1.
  - The Flatpak permissions are added to the threat model: the app has
    access to the network, the Downloads folder, the files chosen through
    the portal, and the theme files (read only).
  - "Supported" is written for a device only once the trials in the name
    rule above have passed.
  - The Deck and the Machine are not added to the three desktops of the
    design rule (KDE, Hyprland, Omarchy); Desktop Mode is KDE. Game Mode
    and the Frame's VR shell enter the release list as separate trial
    items.

- **When to look again:** When Gamescope or SteamOS brings a native portal,
  notification or on-screen keyboard behaviour for non-Steam apps; if the
  Flathub submission is rejected; if the Frame has no Flatpak; if one of the
  hardware details turns out to be wrong; or if Valve objects to the use of
  the names.

## Sources

- The SteamOS 3.9.5 release: [Phoronix](https://www.phoronix.com/news/SteamOS-3.9.5)
- The Steam screenshot folder: [Dexerto](https://www.dexerto.com/tech/where-to-find-steam-screenshot-folder-2619310/)
- Steam game recording: [Shacknews](https://shacknews.com/article/142140/how-to-use-steam-game-recording)
- Steam Machine: [Engadget](https://www.engadget.com/gaming/pc/everything-we-know-about-valves-new-steam-machine-200458817.html)
- Steam Frame and FEX: [GamingOnLinux](https://gamingonlinux.com/2026/01/valves-documentation-highlights-the-different-ways-standalone-games-run-on-steam-frame)
