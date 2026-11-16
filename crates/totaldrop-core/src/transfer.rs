// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
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

use crate::error::{Error, Result, STREAM_FAILED, STREAM_STOPPED};
use crate::framing::{expect_frame, read_frame, write_frame};
use crate::secure::{SecureChannel, recv_control, send_control};
use crate::stop::Stop;
use totaldrop_crypto::channel::Sealer;

/// How long a receiver waits with no data arriving at all.
const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(60);

/// How long the sender waits for the receiver to accept or decline.
const DECISION_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// How long telling the other side about a stop may take.
const STOP_NOTICE: Duration = Duration::from_secs(2);

/// How often a receiver reports its progress, and how often a sender whose
/// receiver does not report updates its estimate (#211).
const PROGRESS_EVERY: Duration = Duration::from_millis(250);

/// More than a sender's count of bytes handed to QUIC can be ahead of what
/// the receiver has written. QUIC charges flow control when the
/// application writes, so what it has taken is at most what the receiver
/// has read plus the connection window (16 MiB, see `transport`); the
/// receiver writes what it has read after at most `PARALLEL_STREAMS`
/// chunks (4 MiB). Twice the window rounds that up.
const MAX_IN_FLIGHT: u64 = 2 * crate::transport::CONNECTION_WINDOW as u64;

/// Check a receiver's progress report against the last one: it must grow,
/// and stay within the bytes the receiver asked for. A report that does not
/// grow is refused too, since an honest receiver never sends one, so a peer
/// cannot keep the sender busy with them.
fn check_report(bytes: u64, last: Option<u64>, wanted: u64) -> Result<u64> {
    if bytes > wanted || last.is_some_and(|l| bytes <= l) {
        return Err(Error::Protocol("bad progress report"));
    }
    Ok(bytes)
}

/// What a sender can honestly show when its receiver does not report
/// progress: the files the receiver confirmed, or what QUIC has taken less
/// what may still be on the way, whichever is larger.
fn lower_bound(confirmed: u64, handed_over: u64) -> u64 {
    confirmed.max(handed_over.saturating_sub(MAX_IN_FLIGHT))
}

/// Whether the other side already closed `conn`, leaving our endpoint in
/// QUIC's passive draining state rather than actively closing.
pub(crate) fn peer_closed(conn: &quinn::Connection) -> bool {
    matches!(
        conn.close_reason(),
        Some(
            quinn::ConnectionError::ApplicationClosed(_)
                | quinn::ConnectionError::ConnectionClosed(_)
        )
    )
}

/// Tell the other side this one stopped, best effort and briefly, and close
/// the connection unless the peer already closed it. Returns the error the
/// stopped side reports.
pub(crate) async fn tell_stopped(
    conn: &quinn::Connection,
    sealer: &mut Sealer,
    send: &mut quinn::SendStream,
    transfer_id: [u8; 16],
) -> Error {
    let notice = Control::Cancel {
        transfer_id,
        reason: "stopped".into(),
    };
    let _ = tokio::time::timeout(STOP_NOTICE, async {
        send_control(sealer, send, &notice).await?;
        let _ = send.finish();
        // Give the message a moment to leave before the connection closes,
        // and let the peer's close arrive so passive draining can be skipped.
        let _ = send.stopped().await;
        let _ = tokio::time::timeout(Duration::from_millis(500), conn.closed()).await;
        Ok::<_, Error>(())
    })
    .await;
    if !peer_closed(conn) {
        conn.close(0u32.into(), b"stopped");
    }
    Error::Stopped
}

/// Read control messages on their own task, because reading a frame cannot
/// be interrupted safely: a `select!` that drops a half-read frame would put
/// the stream out of step. Read errors are passed on; the channel closes
/// when the stream ends.
fn control_reader(
    mut opener: totaldrop_crypto::channel::Opener,
    mut recv: quinn::RecvStream,
) -> (AbortOnDrop, mpsc::Receiver<Result<Control>>) {
    let (tx, rx) = mpsc::channel::<Result<Control>>(8);
    let task = tokio::spawn(async move {
        loop {
            let limit = totaldrop_crypto::channel::MAX_SESSION_MESSAGE.saturating_add(64);
            let msg = match read_frame(&mut recv, limit).await {
                Ok(Some(ct)) => opener
                    .open(&ct)
                    .map_err(Error::from)
                    .and_then(|p| Control::decode(&p).map_err(Error::from)),
                Ok(None) => break,
                Err(e) => Err(e),
            };
            let failed = msg.is_err();
            if tx.send(msg).await.is_err() || failed {
                break;
            }
        }
    });
    (AbortOnDrop(task), rx)
}

/// Why the control stream ended: the connection's close reason if it has
/// one (a "stopped" close reads as the other side stopping).
fn control_ended(conn: &quinn::Connection) -> Error {
    conn.close_reason()
        .map_or(Error::Protocol("control stream ended"), Error::from)
}

/// After a data stream failed, give the other side's cancel a moment to
/// arrive on the control stream: it may explain the failure.
async fn cancel_or(err: Error, ctl: &mut mpsc::Receiver<Result<Control>>) -> Error {
    let wait = async {
        // Messages already on the way (an entry_done) may come first.
        while let Some(msg) = ctl.recv().await {
            if let Ok(Control::Cancel { reason, .. }) = msg {
                return Some(reason);
            }
        }
        None
    };
    match tokio::time::timeout(Duration::from_millis(500), wait).await {
        Ok(Some(reason)) => Error::Cancelled(reason),
        _ => err,
    }
}

/// Aborts a task when dropped, so an early return never leaves it running.
struct AbortOnDrop(tokio::task::JoinHandle<()>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// A partial file that is removed unless it was placed. Dropping it (an
/// error, a stop, an aborted task) removes the file.
struct PartFile {
    path: PathBuf,
    keep: bool,
}

impl Drop for PartFile {
    fn drop(&mut self) {
        if !self.keep {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// Directories this transfer created. Unless the transfer finishes, the
/// empty ones are removed again, deepest first; anything with content stays.
struct CreatedDirs {
    dirs: Vec<PathBuf>,
    keep: bool,
}

impl Drop for CreatedDirs {
    fn drop(&mut self) {
        if !self.keep {
            for dir in self.dirs.iter().rev() {
                let _ = std::fs::remove_dir(dir);
            }
        }
    }
}

/// Data streams in flight at once (spec section 8).
const PARALLEL_STREAMS: usize = 4;

/// Called with `(bytes done, bytes total)` as a transfer moves.
pub type Progress = Arc<dyn Fn(u64, u64) + Send + Sync>;

// ---------------------------------------------------------------------------
// Sender side

/// Where a file's bytes come from.
#[derive(Debug, Clone)]
pub enum Source {
    /// A file on disk, read when it is sent.
    Path(PathBuf),
    /// Bytes held in memory: a text (#313).
    Bytes(Arc<[u8]>),
}

/// A file or directory ready to be offered.
#[derive(Debug, Clone)]
pub struct SourceEntry {
    /// The manifest entry.
    pub entry: Entry,
    /// Where the bytes come from; `None` for directories.
    pub source: Option<Source>,
}

/// The largest text sent and shown as a text (#313): one chunk. A longer
/// one is better read as a file.
pub const MAX_TEXT: u64 = limits::CHUNK_SIZE;

/// The name a receiver that does not know texts (before #313) saves a
/// text under; every sender gives it this name.
pub const TEXT_FILE: &str = "text.txt";

/// A text to send (#313), as the one file of an offer: a receiver that
/// does not know texts saves it as `name`, which should end in `.txt`.
pub fn text_entry(text: &str, name: &str) -> Result<SourceEntry> {
    let bytes: Arc<[u8]> = Arc::from(text.as_bytes());
    let size = bytes.len() as u64;
    if size == 0 {
        return Err(Error::InvalidOffer("nothing to send".into()));
    }
    if size > MAX_TEXT {
        return Err(Error::InvalidOffer(format!(
            "the text is longer than {MAX_TEXT} bytes; send it as a file"
        )));
    }
    let entry = Entry {
        path: vec![name.to_owned()],
        kind: EntryKind::File,
        size,
        hash: Some(*blake3::hash(&bytes).as_bytes()),
        mtime: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs()),
    };
    check_entries(std::iter::once(&entry))?;
    Ok(SourceEntry {
        entry,
        source: Some(Source::Bytes(bytes)),
    })
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
            source: Some(Source::Path(path.to_path_buf())),
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
    stream: &mut quinn::SendStream,
    source: Source,
    entry: Entry,
    index: u64,
    ck: ContentKey,
    transfer_id: [u8; 16],
    done: Arc<AtomicU64>,
) -> Result<()> {
    let chunks = chunk_count(entry.size).ok_or(Error::Protocol("file too large"))?;
    let mut file: Box<dyn tokio::io::AsyncRead + Send + Unpin> = match source {
        Source::Path(path) => {
            let file = tokio::fs::File::open(&path).await?;
            if file.metadata().await?.len() != entry.size {
                return Err(Error::Io(std::io::Error::other(format!(
                    "{} {}",
                    path.display(),
                    crate::error::CHANGED_AFTER_SCAN
                ))));
            }
            Box::new(file)
        }
        Source::Bytes(bytes) => {
            if bytes.len() as u64 != entry.size {
                return Err(Error::Protocol("text size"));
            }
            Box::new(std::io::Cursor::new(bytes))
        }
    };
    let cipher = EntryCipher::new(&ck, &transfer_id, index);
    let header = DataHeader {
        transfer_id,
        entry: index,
        first_chunk: 0,
    };
    write_frame(stream, &header.encode()).await?;
    let mut remaining = entry.size;
    let mut buf = vec![0u8; CHUNK_SIZE];
    for i in 0..chunks {
        let len = usize::try_from(remaining.min(limits::CHUNK_SIZE)).unwrap_or(CHUNK_SIZE);
        let slot = buf.get_mut(..len).ok_or(Error::Protocol("chunk size"))?;
        file.read_exact(slot).await?;
        let last = i.saturating_add(1) == chunks;
        let idx = u32::try_from(i).map_err(|_| Error::Protocol("chunk index"))?;
        let ct = cipher.seal(idx, last, slot)?;
        write_frame(stream, &ct).await?;
        remaining = remaining.saturating_sub(len as u64);
        // Taken by QUIC, not arrived: progress comes from the receiver.
        done.fetch_add(len as u64, Ordering::Relaxed);
    }
    stream
        .finish()
        .map_err(|_| Error::Protocol("stream closed"))?;
    Ok(())
}

/// Send one file on its own stream, waiting for a permit first. A stop cuts
/// the file at once: the stream is reset, so the receiver drops what it
/// buffered instead of completing the file from data sent before or after
/// the stop. This also holds after the last chunk, until the receiver has
/// all of it; a file that has fully arrived stays (#193).
#[allow(clippy::too_many_arguments)]
async fn send_entry_until_stopped(
    conn: quinn::Connection,
    stop: Stop,
    permits: Arc<Semaphore>,
    source: Source,
    entry: Entry,
    index: u64,
    ck: ContentKey,
    transfer_id: [u8; 16],
    done: Arc<AtomicU64>,
) -> Result<()> {
    let permit = tokio::select! {
        biased;
        () = stop.stopped() => return Err(Error::Stopped),
        p = permits.acquire_owned() => p.map_err(|_| Error::Protocol("semaphore closed"))?,
    };
    let mut stream = tokio::select! {
        biased;
        () = stop.stopped() => return Err(Error::Stopped),
        s = conn.open_uni() => s?,
    };
    let sent = tokio::select! {
        biased;
        () = stop.stopped() => Err(Error::Stopped),
        r = send_entry(&mut stream, source, entry, index, ck, transfer_id, done) => r,
    };
    if let Err(e) = sent {
        // A dropped stream would end cleanly and read as a short file.
        let code = if matches!(e, Error::Stopped) {
            STREAM_STOPPED
        } else {
            STREAM_FAILED
        };
        let _ = stream.reset(code);
        return Err(e);
    }
    drop(permit);
    tokio::select! {
        biased;
        () = stop.stopped() => {
            let _ = stream.reset(STREAM_STOPPED);
            Err(Error::Stopped)
        }
        // All of it acknowledged, or the peer or connection ended it.
        _ = stream.stopped() => Ok(()),
    }
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
    text: bool,
    progress: Progress,
    stop: &Stop,
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
        progress: true,
        text,
    };
    send_control(&mut chan.sealer, &mut chan.send, &offer).await?;

    let SecureChannel {
        mut sealer,
        opener,
        mut send,
        recv,
        ..
    } = chan;
    let (_reader, mut ctl_rx) = control_reader(opener, recv);

    let decision = tokio::select! {
        biased;
        () = stop.stopped() => {
            return Err(tell_stopped(conn, &mut sealer, &mut send, transfer_id).await);
        }
        msg = ctl_rx.recv() => match msg {
            Some(m) => m?,
            None => return Err(control_ended(conn)),
        },
        () = tokio::time::sleep(DECISION_TIMEOUT) => {
            conn.close(0u32.into(), b"no answer");
            return Err(Error::Connect(crate::error::msg::RECEIVER_DID_NOT_ANSWER.into()));
        }
    };
    let (wanted, reports) = match decision {
        Control::Accept {
            transfer_id: t,
            wanted,
            progress,
        } if t == transfer_id => (wanted, progress),
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

    // Every file gets its task at once; the semaphore lets PARALLEL_STREAMS
    // of them send at a time. Waiting happens inside the tasks, so the loop
    // below always hears a stop or a cancel.
    let done = Arc::new(AtomicU64::new(0));
    let permits = Arc::new(Semaphore::new(PARALLEL_STREAMS));
    let mut tasks = tokio::task::JoinSet::new();
    for (index, src) in to_send.iter().cloned() {
        let Some(path) = src.source else { continue };
        tasks.spawn(send_entry_until_stopped(
            conn.clone(),
            stop.clone(),
            permits.clone(),
            path,
            src.entry,
            index,
            ck.clone(),
            transfer_id,
            done.clone(),
        ));
    }

    let mut report = SendReport::default();
    let mut pending = to_send.len();
    let mut finished = HashSet::new();
    // What the bar shows: the receiver's reports, or a lower bound for a
    // receiver that does not send them. It never goes back.
    let mut shown = 0u64;
    let mut confirmed = 0u64;
    let mut reported: Option<u64> = None;
    let mut estimate = tokio::time::interval(PROGRESS_EVERY);
    estimate.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    // The total is known now, even before anything has arrived.
    progress(0, wanted_bytes);
    while pending > 0 {
        tokio::select! {
            biased;
            () = stop.stopped() => {
                // Every task hears the stop too and resets its stream, which
                // drops what has not been sent or read yet. Let them,
                // briefly, before the cancel and the close.
                let _ = tokio::time::timeout(STOP_NOTICE, async {
                    while tasks.join_next().await.is_some() {}
                })
                .await;
                let e = tell_stopped(conn, &mut sealer, &mut send, transfer_id).await;
                tasks.abort_all();
                return Err(e);
            }
            msg = ctl_rx.recv() => match msg {
                Some(Ok(Control::EntryDone { transfer_id: t, index, ok })) if t == transfer_id => {
                    let Some((_, src)) = to_send.iter().find(|(i, _)| *i == index) else {
                        return Err(Error::Protocol("unexpected entry_done"));
                    };
                    if !finished.insert(index) {
                        return Err(Error::Protocol("unexpected entry_done"));
                    }
                    confirmed = confirmed.saturating_add(src.entry.size);
                    if ok { report.files_ok = report.files_ok.saturating_add(1); }
                    else { report.files_failed = report.files_failed.saturating_add(1); }
                    pending = pending.saturating_sub(1);
                }
                Some(Ok(Control::Received { transfer_id: t, bytes })) if reports && t == transfer_id => {
                    reported = Some(check_report(bytes, reported, wanted_bytes)?);
                    if bytes > shown {
                        shown = bytes;
                        progress(shown, wanted_bytes);
                    }
                }
                Some(Ok(Control::Cancel { reason, .. })) => return Err(Error::Cancelled(reason)),
                Some(Ok(_)) => return Err(Error::Protocol("unexpected control message")),
                Some(Err(e)) => return Err(e),
                None => return Err(control_ended(conn)),
            },
            _ = estimate.tick(), if !reports => {
                let now = lower_bound(confirmed, done.load(Ordering::Relaxed)).min(wanted_bytes);
                if now > shown {
                    shown = now;
                    progress(shown, wanted_bytes);
                }
            }
            joined = tasks.join_next(), if !tasks.is_empty() => {
                match joined {
                    Some(Ok(Ok(()))) | None => {}
                    // A stream can fail because the receiver stopped; its
                    // cancel may still be on the way.
                    Some(Ok(Err(e))) => {
                        let e = cancel_or(e, &mut ctl_rx).await;
                        // Close before the other streams are dropped: a
                        // dropped stream ends cleanly and would look like a
                        // short file to the receiver.
                        conn.close(0u32.into(), b"failed");
                        return Err(e);
                    }
                    Some(Err(_)) => {
                        conn.close(0u32.into(), b"failed");
                        return Err(Error::Protocol("send task failed"));
                    }
                }
            }
        }
    }
    // Every file is confirmed. A stop now only cuts streams the receiver
    // has finished, so it is not reported as one.
    while let Some(joined) = tasks.join_next().await {
        match joined.map_err(|_| Error::Protocol("send task failed"))? {
            Ok(()) | Err(Error::Stopped) => {}
            Err(e) => return Err(e),
        }
    }
    if shown < wanted_bytes {
        progress(wanted_bytes, wanted_bytes);
    }
    report.bytes = done.load(Ordering::Relaxed);
    let _ = send.finish();
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
    /// A text (#313): one file, small enough to show instead of saving
    /// it; its length in bytes is `total_size`. A caller that shows texts
    /// accepts it as a text; any other accept saves it as a file.
    pub text: bool,
}

/// A verified offer waiting for a decision.
pub struct VerifiedOffer {
    manifest: Manifest,
    content_key: ContentKey,
    paths: Vec<SafePath>,
    /// The sender wants `received` reports (offer field 7).
    reports: bool,
    /// What to show the person.
    pub summary: OfferSummary,
}

impl VerifiedOffer {
    /// The transfer this offer starts.
    pub(crate) fn transfer_id(&self) -> [u8; 16] {
        self.manifest.transfer_id
    }
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
        progress: reports,
        text,
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
    // Shown as a text only when it can be: one file, small enough to
    // show. Anything else marked as a text is received as files.
    // A text is one small, non-empty file at the top level, nothing else
    // (#313).
    let text = text
        && files == 1
        && (1..=MAX_TEXT).contains(&manifest.total_size)
        && matches!(manifest.entries.as_slice(), [e] if e.kind == EntryKind::File && e.path.len() == 1);
    let summary = OfferSummary {
        files,
        dirs: manifest.entries.len().saturating_sub(files),
        total_size: manifest.total_size,
        top_level,
        text,
    };
    Ok(VerifiedOffer {
        manifest,
        content_key: ContentKey::from_bytes(content_key),
        paths,
        reports,
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
/// Returns whether it created the directory.
fn make_dir(path: &Path) -> Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_dir() => Ok(false),
        Ok(_) => Err(Error::Io(std::io::Error::other(format!(
            "{} exists and is not a directory",
            path.display()
        )))),
        Err(_) => {
            std::fs::create_dir(path)?;
            Ok(true)
        }
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
    /// The text, when a text was accepted as one (#313). Nothing is then
    /// written to disk, and `placed` is empty.
    pub text: Option<String>,
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
    abandoned: Arc<std::sync::atomic::AtomicBool>,
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
    // Opened right here, not on a blocking thread: an abort between an
    // asynchronous open finishing and the guard below would leave the file.
    let std_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_path)?;
    // From here on the partial file goes away unless it is placed, also
    // when this task is aborted.
    let mut part = PartFile {
        path: tmp_path.clone(),
        keep: false,
    };
    let mut file = tokio::fs::File::from_std(std_file);

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
                // Placing cannot be aborted once it runs, so it checks first
                // whether the receive was given up meanwhile.
                tokio::task::spawn_blocking(move || {
                    if abandoned.load(Ordering::SeqCst) {
                        return Err(Error::Stopped);
                    }
                    place_no_clobber(&tmp, &parent, &name)
                })
                .await
                .map_err(|_| Error::Protocol("placement task failed"))?
            };
            // Placing moved or linked the file away; whatever is left at
            // the temporary path is removed by the guard.
            placed?;
            part.keep = !tmp_path.exists();
            Ok((index, true))
        }
        Ok(false) => {
            drop(file);
            Ok((index, false))
        }
        Err(e) => {
            drop(file);
            Err(e)
        }
    }
}

fn resolve_destination_dir(dest: &Path) -> Result<PathBuf> {
    let dir = std::fs::canonicalize(dest)?;
    if !std::fs::metadata(&dir)?.is_dir() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            format!("{} is not a directory", dest.display()),
        )));
    }
    Ok(dir)
}

fn probe_destination_writable(dir: &Path) -> Result<()> {
    let mut tag = [0u8; 8];
    getrandom::fill(&mut tag).map_err(|_| Error::Random)?;
    let probe = dir.join(format!(
        ".totaldrop-check-{}",
        tag.iter().map(|b| format!("{b:02x}")).collect::<String>()
    ));
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)?;
    let _ = std::fs::remove_file(&probe);
    Ok(())
}

/// Check the folder a receive will write into, before anything is shared:
/// it must exist, be a folder and be writable. A symbolic link given as the
/// folder itself is followed; nothing beneath it ever is. Returns the folder
/// as given, so paths shown to the person keep their spelling (no `/private`
/// on macOS, no `\\?\` on Windows). Errors are [`Error::Io`] with kind
/// `NotFound`, `NotADirectory` or the reason writing failed.
pub fn check_destination(dest: &Path) -> Result<PathBuf> {
    let dir = resolve_destination_dir(dest)?;
    probe_destination_writable(&dir)?;
    Ok(dest.to_path_buf())
}

/// Accept a text (#313) and receive it in memory: nothing is written to
/// disk. Only for an offer whose summary says it is a text; the caller
/// shows it. A text whose hash does not match, or that is not UTF-8, is
/// dropped: the report then counts a failed file and holds no text.
pub async fn accept_text(
    conn: &quinn::Connection,
    mut chan: SecureChannel,
    offer: VerifiedOffer,
    stop: &Stop,
) -> Result<ReceiveReport> {
    let transfer_id = offer.manifest.transfer_id;
    let entry = match offer.manifest.entries.first() {
        Some(e) if offer.summary.text => e.clone(),
        _ => return Err(Error::Protocol("not a text")),
    };
    if stop.is_stopped() {
        return Err(tell_stopped(conn, &mut chan.sealer, &mut chan.send, transfer_id).await);
    }
    // One chunk at most: no progress reports are needed.
    send_control(
        &mut chan.sealer,
        &mut chan.send,
        &Control::Accept {
            transfer_id,
            wanted: vec![(0, Vec::new())],
            progress: false,
        },
    )
    .await?;
    let SecureChannel {
        mut sealer,
        opener,
        mut send,
        recv,
        ..
    } = chan;
    let (_reader, mut ctl_rx) = control_reader(opener, recv);
    let ck = offer.content_key.clone();
    let read = async {
        let mut stream = conn.accept_uni().await?;
        read_text(&mut stream, &entry, &ck, transfer_id).await
    };
    let got = tokio::select! {
        biased;
        () = stop.stopped() => {
            return Err(tell_stopped(conn, &mut sealer, &mut send, transfer_id).await);
        }
        msg = ctl_rx.recv() => match msg {
            Some(Ok(Control::Cancel { reason, .. })) => return Err(Error::Cancelled(reason)),
            Some(Ok(_)) => return Err(Error::Protocol("unexpected control message")),
            Some(Err(e)) => return Err(e),
            None => return Err(control_ended(conn)),
        },
        r = tokio::time::timeout(INACTIVITY_TIMEOUT, read) => match r {
            Err(_) => {
                return Err(Error::Connect(crate::error::msg::SENDER_STOPPED_SENDING.into()));
            }
            Ok(Err(e)) => return Err(cancel_or(e, &mut ctl_rx).await),
            Ok(Ok(bytes)) => bytes,
        },
    };
    let text = got.and_then(|b| String::from_utf8(b).ok());
    send_control(
        &mut sealer,
        &mut send,
        &Control::EntryDone {
            transfer_id,
            index: 0,
            ok: text.is_some(),
        },
    )
    .await?;
    let mut report = ReceiveReport {
        bytes: entry.size,
        ..ReceiveReport::default()
    };
    if text.is_some() {
        report.files_ok = 1;
    } else {
        report.files_failed = 1;
    }
    report.text = text;
    let _ = send.finish();
    let _ = tokio::time::timeout(Duration::from_secs(5), conn.closed()).await;
    Ok(report)
}

/// The one data stream of a text: its plaintext, or `None` when the hash
/// does not match.
async fn read_text(
    stream: &mut quinn::RecvStream,
    entry: &Entry,
    ck: &ContentKey,
    transfer_id: [u8; 16],
) -> Result<Option<Vec<u8>>> {
    let header = DataHeader::decode(&expect_frame(stream, 1024).await?)?;
    if header.transfer_id != transfer_id || header.entry != 0 || header.first_chunk != 0 {
        return Err(Error::Protocol("bad data stream header"));
    }
    let size = entry.size;
    if size > MAX_TEXT {
        return Err(Error::Protocol("text too large"));
    }
    let expected = entry.hash.ok_or(Error::Protocol("file without hash"))?;
    let chunks = chunk_count(size).ok_or(Error::Protocol("file too large"))?;
    let cipher = EntryCipher::new(ck, &transfer_id, 0);
    let mut out = Vec::with_capacity(usize::try_from(size).unwrap_or(0));
    let mut remaining = size;
    for i in 0..chunks {
        let ct = expect_frame(stream, MAX_FRAME).await?;
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
        out.extend_from_slice(&plain);
        remaining = remaining.saturating_sub(want);
    }
    if read_frame(stream, MAX_FRAME).await?.is_some() {
        return Err(Error::Protocol("data after the final chunk"));
    }
    Ok((blake3::hash(&out).as_bytes() == &expected).then_some(out))
}

/// Accept an offer and write everything beneath `dest`. A text (#313)
/// offered this way is saved as the file it also is.
pub async fn accept(
    conn: &quinn::Connection,
    chan: SecureChannel,
    offer: VerifiedOffer,
    dest: &Path,
    progress: Progress,
    stop: &Stop,
) -> Result<ReceiveReport> {
    accept_with_hooks(
        conn,
        chan,
        offer,
        dest,
        progress,
        stop,
        |p| fs4::available_space(p),
        probe_destination_writable,
        |_| {},
    )
    .await
}

/// Tell the sender there is not enough free space on `dest` and wait briefly
/// for the message to be acknowledged and the sender to close before the
/// caller closes the connection (#455).
async fn reject_no_space(
    conn: &quinn::Connection,
    chan: &mut SecureChannel,
    transfer_id: [u8; 16],
    io_err: std::io::Error,
) -> Error {
    let _ = tokio::time::timeout(STOP_NOTICE, async {
        send_control(
            &mut chan.sealer,
            &mut chan.send,
            &Control::Reject {
                transfer_id,
                reason: RejectReason::NoSpace,
            },
        )
        .await?;
        let _ = chan.send.finish();
        let _ = chan.send.stopped().await;
        let _ = tokio::time::timeout(Duration::from_millis(500), conn.closed()).await;
        Ok::<_, Error>(())
    })
    .await;
    Error::Io(io_err)
}

async fn drop_created_blocking(created: CreatedDirs) {
    if !created.keep && !created.dirs.is_empty() {
        let _ = tokio::task::spawn_blocking(move || drop(created)).await;
    }
}

enum PrepareOutcome {
    Ready {
        offer: Box<VerifiedOffer>,
        renames: Vec<(String, String)>,
        targets: Vec<Option<Target>>,
        wanted: Vec<(u64, Vec<u8>)>,
        created: CreatedDirs,
    },
    NoSpace(std::io::Error),
    Stopped,
}

#[allow(clippy::too_many_arguments)]
async fn accept_with_hooks<S, P, D>(
    conn: &quinn::Connection,
    mut chan: SecureChannel,
    offer: VerifiedOffer,
    dest: &Path,
    progress: Progress,
    stop: &Stop,
    space_fn: S,
    probe_fn: P,
    mut on_dir: D,
) -> Result<ReceiveReport>
where
    S: FnOnce(&Path) -> std::io::Result<u64> + Send + 'static,
    P: FnOnce(&Path) -> Result<()> + Send + 'static,
    D: FnMut(&Path) + Send + 'static,
{
    let transfer_id = offer.manifest.transfer_id;
    if stop.is_stopped() {
        return Err(tell_stopped(conn, &mut chan.sealer, &mut chan.send, transfer_id).await);
    }

    let dest_owned = dest.to_path_buf();
    let dest_buf = dest_owned.clone();
    let stop_prep = stop.clone();
    let mut prep = tokio::task::spawn_blocking(move || -> Result<PrepareOutcome> {
        let dest = dest_buf.as_path();
        if stop_prep.is_stopped() {
            return Ok(PrepareOutcome::Stopped);
        }
        let canonical_dir = resolve_destination_dir(dest)?;
        if stop_prep.is_stopped() {
            return Ok(PrepareOutcome::Stopped);
        }
        if let Ok(free) = space_fn(&canonical_dir)
            && free < offer.manifest.total_size
        {
            return Ok(PrepareOutcome::NoSpace(std::io::Error::new(
                std::io::ErrorKind::StorageFull,
                "not enough free disk space",
            )));
        }
        if stop_prep.is_stopped() {
            return Ok(PrepareOutcome::Stopped);
        }
        match probe_fn(&canonical_dir) {
            Ok(()) => {}
            Err(Error::Io(e))
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::StorageFull | std::io::ErrorKind::QuotaExceeded
                ) =>
            {
                return Ok(PrepareOutcome::NoSpace(e));
            }
            Err(e) => return Err(e),
        }
        if stop_prep.is_stopped() {
            return Ok(PrepareOutcome::Stopped);
        }

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
            if stop_prep.is_stopped() {
                return Ok(PrepareOutcome::Stopped);
            }
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
        let mut created = CreatedDirs {
            dirs: Vec::new(),
            keep: false,
        };
        for (i, (entry, safe)) in offer.manifest.entries.iter().zip(&offer.paths).enumerate() {
            if stop_prep.is_stopped() {
                return Ok(PrepareOutcome::Stopped);
            }
            let mut path = dest.to_path_buf();
            for (n, seg) in safe.segments().iter().enumerate() {
                if n == 0 {
                    path.push(rename_top(seg));
                } else {
                    path.push(seg);
                }
                check_beneath(dest, &path)?;
                if n.saturating_add(1) < safe.segments().len() || entry.kind == EntryKind::Dir {
                    if stop_prep.is_stopped() {
                        return Ok(PrepareOutcome::Stopped);
                    }
                    if make_dir(&path)? {
                        created.dirs.push(path.clone());
                        on_dir(&path);
                    }
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
        if stop_prep.is_stopped() {
            return Ok(PrepareOutcome::Stopped);
        }
        Ok(PrepareOutcome::Ready {
            offer: Box::new(offer),
            renames,
            targets,
            wanted,
            created,
        })
    });

    let prep = tokio::select! {
        biased;
        () = stop.stopped() => {
            let (e, _) = tokio::join!(
                tell_stopped(conn, &mut chan.sealer, &mut chan.send, transfer_id),
                tokio::time::timeout(STOP_NOTICE, &mut prep),
            );
            return Err(e);
        }
        res = &mut prep => res.map_err(|_| Error::Protocol("preparation task failed"))??,
    };

    let (offer, renames, targets, wanted, mut created) = match prep {
        PrepareOutcome::Stopped => {
            return Err(tell_stopped(conn, &mut chan.sealer, &mut chan.send, transfer_id).await);
        }
        PrepareOutcome::NoSpace(io_err) => {
            return Err(reject_no_space(conn, &mut chan, transfer_id, io_err).await);
        }
        PrepareOutcome::Ready {
            offer,
            renames,
            targets,
            wanted,
            created,
        } => (*offer, renames, targets, wanted, created),
    };
    let dest = dest_owned.as_path();

    if let Err(e) = send_control(
        &mut chan.sealer,
        &mut chan.send,
        &Control::Accept {
            transfer_id,
            wanted: wanted.clone(),
            progress: offer.reports,
        },
    )
    .await
    {
        drop_created_blocking(created).await;
        return Err(e);
    }

    let total = offer.manifest.total_size;
    let targets = Arc::new(targets);
    let claimed = Arc::new(std::sync::Mutex::new(HashSet::new()));
    let done = Arc::new(AtomicU64::new(0));
    let permits = Arc::new(Semaphore::new(PARALLEL_STREAMS));
    let (result_tx, mut result_rx) = mpsc::channel::<Result<(u64, bool)>>(16);

    // Control messages from the sender (a cancel) arrive on their own task.
    let SecureChannel {
        mut sealer,
        opener,
        mut send,
        recv,
        ..
    } = chan;
    let (_reader, mut ctl_rx) = control_reader(opener, recv);
    let abandoned = Arc::new(std::sync::atomic::AtomicBool::new(false));
    // Every data stream task belongs to this call: returning for any reason
    // (done, an error, a stop) aborts the ones still running.
    let mut streams = tokio::task::JoinSet::new();

    let mut report = ReceiveReport::default();
    for (_, to) in &renames {
        report.placed.push(dest.join(to));
    }
    let mut pending = wanted.len();
    let mut last_activity = tokio::time::Instant::now();
    let done_at_check = Arc::clone(&done);
    let mut seen_bytes = 0u64;
    // Tell a sender that asked for it how much has arrived (#211).
    let mut told = 0u64;
    let mut report_tick = tokio::time::interval(PROGRESS_EVERY);
    report_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let outcome: Result<()> = async {
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
                        return Err(Error::Connect(crate::error::msg::SENDER_STOPPED_SENDING.into()));
                    }
                }
                stream = conn.accept_uni() => {
                    let stream = stream?;
                    let tx = result_tx.clone();
                    let (targets, claimed, ck, done, progress, permits, abandoned) = (
                        targets.clone(), claimed.clone(), offer.content_key.clone(),
                        done.clone(), progress.clone(), permits.clone(), abandoned.clone(),
                    );
                    // The permit is waited for inside the task, so this loop
                    // keeps hearing stops, cancels and the inactivity timer.
                    streams.spawn(async move {
                        let r = match permits.acquire_owned().await {
                            Ok(_permit) => {
                                receive_stream(stream, targets, claimed, ck, transfer_id, done, total, progress, abandoned).await
                            }
                            Err(_) => Err(Error::Protocol("semaphore closed")),
                        };
                        let _ = tx.send(r).await;
                    });
                    // Reap finished tasks so the set does not grow.
                    while streams.try_join_next().is_some() {}
                }
                Some(r) = result_rx.recv() => {
                    last_activity = tokio::time::Instant::now();
                    let (index, ok) = match r {
                    Ok(v) => v,
                    // A stream can break because the sender stopped; its
                    // cancel may still be on the way on the control stream.
                    Err(e) => return Err(cancel_or(e, &mut ctl_rx).await),
                };
                    if ok { report.files_ok = report.files_ok.saturating_add(1); }
                    else { report.files_failed = report.files_failed.saturating_add(1); }
                    send_control(&mut sealer, &mut send,
                        &Control::EntryDone { transfer_id, index, ok }).await?;
                    pending = pending.saturating_sub(1);
                }
                msg = ctl_rx.recv() => match msg {
                    Some(Ok(Control::Cancel { reason, .. })) => return Err(Error::Cancelled(reason)),
                    Some(Ok(_)) => return Err(Error::Protocol("unexpected control message")),
                    Some(Err(e)) => return Err(e),
                    None => return Err(control_ended(conn)),
                },
                _ = report_tick.tick(), if offer.reports => {
                    let bytes = done_at_check.load(Ordering::Relaxed);
                    if bytes != told {
                        send_control(&mut sealer, &mut send,
                            &Control::Received { transfer_id, bytes }).await?;
                        told = bytes;
                    }
                }
                () = stop.stopped() => {
                    // Cancel and close first: dropping a stream that is still
                    // reading tells the sender to stop sending, which would
                    // reach it before the cancel.
                    let e = tell_stopped(conn, &mut sealer, &mut send, transfer_id).await;
                    abandoned.store(true, Ordering::SeqCst);
                    streams.abort_all();
                    return Err(e);
                }
            }
        }
        Ok(())
    }
    .await;
    if let Err(e) = outcome {
        // No file is placed from now on; let every stream task end and drop
        // its partial file before the folders this transfer created go.
        abandoned.store(true, Ordering::SeqCst);
        streams.shutdown().await;
        drop_created_blocking(created).await;
        return Err(e);
    }
    created.keep = true;
    report.bytes = done.load(Ordering::Relaxed);
    let _ = send.finish();
    // Wait briefly for the sender to close, so our last messages arrive.
    let _ = tokio::time::timeout(Duration::from_secs(5), conn.closed()).await;
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

    /// A stop cuts a file whose data is still on the way, even when the
    /// sender has handed all of it to the connection already: the stream
    /// is reset, so the receiver cannot complete the file from what
    /// arrives after the stop (#193). Before, the stream was dropped,
    /// which ends it cleanly, and the receiver read the whole file.
    #[tokio::test]
    async fn stop_cuts_a_file_still_on_the_way() {
        // ~1.2 MB/s, so the file takes several seconds to arrive. It fits
        // the stream window, so the sender hands all of it to the
        // connection at once. What is on the way at the stop is bounded by
        // the relay pipe's queue (path::QUEUE datagrams, about 1.3 MB),
        // well under what is left to send.
        let (sender, receiver) =
            crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
        let size = 6 * limits::CHUNK_SIZE;
        let dir = std::env::temp_dir().join(format!("td-cut-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("cut.bin");
        std::fs::write(&source, vec![7u8; usize::try_from(size).unwrap()]).unwrap();
        let entry = Entry {
            path: vec!["cut.bin".into()],
            kind: EntryKind::File,
            size,
            hash: Some([0; 32]),
            mtime: None,
        };
        let stop = Stop::new();
        let done = Arc::new(AtomicU64::new(0));
        let task = tokio::spawn(send_entry_until_stopped(
            sender.clone(),
            stop.clone(),
            Arc::new(Semaphore::new(1)),
            Source::Path(source),
            entry,
            0,
            ContentKey::generate().unwrap(),
            [1; 16],
            done.clone(),
        ));
        let mut stream = receiver.accept_uni().await.unwrap();
        expect_frame(&mut stream, 1024).await.unwrap();
        expect_frame(&mut stream, MAX_FRAME).await.unwrap();
        // The first chunk is here; the rest is still on the way, although
        // the sender has handed all of it over.
        assert_eq!(done.load(Ordering::Relaxed), size);
        stop.stop();
        assert!(matches!(task.await.unwrap(), Err(Error::Stopped)));
        let mut buf = vec![0u8; 64 * 1024];
        let end = loop {
            match stream.read(&mut buf).await {
                Ok(Some(_)) => {}
                other => break other,
            }
        };
        assert!(
            matches!(end, Err(quinn::ReadError::Reset(code)) if code == STREAM_STOPPED),
            "the file was not cut: {end:?}"
        );
        // The receiver reports it as the sender stopping, even before the
        // cancel message arrives.
        assert!(matches!(Error::from(end.unwrap_err()), Error::Cancelled(r) if r == "stopped"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Both ends of a transfer over `over_slow_relay`, with a secure
    /// channel between them.
    async fn secure_pair(
        sender: &quinn::Connection,
        receiver: &quinn::Connection,
    ) -> (SecureChannel, SecureChannel, DeviceKeys, Certificate) {
        let (sk, rk) = (
            DeviceKeys::generate().unwrap(),
            DeviceKeys::generate().unwrap(),
        );
        let (sc, rc) = (
            crate::pairing::issue_certificate(&sk),
            crate::pairing::issue_certificate(&rk),
        );
        let input = |keys, peer| crate::secure::HandshakeInput {
            keys,
            peer,
            psk: [3; 32],
            mailbox_id: [4; 16],
        };
        let (s, r) = tokio::join!(
            crate::secure::handshake(sender, input(&sk, &rc), false),
            crate::secure::handshake(receiver, input(&rk, &sc), true),
        );
        (s.unwrap(), r.unwrap(), sk, sc)
    }

    /// The sender's bar shows what has arrived, not what QUIC has taken
    /// (#211). Over a slow path the whole file fits QUIC's buffers, so
    /// before, the sender showed it all sent while most of it was still on
    /// the way.
    #[tokio::test]
    async fn sender_progress_follows_what_arrived() {
        progress_over_a_slow_path(true).await;
    }

    /// A receiver that does not report (as before #211) gets no reports
    /// asked of it, and its sender shows only a lower bound.
    #[tokio::test]
    async fn sender_estimates_progress_without_reports() {
        progress_over_a_slow_path(false).await;
    }

    #[test]
    fn reports_must_grow_within_what_was_asked() {
        assert_eq!(check_report(0, None, 10).unwrap(), 0);
        assert_eq!(check_report(10, Some(3), 10).unwrap(), 10);
        assert!(check_report(11, None, 10).is_err());
        assert!(check_report(3, Some(3), 10).is_err());
        assert!(check_report(2, Some(3), 10).is_err());
    }

    /// The receiver stops while its streams wait for data that is slow to
    /// come: the sender hears it stopped, and nothing is left behind.
    #[tokio::test]
    async fn receiver_stops_while_waiting_for_data() {
        let (sender, receiver) =
            crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
        let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
        let dir = std::env::temp_dir().join(format!("td-rstop-{}", std::process::id()));
        let (src, dest) = (dir.join("src"), dir.join("dest"));
        std::fs::create_dir_all(src.join("f")).unwrap();
        std::fs::create_dir_all(&dest).unwrap();
        for n in 0..2 {
            std::fs::write(src.join("f").join(format!("{n}.bin")), vec![1u8; 4 << 20]).unwrap();
        }
        let (entries, _) = scan(&[src.join("f")]).unwrap();
        let send = {
            let sender = sender.clone();
            tokio::spawn(async move {
                send_files(
                    &sender,
                    schan,
                    &skeys,
                    entries,
                    false,
                    Arc::new(|_, _| {}),
                    &Stop::new(),
                )
                .await
            })
        };
        let offer = receive_offer(&mut rchan, &scert).await.unwrap();
        let stop = Stop::new();
        let on_receive: Progress = {
            let stop = stop.clone();
            Arc::new(move |n, _| {
                if n >= 1 << 20 {
                    stop.stop();
                }
            })
        };
        let r = accept(&receiver, rchan, offer, &dest, on_receive, &stop).await;
        assert!(matches!(r, Err(Error::Stopped)), "{r:?}");
        let s = tokio::time::timeout(Duration::from_secs(10), send)
            .await
            .unwrap()
            .unwrap();
        assert!(
            matches!(s, Err(Error::Cancelled(ref reason)) if reason == "stopped"),
            "{s:?}"
        );
        assert_eq!(std::fs::read_dir(&dest).unwrap().count(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A sender and receiver over a relay stand-in that forwards up to 16
    /// sender datagrams per millisecond (~9 MB/s): slow enough to catch a
    /// multi-megabyte transfer in flight, without queueing > 2 s of packets
    /// behind Tokio's 1 ms timer wheel.
    async fn paced_pair() -> (quinn::Connection, quinn::Connection) {
        use crate::transport::{
            EndpointKey, Paths, endpoint, receiver_connection, sender_connection,
        };
        use std::net::{SocketAddr, UdpSocket};

        let (ka, kb) = (
            EndpointKey::generate().unwrap(),
            EndpointKey::generate().unwrap(),
        );
        let sa = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let sb = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let (sender, pa, _) = endpoint(sa, &ka, &kb.public()).unwrap();
        let (receiver, pb, _) = endpoint(sb, &kb, &ka.public()).unwrap();
        let (mut a_out, a_in, mut b_out, b_in) =
            (pa.outgoing, pa.incoming, pb.outgoing, pb.incoming);
        tokio::spawn(async move {
            while let Some(first) = a_out.recv().await {
                tokio::time::sleep(Duration::from_millis(1)).await;
                let _ = b_in.send(first).await;
                for _ in 1..16 {
                    let Ok(d) = a_out.try_recv() else { break };
                    let _ = b_in.send(d).await;
                }
            }
        });
        tokio::spawn(async move {
            while let Some(d) = b_out.recv().await {
                let _ = a_in.send(d).await;
            }
        });
        let gate = crate::relay::Gate::default();
        gate.open();
        let paths = Paths {
            direct: Vec::new(),
            relay: Some(gate),
            relay_only: true,
        };
        let wait = Duration::from_secs(10);
        let recv_side = async {
            let conn = receiver_connection(&receiver, &paths, wait).await.unwrap();
            let (mut send, _recv) = conn.open_bi().await.unwrap();
            send.write_all(b"x").await.unwrap();
            (conn, send)
        };
        let send_side = async { sender_connection(&sender, &paths, wait).await.unwrap() };
        let ((receiver_conn, _ctl), (sender_conn, _send, _recv)) =
            tokio::join!(recv_side, send_side);
        (sender_conn, receiver_conn)
    }

    /// The sender stops mid-transfer (#359): both sides end within 2 s, the
    /// receiver hears `Cancelled("stopped")` rather than a connection error,
    /// and partial files and created folders are removed.
    #[tokio::test]
    async fn sender_stops_while_sending() {
        let (sender, receiver) = paced_pair().await;
        let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
        let dir = std::env::temp_dir().join(format!("td-sstop-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (src, dest) = (dir.join("src"), dir.join("dest"));
        std::fs::create_dir_all(src.join("f")).unwrap();
        std::fs::create_dir_all(&dest).unwrap();
        for n in 0..3 {
            std::fs::write(src.join("f").join(format!("{n}.bin")), vec![2u8; 6 << 20]).unwrap();
        }
        let (entries, _) = scan(&[src.join("f")]).unwrap();
        let stop = Stop::new();
        let send = {
            let (sender, stop) = (sender.clone(), stop.clone());
            tokio::spawn(async move {
                send_files(
                    &sender,
                    schan,
                    &skeys,
                    entries,
                    false,
                    Arc::new(|_, _| {}),
                    &stop,
                )
                .await
            })
        };
        let offer = receive_offer(&mut rchan, &scert).await.unwrap();
        let started = Arc::new(tokio::sync::Notify::new());
        let on_receive: Progress = {
            let started = Arc::clone(&started);
            Arc::new(move |n, _| {
                if n >= 1 << 20 {
                    started.notify_one();
                }
            })
        };
        let recv = tokio::spawn(async move {
            accept(&receiver, rchan, offer, &dest, on_receive, &Stop::new()).await
        });
        tokio::time::timeout(Duration::from_secs(10), started.notified())
            .await
            .unwrap();
        let stopped_at = tokio::time::Instant::now();
        stop.stop();
        let s = tokio::time::timeout(Duration::from_secs(2), send)
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(s, Err(Error::Stopped)), "{s:?}");
        let r = tokio::time::timeout(Duration::from_secs(2), recv)
            .await
            .unwrap()
            .unwrap();
        assert!(
            matches!(r, Err(Error::Cancelled(ref reason)) if reason == "stopped"),
            "{r:?}"
        );
        assert!(stopped_at.elapsed() < Duration::from_secs(2));
        assert_eq!(std::fs::read_dir(dir.join("dest")).unwrap().count(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// How the test receiver takes an offer.
    #[derive(Clone, Copy, PartialEq)]
    enum Take {
        /// Like the CLI after #313: a text in memory, files on disk.
        AsShown,
        /// Always on disk, like a receiver before #313 or one that saves.
        Saved,
    }

    /// Sends `entries` (as a text when `text`) and receives them into a
    /// fresh folder as `take` says. Returns whether the receiver saw a
    /// text, its report, what the sender heard, and the folder.
    async fn send_text(
        entries: Vec<SourceEntry>,
        text: bool,
        take: Take,
        tag: &str,
    ) -> (bool, ReceiveReport, Result<SendReport>, PathBuf) {
        let (sender, receiver) =
            crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
        let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
        let dest = std::env::temp_dir().join(format!("td-text-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dest);
        std::fs::create_dir_all(&dest).unwrap();
        let send = {
            let sender = sender.clone();
            tokio::spawn(async move {
                send_files(
                    &sender,
                    schan,
                    &skeys,
                    entries,
                    text,
                    Arc::new(|_, _| {}),
                    &Stop::new(),
                )
                .await
            })
        };
        let offer = receive_offer(&mut rchan, &scert).await.unwrap();
        let shown = offer.summary.text;
        let report = if shown && take == Take::AsShown {
            accept_text(&receiver, rchan, offer, &Stop::new()).await
        } else {
            accept(
                &receiver,
                rchan,
                offer,
                &dest,
                Arc::new(|_, _| {}),
                &Stop::new(),
            )
            .await
        }
        .unwrap();
        let sent = send.await.unwrap();
        (shown, report, sent, dest)
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut n: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        n.sort();
        n
    }

    /// One file entry holding `bytes` under `path`, with `hash`.
    fn bytes_entry(path: &[&str], bytes: &[u8], hash: [u8; 32]) -> SourceEntry {
        SourceEntry {
            entry: Entry {
                path: path.iter().map(|p| (*p).to_owned()).collect(),
                kind: EntryKind::File,
                size: bytes.len() as u64,
                hash: Some(hash),
                mtime: None,
            },
            source: Some(Source::Bytes(Arc::from(bytes))),
        }
    }

    /// A text arrives as a text: shown, with nothing written to disk (#313).
    #[tokio::test]
    async fn a_text_is_shown_not_saved() {
        let words = "Merhaba 🌸\nhttps://totaldrop.app";
        let entry = text_entry(words, "text.txt").unwrap();
        let (shown, report, sent, dest) = send_text(vec![entry], true, Take::AsShown, "new").await;
        assert!(shown);
        assert_eq!(report.text.as_deref(), Some(words));
        assert_eq!((report.files_ok, report.files_failed), (1, 0));
        assert!(report.placed.is_empty());
        assert!(names(&dest).is_empty(), "{:?}", names(&dest));
        assert!(sent.is_ok(), "{sent:?}");
        let _ = std::fs::remove_dir_all(&dest);
    }

    /// A receiver that saves (or one from before #313, which ignores the
    /// mark) gets the text as the file it also is.
    #[tokio::test]
    async fn a_saved_text_is_a_file() {
        let entry = text_entry("hello", "text.txt").unwrap();
        let (shown, report, _, dest) = send_text(vec![entry], true, Take::Saved, "old").await;
        assert!(shown);
        assert_eq!(report.text, None);
        assert_eq!(names(&dest), ["text.txt"]);
        assert_eq!(std::fs::read(dest.join("text.txt")).unwrap(), b"hello");
        let _ = std::fs::remove_dir_all(&dest);
    }

    /// The mark on anything but one small top-level file is ignored:
    /// files are saved as files.
    #[tokio::test]
    async fn only_one_small_file_can_be_a_text() {
        let two = vec![
            text_entry("a", "a.txt").unwrap(),
            text_entry("b", "b.txt").unwrap(),
        ];
        let (shown, report, _, dest) = send_text(two, true, Take::AsShown, "two").await;
        assert!(!shown);
        assert_eq!(report.text, None);
        assert_eq!(names(&dest), ["a.txt", "b.txt"]);
        let _ = std::fs::remove_dir_all(&dest);

        let big = vec![0x41_u8; usize::try_from(MAX_TEXT).unwrap() + 1];
        let entry = bytes_entry(&["big.txt"], &big, *blake3::hash(&big).as_bytes());
        let (shown, _, _, dest) = send_text(vec![entry], true, Take::AsShown, "big").await;
        assert!(!shown);
        assert_eq!(names(&dest), ["big.txt"]);
        let _ = std::fs::remove_dir_all(&dest);
    }

    /// An empty file is not a text: a sender never marks one, and a
    /// receiver does not take one as a text either.
    #[tokio::test]
    async fn an_empty_file_is_not_a_text() {
        let entry = bytes_entry(&["t.txt"], b"", *blake3::hash(b"").as_bytes());
        let (shown, report, _, dest) = send_text(vec![entry], true, Take::AsShown, "empty").await;
        assert!(!shown);
        assert_eq!(report.text, None);
        assert_eq!(names(&dest), ["t.txt"]);
        let _ = std::fs::remove_dir_all(&dest);
    }

    /// A single file below a folder is not a text either.
    #[tokio::test]
    async fn a_nested_file_is_not_a_text() {
        let entry = bytes_entry(&["d", "t.txt"], b"x", *blake3::hash(b"x").as_bytes());
        let (shown, report, _, dest) = send_text(vec![entry], true, Take::AsShown, "nest").await;
        assert!(!shown);
        assert_eq!(report.text, None);
        assert_eq!(std::fs::read(dest.join("d").join("t.txt")).unwrap(), b"x");
        let _ = std::fs::remove_dir_all(&dest);
    }

    /// Bytes that are not UTF-8 are not shown, and nothing is saved: the
    /// file is counted as failed and the sender hears so.
    #[tokio::test]
    async fn a_text_that_is_not_utf8_is_dropped() {
        let bytes = [0xff_u8, 0xfe, 0x00, 0x41];
        let entry = bytes_entry(&["odd.txt"], &bytes, *blake3::hash(&bytes).as_bytes());
        let (shown, report, sent, dest) = send_text(vec![entry], true, Take::AsShown, "odd").await;
        assert!(shown);
        assert_eq!(report.text, None);
        assert_eq!((report.files_ok, report.files_failed), (0, 1));
        assert!(names(&dest).is_empty());
        assert!(
            !matches!(
                sent,
                Ok(SendReport {
                    files_failed: 0,
                    ..
                })
            ),
            "{sent:?}"
        );
        let _ = std::fs::remove_dir_all(&dest);
    }

    /// A text whose hash does not match is not shown.
    #[tokio::test]
    async fn a_text_with_the_wrong_hash_is_dropped() {
        let entry = bytes_entry(&["t.txt"], b"hello", *blake3::hash(b"other").as_bytes());
        let (shown, report, _, dest) = send_text(vec![entry], true, Take::AsShown, "hash").await;
        assert!(shown);
        assert_eq!(report.text, None);
        assert_eq!((report.files_ok, report.files_failed), (0, 1));
        assert!(names(&dest).is_empty());
        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn text_limits() {
        assert!(text_entry("", "t.txt").is_err());
        assert!(text_entry(&"a".repeat(usize::try_from(MAX_TEXT).unwrap()), "t.txt").is_ok());
        assert!(text_entry(&"a".repeat(usize::try_from(MAX_TEXT).unwrap() + 1), "t.txt").is_err());
        assert!(text_entry("x", "../t.txt").is_err());
    }

    #[test]
    fn the_estimate_never_runs_ahead() {
        assert_eq!(lower_bound(0, MAX_IN_FLIGHT), 0);
        assert_eq!(lower_bound(0, MAX_IN_FLIGHT + 5), 5);
        assert_eq!(lower_bound(9, MAX_IN_FLIGHT + 5), 9);
        assert_eq!(MAX_IN_FLIGHT, 32 << 20);
    }

    /// Send one file over a slow path and check the sender's bar never
    /// shows more than had arrived. One chunk of slack covers a report
    /// racing the receiver's own progress callback.
    async fn progress_over_a_slow_path(reports: bool) {
        let (sender, receiver) =
            crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
        let (schan, rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
        let size = 6 * limits::CHUNK_SIZE;
        let dir =
            std::env::temp_dir().join(format!("td-progress-{reports}-{}", std::process::id()));
        let (src, dest) = (dir.join("src"), dir.join("dest"));
        std::fs::create_dir_all(&src).unwrap();
        std::fs::create_dir_all(&dest).unwrap();
        std::fs::write(src.join("p.bin"), vec![5u8; usize::try_from(size).unwrap()]).unwrap();
        let (entries, _) = scan(&[src.join("p.bin")]).unwrap();

        let arrived = Arc::new(AtomicU64::new(0));
        let ahead = Arc::new(AtomicU64::new(0));
        let last_shown = Arc::new(AtomicU64::new(0));
        let between = Arc::new(AtomicU64::new(0));
        let on_receive: Progress = {
            let arrived = arrived.clone();
            Arc::new(move |n, _| {
                arrived.fetch_max(n, Ordering::SeqCst);
            })
        };
        let on_send: Progress = {
            let (arrived, ahead, last_shown, between) = (
                arrived.clone(),
                ahead.clone(),
                last_shown.clone(),
                between.clone(),
            );
            Arc::new(move |n, total| {
                assert_eq!(total, size);
                if n > 0 && n < size {
                    between.fetch_add(1, Ordering::SeqCst);
                }
                let got = arrived.load(Ordering::SeqCst);
                ahead.fetch_max(n.saturating_sub(got), Ordering::SeqCst);
                last_shown.store(n, Ordering::SeqCst);
            })
        };
        let stop = Stop::new();
        let send = {
            let (sender, stop) = (sender.clone(), stop.clone());
            tokio::spawn(async move {
                send_files(&sender, schan, &skeys, entries, false, on_send, &stop).await
            })
        };
        let mut rchan = rchan;
        let mut offer = receive_offer(&mut rchan, &scert).await.unwrap();
        assert!(offer.reports);
        // As an old receiver: it ignores the field, so it never reports. A
        // report sent anyway would fail the sender.
        offer.reports = reports;
        let received = accept(&receiver, rchan, offer, &dest, on_receive, &stop)
            .await
            .unwrap();
        let sent = send.await.unwrap().unwrap();
        assert_eq!((sent.files_ok, received.files_ok), (1, 1));
        let ahead = ahead.load(Ordering::SeqCst);
        assert!(
            ahead <= limits::CHUNK_SIZE,
            "the sender showed {ahead} bytes more than had arrived"
        );
        // It still ends at everything, and with reports it moves on the
        // way there.
        assert_eq!(last_shown.load(Ordering::SeqCst), size);
        if reports {
            assert!(between.load(Ordering::SeqCst) > 0, "no report was shown");
        }
        let _ = std::fs::remove_dir_all(&dir);
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

    #[test]
    fn destination_checks() {
        let mut tag = [0u8; 6];
        getrandom::fill(&mut tag).unwrap();
        let base = std::env::temp_dir().join(format!(
            "totaldrop-dest-{}",
            tag.iter().map(|b| format!("{b:02x}")).collect::<String>()
        ));
        let real = base.join("real");
        std::fs::create_dir_all(&real).unwrap();
        // A folder, and a symbolic link to it given as the folder.
        assert_eq!(check_destination(&real).unwrap(), real);
        #[cfg(unix)]
        {
            let link = base.join("link");
            std::os::unix::fs::symlink(&real, &link).unwrap();
            assert_eq!(check_destination(&link).unwrap(), link);
        }
        // Missing, and a file.
        let missing = check_destination(&base.join("nope")).unwrap_err();
        assert!(matches!(missing, Error::Io(ref e) if e.kind() == std::io::ErrorKind::NotFound));
        std::fs::write(base.join("f"), b"x").unwrap();
        let file = check_destination(&base.join("f")).unwrap_err();
        assert!(matches!(file, Error::Io(ref e) if e.kind() == std::io::ErrorKind::NotADirectory));
        // The probe leaves nothing behind.
        assert_eq!(std::fs::read_dir(&real).unwrap().count(), 0);
        std::fs::remove_dir_all(&base).unwrap();
    }

    fn temp_test_dir(label: &str) -> PathBuf {
        let mut tag = [0u8; 6];
        getrandom::fill(&mut tag).unwrap();
        let dir = std::env::temp_dir().join(format!(
            "td-455-{label}-{}",
            tag.iter().map(|b| format!("{b:02x}")).collect::<String>()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// #455: directory creation runs on `spawn_blocking` rather than stalling
    /// a single-threaded async executor, checks `stop` between directories,
    /// cleans up directories already created, and tells the sender.
    #[test]
    fn directory_creation_runs_off_the_async_thread_and_stops_cleanly() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let runtime_thread = std::thread::current().id();
        let base = temp_test_dir("dirs");
        let dest = base.join("dest");
        std::fs::create_dir_all(&dest).unwrap();

        rt.block_on(async {
            let (sender, receiver) =
                crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
            let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
            let entries = vec![
                bytes_entry(
                    &["dir_a", "sub", "one.bin"],
                    b"hello",
                    *blake3::hash(b"hello").as_bytes(),
                ),
                bytes_entry(
                    &["dir_b", "sub", "two.bin"],
                    b"world",
                    *blake3::hash(b"world").as_bytes(),
                ),
            ];
            let send = {
                let sender = sender.clone();
                tokio::spawn(async move {
                    send_files(
                        &sender,
                        schan,
                        &skeys,
                        entries,
                        false,
                        Arc::new(|_, _| {}),
                        &Stop::new(),
                    )
                    .await
                })
            };

            let offer = receive_offer(&mut rchan, &scert).await.unwrap();
            let stop = Stop::new();
            let (entered_tx, entered_rx) = tokio::sync::oneshot::channel::<()>();
            let (resume_tx, resume_rx) = std::sync::mpsc::channel::<()>();
            let dirs_created = Arc::new(AtomicU64::new(0));
            let ran_off_async_thread = Arc::new(std::sync::atomic::AtomicBool::new(false));

            let stopper = {
                let stop = stop.clone();
                tokio::spawn(async move {
                    entered_rx.await.unwrap();
                    stop.stop();
                    resume_tx.send(()).unwrap();
                })
            };

            let dirs_seen = Arc::clone(&dirs_created);
            let off_thread = Arc::clone(&ran_off_async_thread);
            let mut entered_tx = Some(entered_tx);
            let r = accept_with_hooks(
                &receiver,
                rchan,
                offer,
                &dest,
                Arc::new(|_, _| {}),
                &stop,
                |p| fs4::available_space(p),
                probe_destination_writable,
                move |_dir| {
                    let seen = dirs_seen.fetch_add(1, Ordering::SeqCst).saturating_add(1);
                    off_thread.store(
                        std::thread::current().id() != runtime_thread,
                        Ordering::SeqCst,
                    );
                    if seen == 2
                        && let Some(tx) = entered_tx.take()
                    {
                        let _ = tx.send(());
                        assert!(
                            resume_rx.recv_timeout(Duration::from_secs(10)).is_ok(),
                            "directory creation blocked the single-threaded async runtime"
                        );
                    }
                },
            )
            .await;

            stopper.await.unwrap();
            assert!(
                ran_off_async_thread.load(Ordering::SeqCst),
                "directory creation ran on the async runtime thread"
            );
            assert_eq!(
                dirs_created.load(Ordering::SeqCst),
                2,
                "stop did not halt directory creation before dir_b"
            );
            assert!(
                matches!(r, Err(Error::Stopped)),
                "expected Stopped, got {r:?}"
            );
            let s = tokio::time::timeout(Duration::from_secs(5), send)
                .await
                .unwrap()
                .unwrap();
            assert!(
                matches!(s, Err(Error::Cancelled(ref reason)) if reason == "stopped"),
                "expected Cancelled(\"stopped\"), got {s:?}"
            );
            assert_eq!(
                std::fs::read_dir(&dest).unwrap().count(),
                0,
                "created directory was not removed on stop"
            );
        });

        let _ = std::fs::remove_dir_all(&base);
    }

    /// #455: pressing stop while a directory syscall is still hung notifies
    /// the sender immediately via `tokio::select!` without waiting for the
    /// blocking syscall to return first.
    #[tokio::test]
    async fn stop_notifies_sender_before_hung_directory_syscall_returns() {
        let base = temp_test_dir("hung-stop");
        let dest = base.join("dest");
        std::fs::create_dir_all(&dest).unwrap();

        let (sender, receiver) =
            crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
        let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
        let entries = vec![bytes_entry(
            &["dir_a", "one.bin"],
            b"hello",
            *blake3::hash(b"hello").as_bytes(),
        )];
        let send = {
            let sender = sender.clone();
            tokio::spawn(async move {
                send_files(
                    &sender,
                    schan,
                    &skeys,
                    entries,
                    false,
                    Arc::new(|_, _| {}),
                    &Stop::new(),
                )
                .await
            })
        };

        let offer = receive_offer(&mut rchan, &scert).await.unwrap();
        let stop = Stop::new();
        let (entered_tx, entered_rx) = tokio::sync::oneshot::channel::<()>();
        let (resume_tx, resume_rx) = std::sync::mpsc::channel::<()>();

        let recv_task = {
            let receiver = receiver.clone();
            let dest_clone = dest.clone();
            let stop_clone = stop.clone();
            let mut entered_tx = Some(entered_tx);
            tokio::spawn(async move {
                accept_with_hooks(
                    &receiver,
                    rchan,
                    offer,
                    &dest_clone,
                    Arc::new(|_, _| {}),
                    &stop_clone,
                    |p| fs4::available_space(p),
                    probe_destination_writable,
                    move |_dir| {
                        if let Some(tx) = entered_tx.take() {
                            let _ = tx.send(());
                            assert!(
                                resume_rx.recv_timeout(Duration::from_secs(10)).is_ok(),
                                "hung directory syscall was never resumed"
                            );
                        }
                    },
                )
                .await
            })
        };

        entered_rx.await.unwrap();
        // Trigger stop while `on_dir` is still blocked on `resume_rx`.
        stop.stop();
        // Prove the sender receives Control::Cancel("stopped") BEFORE the
        // hung syscall is unblocked!
        let s = tokio::time::timeout(Duration::from_secs(5), send)
            .await
            .unwrap()
            .unwrap();
        assert!(
            matches!(s, Err(Error::Cancelled(ref reason)) if reason == "stopped"),
            "expected sender to hear Cancelled(\"stopped\") while syscall is hung, got {s:?}"
        );

        // Now unblock the hung syscall and verify receiver cleanup completes.
        resume_tx.send(()).unwrap();
        let r = tokio::time::timeout(Duration::from_secs(5), recv_task)
            .await
            .unwrap()
            .unwrap();
        assert!(
            matches!(r, Err(Error::Stopped)),
            "expected Stopped, got {r:?}"
        );
        assert_eq!(
            std::fs::read_dir(&dest).unwrap().count(),
            0,
            "created directory must be removed once the blocking task finishes"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    /// #455: when free space on `dest` is less than `offer.manifest.total_size`
    /// OR when the destination write probe fails with `StorageFull` /
    /// `QuotaExceeded` (e.g. 0 free inodes or user quota exhausted), `accept`
    /// rejects with `RejectReason::NoSpace`, returns `Cause::DiskFull` locally,
    /// and creates nothing under `dest` — both when the sender closes on error
    /// like `PreparedSend::run_with` and when the receiver immediately closes
    /// with `b"done"` like `IncomingOffer::accept_with`.
    #[tokio::test]
    async fn insufficient_disk_space_rejects_with_no_space() {
        let base = temp_test_dir("nospace");

        type SpaceHook = Box<dyn FnOnce(&Path) -> std::io::Result<u64> + Send>;
        type ProbeHook = Box<dyn FnOnce(&Path) -> Result<()> + Send>;
        let cases: Vec<(&str, bool, SpaceHook, ProbeHook, std::io::ErrorKind)> = vec![
            (
                "short-bytes-sender-closes",
                true,
                Box::new(|_| Ok(4095)),
                Box::new(probe_destination_writable),
                std::io::ErrorKind::StorageFull,
            ),
            (
                "short-bytes-receiver-closes-first",
                false,
                Box::new(|_| Ok(4095)),
                Box::new(probe_destination_writable),
                std::io::ErrorKind::StorageFull,
            ),
            (
                "probe-storage-full",
                true,
                Box::new(|_| Ok(u64::MAX)),
                Box::new(|_| {
                    Err(Error::Io(std::io::Error::new(
                        std::io::ErrorKind::StorageFull,
                        "no inodes left",
                    )))
                }),
                std::io::ErrorKind::StorageFull,
            ),
            (
                "probe-quota-exceeded",
                true,
                Box::new(|_| Err(std::io::Error::from(std::io::ErrorKind::Unsupported))),
                Box::new(|_| {
                    Err(Error::Io(std::io::Error::new(
                        std::io::ErrorKind::QuotaExceeded,
                        "disk quota exceeded",
                    )))
                }),
                std::io::ErrorKind::QuotaExceeded,
            ),
        ];

        for (label, sender_closes_on_err, space_fn, probe_fn, expected_kind) in cases {
            let dest = base.join(label);
            std::fs::create_dir_all(&dest).unwrap();

            let (sender, receiver) =
                crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
            let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
            let payload = vec![0x5a_u8; 4096];
            let entries = vec![bytes_entry(
                &["album", "photo.bin"],
                &payload,
                *blake3::hash(&payload).as_bytes(),
            )];
            let send = {
                let sender = sender.clone();
                tokio::spawn(async move {
                    let r = send_files(
                        &sender,
                        schan,
                        &skeys,
                        entries,
                        false,
                        Arc::new(|_, _| {}),
                        &Stop::new(),
                    )
                    .await;
                    if sender_closes_on_err && r.is_err() {
                        sender.close(0u32.into(), b"failed");
                    }
                    r
                })
            };

            let offer = receive_offer(&mut rchan, &scert).await.unwrap();
            let r = accept_with_hooks(
                &receiver,
                rchan,
                offer,
                &dest,
                Arc::new(|_, _| {}),
                &Stop::new(),
                space_fn,
                probe_fn,
                |_| {},
            )
            .await;
            // Match IncomingOffer::accept_with closing right after accept returns.
            receiver.close(0u32.into(), b"done");

            let err = r.unwrap_err();
            assert!(
                matches!(err, Error::Io(ref e) if e.kind() == expected_kind),
                "case {label}: expected {expected_kind:?} Io error, got {err:?}"
            );
            assert_eq!(err.cause(), crate::error::Cause::DiskFull, "case {label}");

            let s = tokio::time::timeout(Duration::from_secs(5), send)
                .await
                .unwrap()
                .unwrap();
            assert!(
                matches!(s, Err(Error::Rejected(RejectReason::NoSpace))),
                "case {label}: expected Rejected(NoSpace), got {s:?}"
            );
            assert_eq!(
                std::fs::read_dir(&dest).unwrap().count(),
                0,
                "case {label}: rejecting for low space must create nothing under dest"
            );
        }

        let _ = std::fs::remove_dir_all(&base);
    }

    /// #455: exact fit (`free == total_size`), 0-byte offers (`free == 0`), and
    /// filesystems where `available_space` returns `Err` all proceed normally.
    #[tokio::test]
    async fn exact_space_zero_byte_and_unqueryable_filesystems_accept() {
        let base = temp_test_dir("space-ok");
        assert!(
            fs4::available_space(&base).is_ok(),
            "fs4::available_space should succeed on the temporary directory"
        );

        type SpaceHook = Box<dyn FnOnce(&Path) -> std::io::Result<u64> + Send>;
        let cases: Vec<(&str, &[u8], SpaceHook)> = vec![
            ("exact", b"exact-fit", Box::new(|_| Ok(9))),
            ("zero", b"", Box::new(|_| Ok(0))),
            (
                "unsupported",
                b"virtual-fs",
                Box::new(|_| Err(std::io::Error::from(std::io::ErrorKind::Unsupported))),
            ),
        ];

        for (name, bytes, space_fn) in cases {
            let dest = base.join(name);
            std::fs::create_dir_all(&dest).unwrap();
            let (sender, receiver) =
                crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
            let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
            let entries = vec![bytes_entry(
                &["item.bin"],
                bytes,
                *blake3::hash(bytes).as_bytes(),
            )];
            let send = {
                let sender = sender.clone();
                tokio::spawn(async move {
                    send_files(
                        &sender,
                        schan,
                        &skeys,
                        entries,
                        false,
                        Arc::new(|_, _| {}),
                        &Stop::new(),
                    )
                    .await
                })
            };
            let offer = receive_offer(&mut rchan, &scert).await.unwrap();
            let received = accept_with_hooks(
                &receiver,
                rchan,
                offer,
                &dest,
                Arc::new(|_, _| {}),
                &Stop::new(),
                space_fn,
                probe_destination_writable,
                |_| {},
            )
            .await
            .unwrap();
            let sent = send.await.unwrap().unwrap();
            assert_eq!((sent.files_ok, received.files_ok), (1, 1), "case {name}");
            assert_eq!(std::fs::read(dest.join("item.bin")).unwrap(), bytes);
        }

        let _ = std::fs::remove_dir_all(&base);
    }
}
