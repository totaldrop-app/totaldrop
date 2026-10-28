// SPDX-License-Identifier: AGPL-3.0-or-later
//
// TotalDrop relay on Cloudflare Workers (spec section 6, "Relay").
//
//   wss://relay.totaldrop.app/v0/relay/<key A>-<key B>
//
// The two single-use endpoint keys of a transfer name one Pair object. Both
// devices connect to it, prove their key by signing a challenge, and the
// object passes the encrypted QUIC datagrams between them. The wire format
// is the one in crates/totaldrop-proto/src/relay.rs. Address discovery is not
// here: clients use a STUN server for that.
//
// The Worker keeps no state and logs nothing.

import { Pair, parsePairName } from "./pair";

export { Pair };

export interface Env {
  PAIR: DurableObjectNamespace<Pair>;
}

const PREFIX = "/v0/relay/";

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const upgrade = request.headers.get("Upgrade")?.toLowerCase() === "websocket";
    if (request.method !== "GET") return new Response(null, { status: 405 });
    if (!url.pathname.startsWith(PREFIX)) {
      return new Response("TotalDrop relay", { status: upgrade ? 404 : 200 });
    }
    const name = url.pathname.slice(PREFIX.length);
    if (!upgrade || !parsePairName(name)) return new Response(null, { status: 400 });
    return env.PAIR.getByName(name).fetch(request);
  },
} satisfies ExportedHandler<Env>;
