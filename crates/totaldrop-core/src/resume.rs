// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! What a receiver keeps about an interrupted transfer (decision 0020).
//!
//! When a connection is lost for good, the receiver keeps the partial files
//! it was writing and writes a [`ResumeRecord`] into its own state folder.
//! The record is named after the offer's entries and the destination, so
//! the same files sent again to the same folder find it, whatever their new
//! transfer ID. Records are read once: taking one removes it. A record
//! older than [`KEEP_FOR`] is discarded, with its partial files, the next
//! time a receive looks at the state folder.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use totaldrop_proto::wire::{MAX_RESUME_RECORD, Manifest, ResumeRecord};

/// How long an interrupted transfer can be continued.
pub const KEEP_FOR: Duration = Duration::from_secs(24 * 60 * 60);

/// BLAKE3 key derivation context for a record's name.
const NAME_CONTEXT: &str = "TotalDrop 2026-11-11 resume record name v0";

/// File name ending of a record.
const RECORD_SUFFIX: &str = ".resume";

/// Records looked at in one sweep, so a state folder full of files cannot
/// hold a receive up.
const SWEEP_LIMIT: usize = 1024;

/// Seconds since the Unix epoch.
pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Whether `name` is a partial file name this crate makes:
/// `.totaldrop-` followed by 16 lowercase hex digits and `.part`. Anything
/// else named in a record is never touched.
pub(crate) fn is_part_name(name: &str) -> bool {
    name.strip_prefix(".totaldrop-")
        .and_then(|n| n.strip_suffix(".part"))
        .is_some_and(|hex| {
            hex.len() == 16
                && hex
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
}

/// Whether `path` is a partial file beneath `dest`: an absolute path whose
/// file name [`is_part_name`] and whose folders are plain names below
/// `dest`.
pub(crate) fn is_part_beneath(dest: &Path, path: &Path) -> bool {
    let Ok(rest) = path.strip_prefix(dest) else {
        return false;
    };
    let plain = rest
        .components()
        .all(|c| matches!(c, std::path::Component::Normal(_)));
    plain
        && path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(is_part_name)
}

/// The resume records of one state folder.
#[derive(Debug, Clone)]
pub struct Store {
    dir: PathBuf,
}

impl Store {
    /// Records live in `dir`, which is created when the first one is
    /// written.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// Where the record for `manifest`'s entries received into `dest`
    /// lives. `None` when `dest` is not valid UTF-8: such a destination is
    /// never resumed.
    pub(crate) fn path_for(&self, manifest: &Manifest, dest: &Path) -> Option<PathBuf> {
        let dest = dest.to_str()?;
        let mut h = blake3::Hasher::new_derive_key(NAME_CONTEXT);
        let entries = manifest.entries_bytes();
        h.update(&(entries.len() as u64).to_le_bytes());
        h.update(&entries);
        h.update(dest.as_bytes());
        let hex: String = h.finalize().as_bytes()[..16]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        Some(self.dir.join(format!("{hex}{RECORD_SUFFIX}")))
    }

    /// Take the record at `path`: read it and remove it. `None` when there
    /// is none, it cannot be read or removed (another receive took it), it
    /// does not decode, or it is too old (then its files are discarded).
    pub(crate) fn take(&self, path: &Path) -> Option<ResumeRecord> {
        let record = read(path);
        // Removing it is what makes it ours.
        std::fs::remove_file(path).ok()?;
        let record = record?;
        if expired(&record) {
            discard(&record);
            return None;
        }
        Some(record)
    }

    /// Write `record` to `path`, replacing it whole or not at all.
    pub(crate) fn save(&self, path: &Path, record: &ResumeRecord) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let mut tmp_name = [0u8; 8];
        getrandom::fill(&mut tmp_name).map_err(|_| std::io::Error::other("random"))?;
        let tmp = self.dir.join(format!(
            ".{}.tmp",
            tmp_name
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        ));
        let written = (|| {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&tmp)?;
            f.write_all(&record.encode())?;
            f.sync_all()?;
            std::fs::rename(&tmp, path)
        })();
        if written.is_err() {
            let _ = std::fs::remove_file(&tmp);
        }
        written
    }

    /// Discard every record older than [`KEEP_FOR`], and every one that
    /// does not decode, with the partial files they name.
    pub(crate) fn sweep(&self) {
        let Ok(list) = std::fs::read_dir(&self.dir) else {
            return;
        };
        for entry in list.flatten().take(SWEEP_LIMIT) {
            let path = entry.path();
            let is_record = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(RECORD_SUFFIX));
            if !is_record {
                continue;
            }
            match read(&path) {
                Some(r) if !expired(&r) => {}
                Some(r) => {
                    if std::fs::remove_file(&path).is_ok() {
                        discard(&r);
                    }
                }
                None => {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }
}

/// Read and decode a record, bounded in size.
fn read(path: &Path) -> Option<ResumeRecord> {
    use std::io::Read;
    let f = std::fs::File::open(path).ok()?;
    let mut buf = Vec::new();
    f.take((MAX_RESUME_RECORD as u64).saturating_add(1))
        .read_to_end(&mut buf)
        .ok()?;
    ResumeRecord::decode(&buf).ok()
}

fn expired(record: &ResumeRecord) -> bool {
    record.interrupted.saturating_add(KEEP_FOR.as_secs()) <= now()
        // A time from the future is as untrustworthy as an old one.
        || record.interrupted > now().saturating_add(KEEP_FOR.as_secs())
}

/// Remove the partial files a record names and then the empty folders it
/// created, deepest first. Only partial file names beneath the record's
/// destination are touched; folders are removed only when empty.
pub(crate) fn discard(record: &ResumeRecord) {
    let dest = Path::new(&record.dest);
    for f in &record.files {
        if let Some(p) = &f.part {
            let p = Path::new(p);
            if is_part_beneath(dest, p) {
                let _ = std::fs::remove_file(p);
            }
        }
    }
    for d in record.dirs.iter().rev() {
        let d = Path::new(d);
        if d.strip_prefix(dest).is_ok_and(|r| {
            r.components()
                .all(|c| matches!(c, std::path::Component::Normal(_)))
        }) && d != dest
        {
            let _ = std::fs::remove_dir(d);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use totaldrop_proto::wire::{Entry, EntryKind, ResumeFile};

    fn manifest(size: u64) -> Manifest {
        Manifest {
            transfer_id: [9; 16],
            entries: vec![Entry {
                path: vec!["a.bin".into()],
                kind: EntryKind::File,
                size,
                hash: Some([1; 32]),
                mtime: None,
            }],
            total_size: size,
            chunk_size: 1 << 20,
        }
    }

    fn scratch(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("td-resume-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn part_names() {
        assert!(is_part_name(".totaldrop-0123456789abcdef.part"));
        assert!(!is_part_name(".totaldrop-0123456789ABCDEF.part"));
        assert!(!is_part_name(".totaldrop-0123456789abcde.part"));
        assert!(!is_part_name("photo.jpg"));
        assert!(!is_part_name(".totaldrop-../../etc.part"));
        let dest = Path::new("/d");
        assert!(is_part_beneath(
            dest,
            Path::new("/d/x/.totaldrop-0123456789abcdef.part")
        ));
        assert!(!is_part_beneath(
            dest,
            Path::new("/e/.totaldrop-0123456789abcdef.part")
        ));
        assert!(!is_part_beneath(
            dest,
            Path::new("/d/../e/.totaldrop-0123456789abcdef.part")
        ));
    }

    /// The name depends on the entries and the destination, not on the
    /// transfer ID, which is new every time.
    #[test]
    fn a_record_is_named_after_entries_and_destination() {
        let s = Store::new("/state");
        let mut m = manifest(5);
        let a = s.path_for(&m, Path::new("/d")).unwrap();
        m.transfer_id = [7; 16];
        assert_eq!(s.path_for(&m, Path::new("/d")).unwrap(), a);
        assert_ne!(s.path_for(&m, Path::new("/e")).unwrap(), a);
        assert_ne!(s.path_for(&manifest(6), Path::new("/d")).unwrap(), a);
    }

    #[test]
    fn a_record_is_taken_once() {
        let dir = scratch("once");
        let s = Store::new(dir.join("state"));
        let path = s.path_for(&manifest(5), &dir).unwrap();
        let r = ResumeRecord {
            interrupted: now(),
            dest: dir.to_str().unwrap().into(),
            tops: vec![],
            dirs: vec![],
            files: vec![ResumeFile {
                index: 0,
                part: None,
                chunks: 0,
            }],
        };
        s.save(&path, &r).unwrap();
        assert_eq!(s.take(&path), Some(r));
        assert_eq!(s.take(&path), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// An old record goes, with its partial file; a file that is not a
    /// partial file of ours stays even if a record names it.
    #[test]
    fn an_old_record_is_discarded_with_its_files() {
        let dir = scratch("old");
        let s = Store::new(dir.join("state"));
        let part = dir.join(".totaldrop-0123456789abcdef.part");
        let other = dir.join("keep.txt");
        std::fs::write(&part, b"x").unwrap();
        std::fs::write(&other, b"x").unwrap();
        let path = s.path_for(&manifest(5), &dir).unwrap();
        let file = |p: &Path| ResumeFile {
            index: 0,
            part: Some(p.to_str().unwrap().into()),
            chunks: 0,
        };
        let r = ResumeRecord {
            interrupted: now().saturating_sub(KEEP_FOR.as_secs()),
            dest: dir.to_str().unwrap().into(),
            tops: vec![],
            dirs: vec![],
            files: vec![file(&part), file(&other)],
        };
        s.save(&path, &r).unwrap();
        s.sweep();
        assert!(!path.exists());
        assert!(!part.exists());
        assert!(other.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
