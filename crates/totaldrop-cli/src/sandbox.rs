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
    // Every mount point that holds `dir` lies on its path, so a later one
    // covers the earlier ones: the last line that holds `dir` is the mount
    // the file lands on.
    let mut memory = None;
    for line in mountinfo.lines() {
        // id parent major:minor root mount-point options [optional…] - type source super
        let mut halves = line.splitn(2, " - ");
        let (Some(left), Some(right)) = (halves.next(), halves.next()) else {
            continue;
        };
        let mut fields = left.split(' ');
        let (Some(point), Some(kind)) = (fields.nth(4), right.split(' ').next()) else {
            continue;
        };
        if !dir.starts_with(path_of(&unescape(point))) {
            continue;
        }
        // After the options come the optional fields, such as master:1.
        let from_outside = fields
            .skip(1)
            .any(|f| f.starts_with("master:") || f.starts_with("shared:"));
        memory = Some(matches!(kind, "tmpfs" | "ramfs") && !from_outside);
    }
    memory != Some(true)
}

/// A mount point's bytes as a path. Mount tables exist only where paths
/// are bytes (Linux); elsewhere `kept` is never asked, as there is no
/// Flatpak, and the bytes are read as UTF-8.
#[cfg(unix)]
fn path_of(bytes: &[u8]) -> PathBuf {
    use std::os::unix::ffi::OsStrExt as _;
    PathBuf::from(std::ffi::OsStr::from_bytes(bytes))
}

#[cfg(not(unix))]
fn path_of(bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}

/// Undoes the octal escapes (`\040` for a space) of a mountinfo field.
fn unescape(field: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(field.len());
    let mut rest = field.as_bytes();
    while let Some((&b, tail)) = rest.split_first() {
        if b == b'\\'
            && let Some((digits, after)) = tail.split_first_chunk::<3>()
            && let Some(v) = octal(*digits)
        {
            out.push(v);
            rest = after;
        } else {
            out.push(b);
            rest = tail;
        }
    }
    out
}

/// Three octal digits as one byte, if they are octal and fit in it.
fn octal(digits: [u8; 3]) -> Option<u8> {
    if !digits.iter().all(|d| (b'0'..=b'7').contains(d)) {
        return None;
    }
    std::str::from_utf8(&digits)
        .ok()
        .and_then(|d| u8::from_str_radix(d, 8).ok())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// The mount table inside the beta.6 Flatpak on Omarchy, shortened
    /// (`flatpak run --command=cat app.totaldrop.TotalDrop
    /// /proc/self/mountinfo`, 7 Nov 2026), with the user renamed. The root
    /// is the sandbox's tmpfs; the home folder has no mount of its own.
    const FLATPAK: &str = "\
708 707 0:84 / / rw,nosuid,nodev,relatime - tmpfs tmpfs rw,uid=1000,gid=1000,inode64
709 708 0:29 /@home/ada/.local/share/flatpak/runtime/org.kde.Platform/x86_64/6.11/active/files /usr ro,nosuid,nodev,relatime master:162 - btrfs /dev/mapper/root rw,compress=zstd:3,subvol=/@home
767 708 0:29 /@home/ada/.var/app/app.totaldrop.TotalDrop/cache/tmp /var/tmp rw,nosuid,nodev,relatime master:162 - btrfs /dev/mapper/root rw,compress=zstd:3,subvol=/@home
779 708 0:29 /@home/ada/.cache/.flatpak/app.totaldrop.TotalDrop/xdg-run /run/user/1000 rw,nosuid,nodev,relatime master:162 - btrfs /dev/mapper/root rw,compress=zstd:3,subvol=/@home
790 708 0:29 /@home/ada/.cache/.flatpak/app.totaldrop.TotalDrop/tmp /tmp rw,nosuid,nodev,relatime master:162 - btrfs /dev/mapper/root rw,compress=zstd:3,subvol=/@home
797 708 0:29 /@home/ada/.var/app/app.totaldrop.TotalDrop /home/ada/.var/app/app.totaldrop.TotalDrop rw,nosuid,nodev,relatime master:162 - btrfs /dev/mapper/root rw,compress=zstd:3,subvol=/@home
798 708 0:29 /@home/ada/Downloads/TotalDrop /home/ada/Downloads/TotalDrop rw,nosuid,nodev,relatime master:162 - btrfs /dev/mapper/root rw,compress=zstd:3,subvol=/@home
804 797 0:85 /bindfileta5rPB//deleted /home/ada/.var/app/app.totaldrop.TotalDrop/config/user-dirs.dirs ro,nosuid,nodev,relatime - tmpfs tmpfs rw,uid=1000,gid=1000,inode64,huge=advise
811 708 0:86 / /dev/shm rw,nosuid,nodev - tmpfs tmpfs rw
";

    #[test]
    fn the_downloads_folder_is_always_absolute() {
        // #441: with no download folder and no home, or a relative `$HOME`
        // (`dirs` takes `$HOME` as it is), the folder was relative, and a
        // `file://` URL made from it named another place.
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(downloads_in(None, None), cwd.join("TotalDrop"));
        assert_eq!(
            downloads_in(None, Some(PathBuf::from("ada"))),
            cwd.join("ada/Downloads/TotalDrop")
        );
        assert_eq!(
            downloads_in(Some(PathBuf::from("dl")), None),
            cwd.join("dl/TotalDrop")
        );
        // An absolute folder is kept as it is.
        assert_eq!(
            downloads_in(
                Some(PathBuf::from("/home/ada/İndirilenler")),
                Some(PathBuf::from("/home/ada"))
            ),
            PathBuf::from("/home/ada/İndirilenler/TotalDrop")
        );
        assert_eq!(
            downloads_in(None, Some(PathBuf::from("/home/ada"))),
            PathBuf::from("/home/ada/Downloads/TotalDrop")
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
        // Flatpak keeps the app's /tmp and /run/user on disk.
        assert!(kept(FLATPAK, Path::new("/tmp/x")));
        assert!(kept(FLATPAK, Path::new("/run/user/1000/x")));
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
