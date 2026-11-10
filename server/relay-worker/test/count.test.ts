// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The relay's count of bytes given to a socket after the object wakes
// (#190, #426).

import assert from "node:assert/strict";
import { test } from "node:test";

import { PROBE_STEP, restored } from "../src/count.ts";

test("a socket that got no probe yet may hold up to a step: it counts high (#426)", () => {
  // Up to PROBE_STEP - 1 bytes can be given before the first probe, and
  // none of them reach the attachment. Before #426 they counted as 0.
  assert.deepEqual(restored({}), [PROBE_STEP, 0]);
  assert.deepEqual(restored(null), [PROBE_STEP, 0]);
  const before = PROBE_STEP - 1;
  assert.ok(restored({})[0] >= before, "counted lower than what it may hold");
});

test("after a probe, up to another step may follow it", () => {
  assert.deepEqual(restored({ given: PROBE_STEP }), [2 * PROBE_STEP, 0]);
  assert.deepEqual(restored({ given: 3 * PROBE_STEP, answered: 2 * PROBE_STEP }), [
    4 * PROBE_STEP,
    2 * PROBE_STEP,
  ]);
});

test("what a woken socket holds unanswered is never below what it may hold", () => {
  // Any history: the true count is between the last probe and one step
  // after it; the restored one is the top of that range.
  for (const probes of [0, 1, 2, 7]) {
    for (const answered of [0, probes]) {
      const att =
        probes === 0 ? {} : { given: probes * PROBE_STEP, answered: answered * PROBE_STEP };
      const truly = probes * PROBE_STEP + (PROBE_STEP - 1);
      const [given, ans] = restored(att);
      assert.ok(given - ans >= truly - answered * PROBE_STEP, JSON.stringify(att));
    }
  }
});
