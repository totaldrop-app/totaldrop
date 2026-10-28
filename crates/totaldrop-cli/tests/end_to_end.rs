//! End-to-end tests: a real rendezvous server and two real clients on this
//! machine, talking over WebSocket and QUIC exactly as they would over the
//! internet.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use totaldrop_core::rendezvous::{self, Mailbox};
use totaldrop_core::{
    Error, IncomingOffer, PathOptions, PreparedSend, ReceiveOptions, SendOptions, no_events,
};
use totaldrop_proto::wire::Mood;

/// The server under test: the Rust server in-process, or another
/// implementation (for example the Cloudflare Worker under `wrangler dev`)
/// when `TOTALDROP_TEST_SERVER` names one.
async fn server() -> String {
    if let Ok(url) = std::env::var("TOTALDROP_TEST_SERVER") {
        return url;
    }
    let (addr, _server, _task) =
        totaldrop_rendezvous::spawn("127.0.0.1:0".parse().unwrap(), Default::default())
            .await
            .unwrap();
    format!("ws://{addr}")
}

/// Most tests run without a relay, on loopback only.
fn no_relay() -> PathOptions {
    PathOptions {
        relay: String::new(),
        stun: String::new(),
        relay_only: false,
    }
}

/// The relay under test: the Rust relay in-process (which also answers
/// STUN), or another implementation (for example the Cloudflare Worker under
/// `wrangler dev`) when `TOTALDROP_TEST_RELAY` names one. The relay is
/// returned when it is the in-process one, to read its counter.
async fn relay() -> (String, String, Option<totaldrop_relay::Relay>) {
    if let Ok(url) = std::env::var("TOTALDROP_TEST_RELAY") {
        return (url, String::new(), None);
    }
    let (url, stun, relay, _task) = totaldrop_relay::spawn_local(Default::default())
        .await
        .unwrap();
    (url, stun, Some(relay))
}

fn scratch(name: &str) -> PathBuf {
    let mut rnd = [0u8; 6];
    getrandom::fill(&mut rnd).unwrap();
    let dir = std::env::temp_dir().join(format!(
        "totaldrop-e2e-{name}-{}",
        rnd.iter().map(|b| format!("{b:02x}")).collect::<String>()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn pseudo_random(len: usize, seed: u8) -> Vec<u8> {
    let mut x = u32::from(seed).wrapping_mul(2_654_435_761).wrapping_add(1);
    (0..len)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x as u8
        })
        .collect()
}

fn same_tree(a: &Path, b: &Path) {
    let mut left: Vec<_> = walk(a)
        .into_iter()
        .map(|p| p.strip_prefix(a).unwrap().to_path_buf())
        .collect();
    let mut right: Vec<_> = walk(b)
        .into_iter()
        .map(|p| p.strip_prefix(b).unwrap().to_path_buf())
        .collect();
    left.sort();
    right.sort();
    assert_eq!(left, right, "different file lists");
    for rel in left {
        let (x, y) = (a.join(&rel), b.join(&rel));
        if x.is_file() {
            assert_eq!(
                std::fs::read(&x).unwrap(),
                std::fs::read(&y).unwrap(),
                "{rel:?} differs"
            );
        }
    }
}

fn walk(p: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(p).unwrap() {
        let path = e.unwrap().path();
        out.push(path.clone());
        if path.is_dir() {
            out.extend(walk(&path));
        }
    }
    out
}

async fn transfer(
    server: &str,
    paths: Vec<PathBuf>,
    dest: &Path,
) -> (totaldrop_core::SendReport, totaldrop_core::ReceiveReport) {
    let prepared = PreparedSend::new(
        &SendOptions {
            server: server.to_owned(),
            words: 2,
            paths: no_relay(),
        },
        paths,
    )
    .await
    .unwrap();
    let code = prepared.code().to_string();
    let sender = tokio::spawn(prepared.run(no_events()));
    let offer = IncomingOffer::connect(
        &ReceiveOptions {
            server: server.to_owned(),
            paths: no_relay(),
        },
        &code,
    )
    .await
    .unwrap();
    let received = offer.accept(dest, no_events()).await.unwrap();
    let sent = sender.await.unwrap().unwrap();
    (sent, received)
}

#[tokio::test(flavor = "multi_thread")]
async fn folder_with_large_empty_and_nested_files_arrives_intact() {
    let server = server().await;
    let src = scratch("src");
    let dest = scratch("dest");
    let album = src.join("album");
    std::fs::create_dir_all(album.join("nested/deeper")).unwrap();
    // 2.5 MiB: three chunks, the last one partial.
    std::fs::write(album.join("big.bin"), pseudo_random(5 * 512 * 1024, 1)).unwrap();
    // Exactly one chunk.
    std::fs::write(album.join("one-chunk.bin"), pseudo_random(1 << 20, 2)).unwrap();
    std::fs::write(album.join("empty.txt"), b"").unwrap();
    std::fs::write(album.join("nested/deeper/şarkı ü.txt"), "merhaba dünya").unwrap();
    std::fs::create_dir_all(album.join("empty-folder")).unwrap();

    let (sent, received) = transfer(&server, vec![album.clone()], &dest).await;
    assert_eq!(sent.files_ok, 4);
    assert_eq!(sent.files_failed, 0);
    assert_eq!(received.files_ok, 4);
    same_tree(&src, &dest);

    // Modification times travel with the files.
    let a = std::fs::metadata(album.join("big.bin"))
        .unwrap()
        .modified()
        .unwrap();
    let b = std::fs::metadata(dest.join("album/big.bin"))
        .unwrap()
        .modified()
        .unwrap();
    let diff = a.duration_since(b).unwrap_or_else(|e| e.duration());
    assert!(diff < Duration::from_secs(1));

    // No temporary files are left behind.
    assert!(
        walk(&dest)
            .iter()
            .all(|p| !p.to_string_lossy().contains(".totaldrop-"))
    );
    std::fs::remove_dir_all(src).unwrap();
    std::fs::remove_dir_all(dest).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn existing_files_are_never_overwritten() {
    let server = server().await;
    let src = scratch("src");
    let dest = scratch("dest");
    std::fs::write(src.join("note.txt"), "new").unwrap();
    std::fs::write(dest.join("note.txt"), "old").unwrap();
    let (_, received) = transfer(&server, vec![src.join("note.txt")], &dest).await;
    assert_eq!(
        std::fs::read_to_string(dest.join("note.txt")).unwrap(),
        "old"
    );
    assert_eq!(
        std::fs::read_to_string(dest.join("note (2).txt")).unwrap(),
        "new"
    );
    assert_eq!(received.placed, vec![dest.join("note (2).txt")]);
    std::fs::remove_dir_all(src).unwrap();
    std::fs::remove_dir_all(dest).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn wrong_code_fails_on_both_sides_and_burns_the_nameplate() {
    let server = server().await;
    let src = scratch("src");
    std::fs::write(src.join("secret.txt"), "top secret").unwrap();
    let prepared = PreparedSend::new(
        &SendOptions {
            server: server.clone(),
            words: 2,
            paths: no_relay(),
        },
        vec![src.join("secret.txt")],
    )
    .await
    .unwrap();
    let real = prepared.code().clone();
    let sender = tokio::spawn(prepared.run(no_events()));

    // An attacker who knows the nameplate guesses the words.
    let mut guess = real.canonical();
    let last = guess.rsplit('-').next().unwrap().parse::<u16>().unwrap();
    let wrong = (last + 1) % 2048;
    guess = format!("{}-{wrong}", guess.rsplit_once('-').unwrap().0);
    let opts = ReceiveOptions {
        server: server.clone(),
        paths: no_relay(),
    };
    let attacker = IncomingOffer::connect(&opts, &guess).await;
    assert!(matches!(attacker, Err(Error::WrongCode)), "{attacker:?}");
    let sent = sender.await.unwrap();
    assert!(matches!(sent, Err(Error::WrongCode)), "{sent:?}");

    // The real receiver cannot get in any more: one guess per code.
    let late = IncomingOffer::connect(&opts, &real.to_string()).await;
    assert!(matches!(late, Err(Error::Rendezvous(_))), "{late:?}");
    std::fs::remove_dir_all(src).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn declined_offer_is_reported_to_the_sender() {
    let server = server().await;
    let src = scratch("src");
    std::fs::write(src.join("a.txt"), "a").unwrap();
    let prepared = PreparedSend::new(
        &SendOptions {
            server: server.clone(),
            words: 3,
            paths: no_relay(),
        },
        vec![src.join("a.txt")],
    )
    .await
    .unwrap();
    let code = prepared.code().to_string();
    assert_eq!(code.split('-').count(), 4, "three words plus the nameplate");
    let sender = tokio::spawn(prepared.run(no_events()));
    let offer = IncomingOffer::connect(
        &ReceiveOptions {
            server,
            paths: no_relay(),
        },
        &code,
    )
    .await
    .unwrap();
    assert_eq!(offer.summary().files, 1);
    assert_eq!(offer.words().len(), 4);
    offer.decline().await.unwrap();
    let sent = sender.await.unwrap();
    assert!(
        matches!(
            sent,
            Err(Error::Rejected(
                totaldrop_proto::wire::RejectReason::Declined
            ))
        ),
        "{sent:?}"
    );
    std::fs::remove_dir_all(src).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn server_refuses_a_third_claim() {
    let server = server().await;
    let alloc = rendezvous::allocate(&server).await.unwrap();
    let n = alloc.nameplate;
    let a = Mailbox::claim(&server, n, Some(&alloc)).await.unwrap();
    let b = Mailbox::claim(&server, n, None).await.unwrap();
    assert_eq!(a.id, b.id);
    let c = Mailbox::claim(&server, n, None).await;
    assert!(matches!(c, Err(Error::Rendezvous(r)) if r == "nameplate full"));
}

#[tokio::test(flavor = "multi_thread")]
async fn only_the_allocator_can_claim_first() {
    let server = server().await;
    let alloc = rendezvous::allocate(&server).await.unwrap();
    // Someone who only knows the number cannot take the first seat.
    assert!(matches!(
        Mailbox::claim(&server, alloc.nameplate, None).await,
        Err(Error::Rendezvous(_))
    ));
    // And nothing can be claimed that was never allocated.
    assert!(matches!(
        Mailbox::claim(&server, 4242, None).await,
        Err(Error::Rendezvous(_))
    ));
    Mailbox::claim(&server, alloc.nameplate, Some(&alloc))
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn scary_close_retires_the_nameplate() {
    let server = server().await;
    let alloc = rendezvous::allocate(&server).await.unwrap();
    let n = alloc.nameplate;
    let mut a = Mailbox::claim(&server, n, Some(&alloc)).await.unwrap();
    let mut b = Mailbox::claim(&server, n, None).await.unwrap();
    a.add("pake", vec![1; 32]).await.unwrap();
    b.add("pake", vec![2; 32]).await.unwrap();
    assert_eq!(a.expect("pake").await.unwrap(), vec![2; 32]);
    assert_eq!(b.expect("pake").await.unwrap(), vec![1; 32]);
    a.close(Mood::Scary).await;
    assert!(matches!(b.expect("confirm").await, Err(Error::WrongCode)));
    b.close(Mood::Lonely).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let next = rendezvous::allocate(&server).await.unwrap();
    assert_ne!(next.nameplate, n, "a retired nameplate must not be reused");
}

/// Both sides close `scary` after a wrong code; the attack-wave counter goes
/// up once, not twice.
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_code_counts_once() {
    let (addr, server, _task) =
        totaldrop_rendezvous::spawn("127.0.0.1:0".parse().unwrap(), Default::default())
            .await
            .unwrap();
    let url = format!("ws://{addr}");
    let alloc = rendezvous::allocate(&url).await.unwrap();
    let n = alloc.nameplate;
    let mut a = Mailbox::claim(&url, n, Some(&alloc)).await.unwrap();
    let mut b = Mailbox::claim(&url, n, None).await.unwrap();
    a.add("pake", vec![1; 32]).await.unwrap();
    b.add("pake", vec![2; 32]).await.unwrap();
    a.expect("pake").await.unwrap();
    b.expect("pake").await.unwrap();
    a.close(Mood::Scary).await;
    b.close(Mood::Scary).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(server.scary_count().await, 1);
}

/// A connection that never claims is closed at the claim deadline, pings or
/// not.
#[tokio::test(flavor = "multi_thread")]
async fn an_unclaimed_connection_is_closed() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;
    let config = totaldrop_rendezvous::Config {
        claim_timeout: Duration::from_millis(400),
        ..Default::default()
    };
    let (addr, _server, _task) =
        totaldrop_rendezvous::spawn("127.0.0.1:0".parse().unwrap(), config)
            .await
            .unwrap();
    let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/v0/mailbox/7"))
        .await
        .unwrap();
    let start = std::time::Instant::now();
    let mut closed = false;
    while start.elapsed() < Duration::from_secs(3) {
        if ws.send(Message::Ping(Vec::new().into())).await.is_err() {
            closed = true;
            break;
        }
        if let Ok(None | Some(Err(_)) | Some(Ok(Message::Close(_)))) =
            tokio::time::timeout(Duration::from_millis(100), ws.next()).await
        {
            closed = true;
            break;
        }
    }
    assert!(closed);
    assert!(start.elapsed() < Duration::from_secs(2));
}

#[tokio::test(flavor = "multi_thread")]
async fn expired_nameplates_cannot_be_claimed() {
    let config = totaldrop_rendezvous::Config {
        unclaimed_ttl: Duration::from_millis(300),
        sweep_interval: Duration::from_millis(100),
        ..Default::default()
    };
    let (addr, _server, _task) =
        totaldrop_rendezvous::spawn("127.0.0.1:0".parse().unwrap(), config)
            .await
            .unwrap();
    let url = format!("ws://{addr}");
    let stale = rendezvous::allocate(&url).await.unwrap();
    tokio::time::sleep(Duration::from_millis(800)).await;
    assert!(
        Mailbox::claim(&url, stale.nameplate, Some(&stale))
            .await
            .is_err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn plain_websocket_to_another_host_is_refused() {
    let r = rendezvous::allocate("ws://example.com").await;
    assert!(matches!(r, Err(Error::Rendezvous(_))));
}

async fn transfer_with(
    server: &str,
    send_paths: PathOptions,
    recv_paths: PathOptions,
    files: Vec<PathBuf>,
    dest: &Path,
) -> (totaldrop_core::SendReport, totaldrop_core::ReceiveReport) {
    let prepared = PreparedSend::new(
        &SendOptions {
            server: server.to_owned(),
            words: 2,
            paths: send_paths,
        },
        files,
    )
    .await
    .unwrap();
    let code = prepared.code().to_string();
    let sender = tokio::spawn(prepared.run(no_events()));
    let offer = IncomingOffer::connect(
        &ReceiveOptions {
            server: server.to_owned(),
            paths: recv_paths,
        },
        &code,
    )
    .await
    .unwrap();
    let received = offer.accept(dest, no_events()).await.unwrap();
    let sent = sender.await.unwrap().unwrap();
    (sent, received)
}

#[tokio::test(flavor = "multi_thread")]
async fn relay_only_transfer_goes_through_the_relay() {
    let server = server().await;
    let (relay_url, stun, relay) = relay().await;
    let src = scratch("src");
    let dest = scratch("dest");
    std::fs::write(src.join("film.bin"), pseudo_random(3 << 20, 7)).unwrap();
    let _ = stun;
    let only = PathOptions {
        relay: relay_url,
        stun: String::new(),
        relay_only: true,
    };
    // The receiver uses the sender's relay because it is also its own.
    let (sent, received) = transfer_with(
        &server,
        only.clone(),
        only,
        vec![src.join("film.bin")],
        &dest,
    )
    .await;
    assert_eq!((sent.files_ok, received.files_ok), (1, 1));
    same_tree(&src, &dest);
    if let Some(relay) = relay {
        assert!(
            relay.forwarded_bytes() > 3 << 20,
            "{}",
            relay.forwarded_bytes()
        );
    }
    std::fs::remove_dir_all(src).unwrap();
    std::fs::remove_dir_all(dest).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_direct_path_wins_over_the_relay() {
    let server = server().await;
    let (relay_url, stun, relay) = relay().await;
    let src = scratch("src");
    let dest = scratch("dest");
    std::fs::write(src.join("photo.bin"), pseudo_random(2 << 20, 8)).unwrap();
    let both = PathOptions {
        relay: relay_url,
        stun,
        relay_only: false,
    };
    let (sent, received) = transfer_with(
        &server,
        both.clone(),
        both,
        vec![src.join("photo.bin")],
        &dest,
    )
    .await;
    assert_eq!((sent.files_ok, received.files_ok), (1, 1));
    same_tree(&src, &dest);
    // At most a few handshake packets went through the relay.
    if let Some(relay) = relay {
        assert!(
            relay.forwarded_bytes() < 256 * 1024,
            "{}",
            relay.forwarded_bytes()
        );
    }
    std::fs::remove_dir_all(src).unwrap();
    std::fs::remove_dir_all(dest).unwrap();
}

/// A receiver in privacy mode never contacts a relay only the sender chose,
/// and says why at once (#68).
#[tokio::test(flavor = "multi_thread")]
async fn receiver_never_contacts_a_relay_only_the_sender_chose() {
    let server = server().await;
    let (senders_url, _, senders, _senders_task) = totaldrop_relay::spawn_local(Default::default())
        .await
        .unwrap();
    let (receivers_url, _, receivers, _receivers_task) =
        totaldrop_relay::spawn_local(Default::default())
            .await
            .unwrap();
    let src = scratch("src");
    std::fs::write(src.join("note.txt"), b"hello").unwrap();
    let prepared = PreparedSend::new(
        &SendOptions {
            server: server.clone(),
            words: 2,
            paths: PathOptions {
                relay: senders_url,
                stun: String::new(),
                relay_only: true,
            },
        },
        vec![src.join("note.txt")],
    )
    .await
    .unwrap();
    let code = prepared.code().to_string();
    let sender = tokio::spawn(prepared.run(no_events()));
    let started = std::time::Instant::now();
    let offer = IncomingOffer::connect(
        &ReceiveOptions {
            server,
            paths: PathOptions {
                relay: receivers_url,
                stun: String::new(),
                relay_only: true,
            },
        },
        &code,
    )
    .await;
    assert!(matches!(offer, Err(Error::Connect(_))));
    assert!(started.elapsed() < Duration::from_secs(10));
    // The sender sees the mismatch in the receiver's HINTS and stops too.
    assert!(sender.await.unwrap().is_err());
    // Neither side's relay heard of the transfer.
    assert_eq!(senders.connections_accepted(), 0);
    assert_eq!(receivers.connections_accepted(), 0);
    std::fs::remove_dir_all(src).unwrap();
}

/// Progress events that count the bytes moved and, if `slow`, take a little
/// time per chunk so a stop lands mid-transfer.
fn counting(
    moved: std::sync::Arc<std::sync::atomic::AtomicU64>,
    slow: bool,
) -> totaldrop_core::Events {
    std::sync::Arc::new(move |e| {
        if let totaldrop_core::Event::Progress { done, .. } = e {
            moved.store(done, std::sync::atomic::Ordering::SeqCst);
            if slow {
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    })
}

/// Stop `stop` from its own task, as a person pressing Stop would, once
/// `moved` reaches `after` bytes.
fn stop_when(
    stop: totaldrop_core::Stop,
    moved: std::sync::Arc<std::sync::atomic::AtomicU64>,
    after: u64,
) {
    tokio::spawn(async move {
        while moved.load(std::sync::atomic::Ordering::SeqCst) < after {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        stop.stop();
    });
}

/// Nothing this transfer started is left in `dest`: no partial files and no
/// folders it created.
fn left_nothing(dest: &Path) {
    let names: Vec<_> = std::fs::read_dir(dest).unwrap().collect();
    assert!(names.is_empty(), "left behind: {:?}", walk(dest));
}

/// A folder of `files` files of `size` bytes each, in a subfolder.
fn folder(src: &Path, files: usize, size: usize) -> PathBuf {
    let dir = src.join("album");
    std::fs::create_dir_all(dir.join("raw")).unwrap();
    for i in 0..files {
        let seed = u8::try_from(i % 251).unwrap();
        std::fs::write(
            dir.join("raw").join(format!("{i:03}.bin")),
            pseudo_random(size, seed),
        )
        .unwrap();
    }
    dir
}

async fn prepared(server: &str, what: PathBuf) -> PreparedSend {
    PreparedSend::new(
        &SendOptions {
            server: server.to_owned(),
            words: 2,
            paths: no_relay(),
        },
        vec![what],
    )
    .await
    .unwrap()
}

async fn offer(server: String, code: &str) -> IncomingOffer {
    IncomingOffer::connect(
        &ReceiveOptions {
            server,
            paths: no_relay(),
        },
        code,
    )
    .await
    .unwrap()
}

/// The receiver stops while its streams wait for data: the sender is told,
/// and nothing is left in the destination.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn receiver_stops_mid_transfer() {
    let server = server().await;
    let (src, dest) = (scratch("src"), scratch("dest"));
    let p = prepared(&server, folder(&src, 2, 32 << 20)).await;
    let code = p.code().to_string();
    // The sender is the slow side, so the receiver's streams sit idle on a
    // read when the stop comes, as they would behind a real network.
    let sender = tokio::spawn(p.run(counting(Default::default(), true)));
    let o = offer(server, &code).await;
    let (stop, moved) = (totaldrop_core::Stop::new(), std::sync::Arc::default());
    stop_when(stop.clone(), std::sync::Arc::clone(&moved), 4 << 20);
    let r = o.accept_with(&dest, counting(moved, false), &stop).await;
    assert!(matches!(r, Err(Error::Stopped)), "{r:?}");
    let s = tokio::time::timeout(Duration::from_secs(10), sender)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(s, Err(Error::Cancelled(_))), "{s:?}");
    left_nothing(&dest);
    std::fs::remove_dir_all(src).unwrap();
    std::fs::remove_dir_all(dest).unwrap();
}

/// The sender stops in the middle of a folder of many files: it stops at
/// once, not after the files still waiting for a stream. The slow receiver
/// blocks worker threads, so this test sets how many there are.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn sender_stops_mid_folder() {
    let server = server().await;
    let (src, dest) = (scratch("src"), scratch("dest"));
    let total: u64 = 24 * (2 << 20);
    let p = prepared(&server, folder(&src, 24, 2 << 20)).await;
    let code = p.code().to_string();
    let (stop, moved) = (totaldrop_core::Stop::new(), std::sync::Arc::default());
    stop_when(stop.clone(), std::sync::Arc::clone(&moved), 4 << 20);
    let sender = tokio::spawn(p.run_with(counting(std::sync::Arc::clone(&moved), false), stop));
    let o = offer(server, &code).await;
    // The receiver is the slow side here.
    let r = o.accept(&dest, counting(Default::default(), true)).await;
    assert!(matches!(r, Err(Error::Cancelled(_))), "{r:?}");
    let s = sender.await.unwrap();
    assert!(matches!(s, Err(Error::Stopped)), "{s:?}");
    let sent = moved.load(std::sync::atomic::Ordering::SeqCst);
    assert!(
        sent < total / 2,
        "sent {sent} of {total} bytes after the stop"
    );
    left_nothing(&dest);
    std::fs::remove_dir_all(src).unwrap();
    std::fs::remove_dir_all(dest).unwrap();
}

/// The receiver leaves while the offer is shown: the sender hears that it
/// stopped, not a connection error.
#[tokio::test(flavor = "multi_thread")]
async fn receiver_stops_at_the_offer() {
    let server = server().await;
    let src = scratch("src");
    std::fs::write(src.join("a.txt"), b"a").unwrap();
    let p = prepared(&server, src.join("a.txt")).await;
    let code = p.code().to_string();
    let sender = tokio::spawn(p.run(no_events()));
    let o = offer(server, &code).await;
    assert!(matches!(o.stop().await, Error::Stopped));
    let s = tokio::time::timeout(Duration::from_secs(10), sender)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(s, Err(Error::Cancelled(_))), "{s:?}");
    std::fs::remove_dir_all(src).unwrap();
}

/// Stopping before anyone connected just leaves, at once; so does a
/// receiver stopped while it connects.
#[tokio::test(flavor = "multi_thread")]
async fn stop_while_waiting_and_while_connecting() {
    let server = server().await;
    let src = scratch("src");
    std::fs::write(src.join("a.txt"), b"a").unwrap();
    let p = prepared(&server, src.join("a.txt")).await;
    let code = p.code().to_string();
    let stop = totaldrop_core::Stop::new();
    let sender = tokio::spawn(p.run_with(no_events(), stop.clone()));
    tokio::time::sleep(Duration::from_millis(200)).await;
    stop.stop();
    let s = tokio::time::timeout(Duration::from_secs(1), sender)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(s, Err(Error::Stopped)), "{s:?}");

    // A receiver stopped while it is connecting to that (now gone) sender.
    let stop = totaldrop_core::Stop::new();
    let connecting = {
        let stop = stop.clone();
        tokio::spawn(async move {
            IncomingOffer::connect_with(
                &ReceiveOptions {
                    server,
                    paths: no_relay(),
                },
                &code,
                &stop,
            )
            .await
        })
    };
    tokio::time::sleep(Duration::from_millis(50)).await;
    stop.stop();
    let r = tokio::time::timeout(Duration::from_secs(1), connecting)
        .await
        .unwrap()
        .unwrap();
    assert!(
        matches!(r, Err(Error::Stopped) | Err(Error::Rendezvous(_))),
        "{r:?}"
    );
    std::fs::remove_dir_all(src).unwrap();
}
