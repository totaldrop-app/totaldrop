// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Flatpak `/proc/self/mountinfo` helpers shared by the CLI and Linux desktop
//! app (#350, #357).
//!
//! Inside Flatpak, bubblewrap constructs memory-only `tmpfs` parent folders
//! (such as `$HOME` and `~/Downloads`) that have no mount peer group
//! (`master:` / `shared:`). Files placed there vanish when the sandbox exits,
//! and host files are not visible there. Host-bound folders and portal mounts
//! either use another filesystem or carry a peer group from the host.

use std::path::{Path, PathBuf};

/// Whether `path` (an absolute, canonical path) sits on a mount that comes
/// from the host rather than bubblewrap's own memory-only `tmpfs`/`ramfs`,
/// judged from `mountinfo` (`/proc/self/mountinfo`).
///
/// An empty or unreadable table says nothing, so `path` counts as host-backed.
pub fn from_host(mountinfo: &str, path: &Path) -> bool {
    // Every mount point that holds `path` lies on its path, so a later one
    // covers the earlier ones: the last matching line decides the mount.
    let mut sandbox_memory = None;
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
        if !path.starts_with(path_of(&unescape_mount(point))) {
            continue;
        }
        // After the options come optional peer-group fields such as `master:1`.
        let shared = fields
            .skip(1)
            .any(|f| f.starts_with("master:") || f.starts_with("shared:"));
        sandbox_memory = Some(matches!(kind, "tmpfs" | "ramfs") && !shared);
    }
    sandbox_memory != Some(true)
}

/// Undo `\040`-style octal escapes in a `mountinfo` mount-point field.
pub fn unescape_mount(field: &str) -> Vec<u8> {
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

/// Three octal digits as one byte, if all are `'0'..='7'` and fit in `u8`.
fn octal(digits: [u8; 3]) -> Option<u8> {
    if !digits.iter().all(|d| (b'0'..=b'7').contains(d)) {
        return None;
    }
    std::str::from_utf8(&digits)
        .ok()
        .and_then(|d| u8::from_str_radix(d, 8).ok())
}

/// Convert raw mount-point bytes to a [`PathBuf`].
#[cfg(unix)]
fn path_of(bytes: &[u8]) -> PathBuf {
    use std::os::unix::ffi::OsStrExt as _;
    PathBuf::from(std::ffi::OsStr::from_bytes(bytes))
}

#[cfg(not(unix))]
fn path_of(bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_and_sandbox_tmpfs_mounts_are_distinguished() {
        let table = "\
1 0 0:40 / / rw - tmpfs tmpfs rw
2 1 0:41 / /srv/ram rw master:7 - tmpfs tmpfs rw
3 1 0:42 / /srv/ram2 rw shared:8 - ramfs ramfs rw
4 1 0:43 / /srv/ram3 rw - ramfs ramfs rw
5 1 0:44 / /home/ada/My\\040Files rw master:1 - ext4 /dev/sda1 rw
";
        assert!(!from_host(table, Path::new("/home/ada")));
        assert!(from_host(table, Path::new("/srv/ram/x")));
        assert!(from_host(table, Path::new("/srv/ram2/x")));
        assert!(!from_host(table, Path::new("/srv/ram3/x")));
        assert!(from_host(table, Path::new("/home/ada/My Files/a.pdf")));
        assert!(!from_host(table, Path::new("/home/ada/My FilesX/a.pdf")));
        assert!(from_host("", Path::new("/home/ada")));
    }

    #[test]
    fn octal_escapes_are_undone_strictly() {
        assert_eq!(unescape_mount("/a\\040b\\011c"), b"/a b\tc");
        assert_eq!(unescape_mount("/a\\9"), b"/a\\9");
        assert_eq!(unescape_mount("/a\\+12"), b"/a\\+12");
        assert_eq!(unescape_mount("/a\\777"), b"/a\\777");
        assert_eq!(unescape_mount("/a\\"), b"/a\\");
    }
}
