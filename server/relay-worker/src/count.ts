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
  /** Bytes given to the socket when its last probe went out: at its first
   * bytes, then each time the count is a full PROBE_STEP past it, so the
   * count is always less than a step above. Unset: given nothing. */
  given?: number;
  /** Bytes given to it when its last answered probe went out. */
  answered?: number;
}

/** Whether giving a socket bytes up to `after`, with `written` last
 * written to its attachment, may change the attachment (see
 * {@link noteGiven}); when not, pair.ts does not read it at all. */
export function mayChange(written: number | undefined, after: number): boolean {
  return written === undefined || after - written >= PROBE_STEP;
}

/** Whether a socket that was given `after` bytes needs a probe, and with
 * it a write of the count to `att` (which is updated): at its first bytes,
 * then whenever the count is a full PROBE_STEP past the last write. So the
 * count is always less than a step above what was written, and each write
 * comes with a probe, whose answer brings the count of unanswered bytes
 * back down also after a wake has counted a step high (#426). */
export function noteGiven(att: Counted, after: number): boolean {
  if (att.given !== undefined && after - att.given < PROBE_STEP) return false;
  att.given = after;
  return true;
}

/** One step of giving a socket bytes, shared by pair.ts and its tests:
 * with `written` last written to its attachment and `after` bytes given
 * in all, read the attachment only when it may change ({@link mayChange}),
 * note the bytes in it ({@link noteGiven}), and say what is now written
 * and whether a probe goes out. `att` is the attachment read, to send the
 * probe and write it back; null when it was not read. */
export function give<T extends Counted>(
  written: number | undefined,
  after: number,
  read: () => T | null,
): { written: number | undefined; att: T | null; probe: boolean } {
  if (!mayChange(written, after)) return { written, att: null, probe: false };
  const att = read();
  if (!att) return { written, att: null, probe: false };
  const probe = noteGiven(att, after);
  return { written: att.given, att, probe };
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
