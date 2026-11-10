// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The relay's count of bytes given to a socket, across the object waking
// from hibernation (#190, #426).

import assert from "node:assert/strict";
import { test } from "node:test";

import { type Counted, PROBE_STEP, noteGiven, restored } from "../src/count.ts";

/** Give a socket datagrams of `sizes` bytes the way pair.ts does, and
 * return what its attachment keeps, as the woken object reads it back. */
function give(sizes: number[]): { att: Counted; given: number; probes: number } {
  const att: Counted = {};
  let given = 0;
  let probes = 0;
  for (const n of sizes) {
    if (noteGiven(att, given, given + n) === "probe") probes++;
    given += n;
  }
  return { att: structuredClone(att), given, probes };
}

test("bytes given before the first probe still count after waking (#426)", () => {
  // Before #426 the attachment had nothing until the first probe, and a
  // woken object counted up to PROBE_STEP - 1 such bytes as 0.
  const { att, given, probes } = give([1200, 1200, 64 * 1024]);
  assert.equal(probes, 0);
  const [restoredGiven] = restored(att);
  assert.ok(restoredGiven >= given, `counted ${restoredGiven} for ${given} given`);
  assert.equal(restoredGiven, PROBE_STEP);
});

test("a socket that was given nothing counts nothing after waking", () => {
  // Replaced sockets count against their key (KEY_UNANSWERED); an empty
  // one must not take a step of it.
  assert.deepEqual(restored(give([]).att), [0, 0]);
  assert.deepEqual(restored(null), [0, 0]);
});

test("only the first bytes and each step write to the attachment", () => {
  const att: Counted = {};
  assert.equal(noteGiven(att, 0, 1200), "first");
  assert.equal(noteGiven(att, 1200, 2400), "none");
  assert.equal(noteGiven(att, 2400, PROBE_STEP + 10), "probe");
  assert.equal(att.given, PROBE_STEP + 10);
  assert.equal(noteGiven(att, PROBE_STEP + 10, PROBE_STEP + 20), "none");
});

test("after waking the count is never below what was given, and at most a step above", () => {
  const step = 64 * 1024;
  for (const count of [1, 10, 63, 64, 65, 200, 700]) {
    const { att, given } = give(Array.from({ length: count }, () => step));
    const [restoredGiven] = restored(att);
    assert.ok(restoredGiven >= given, `${count}: counted ${restoredGiven} for ${given}`);
    assert.ok(restoredGiven <= given + PROBE_STEP, `${count}: counted ${restoredGiven} for ${given}`);
  }
});
