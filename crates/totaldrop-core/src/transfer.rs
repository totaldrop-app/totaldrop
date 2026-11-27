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
/// arrive on the control stream: it may explain the failure. If a
/// `"stopped"` connection close overtook the cancel frame, `ctl` or `conn`
/// carries that too.
async fn cancel_or(
    err: Error,
    ctl: &mut mpsc::Receiver<Result<Control>>,
    conn: &quinn::Connection,
) -> Error {
    if matches!(err, Error::Cancelled(_)) {
        return err;
    }
    let wait = async {
        // Messages already on the way (an entry_done) may come first.
        while let Some(msg) = ctl.recv().await {
            match msg {
                Ok(Control::Cancel { reason, .. }) | Err(Error::Cancelled(reason)) => {
                    return Some(reason);
                }
                _ => {}
            }
        }
        if let Error::Cancelled(reason) = control_ended(conn) {
            return Some(reason);
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
/// error, a stop, an aborted task) schedules removal through `gate` on a
/// tracked blocking task when available so the async executor never blocks on
/// `remove_file` and `PlacementGate::shutdown` awaits every unlink before
/// created directories are removed.
struct PartFile {
    path: PathBuf,
    keep: bool,
    gate: Option<Arc<PlacementGate>>,
}

impl Drop for PartFile {
    fn drop(&mut self) {
        if !self.keep {
            let path = std::mem::take(&mut self.path);
            if !path.as_os_str().is_empty() {
                if let Some(gate) = self.gate.take() {
                    gate.spawn_unlink(path);
                } else {
                    let _ = std::fs::remove_file(path);
                }
            }
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

type PlacementHook = Arc<dyn Fn() -> Result<()> + Send + Sync>;

/// Coordinates blocking file placements and `.part` unlinks with stopping or
/// aborting a receive:
/// - normal placements check `halted` via an atomic load without acquiring any
///   blocking lock, so all `PARALLEL_STREAMS` streams place files concurrently;
/// - once `stop` fires or `signal_halt` runs, no new placement may begin;
/// - every placement and `.part` unlink `JoinHandle` is tracked (pruning
///   finished handles via `retain(|h| !h.is_finished())`), and `shutdown`
///   awaits all of them before `accept` removes created directories or returns.
struct PlacementGate {
    halted: std::sync::atomic::AtomicBool,
    tasks: std::sync::Mutex<PlacementTasks>,
    hook: Option<PlacementHook>,
}

struct PlacementTasks {
    closed: bool,
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl PlacementGate {
    fn new(hook: Option<PlacementHook>) -> Self {
        Self {
            halted: std::sync::atomic::AtomicBool::new(false),
            tasks: std::sync::Mutex::new(PlacementTasks {
                closed: false,
                handles: Vec::new(),
            }),
            hook,
        }
    }

    fn signal_halt(&self) {
        self.halted.store(true, Ordering::SeqCst);
        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.closed = true;
        }
    }

    fn is_halted(&self) -> bool {
        self.halted.load(Ordering::SeqCst)
    }

    fn spawn_unlink(&self, path: PathBuf) {
        if let Ok(rt) = tokio::runtime::Handle::try_current()
            && let Ok(mut tasks) = self.tasks.lock()
        {
            tasks.handles.retain(|h| !h.is_finished());
            tasks.handles.push(rt.spawn_blocking(move || {
                let _ = std::fs::remove_file(path);
            }));
            return;
        }
        let _ = std::fs::remove_file(path);
    }

    async fn cleanup_part(&self, mut part: PartFile) {
        if part.keep {
            return;
        }
        part.keep = true;
        part.gate = None;
        let path = std::mem::take(&mut part.path);
        if path.as_os_str().is_empty() {
            return;
        }
        let rx_opt = match self.tasks.lock() {
            Ok(mut tasks) => {
                tasks.handles.retain(|h| !h.is_finished());
                let (tx, rx) = tokio::sync::oneshot::channel();
                tasks.handles.push(tokio::task::spawn_blocking(move || {
                    let _ = std::fs::remove_file(path);
                    let _ = tx.send(());
                }));
                Some(rx)
            }
            Err(_) => {
                let _ = std::fs::remove_file(path);
                None
            }
        };
        if let Some(rx) = rx_opt {
            let _ = rx.await;
        }
    }

    async fn place_part(
        self: &Arc<Self>,
        stop: &Stop,
        part: PartFile,
        dir: PathBuf,
        name: String,
    ) -> Result<PathBuf> {
        match self.try_spawn_part(stop, part, dir, name)? {
            Ok(rx) => rx
                .await
                .map_err(|_| Error::Protocol("placement task failed"))?,
            Err(unplaced) => {
                self.cleanup_part(unplaced).await;
                Err(Error::Stopped)
            }
        }
    }

    fn try_spawn_part(
        self: &Arc<Self>,
        stop: &Stop,
        mut part: PartFile,
        dir: PathBuf,
        name: String,
    ) -> Result<core::result::Result<tokio::sync::oneshot::Receiver<Result<PathBuf>>, PartFile>>
    {
        if self.halted.load(Ordering::SeqCst) || stop.is_stopped() {
            return Ok(Err(part));
        }
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|_| Error::Protocol("internal lock"))?;
        if tasks.closed || self.halted.load(Ordering::SeqCst) || stop.is_stopped() {
            return Ok(Err(part));
        }
        tasks.handles.retain(|h| !h.is_finished());
        part.gate = None;
        let (tx, rx) = tokio::sync::oneshot::channel();
        let gate = Arc::clone(self);
        let stop_check = stop.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let res = if gate.halted.load(Ordering::SeqCst) || stop_check.is_stopped() {
                Err(Error::Stopped)
            } else {
                match &gate.hook {
                    Some(hook) => hook().and_then(|()| place_no_clobber(&part.path, &dir, &name)),
                    None => place_no_clobber(&part.path, &dir, &name),
                }
            };
            if res.is_ok() {
                part.keep = !part.path.exists();
            }
            drop(part);
            let _ = tx.send(res);
        });
        tasks.handles.push(handle);
        Ok(Ok(rx))
    }

    async fn shutdown(&self) {
        loop {
            let handles = match self.tasks.lock() {
                Ok(mut tasks) => std::mem::take(&mut tasks.handles),
                Err(_) => Vec::new(),
            };
            if handles.is_empty() {
                break;
            }
            for handle in handles {
                let _ = handle.await;
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
    if stop.is_stopped() {
        return Err(tell_stopped(conn, &mut chan.sealer, &mut chan.send, transfer_id).await);
    }
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
                // drops what has not been sent or read yet. Let them reset in
                // parallel while Control::Cancel is on the wire, and abort any
                // straggler once `tell_stopped` has closed the connection (#651).
                let e = tokio::select! {
                    e = tell_stopped(conn, &mut sealer, &mut send, transfer_id) => e,
                    () = async {
                        while tasks.join_next().await.is_some() {}
                        std::future::pending::<()>().await;
                    } => Error::Stopped,
                };
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
                Some(Ok(Control::Cancel { reason, .. })) => {
                    // Close before dropping tasks so CONNECTION_CLOSE leaves
                    // immediately and the peer enters passive draining (#651).
                    conn.close(0u32.into(), b"done");
                    return Err(Error::Cancelled(reason));
                }
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
                    Some(Ok(Err(e))) if matches!(e, Error::Stopped) || stop.is_stopped() => {
                        let _ = tokio::time::timeout(STOP_NOTICE, async {
                            while tasks.join_next().await.is_some() {}
                        })
                        .await;
                        let e = tell_stopped(conn, &mut sealer, &mut send, transfer_id).await;
                        tasks.abort_all();
                        return Err(e);
                    }
                    // A stream can fail because the receiver stopped; its
                    // cancel may still be on the way.
                    Some(Ok(Err(e))) => {
                        let e = cancel_or(e, &mut ctl_rx, conn).await;
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
    let (manifest, content_key, signature, reports, text) = match msg {
        Control::Offer {
            manifest,
            content_key,
            signature,
            progress,
            text,
        } => (manifest, content_key, signature, progress, text),
        Control::Cancel { reason, .. } => return Err(Error::Cancelled(reason)),
        _ => return Err(Error::Protocol("expected offer")),
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
    stop: Stop,
    gate: Arc<PlacementGate>,
) -> Result<(u64, Option<PathBuf>)> {
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
    let part = PartFile {
        path: tmp_path.clone(),
        keep: false,
        gate: Some(Arc::clone(&gate)),
    };
    let mut file = tokio::fs::File::from_std(std_file);

    let result: Result<bool> = async {
        let mut hasher = blake3::Hasher::new();
        let mut remaining = size;
        for i in 0..chunks {
            if gate.is_halted() {
                // Keep `stream` open until `accept` aborts us after
                // `tell_stopped`, so no `STOP_SENDING` races ahead of
                // `Control::Cancel`, while halting CPU and disk work now (#651).
                std::future::pending::<()>().await;
            }
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
            if gate.is_halted() {
                std::future::pending::<()>().await;
            }
            file.write_all(&plain).await?;
            remaining = remaining.saturating_sub(want);
            let now = done.fetch_add(want, Ordering::Relaxed).saturating_add(want);
            progress(now, total);
        }
        if read_frame(&mut stream, MAX_FRAME).await?.is_some() {
            return Err(Error::Protocol("data after the final chunk"));
        }
        if gate.is_halted() {
            return Err(Error::Stopped);
        }
        file.flush().await?;
        file.sync_all().await?;
        Ok(hasher.finalize().as_bytes() == &expected_hash)
    }
    .await;

    match result {
        Ok(true) => {
            if stop.is_stopped() {
                drop(file);
                gate.cleanup_part(part).await;
                return Err(Error::Stopped);
            }
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
                .ok_or(Error::Protocol("bad file name"))?
                .to_owned();
            let placed = gate.place_part(&stop, part, parent, name).await?;
            Ok((index, Some(placed)))
        }
        Ok(false) => {
            drop(file);
            gate.cleanup_part(part).await;
            Ok((index, None))
        }
        Err(e) => {
            drop(file);
            gate.cleanup_part(part).await;
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

/// Verify `dir` can create and unlink a temporary file. Checking metadata
/// permissions alone misses mount options (`ro`), POSIX/NFSv4 ACLs, quotas and
/// sandboxes, and `unsafe_code = "forbid"` rules out Linux-only `O_TMPFILE`,
/// so one named probe is created and immediately unlinked. Closing the handle
/// before unlinking and propagating any unlink error other than `NotFound`
/// ensures mounts that allow creation without unlink fail up front instead of
/// leaving `.totaldrop-*.part` files behind.
fn probe_destination_with_unlink<U>(dir: &Path, unlink: U) -> Result<()>
where
    U: FnOnce(&Path) -> std::io::Result<()>,
{
    let mut tag = [0u8; 8];
    getrandom::fill(&mut tag).map_err(|_| Error::Random)?;
    let probe = dir.join(format!(
        ".totaldrop-check-{}",
        tag.iter().map(|b| format!("{b:02x}")).collect::<String>()
    ));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)?;
    drop(file);
    match unlink(&probe) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::Io(e)),
    }
}

fn probe_destination_writable(dir: &Path) -> Result<()> {
    probe_destination_with_unlink(dir, |p| std::fs::remove_file(p))
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
            Ok(Err(e)) => return Err(cancel_or(e, &mut ctl_rx, conn).await),
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
        top_file_slots: std::collections::HashMap<u64, usize>,
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
    chan: SecureChannel,
    offer: VerifiedOffer,
    dest: &Path,
    progress: Progress,
    stop: &Stop,
    space_fn: S,
    probe_fn: P,
    on_dir: D,
) -> Result<ReceiveReport>
where
    S: FnOnce(&Path) -> std::io::Result<u64> + Send + 'static,
    P: FnOnce(&Path) -> Result<()> + Send + 'static,
    D: FnMut(&Path) + Send + 'static,
{
    accept_with_all_hooks(
        conn, chan, offer, dest, progress, stop, space_fn, probe_fn, on_dir, None,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn accept_with_all_hooks<S, P, D>(
    conn: &quinn::Connection,
    mut chan: SecureChannel,
    offer: VerifiedOffer,
    dest: &Path,
    progress: Progress,
    stop: &Stop,
    space_fn: S,
    probe_fn: P,
    mut on_dir: D,
    place_hook: Option<PlacementHook>,
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

        // Every top-level item gets a name beneath `canonical_dir` that does
        // not exist yet, so nothing already on disk is ever merged into or
        // overwritten, and retargeting a destination symlink mid-transfer
        // cannot split files across folders.
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
            let free = if std::fs::symlink_metadata(canonical_dir.join(top)).is_ok() {
                free_name(&canonical_dir, top, &taken)?
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
        let top_slot_by_name: std::collections::HashMap<&str, usize> = renames
            .iter()
            .enumerate()
            .map(|(idx, (from, _))| (from.as_str(), idx))
            .collect();
        let rename_map: std::collections::HashMap<&str, &str> = renames
            .iter()
            .map(|(from, to)| (from.as_str(), to.as_str()))
            .collect();
        let rename_top = |seg: &str| {
            rename_map
                .get(seg)
                .map_or_else(|| seg.to_owned(), |t| (*t).to_owned())
        };

        let mut top_file_slots = std::collections::HashMap::new();
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
            let mut path = canonical_dir.clone();
            for (n, seg) in safe.segments().iter().enumerate() {
                if n == 0 {
                    path.push(rename_top(seg));
                } else {
                    path.push(seg);
                }
                check_beneath(&canonical_dir, &path)?;
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
                let entry_idx = i as u64;
                if safe.segments().len() == 1
                    && let Some(first) = safe.segments().first()
                    && let Some(&slot) = top_slot_by_name.get(first.as_str())
                {
                    top_file_slots.insert(entry_idx, slot);
                }
                wanted.push((entry_idx, Vec::new()));
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
            top_file_slots,
            targets,
            wanted,
            created,
        })
    });

    let prep = tokio::select! {
        biased;
        () = stop.stopped() => {
            let (e, joined) = tokio::join!(
                tell_stopped(conn, &mut chan.sealer, &mut chan.send, transfer_id),
                tokio::time::timeout(STOP_NOTICE, &mut prep),
            );
            if let Ok(Ok(Ok(PrepareOutcome::Ready { created, .. }))) = joined {
                drop_created_blocking(created).await;
            }
            return Err(e);
        }
        res = &mut prep => res.map_err(|_| Error::Protocol("preparation task failed"))??,
    };

    let (offer, renames, top_file_slots, targets, wanted, mut created) = match prep {
        PrepareOutcome::Stopped => {
            return Err(tell_stopped(conn, &mut chan.sealer, &mut chan.send, transfer_id).await);
        }
        PrepareOutcome::NoSpace(io_err) => {
            return Err(reject_no_space(conn, &mut chan, transfer_id, io_err).await);
        }
        PrepareOutcome::Ready {
            offer,
            renames,
            top_file_slots,
            targets,
            wanted,
            created,
        } => (*offer, renames, top_file_slots, targets, wanted, created),
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
    let (result_tx, mut result_rx) = mpsc::channel::<Result<(u64, Option<PathBuf>)>>(16);

    // Control messages from the sender (a cancel) arrive on their own task.
    let SecureChannel {
        mut sealer,
        opener,
        mut send,
        recv,
        ..
    } = chan;
    let (_reader, mut ctl_rx) = control_reader(opener, recv);
    let gate = Arc::new(PlacementGate::new(place_hook));
    // Every data stream task belongs to this call: returning for any reason
    // (done, an error, a stop) aborts the ones still running.
    let mut streams = tokio::task::JoinSet::new();

    let top_file_slots_set: HashSet<usize> = top_file_slots.values().copied().collect();
    let mut top_placed: Vec<Option<PathBuf>> = renames
        .iter()
        .enumerate()
        .map(|(slot, (_, to))| {
            if top_file_slots_set.contains(&slot) {
                None
            } else {
                Some(dest.join(to))
            }
        })
        .collect();

    let mut report = ReceiveReport::default();
    let mut pending = wanted.len();
    let mut last_activity = tokio::time::Instant::now();
    let done_at_check = Arc::clone(&done);
    let mut seen_bytes = 0u64;
    // Tell a sender that asked for it how much has arrived (#211).
    let mut told = 0u64;
    let mut report_tick = tokio::time::interval(PROGRESS_EVERY);
    report_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut stopped_notified = false;
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
                biased;
                () = stop.stopped() => {
                    // Close the placement gate immediately so running streams
                    // stop decrypting, fsyncing, or placing files while
                    // `tell_stopped` sends the cancel notice (#92, #651).
                    gate.signal_halt();
                    // Cancel and close before aborting streams: dropping a
                    // stream that is still reading sends `STOP_SENDING`, which
                    // would otherwise reach the sender ahead of the cancel.
                    stopped_notified = true;
                    let e = tell_stopped(conn, &mut sealer, &mut send, transfer_id).await;
                    streams.abort_all();
                    return Err(e);
                }
                msg = ctl_rx.recv() => match msg {
                    Some(Ok(Control::Cancel { reason, .. })) => return Err(Error::Cancelled(reason)),
                    Some(Ok(_)) => return Err(Error::Protocol("unexpected control message")),
                    Some(Err(e)) => return Err(e),
                    None => return Err(control_ended(conn)),
                },
                Some(r) = result_rx.recv() => {
                    last_activity = tokio::time::Instant::now();
                    let (index, placed_opt) = match r {
                        Ok(v) => v,
                        Err(e) if matches!(e, Error::Stopped) || stop.is_stopped() => {
                            gate.signal_halt();
                            stopped_notified = true;
                            let e = tell_stopped(conn, &mut sealer, &mut send, transfer_id).await;
                            streams.abort_all();
                            return Err(e);
                        }
                        // A stream can break because the sender stopped; its
                        // cancel may still be on the way on the control stream.
                        Err(e) => return Err(cancel_or(e, &mut ctl_rx, conn).await),
                    };
                    let ok = if let Some(placed_path) = placed_opt {
                        if let Some(&slot) = top_file_slots.get(&index) {
                            let name = placed_path
                                .file_name()
                                .ok_or(Error::Protocol("bad file name"))?;
                            if let Some(out_slot) = top_placed.get_mut(slot) {
                                *out_slot = Some(dest.join(name));
                            }
                        }
                        report.files_ok = report.files_ok.saturating_add(1);
                        true
                    } else {
                        report.files_failed = report.files_failed.saturating_add(1);
                        false
                    };
                    send_control(&mut sealer, &mut send,
                        &Control::EntryDone { transfer_id, index, ok }).await?;
                    pending = pending.saturating_sub(1);
                }
                stream = conn.accept_uni() => {
                    let stream = match stream {
                        Ok(s) => s,
                        Err(_) if stop.is_stopped() => {
                            gate.signal_halt();
                            stopped_notified = true;
                            let e = tell_stopped(conn, &mut sealer, &mut send, transfer_id).await;
                            streams.abort_all();
                            return Err(e);
                        }
                        Err(e) => return Err(e.into()),
                    };
                    let tx = result_tx.clone();
                    let (targets, claimed, ck, done, progress, permits, stop_stream, gate) = (
                        targets.clone(), claimed.clone(), offer.content_key.clone(),
                        done.clone(), progress.clone(), permits.clone(), stop.clone(),
                        Arc::clone(&gate),
                    );
                    // The permit is waited for inside the task, so this loop
                    // keeps hearing stops, cancels and the inactivity timer.
                    streams.spawn(async move {
                        let r = match permits.acquire_owned().await {
                            Ok(_permit) => {
                                receive_stream(stream, targets, claimed, ck, transfer_id, done, total, progress, stop_stream, gate).await
                            }
                            Err(_) => Err(Error::Protocol("semaphore closed")),
                        };
                        let _ = tx.send(r).await;
                    });
                    // Reap finished tasks so the set does not grow.
                    while streams.try_join_next().is_some() {}
                }
                _ = report_tick.tick(), if offer.reports => {
                    let bytes = done_at_check.load(Ordering::Relaxed);
                    if bytes != told {
                        send_control(&mut sealer, &mut send,
                            &Control::Received { transfer_id, bytes }).await?;
                        told = bytes;
                    }
                }
                () = tokio::time::sleep_until(deadline) => {
                    if done_at_check.load(Ordering::Relaxed) == seen_bytes {
                        return Err(Error::Connect(crate::error::msg::SENDER_STOPPED_SENDING.into()));
                    }
                }
            }
        }
        Ok(())
    }
    .await;
    if let Err(mut e) = outcome {
        // Halt placement, tell the sender if a local stop raced with another
        // error branch, close the connection immediately on remote cancel so
        // a stopped sender enters passive draining right away (#651), wait for
        // every async stream task (whose `PartFile` drops schedule tracked
        // unlinks) and every spawned blocking placement or unlink task to
        // finish, then remove empty created directories.
        gate.signal_halt();
        if !stopped_notified && (matches!(e, Error::Stopped) || stop.is_stopped()) {
            e = tell_stopped(conn, &mut sealer, &mut send, transfer_id).await;
        }
        if !matches!(e, Error::Stopped) && !peer_closed(conn) {
            conn.close(0u32.into(), b"done");
        }
        streams.shutdown().await;
        gate.shutdown().await;
        drop_created_blocking(created).await;
        return Err(e);
    }
    gate.shutdown().await;
    created.keep = true;
    report.placed = top_placed.into_iter().flatten().collect();
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

    /// #651: the control stream is given higher QUIC priority than bulk file
    /// streams so `Control::Cancel` preempts queued file data on busy paths.
    #[tokio::test]
    async fn control_stream_has_higher_priority_than_bulk_data_streams() {
        let (sender, receiver) =
            crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
        let (schan, rchan, _skeys, _scert) = secure_pair(&sender, &receiver).await;
        let bulk = sender.open_uni().await.unwrap();
        let bulk_prio = bulk.priority().unwrap();
        assert!(
            schan.send.priority().unwrap() > bulk_prio,
            "sender control stream priority should exceed bulk stream priority"
        );
        assert!(
            rchan.send.priority().unwrap() > bulk_prio,
            "receiver control stream priority should exceed bulk stream priority"
        );
    }

    /// #651: `accept` closes the QUIC connection as soon as it receives
    /// `Control::Cancel` (without waiting for the caller's outer close), so a
    /// stopped sender observes `peer_closed` and skips passive draining.
    #[tokio::test]
    async fn accept_closes_connection_immediately_on_sender_cancel() {
        let (sender, receiver) = paced_pair().await;
        let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
        let base = std::env::temp_dir().join(format!(
            "td-cancel-close-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let (src, dest) = (base.join("src"), base.join("dest"));
        std::fs::create_dir_all(src.join("f")).unwrap();
        std::fs::create_dir_all(&dest).unwrap();
        for n in 0..2 {
            std::fs::write(src.join("f").join(format!("{n}.bin")), vec![3u8; 4 << 20]).unwrap();
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
                if n >= 512 * 1024 {
                    started.notify_one();
                }
            })
        };
        let recv = {
            let receiver = receiver.clone();
            let dest = dest.clone();
            tokio::spawn(async move {
                accept(&receiver, rchan, offer, &dest, on_receive, &Stop::new()).await
            })
        };
        tokio::time::timeout(Duration::from_secs(10), started.notified())
            .await
            .unwrap();
        stop.stop();
        let s = tokio::time::timeout(Duration::from_secs(5), send)
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(s, Err(Error::Stopped)), "{s:?}");
        let r = tokio::time::timeout(Duration::from_secs(5), recv)
            .await
            .unwrap()
            .unwrap();
        assert!(
            matches!(r, Err(Error::Cancelled(ref reason)) if reason == "stopped"),
            "{r:?}"
        );
        assert!(
            matches!(
                receiver.close_reason(),
                Some(quinn::ConnectionError::LocallyClosed)
            ),
            "accept should close the connection locally on cancel: {:?}",
            receiver.close_reason()
        );
        assert!(
            peer_closed(&sender),
            "sender should see peer's close to skip passive draining: {:?}",
            sender.close_reason()
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    /// #92: retargeting a destination symlink mid-transfer must never split
    /// files across two folders or escape the originally resolved destination,
    /// while `ReceiveReport::placed` keeps the user's spelling (`dest_link`).
    #[cfg(unix)]
    #[tokio::test]
    async fn retargeting_destination_symlink_mid_transfer_keeps_files_in_original_folder() {
        let base = temp_test_dir("symlink-retarget");
        let dir_a = base.join("dir_a");
        let dir_b = base.join("dir_b");
        std::fs::create_dir_all(&dir_a).unwrap();
        std::fs::create_dir_all(&dir_b).unwrap();
        std::fs::write(dir_a.join("existing.txt"), b"old-a").unwrap();
        std::fs::write(dir_b.join("untouched.txt"), b"old-b").unwrap();

        let dest_link = base.join("dest_link");
        std::os::unix::fs::symlink(&dir_a, &dest_link).unwrap();
        assert_eq!(check_destination(&dest_link).unwrap(), dest_link);

        let (sender, receiver) =
            crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
        let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
        let entries = vec![
            bytes_entry(
                &["existing.txt"],
                b"new-top",
                *blake3::hash(b"new-top").as_bytes(),
            ),
            bytes_entry(
                &["sub_one", "first.bin"],
                b"first-payload",
                *blake3::hash(b"first-payload").as_bytes(),
            ),
            bytes_entry(
                &["sub_two", "second.bin"],
                b"second-payload",
                *blake3::hash(b"second-payload").as_bytes(),
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
        let retarget_link = dest_link.clone();
        let retarget_to = dir_b.clone();
        let mut retargeted = false;
        let report = accept_with_hooks(
            &receiver,
            rchan,
            offer,
            &dest_link,
            Arc::new(|_, _| {}),
            &Stop::new(),
            |p| fs4::available_space(p),
            probe_destination_writable,
            move |_dir| {
                if !retargeted {
                    retargeted = true;
                    std::fs::remove_file(&retarget_link).unwrap();
                    std::os::unix::fs::symlink(&retarget_to, &retarget_link).unwrap();
                }
            },
        )
        .await
        .unwrap();

        let sent = send.await.unwrap().unwrap();
        assert_eq!((sent.files_ok, report.files_ok), (3, 3));
        assert_eq!(
            report.placed,
            vec![
                dest_link.join("existing (2).txt"),
                dest_link.join("sub_one"),
                dest_link.join("sub_two"),
            ]
        );
        assert_eq!(std::fs::read(dir_a.join("existing.txt")).unwrap(), b"old-a");
        assert_eq!(
            std::fs::read(dir_a.join("existing (2).txt")).unwrap(),
            b"new-top"
        );
        assert_eq!(
            std::fs::read(dir_a.join("sub_one/first.bin")).unwrap(),
            b"first-payload"
        );
        assert_eq!(
            std::fs::read(dir_a.join("sub_two/second.bin")).unwrap(),
            b"second-payload"
        );
        assert_eq!(names(&dir_b), ["untouched.txt"]);
        assert_eq!(
            std::fs::read(dir_b.join("untouched.txt")).unwrap(),
            b"old-b"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    /// #92: once `stop.stop()` fires right before placement, zero files may be
    /// placed at `final_path`.
    #[tokio::test]
    async fn no_file_placement_runs_after_stop() {
        let base = temp_test_dir("no-place-after-stop");
        let dest = base.join("dest");
        std::fs::create_dir_all(&dest).unwrap();

        let (sender, receiver) =
            crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
        let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
        let payload = b"complete-chunk-before-stop";
        let entries = vec![bytes_entry(
            &["sub", "item.bin"],
            payload,
            *blake3::hash(payload).as_bytes(),
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
        let progress_stop = stop.clone();
        // Trigger `stop.stop()` the instant the final byte has been written to
        // `.part`, right before sync and placement run.
        let on_receive: Progress = Arc::new(move |done, total| {
            if done >= total {
                progress_stop.stop();
            }
        });
        let r = accept(&receiver, rchan, offer, &dest, on_receive, &stop).await;
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
            "expected sender Cancelled(\"stopped\"), got {s:?}"
        );
        assert!(
            names(&dest).is_empty(),
            "no file or created directory may remain after stop: {:?}",
            names(&dest)
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    /// #92: `probe_destination_writable` propagates unlink errors other than
    /// `NotFound` so mounts that allow file creation without unlink fail
    /// `check_destination` immediately instead of silently leaving probe and
    /// `.part` files behind.
    #[test]
    fn destination_probe_propagates_unlink_failure() {
        let base = temp_test_dir("probe-unlink");
        let created_probe = std::sync::Mutex::new(None::<PathBuf>);
        let err = probe_destination_with_unlink(&base, |p| {
            assert!(
                p.exists(),
                "probe file must exist and be closed when unlinked"
            );
            *created_probe.lock().unwrap() = Some(p.to_path_buf());
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "ACL denies delete_child",
            ))
        })
        .unwrap_err();
        if let Some(p) = created_probe.lock().unwrap().take() {
            let _ = std::fs::remove_file(p);
        }
        assert!(
            matches!(err, Error::Io(ref e) if e.kind() == std::io::ErrorKind::PermissionDenied),
            "expected PermissionDenied, got {err:?}"
        );
        assert_eq!(err.cause(), crate::error::Cause::NoPermission);

        // `NotFound` during unlink (e.g. removed concurrently) is harmless.
        probe_destination_with_unlink(&base, |p| {
            std::fs::remove_file(p).unwrap();
            Err(std::io::Error::from(std::io::ErrorKind::NotFound))
        })
        .unwrap();

        let _ = std::fs::remove_dir_all(&base);
    }

    /// #92: stopping right after the secure handshake finishes (before offer /
    /// accept) returns `Error::Stopped` on the stopping side and
    /// `Error::Cancelled("stopped")` on the peer without leaking an offer or
    /// raising a protocol error.
    #[tokio::test]
    async fn stopping_right_after_handshake_finishes() {
        // Case 1: sender stops right after handshake finishes.
        {
            let (sender, receiver) =
                crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
            let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
            let stop = Stop::new();
            stop.stop();
            let entries = vec![bytes_entry(
                &["secret.txt"],
                b"hello",
                *blake3::hash(b"hello").as_bytes(),
            )];
            let (sent, offered) = tokio::join!(
                send_files(
                    &sender,
                    schan,
                    &skeys,
                    entries,
                    false,
                    Arc::new(|_, _| {}),
                    &stop,
                ),
                receive_offer(&mut rchan, &scert),
            );
            assert!(
                matches!(sent, Err(Error::Stopped)),
                "expected sender Stopped, got {sent:?}"
            );
            assert!(
                matches!(offered, Err(Error::Cancelled(ref r)) if r == "stopped"),
                "expected receiver Cancelled(\"stopped\"), got {offered:?}"
            );
        }

        // Case 2: receiver stops right after receiving the offer (before accept/decline).
        {
            let base = temp_test_dir("stop-after-handshake-recv");
            let dest = base.join("dest");
            std::fs::create_dir_all(&dest).unwrap();
            let (sender, receiver) =
                crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
            let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
            let entries = vec![bytes_entry(
                &["secret.txt"],
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
            stop.stop();
            let recv_res = accept(&receiver, rchan, offer, &dest, Arc::new(|_, _| {}), &stop).await;
            assert!(
                matches!(recv_res, Err(Error::Stopped)),
                "expected receiver Stopped, got {recv_res:?}"
            );
            let sent = tokio::time::timeout(Duration::from_secs(5), send)
                .await
                .unwrap()
                .unwrap();
            assert!(
                matches!(sent, Err(Error::Cancelled(ref r)) if r == "stopped"),
                "expected sender Cancelled(\"stopped\"), got {sent:?}"
            );
            assert!(names(&dest).is_empty());
            let _ = std::fs::remove_dir_all(&base);
        }
    }

    /// #92: if one file's placement has already begun when `stop.stop()` fires,
    /// `accept` notifies the sender without waiting on that placement, waits
    /// for the in-flight placement task to finish before returning, and never
    /// starts placing any subsequent file.
    #[tokio::test]
    async fn accept_waits_for_in_flight_placement_and_blocks_subsequent_placements() {
        let base = temp_test_dir("inflight-place-stop");
        let dest = base.join("dest");
        std::fs::create_dir_all(&dest).unwrap();

        let (sender, receiver) =
            crate::transport::testing::over_slow_relay(Duration::from_millis(10)).await;
        let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
        // `first.bin` is a single 3-byte frame and reaches `hook` first while
        // `second.bin` (two chunks over the 10 ms link) is still arriving; once
        // `first.bin` enters `hook`, `stop.stop()` fires so `second.bin` must
        // neither be placed nor leave any `.part` or empty directory behind.
        let second_payload = vec![0x7a_u8; 1_100_000];
        let entries = vec![
            bytes_entry(&["first.bin"], b"one", *blake3::hash(b"one").as_bytes()),
            bytes_entry(
                &["nested", "second.bin"],
                &second_payload,
                *blake3::hash(&second_payload).as_bytes(),
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
        let entered_tx = std::sync::Mutex::new(Some(entered_tx));
        let resume_rx = std::sync::Mutex::new(Some(resume_rx));
        let place_calls = Arc::new(AtomicU64::new(0));
        let place_calls_hook = Arc::clone(&place_calls);

        let hook: PlacementHook = Arc::new(move || {
            let call = place_calls_hook.fetch_add(1, Ordering::SeqCst);
            if call == 0 {
                if let Some(tx) = entered_tx.lock().unwrap().take() {
                    let _ = tx.send(());
                }
                let rx = resume_rx.lock().unwrap().take().unwrap();
                assert!(
                    rx.recv_timeout(Duration::from_secs(10)).is_ok(),
                    "in-flight placement was never resumed"
                );
            }
            Ok(())
        });

        let recv_task = {
            let receiver = receiver.clone();
            let dest_clone = dest.clone();
            let stop_clone = stop.clone();
            tokio::spawn(async move {
                accept_with_all_hooks(
                    &receiver,
                    rchan,
                    offer,
                    &dest_clone,
                    Arc::new(|_, _| {}),
                    &stop_clone,
                    |p| fs4::available_space(p),
                    probe_destination_writable,
                    |_| {},
                    Some(hook),
                )
                .await
            })
        };

        entered_rx.await.unwrap();
        stop.stop();

        // Sender hears `Cancelled("stopped")` while the first placement is
        // still blocked inside `hook`.
        let sent = tokio::time::timeout(Duration::from_secs(5), send)
            .await
            .unwrap()
            .unwrap();
        assert!(
            matches!(sent, Err(Error::Cancelled(ref r)) if r == "stopped"),
            "expected sender Cancelled(\"stopped\"), got {sent:?}"
        );
        // `accept` must NOT return while the in-flight placement task is still
        // running.
        tokio::task::yield_now().await;
        assert!(
            !recv_task.is_finished(),
            "accept returned while an in-flight placement task was still running"
        );

        resume_tx.send(()).unwrap();
        let recv_res = tokio::time::timeout(Duration::from_secs(5), recv_task)
            .await
            .unwrap()
            .unwrap();
        assert!(
            matches!(recv_res, Err(Error::Stopped)),
            "expected Stopped, got {recv_res:?}"
        );
        assert_eq!(
            place_calls.load(Ordering::SeqCst),
            1,
            "no second placement may run once stop has fired"
        );
        // `first.bin` finished its already-started placement, while `nested/`
        // had its `.part` file unlinked on the tracked task BEFORE `CreatedDirs`
        // removed `nested/`.
        assert_eq!(names(&dest), ["first.bin"]);

        let _ = std::fs::remove_dir_all(&base);
    }

    /// #92: when a stream task observes `stop` first and sends
    /// `Err(Error::Stopped)` on `result_tx` before `stop.stopped()` wins
    /// `select!` (as well as when `stop.stop()` races across concurrent streams,
    /// and when parallel streams place files concurrently without serializing on
    /// a `Mutex`), `accept` always runs `tell_stopped` (notifying the sender
    /// with `Error::Cancelled("stopped")`) and removes all `.part` files and
    /// empty created folders.
    #[tokio::test]
    async fn stream_stopped_result_always_notifies_sender_and_cleans_dirs() {
        let base = temp_test_dir("stream-stopped-notify");

        // 1. Deterministic `Some(r) = result_rx.recv() => Err(Error::Stopped)`
        //    branch: `stop.stopped()` remains `Pending` when the stream sends
        //    `Err(Error::Stopped)` onto `result_tx`, reproducing the exact
        //    window where a stream task observes `stop` first. Without the
        //    `Err(Error::Stopped)` guard on `result_rx`, `accept` would fall
        //    into `cancel_or` (skipping `tell_stopped` so the sender hangs).
        {
            let dest = base.join("deterministic-result-rx");
            std::fs::create_dir_all(&dest).unwrap();

            let (sender, receiver) =
                crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
            let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
            let second_payload = vec![0x33_u8; 512 * 1024];
            let entries = vec![
                bytes_entry(
                    &["a", "one.bin"],
                    b"first",
                    *blake3::hash(b"first").as_bytes(),
                ),
                bytes_entry(
                    &["b", "two.bin"],
                    &second_payload,
                    *blake3::hash(&second_payload).as_bytes(),
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
            let hook: PlacementHook = Arc::new(|| Err(Error::Stopped));
            let r = accept_with_all_hooks(
                &receiver,
                rchan,
                offer,
                &dest,
                Arc::new(|_, _| {}),
                &stop,
                |p| fs4::available_space(p),
                probe_destination_writable,
                |_| {},
                Some(hook),
            )
            .await;
            assert!(
                matches!(r, Err(Error::Stopped)),
                "expected receiver Stopped on result_rx branch, got {r:?}"
            );
            let s = tokio::time::timeout(Duration::from_secs(5), send)
                .await
                .unwrap()
                .unwrap();
            assert!(
                matches!(s, Err(Error::Cancelled(ref reason)) if reason == "stopped"),
                "expected sender Cancelled(\"stopped\") on result_rx branch, got {s:?}"
            );
            assert!(
                names(&dest).is_empty(),
                "both `a/` and `b/` and their `.part` files must be removed, got {:?}",
                names(&dest)
            );
        }

        // 2. End-to-end `Stop::stop()` race across concurrent streams.
        for round in 0..5_u32 {
            let dest = base.join(format!("race-{round}"));
            std::fs::create_dir_all(&dest).unwrap();

            let (sender, receiver) =
                crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
            let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
            let entries = vec![
                bytes_entry(&["a", "one.bin"], b"", *blake3::hash(b"").as_bytes()),
                bytes_entry(&["b", "two.bin"], b"", *blake3::hash(b"").as_bytes()),
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
            let stop_hook = stop.clone();
            let calls = Arc::new(AtomicU64::new(0));
            let calls_hook = Arc::clone(&calls);
            let hook: PlacementHook = Arc::new(move || {
                if calls_hook.fetch_add(1, Ordering::SeqCst) == 0 {
                    stop_hook.stop();
                }
                Ok(())
            });
            let r = accept_with_all_hooks(
                &receiver,
                rchan,
                offer,
                &dest,
                Arc::new(|_, _| {}),
                &stop,
                |p| fs4::available_space(p),
                probe_destination_writable,
                |_| {},
                Some(hook),
            )
            .await;
            assert!(
                matches!(r, Err(Error::Stopped)),
                "round {round}: expected receiver Stopped, got {r:?}"
            );
            let s = tokio::time::timeout(Duration::from_secs(5), send)
                .await
                .unwrap()
                .unwrap();
            assert!(
                matches!(s, Err(Error::Cancelled(ref reason)) if reason == "stopped"),
                "round {round}: expected sender Cancelled(\"stopped\"), got {s:?}"
            );
            for entry_name in names(&dest) {
                let sub = dest.join(&entry_name);
                assert!(
                    !names(&sub).is_empty(),
                    "round {round}: empty directory {entry_name:?} leaked after stop"
                );
            }
        }

        // 3. Parallel placements run concurrently without serializing on a Mutex:
        //    stream 0 waits inside `hook` until stream 1 enters `hook` too.
        {
            let dest = base.join("parallel-placements");
            std::fs::create_dir_all(&dest).unwrap();

            let (sender, receiver) =
                crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
            let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
            let entries = vec![
                bytes_entry(&["one.bin"], b"alpha", *blake3::hash(b"alpha").as_bytes()),
                bytes_entry(&["two.bin"], b"bravo", *blake3::hash(b"bravo").as_bytes()),
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
            let (second_tx, second_rx) = std::sync::mpsc::channel::<()>();
            let second_tx = std::sync::Mutex::new(Some(second_tx));
            let second_rx = std::sync::Mutex::new(Some(second_rx));
            let entered = Arc::new(AtomicU64::new(0));
            let entered_hook = Arc::clone(&entered);
            let hook: PlacementHook = Arc::new(move || {
                let n = entered_hook.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    let rx = second_rx.lock().unwrap().take().unwrap();
                    assert!(
                        rx.recv_timeout(Duration::from_secs(5)).is_ok(),
                        "placements were serialized instead of running concurrently"
                    );
                } else if n == 1
                    && let Some(tx) = second_tx.lock().unwrap().take()
                {
                    let _ = tx.send(());
                }
                Ok(())
            });
            let report = accept_with_all_hooks(
                &receiver,
                rchan,
                offer,
                &dest,
                Arc::new(|_, _| {}),
                &Stop::new(),
                |p| fs4::available_space(p),
                probe_destination_writable,
                |_| {},
                Some(hook),
            )
            .await
            .unwrap();
            let sent = send.await.unwrap().unwrap();
            assert_eq!((sent.files_ok, report.files_ok), (2, 2));
            assert_eq!(names(&dest), ["one.bin", "two.bin"]);
        }

        let _ = std::fs::remove_dir_all(&base);
    }

    /// #92: when the receiver never answers an offer, `send_files` keeps the
    /// QUIC session alive via keep-alives and times out right at
    /// `DECISION_TIMEOUT` (10 minutes) with `RECEIVER_DID_NOT_ANSWER`, verified
    /// deterministically in milliseconds via `tokio::time::pause`.
    #[tokio::test]
    async fn decision_timeout_fires_after_ten_minutes_with_paused_clock() {
        tokio::time::pause();
        let wall = std::time::Instant::now();

        let (sender, receiver) =
            crate::transport::testing::over_slow_relay(Duration::from_millis(1)).await;
        let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
        let entries = vec![bytes_entry(
            &["doc.txt"],
            b"waiting",
            *blake3::hash(b"waiting").as_bytes(),
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

        let _offer = receive_offer(&mut rchan, &scert).await.unwrap();
        tokio::task::yield_now().await;

        // Step virtual time in 1-second increments up to 9m59s so 2-second QUIC
        // keep-alives traverse the simulated 1 ms link without tripping the
        // 30-second QUIC idle timeout.
        for _ in 0..599 {
            tokio::time::advance(Duration::from_secs(1)).await;
        }
        tokio::task::yield_now().await;
        assert!(
            !send.is_finished(),
            "decision timeout must not fire before 10 minutes"
        );

        for _ in 0..2 {
            tokio::time::advance(Duration::from_secs(1)).await;
        }
        let res = send.await.unwrap();
        assert!(
            matches!(res, Err(Error::Connect(ref m)) if m == crate::error::msg::RECEIVER_DID_NOT_ANSWER),
            "expected RECEIVER_DID_NOT_ANSWER, got {res:?}"
        );
        assert!(
            wall.elapsed() < Duration::from_secs(5),
            "paused-clock test took {:?}",
            wall.elapsed()
        );
    }

    /// #92: when `conn.close(0, b"stopped")` overtakes `Control::Cancel` so no
    /// `Control::Cancel` frame ever reaches the peer (meaning `cancel_or`
    /// cannot rescue it from a control message), both `send_files` and
    /// `accept` / `accept_text` still report `Error::Cancelled("stopped")`.
    #[tokio::test]
    async fn close_overtaking_cancel_reports_cancelled_without_cancel_or() {
        // 1. Receiver queues Cancel and immediately closes without waiting for
        //    `send.stopped()`, and also closes mid-transfer without sending
        //    Cancel at all; `send_files` reports `Cancelled("stopped")`.
        for send_cancel_frame in [true, false] {
            let (sender, receiver) =
                crate::transport::testing::over_slow_relay(Duration::from_millis(20)).await;
            let (schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
            let payload = vec![0x42_u8; 64 * 1024];
            let entries = vec![bytes_entry(
                &["a.bin"],
                &payload,
                *blake3::hash(&payload).as_bytes(),
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
            let tid = offer.transfer_id();
            if send_cancel_frame {
                // Queue Cancel and immediately close: Quinn drops unsent stream
                // data on `conn.close`, so `CONNECTION_CLOSE(b"stopped")`
                // overtakes the `Control::Cancel` frame.
                let _ = send_control(
                    &mut rchan.sealer,
                    &mut rchan.send,
                    &Control::Cancel {
                        transfer_id: tid,
                        reason: "stopped".into(),
                    },
                )
                .await;
                receiver.close(0u32.into(), b"stopped");
            } else {
                // Accept so `send_files` enters the data-stream loop, then
                // close with `b"stopped"` and no `Control::Cancel` at all.
                send_control(
                    &mut rchan.sealer,
                    &mut rchan.send,
                    &Control::Accept {
                        transfer_id: tid,
                        wanted: vec![(0, Vec::new())],
                        progress: false,
                    },
                )
                .await
                .unwrap();
                let mut uni = receiver.accept_uni().await.unwrap();
                expect_frame(&mut uni, 1024).await.unwrap();
                receiver.close(0u32.into(), b"stopped");
            }
            let sent = tokio::time::timeout(Duration::from_secs(5), send)
                .await
                .unwrap()
                .unwrap();
            assert!(
                matches!(sent, Err(Error::Cancelled(ref r)) if r == "stopped"),
                "send_cancel_frame={send_cancel_frame}: expected Cancelled(\"stopped\"), got {sent:?}"
            );
        }

        // 2. Sender closes with `b"stopped"` while receiver is in `accept`
        //    (mid-stream, with no `Control::Cancel` sent at all, so `cancel_or`
        //    sees no `Control::Cancel` message) and in `accept_text`.
        {
            let base = temp_test_dir("close-overtakes-cancel");
            let dest = base.join("dest");
            std::fs::create_dir_all(&dest).unwrap();

            let (sender, receiver) =
                crate::transport::testing::over_slow_relay(Duration::from_millis(5)).await;
            let (mut schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
            let payload = vec![0x24_u8; 64 * 1024];
            let entry = Entry {
                path: vec!["half.bin".into()],
                kind: EntryKind::File,
                size: payload.len() as u64,
                hash: Some(*blake3::hash(&payload).as_bytes()),
                mtime: None,
            };
            let tid = [9u8; 16];
            let ck = ContentKey::generate().unwrap();
            let manifest = Manifest {
                transfer_id: tid,
                entries: vec![entry],
                total_size: payload.len() as u64,
                chunk_size: limits::CHUNK_SIZE,
            };
            let signature = skeys.sign(&manifest.signed_bytes());
            send_control(
                &mut schan.sealer,
                &mut schan.send,
                &Control::Offer {
                    manifest,
                    content_key: *ck.as_bytes(),
                    signature,
                    progress: false,
                    text: false,
                },
            )
            .await
            .unwrap();

            let offer = receive_offer(&mut rchan, &scert).await.unwrap();
            let peer_task = {
                let sender = sender.clone();
                tokio::spawn(async move {
                    let msg = recv_control(&mut schan.opener, &mut schan.recv)
                        .await
                        .unwrap();
                    assert!(matches!(msg, Control::Accept { .. }));
                    // Open a data stream and write only the header so the
                    // receiver is mid-stream, then close with `b"stopped"` and
                    // NO `Control::Cancel` message at all.
                    let mut uni = sender.open_uni().await.unwrap();
                    let header = DataHeader {
                        transfer_id: tid,
                        entry: 0,
                        first_chunk: 0,
                    };
                    write_frame(&mut uni, &header.encode()).await.unwrap();
                    sender.close(0u32.into(), b"stopped");
                })
            };

            let recv_res = accept(
                &receiver,
                rchan,
                offer,
                &dest,
                Arc::new(|_, _| {}),
                &Stop::new(),
            )
            .await;
            peer_task.await.unwrap();
            assert!(
                matches!(recv_res, Err(Error::Cancelled(ref r)) if r == "stopped"),
                "expected receiver Cancelled(\"stopped\"), got {recv_res:?}"
            );
            assert!(names(&dest).is_empty());

            let _ = std::fs::remove_dir_all(&base);
        }

        // 3. `accept_text` when sender's `b"stopped"` close overtakes Cancel.
        {
            let (sender, receiver) =
                crate::transport::testing::over_slow_relay(Duration::from_millis(5)).await;
            let (mut schan, mut rchan, skeys, scert) = secure_pair(&sender, &receiver).await;
            let words = b"short text";
            let tid = [11u8; 16];
            let ck = ContentKey::generate().unwrap();
            let manifest = Manifest {
                transfer_id: tid,
                entries: vec![Entry {
                    path: vec!["text.txt".into()],
                    kind: EntryKind::File,
                    size: words.len() as u64,
                    hash: Some(*blake3::hash(words).as_bytes()),
                    mtime: None,
                }],
                total_size: words.len() as u64,
                chunk_size: limits::CHUNK_SIZE,
            };
            let signature = skeys.sign(&manifest.signed_bytes());
            send_control(
                &mut schan.sealer,
                &mut schan.send,
                &Control::Offer {
                    manifest,
                    content_key: *ck.as_bytes(),
                    signature,
                    progress: false,
                    text: true,
                },
            )
            .await
            .unwrap();

            let offer = receive_offer(&mut rchan, &scert).await.unwrap();
            let peer_task = {
                let sender = sender.clone();
                tokio::spawn(async move {
                    let _ = recv_control(&mut schan.opener, &mut schan.recv)
                        .await
                        .unwrap();
                    sender.close(0u32.into(), b"stopped");
                })
            };
            let recv_res = accept_text(&receiver, rchan, offer, &Stop::new()).await;
            peer_task.await.unwrap();
            assert!(
                matches!(recv_res, Err(Error::Cancelled(ref r)) if r == "stopped"),
                "expected accept_text Cancelled(\"stopped\"), got {recv_res:?}"
            );
        }
    }
}
