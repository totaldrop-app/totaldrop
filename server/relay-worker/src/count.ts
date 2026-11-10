// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// What a socket's attachment keeps of the bytes the Pair object gave it,
// so that the count survives the object waking from hibernation (#190,
// #426). Kept apart from pair.ts so that it runs on Node's test runner,
// without the Workers runtime.

/** A probe goes into a socket's stream every PROBE_STEP bytes given to it
 * (pair.ts, "UNANSWERED"). */
export const PROBE_STEP = 4 * 1024 * 1024;

/** What a socket's attachment keeps of the bytes given to it. */
export interface Counted {
  /** Bytes given to the socket when its last probe went out; 0 once it
   * was given anything before its first probe. Unset: given nothing. */
  given?: number;
  /** Bytes given to it when its last answered probe went out. */
  answered?: number;
}

/** What `att` needs after the bytes given to a socket went from `before`
 * to `after`: a probe (each PROBE_STEP passed), a note that it was given
 * something at all (the first bytes; #426), or nothing. `att` is updated;
 * the caller writes it back unless the answer is "none". */
export function noteGiven(att: Counted, before: number, after: number): "probe" | "first" | "none" {
  if (Math.floor(after / PROBE_STEP) > Math.floor(before / PROBE_STEP)) {
    att.given = after;
    return "probe";
  }
  if (att.given === undefined && after > 0) {
    att.given = 0;
    return "first";
  }
  return "none";
}

/** Bytes given to a socket, and given at its last answered probe, as far
 * as its attachment tells after the object woke. Fewer than PROBE_STEP
 * bytes can follow the last write to the attachment, so adding one step
 * counts high, never low; a socket that was given nothing counts 0. */
export function restored(att: Counted | null | undefined): [number, number] {
  const given = att?.given === undefined ? 0 : att.given + PROBE_STEP;
  return [given, att?.answered ?? 0];
}
