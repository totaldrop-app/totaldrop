// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Running byte and time totals for one pair across reconnects and hibernation
// wakes (#190, #426). Kept free of Workers-only globals so it runs under
// Node's test runner as well.

/** The count of bytes sent is persisted when a side starts sending after a
 * flush or wake, when it passes another step of this size, and when its
 * socket closes; an ordinary datagram costs no extra write, and waking
 * counts at most one step high, never low (#426). */
export const SENT_STEP = 1024 * 1024;

export type Side = 0 | 1;

/** What Durable Object storage keeps under `"totals"` for one pair. */
export interface StoredTotals {
  /** When the pair's first key signed in, in ms since the epoch. */
  started: number;
  /** Bytes sent by `[pair[0], pair[1]]` at each side's last write. */
  sent: [number, number];
  /** Whether side `i` may have sent fewer than {@link SENT_STEP} unwritten
   * bytes before the object hibernated; cleared when its socket closes. */
  unflushed: [boolean, boolean];
  /** Set once either key strictly exceeds `sessionBytes`, refusing further
   * sign-ins until the pair's session window expires. */
  over: boolean;
}

/** Synchronous key-value storage matching `DurableObjectStorage["kv"]`. */
export interface TotalsKv {
  get<T = unknown>(key: string): T | undefined;
  put<T>(key: string, value: T): void;
  delete(key: string): boolean;
}

interface LiveTotals {
  started: number;
  /** Running bytes counted in memory for each side. */
  sent: [number, number];
  /** Exact values currently stored in KV for each side, so saving one side
   * never alters an idle peer across wakes. */
  diskSent: [number, number];
  /** Unwritten bytes accumulated in memory for side `i` since its last KV
   * write (`SENT_STEP` right after waking with `unflushed[i]` true so the
   * first post-wake frame writes once and resets to `0`). */
  sinceWrite: [number, number];
  unflushed: [boolean, boolean];
  over: boolean;
}

const KEY = "totals";

/** Tracks one pair's session deadline and per-key sent bytes across
 * reconnects and hibernation wakes. */
export class PairTotals {
  private readonly kv: TotalsKv;
  private mem: LiveTotals | null | undefined = undefined;

  constructor(kv: TotalsKv) {
    this.kv = kv;
  }

  private load(): LiveTotals | null {
    if (this.mem !== undefined) return this.mem;
    const s = this.kv.get<StoredTotals>(KEY);
    if (!s) {
      this.mem = null;
      return null;
    }
    const sent0 = s.unflushed[0] ? s.sent[0] + SENT_STEP : s.sent[0];
    const sent1 = s.unflushed[1] ? s.sent[1] + SENT_STEP : s.sent[1];
    this.mem = {
      started: s.started,
      sent: [sent0, sent1],
      diskSent: [s.sent[0], s.sent[1]],
      sinceWrite: [s.unflushed[0] ? SENT_STEP : 0, s.unflushed[1] ? SENT_STEP : 0],
      unflushed: [s.unflushed[0], s.unflushed[1]],
      over: s.over,
    };
    return this.mem;
  }

  /** Strict `>` matches `spend()` and the Rust relay (`*slot > session_bytes`).
   * Evaluated against current `t.sent` rather than cached at first `load()`, so
   * callers that inspect `deadline()` before `admitSignIn()` still see a
   * wake-inflated count above `sessionBytes`. */
  private isOver(t: LiveTotals, sessionBytes?: number): boolean {
    return (
      t.over ||
      (sessionBytes !== undefined &&
        (t.sent[0] > sessionBytes || t.sent[1] > sessionBytes))
    );
  }

  private save(t: LiveTotals): void {
    const stored: StoredTotals = {
      started: t.started,
      sent: [t.diskSent[0], t.diskSent[1]],
      unflushed: [t.unflushed[0], t.unflushed[1]],
      over: t.over,
    };
    this.kv.put(KEY, stored);
  }

  /** Lazily initialize `"totals"` if an already-signed-in hibernated socket
   * wakes after a rolling deploy before `"totals"` exists in KV. */
  ensure(started: number, seedSent: [number, number] = [0, 0], sessionBytes?: number): void {
    if (this.load()) return;
    const over =
      sessionBytes !== undefined &&
      (seedSent[0] > sessionBytes || seedSent[1] > sessionBytes);
    const t: LiveTotals = {
      started,
      sent: [seedSent[0], seedSent[1]],
      diskSent: [seedSent[0], seedSent[1]],
      sinceWrite: [0, 0],
      unflushed: [false, false],
      over,
    };
    this.mem = t;
    this.save(t);
  }

  /** Deadline (`started + sessionTime`) of the active pair, or `null` if no
   * key has signed in yet. */
  deadline(sessionTime: number): number | null {
    const t = this.load();
    return t ? t.started + sessionTime : null;
  }

  /** Whether an active pair's session window has ended at `now`. */
  isExpired(now: number, sessionTime: number): boolean {
    const end = this.deadline(sessionTime);
    return end !== null && now >= end;
  }

  /** Erase all stored totals for this pair. */
  clear(): void {
    this.mem = null;
    this.kv.delete(KEY);
  }

  /** Admit a verified sign-in at `now`: starts the pair's window on the
   * first sign-in, and refuses once the byte or time limit is spent. */
  admitSignIn(now: number, sessionTime: number, sessionBytes?: number): boolean {
    let t = this.load();
    if (!t) {
      t = {
        started: now,
        sent: [0, 0],
        diskSent: [0, 0],
        sinceWrite: [0, 0],
        unflushed: [false, false],
        over: false,
      };
      this.mem = t;
      this.save(t);
      return true;
    }
    if (this.isOver(t, sessionBytes)) {
      if (!t.over) {
        t.over = true;
        this.save(t);
      }
      return false;
    }
    return now < t.started + sessionTime;
  }

  /** Count `bytes` sent by `side`; `false` when `sessionBytes` is strictly
   * exceeded. If `"totals"` was absent on a rolling-deploy wake, initializes
   * it at `fallbackStart` instead of dropping an active transfer. */
  spend(
    side: Side,
    bytes: number,
    sessionBytes: number,
    fallbackStart?: number,
    seedSent?: [number, number],
  ): boolean {
    let t = this.load();
    if (!t) {
      if (fallbackStart === undefined) return false;
      this.ensure(fallbackStart, seedSent ?? [0, 0], sessionBytes);
      t = this.load();
      if (!t) return false;
    }
    if (this.isOver(t, sessionBytes)) {
      if (!t.over) {
        t.over = true;
        this.save(t);
      }
      return false;
    }
    const after = t.sent[side] + bytes;
    t.sent[side] = after;
    if (after > sessionBytes) {
      t.over = true;
      t.diskSent[side] = after;
      t.sinceWrite[side] = 0;
      t.unflushed[side] = false;
      this.save(t);
      return false;
    }
    t.sinceWrite[side] += bytes;
    if (!t.unflushed[side] || t.sinceWrite[side] >= SENT_STEP) {
      t.diskSent[side] = after;
      t.sinceWrite[side] = 0;
      t.unflushed[side] = true;
      this.save(t);
    }
    return true;
  }

  /** Flush exact bytes for `side` when its socket closes or is replaced, so
   * reconnects below {@link SENT_STEP} lose nothing and idle wakes add nothing. */
  flushSide(side: Side): void {
    const t = this.load();
    if (!t || !t.unflushed[side]) return;
    t.diskSent[side] = t.sent[side];
    t.sinceWrite[side] = 0;
    t.unflushed[side] = false;
    this.save(t);
  }

  /** Current counted bytes for `side` (for tests). */
  counted(side: Side): number {
    const t = this.load();
    return t ? t.sent[side] : 0;
  }
}
