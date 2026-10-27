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
      `);
    });
  }

  private now(): number {
    return Date.now();
  }

  private expireActive(now: number): void {
    const sql = this.ctx.storage.sql;
    const expired = sql
      .exec<{ n: number }>(
        "SELECT n FROM active WHERE (full = 0 AND created < ?) OR created < ?",
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
    const active = new Set(
      sql.exec<{ n: number }>("SELECT n FROM active").toArray().map((r) => r.n),
    );
    if (active.size >= LIMITS.maxNameplates) return { error: "server busy" };
    const retired = new Set<number>();
    const cooling = new Set<number>();
    for (const r of sql
      .exec<{ n: number; retired: number }>("SELECT n, retired FROM resting WHERE until > ?", now)
      .toArray()) {
      (r.retired ? retired : cooling).add(r.n);
    }
    const range = LIMITS.maxNameplates * 4;
    let n = 0;
    for (let i = 1; i <= range; i++) {
      if (!active.has(i) && !retired.has(i) && !cooling.has(i)) {
        n = i;
        break;
      }
    }
    if (n === 0) {
      for (let i = 1; i <= range; i++) {
        if (!active.has(i) && !retired.has(i)) {
          n = i;
          break;
        }
      }
    }
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

  async lookup(n: number): Promise<Lookup | null> {
    this.expireActive(this.now());
    const row = this.ctx.storage.sql
      .exec<{ mailbox: string; token: string; created: number }>(
        "SELECT mailbox, token, created FROM active WHERE n = ?",
        n,
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
