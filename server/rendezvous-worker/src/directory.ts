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
    ctx.blockConcurrencyWhile(async () => {
      this.ctx.storage.sql.exec(`
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
        CREATE INDEX IF NOT EXISTS active_created ON active (created);
        CREATE INDEX IF NOT EXISTS resting_until ON resting (until);
        CREATE INDEX IF NOT EXISTS scary_t ON scary (t);
      `);
    });
  }

  private now(): number {
    return Date.now();
  }

  /** Move expired nameplates to resting. The index on `created` keeps this
   * to the rows older than the shorter lifetime, not the whole table. */
  private expireActive(now: number): void {
    const sql = this.ctx.storage.sql;
    // unclaimedTtl < mailboxTtl, so both kinds of expiry are older than this.
    const expired = sql
      .exec<{ n: number }>(
        "SELECT n FROM active WHERE created < ? AND (full = 0 OR created < ?)",
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
    const active = sql.exec<{ c: number }>("SELECT count(*) AS c FROM active").one().c;
    if (active >= LIMITS.maxNameplates) return { error: "server busy" };
    // The lowest number free of both, else the lowest that is only cooling
    // down: either 1, or one past a number in use. Found in SQL, one indexed
    // probe per candidate, instead of reading every row into memory.
    const lowest = (blocked: string): number => {
      const free = (x: string) =>
        `NOT EXISTS (SELECT 1 FROM active a WHERE a.n = ${x}) AND ` +
        `NOT EXISTS (SELECT 1 FROM resting r WHERE r.n = ${x} AND r.until > ?1 AND ${blocked})`;
      const row = sql
        .exec<{ n: number }>(
          `SELECT n FROM (
             SELECT 1 AS n WHERE ${free("1")}
             UNION ALL
             SELECT u.n + 1 AS n FROM (
               SELECT n FROM active UNION SELECT n FROM resting WHERE until > ?1
             ) u
             WHERE u.n + 1 <= ?2 AND ${free("u.n + 1")}
           ) ORDER BY n LIMIT 1`,
          now,
          LIMITS.maxNameplates * 4,
        )
        .toArray()[0];
      return row?.n ?? 0;
    };
    let n = lowest("1");
    if (n === 0) n = lowest("r.retired = 1");
    if (n === 0) return { error: "server busy" };
    const token = hex(randomBytes(16));
    const mailbox = hex(randomBytes(16));
    sql.exec("DELETE FROM resting WHERE n = ?", n);
    sql.exec(
      "INSERT INTO active (n, mailbox, token, created) VALUES (?, ?, ?, ?)",
      n,
      mailbox,
      token,
      now,
    );
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
    sql.exec("DELETE FROM active WHERE n = ? AND mailbox = ?", n, mailbox);
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
    sql.exec("DELETE FROM resting WHERE until <= ?", now);
    sql.exec("DELETE FROM scary WHERE t <= ?", now - LIMITS.scaryWindow);
    const left =
      sql.exec<{ c: number }>("SELECT count(*) AS c FROM active").one().c +
      sql.exec<{ c: number }>("SELECT count(*) AS c FROM resting").one().c;
    if (left > 0) await this.ctx.storage.setAlarm(now + 60_000);
  }
}
