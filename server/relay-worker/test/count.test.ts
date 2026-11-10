// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The relay's count of bytes given to a socket, across the object waking
// from hibernation, once or many times (#190, #426).

import assert from "node:assert/strict";
import { test } from "node:test";

import { type Counted, PROBE_STEP, mayChange, noteGiven, restore } from "../src/count.ts";

const S = PROBE_STEP;

/** One socket as pair.ts keeps it: the attachment, which survives a wake,
 * and the object's memory (`counts` and `gave`), which does not. */
class Socket {
  att: Counted = {};
  /** Bytes really given. */
  truly = 0;
  probes = 0;
  writes = 0;
  /** Counts at the probes not answered yet (the attachment's `probes`). */
  pending: number[] = [];
  private mem: { given: number; answered: number; written: number | undefined } | null = null;

  /** `counts()`: what the object knows, read back after a wake. */
  count(): number {
    if (!this.mem) {
      const r = restore(structuredClone(this.att));
      this.mem = { given: r.given, answered: r.answered, written: r.written };
    }
    return this.mem.given;
  }

  /** What pair.ts holds against the UNANSWERED limits. */
  unanswered(): number {
    this.count();
    return this.mem!.given - this.mem!.answered;
  }

  /** The device read everything: it answers every probe (`answered()`). */
  answerAll(): void {
    const last = this.pending.at(-1);
    if (last === undefined) return;
    this.att.answered = last;
    this.pending = [];
    this.count();
    this.mem!.answered = last;
  }

  /** Forward a datagram of `n` bytes: `counts()`, then `gave()`. */
  give(n: number): void {
    const before = this.count();
    const after = before + n;
    this.truly += n;
    const mem = this.mem!;
    mem.given = after;
    if (!mayChange(mem.written, before, after)) return;
    const step = noteGiven(this.att, before, after);
    if (this.att.given !== undefined) mem.written = this.att.given;
    if (step === "none") return;
    if (step === "probe") {
      this.probes++;
      this.pending.push(after);
    }
    this.writes++;
  }

  /** Hibernate and wake: memory is lost, the attachment stays. */
  wake(): void {
    this.mem = null;
  }
}

test("bytes given before the first probe still count after waking (#426)", () => {
  // Before #426 the attachment had nothing until the first probe, and a
  // woken object counted up to PROBE_STEP - 1 such bytes as 0.
  const s = new Socket();
  for (const n of [1200, 1200, 64 * 1024]) s.give(n);
  assert.equal(s.probes, 0);
  s.wake();
  assert.ok(s.count() >= s.truly, `counted ${s.count()} for ${s.truly} given`);
});

test("waking again and again never counts low (#426 review)", () => {
  // Just under a step between wakes: with a count written only at probes
  // (or only at the first bytes), each wake lost up to one more step.
  const s = new Socket();
  for (let wake = 0; wake < 20; wake++) {
    s.give(S - 1);
    s.wake();
    assert.ok(s.count() >= s.truly, `wake ${wake}: counted ${s.count()} for ${s.truly}`);
  }
});

test("bytes given after a wake, short of the next probe, still count at the next wake (#426 review)", () => {
  // After a wake the count is a step above the last write; the next probe
  // can be almost two steps above it. Without a write a full step past
  // the last one, the bytes in between were lost at the next wake.
  const s = new Socket();
  s.give(1);
  s.give(S - 2);
  s.wake();
  s.give(S - 2);
  s.wake();
  assert.ok(s.count() >= s.truly, `counted ${s.count()} for ${s.truly} given`);
});

test("a device that reads everything is held to about a step, however often the relay wakes", () => {
  // Counting high costs a step per wake that had bytes given after it,
  // until the next answered probe: the answers are counted in the same
  // (high) terms, so they take the excess away again.
  const s = new Socket();
  for (let wake = 0; wake < 40; wake++) {
    for (let i = 0; i < 70; i++) s.give(64 * 1024);
    s.answerAll();
    assert.ok(s.unanswered() <= 2 * S, `wake ${wake}: ${s.unanswered()} unanswered`);
    s.wake();
    assert.ok(s.unanswered() <= 3 * S, `wake ${wake}, woken: ${s.unanswered()} unanswered`);
  }
});

test("waking with nothing given since adds nothing", () => {
  // A socket that sits idle across many wakes must not drift up to its
  // UNANSWERED limit (no probe would then ever bring it back down).
  const s = new Socket();
  s.give(1200);
  s.wake();
  const once = s.count();
  for (let i = 0; i < 50; i++) {
    s.wake();
    assert.equal(s.count(), once);
  }
});

test("a socket that was given nothing counts nothing after waking", () => {
  // Replaced sockets count against their key (KEY_UNANSWERED); an empty
  // one must not take a step of it.
  const s = new Socket();
  s.wake();
  assert.equal(s.count(), 0);
  assert.deepEqual(restore(null), { given: 0, answered: 0, written: undefined });
});

test("datagrams of every size, with wakes in between, never count low", () => {
  const sizes = [40, 1200, 64 * 1024, S - 1, S, S + 1];
  for (const size of sizes) {
    for (const every of [1, 3, 17]) {
      const s = new Socket();
      for (let i = 0; i < 60; i++) {
        s.give(size);
        if (i % every === 0) s.wake();
        assert.ok(s.count() >= s.truly, `${size}/${every}/${i}: ${s.count()} for ${s.truly}`);
      }
    }
  }
});

test("the attachment is written about once a step, not per datagram", () => {
  const s = new Socket();
  for (let i = 0; i < 1000; i++) s.give(64 * 1024);
  // 1000 datagrams of 64 KiB are about 15.6 steps: the first bytes, then
  // one write per step.
  assert.ok(s.writes <= 17, `${s.writes} writes`);
  assert.equal(s.probes, 15);
});
