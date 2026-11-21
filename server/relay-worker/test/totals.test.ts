// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Per-pair byte and session-time limits across reconnects and hibernation
// wakes (#190, #426).

import assert from "node:assert/strict";
import { test } from "node:test";

import { PairTotals, SENT_STEP, type TotalsKv } from "../src/totals.ts";

const S = SENT_STEP;
const HOUR = 60 * 60 * 1000;

class FakeStorage implements TotalsKv {
  private map = new Map<string, unknown>();
  writes = 0;

  get<T = unknown>(key: string): T | undefined {
    const v = this.map.get(key);
    return v === undefined ? undefined : structuredClone(v as T);
  }

  put<T>(key: string, value: T): void {
    this.writes++;
    this.map.set(key, structuredClone(value));
  }

  delete(key: string): boolean {
    return this.map.delete(key);
  }

  get empty(): boolean {
    return this.map.size === 0;
  }
}

test("reconnecting below SENT_STEP keeps exact byte totals per key (#190)", () => {
  const kv = new FakeStorage();
  const limit = 2_000;
  let pair = new PairTotals(kv);

  assert.equal(pair.admitSignIn(1_000, 6 * HOUR, limit), true);
  assert.equal(pair.spend(0, 1_400, limit), true);
  pair.flushSide(0);

  // Simulate hibernation and reconnect of both sides.
  pair = new PairTotals(kv);
  assert.equal(pair.admitSignIn(2_000, 6 * HOUR, limit), true);
  assert.equal(pair.counted(0, limit), 1_400);
  // Side 1 has its own independent allowance.
  assert.equal(pair.spend(1, 1_400, limit), true);
  // Side 0 crosses 2,000 B across its two connections.
  assert.equal(pair.spend(0, 1_000, limit), false);
  // Once exceeded, further sign-ins on the pair are refused until expiry.
  pair = new PairTotals(kv);
  assert.equal(pair.admitSignIn(3_000, 6 * HOUR, limit), false);
});

test("exact sessionBytes boundary uses strict > across reconnects and wakes", () => {
  const kv = new FakeStorage();
  const limit = 2_000;
  let pair = new PairTotals(kv);

  assert.equal(pair.admitSignIn(1_000, 6 * HOUR, limit), true);
  assert.equal(pair.spend(0, 1_200, limit), true);
  pair.flushSide(0);

  // Reconnect and send the exact remaining 800 B to reach 2,000 == limit.
  pair = new PairTotals(kv);
  assert.equal(pair.admitSignIn(1_500, 6 * HOUR, limit), true);
  assert.equal(pair.spend(0, 800, limit), true);
  pair.flushSide(0);

  // Even after waking at exact 2,000 == limit, `over` is still false (`>` not `>=`).
  pair = new PairTotals(kv);
  assert.equal(pair.admitSignIn(1_800, 6 * HOUR, limit), true);
  // One extra byte (2,001 > 2,000) crosses the limit.
  assert.equal(pair.spend(0, 1, limit), false);
  assert.equal(pair.admitSignIn(1_900, 6 * HOUR, limit), false);
});

test("rolling-deploy wake without stored totals initializes lazily instead of dropping", () => {
  const kv = new FakeStorage();
  const pair = new PairTotals(kv);

  // Socket signed in before deploy (`since = 5_000`, legacy `sent = 700`) sends after wake.
  assert.equal(pair.spend(0, 500, 2_000, 5_000, [700, 0]), true);
  assert.equal(pair.deadline(900), 5_900);
  assert.equal(pair.counted(0), 1_200);
  assert.equal(pair.spend(0, 900, 2_000, 5_000), false);
});

test("reconnecting keeps the pair's original session deadline and expiry clears storage (#190)", () => {
  const kv = new FakeStorage();
  const sessionTime = 900;
  let pair = new PairTotals(kv);

  assert.equal(pair.admitSignIn(10_000, sessionTime), true);
  assert.equal(pair.deadline(sessionTime), 10_900);
  pair.flushSide(0);

  // Reconnect halfway through: deadline stays 10_900, not 10_450 + 900.
  pair = new PairTotals(kv);
  assert.equal(pair.admitSignIn(10_450, sessionTime), true);
  assert.equal(pair.deadline(sessionTime), 10_900);
  assert.equal(pair.isExpired(10_899, sessionTime), false);
  assert.equal(pair.isExpired(10_900, sessionTime), true);

  pair.clear();
  assert.equal(kv.empty, true);
  assert.equal(pair.deadline(sessionTime), null);
});

test("waking from hibernation with unwritten bytes never counts low (#190, #426)", () => {
  for (const size of [40, 1_200, 64 * 1024, S - 1, S, S + 1]) {
    for (const every of [1, 4, 13]) {
      const kv = new FakeStorage();
      let pair = new PairTotals(kv);
      pair.admitSignIn(0, 6 * HOUR);
      let truly = 0;
      for (let i = 0; i < 50; i++) {
        assert.equal(pair.spend(0, size, 32 * 1024 * 1024 * 1024), true);
        truly += size;
        if (i % every === 0) pair = new PairTotals(kv);
        assert.ok(
          pair.counted(0) >= truly,
          `${size}/${every}/${i}: counted ${pair.counted(0)} for ${truly}`,
        );
      }
    }
  }
});

test("idle wakes and peer writes never drift an idle side's count", () => {
  const kv = new FakeStorage();
  let pair = new PairTotals(kv);
  pair.admitSignIn(0, 6 * HOUR);
  pair.spend(0, 1_200, 32 * 1024 * 1024 * 1024);

  // Side 0 stays open (unflushed) while Side 1 sends and the object wakes 30 times.
  pair = new PairTotals(kv);
  const afterFirstWake = pair.counted(0);
  for (let i = 0; i < 30; i++) {
    pair.spend(1, 500, 32 * 1024 * 1024 * 1024);
    pair = new PairTotals(kv);
    assert.equal(pair.counted(0), afterFirstWake);
  }
});

test("writes to storage are batched by SENT_STEP and never fire on consecutive small frames", () => {
  const kv = new FakeStorage();
  let pair = new PairTotals(kv);
  pair.admitSignIn(0, 6 * HOUR);

  let prevWrote = false;
  // Send 3 MiB in 1 KiB frames, including a hibernation wake mid-stream.
  for (let i = 0; i < 3 * 1_024; i++) {
    if (i === 1_500) {
      pair = new PairTotals(kv);
      prevWrote = false;
    }
    const before = kv.writes;
    pair.spend(0, 1_024, 32 * 1024 * 1024 * 1024);
    const wrote = kv.writes > before;
    assert.ok(!(prevWrote && wrote), `back-to-back KV writes at frame ${i}`);
    prevWrote = wrote;
  }
  pair.flushSide(0);
  assert.ok(kv.writes <= 8, `expected <= 8 total KV writes, saw ${kv.writes}`);
});
