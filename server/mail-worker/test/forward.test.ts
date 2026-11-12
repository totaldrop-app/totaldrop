// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The mail Worker's forwarding (#415), with a stand-in for the incoming
// message that records each forward.

import assert from "node:assert/strict";
import { test } from "node:test";

import { MAX_RECIPIENTS, forwardAll, handle, recipients } from "../src/forward.ts";

/** A message whose `forward` fails for the addresses in `failFor`. */
function message(failFor: string[] = []) {
  const sent: string[] = [];
  return {
    sent,
    async forward(to: string) {
      if (failFor.includes(to)) throw new Error("destination address not verified");
      sent.push(to);
      return { messageId: "x" };
    },
  };
}

test("two addresses separated by a comma, spaces or a new line", () => {
  assert.deepEqual(recipients("a@example.com,b@example.org"), ["a@example.com", "b@example.org"]);
  assert.deepEqual(recipients(" a@example.com ,\n b@example.org \n"), [
    "a@example.com",
    "b@example.org",
  ]);
});

test("one address is enough", () => {
  assert.deepEqual(recipients("a@example.com"), ["a@example.com"]);
});

test("the same address twice, in any case, is kept once and in order", () => {
  assert.deepEqual(recipients("A@Example.com, b@example.org, a@example.com"), [
    "A@Example.com",
    "b@example.org",
  ]);
});

test("a missing or empty setting fails closed", () => {
  for (const raw of [undefined, "", "  ", ",, \n"]) {
    assert.throws(() => recipients(raw), /FORWARD_TO is not set/, JSON.stringify(raw));
  }
});

test("an entry that is not an address fails, and the error does not repeat it", () => {
  for (const bad of ["ozan", "a@b", "a@x..com", "a@.x.com", "a@-x.com", "a@x-.com", "a@x.c0m", "@example.com", "a@", "a@@example.com", "<a@example.com>", "a@example.", '"a"@example.com']) {
    assert.throws(
      () => recipients(`ok@example.com, ${bad}`),
      (e: Error) => /entry 2 is not an email address/.test(e.message) && !e.message.includes(bad),
      bad,
    );
  }
  assert.throws(() => recipients(`${"a".repeat(250)}@example.com`), /entry 1/);
});

test("an address on our own domain would loop and fails", () => {
  for (const own of ["hello@totaldrop.app", "x@TotalDrop.App", "x@mail.totaldrop.app"]) {
    assert.throws(() => recipients(`a@example.com,${own}`), /would loop/, own);
  }
  // A look-alike domain is someone else's and is fine.
  assert.deepEqual(recipients("x@nottotaldrop.app"), ["x@nottotaldrop.app"]);
});

test(`at most ${MAX_RECIPIENTS} destinations`, () => {
  const list = (n: number) => Array.from({ length: n }, (_, i) => `m${i}@example.com`).join(",");
  assert.equal(recipients(list(MAX_RECIPIENTS)).length, MAX_RECIPIENTS);
  assert.throws(() => recipients(list(MAX_RECIPIENTS + 1)), /at most/);
  // Repeats do not count towards the limit.
  assert.equal(recipients(`${list(MAX_RECIPIENTS)},m0@example.com`).length, MAX_RECIPIENTS);
});

test("every destination gets the message once", async () => {
  const m = message();
  assert.equal(await forwardAll(m, ["a@example.com", "b@example.org"]), 0);
  assert.deepEqual(m.sent.sort(), ["a@example.com", "b@example.org"]);
});

test("one failed destination does not stop the other", async () => {
  const m = message(["b@example.org"]);
  assert.equal(await forwardAll(m, ["a@example.com", "b@example.org"]), 1);
  assert.deepEqual(m.sent, ["a@example.com"]);
  const n = message(["a@example.com"]);
  assert.equal(await forwardAll(n, ["a@example.com", "b@example.org"]), 1);
  assert.deepEqual(n.sent, ["b@example.org"]);
});

test("a destination whose forward throws at once does not stop the other", async () => {
  const sent: string[] = [];
  const m = {
    forward(to: string): Promise<unknown> {
      if (to === "a@example.com") throw new Error("refused before any promise");
      sent.push(to);
      return Promise.resolve();
    },
  };
  assert.equal(await forwardAll(m, ["a@example.com", "b@example.org"]), 1);
  assert.deepEqual(sent, ["b@example.org"]);
});

test("when every destination fails, the message fails", async () => {
  const m = message(["a@example.com", "b@example.org"]);
  await assert.rejects(forwardAll(m, ["a@example.com", "b@example.org"]), /all 2 destinations/);
});

test("no destination at all is an error, not a silent drop", async () => {
  await assert.rejects(forwardAll(message(), []), /no destination/);
});

test("the handler forwards to every configured address and reports nothing", async () => {
  const m = message();
  const lines: string[] = [];
  await handle(m, "a@example.com, b@example.org", (l) => lines.push(l));
  assert.deepEqual(m.sent.sort(), ["a@example.com", "b@example.org"]);
  assert.deepEqual(lines, []);
});

test("the handler with no setting forwards nothing and fails", async () => {
  const m = message();
  const lines: string[] = [];
  await assert.rejects(handle(m, undefined, (l) => lines.push(l)), /FORWARD_TO is not set/);
  assert.deepEqual(m.sent, []);
  assert.deepEqual(lines, []);
});

test("the handler reports a partial failure as a count, without the address", async () => {
  const m = message(["b@example.org"]);
  const lines: string[] = [];
  await handle(m, "a@example.com,b@example.org", (l) => lines.push(l));
  assert.deepEqual(m.sent, ["a@example.com"]);
  assert.deepEqual(lines, ["forwarding failed for 1 destination(s)"]);
});
