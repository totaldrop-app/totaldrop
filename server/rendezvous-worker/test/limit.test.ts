// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The per-client rate limit. The Rust server's unit tests
// (`server/totaldrop-rendezvous/src/lib.rs`, module `tests`) check the same
// cases; the end-to-end suite checks both servers from outside.

import assert from "node:assert/strict";
import { test } from "node:test";

import { RATE_DEFAULTS, RateLimiter, clientOf, rateConfig } from "../src/limit.ts";

const config = { burst: 4, intervalMs: 2_000, maxClients: 3 };

test("a burst is allowed, the next request is not, and one more comes back each interval", async () => {
  const l = new RateLimiter(config);
  for (let i = 0; i < 4; i++) assert.equal(await l.admit("a", 0), true, `request ${i}`);
  assert.equal(await l.admit("a", 0), false);
  assert.equal(await l.admit("a", 1_999), false);
  assert.equal(await l.admit("a", 2_000), true);
  assert.equal(await l.admit("a", 2_000), false);
});

test("refused requests are not counted", async () => {
  const l = new RateLimiter(config);
  for (let i = 0; i < 4; i++) await l.admit("a", 0);
  for (let i = 0; i < 100; i++) assert.equal(await l.admit("a", 0), false);
  assert.equal(await l.admit("a", 2_000), true);
});

test("clients are counted apart", async () => {
  const l = new RateLimiter(config);
  for (let i = 0; i < 4; i++) await l.admit("a", 0);
  assert.equal(await l.admit("a", 0), false);
  assert.equal(await l.admit("b", 0), true);
});

test("a quiet client is forgotten once its burst is full again", async () => {
  const l = new RateLimiter(config);
  for (let i = 0; i < 4; i++) await l.admit("a", 0);
  await l.admit("b", 0);
  assert.equal(l.size, 2);
  // "a" is back at a full burst at 8 s, "b" at 2 s; the pass at 8 s forgets
  // both before counting "c".
  await l.admit("c", 8_000);
  assert.equal(l.size, 1);
});

test("beyond the most clients, a new client is let through uncounted", async () => {
  const l = new RateLimiter(config);
  for (const c of ["a", "b", "c"]) await l.admit(c, 0);
  for (let i = 0; i < 10; i++) assert.equal(await l.admit("d", 0), true);
  assert.equal(l.size, 3);
});

test("the defaults hold unless a test sets whole positive numbers", () => {
  assert.deepEqual(rateConfig(undefined, undefined), RATE_DEFAULTS);
  assert.deepEqual(rateConfig("0", "-5"), RATE_DEFAULTS);
  assert.deepEqual(rateConfig("1.5", "1e3"), RATE_DEFAULTS);
  assert.deepEqual(rateConfig("4", "2000"), { ...RATE_DEFAULTS, burst: 4, intervalMs: 2_000 });
});

test("a client is an IPv4 address or an IPv6 /64", () => {
  assert.equal(clientOf("203.0.113.7"), "203.0.113.7");
  assert.equal(clientOf("::ffff:203.0.113.7"), "203.0.113.7");
  assert.equal(clientOf("::FFFF:cb00:7107"), "203.0.113.7");
  assert.equal(clientOf("2001:db8:1:2:3:4:5:6"), "2001:db8:1:2");
  assert.equal(clientOf("2001:db8:1:2::9"), "2001:db8:1:2");
  assert.equal(clientOf("2001:0DB8:0001:0002:aaaa:bbbb:cccc:dddd"), "2001:db8:1:2");
  assert.equal(clientOf("2001:db8::"), "2001:db8:0:0");
  assert.equal(clientOf("::1"), "0:0:0:0");
  assert.equal(clientOf("64:ff9b::192.0.2.33"), "64:ff9b:0:0");
});

test("anything else is one shared client", () => {
  for (const bad of [null, "", "localhost", "1:2:3:4:5:6:7:8:9", "1::2::3", "1:2:3:4:5:6:7", "12345::", "::1.2.3.256", "1:2:3:4:5:6:7::8"]) {
    assert.equal(clientOf(bad), "", String(bad));
  }
});
