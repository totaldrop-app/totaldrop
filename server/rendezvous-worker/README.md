# TotalDrop rendezvous on Cloudflare Workers

This is the production rendezvous server for `rendezvous.totaldrop.app`. It speaks exactly the same protocol as the Rust server in `../totaldrop-rendezvous`, and the same end-to-end tests run against both, in CI whenever a Worker, a Rust server or the client crates change (`.github/workflows/server.yml`). Against a Worker a few checks are weaker: byte counts are not read and STUN is not tried. Licensed AGPL-3.0-or-later.

| Path | What it does |
| --- | --- |
| `/v0/allocate` | One `allocate` message, one `allocated` reply with the nameplate and an allocator token |
| `/v0/mailbox/<n>` | `claim`, `add` and `close` for one mailbox |

How it is built:

- The Worker keeps no state and logs nothing. Observability is turned off in `wrangler.jsonc`.
- One `Directory` Durable Object decides which nameplates are free, in use, retired after a failed code (24 hours), or resting before reuse (10 minutes). It never sees mailbox contents.
- Each mailbox is its own `Mailbox` Durable Object, named by its random 16-byte mailbox ID. It holds at most two sides and briefly buffers messages for a side that has not arrived yet. It erases all of its storage when both sides have left, or when it expires (15 minutes unclaimed, one hour at most).

## Develop and test

```bash
npm ci
npm run check                      # type check
npx wrangler dev --local --port 8799
# in another terminal, from the repository root:
TOTALDROP_TEST_SERVER=ws://127.0.0.1:8799 cargo test -p totaldrop-cli --test end_to_end
```

## Deploy

Deploys run from CI, from `main` only, after the owner approves them in the
`cloudflare` environment (`.github/workflows/deploy.yml`). A health probe
(`server/tools/probe.mjs rendezvous`) follows every deploy. Setup and rollback
are in [docs/surec.md](../../docs/surec.md), section "Dağıtım".

The route in `wrangler.jsonc` attaches the Worker to the custom domain `rendezvous.totaldrop.app`. Cloudflare creates the DNS record itself.
