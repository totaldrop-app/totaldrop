//! The transfer protocol (spec section 8): offer, accept, encrypted chunks
//! on data streams, verification and atomic placement.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Semaphore, mpsc};
use totaldrop_crypto::identity::{DeviceKeys, verify_signature};
use totaldrop_crypto::{CHUNK_SIZE, ContentKey, EntryCipher, TAG_SIZE};
use totaldrop_proto::wire::{
    Certificate, Control, DataHeader, Entry, EntryKind, MAX_FRAME, Manifest, RejectReason,
};
use totaldrop_proto::{SafePath, chunk_count, limits, validate_path};

use crate::error::{Error, Result};
use crate::framing::{expect_frame, read_frame, write_frame};
use crate::secure::{SecureChannel, recv_control, send_control};

/// How long a receiver waits with no data arriving at all.
const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(60);

/// Data streams in flight at once (spec section 8).
const PARALLEL_STREAMS: usize = 4;

/// Called with `(bytes done, bytes total)` as a transfer moves.
pub type Progress = Arc<dyn Fn(u64, u64) + Send + Sync>;

// ---------------------------------------------------------------------------
// Sender side

/// A file or directory ready to be offered.
#[derive(Debug, Clone)]
pub struct SourceEntry {
    /// The manifest entry.
    pub entry: Entry,
    /// Where the bytes come from; `None` for directories.
    pub source: Option<PathBuf>,
}

fn utf8_name(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(str::to_owned)
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{} has no usable UTF-8 name", path.display()),
            ))
        })
}

fn mtime(meta: &std::fs::Metadata) -> Option<u64> {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

fn hash_file(path: &Path) -> Result<[u8; 32]> {
    let mut hasher = blake3::Hasher::new();
    let file = std::fs::File::open(path)?;
    hasher.update_reader(file)?;
    Ok(*hasher.finalize().as_bytes())
}

fn walk(
    path: &Path,
    prefix: &[String],
    out: &mut Vec<SourceEntry>,
    skipped: &mut usize,
) -> Result<()> {
    let meta = std::fs::symlink_metadata(path)?;
    let mut segs = prefix.to_vec();
    segs.push(utf8_name(path)?);
    if segs.len() > limits::MAX_PATH_DEPTH {
        return Err(Error::InvalidOffer(format!(
            "{} is nested too deeply",
            path.display()
        )));
    }
    if out.len() >= limits::MAX_ENTRIES {
        return Err(Error::InvalidOffer("too many files in one transfer".into()));
    }
    let ft = meta.file_type();
    if ft.is_file() {
        out.push(SourceEntry {
            entry: Entry {
                path: segs,
                kind: EntryKind::File,
                size: meta.len(),
                hash: Some(hash_file(path)?),
                mtime: mtime(&meta),
            },
            source: Some(path.to_path_buf()),
        });
    } else if ft.is_dir() {
        out.push(SourceEntry {
            entry: Entry {
                path: segs.clone(),
                kind: EntryKind::Dir,
                size: 0,
                hash: None,
                mtime: mtime(&meta),
            },
            source: None,
        });
        let mut children: Vec<PathBuf> = std::fs::read_dir(path)?
            .map(|e| e.map(|e| e.path()))
            .collect::<std::io::Result<_>>()?;
        children.sort();
        for child in children {
            walk(&child, &segs, out, skipped)?;
        }
    } else {
        // Symbolic links, sockets, devices: never sent, never followed.
        *skipped = skipped.saturating_add(1);
    }
    Ok(())
}

/// Scan and hash what will be sent. Symbolic links and special files are
/// skipped; the count is returned so the person can be told.
pub fn scan(paths: &[PathBuf]) -> Result<(Vec<SourceEntry>, usize)> {
    let mut out = Vec::new();
    let mut skipped = 0usize;
    for p in paths {
        let meta = std::fs::symlink_metadata(p)?;
        if meta.file_type().is_symlink() {
            return Err(Error::InvalidOffer(format!(
                "{} is a symbolic link; send what it points to instead",
                p.display()
            )));
        }
        walk(p, &[], &mut out, &mut skipped)?;
    }
    // Check our own manifest by the same rules the receiver will apply.
    check_entries(out.iter().map(|s| &s.entry))?;
    if out.is_empty() {
        return Err(Error::InvalidOffer("nothing to send".into()));
    }
    Ok((out, skipped))
}

/// Path safety, case-insensitive conflicts, and file-versus-directory
/// conflicts (spec section 11). Returns the normalized paths.
fn check_entries<'a>(entries: impl Iterator<Item = &'a Entry>) -> Result<Vec<SafePath>> {
    let mut safe = Vec::new();
    let mut keys = HashSet::new();
    let mut file_keys = HashSet::new();
    for e in entries {
        let p = validate_path(&e.path)
            .map_err(|err| Error::InvalidOffer(format!("unsafe path {:?}: {err:?}", e.path)))?;
        let key = p.collision_key();
        if !keys.insert(key.clone()) {
            return Err(Error::InvalidOffer(format!(
                "two entries differ only in case: {:?}",
                e.path
            )));
        }
        if e.kind == EntryKind::File {
            file_keys.insert(key);
        }
        safe.push(p);
    }
    // No entry may sit beneath a file.
    for p in &safe {
        let segs = p.segments();
        for n in 1..segs.len() {
            let parent = segs
                .get(..n)
                .unwrap_or_default()
                .iter()
                .map(|s| s.to_lowercase())
                .collect::<Vec<_>>()
                .join("/");
            if file_keys.contains(&parent) {
                return Err(Error::InvalidOffer("an entry is inside a file".into()));
            }
        }
    }
    Ok(safe)
}

#[allow(clippy::too_many_arguments)]
async fn send_entry(
    conn: quinn::Connection,
    source: PathBuf,
    entry: Entry,
    index: u64,
    ck: ContentKey,
    transfer_id: [u8; 16],
    done: Arc<AtomicU64>,
    total: u64,
    progress: Progress,
) -> Result<()> {
    let chunks = chunk_count(entry.size).ok_or(Error::Protocol("file too large"))?;
    let mut file = tokio::fs::File::open(&source).await?;
    if file.metadata().await?.len() != entry.size {
        return Err(Error::Io(std::io::Error::other(format!(
            "{} changed after it was scanned",
            source.display()
        ))));
    }
    let cipher = EntryCipher::new(&ck, &transfer_id, index);
    let mut stream = conn.open_uni().await?;
    let header = DataHeader {
        transfer_id,
        entry: index,
        first_chunk: 0,
    };
    write_frame(&mut stream, &header.encode()).await?;
    let mut remaining = entry.size;
    let mut buf = vec![0u8; CHUNK_SIZE];
    for i in 0..chunks {
        let len = usize::try_from(remaining.min(limits::CHUNK_SIZE)).unwrap_or(CHUNK_SIZE);
        let slot = buf.get_mut(..len).ok_or(Error::Protocol("chunk size"))?;
        file.read_exact(slot).await?;
        let last = i.saturating_add(1) == chunks;
        let idx = u32::try_from(i).map_err(|_| Error::Protocol("chunk index"))?;
        let ct = cipher.seal(idx, last, slot)?;
        write_frame(&mut stream, &ct).await?;
        remaining = remaining.saturating_sub(len as u64);
        let now = done
            .fetch_add(len as u64, Ordering::Relaxed)
            .saturating_add(len as u64);
        progress(now, total);
    }
    stream
        .finish()
        .map_err(|_| Error::Protocol("stream closed"))?;
    Ok(())
}

/// Result of a finished send.
#[derive(Debug, Clone, Default)]
pub struct SendReport {
    /// Files the receiver verified.
    pub files_ok: usize,
    /// Files whose hash did not match on arrival.
    pub files_failed: usize,
    /// Bytes sent.
    pub bytes: u64,
}

/// Offer `entries` and send them over an established session.
pub async fn send_files(
    conn: &quinn::Connection,
    mut chan: SecureChannel,
    keys: &DeviceKeys,
    entries: Vec<SourceEntry>,
    progress: Progress,
) -> Result<SendReport> {
    let transfer_id = totaldrop_crypto::new_transfer_id()?;
    let ck = ContentKey::generate()?;
    let total: u64 = entries.iter().map(|e| e.entry.size).sum();
    let manifest = Manifest {
        transfer_id,
        entries: entries.iter().map(|e| e.entry.clone()).collect(),
        total_size: total,
        chunk_size: limits::CHUNK_SIZE,
    };
    let signature = keys.sign(&manifest.signed_bytes());
    let offer = Control::Offer {
        manifest,
        content_key: *ck.as_bytes(),
        signature,
    };
    send_control(&mut chan.sealer, &mut chan.send, &offer).await?;

    let wanted = match recv_control(&mut chan.opener, &mut chan.recv).await? {
        Control::Accept {
            transfer_id: t,
            wanted,
        } if t == transfer_id => wanted,
        Control::Reject { reason, .. } => return Err(Error::Rejected(reason)),
        Control::Cancel { reason, .. } => return Err(Error::Cancelled(reason)),
        _ => return Err(Error::Protocol("expected accept")),
    };

    // Only files carry data; the receiver may not ask for anything else.
    let mut to_send = Vec::new();
    let mut seen = HashSet::new();
    for (index, _bitmap) in wanted {
        let i = usize::try_from(index).map_err(|_| Error::Protocol("bad entry index"))?;
        let src = entries.get(i).ok_or(Error::Protocol("bad entry index"))?;
        if src.entry.kind != EntryKind::File || !seen.insert(index) {
            return Err(Error::Protocol("bad entry index"));
        }
        to_send.push((index, src.clone()));
    }
    let wanted_bytes: u64 = to_send.iter().map(|(_, s)| s.entry.size).sum();

    let done = Arc::new(AtomicU64::new(0));
    let permits = Arc::new(Semaphore::new(PARALLEL_STREAMS));
    let mut tasks = tokio::task::JoinSet::new();
    for (index, src) in to_send.iter().cloned() {
        let Some(path) = src.source else { continue };
        let permit = permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| Error::Protocol("semaphore closed"))?;
        let conn = conn.clone();
        let ck = ck.clone();
        let done = done.clone();
        let progress = progress.clone();
        tasks.spawn(async move {
            let r = send_entry(
                conn,
                path,
                src.entry,
                index,
                ck,
                transfer_id,
                done,
                wanted_bytes,
                progress,
            )
            .await;
            drop(permit);
            r
        });
        // Collect failures early, without blocking on progress.
        while let Some(joined) = tasks.try_join_next() {
            joined.map_err(|_| Error::Protocol("send task failed"))??;
        }
    }

    let mut report = SendReport::default();
    let mut pending = to_send.len();
    let mut finished = HashSet::new();
    while pending > 0 {
        tokio::select! {
            joined = tasks.join_next(), if !tasks.is_empty() => {
                if let Some(joined) = joined {
                    joined.map_err(|_| Error::Protocol("send task failed"))??;
                }
            }
            msg = recv_control(&mut chan.opener, &mut chan.recv) => match msg? {
                Control::EntryDone { transfer_id: t, index, ok } if t == transfer_id => {
                    if !to_send.iter().any(|(i, _)| *i == index) || !finished.insert(index) {
                        return Err(Error::Protocol("unexpected entry_done"));
                    }
                    if ok { report.files_ok = report.files_ok.saturating_add(1); }
                    else { report.files_failed = report.files_failed.saturating_add(1); }
                    pending = pending.saturating_sub(1);
                }
                Control::Cancel { reason, .. } => return Err(Error::Cancelled(reason)),
                _ => return Err(Error::Protocol("unexpected control message")),
            },
        }
    }
    while let Some(joined) = tasks.join_next().await {
        joined.map_err(|_| Error::Protocol("send task failed"))??;
    }
    report.bytes = done.load(Ordering::Relaxed);
    let _ = chan.send.finish();
    conn.close(0u32.into(), b"done");
    Ok(report)
}

// ---------------------------------------------------------------------------
// Receiver side

/// What the receiver is asked to accept.
#[derive(Debug, Clone)]
pub struct OfferSummary {
    /// Number of files.
    pub files: usize,
    /// Number of directories.
    pub dirs: usize,
    /// Total size in bytes.
    pub total_size: u64,
    /// Top-level names, normalized.
    pub top_level: Vec<String>,
}

/// A verified offer waiting for a decision.
pub struct VerifiedOffer {
    manifest: Manifest,
    content_key: ContentKey,
    paths: Vec<SafePath>,
    /// What to show the person.
    pub summary: OfferSummary,
}

impl core::fmt::Debug for VerifiedOffer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VerifiedOffer")
            .field("summary", &self.summary)
            .finish_non_exhaustive()
    }
}

/// Tell the sender its offer broke a rule, and pass the error on.
async fn reject_invalid(chan: &mut SecureChannel, transfer_id: [u8; 16], err: Error) -> Error {
    let _ = send_control(
        &mut chan.sealer,
        &mut chan.send,
        &Control::Reject {
            transfer_id,
            reason: RejectReason::Invalid,
        },
    )
    .await;
    let _ = chan.send.finish();
    err
}

/// Receive and check the sender's offer: signature by the device that
/// confirmed the code, path safety and limits.
pub async fn receive_offer(chan: &mut SecureChannel, peer: &Certificate) -> Result<VerifiedOffer> {
    let msg = recv_control(&mut chan.opener, &mut chan.recv).await?;
    let Control::Offer {
        manifest,
        content_key,
        signature,
    } = msg
    else {
        return Err(Error::Protocol("expected offer"));
    };
    let tid = manifest.transfer_id;
    if verify_signature(&peer.signing_key, &manifest.signed_bytes(), &signature).is_err() {
        return Err(reject_invalid(chan, tid, Error::InvalidOffer("bad signature".into())).await);
    }
    let paths = match check_entries(manifest.entries.iter()) {
        Ok(p) => p,
        Err(e) => return Err(reject_invalid(chan, tid, e).await),
    };
    let mut top_level: Vec<String> = Vec::new();
    let mut seen_top = HashSet::new();
    for p in &paths {
        if let Some(first) = p.segments().first()
            && seen_top.insert(first.clone())
        {
            top_level.push(first.clone());
        }
    }
    let files = manifest
        .entries
        .iter()
        .filter(|e| e.kind == EntryKind::File)
        .count();
    let summary = OfferSummary {
        files,
        dirs: manifest.entries.len().saturating_sub(files),
        total_size: manifest.total_size,
        top_level,
    };
    Ok(VerifiedOffer {
        manifest,
        content_key: ContentKey::from_bytes(content_key),
        paths,
        summary,
    })
}

/// Decline an offer.
pub async fn decline(chan: &mut SecureChannel, offer: &VerifiedOffer) -> Result<()> {
    send_control(
        &mut chan.sealer,
        &mut chan.send,
        &Control::Reject {
            transfer_id: offer.manifest.transfer_id,
            reason: RejectReason::Declined,
        },
    )
    .await?;
    let _ = chan.send.finish();
    Ok(())
}

/// A name under `dir` based on `name` that does not exist yet:
/// `name`, then `name (2)`, `name (3)`, … keeping the extension.
fn free_name(dir: &Path, name: &str, taken: &HashSet<String>) -> Result<String> {
    let exists = |n: &str| {
        taken.contains(&n.to_lowercase()) || std::fs::symlink_metadata(dir.join(n)).is_ok()
    };
    if !exists(name) {
        return Ok(name.to_owned());
    }
    let (stem, ext) = match name.rfind('.') {
        Some(i) if i > 0 => (name.get(..i).unwrap_or(name), name.get(i..).unwrap_or("")),
        _ => (name, ""),
    };
    for n in 2u32..10_000 {
        let candidate = format!("{stem} ({n}){ext}");
        if !exists(&candidate) {
            return Ok(candidate);
        }
    }
    Err(Error::Io(std::io::Error::other("no free name")))
}

/// Move a verified temporary file to `name` in `dir` without ever replacing
/// an existing file. A hard link fails atomically if the name exists, so a
/// file that appears at the last moment is never overwritten; then the
/// temporary name is removed. Filesystems without hard links fall back to a
/// checked rename.
fn place_no_clobber(tmp: &Path, dir: &Path, name: &str) -> Result<PathBuf> {
    let mut taken = HashSet::new();
    for _ in 0..10_000 {
        let candidate = dir.join(free_name(dir, name, &taken)?);
        match std::fs::hard_link(tmp, &candidate) {
            Ok(()) => {
                std::fs::remove_file(tmp)?;
                return Ok(candidate);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if let Some(n) = candidate.file_name().and_then(|n| n.to_str()) {
                    taken.insert(n.to_lowercase());
                }
            }
            Err(_) => {
                if std::fs::symlink_metadata(&candidate).is_ok() {
                    if let Some(n) = candidate.file_name().and_then(|n| n.to_str()) {
                        taken.insert(n.to_lowercase());
                    }
                    continue;
                }
                std::fs::rename(tmp, &candidate)?;
                return Ok(candidate);
            }
        }
    }
    Err(Error::Io(std::io::Error::other("no free name")))
}

/// Refuse any path that is not `dest` followed by plain names. This is a
/// second line of defence behind `validate_path`: on Windows, a segment
/// with a drive prefix would otherwise replace the whole path.
fn check_beneath(dest: &Path, path: &Path) -> Result<()> {
    let rest = path
        .strip_prefix(dest)
        .map_err(|_| Error::InvalidOffer("path escapes the destination".into()))?;
    if rest
        .components()
        .all(|c| matches!(c, std::path::Component::Normal(_)))
    {
        Ok(())
    } else {
        Err(Error::InvalidOffer("path escapes the destination".into()))
    }
}

/// Create a directory, refusing to go through anything that is not a plain
/// directory we can see (never follows symbolic links).
fn make_dir(path: &Path) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_dir() => Ok(()),
        Ok(_) => Err(Error::Io(std::io::Error::other(format!(
            "{} exists and is not a directory",
            path.display()
        )))),
        Err(_) => Ok(std::fs::create_dir(path)?),
    }
}

/// Result of a finished receive.
#[derive(Debug, Clone, Default)]
pub struct ReceiveReport {
    /// Files written and verified.
    pub files_ok: usize,
    /// Files discarded because the hash did not match.
    pub files_failed: usize,
    /// Bytes received.
    pub bytes: u64,
    /// Top-level items as they were named on disk (after renaming any that
    /// already existed).
    pub placed: Vec<PathBuf>,
}

struct Target {
    entry: Entry,
    final_path: PathBuf,
}

#[allow(clippy::too_many_arguments)]
async fn receive_stream(
    mut stream: quinn::RecvStream,
    targets: Arc<Vec<Option<Target>>>,
    claimed: Arc<std::sync::Mutex<HashSet<u64>>>,
    ck: ContentKey,
    transfer_id: [u8; 16],
    done: Arc<AtomicU64>,
    total: u64,
    progress: Progress,
) -> Result<(u64, bool)> {
    let header = DataHeader::decode(&expect_frame(&mut stream, 1024).await?)?;
    if header.transfer_id != transfer_id || header.first_chunk != 0 {
        return Err(Error::Protocol("bad data stream header"));
    }
    let index = header.entry;
    let target = usize::try_from(index)
        .ok()
        .and_then(|i| targets.get(i))
        .and_then(Option::as_ref)
        .ok_or(Error::Protocol("data for an entry that was not accepted"))?;
    {
        let mut c = claimed
            .lock()
            .map_err(|_| Error::Protocol("internal lock"))?;
        if !c.insert(index) {
            return Err(Error::Protocol("entry sent twice"));
        }
    }
    let size = target.entry.size;
    let expected_hash = target
        .entry
        .hash
        .ok_or(Error::Protocol("file without hash"))?;
    let chunks = chunk_count(size).ok_or(Error::Protocol("file too large"))?;
    let cipher = EntryCipher::new(&ck, &transfer_id, index);

    let parent = target
        .final_path
        .parent()
        .ok_or(Error::Protocol("no parent directory"))?
        .to_path_buf();
    let mut tmp_name = [0u8; 8];
    getrandom::fill(&mut tmp_name).map_err(|_| Error::Random)?;
    let tmp_path = parent.join(format!(
        ".totaldrop-{}.part",
        tmp_name
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ));
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_path)
        .await?;

    let result: Result<bool> = async {
        let mut hasher = blake3::Hasher::new();
        let mut remaining = size;
        for i in 0..chunks {
            let ct = expect_frame(&mut stream, MAX_FRAME).await?;
            if ct.len() > CHUNK_SIZE.saturating_add(TAG_SIZE) {
                return Err(Error::Protocol("chunk too large"));
            }
            let last = i.saturating_add(1) == chunks;
            let idx = u32::try_from(i).map_err(|_| Error::Protocol("chunk index"))?;
            let plain = cipher.open(idx, last, &ct)?;
            let want = remaining.min(limits::CHUNK_SIZE);
            if plain.len() as u64 != want {
                return Err(Error::Protocol("chunk has the wrong length"));
            }
            hasher.update(&plain);
            file.write_all(&plain).await?;
            remaining = remaining.saturating_sub(want);
            let now = done.fetch_add(want, Ordering::Relaxed).saturating_add(want);
            progress(now, total);
        }
        if read_frame(&mut stream, MAX_FRAME).await?.is_some() {
            return Err(Error::Protocol("data after the final chunk"));
        }
        file.flush().await?;
        file.sync_all().await?;
        Ok(hasher.finalize().as_bytes() == &expected_hash)
    }
    .await;

    match result {
        Ok(true) => {
            if let Some(m) = target.entry.mtime {
                let std_file = file.into_std().await;
                if let Some(t) = SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(m)) {
                    let _ = std_file.set_modified(t);
                }
                drop(std_file);
            } else {
                drop(file);
            }
            // Never overwrite: if something appeared at the final path in
            // the meantime, pick a fresh name next to it.
            let name = target
                .final_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or(Error::Protocol("bad file name"))?;
            let placed = {
                let (parent, name, tmp) = (parent.clone(), name.to_owned(), tmp_path.clone());
                tokio::task::spawn_blocking(move || place_no_clobber(&tmp, &parent, &name))
                    .await
                    .map_err(|_| Error::Protocol("placement task failed"))?
            };
            if let Err(e) = placed {
                let _ = tokio::fs::remove_file(&tmp_path).await;
                return Err(e);
            }
            Ok((index, true))
        }
        Ok(false) => {
            drop(file);
            let _ = tokio::fs::remove_file(&tmp_path).await;
            Ok((index, false))
        }
        Err(e) => {
            drop(file);
            let _ = tokio::fs::remove_file(&tmp_path).await;
            Err(e)
        }
    }
}

/// Accept an offer and write everything beneath `dest`.
pub async fn accept(
    conn: &quinn::Connection,
    mut chan: SecureChannel,
    offer: VerifiedOffer,
    dest: &Path,
    progress: Progress,
) -> Result<ReceiveReport> {
    let meta = std::fs::symlink_metadata(dest)?;
    if !meta.file_type().is_dir() {
        return Err(Error::Io(std::io::Error::other(format!(
            "{} is not a directory",
            dest.display()
        ))));
    }
    let transfer_id = offer.manifest.transfer_id;

    // Every top-level item gets a name that does not exist yet, so nothing
    // already on disk is ever merged into or overwritten.
    // Names the other top-level items keep are taken too. Built once, so a
    // transfer of many loose files stays linear.
    let mut taken: HashSet<String> = offer
        .summary
        .top_level
        .iter()
        .map(|t| t.to_lowercase())
        .collect();
    let mut renames: Vec<(String, String)> = Vec::new();
    for top in &offer.summary.top_level {
        let own = top.to_lowercase();
        taken.remove(&own);
        let free = if std::fs::symlink_metadata(dest.join(top)).is_ok() {
            free_name(dest, top, &taken)?
        } else {
            top.clone()
        };
        taken.insert(free.to_lowercase());
        if free.to_lowercase() != own {
            // The original name stays reserved for nobody else to take.
            taken.insert(own);
        }
        renames.push((top.clone(), free));
    }
    let rename_map: std::collections::HashMap<&str, &str> = renames
        .iter()
        .map(|(from, to)| (from.as_str(), to.as_str()))
        .collect();
    let rename_top = |seg: &str| {
        rename_map
            .get(seg)
            .map_or_else(|| seg.to_owned(), |t| (*t).to_owned())
    };

    let mut targets: Vec<Option<Target>> = Vec::with_capacity(offer.paths.len());
    let mut wanted = Vec::new();
    for (i, (entry, safe)) in offer.manifest.entries.iter().zip(&offer.paths).enumerate() {
        let mut path = dest.to_path_buf();
        for (n, seg) in safe.segments().iter().enumerate() {
            if n == 0 {
                path.push(rename_top(seg));
            } else {
                path.push(seg);
            }
            check_beneath(dest, &path)?;
            if n.saturating_add(1) < safe.segments().len() || entry.kind == EntryKind::Dir {
                make_dir(&path)?;
            }
        }
        if entry.kind == EntryKind::File {
            wanted.push((i as u64, Vec::new()));
            targets.push(Some(Target {
                entry: entry.clone(),
                final_path: path,
            }));
        } else {
            targets.push(None);
        }
    }

    send_control(
        &mut chan.sealer,
        &mut chan.send,
        &Control::Accept {
            transfer_id,
            wanted: wanted.clone(),
        },
    )
    .await?;

    let total = offer.manifest.total_size;
    let targets = Arc::new(targets);
    let claimed = Arc::new(std::sync::Mutex::new(HashSet::new()));
    let done = Arc::new(AtomicU64::new(0));
    let permits = Arc::new(Semaphore::new(PARALLEL_STREAMS));
    let (result_tx, mut result_rx) = mpsc::channel::<Result<(u64, bool)>>(16);

    // Control messages from the sender (a cancel) arrive on their own task,
    // because reading a frame cannot be interrupted safely.
    let (ctl_tx, mut ctl_rx) = mpsc::channel::<Result<Control>>(4);
    let mut opener = chan.opener;
    let mut ctl_recv = chan.recv;
    let ctl_task = tokio::spawn(async move {
        while let Ok(Some(ct)) = read_frame(&mut ctl_recv, MAX_FRAME).await {
            let msg = opener
                .open(&ct)
                .map_err(Error::from)
                .and_then(|p| Control::decode(&p).map_err(Error::from));
            if ctl_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut report = ReceiveReport::default();
    for (_, to) in &renames {
        report.placed.push(dest.join(to));
    }
    let mut pending = wanted.len();
    let mut last_activity = tokio::time::Instant::now();
    let done_at_check = Arc::clone(&done);
    let mut seen_bytes = 0u64;
    while pending > 0 {
        // Give up if nothing at all moves for a while: keep-alives alone
        // would otherwise hold the receiver forever.
        let now_bytes = done_at_check.load(Ordering::Relaxed);
        if now_bytes != seen_bytes {
            seen_bytes = now_bytes;
            last_activity = tokio::time::Instant::now();
        }
        let deadline = last_activity
            .checked_add(INACTIVITY_TIMEOUT)
            .unwrap_or(last_activity);
        tokio::select! {
            () = tokio::time::sleep_until(deadline) => {
                if done_at_check.load(Ordering::Relaxed) == seen_bytes {
                    return Err(Error::Connect("the sender stopped sending".into()));
                }
            }
            stream = conn.accept_uni() => {
                let stream = stream?;
                let permit = permits.clone().acquire_owned().await
                    .map_err(|_| Error::Protocol("semaphore closed"))?;
                let tx = result_tx.clone();
                let (targets, claimed, ck, done, progress) = (
                    targets.clone(), claimed.clone(), offer.content_key.clone(),
                    done.clone(), progress.clone(),
                );
                tokio::spawn(async move {
                    let r = receive_stream(stream, targets, claimed, ck, transfer_id, done, total, progress).await;
                    drop(permit);
                    let _ = tx.send(r).await;
                });
            }
            Some(r) = result_rx.recv() => {
                last_activity = tokio::time::Instant::now();
                let (index, ok) = r?;
                if ok { report.files_ok = report.files_ok.saturating_add(1); }
                else { report.files_failed = report.files_failed.saturating_add(1); }
                send_control(&mut chan.sealer, &mut chan.send,
                    &Control::EntryDone { transfer_id, index, ok }).await?;
                pending = pending.saturating_sub(1);
            }
            Some(msg) = ctl_rx.recv() => match msg? {
                Control::Cancel { reason, .. } => return Err(Error::Cancelled(reason)),
                _ => return Err(Error::Protocol("unexpected control message")),
            },
        }
    }
    report.bytes = done.load(Ordering::Relaxed);
    let _ = chan.send.finish();
    // Wait briefly for the sender to close, so our last messages arrive.
    let _ = tokio::time::timeout(Duration::from_secs(5), conn.closed()).await;
    ctl_task.abort();
    Ok(report)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn file(path: &[&str]) -> Entry {
        Entry {
            path: path.iter().map(|s| (*s).to_owned()).collect(),
            kind: EntryKind::File,
            size: 1,
            hash: Some([0; 32]),
            mtime: None,
        }
    }

    #[test]
    fn entry_checks() {
        assert!(check_entries([file(&["a.txt"]), file(&["b.txt"])].iter()).is_ok());
        assert!(check_entries([file(&["A.txt"]), file(&["a.TXT"])].iter()).is_err());
        assert!(check_entries([file(&["a"]), file(&["a", "b"])].iter()).is_err());
        assert!(check_entries([file(&[".."])].iter()).is_err());
        assert!(check_entries([file(&["x", "CON.txt"])].iter()).is_err());
    }

    #[test]
    fn free_names() {
        let dir = std::env::temp_dir().join(format!("td-free-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let none = HashSet::new();
        assert_eq!(free_name(&dir, "a.txt", &none).unwrap(), "a.txt");
        std::fs::write(dir.join("a.txt"), b"x").unwrap();
        assert_eq!(free_name(&dir, "a.txt", &none).unwrap(), "a (2).txt");
        std::fs::write(dir.join("a (2).txt"), b"x").unwrap();
        assert_eq!(free_name(&dir, "a.txt", &none).unwrap(), "a (3).txt");
        let taken: HashSet<String> = ["a (3).txt".to_owned()].into();
        assert_eq!(free_name(&dir, "a.txt", &taken).unwrap(), "a (4).txt");
        assert_eq!(free_name(&dir, ".hidden", &none).unwrap(), ".hidden");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn placement_never_replaces_a_file() {
        let dir = std::env::temp_dir().join(format!("td-place-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), b"old").unwrap();
        std::fs::write(dir.join(".tmp1"), b"new").unwrap();
        let placed = place_no_clobber(&dir.join(".tmp1"), &dir, "a.txt").unwrap();
        assert_eq!(placed, dir.join("a (2).txt"));
        assert_eq!(std::fs::read(dir.join("a.txt")).unwrap(), b"old");
        assert_eq!(std::fs::read(&placed).unwrap(), b"new");
        assert!(!dir.join(".tmp1").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn paths_must_stay_beneath_the_destination() {
        let dest = Path::new("/tmp/dest");
        assert!(check_beneath(dest, &dest.join("a").join("b")).is_ok());
        assert!(check_beneath(dest, Path::new("/etc/passwd")).is_err());
        assert!(check_beneath(dest, &dest.join("..").join("x")).is_err());
    }
}
