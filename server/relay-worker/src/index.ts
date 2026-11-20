// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
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

import { Pair } from "./pair.ts";
import { handleRelayRequest } from "./pair-core.ts";

export { Pair };

export interface Env {
  PAIR: DurableObjectNamespace<Pair>;
  /** Overrides of the pair limits, for local tests only (`wrangler dev --var`);
   * production sets neither. */
  SESSION_BYTES?: string;
  SESSION_TIME_MS?: string;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    return handleRelayRequest(request, env.PAIR);
  },
} satisfies ExportedHandler<Env>;
