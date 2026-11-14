# Linux: sending from the file manager

Select one or more files and folders in the file manager and right-click,
and "Send with TotalDrop" appears (#303, the first step of #54). TotalDrop
opens, or its open window comes to the front; the selection is listed on
the Send screen. If the app is open, no second window opens; the selection
is added to the open window (`linux/src/instance.rs`).

| File manager | How | Comes with the package |
| --- | --- | --- |
| Dolphin (KDE Plasma) | A KIO service menu, `/usr/share/kio/servicemenus/app.totaldrop.TotalDrop.send.desktop` | Yes |
| Nautilus / Files (GNOME, Omarchy) | A nautilus-python extension, `/usr/share/nautilus-python/extensions/totaldrop_nautilus.py` | Yes; `python-nautilus` (Arch), `python3-nautilus` (Debian), `nautilus-python` (Fedora) must be installed |
| Nemo (Cinnamon) | `/usr/share/nemo/actions/app.totaldrop.TotalDrop.nemo_action` | Yes |
| Thunar (Xfce) | The user's own custom action (below) | No |
| All | "Open with → Other application" and drag and drop onto the window | Yes |

The menus appear only for local files; they do not appear for network
locations (SFTP, SMB) or items in the Trash, because TotalDrop reads the
file from disk.

The selection reaches the app without running a shell: Dolphin and
Nautilus use the desktop entry's path (`%U`), and Nemo gives every address
in double quotes (`Quote=double`; GLib does not encode the characters `'`,
`$`, `&`, `(`, `)` in file addresses). Files whose names start with `-` or
contain spaces, quotes or `;` are not split and are not mistaken for
options; the tests in `linux/src/instance.rs` and the generator's test of
the Nemo file check this. In Dolphin the item appears in the menu itself,
not in the "Actions" submenu (`X-KDE-Priority=TopLevel`). In Nautilus,
items in the "Recent" view can be sent too.

## Why "every file type" is not registered

The desktop entry registers only `totaldrop://` links. Registering every
file type (`application/octet-stream`, `inode/directory`) could, on
systems with little configuration (a bare Hyprland), make TotalDrop the
default opener for unknown files and folders: `xdg-open folder` would open
TotalDrop. TotalDrop still appears in the "Open with → Other application"
list.

## Flatpak

Flatpak does not let an app add itself to other programs' menus; in the
Flatpak release only "Open with" and drag and drop work.
`packaging/linux/install.sh` does not install the menu files under the
`/app` prefix.

## Thunar

Thunar's custom actions are kept in a file per user
(`~/.config/Thunar/uca.xml`), and a package cannot install them. To add it
by hand: Edit → Configure custom actions → +

- Name: Send with TotalDrop
- Command: `totaldrop-desktop %F` (Thunar quotes every path; because the
  paths start with `/`, they are not mistaken for options)
- Icon: `app.totaldrop.TotalDrop`
- On the appearance conditions tab: file pattern `*`, every type and
  "Directories" ticked.

## Texts

The menu texts and the desktop entry's name, comment and search keywords
come from the `desktop*` keys in `l10n/app` (`sendTitle` and
`receiveTitle` in the desktop entry's "Send files" and "Receive files"
actions). The summary, introduction paragraph and feature items that
software centres (Discover, GNOME Software, Flathub) show come from the
`metainfo*` keys: in the metainfo file, only the block between the
`GENERATED` comments is generated, and the release notes are written by
hand (#324). When a text changes:

```sh
node tools/desktop-entries.mjs write
```

CI (`node --test tools/test-desktop-entries.mjs`) fails if the generated
files do not match the ARB files. The Nautilus extension's logic is
written by hand; only the text table between the `GENERATED` lines is
generated.

On every PR that touches the Linux area and on every release, CI checks in
every package it builds (Arch; .deb and .rpm for x86-64 and ARM64) that the
three menu files are in place, that Nautilus's Python bindings
(`python-nautilus`, `python3-nautilus`, `nautilus-python`) are written as
only recommended, not required, and that the Flatpak has no menus
(`packaging/check-package-contents.sh`, #319). The container in which the
package is read with the distribution's own tool is the one it was built
in with `packaging/build.sh`:

```sh
packaging/build.sh deb
packaging/check-package-contents.sh dist/packages/totaldrop_*.deb
```

## Manual trial

Before every release, on a system with the package installed:

1. KDE Plasma (Dolphin), Hyprland/Omarchy (Nautilus), GNOME (Nautilus) and,
   if possible, Cinnamon (Nemo): select a file, several files and a folder,
   right-click and choose "Send with TotalDrop". Try it with TotalDrop
   closed and open, separately.
2. Try files named `-a "b".txt` and `Bob's (1) & $x.txt`, and a folder with
   Turkish characters.
3. Look at the menu text with the system language set to Turkish and to
   Arabic.
4. Right-click TotalDrop in the taskbar or the launcher and try "Send
   files" and "Receive files".
5. In Dolphin, look at a local file from other views too: the desktop
   (`desktop:/`), recently used (`recentlyused:/`) and Baloo search. If the
   menu shows, check that it sends the right file; if not, write it here as
   a known limit (it depends on whether KF6 checks `url()` or
   `mostLocalUrl()` for `X-KDE-Protocols=file`).
6. If the menu does not work in Nautilus, the reason is in the log:
   `journalctl --user -b | grep TotalDrop`.

Nautilus reads the menu only after a restart: `nautilus -q`.
