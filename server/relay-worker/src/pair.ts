// SPDX-License-Identifier: AGPL-3.0-or-later
//
// One Pair object per transfer: at most two WebSockets, one per endpoint key
// named in the object's name. Everything else is refused.

import { DurableObject } from "cloudflare:workers";
import type { Env } from "./index";

const MAGIC = new TextEncoder().encode("TDR0");
const AUTH_CONTEXT = new TextEncoder().encode("totaldrop relay v0 auth");
const OK = new Uint8Array([...MAGIC, ...new TextEncoder().encode("ok")]);

/** Largest message either side may send (relay.rs MAX_MESSAGE). */
const MAX_MESSAGE = 64 * 1024;
/** Time to sign in after connecting. */
const SIGN_IN_TIMEOUT = 10_000;
/** How long one transfer may use the relay. */
const SESSION_TIME = 6 * 60 * 60 * 1000;
/** Bytes one side may send in total. */
const SESSION_BYTES = 32 * 1024 * 1024 * 1024;

/** WebSocket readyState OPEN. */
const OPEN = 1;

interface Attachment {
  /** Hex of the challenge sent to this socket. */
  challenge: string;
  /** Hex of the endpoint key, once signed in. */
  key: string | null;
  /** When the socket connected, in ms since the epoch. */
  since: number;
  /** Set when this object closed the socket; it may linger as CLOSING. */
  ended?: boolean;
}

function attachment(ws: WebSocket): Attachment | null {
  return ws.deserializeAttachment() as Attachment | null;
}

function hex(b: Uint8Array): string {
  return Array.from(b, (x) => x.toString(16).padStart(2, "0")).join("");
}

function unhex(s: string): Uint8Array {
  const out = new Uint8Array(s.length / 2);
  for (let i = 0; i < out.length; i++) out[i] = parseInt(s.slice(2 * i, 2 * i + 2), 16);
  return out;
}

/** The two keys of `<key A>-<key B>`: lowercase hex, A strictly lower. */
export function parsePairName(name: string): [string, string] | null {
  const m = /^([0-9a-f]{64})-([0-9a-f]{64})$/.exec(name);
  if (!m || !m[1] || !m[2] || !(m[1] < m[2])) return null;
  return [m[1], m[2]];
}

function startsWith(m: Uint8Array, prefix: Uint8Array): boolean {
  return m.length >= prefix.length && prefix.every((b, i) => m[i] === b);
}

async function bytes(data: string | ArrayBuffer): Promise<Uint8Array | null> {
  return typeof data === "string" ? null : new Uint8Array(data);
}

async function verify(key: Uint8Array, signature: Uint8Array, message: Uint8Array): Promise<boolean> {
  try {
    const k = await crypto.subtle.importKey("raw", key, { name: "Ed25519" }, false, ["verify"]);
    return await crypto.subtle.verify({ name: "Ed25519" }, k, signature, message);
  } catch {
    return false;
  }
}

export class Pair extends DurableObject<Env> {
  /** Bytes sent by each key since this object woke; a cap, not a record. */
  private sent = new Map<string, number>();
  /** Signed-in sockets by key, rebuilt after the object wakes. */
  private byKey = new Map<string, WebSocket>();

  /** Sockets that still count: open, and not closed by this object. A
   * socket this object closed can stay in getWebSockets() as CLOSING for as
   * long as its client ignores the close frame; it must not hold a slot. */
  private live(): WebSocket[] {
    return this.ctx.getWebSockets().filter((o) => o.readyState === OPEN && !attachment(o)?.ended);
  }

  private socketFor(key: string): WebSocket | undefined {
    const known = this.byKey.get(key);
    if (known && this.live().includes(known)) return known;
    this.byKey.delete(key);
    const found = this.live().find((o) => attachment(o)?.key === key);
    if (found) this.byKey.set(key, found);
    return found;
  }

  /** Close `ws` and remember that this object did, so it stops counting. */
  private shut(ws: WebSocket, code: number, reason: string): void {
    const att = attachment(ws);
    if (att && !att.ended) {
      att.ended = true;
      ws.serializeAttachment(att);
    }
    try {
      ws.close(code, reason);
    } catch {
      // Already closed.
    }
  }

  async fetch(request: Request): Promise<Response> {
    const name = new URL(request.url).pathname.split("/").pop() ?? "";
    if (!parsePairName(name)) return new Response(null, { status: 400 });
    if (this.live().length >= 2) return new Response(null, { status: 409 });
    const [client, server] = Object.values(new WebSocketPair()) as [WebSocket, WebSocket];
    const challenge = crypto.getRandomValues(new Uint8Array(32));
    this.ctx.acceptWebSocket(server, [name]);
    const attachment: Attachment = { challenge: hex(challenge), key: null, since: Date.now() };
    server.serializeAttachment(attachment);
    server.send(new Uint8Array([...MAGIC, ...challenge]));
    await this.schedule();
    return new Response(null, { status: 101, webSocket: client });
  }

  private pairOf(ws: WebSocket): [string, string] | null {
    const tag = this.ctx.getTags(ws)[0];
    return tag ? parsePairName(tag) : null;
  }

  async webSocketMessage(ws: WebSocket, data: string | ArrayBuffer): Promise<void> {
    // A socket this object closed may still send while CLOSING: ignore it.
    if (ws.readyState !== OPEN || attachment(ws)?.ended) return;
    const m = await bytes(data);
    if (!m || m.length > MAX_MESSAGE) return this.shut(ws, 1008, "bad message");
    const own = this.keyOf(ws);
    if (own !== null) return this.forward(ws, own, m);

    const att = ws.deserializeAttachment() as Attachment | null;
    const pair = this.pairOf(ws);
    if (!att || !pair) return this.shut(ws, 1008, "bad message");

    // Sign in: TDR0 || key || signature over context || challenge.
    if (m.length !== 4 + 32 + 64 || !startsWith(m, MAGIC)) return this.shut(ws, 1008, "bad hello");
    const key = m.slice(4, 36);
    const keyHex = hex(key);
    if (keyHex !== pair[0] && keyHex !== pair[1]) return this.shut(ws, 1008, "not this pair");
    const signed = new Uint8Array([...AUTH_CONTEXT, ...unhex(att.challenge)]);
    if (!(await verify(key, m.slice(36), signed))) return this.shut(ws, 1008, "bad signature");
    // Other events can run during verify(); the socket may be gone by now.
    if (ws.readyState !== OPEN || attachment(ws)?.ended) return;
    // Checked after the signature, so two sign-ins racing for one key
    // cannot both pass.
    const taken = this.socketFor(keyHex);
    if (taken && taken !== ws) return this.shut(ws, 1008, "key in use");
    att.key = keyHex;
    ws.serializeAttachment(att);
    this.byKey.set(keyHex, ws);
    ws.send(OK);
    await this.schedule();
  }

  /** The signed-in key of `ws`, from memory or its attachment. */
  private keyOf(ws: WebSocket): string | null {
    for (const [k, o] of this.byKey) if (o === ws) return k;
    const key = (ws.deserializeAttachment() as Attachment | null)?.key ?? null;
    if (key !== null) this.byKey.set(key, ws);
    return key;
  }

  /** Datagrams: destination key || (length || datagram)+. Only the other key
   * of the pair is a valid destination; the rest passes through. */
  private forward(ws: WebSocket, own: string, m: Uint8Array): void {
    const pair = this.pairOf(ws);
    if (!pair || m.length <= 32 + 2) return this.shut(ws, 1008, "bad frame");
    const other = own === pair[0] ? pair[1] : pair[0];
    if (hex(m.subarray(0, 32)) !== other) return this.shut(ws, 1008, "bad destination");
    const total = (this.sent.get(own) ?? 0) + m.length;
    if (total > SESSION_BYTES) return this.shut(ws, 1008, "limit reached");
    this.sent.set(own, total);
    const peer = this.socketFor(other);
    // Nobody there yet: drop, like a lossy network. QUIC retransmits.
    if (!peer) return;
    m.set(unhex(own), 0);
    try {
      peer.send(m);
    } catch {
      // The other side is gone; its close handler tidies up.
    }
  }

  async webSocketClose(ws: WebSocket): Promise<void> {
    await this.end(ws);
  }

  async webSocketError(ws: WebSocket): Promise<void> {
    await this.end(ws);
  }

  /** When a signed-in side leaves, the transfer is over: close the other
   * too. A socket that never signed in, or one this object already closed,
   * takes nobody with it. Afterwards the alarm follows whatever is left,
   * and is deleted when nothing is. */
  private async end(ws: WebSocket): Promise<void> {
    const att = attachment(ws);
    const cascade = att !== null && att.key !== null && !att.ended;
    this.shut(ws, 1000, "closed");
    if (cascade) {
      for (const o of this.live()) this.shut(o, 1000, "peer left");
      this.sent.clear();
      this.byKey.clear();
    } else if (att?.key) {
      this.byKey.delete(att.key);
    }
    await this.schedule();
  }

  /** When `ws` must have signed in, or must leave. */
  private static deadline(att: Attachment): number {
    return att.key === null ? att.since + SIGN_IN_TIMEOUT : att.since + SESSION_TIME;
  }

  /** Set the one alarm to the earliest deadline of any socket, so a sign-in
   * does not push back the other socket's sign-in timeout. */
  private async schedule(): Promise<void> {
    let next = Infinity;
    for (const ws of this.live()) {
      const att = attachment(ws);
      if (att) next = Math.min(next, Pair.deadline(att));
    }
    if (next === Infinity) await this.ctx.storage.deleteAlarm();
    else await this.ctx.storage.setAlarm(next);
  }

  /** Close sockets that did not sign in in time, or outlived the session. */
  async alarm(): Promise<void> {
    const now = Date.now();
    let next = Infinity;
    for (const ws of this.live()) {
      const att = attachment(ws);
      if (!att) continue;
      const deadline = Pair.deadline(att);
      if (now >= deadline) {
        this.shut(ws, 1008, att.key === null ? "sign-in timeout" : "session over");
      } else {
        next = Math.min(next, deadline);
      }
    }
    if (next !== Infinity) await this.ctx.storage.setAlarm(next);
  }
}
