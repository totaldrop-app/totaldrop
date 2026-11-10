// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The directory's alarm keeps running while clients are counted (#393).
// The Directory itself needs the Workers runtime; these tests drive the
// same steps it takes (`directory.ts`: `admit`, `alarm`) with the real
// rate limiter and a stand-in for its storage.

import assert from "node:assert/strict";
import { test } from "node:test";

import {
  ALARM_EVERY_MS,
  type AlarmStorage,
  afterCounted,
  ensureAlarm,
  nextAlarm,
} from "../src/alarm.ts";
import { RATE_DEFAULTS, RateLimiter } from "../src/limit.ts";

/** Storage with one alarm, as a Durable Object has. */
function storage(): AlarmStorage & { alarm: number | null; sets: number } {
  const s = {
    alarm: null as number | null,
    sets: 0,
    async getAlarm() {
      return s.alarm;
    },
    async setAlarm(t: number) {
      s.alarm = t;
      s.sets++;
    },
  };
  return s;
}

/** One lookup by `client` at `now`, as `Directory.lookup` counts it. */
async function lookup(s: AlarmStorage, l: RateLimiter, client: string, now: number) {
  const allowed = await l.admit(client, now);
  await afterCounted(s, l.size, now);
  return allowed;
}

/** The alarm firing at its time with no nameplate in use, as
 * `Directory.alarm` runs it. */
function fire(s: ReturnType<typeof storage>, l: RateLimiter) {
  const now = s.alarm!;
  s.alarm = null;
  l.prune(now);
  const next = nextAlarm(now, false, l.size);
  if (next !== null) s.alarm = next;
  return now;
}

test("a lookup with no nameplate in use sets the alarm, and the alarm forgets the counter", async () => {
  const s = storage();
  const l = new RateLimiter(RATE_DEFAULTS);
  assert.equal(await lookup(s, l, "203.0.113.7", 0), true);
  assert.equal(l.size, 1);
  assert.equal(s.alarm, ALARM_EVERY_MS);
  fire(s, l);
  assert.equal(l.size, 0, "the counter is forgotten at the first alarm");
  assert.equal(s.alarm, null, "nothing left: the alarm is not set again");
});

test("a client that spent its whole burst is forgotten within two alarm intervals", async () => {
  const s = storage();
  const l = new RateLimiter(RATE_DEFAULTS);
  // The alarm is already set, early, by some earlier request.
  await ensureAlarm(s, -30_000);
  for (let i = 0; i < RATE_DEFAULTS.burst; i++) await lookup(s, l, "198.51.100.1", 0);
  assert.equal(await lookup(s, l, "198.51.100.1", 0), false);
  const last = 0; // the client's last request
  let forgotten: number | null = null;
  for (let i = 0; i < 5 && s.alarm !== null; i++) {
    const at = fire(s, l);
    if (l.size === 0 && forgotten === null) forgotten = at;
  }
  assert.notEqual(forgotten, null);
  assert.ok(forgotten! - last <= 2 * ALARM_EVERY_MS, `forgotten after ${forgotten! - last} ms`);
  assert.equal(s.alarm, null);
});

test("the alarm stays set while a client is counted, even with no nameplate", () => {
  assert.equal(nextAlarm(1_000, false, 1), 1_000 + ALARM_EVERY_MS);
  assert.equal(nextAlarm(1_000, true, 0), 1_000 + ALARM_EVERY_MS);
  assert.equal(nextAlarm(1_000, false, 0), null);
});

test("no counter and no nameplate: a request does not set the alarm", async () => {
  const s = storage();
  await afterCounted(s, 0, 0);
  assert.equal(s.alarm, null);
});

test("an alarm already set is not moved by later requests", async () => {
  const s = storage();
  const l = new RateLimiter(RATE_DEFAULTS);
  await lookup(s, l, "a", 0);
  await lookup(s, l, "a", 10_000);
  assert.equal(s.alarm, ALARM_EVERY_MS);
  assert.equal(s.sets, 1);
});
