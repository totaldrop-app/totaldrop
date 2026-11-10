// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// One Pair object per transfer: at most two WebSockets, one per endpoint key
// named in the object's name. Everything else is refused.

import { DurableObject } from "cloudflare:workers";
import { PROBE_STEP, restored } from "./count";
import type { Env } from "./index";

const MAGIC = new TextEncoder().encode("TDR0");
const AUTH_CONTEXT = new TextEncoder().encode("totaldrop relay v0 auth");
const OK = new Uint8Array([...MAGIC, ...new TextEncoder().encode("ok")]);
const PROBE = new Uint8Array([...MAGIC, ...new TextEncoder().encode("probe")]);
/** A probe: TDR0 || "probe" || 16 random bytes (protocol note 20). */
const PROBE_LENGTH = PROBE.length + 16;

/** Largest message either side may send (relay.rs MAX_MESSAGE). */
const MAX_MESSAGE = 64 * 1024;
/** Time to sign in after connecting. */
const SIGN_IN_TIMEOUT = 10_000;
/** How long one transfer may use the relay. */
const SESSION_TIME = 6 * 60 * 60 * 1000;
/** Bytes one side may send in total. */
const SESSION_BYTES = 32 * 1024 * 1024 * 1024;
/** The count of bytes sent is written to the socket's attachment, which
 * survives hibernation, each time it passes another step of this size; a
 * datagram costs no extra write, and waking loses less than one step. */
const SENT_STEP = 1024 * 1024;

/** A Workers WebSocket does not say how much it still holds to send, and
 * holds all of it in memory: in a local test, 512 MiB for a receiver that
 * did not read (#190). So the object puts a probe in the stream every
 * PROBE_STEP bytes it gives a socket; the device sends it back once it has
 * read everything before it. Datagrams for a socket that has more than
 * UNANSWERED bytes after its last answered probe are dropped, like a lossy
 * network; QUIC sends them again. A TotalDrop device keeps at most 16 MiB
 * in flight (QUIC's connection window); in a local test under that window
 * no socket went past 18 MiB unanswered.
 *
 * A key's replaced sockets keep what they were given until they are gone
 * (a client can hold TCP open and never read), so it counts against the
 * key too: at most KEY_UNANSWERED over all its sockets. An honest key whose
 * old socket died with up to 16 MiB queued still gets its full window. */
const UNANSWERED = 32 * 1024 * 1024;
const KEY_UNANSWERED = 2 * UNANSWERED;

/** WebSocket readyState OPEN. */
const OPEN = 1;
/** Sockets an object holds at most, lingering closed ones included: a
 * client that ignores close frames cannot pile them up. Each replaced
 * socket of a long transfer may linger too (its client is gone and never
 * answers the close frame), so this leaves room for many reconnects. */
const MAX_SOCKETS = 32;
/** Open sockets that have not signed in yet. Not a cap on all open sockets:
 * a side whose old socket died unnoticed must still get in to replace it. */
const MAX_WAITING = 2;

interface Attachment {
  /** Hex of the challenge sent to this socket. */
  challenge: string;
  /** Hex of the endpoint key, once signed in. */
  key: string | null;
  /** When the socket connected, in ms since the epoch. */
  since: number;
  /** Set when this object closed the socket; it may linger as CLOSING. */
  ended?: boolean;
  /** Bytes this socket had sent, at the last SENT_STEP passed. */
  sent?: number;
  /** Bytes given to this socket when its last probe went out. */
  given?: number;
  /** Bytes given to it when its last answered probe went out. */
  answered?: number;
  /** Probes not answered yet, oldest first: nonce hex and bytes given when
   * it went out. At most UNANSWERED / PROBE_STEP, a few hundred bytes. */
  probes?: [string, number][];
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
  /** Bytes each socket sent: the cap is per connection, as on the Rust
   * relay. Read from the attachment after the object wakes (a woken object
   * sees new socket objects), then counted here. */
  private sent = new WeakMap<WebSocket, number>();
  /** Bytes given to each socket, then given at its last answered probe.
   * After the object wakes (a woken object sees new socket objects), read
   * from the attachment ({@link restored}): fewer than PROBE_STEP bytes
   * follow the last probe, or the start, so adding one step counts high,
   * never low. */
  private given = new WeakMap<WebSocket, number>();
  private answeredAt = new WeakMap<WebSocket, number>();
  /** Unanswered bytes on the closed (replaced) sockets of each key, which
   * no longer change; dropped when one of them goes. */
  private stale = new Map<string, number>();
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
    try {
      const att = attachment(ws);
      if (att && !att.ended && ws.readyState === OPEN) {
        att.ended = true;
        ws.serializeAttachment(att);
      }
      ws.close(code, reason);
    } catch {
      // Already closed.
    }
  }

  async fetch(request: Request): Promise<Response> {
    const name = new URL(request.url).pathname.split("/").pop() ?? "";
    if (!parsePairName(name)) return new Response(null, { status: 400 });
    const waiting = this.live().filter((o) => attachment(o)?.key === null).length;
    if (waiting >= MAX_WAITING || this.ctx.getWebSockets().length >= MAX_SOCKETS)
      return new Response(null, { status: 409 });
    const [client, server] = Object.values(new WebSocketPair()) as [WebSocket, WebSocket];
    const challenge = crypto.getRandomValues(new Uint8Array(32));
    this.ctx.acceptWebSocket(server, [name]);
    const att: Attachment = { challenge: hex(challenge), key: null, since: Date.now() };
    server.serializeAttachment(att);
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
    const own = this.keyOf(ws);
    if (!m || m.length > MAX_MESSAGE) return this.fail(ws, own, "bad message");
    if (own !== null && m.length === PROBE_LENGTH && startsWith(m, PROBE)) return this.answered(ws, m);
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
    // The holder of the key signed in again: its old socket died without
    // this object noticing (a Wi-Fi switch, NAT rebinding). The new one
    // takes over at once, before QUIC gives up (#190). Closed through
    // shut(), the old socket takes nobody with it. Checked after the
    // signature, so only the key's holder can do this.
    const taken = this.socketFor(keyHex);
    if (taken && taken !== ws) this.shut(taken, 1000, "replaced");
    this.stale.delete(keyHex);
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
    if (!pair || m.length <= 32 + 2) return this.fail(ws, own, "bad frame");
    const other = own === pair[0] ? pair[1] : pair[0];
    if (hex(m.subarray(0, 32)) !== other) return this.fail(ws, own, "bad destination");
    const before = this.sent.get(ws) ?? attachment(ws)?.sent ?? 0;
    const total = before + m.length;
    if (total > SESSION_BYTES) return this.fail(ws, own, "limit reached");
    this.sent.set(ws, total);
    if (Math.floor(total / SENT_STEP) > Math.floor(before / SENT_STEP)) {
      const att = attachment(ws);
      if (att) {
        att.sent = total;
        ws.serializeAttachment(att);
      }
    }
    const peer = this.socketFor(other);
    // Nobody there yet: drop, like a lossy network. QUIC retransmits.
    if (!peer) return;
    // The other side holds too much it has not read: drop, as above.
    const [given, answered] = this.counts(peer);
    const unanswered = given + m.length - answered;
    if (unanswered > UNANSWERED || unanswered + this.staleFor(other, peer) > KEY_UNANSWERED) return;
    m.set(unhex(own), 0);
    try {
      peer.send(m);
      this.gave(peer, given, given + m.length);
    } catch {
      // The other side is gone; its close handler tidies up.
    }
  }

  /** Bytes given to `ws`, and given at its last answered probe. */
  private counts(ws: WebSocket): [number, number] {
    let given = this.given.get(ws);
    let answered = this.answeredAt.get(ws);
    if (given === undefined || answered === undefined) {
      [given, answered] = restored(attachment(ws));
      this.given.set(ws, given);
      this.answeredAt.set(ws, answered);
    }
    return [given, answered];
  }

  /** Unanswered bytes on the other sockets of `key` than `live`. */
  private staleFor(key: string, live: WebSocket): number {
    const known = this.stale.get(key);
    if (known !== undefined) return known;
    let sum = 0;
    for (const o of this.ctx.getWebSockets()) {
      if (o === live || attachment(o)?.key !== key) continue;
      const [given, answered] = this.counts(o);
      sum += given - answered;
    }
    this.stale.set(key, sum);
    return sum;
  }

  /** Count `before` to `after` bytes given to `ws`; each PROBE_STEP passed
   * sends a probe after them. */
  private gave(ws: WebSocket, before: number, after: number): void {
    this.given.set(ws, after);
    if (Math.floor(after / PROBE_STEP) <= Math.floor(before / PROBE_STEP)) return;
    const att = attachment(ws);
    if (!att) return;
    const nonce = crypto.getRandomValues(new Uint8Array(16));
    ws.send(new Uint8Array([...PROBE, ...nonce]));
    att.given = after;
    att.probes = [...(att.probes ?? []), [hex(nonce), after]];
    ws.serializeAttachment(att);
  }

  /** A probe came back: everything given before it was read. Only a nonce
   * this object sent counts; any other answer is ignored, and none is
   * forwarded or counted against SESSION_BYTES. */
  private answered(ws: WebSocket, m: Uint8Array): void {
    const att = attachment(ws);
    const probes = att?.probes ?? [];
    const nonce = hex(m.subarray(PROBE.length));
    const at = probes.findIndex(([n]) => n === nonce);
    const probe = probes[at];
    if (!att || !probe) return;
    att.answered = probe[1];
    this.answeredAt.set(ws, probe[1]);
    att.probes = probes.slice(at + 1);
    ws.serializeAttachment(att);
  }

  async webSocketClose(ws: WebSocket): Promise<void> {
    await this.end(ws);
  }

  async webSocketError(ws: WebSocket): Promise<void> {
    await this.end(ws);
  }

  /** This object ends `ws` for breaking a rule. A signed-in socket takes
   * the other side with it, as when it leaves (see end()). */
  private fail(ws: WebSocket, own: string | null, reason: string): void {
    this.shut(ws, 1008, reason);
    if (own !== null) this.endOthers(ws);
  }

  /** Close the other signed-in socket; sockets still signing in stay (one
   * may be the other side's reconnect), as on the Rust relay. */
  private endOthers(ws: WebSocket): void {
    for (const o of this.live()) {
      const key = attachment(o)?.key ?? null;
      if (o === ws || key === null) continue;
      this.shut(o, 1000, "peer left");
      this.byKey.delete(key);
    }
  }

  /** When a signed-in side leaves, the transfer is over: close the other
   * too. A socket that never signed in, or one this object already closed
   * (replaced, or ended through fail(), which closes the other itself),
   * takes nobody with it. Afterwards the alarm follows whatever is left,
   * and is deleted when nothing is. */
  private async end(ws: WebSocket): Promise<void> {
    const att = attachment(ws);
    const cascade = att !== null && att.key !== null && !att.ended;
    this.shut(ws, 1000, "closed");
    if (att?.key) this.stale.delete(att.key);
    if (cascade) {
      this.endOthers(ws);
      if (att.key && this.byKey.get(att.key) === ws) this.byKey.delete(att.key);
    } else if (att?.key && this.byKey.get(att.key) === ws) {
      // Not when a newer socket replaced this one under its key.
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
        if (att.key === null) this.shut(ws, 1008, "sign-in timeout");
        else this.fail(ws, att.key, "session over");
      } else {
        next = Math.min(next, deadline);
      }
    }
    if (next !== Infinity) await this.ctx.storage.setAlarm(next);
  }
}
