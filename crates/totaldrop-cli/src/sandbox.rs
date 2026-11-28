// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Where `receive` saves inside the Flatpak sandbox (#350).
//!
//! The Flatpak may write to `~/Downloads/TotalDrop` (`packaging/flatpak`)
//! and to its own folders on disk. The rest of what the sandbox shows,
//! the home folder among it, lives on the sandbox's own tmpfs and is gone
//! when the command ends. So inside Flatpak `receive` saves to
//! `~/Downloads/TotalDrop` when no `--out` is given, the same folder as the
//! window, and refuses a folder that would not be kept rather than say
//! "Saved" for a file that is about to vanish.

use std::path::{Path, PathBuf};

/// Whether the program runs inside a Flatpak sandbox.
pub fn sandboxed() -> bool {
    Path::new("/.flatpak-info").exists()
}

/// `~/Downloads/TotalDrop`, localized by XDG: the folder the window saves
/// to too. Keep it the same as `downloads` in `linux/src/desktop.rs`.
pub fn downloads() -> PathBuf {
    downloads_in(dirs::download_dir(), dirs::home_dir())
}

/// The folder for what `dirs` found, made absolute against the current
/// folder (#441): `dirs` takes a relative `$HOME` as it is, and with no
/// download folder and no home the folder is `./TotalDrop`. Only if the
/// current folder cannot be read does it stay relative.
fn downloads_in(download: Option<PathBuf>, home: Option<PathBuf>) -> PathBuf {
    let dir = download
        .or_else(|| home.map(|h| h.join("Downloads")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("TotalDrop");
    std::path::absolute(&dir).unwrap_or(dir)
}

/// The folder to save into when no `--out` is given: the `current` folder,
/// or inside the sandbox the one folder Flatpak keeps, `downloads`.
pub fn default_destination(
    sandboxed: bool,
    current: impl FnOnce() -> std::io::Result<PathBuf>,
    downloads: impl FnOnce() -> PathBuf,
) -> std::io::Result<PathBuf> {
    if sandboxed {
        Ok(downloads())
    } else {
        current()
    }
}

/// Whether a file written into `dir` (an absolute, canonical path) is kept,
/// judged from `mountinfo` (`/proc/self/mountinfo`). It is not when the
/// mount that holds `dir` is the sandbox's own memory: a tmpfs or ramfs
/// that bubblewrap made itself. Such a mount has no peer group. A memory
/// folder of the person's own that Flatpak was allowed to see is bound in
/// from outside and carries `master:` or `shared:`, so it is kept. An
/// unreadable table says nothing, so the folder counts as kept.
pub fn kept(mountinfo: &str, dir: &Path) -> bool {
    totaldrop_core::sandbox::from_host(mountinfo, dir)
}

#[cfg(test)]
fn unescape(field: &str) -> Vec<u8> {
    totaldrop_core::sandbox::unescape_mount(field)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// The mount table inside the beta.6 Flatpak on Omarchy, shortened
    /// (`flatpak run --command=cat app.totaldrop.TotalDrop
    /// /proc/self/mountinfo`, 7 Nov 2026), with the user renamed plus a
    /// portal mount and an escaped host tmpfs share. Shared with
    /// `linux/src/instance.rs` to exercise `totaldrop_core::sandbox::from_host`.
    const FLATPAK: &str = include_str!("../tests/flatpak-mountinfo.txt");

    #[test]
    fn the_downloads_folder_is_always_absolute() {
        // #441: with no download folder and no home, or a relative `$HOME`
        // (`dirs` takes `$HOME` as it is), the folder was relative, and a
        // `file://` URL made from it named another place.
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(downloads_in(None, None), cwd.join("TotalDrop"));
        assert_eq!(
            downloads_in(None, Some(PathBuf::from("ada"))),
            cwd.join("ada").join("Downloads").join("TotalDrop")
        );
        assert_eq!(
            downloads_in(Some(PathBuf::from("dl")), None),
            cwd.join("dl").join("TotalDrop")
        );
        // An absolute folder is kept as it is. Built from the current
        // folder, so that it is absolute on Windows too, where this module
        // is compiled and tested as well.
        let home = cwd.join("home").join("ada");
        assert_eq!(
            downloads_in(Some(home.join("İndirilenler")), Some(home.clone())),
            home.join("İndirilenler").join("TotalDrop")
        );
        assert_eq!(
            downloads_in(None, Some(home.clone())),
            home.join("Downloads").join("TotalDrop")
        );
    }

    #[test]
    fn without_out_the_sandbox_saves_to_downloads() {
        // #350: the current folder was used inside Flatpak too, and the
        // file was lost with the sandbox's memory-only home.
        let cwd = || Ok(PathBuf::from("/home/ada"));
        let dl = || PathBuf::from("/home/ada/Downloads/TotalDrop");
        assert_eq!(default_destination(true, cwd, dl).unwrap(), dl());
        assert_eq!(default_destination(false, cwd, dl).unwrap(), cwd().unwrap());
        // Inside the sandbox the current folder is not even asked for.
        let gone = || Err(std::io::Error::other("no current folder"));
        assert_eq!(default_destination(true, gone, dl).unwrap(), dl());
    }

    #[test]
    fn the_sandbox_home_is_not_kept() {
        // No mount of its own: the file lands on the tmpfs root.
        assert!(!kept(FLATPAK, Path::new("/home/ada")));
        assert!(!kept(FLATPAK, Path::new("/home/ada/Documents")));
        assert!(!kept(FLATPAK, Path::new("/")));
        assert!(!kept(FLATPAK, Path::new("/dev/shm/x")));
    }

    #[test]
    fn granted_and_disk_folders_are_kept() {
        assert!(kept(FLATPAK, Path::new("/home/ada/Downloads/TotalDrop")));
        assert!(kept(
            FLATPAK,
            Path::new("/home/ada/Downloads/TotalDrop/a b")
        ));
        assert!(kept(
            FLATPAK,
            Path::new("/home/ada/.var/app/app.totaldrop.TotalDrop/data")
        ));
        // Flatpak keeps the app's /tmp, /run/user, portal mounts, and
        // host-bound tmpfs shares.
        assert!(kept(FLATPAK, Path::new("/tmp/x")));
        assert!(kept(FLATPAK, Path::new("/run/user/1000/x")));
        assert!(kept(
            FLATPAK,
            Path::new("/run/user/1000/doc/9f8e7d/photo.jpg")
        ));
        assert!(kept(FLATPAK, Path::new("/home/ada/My Share/file.txt")));
    }

    #[test]
    fn a_name_that_only_starts_like_a_mount_is_not_inside_it() {
        // Path prefixes go by whole components.
        assert!(!kept(FLATPAK, Path::new("/home/ada/Downloads/TotalDropX")));
    }

    #[test]
    fn a_memory_folder_from_outside_is_kept() {
        // flatpak override --filesystem=/srv/ram: the person's own tmpfs,
        // bound in from the host, outlives the command.
        let table = "\
1 0 0:40 / / rw - tmpfs tmpfs rw
2 1 0:41 / /srv/ram rw master:7 - tmpfs tmpfs rw
3 1 0:42 / /srv/ram2 rw shared:8 - ramfs ramfs rw
4 1 0:43 / /srv/ram3 rw - ramfs ramfs rw
";
        assert!(kept(table, Path::new("/srv/ram/x")));
        assert!(kept(table, Path::new("/srv/ram2/x")));
        assert!(!kept(table, Path::new("/srv/ram3/x")));
    }

    #[test]
    fn escaped_and_non_utf8_mount_points_match() {
        let table = "\
1 0 0:40 / / rw - tmpfs tmpfs rw
2 1 0:41 / /home/ada/My\\040Files rw master:1 - ext4 /dev/sda1 rw
3 1 0:42 / /home/ada/\\377 rw master:1 - ext4 /dev/sda1 rw
";
        assert!(kept(table, Path::new("/home/ada/My Files/x")));
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt as _;
            let odd = std::ffi::OsStr::from_bytes(b"/home/ada/\xff/x");
            assert!(kept(table, Path::new(odd)));
        }
    }

    #[test]
    fn a_later_mount_covers_earlier_ones() {
        let table = "\
1 0 0:40 / / rw - ext4 /dev/sda1 rw
2 1 0:41 / /data rw - ext4 /dev/sdb1 rw
3 1 0:42 / /data rw - tmpfs tmpfs rw
4 1 0:43 / /a/b rw - ext4 /dev/sdc1 rw
5 1 0:44 / /a rw - tmpfs tmpfs rw
";
        assert!(!kept(table, Path::new("/data/x")));
        // A mount on a parent hides one made on a child before it.
        assert!(!kept(table, Path::new("/a/b/x")));
        assert!(kept(table, Path::new("/srv")));
    }

    #[test]
    fn an_unreadable_table_counts_as_kept() {
        assert!(kept("", Path::new("/home/ada")));
        assert!(kept("garbage\nmore garbage", Path::new("/home/ada")));
    }

    #[test]
    fn octal_escapes_are_undone() {
        assert_eq!(unescape("/a\\040b\\011c"), b"/a b\tc");
        assert_eq!(unescape("/a\\9"), b"/a\\9");
        assert_eq!(unescape("/a\\+12"), b"/a\\+12");
        assert_eq!(unescape("/a\\777"), b"/a\\777");
        assert_eq!(unescape("/a\\"), b"/a\\");
    }
}
