// SPDX-License-Identifier: AGPL-3.0-or-later
//
// One mailbox: at most two sides, opaque messages between them, and the
// rules that limit an attacker to one guess per code. Each mailbox is its
// own Durable Object, named by its random mailbox ID, and erases all of its
// storage when both sides have left or when it expires.

import { DurableObject } from "cloudflare:workers";

import type { Directory } from "./directory";
import { LIMITS } from "./directory";
import {
  decodeClient,
  hex,
  type Mood,
  randomBytes,
  sameBytes,
  server,
  unhex,
} from "./messages";

/** Most connections a mailbox holds that have not claimed a side yet. A
 * newer one pushes out the oldest, so idle connections can delay a real
 * claim by a moment at most, never keep it out. */
const MAX_UNCLAIMED = 2;
/** WebSocket readyState OPEN (the runtime has no WebSocket.OPEN). */
const OPEN = 1;
/** Most sockets at all, lingering closed ones included. */
const MAX_SOCKETS = 8;
/** A connection that has not claimed a side by then is closed, as on the
 * Rust server; otherwise anyone who knows a nameplate could hold
 * connections on someone else's mailbox until it expires. */
const CLAIM_TIMEOUT = 10_000;

/** Most messages one side may post. */
const MAX_MESSAGES_PER_SIDE = 32;
/** Most messages and bytes buffered for a side that has not arrived yet. */
const MAX_PENDING_MESSAGES = 8;
const MAX_PENDING_BYTES = 32 * 1024;

export interface Env {
  DIRECTORY: DurableObjectNamespace<Directory>;
}

type Attachment = { conn: string; side: number | null; opened?: number };

type Meta = { nameplate: number; mailbox: string; token: string; created: number };

class Refusal extends Error {}

export class Mailbox extends DurableObject<Env> {
  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
    ctx.blockConcurrencyWhile(async () => this.schema());
  }

  private schema(): void {
    this.ctx.storage.sql.exec(`
        CREATE TABLE IF NOT EXISTS meta (
          id INTEGER PRIMARY KEY CHECK (id = 0),
          nameplate INTEGER NOT NULL,
          mailbox TEXT NOT NULL,
          token TEXT NOT NULL,
          created INTEGER NOT NULL,
          released INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS sides (
          side INTEGER PRIMARY KEY,
          conn TEXT NOT NULL,
          posted INTEGER NOT NULL DEFAULT 0,
          closed INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS pending (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          from_side INTEGER NOT NULL,
          phase TEXT NOT NULL,
          body BLOB NOT NULL
        );
      `);
  }

  private meta(): Meta | null {
    return (
      this.ctx.storage.sql
        .exec<Meta>("SELECT nameplate, mailbox, token, created FROM meta WHERE id = 0")
        .toArray()[0] ?? null
    );
  }

  private sides(): Array<{ side: number; conn: string; posted: number; closed: number }> {
    return this.ctx.storage.sql
      .exec<{ side: number; conn: string; posted: number; closed: number }>(
        "SELECT side, conn, posted, closed FROM sides ORDER BY side",
      )
      .toArray();
  }

  private socketOf(side: number): WebSocket | null {
    for (const ws of this.ctx.getWebSockets()) {
      const a = ws.deserializeAttachment() as Attachment | null;
      if (a && a.side === side) return ws;
    }
    return null;
  }

  private directory() {
    return this.env.DIRECTORY.getByName("directory");
  }

  /** The Worker forwards the WebSocket upgrade here, with the nameplate's
   * record from the directory in trusted headers. */
  async fetch(request: Request): Promise<Response> {
    const pair = new WebSocketPair();
    const [client, socket] = [pair[0], pair[1]];
    let meta = this.meta();
    if (!meta) {
      const nameplate = Number(request.headers.get("x-td-nameplate"));
      const mailbox = request.headers.get("x-td-mailbox") ?? "";
      const token = request.headers.get("x-td-token") ?? "";
      const created = Number(request.headers.get("x-td-created"));
      if (!Number.isInteger(nameplate) || !mailbox || !token || !Number.isFinite(created)) {
        return new Response("bad forward", { status: 500 });
      }
      this.ctx.storage.sql.exec(
        "INSERT INTO meta (id, nameplate, mailbox, token, created) VALUES (0, ?, ?, ?, ?)",
        nameplate,
        mailbox,
        token,
        created,
      );
      meta = { nameplate, mailbox, token, created };
      await this.ctx.storage.setAlarm(created + LIMITS.unclaimedTtl);
    }
    const now = Date.now();
    const all = this.ctx.getWebSockets();
    // Closed sockets a client keeps from going away still use the object.
    if (all.length >= MAX_SOCKETS) {
      socket.accept();
      this.refuse(socket, "mailbox busy");
      return new Response(null, { status: 101, webSocket: client });
    }
    const waiting = all
      .map((ws) => ({ ws, a: ws.deserializeAttachment() as Attachment | null }))
      .filter(({ ws, a }) => ws.readyState === OPEN && (!a || a.side === null))
      .sort((x, y) => (x.a?.opened ?? 0) - (y.a?.opened ?? 0));
    for (const { ws: old } of waiting.slice(0, Math.max(0, waiting.length - MAX_UNCLAIMED + 1))) {
      try {
        old.close(1000, "replaced");
      } catch {
        // Gone already.
      }
    }
    const attachment: Attachment = { conn: hex(randomBytes(8)), side: null, opened: now };
    this.ctx.acceptWebSocket(socket);
    socket.serializeAttachment(attachment);
    await this.wakeBy(now + CLAIM_TIMEOUT);
    return new Response(null, { status: 101, webSocket: client });
  }

  /** Make sure the alarm runs by `when` (it may already run earlier). */
  private async wakeBy(when: number): Promise<void> {
    const at = await this.ctx.storage.getAlarm();
    if (at === null || at > when) await this.ctx.storage.setAlarm(when);
  }

  /** Close connections that have not claimed a side in time; returns when
   * the next one would be due, if any. */
  private closeIdle(now: number): number | null {
    let next: number | null = null;
    for (const ws of this.ctx.getWebSockets()) {
      const a = ws.deserializeAttachment() as Attachment | null;
      if (ws.readyState !== OPEN || !a || a.side !== null) continue;
      // A connection from before this field existed is due at once.
      const due = (a.opened ?? 0) + CLAIM_TIMEOUT;
      if (due <= now) {
        try {
          ws.close(1000, "claim timeout");
        } catch {
          // Gone already.
        }
      } else if (next === null || due < next) {
        next = due;
      }
    }
    return next;
  }

  private refuse(ws: WebSocket, reason: string): void {
    try {
      ws.send(server.error(reason));
      ws.close(1000, "refused");
    } catch {
      // Already gone.
    }
  }

  async webSocketMessage(ws: WebSocket, data: ArrayBuffer | string): Promise<void> {
    if (typeof data === "string") return this.refuse(ws, "malformed message");
    let msg;
    try {
      msg = decodeClient(
        ArrayBuffer.isView(data)
          ? new Uint8Array(data.buffer, data.byteOffset, data.byteLength)
          : new Uint8Array(data),
      );
    } catch {
      return this.refuse(ws, "malformed message");
    }
    try {
      switch (msg.type) {
        case "allocate":
          throw new Refusal("allocate on /v0/allocate");
        case "claim":
          return await this.claim(ws, msg.nameplate, msg.token);
        case "add":
          return await this.add(ws, msg.phase, msg.body);
        case "close": {
          const a = ws.deserializeAttachment() as Attachment;
          if (a.side === null) throw new Refusal("claim first");
          await this.leave(a.side, msg.mood);
          ws.close(1000, "closed");
          return;
        }
      }
    } catch (e) {
      if (e instanceof Refusal) return this.refuse(ws, e.message);
      throw e;
    }
  }

  private async claim(ws: WebSocket, nameplate: number, token: Uint8Array | null): Promise<void> {
    const a = ws.deserializeAttachment() as Attachment;
    const meta = this.meta();
    if (!meta) throw new Refusal("unknown nameplate");
    if (a.side !== null || meta.nameplate !== nameplate) {
      throw new Refusal("claim the mailbox of this address once");
    }
    const sides = this.sides();
    if (sides.length >= 2) throw new Refusal("nameplate full");
    if (sides.length === 0 && !(token && sameBytes(token, unhex(meta.token)))) {
      // The allocator always claims first, with its token; anyone else
      // claiming an empty nameplate is guessing.
      throw new Refusal("unknown nameplate");
    }
    const side = sides.length;
    const sql = this.ctx.storage.sql;
    sql.exec("INSERT INTO sides (side, conn) VALUES (?, ?)", side, a.conn);
    ws.serializeAttachment({ ...a, side } satisfies Attachment);
    ws.send(server.claimed(unhex(meta.mailbox)));
    // Hand over anything the other side posted before we came.
    const pending = sql
      .exec<{ id: number; phase: string; body: ArrayBuffer }>(
        "SELECT id, phase, body FROM pending WHERE from_side != ? ORDER BY id",
        side,
      )
      .toArray();
    for (const p of pending) ws.send(server.message(p.phase, new Uint8Array(p.body)));
    sql.exec("DELETE FROM pending WHERE from_side != ?", side);
    if (side === 1) {
      await this.directory().markFull(meta.nameplate, meta.mailbox);
      await this.ctx.storage.setAlarm(meta.created + LIMITS.mailboxTtl);
      // Still owed: the claim deadline of any connection that has not claimed.
      const idle = this.closeIdle(Date.now());
      if (idle !== null) await this.wakeBy(idle);
    }
  }

  private async add(ws: WebSocket, phase: string, body: Uint8Array): Promise<void> {
    const a = ws.deserializeAttachment() as Attachment;
    if (a.side === null) throw new Refusal("claim first");
    const sql = this.ctx.storage.sql;
    const me = this.sides().find((s) => s.side === a.side);
    if (!me) throw new Refusal("claim first");
    if (me.closed) throw new Refusal("mailbox closed");
    if (me.posted + 1 > MAX_MESSAGES_PER_SIDE) throw new Refusal("too many messages");
    sql.exec("UPDATE sides SET posted = posted + 1 WHERE side = ?", a.side);
    const other = 1 - a.side;
    const peer = this.sides().find((s) => s.side === other);
    const peerWs = peer && !peer.closed ? this.socketOf(other) : null;
    if (peerWs) {
      try {
        peerWs.send(server.message(phase, body));
      } catch {
        // A vanished peer must not stall us.
      }
      return;
    }
    const totals = sql
      .exec<{ c: number; b: number }>(
        "SELECT count(*) AS c, coalesce(sum(length(body)), 0) AS b FROM pending",
      )
      .one();
    if (totals.c >= MAX_PENDING_MESSAGES || totals.b + body.length > MAX_PENDING_BYTES) {
      throw new Refusal("mailbox full");
    }
    sql.exec(
      "INSERT INTO pending (from_side, phase, body) VALUES (?, ?, ?)",
      a.side,
      phase,
      body,
    );
  }

  /** A side leaves, explicitly or by disconnecting. */
  private async leave(side: number, mood: Mood): Promise<void> {
    const meta = this.meta();
    if (!meta) return;
    const sql = this.ctx.storage.sql;
    const sides = this.sides();
    const me = sides.find((s) => s.side === side);
    if (!me || me.closed) return;
    sql.exec("UPDATE sides SET closed = 1 WHERE side = ?", side);
    const other = sides.find((s) => s.side !== side);
    if (other && !other.closed) {
      const peerWs = this.socketOf(other.side);
      try {
        peerWs?.send(server.peerClosed(mood));
      } catch {
        // Gone already.
      }
    }
    // A scary close only counts when two sides really exchanged messages;
    // otherwise anyone could retire nameplates at will.
    const exchanged = sides.length === 2 && sides.every((s) => s.posted > 0);
    const scary = mood === "scary" && exchanged;
    const allClosed = !other || other.closed;
    if (scary || allClosed) sql.exec("DELETE FROM pending");
    if (scary) await this.release(meta, true);
    if (allClosed) await this.finish(meta, null);
  }

  private async release(meta: Meta, retire: boolean): Promise<void> {
    const sql = this.ctx.storage.sql;
    const done = sql.exec<{ released: number }>("SELECT released FROM meta WHERE id = 0").one();
    if (done.released) return;
    sql.exec("UPDATE meta SET released = 1 WHERE id = 0");
    await this.directory().release(meta.nameplate, meta.mailbox, retire);
  }

  /** Tell everyone still here, close their sockets, release the nameplate
   * and erase every trace of the mailbox. */
  private async finish(meta: Meta, reason: string | null): Promise<void> {
    for (const ws of this.ctx.getWebSockets()) {
      try {
        if (reason) ws.send(server.error(reason));
        ws.close(1000, reason ?? "closed");
      } catch {
        // Gone already.
      }
    }
    await this.release(meta, false);
    await this.ctx.storage.deleteAlarm();
    await this.ctx.storage.deleteAll();
    this.schema();
  }

  async webSocketClose(ws: WebSocket): Promise<void> {
    const a = ws.deserializeAttachment() as Attachment | null;
    // A client that disappears without CLOSE counts as a lonely close.
    if (a && a.side !== null) await this.leave(a.side, "lonely");
  }

  async webSocketError(ws: WebSocket): Promise<void> {
    await this.webSocketClose(ws);
  }

  async alarm(): Promise<void> {
    const meta = this.meta();
    if (!meta) return;
    const now = Date.now();
    const full = this.sides().length >= 2;
    const deadline = meta.created + (full ? LIMITS.mailboxTtl : LIMITS.unclaimedTtl);
    if (now >= deadline) {
      await this.finish(meta, "mailbox expired");
      return;
    }
    const idle = this.closeIdle(now);
    await this.ctx.storage.setAlarm(idle !== null && idle < deadline ? idle : deadline);
  }
}
