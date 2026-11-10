# TotalDrop relay on Cloudflare Workers

This is the production relay for `relay.totaldrop.app`. It speaks the same protocol as the Rust relay in `../totaldrop-relay` (see `crates/totaldrop-proto/src/relay.rs`), and the same end-to-end tests run against both, in CI whenever a Worker, a Rust server or the client crates change (`.github/workflows/server.yml`). Against a Worker a few checks are weaker: byte counts are not read and STUN is not tried. Licensed AGPL-3.0-or-later.

| Path | What it does |
| --- | --- |
| `/v0/relay/<key A>-<key B>` | The two devices of one transfer sign in with their single-use endpoint keys and exchange encrypted QUIC datagrams |

How it is built:

- The Worker keeps no state and logs nothing. Observability is turned off in `wrangler.jsonc`.
- A WebSocket that carries an `Origin` header, which every browser sends and the apps never do, gets 403 (#450), as on the Rust relay.
- Each transfer is its own `Pair` Durable Object, named by its two keys. It admits only those two keys, one socket each, after checking an Ed25519 signature over a fresh challenge; a new sign-in with a key replaces that key's old socket. It forwards datagram messages from one to the other, swapping the destination key for the source key, and closes both sockets when either leaves or breaks a rule, or when a connection reaches 6 hours or 32 GiB (counted per connection).
- A Workers WebSocket does not say how much it still holds to send, so the object bounds it with probes (protocol note 20): every 4 MiB it gives a socket, it sends a probe that the device sends back once it has read everything before it. Datagrams for a socket with more than 32 MiB after its last answered probe, or for a key with more than 64 MiB over all its sockets (a replaced socket counts until it is gone), are dropped, like a lossy network. Devices from before 0.3.0-beta.4 never answer, so a connection of theirs gets at most 32 MiB through this relay, sending or receiving. The bound is per pair; nothing limits how many pairs one client opens, so many pairs together can still fill an isolate.
- It uses the WebSocket Hibernation API, so a pair that waits costs nothing.
- Address discovery is not part of this Worker: clients use Cloudflare's STUN server, `stun.cloudflare.com:3478`.

## Develop and test

```bash
npm ci
npm run check                      # type check
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
