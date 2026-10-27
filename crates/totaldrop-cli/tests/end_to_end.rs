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

use totaldrop_core::rendezvous::Mailbox;
use totaldrop_core::{Error, IncomingOffer, PreparedSend, ReceiveOptions, SendOptions, no_events};
use totaldrop_proto::wire::Mood;

async fn server() -> String {
    let (addr, _server, _task) =
        totaldrop_rendezvous::spawn("127.0.0.1:0".parse().unwrap(), Default::default())
            .await
            .unwrap();
    format!("ws://{addr}")
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
        },
        vec![src.join("a.txt")],
    )
    .await
    .unwrap();
    let code = prepared.code().to_string();
    assert_eq!(code.split('-').count(), 4, "three words plus the nameplate");
    let sender = tokio::spawn(prepared.run(no_events()));
    let offer = IncomingOffer::connect(&ReceiveOptions { server }, &code)
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
    let mut a = Mailbox::connect(&server).await.unwrap();
    let n = a.allocate().await.unwrap();
    let id_a = a.claim(n).await.unwrap();
    let mut b = Mailbox::connect(&server).await.unwrap();
    let id_b = b.claim(n).await.unwrap();
    assert_eq!(id_a, id_b);
    let mut c = Mailbox::connect(&server).await.unwrap();
    assert!(matches!(c.claim(n).await, Err(Error::Rendezvous(r)) if r == "nameplate full"));
}

#[tokio::test(flavor = "multi_thread")]
async fn server_does_not_hand_out_unallocated_nameplates() {
    let server = server().await;
    let mut c = Mailbox::connect(&server).await.unwrap();
    assert!(matches!(c.claim(42).await, Err(Error::Rendezvous(_))));
}

#[tokio::test(flavor = "multi_thread")]
async fn scary_close_retires_the_nameplate() {
    let server = server().await;
    let mut a = Mailbox::connect(&server).await.unwrap();
    let n = a.allocate().await.unwrap();
    a.claim(n).await.unwrap();
    let mut b = Mailbox::connect(&server).await.unwrap();
    b.claim(n).await.unwrap();
    a.add("pake", vec![1; 32]).await.unwrap();
    b.add("pake", vec![2; 32]).await.unwrap();
    assert_eq!(a.expect("pake").await.unwrap(), vec![2; 32]);
    a.close(Mood::Scary).await;
    b.close(Mood::Lonely).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let mut d = Mailbox::connect(&server).await.unwrap();
    let next = d.allocate().await.unwrap();
    assert_ne!(next, n, "a retired nameplate must not be reused");
}

#[tokio::test(flavor = "multi_thread")]
async fn plain_websocket_to_another_host_is_refused() {
    let r = Mailbox::connect("ws://example.com/v0").await;
    assert!(matches!(r, Err(Error::Rendezvous(_))));
}

#[tokio::test(flavor = "multi_thread")]
async fn expired_nameplate_holders_are_disconnected() {
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
    let mut stale = Mailbox::connect(&url).await.unwrap();
    let n = stale.allocate().await.unwrap();
    tokio::time::sleep(Duration::from_millis(800)).await;
    // The number goes to someone else; the stale holder can no longer act.
    let mut fresh = Mailbox::connect(&url).await.unwrap();
    let m = fresh.allocate().await.unwrap();
    fresh.claim(m).await.unwrap();
    assert!(stale.claim(n).await.is_err());
}
