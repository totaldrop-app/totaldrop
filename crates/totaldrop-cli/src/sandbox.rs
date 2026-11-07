// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Where `receive` saves inside the Flatpak sandbox (#350).
//!
//! The Flatpak may write only to `~/Downloads/TotalDrop`
//! (`packaging/flatpak`). Everything else the sandbox shows as writable,
//! its home folder and `/tmp` among them, lives in memory (tmpfs) and is
//! gone when the command ends. So inside Flatpak `receive` saves to
//! `~/Downloads/TotalDrop` when no `--out` is given, the same folder as the
//! window, and refuses a folder that would not be kept rather than say
//! "Saved" for a file that is about to vanish.

use std::path::{Path, PathBuf};

/// Whether the program runs inside a Flatpak sandbox.
pub fn sandboxed() -> bool {
    Path::new("/.flatpak-info").exists()
}

/// `~/Downloads/TotalDrop`, localized by XDG: the folder the window saves
/// to too (`linux/src/desktop.rs`).
pub fn downloads() -> PathBuf {
    dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("TotalDrop")
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
/// judged from `mountinfo` (`/proc/self/mountinfo`): it is not when the
/// mount that holds `dir` is a tmpfs. Inside Flatpak every tmpfs belongs to
/// the sandbox. An unreadable table says nothing, so the folder counts as
/// kept.
pub fn kept(mountinfo: &str, dir: &Path) -> bool {
    let mut best: Option<(usize, bool)> = None;
    for line in mountinfo.lines() {
        // id parent major:minor root mount-point options [optional…] - type source super
        let mut halves = line.splitn(2, " - ");
        let (Some(left), Some(right)) = (halves.next(), halves.next()) else {
            continue;
        };
        let (Some(point), Some(kind)) = (left.split(' ').nth(4), right.split(' ').next()) else {
            continue;
        };
        let point = PathBuf::from(unescape(point));
        if !dir.starts_with(&point) {
            continue;
        }
        let depth = point.components().count();
        // A later mount on the same point covers the earlier one.
        if best.is_none_or(|(d, _)| depth >= d) {
            best = Some((depth, kind == "tmpfs"));
        }
    }
    !best.is_some_and(|(_, memory)| memory)
}

/// Undoes the octal escapes (`\040` for a space) of a mountinfo field.
fn unescape(field: &str) -> String {
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
    String::from_utf8_lossy(&out).into_owned()
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

    /// The mount table inside the beta.6 Flatpak, shortened: the sandbox's
    /// root and home are tmpfs, the download folder is the person's disk.
    const FLATPAK: &str = "\
1 0 0:40 / / rw,nosuid,nodev,relatime - tmpfs tmpfs rw,uid=1000,gid=1000
2 1 0:30 /usr /usr ro,nosuid,nodev,relatime - overlay overlay ro
3 1 0:41 / /tmp rw,nosuid,nodev,relatime - tmpfs tmpfs rw
4 1 0:42 / /home/ada rw,nosuid,nodev,relatime - tmpfs tmpfs rw
5 4 259:2 /home/ada/Downloads/TotalDrop /home/ada/Downloads/TotalDrop rw,relatime master:1 - btrfs /dev/nvme0n1p2 rw
6 4 259:2 /home/ada/.var/app/app.totaldrop.TotalDrop /home/ada/.var/app/app.totaldrop.TotalDrop rw,relatime master:1 - btrfs /dev/nvme0n1p2 rw
7 4 259:2 /home/ada/My\\040Files /home/ada/My\\040Files rw,relatime - ext4 /dev/sda1 rw
";

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
    fn the_sandbox_home_and_tmp_are_not_kept() {
        assert!(!kept(FLATPAK, Path::new("/home/ada")));
        assert!(!kept(FLATPAK, Path::new("/home/ada/Documents")));
        assert!(!kept(FLATPAK, Path::new("/tmp/x")));
        assert!(!kept(FLATPAK, Path::new("/")));
    }

    #[test]
    fn granted_folders_are_kept() {
        assert!(kept(FLATPAK, Path::new("/home/ada/Downloads/TotalDrop")));
        assert!(kept(
            FLATPAK,
            Path::new("/home/ada/Downloads/TotalDrop/a b")
        ));
        assert!(kept(
            FLATPAK,
            Path::new("/home/ada/.var/app/app.totaldrop.TotalDrop/data")
        ));
        assert!(kept(FLATPAK, Path::new("/home/ada/My Files/x")));
    }

    #[test]
    fn a_name_that_only_starts_like_a_mount_is_not_inside_it() {
        // Path prefixes go by whole components.
        assert!(!kept(FLATPAK, Path::new("/home/ada/Downloads/TotalDropX")));
    }

    #[test]
    fn a_later_mount_on_the_same_point_wins() {
        let table = "\
1 0 0:40 / / rw - ext4 /dev/sda1 rw
2 1 0:41 / /data rw - ext4 /dev/sdb1 rw
3 1 0:42 / /data rw - tmpfs tmpfs rw
";
        assert!(!kept(table, Path::new("/data/x")));
        assert!(kept(table, Path::new("/srv")));
    }

    #[test]
    fn an_unreadable_table_counts_as_kept() {
        assert!(kept("", Path::new("/home/ada")));
        assert!(kept("garbage\nmore garbage", Path::new("/home/ada")));
    }

    #[test]
    fn octal_escapes_are_undone() {
        assert_eq!(unescape("/a\\040b\\011c"), "/a b\tc");
        assert_eq!(unescape("/a\\9"), "/a\\9");
        assert_eq!(unescape("/a\\+12"), "/a\\+12");
        assert_eq!(unescape("/a\\777"), "/a\\777");
        assert_eq!(unescape("/a\\"), "/a\\");
    }
}
