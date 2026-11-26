// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// One Pair Durable Object per transfer: delegates all protocol, replacement,
// probe-backpressure and per-pair byte/time limit logic to PairCore
// (pair-core.ts) so the state machine runs identically in production and
// under Node's unit test runner (#190, #426).

import { DurableObject } from "cloudflare:workers";
import type { Env } from "./index.ts";
import { AUTH_HEADER, PairCore, parsePairName } from "./pair-core.ts";

export { parsePairName };

export class Pair extends DurableObject<Env> {
  private readonly core: PairCore<WebSocket>;

  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
    this.core = new PairCore<WebSocket>({
      getWebSockets: () => this.ctx.getWebSockets(),
      acceptWebSocket: (ws, tags) => this.ctx.acceptWebSocket(ws, tags),
      getTags: (ws) => this.ctx.getTags(ws),
      storage: {
        kv: this.ctx.storage.kv,
        setAlarm: (t) => this.ctx.storage.setAlarm(t),
        deleteAlarm: () => this.ctx.storage.deleteAlarm(),
        deleteAll: () => this.ctx.storage.deleteAll(),
      },
      env: this.env,
    });
  }

  async fetch(request: Request): Promise<Response> {
    const name = new URL(request.url).pathname.split("/").pop() ?? "";
    let client: WebSocket | undefined;
    const status = await this.core.accept(
      name,
      () => {
        const [c, server] = Object.values(new WebSocketPair()) as [WebSocket, WebSocket];
        client = c;
        return server;
      },
      undefined,
      {
        header: request.headers.get(AUTH_HEADER),
      },
    );
    if (status !== 101 || !client) return new Response(null, { status });
    return new Response(null, { status: 101, webSocket: client });
  }

  async webSocketMessage(ws: WebSocket, data: string | ArrayBuffer): Promise<void> {
    await this.core.onMessage(ws, data);
  }

  async webSocketClose(ws: WebSocket): Promise<void> {
    await this.core.end(ws);
  }

  async webSocketError(ws: WebSocket): Promise<void> {
    await this.core.end(ws);
  }

  async alarm(): Promise<void> {
    await this.core.alarm();
  }
}
