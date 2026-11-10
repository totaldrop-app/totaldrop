# TotalDrop mail Worker

Cloudflare Email Routing sends each routing rule to a single destination. This Worker lets one project address, such as `hello@totaldrop.app`, reach every maintainer: the address's rule sends mail here, and the Worker forwards each message once to every address in its `FORWARD_TO` secret (#415). It is not part of the TotalDrop protocol and the apps never use it. Licensed AGPL-3.0-or-later.

How it is built:

- It keeps nothing and logs nothing about the mail (observability is off in `wrangler.jsonc`). It has no HTTP address.
- `FORWARD_TO` holds the destinations, separated by commas, spaces or new lines; repeats count once, at most 10. The addresses are personal data and are never in the repository: deploys set them from the `cloudflare` environment's `MAIL_FORWARD_TO` secret (`.github/workflows/deploy.yml`).
- A missing or malformed setting, or an address on `totaldrop.app` (it would loop), fails the message instead of sending it anywhere else. Error messages never contain an address.
- One failed destination, for example one Cloudflare has not verified yet, does not stop the others; when all fail, the message fails and shows as an error in Email Routing's activity log.

## Develop and test

```bash
npm ci
npm run check   # type check, Worker and tests
npm test        # unit tests on Node's test runner
```

Setting up a destination, pointing a rule at the Worker, testing with a real message and rolling back are in `docs/surec.md`, section "E-posta yönlendirmesi" (Turkish).
