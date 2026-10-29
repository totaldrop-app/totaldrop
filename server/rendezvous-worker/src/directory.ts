// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The nameplate directory: which small numbers are in use, retired after a
// failed code, or resting before reuse. It holds no mailbox contents and no
// client addresses; mailboxes live in their own Durable Objects.

import { DurableObject } from "cloudflare:workers";

import { hex, randomBytes } from "./messages";

export const LIMITS = {
  /** An allocated nameplate waits this long for its second claim. */
  unclaimedTtl: 15 * 60_000,
  /** Hard cap on the life of any mailbox. */
  mailboxTtl: 60 * 60_000,
  /** A nameplate stays retired this long after a counted scary close. */
  retireFor: 24 * 60 * 60_000,
  /** A freed nameplate rests this long before reuse, while others are free. */
  cooldown: 10 * 60_000,
  /** Most nameplates in use at once. */
  maxNameplates: 100_000,
  /** Scary closes within this window that count as an attack wave… */
  scaryWindow: 10 * 60_000,
  scaryThreshold: 100,
  /** …delay every allocation by this much. */
  waveDelay: 2_000,
};

export type Allocation = { nameplate: number; token: string; mailbox: string };
export type Lookup = { mailbox: string; token: string; created: number };

export class Directory extends DurableObject {
  constructor(ctx: DurableObjectState, env: Cloudflare.Env) {
    super(ctx, env);
    ctx.blockConcurrencyWhile(async () => this.schema());
  }

  /** Every request costs the same however many nameplates are in use: no
   * query here reads more rows than it changes or returns. Numbers come
   * from `pool` (freed numbers, lowest first) or, when it is empty, from
   * one past the highest number handed out so far; `state` keeps that mark
   * and the count of active nameplates. */
  private schema(): void {
    const sql = this.ctx.storage.sql;
    sql.exec(`
      CREATE TABLE IF NOT EXISTS active (
        n INTEGER PRIMARY KEY,
        mailbox TEXT NOT NULL,
        token TEXT NOT NULL,
        created INTEGER NOT NULL,
        full INTEGER NOT NULL DEFAULT 0
      );
      CREATE TABLE IF NOT EXISTS resting (
        n INTEGER PRIMARY KEY,
        until INTEGER NOT NULL,
        retired INTEGER NOT NULL
      );
      CREATE TABLE IF NOT EXISTS scary (t INTEGER NOT NULL);
      CREATE TABLE IF NOT EXISTS pool (n INTEGER PRIMARY KEY);
      CREATE INDEX IF NOT EXISTS active_full_created ON active (full, created);
      CREATE INDEX IF NOT EXISTS resting_until ON resting (until);
      CREATE INDEX IF NOT EXISTS resting_cooling ON resting (retired, n);
      CREATE INDEX IF NOT EXISTS scary_t ON scary (t);
    `);
    const has = sql
      .exec<{ c: number }>("SELECT count(*) AS c FROM sqlite_master WHERE name = 'state'")
      .one().c;
    if (has) return;
    // First start with this layout: rebuild the mark, the count and the
    // pool once from the tables an earlier version left.
    sql.exec(`
      CREATE TABLE state (
        id INTEGER PRIMARY KEY CHECK (id = 0),
        high INTEGER NOT NULL,
        active INTEGER NOT NULL
      );
      INSERT INTO state (id, high, active) VALUES (0,
        coalesce((SELECT max(n) FROM (SELECT n FROM active UNION ALL SELECT n FROM resting)), 0),
        (SELECT count(*) FROM active));
      WITH RECURSIVE k(n) AS (
        SELECT 1 WHERE (SELECT high FROM state) >= 1
        UNION ALL SELECT n + 1 FROM k WHERE n < (SELECT high FROM state)
      )
      INSERT INTO pool (n) SELECT n FROM k
        WHERE n NOT IN (SELECT n FROM active) AND n NOT IN (SELECT n FROM resting);
    `);
  }

  private now(): number {
    return Date.now();
  }

  private state(): { high: number; active: number } {
    return this.ctx.storage.sql
      .exec<{ high: number; active: number }>("SELECT high, active FROM state WHERE id = 0")
      .one();
  }

  /** Move expired nameplates to resting. Each branch is an index range that
   * holds only rows that really expired. */
  private expireActive(now: number): void {
    const sql = this.ctx.storage.sql;
    const expired = sql
      .exec<{ n: number }>(
        `SELECT n FROM active WHERE full = 0 AND created < ?1
         UNION ALL
         SELECT n FROM active WHERE full = 1 AND created < ?2`,
        now - LIMITS.unclaimedTtl,
        now - LIMITS.mailboxTtl,
      )
      .toArray();
    for (const { n } of expired) {
      sql.exec("DELETE FROM active WHERE n = ?", n);
      sql.exec(
        "INSERT OR REPLACE INTO resting (n, until, retired) VALUES (?, ?, 0)",
        n,
        now + LIMITS.cooldown,
      );
    }
    if (expired.length) {
      sql.exec("UPDATE state SET active = active - ? WHERE id = 0", expired.length);
    }
  }

  /** Numbers whose rest is over go back to the pool. */
  private freeRested(now: number): void {
    const sql = this.ctx.storage.sql;
    sql.exec(
      `INSERT OR IGNORE INTO pool (n) SELECT n FROM resting
       WHERE until <= ? AND n NOT IN (SELECT n FROM active)`,
      now,
    );
    sql.exec("DELETE FROM resting WHERE until <= ?", now);
  }

  private async ensureAlarm(): Promise<void> {
    if ((await this.ctx.storage.getAlarm()) === null) {
      await this.ctx.storage.setAlarm(this.now() + 60_000);
    }
  }

  async allocate(): Promise<Allocation | { error: string }> {
    const sql = this.ctx.storage.sql;
    let now = this.now();
    const scary = sql
      .exec<{ c: number }>("SELECT count(*) AS c FROM scary WHERE t > ?", now - LIMITS.scaryWindow)
      .one().c;
    if (scary >= LIMITS.scaryThreshold) {
      await new Promise((r) => setTimeout(r, LIMITS.waveDelay));
      now = this.now();
    }
    this.expireActive(now);
    this.freeRested(now);
    const state = this.state();
    if (state.active >= LIMITS.maxNameplates) return { error: "server busy" };
    // The lowest freed number, else a new one, else (when all are in use or
    // resting) the lowest that is only cooling down, never a retired one.
    let n = sql.exec<{ n: number }>("SELECT n FROM pool ORDER BY n LIMIT 1").toArray()[0]?.n ?? 0;
    if (n) {
      sql.exec("DELETE FROM pool WHERE n = ?", n);
    } else if (state.high < LIMITS.maxNameplates * 4) {
      n = state.high + 1;
      sql.exec("UPDATE state SET high = ? WHERE id = 0", n);
    } else {
      n =
        sql
          .exec<{ n: number }>("SELECT n FROM resting WHERE retired = 0 ORDER BY n LIMIT 1")
          .toArray()[0]?.n ?? 0;
      if (!n) return { error: "server busy" };
      sql.exec("DELETE FROM resting WHERE n = ?", n);
    }
    const token = hex(randomBytes(16));
    const mailbox = hex(randomBytes(16));
    sql.exec(
      "INSERT INTO active (n, mailbox, token, created) VALUES (?, ?, ?, ?)",
      n,
      mailbox,
      token,
      now,
    );
    sql.exec("UPDATE state SET active = active + 1 WHERE id = 0");
    await this.ensureAlarm();
    return { nameplate: n, token, mailbox };
  }

  /** The live mailbox of nameplate `n`. An expired row is not returned;
   * allocation and the alarm move it to resting. */
  async lookup(n: number): Promise<Lookup | null> {
    const now = this.now();
    const row = this.ctx.storage.sql
      .exec<{ mailbox: string; token: string; created: number }>(
        `SELECT mailbox, token, created FROM active
         WHERE n = ? AND created >= ? AND (full = 1 OR created >= ?)`,
        n,
        now - LIMITS.mailboxTtl,
        now - LIMITS.unclaimedTtl,
      )
      .toArray()[0];
    return row ?? null;
  }

  async markFull(n: number, mailbox: string): Promise<void> {
    this.ctx.storage.sql.exec("UPDATE active SET full = 1 WHERE n = ? AND mailbox = ?", n, mailbox);
  }

  async release(n: number, mailbox: string, retire: boolean): Promise<void> {
    const sql = this.ctx.storage.sql;
    const now = this.now();
    const row = sql
      .exec<{ n: number }>("SELECT n FROM active WHERE n = ? AND mailbox = ?", n, mailbox)
      .toArray()[0];
    if (!row && !retire) return;
    // Already handed out again: the new holder's code is not touched.
    if (!row && sql.exec("SELECT 1 FROM active WHERE n = ?", n).toArray().length) return;
    if (row) {
      sql.exec("DELETE FROM active WHERE n = ? AND mailbox = ?", n, mailbox);
      sql.exec("UPDATE state SET active = active - 1 WHERE id = 0");
    }
    // A retired number must not be handed out again from the pool either.
    sql.exec("DELETE FROM pool WHERE n = ?", n);
    sql.exec(
      "INSERT OR REPLACE INTO resting (n, until, retired) VALUES (?, ?, ?)",
      n,
      now + (retire ? LIMITS.retireFor : LIMITS.cooldown),
      retire ? 1 : 0,
    );
    if (retire) sql.exec("INSERT INTO scary (t) VALUES (?)", now);
  }

  async alarm(): Promise<void> {
    const sql = this.ctx.storage.sql;
    const now = this.now();
    this.expireActive(now);
    this.freeRested(now);
    sql.exec("DELETE FROM scary WHERE t <= ?", now - LIMITS.scaryWindow);
    const left = sql
      .exec<{ x: number }>(
        "SELECT EXISTS (SELECT 1 FROM active) OR EXISTS (SELECT 1 FROM resting) AS x",
      )
      .one().x;
    if (left) await this.ctx.storage.setAlarm(now + 60_000);
  }
}
