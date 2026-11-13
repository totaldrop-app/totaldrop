# TotalDrop relay on Cloudflare Workers

This is the production relay for `relay.totaldrop.app`. It speaks the same protocol as the Rust relay in `../totaldrop-relay` (see `crates/totaldrop-proto/src/relay.rs`), and the same end-to-end tests run against both, in CI whenever a Worker, a Rust server or the client crates change (`.github/workflows/server.yml`). Against a Worker a few checks are weaker: byte counts are not read and STUN is not tried. Licensed AGPL-3.0-or-later.

| Path | What it does |
| --- | --- |
| `/v0/relay/<key A>-<key B>` | The two devices of one transfer sign in with their single-use endpoint keys and exchange encrypted QUIC datagrams |

How it is built:

- The Worker keeps no state and logs nothing. Observability is turned off in `wrangler.jsonc`.
- Each transfer is its own `Pair` Durable Object, named by its two keys. It admits only those two keys, one socket each, after checking an Ed25519 signature over a fresh challenge; a new sign-in with a key replaces that key's old socket. It forwards datagram messages from one to the other, swapping the destination key for the source key, and closes both sockets when either leaves or breaks a rule, or when a connection reaches 6 hours or 32 GiB (counted per connection).
- A Workers WebSocket does not say how much it still holds to send, so the object bounds it with probes (protocol note 20): every 4 MiB it gives a socket, it sends a probe that the device sends back once it has read everything before it. Datagrams for a socket with more than 32 MiB after its last answered probe, or for a key with more than 64 MiB over all its sockets (a replaced socket counts until it is gone), are dropped, like a lossy network. Devices from before 0.3.0-beta.4 never answer, so a connection of theirs gets at most 32 MiB through this relay, sending or receiving. The bound is per pair; nothing limits how many pairs one client opens, so many pairs together can still fill an isolate.
- It uses the WebSocket Hibernation API, so a pair that waits costs nothing.
- Address discovery is not part of this Worker: clients use Cloudflare's STUN server, `stun.cloudflare.com:3478`.

## Develop and test

```bash
npm ci
npm run check                      # type check
npm test                           # unit tests (Node 22.18 or later)
npx wrangler dev --local --port 8798
# in another terminal, from the repository root:
TOTALDROP_TEST_RELAY=ws://127.0.0.1:8798/v0/relay cargo test -p totaldrop-cli --test end_to_end relay
```

## Deploy

Deploys run from CI, from `main` only, after the owner approves them in the
`cloudflare` environment (`.github/workflows/deploy.yml`). A health probe
(`server/tools/probe.mjs relay`) follows every deploy. Setup and rollback
are in [docs/surec.md](../../docs/surec.md), section "Dağıtım".

The route in `wrangler.jsonc` attaches the Worker to the custom domain `relay.totaldrop.app`. Cloudflare creates the DNS record itself.

## Edge rate limit

Anyone can sign in to the relay with any key, and nothing in the Worker limits how many pairs one address opens. So a rate limiting rule at Cloudflare's edge, on the `totaldrop.app` zone, turns away an address that opens connections too fast (#510). It lives in the dashboard (Security → Security rules → Rate limiting rules), not in `wrangler.jsonc`, so this section is its record. Change the two together.

| Setting | Value |
| --- | --- |
| Name | TotalDrop relay: new connections per IP |
| Expression | `(http.host eq "relay.totaldrop.app")` |
| Counted by | IP (`ip.src`, the full address) |
| Rate | 30 requests in 10 seconds |
| Action | Block for 10 seconds |

- It counts every HTTP request to the host. Each WebSocket upgrade, that is each relay connection, counts once; the datagrams inside an open WebSocket do not count. A device holds at most one relay connection per transfer at a time, but can open several during one: again after a connection breaks, and again when a transfer that moved to a direct path falls back to the relay (`Link::park` in `crates/totaldrop-core/src/relay.rs`). A transfer that stays on a direct path may open none.
- A blocked upgrade gets HTTP 429 from Cloudflare and never reaches the Worker or a `Pair` object, so it costs nothing. The client takes it as a failed connection and tries again after 0.5, 1, 2 and then 4 seconds, up to 10 failures in a row (`crates/totaldrop-core/src/relay.rs`). These tries are bounded and may not be enough: a receiver stops trying to connect after 20 seconds (`CONNECT_TIMEOUT` in `crates/totaldrop-core/src/lib.rs`), and others behind the same address can keep it over the rate.
- The Free plan allows one such rule, periods of 10 seconds only, and Block as the only action.
- Limits: counters are kept per Cloudflare data center and updated with a delay of up to a few seconds, so a burst can let some extra requests through. Many people behind one address (carrier-grade NAT, a large office) share one count. A client with many addresses, such as a botnet or an IPv6 range, is counted once per address.
- Cloudflare records each blocked request in the zone's Security Events (sampled; source IP, path, time and rule), kept for at least 31 days (Cloudflare documents this as a minimum and gives no deletion date). Requests the rule lets through are not recorded by it, though any request can appear in the zone's sampled request analytics, which the [privacy policy](https://totaldrop.app/privacy) describes.
- Checked on 13 November 2026: 300 parallel requests from a machine that sends from about six addresses got 164 × 200 and 136 × 429; a single request four seconds later got 200.
