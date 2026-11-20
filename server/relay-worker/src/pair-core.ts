// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Core state machine of one Pair Durable Object, kept free of Workers-only
// globals so hibernation wakes, socket replacement, delayed close events and
// rolling-deploy upgrades run directly under Node's test runner (#190, #426).

import { give, restore } from "./count.ts";
import { PairTotals, type Side, type TotalsKv } from "./totals.ts";

const TEXT_ENCODER = new TextEncoder();

export const MAGIC = TEXT_ENCODER.encode("TDR0");
export const AUTH_CONTEXT = TEXT_ENCODER.encode("totaldrop relay v0 auth");
export const CONNECT_CONTEXT = TEXT_ENCODER.encode("totaldrop relay v0 connect");
export const AUTH_HEADER = "X-TotalDrop-Auth";
export const RELAY_PREFIX = "/v0/relay/";
export const OK = new Uint8Array([...MAGIC, ...TEXT_ENCODER.encode("ok")]);
export const PROBE = new Uint8Array([...MAGIC, ...TEXT_ENCODER.encode("probe")]);
/** A probe: TDR0 || "probe" || 16 random bytes (protocol note 20). */
export const PROBE_LENGTH = PROBE.length + 16;

/** Largest message either side may send (relay.rs MAX_MESSAGE). */
export const MAX_MESSAGE = 64 * 1024;
/** Time to sign in after connecting. */
export const SIGN_IN_TIMEOUT = 10_000;
/** How long one pair may use the relay, from its first sign-in. */
export const SESSION_TIME = 6 * 60 * 60 * 1000;
/** Bytes one key may send on a pair in total, across reconnects. */
export const SESSION_BYTES = 32 * 1024 * 1024 * 1024;

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
 * old socket died with up to 16 MiB queued still gets its full window.
 * After a wake each replaced socket that was given bytes counts up to one
 * PROBE_STEP high (count.ts, never low), so a key with many replaced
 * sockets can see its window shrink until they are gone (#426). */
export const UNANSWERED = 32 * 1024 * 1024;
export const KEY_UNANSWERED = 2 * UNANSWERED;

/** WebSocket readyState OPEN. */
export const OPEN = 1;
/** Sockets an object holds at most, lingering closed ones included: a
 * client that ignores close frames cannot pile them up. Each replaced
 * socket of a long transfer may linger too (its client is gone and never
 * answers the close frame), so this leaves room for many reconnects. */
export const MAX_SOCKETS = 32;
/** Open sockets that have not signed in yet. Not a cap on all open sockets:
 * a side whose old socket died unnoticed must still get in to replace it. */
export const MAX_WAITING = 2;

export interface Attachment {
  /** Hex of the challenge sent to this socket. */
  challenge: string;
  /** Hex of the endpoint key, once signed in. */
  key: string | null;
  /** When the socket connected, in ms since the epoch. */
  since: number;
  /** Persisted when this object closes or replaces the socket, surviving
   * Durable Object hibernation while the socket lingers as CLOSING. */
  closed?: true;
  /** Legacy field name from earlier code, still honoured across deploys. */
  ended?: boolean;
  /** Legacy per-socket sent count from pre-#190 deployments, used only to
   * seed `"totals"` if an already-signed-in socket wakes across a deploy. */
  sent?: number;
  /** Bytes given to this socket when its last probe went out (count.ts):
   * never a full PROBE_STEP below the count. */
  given?: number;
  /** Bytes given to it when its last answered probe went out. */
  answered?: number;
  /** Endpoint key hex verified via `X-TotalDrop-Auth` on Upgrade (#78). */
  preauth?: string;
  /** Probes not answered yet, oldest first: nonce hex and bytes given when
   * it went out. At most UNANSWERED / PROBE_STEP + 1 (one goes out at the
   * first bytes), a few hundred bytes. */
  probes?: [string, number][];
}

/** Minimal WebSocket surface shared by Cloudflare Workers and Node tests. */
export interface RelaySocket {
  readonly readyState: number;
  send(message: ArrayBuffer | ArrayBufferView | string): void;
  close(code?: number, reason?: string): void;
  serializeAttachment(value: unknown): void;
  deserializeAttachment(): unknown;
}

export interface PairStorage {
  readonly kv: TotalsKv;
  setAlarm(scheduledTime: number): Promise<void>;
  deleteAlarm(): Promise<void>;
  deleteAll(): Promise<void>;
}

export interface PairEnv {
  SESSION_BYTES?: string;
  SESSION_TIME_MS?: string;
}

export interface PairHost<S extends RelaySocket = RelaySocket> {
  getWebSockets(): S[];
  acceptWebSocket(ws: S, tags: string[]): void;
  getTags(ws: S): string[];
  readonly storage: PairStorage;
  readonly env: PairEnv;
}

export function attachment(ws: RelaySocket): Attachment | null {
  return ws.deserializeAttachment() as Attachment | null;
}

function isClosedAtt(att: Attachment | null): boolean {
  return Boolean(att?.closed || att?.ended);
}

function hex(b: Uint8Array): string {
  return Array.from(b, (x) => x.toString(16).padStart(2, "0")).join("");
}

function unhex(s: string): Uint8Array {
  const out = new Uint8Array(s.length / 2);
  for (let i = 0; i < out.length; i++) out[i] = parseInt(s.slice(2 * i, 2 * i + 2), 16);
  return out;
}

export interface ConnectAuth {
  header: string | null;
}

export interface PairNamespace {
  getByName(name: string): { fetch(request: Request): Promise<Response> };
}

/** The two keys of `<key A>-<key B>`: lowercase hex, A strictly lower. */
export function parsePairName(name: string): [string, string] | null {
  const m = /^([0-9a-f]{64})-([0-9a-f]{64})$/.exec(name);
  if (!m || !m[1] || !m[2] || !(m[1] < m[2])) return null;
  return [m[1], m[2]];
}

/** Strict `<64-hex-key>.<128-hex-sig>` in lowercase ASCII hex (193 chars, #78). */
export function parseAuthHeader(value: string): [string, string] | null {
  if (value.length !== 193) return null;
  const m = /^([0-9a-f]{64})\.([0-9a-f]{128})$/.exec(value);
  if (!m || !m[1] || !m[2]) return null;
  return [m[1], m[2]];
}

function startsWith(m: Uint8Array, prefix: Uint8Array): boolean {
  return m.length >= prefix.length && prefix.every((b, i) => m[i] === b);
}

function bytes(data: string | ArrayBuffer): Uint8Array | null {
  return typeof data === "string" ? null : new Uint8Array(data);
}

export async function verifySignature(
  key: Uint8Array,
  signature: Uint8Array,
  message: Uint8Array,
): Promise<boolean> {
  try {
    const k = await crypto.subtle.importKey("raw", key, { name: "Ed25519" }, false, ["verify"]);
    return await crypto.subtle.verify({ name: "Ed25519" }, k, signature, message);
  } catch {
    return false;
  }
}

/** Verify `X-TotalDrop-Auth` against `name`; returns the authenticated
 * endpoint key hex or `null` (#78). Does not bind to `Sec-WebSocket-Key`
 * because HTTP/WebSocket proxies (including Cloudflare edge and `workerd`)
 * regenerate that hop-by-hop header; `TDR0` still proves challenge freshness. */
export async function verifyConnectAuth(name: string, header: string): Promise<string | null> {
  const pair = parsePairName(name);
  if (!pair) return null;
  const parsed = parseAuthHeader(header);
  if (!parsed) return null;
  const [keyHex, sigHex] = parsed;
  if (keyHex !== pair[0] && keyHex !== pair[1]) return null;
  const nameBytes = TEXT_ENCODER.encode(name);
  const signed = new Uint8Array(CONNECT_CONTEXT.length + nameBytes.length);
  signed.set(CONNECT_CONTEXT, 0);
  signed.set(nameBytes, CONNECT_CONTEXT.length);
  const ok = await verifySignature(unhex(keyHex), unhex(sigHex), signed);
  return ok ? keyHex : null;
}

/** Validate an incoming edge HTTP request before waking or forwarding to a
 * `Pair` Durable Object (#78, #450). */
export async function handleRelayRequest(
  request: Request,
  pair: PairNamespace,
): Promise<Response> {
  const url = new URL(request.url);
  const upgrade = request.headers.get("Upgrade")?.toLowerCase() === "websocket";
  if (request.method !== "GET") return new Response(null, { status: 405 });
  // A web page's WebSocket always carries Origin; the apps and the
  // command line never send it (#450).
  if (upgrade && request.headers.has("Origin")) {
    return new Response("browsers are not served\n", { status: 403 });
  }
  if (!url.pathname.startsWith(RELAY_PREFIX)) {
    return new Response("TotalDrop relay", { status: upgrade ? 404 : 200 });
  }
  const name = url.pathname.slice(RELAY_PREFIX.length);
  if (!upgrade || !parsePairName(name)) return new Response(null, { status: 400 });
  if (request.headers.has(AUTH_HEADER)) {
    const ok = await verifyConnectAuth(name, request.headers.get(AUTH_HEADER) ?? "");
    if (!ok) return new Response(null, { status: 403 });
  }
  return pair.getByName(name).fetch(request);
}

export class PairCore<S extends RelaySocket = RelaySocket> {
  private readonly host: PairHost<S>;
  /** Per-pair deadline and per-key sent byte counts across reconnects and
   * hibernation wakes (totals.ts, #190). */
  readonly totals: PairTotals;
  /** Sockets closed or replaced in this wake; mirrored in `att.closed` so
   * the flag survives hibernation as well. */
  private closed = new WeakSet<S>();
  /** Bytes given to each socket, then given at its last answered probe. */
  private given = new WeakMap<S, number>();
  private answeredAt = new WeakMap<S, number>();
  /** What the attachment last got of `given` (count.ts); unset while the
   * socket was given nothing. */
  private written = new WeakMap<S, number>();
  /** Unanswered bytes on the closed (replaced) sockets of each key, which
   * no longer change; dropped when one of them goes. */
  private stale = new Map<string, number>();
  /** Signed-in sockets by key, rebuilt after the object wakes. */
  private byKey = new Map<string, S>();
  /** In-flight `expireSession()` promise, serializing async KV/alarm teardown
   * against concurrent `accept()`, `onMessage()`, `end()`, `schedule()`, and
   * `alarm()` continuations (#78). */
  private expiring: Promise<void> | null = null;

  constructor(host: PairHost<S>) {
    this.host = host;
    this.totals = new PairTotals(host.storage.kv);
  }

  private async waitExpiring(): Promise<void> {
    while (this.expiring !== null) {
      const pending = this.expiring;
      try {
        await pending;
      } catch {
        // The initiating expireSession() caller observes the rejection.
      }
      if (this.expiring === pending) this.expiring = null;
    }
  }

  sessionBytes(): number {
    const n = Number(this.host.env.SESSION_BYTES);
    return Number.isInteger(n) && n > 0 ? n : SESSION_BYTES;
  }

  sessionTime(): number {
    const n = Number(this.host.env.SESSION_TIME_MS);
    return Number.isInteger(n) && n > 0 ? n : SESSION_TIME;
  }

  private isClosed(ws: S, att: Attachment | null = attachment(ws)): boolean {
    return this.closed.has(ws) || isClosedAtt(att);
  }

  /** Sockets that still count: open, and not closed by this object. A
   * socket this object closed can stay in getWebSockets() as CLOSING for as
   * long as its client ignores the close frame; it must not hold a slot. */
  live(): S[] {
    return this.host.getWebSockets().filter((o) => o.readyState === OPEN && !this.isClosed(o));
  }

  socketFor(key: string): S | undefined {
    const known = this.byKey.get(key);
    if (known && this.live().includes(known)) return known;
    this.byKey.delete(key);
    const found = this.live().find((o) => attachment(o)?.key === key);
    if (found) this.byKey.set(key, found);
    return found;
  }

  /** Close `ws` and persist `closed: true` in its attachment regardless of
   * `readyState`, so delayed close/message events after hibernation know
   * this object already ended or replaced it. */
  private shut(ws: S, code: number, reason: string): void {
    this.closed.add(ws);
    try {
      const att = attachment(ws);
      if (att && !att.closed) {
        att.closed = true;
        ws.serializeAttachment(att);
      }
    } catch {
      // Attachment unavailable on a vanished socket.
    }
    try {
      ws.close(code, reason);
    } catch {
      // Already closed.
    }
  }

  /** Accept a newly paired server socket for `name`; returns HTTP status
   * (`101` on success, `400` on a bad pair name, `403` on invalid
   * `X-TotalDrop-Auth` or an already-over-limit pair, `409` when slots are
   * full, #78). */
  async accept(
    name: string,
    makeServer: () => S,
    now?: number,
    auth?: ConnectAuth,
  ): Promise<101 | 400 | 403 | 409> {
    if (!parsePairName(name)) return 400;
    let preauthKey: string | null = null;
    if (auth?.header !== null && auth?.header !== undefined) {
      preauthKey = await verifyConnectAuth(name, auth.header);
      if (preauthKey === null) return 403;
    }
    if (this.expiring !== null) await this.waitExpiring();
    // Re-evaluate time and live state strictly after any await.
    let effectiveNow = now ?? Date.now();
    this.seedPreDeployTotals(effectiveNow);
    if (this.totals.isExpired(effectiveNow, this.sessionTime())) {
      await this.expireSession();
      if (this.expiring !== null) await this.waitExpiring();
      effectiveNow = now ?? Date.now();
      this.seedPreDeployTotals(effectiveNow);
    }
    if (preauthKey !== null) {
      if (this.totals.isOver(this.sessionBytes())) return 403;
      const sockets = this.host.getWebSockets();
      if (sockets.length >= MAX_SOCKETS) {
        const authenticatedSockets = sockets.filter((o) => {
          const att = attachment(o);
          return att !== null && (att.key !== null || att.preauth !== undefined);
        }).length;
        if (authenticatedSockets >= MAX_SOCKETS) return 409;
      }
      // Deduplicate any prior waiting socket pre-authenticated by the same key.
      for (const o of this.live()) {
        const att = attachment(o);
        if (att?.key === null && att.preauth === preauthKey) {
          this.shut(o, 1000, "replaced");
        }
      }
      if (this.host.getWebSockets().length >= MAX_SOCKETS) {
        let oldest: { ws: S; since: number } | null = null;
        for (const o of this.live()) {
          const att = attachment(o);
          if (att?.key === null && att.preauth === undefined) {
            if (oldest === null || att.since < oldest.since) {
              oldest = { ws: o, since: att.since };
            }
          }
        }
        if (oldest !== null) {
          this.shut(oldest.ws, 1008, "evicted");
        }
      }
    } else {
      const waiting = this.live().filter((o) => attachment(o)?.key === null).length;
      if (waiting >= MAX_WAITING || this.host.getWebSockets().length >= MAX_SOCKETS) return 409;
    }
    const server = makeServer();
    const challenge = crypto.getRandomValues(new Uint8Array(32));
    this.host.acceptWebSocket(server, [name]);
    const att: Attachment =
      preauthKey !== null
        ? { challenge: hex(challenge), key: null, since: effectiveNow, preauth: preauthKey }
        : { challenge: hex(challenge), key: null, since: effectiveNow };
    server.serializeAttachment(att);
    server.send(new Uint8Array([...MAGIC, ...challenge]));
    await this.schedule(effectiveNow);
    return 101;
  }

  private pairOf(ws: S): [string, string] | null {
    const tag = this.host.getTags(ws)[0];
    return tag ? parsePairName(tag) : null;
  }

  private sideOf(ws: S, key: string): Side | null {
    const pair = this.pairOf(ws);
    if (!pair) return null;
    return key === pair[0] ? 0 : 1;
  }

  /** Seed `"totals"` from any pre-existing signed-in sockets if waking across
   * a rolling deploy before `"totals"` was written to KV. */
  private seedPreDeployTotals(now: number): void {
    if (this.totals.deadline(this.sessionTime()) !== null) return;
    let earliest = Infinity;
    const seed: [number, number] = [0, 0];
    for (const o of this.live()) {
      const att = attachment(o);
      if (!att || att.key === null) continue;
      earliest = Math.min(earliest, att.since);
      const side = this.sideOf(o, att.key);
      if (side !== null && typeof att.sent === "number" && att.sent > seed[side]) {
        seed[side] = att.sent;
      }
    }
    if (earliest !== Infinity) {
      this.totals.ensure(Math.min(earliest, now), seed, this.sessionBytes());
    }
  }

  async onMessage(ws: S, data: string | ArrayBuffer, now?: number): Promise<void> {
    if (this.expiring !== null) await this.waitExpiring();
    const entryNow = now ?? Date.now();
    const att0 = attachment(ws);
    // A socket this object closed or replaced may still send before tearing
    // down, including after hibernation: ignore it.
    if (ws.readyState !== OPEN || this.isClosed(ws, att0)) return;
    this.seedPreDeployTotals(entryNow);
    if (this.totals.isExpired(entryNow, this.sessionTime())) {
      await this.expireSession();
      return;
    }
    const m = bytes(data);
    const own = this.keyOf(ws);
    if (!m || m.length > MAX_MESSAGE) return this.fail(ws, own, "bad message");
    if (own !== null && m.length === PROBE_LENGTH && startsWith(m, PROBE)) return this.answered(ws, m);
    if (own !== null) return this.forward(ws, own, m, att0?.since ?? entryNow);

    const att = attachment(ws);
    const pair = this.pairOf(ws);
    if (!att || !pair) return this.shut(ws, 1008, "bad message");

    // Sign in: TDR0 || key || signature over context || challenge.
    if (m.length !== 4 + 32 + 64 || !startsWith(m, MAGIC)) return this.shut(ws, 1008, "bad hello");
    const key = m.slice(4, 36);
    const keyHex = hex(key);
    if (keyHex !== pair[0] && keyHex !== pair[1]) return this.shut(ws, 1008, "not this pair");
    if (att.preauth !== undefined && keyHex !== att.preauth) {
      return this.shut(ws, 1008, "not this pair");
    }
    const signed = new Uint8Array([...AUTH_CONTEXT, ...unhex(att.challenge)]);
    const ok = await verifySignature(key, m.slice(36), signed);
    if (this.expiring !== null) await this.waitExpiring();
    // Other events can run during verifySignature() or expireSession(); re-check socket state.
    if (ws.readyState !== OPEN || this.isClosed(ws) || this.keyOf(ws) !== null) return;
    if (!ok) return this.shut(ws, 1008, "bad signature");
    const effectiveNow = now ?? Date.now();
    this.seedPreDeployTotals(effectiveNow);
    if (this.totals.isExpired(effectiveNow, this.sessionTime())) {
      await this.expireSession();
      return;
    }
    if (!this.totals.admitSignIn(effectiveNow, this.sessionTime(), this.sessionBytes())) {
      this.shut(ws, 1008, "limit reached");
      await this.schedule(effectiveNow);
      return;
    }
    // The holder of the key signed in again: its old socket died without
    // this object noticing (a Wi-Fi switch, NAT rebinding). The new one
    // takes over at once, before QUIC gives up (#190). Closed through
    // shut(), the old socket gets `closed: true` in its attachment so even
    // after hibernation its delayed close event takes nobody with it.
    const taken = this.socketFor(keyHex);
    if (taken && taken !== ws) {
      this.totals.flushSide(keyHex === pair[0] ? 0 : 1);
      this.shut(taken, 1000, "replaced");
    }
    this.stale.delete(keyHex);
    const freshAtt = attachment(ws) ?? att;
    freshAtt.key = keyHex;
    ws.serializeAttachment(freshAtt);
    this.byKey.set(keyHex, ws);
    ws.send(OK);
    await this.schedule(effectiveNow);
  }

  /** The signed-in key of `ws`, from memory or its attachment. */
  private keyOf(ws: S): string | null {
    for (const [k, o] of this.byKey) if (o === ws) return k;
    const att = attachment(ws);
    if (!att || att.key === null || this.isClosed(ws, att)) return null;
    this.byKey.set(att.key, ws);
    return att.key;
  }

  /** Datagrams: destination key || (length || datagram)+. Only the other key
   * of the pair is a valid destination; the rest passes through. */
  private forward(ws: S, own: string, m: Uint8Array, fallbackStart: number): void {
    const pair = this.pairOf(ws);
    if (!pair || m.length <= 32 + 2) return this.fail(ws, own, "bad frame");
    const side: Side = own === pair[0] ? 0 : 1;
    const other = side === 0 ? pair[1] : pair[0];
    if (hex(m.subarray(0, 32)) !== other) return this.fail(ws, own, "bad destination");
    if (!this.totals.spend(side, m.length, this.sessionBytes(), fallbackStart)) {
      return this.fail(ws, own, "limit reached");
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
      this.gave(peer, given + m.length);
    } catch {
      // The other side is gone; its close handler tidies up.
    }
  }

  /** Bytes given to `ws`, and given at its last answered probe. */
  private counts(ws: S): [number, number] {
    let given = this.given.get(ws);
    let answered = this.answeredAt.get(ws);
    if (given === undefined || answered === undefined) {
      const r = restore(attachment(ws));
      given = r.given;
      answered = r.answered;
      this.given.set(ws, given);
      this.answeredAt.set(ws, answered);
      if (r.written !== undefined) this.written.set(ws, r.written);
    }
    return [given, answered];
  }

  /** Unanswered bytes on the other sockets of `key` than `live`. */
  private staleFor(key: string, live: S): number {
    const known = this.stale.get(key);
    if (known !== undefined) return known;
    let sum = 0;
    for (const o of this.host.getWebSockets()) {
      if (o === live || attachment(o)?.key !== key) continue;
      const [given, answered] = this.counts(o);
      sum += given - answered;
    }
    this.stale.set(key, sum);
    return sum;
  }

  /** Count `after` bytes given to `ws` in all; at its first bytes and then
   * every PROBE_STEP, a probe goes out after them (count.ts). */
  private gave(ws: S, after: number): void {
    this.given.set(ws, after);
    const step = give(this.written.get(ws), after, () => attachment(ws));
    if (step.written !== undefined) this.written.set(ws, step.written);
    const att = step.att;
    if (!step.probe || !att) return;
    const nonce = crypto.getRandomValues(new Uint8Array(16));
    ws.send(new Uint8Array([...PROBE, ...nonce]));
    att.probes = [...(att.probes ?? []), [hex(nonce), after]];
    ws.serializeAttachment(att);
  }

  /** A probe came back: everything given before it was read. Only a nonce
   * this object sent counts; any other answer is ignored, and none is
   * forwarded or counted against SESSION_BYTES. */
  private answered(ws: S, m: Uint8Array): void {
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

  /** This object ends `ws` for breaking a rule. A signed-in socket takes
   * the other side with it, as when it leaves (see end()). */
  private fail(ws: S, own: string | null, reason: string): void {
    if (own !== null) {
      const active = this.socketFor(own);
      if (active === undefined || active === ws) {
        const side = this.sideOf(ws, own);
        if (side !== null) this.totals.flushSide(side);
      }
    }
    this.shut(ws, 1008, reason);
    if (own !== null) {
      const active = this.socketFor(own);
      if (active === undefined || active === ws) {
        if (this.byKey.get(own) === ws) this.byKey.delete(own);
        this.endOthers(ws);
      }
    }
  }

  /** Close the other signed-in socket; sockets still signing in stay (one
   * may be the other side's reconnect), as on the Rust relay. */
  private endOthers(ws: S): void {
    for (const o of this.live()) {
      const key = attachment(o)?.key ?? null;
      if (o === ws || key === null) continue;
      const side = this.sideOf(o, key);
      if (side !== null) this.totals.flushSide(side);
      this.shut(o, 1000, "peer left");
      this.byKey.delete(key);
    }
  }

  /** When the active signed-in socket for a key leaves, the transfer is over:
   * close the other too. A socket that never signed in, one this object
   * already closed/replaced (`this.isClosed`), or a stale socket superseded
   * by a newer replacement (`this.socketFor(att.key) !== ws`) takes nobody
   * with it and never touches the replacement's totals. */
  async end(ws: S): Promise<void> {
    if (this.expiring !== null) {
      if (!this.isClosed(ws)) this.shut(ws, 1000, "closed");
      await this.waitExpiring();
      await this.schedule();
      return;
    }
    const att = attachment(ws);
    const alreadyClosed = this.isClosed(ws, att);
    if (att?.key) {
      // Recompute remaining stale unanswered bytes without this socket.
      this.stale.delete(att.key);
    }
    if (alreadyClosed) {
      if (att?.key && this.byKey.get(att.key) === ws) this.byKey.delete(att.key);
      this.shut(ws, 1000, "closed");
      await this.schedule();
      return;
    }
    const active = att?.key ? this.socketFor(att.key) : undefined;
    const isCurrent = att !== null && att.key !== null && (active === undefined || active === ws);
    if (isCurrent && att?.key) {
      const side = this.sideOf(ws, att.key);
      if (side !== null) this.totals.flushSide(side);
    }
    this.shut(ws, 1000, "closed");
    if (isCurrent && att?.key) {
      if (this.byKey.get(att.key) === ws) this.byKey.delete(att.key);
      this.endOthers(ws);
    } else if (att?.key && this.byKey.get(att.key) === ws) {
      this.byKey.delete(att.key);
    }
    await this.schedule();
  }

  /** End the pair's session window and erase all stored state (#190). */
  async expireSession(): Promise<void> {
    if (this.expiring !== null) return this.expiring;
    let settled = false;
    const run = (async () => {
      try {
        for (const ws of this.host.getWebSockets()) {
          if (!this.isClosed(ws)) this.shut(ws, 1008, "session over");
        }
        for (const ws of this.byKey.values()) {
          if (!this.isClosed(ws)) this.shut(ws, 1008, "session over");
        }
        this.byKey.clear();
        this.stale.clear();
        this.totals.clear();
        await this.host.storage.deleteAll();
        await this.host.storage.deleteAlarm();
      } finally {
        settled = true;
        this.expiring = null;
      }
    })();
    if (!settled) this.expiring = run;
    return run;
  }

  /** Set the one alarm to the earliest sign-in timeout or the pair's session
   * deadline, so a sign-in does not push back another socket's sign-in
   * timeout and stored pair totals are always deleted when the window ends. */
  async schedule(now?: number): Promise<void> {
    if (this.expiring !== null) await this.waitExpiring();
    const effectiveNow = now ?? Date.now();
    this.seedPreDeployTotals(effectiveNow);
    let next = this.totals.deadline(this.sessionTime()) ?? Infinity;
    for (const ws of this.live()) {
      const att = attachment(ws);
      if (att && att.key === null) next = Math.min(next, att.since + SIGN_IN_TIMEOUT);
    }
    if (next === Infinity) await this.host.storage.deleteAlarm();
    else await this.host.storage.setAlarm(next);
  }

  /** Close sockets that did not sign in in time, or end the pair's session
   * window and delete its stored totals. */
  async alarm(now?: number): Promise<void> {
    if (this.expiring !== null) await this.waitExpiring();
    const effectiveNow = now ?? Date.now();
    this.seedPreDeployTotals(effectiveNow);
    if (this.totals.isExpired(effectiveNow, this.sessionTime())) {
      await this.expireSession();
      return;
    }
    for (const ws of this.live()) {
      const att = attachment(ws);
      if (att && att.key === null && effectiveNow >= att.since + SIGN_IN_TIMEOUT) {
        this.shut(ws, 1008, "sign-in timeout");
      }
    }
    await this.schedule(effectiveNow);
  }
}
