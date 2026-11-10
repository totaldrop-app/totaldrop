// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// What a socket's attachment keeps of the bytes the Pair object gave it,
// so that the count survives the object waking from hibernation, however
// often (#190, #426). Kept apart from pair.ts so that it runs on Node's
// test runner, without the Workers runtime.

/** A probe goes into a socket's stream every PROBE_STEP bytes given to it
 * (pair.ts, "UNANSWERED"). */
export const PROBE_STEP = 4 * 1024 * 1024;

/** What a socket's attachment keeps of the bytes given to it. */
export interface Counted {
  /** Bytes given to the socket, as last written. It is written at the
   * first bytes, at each probe, and whenever the count is a full
   * PROBE_STEP past it, so the count is always less than a step above.
   * Unset: given nothing. */
  given?: number;
  /** Bytes given to it when its last answered probe went out. */
  answered?: number;
}

/** Whether giving a socket `before` to `after` bytes, with `written` last
 * written to its attachment, may change the attachment (see
 * {@link noteGiven}); when not, pair.ts does not read it at all. */
export function mayChange(written: number | undefined, before: number, after: number): boolean {
  return (
    written === undefined ||
    after - written >= PROBE_STEP ||
    Math.floor(after / PROBE_STEP) > Math.floor(before / PROBE_STEP)
  );
}

/** What `att` needs after the bytes given to a socket went from `before`
 * to `after`: a probe (each PROBE_STEP passed), a write of the count (the
 * first bytes, or a step past the last write; #426), or nothing. `att` is
 * updated; the caller writes it back unless the answer is "none". */
export function noteGiven(att: Counted, before: number, after: number): "probe" | "write" | "none" {
  const probe = Math.floor(after / PROBE_STEP) > Math.floor(before / PROBE_STEP);
  if (!probe && att.given !== undefined && after - att.given < PROBE_STEP) return "none";
  att.given = after;
  return probe ? "probe" : "write";
}

/** Bytes given to a socket and given at its last answered probe, read back
 * from its attachment after the object woke, and what was last written of
 * the first. Less than a step was given past the last write, so adding one
 * step counts high, never low, over any number of wakes; a socket that was
 * given nothing counts 0. Nothing is written back: a wake with nothing
 * given since adds nothing. */
export function restore(att: Counted | null | undefined): {
  given: number;
  answered: number;
  written: number | undefined;
} {
  const written = att?.given;
  return {
    given: written === undefined ? 0 : written + PROBE_STEP,
    answered: att?.answered ?? 0,
    written,
  };
}
