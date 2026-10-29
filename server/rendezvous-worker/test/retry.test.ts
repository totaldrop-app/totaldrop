// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Unit tests for src/retry.ts (#119). Run with `npm test` (node --test).

import assert from "node:assert/strict";
import { test } from "node:test";

import { ATTEMPTS, BUSY, FAILED, mayRetry, RETRY_DELAY, withRetry } from "../src/retry.ts";

/** A call that fails with each of `errors` in turn, then answers `value`. */
function flaky<T>(errors: unknown[], value: T) {
  const calls = { n: 0 };
  const call = async () => {
    const e = errors[calls.n++];
    if (e !== undefined) throw e;
    return value;
  };
  return { call, calls };
}

function recorder() {
  const pauses: number[] = [];
  return { pauses, sleep: async (ms: number) => void pauses.push(ms) };
}

/** What workerd threw when a new Durable Object's first request failed. */
function internalError(): Error {
  return Object.assign(new Error("internal error; reference = m08h44nd35cvdgpphdor6jd1"), {
    remote: true,
  });
}

test("a first try that throws is tried once more and succeeds", async () => {
  const ok = new Response("opened");
  const { call, calls } = flaky([internalError()], ok);
  const { pauses, sleep } = recorder();
  const out = await withRetry(call, sleep);
  assert.deepEqual(out, { ok: true, value: ok });
  assert.equal(calls.n, 2);
  assert.deepEqual(pauses, [RETRY_DELAY]);
});

test("every try failing gives a readable refusal, not an exception", async () => {
  const { call, calls } = flaky([internalError(), internalError(), internalError()], null);
  const { pauses, sleep } = recorder();
  const out = await withRetry(call, sleep);
  assert.deepEqual(out, { ok: false, reason: FAILED });
  assert.equal(FAILED, "server error, try again");
  assert.equal(calls.n, ATTEMPTS);
  assert.deepEqual(pauses, [RETRY_DELAY, 2 * RETRY_DELAY]);
});

test("an overloaded object is not tried again and the client hears busy", async () => {
  const e = Object.assign(new Error("Durable Object is overloaded."), {
    overloaded: true,
    retryable: true,
  });
  const { call, calls } = flaky([e], "never");
  const { pauses, sleep } = recorder();
  const out = await withRetry(call, sleep);
  assert.deepEqual(out, { ok: false, reason: BUSY });
  assert.equal(BUSY, "server busy");
  assert.equal(calls.n, 1);
  assert.deepEqual(pauses, []);
});

test("a response, even an error response, is the answer", async () => {
  const bad = new Response("bad forward", { status: 500 });
  const { call, calls } = flaky([], bad);
  const out = await withRetry(call, recorder().sleep);
  assert.deepEqual(out, { ok: true, value: bad });
  assert.equal(calls.n, 1);
});

test("an error thrown by our own object code is not tried again", async () => {
  const e = Object.assign(new TypeError("x is undefined"), { remote: true });
  const { call, calls } = flaky([e], "never");
  const out = await withRetry(call, recorder().sleep);
  assert.deepEqual(out, { ok: false, reason: FAILED });
  assert.equal(calls.n, 1);
});

test("a thrown non-object did not come from inside the object and is tried again", async () => {
  const { call, calls } = flaky(["x"], "opened");
  const { pauses, sleep } = recorder();
  const out = await withRetry(call, sleep);
  assert.deepEqual(out, { ok: true, value: "opened" });
  assert.equal(calls.n, 2);
  assert.deepEqual(pauses, [RETRY_DELAY]);
});

test("an object that turns out overloaded on the second try ends as busy", async () => {
  const overloaded = Object.assign(new Error("Durable Object is overloaded."), {
    overloaded: true,
    remote: true,
  });
  const { call, calls } = flaky([internalError(), overloaded], "never");
  const { pauses, sleep } = recorder();
  const out = await withRetry(call, sleep);
  assert.deepEqual(out, { ok: false, reason: BUSY });
  assert.equal(calls.n, 2);
  assert.deepEqual(pauses, [RETRY_DELAY]);
});

test("which errors may be tried again", () => {
  assert.equal(mayRetry(Object.assign(new Error("x"), { remote: true, retryable: true })), true);
  // Not thrown inside the object: from the call itself.
  assert.equal(mayRetry(new Error("Network connection lost.")), true);
  assert.equal(mayRetry(internalError()), true);
  // Only workerd's exact wording counts as its internal error.
  assert.equal(
    mayRetry(Object.assign(new Error("internal error in our parser"), { remote: true })),
    false,
  );
  assert.equal(mayRetry(Object.assign(new Error("x"), { remote: true })), false);
  assert.equal(
    mayRetry(Object.assign(new Error("x"), { overloaded: true, retryable: true })),
    false,
  );
});
