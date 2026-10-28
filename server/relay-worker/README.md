# TotalDrop relay on Cloudflare Workers

This is the production relay for `relay.totaldrop.app`. It speaks the same protocol as the Rust relay in `../totaldrop-relay` (see `crates/totaldrop-proto/src/relay.rs`), and the same end-to-end tests run against both, in CI whenever a Worker, a Rust server or the client crates change (`.github/workflows/server.yml`). Against a Worker a few checks are weaker: byte counts are not read and STUN is not tried. Licensed AGPL-3.0-or-later.

| Path | What it does |
| --- | --- |
| `/v0/relay/<key A>-<key B>` | The two devices of one transfer sign in with their single-use endpoint keys and exchange encrypted QUIC datagrams |

How it is built:

- The Worker keeps no state and logs nothing. Observability is turned off in `wrangler.jsonc`.
- Each transfer is its own `Pair` Durable Object, named by its two keys. It admits only those two keys, each once, after checking an Ed25519 signature over a fresh challenge. It forwards datagram messages from one to the other, swapping the destination key for the source key, and closes both sockets when either leaves, after 6 hours, or after 32 GiB.
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

```bash
npx wrangler login
npm run deploy
```

The route in `wrangler.jsonc` attaches the Worker to the custom domain `relay.totaldrop.app`. Cloudflare creates the DNS record itself.
