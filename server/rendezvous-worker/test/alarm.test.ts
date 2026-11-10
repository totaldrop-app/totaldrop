// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The directory's alarm keeps running while clients are counted (#393).
// `Directory` needs the Workers runtime, so these tests call the two
// functions it uses for the rate limit's part (`admitCounted` in `admit`,
// `finishAlarm` at the end of `alarm`) with the real rate limiter and a
// stand-in for its storage.

import assert from "node:assert/strict";
import { test } from "node:test";

import { ALARM_EVERY_MS, type AlarmStorage, admitCounted, finishAlarm } from "../src/alarm.ts";
import { RATE_DEFAULTS, RateLimiter } from "../src/limit.ts";

/** Storage with one alarm, as a Durable Object has. While the alarm
 * handler runs, `getAlarm` answers null, as Cloudflare's does. */
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

type Store = ReturnType<typeof storage>;

/** Run the alarm at its time, with no nameplate in use. */
async function fire(s: Store, l: RateLimiter): Promise<number> {
  const now = s.alarm!;
  s.alarm = null;
  await finishAlarm(s, l, false, now);
  return now;
}

/** Fire alarms until none is set; when the counters were all gone. */
async function forgottenAt(s: Store, l: RateLimiter): Promise<number> {
  for (let i = 0; i < 10; i++) {
    const at = await fire(s, l);
    if (l.size === 0) {
      assert.equal(s.alarm, null, "nothing left: the alarm is not set again");
      return at;
    }
  }
  throw new Error("the counter was never forgotten");
}

test("a lookup with no nameplate in use sets the alarm, and the alarm forgets the counter", async () => {
  const s = storage();
  const l = new RateLimiter(RATE_DEFAULTS);
  assert.equal(await admitCounted(s, l, "203.0.113.7", 0), true);
  assert.equal(l.size, 1);
  assert.equal(s.alarm, ALARM_EVERY_MS);
  assert.equal(await forgottenAt(s, l), ALARM_EVERY_MS);
});

test("a refused request keeps the alarm set too", async () => {
  const s = storage();
  const l = new RateLimiter({ ...RATE_DEFAULTS, burst: 1 });
  await admitCounted(s, l, "a", 0);
  s.alarm = null; // say the alarm was lost
  assert.equal(await admitCounted(s, l, "a", 0), false);
  assert.notEqual(s.alarm, null);
});

test("a spent burst is forgotten within two alarm intervals, whatever the alarm's phase", async () => {
  const burst = RATE_DEFAULTS.burst;
  for (let phase = 0; phase < ALARM_EVERY_MS; phase += 500) {
    const s = storage();
    const l = new RateLimiter(RATE_DEFAULTS);
    // An alarm already set by an earlier request, `phase` ms from now.
    await s.setAlarm(phase);
    for (let i = 0; i < burst; i++) await admitCounted(s, l, "198.51.100.1", 0);
    const at = await forgottenAt(s, l);
    assert.ok(at <= 2 * ALARM_EVERY_MS, `phase ${phase}: forgotten after ${at} ms`);
  }
});

test("a request just before the alarm does not make the alarm skip its pass", async () => {
  // The whole burst at 0.4 s: due at 60.4 s. The first alarm comes at 0.5 s,
  // the next at 60.5 s. Another client's request at 60.2 s runs the
  // limiter's own pass, too early for the spent client, and the limiter
  // then skips its passes for one interval. The alarm at 60.5 s must still
  // forget the spent client, not one alarm interval later.
  const s = storage();
  const l = new RateLimiter(RATE_DEFAULTS);
  await s.setAlarm(500);
  for (let i = 0; i < RATE_DEFAULTS.burst; i++) await admitCounted(s, l, "spent", 400);
  await fire(s, l);
  assert.equal(s.alarm, 60_500);
  await l.admit("other", 60_200); // counted, its prune pass runs at 60.2 s
  const before = l.size;
  await fire(s, l); // 60.5 s
  assert.equal(before, 2);
  assert.equal(l.size, 1, "only the client of 60.2 s is still counted");
});

test("no counter and no nameplate: the alarm is not set again", async () => {
  const s = storage();
  await finishAlarm(s, new RateLimiter(RATE_DEFAULTS), false, 0);
  assert.equal(s.alarm, null);
});

test("nameplates left: the alarm is set again with no counter", async () => {
  const s = storage();
  await finishAlarm(s, new RateLimiter(RATE_DEFAULTS), true, 0);
  assert.equal(s.alarm, ALARM_EVERY_MS);
});

test("an alarm already set is not moved by later requests", async () => {
  const s = storage();
  const l = new RateLimiter(RATE_DEFAULTS);
  await admitCounted(s, l, "a", 0);
  await admitCounted(s, l, "a", 10_000);
  assert.equal(s.alarm, ALARM_EVERY_MS);
  assert.equal(s.sets, 1);
});
