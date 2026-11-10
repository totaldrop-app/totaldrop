// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// What the Pair object knows, after waking, of the bytes it gave a socket
// (#190, #426). Kept apart from pair.ts so that it runs on Node's test
// runner, without the Workers runtime.

/** A probe goes into a socket's stream every PROBE_STEP bytes given to it
 * (pair.ts, "UNANSWERED"). */
export const PROBE_STEP = 4 * 1024 * 1024;

/** What a socket's attachment keeps of the bytes given to it. */
export interface Counted {
  /** Bytes given to the socket when its last probe went out. */
  given?: number;
  /** Bytes given to it when its last answered probe went out. */
  answered?: number;
}

/** Bytes given to a socket, and given at its last answered probe, as far
 * as its attachment tells after the object woke. Fewer than PROBE_STEP
 * bytes can follow the last probe (or, before the first probe, the start)
 * without a write to the attachment, so adding one step counts high, never
 * low. */
export function restored(att: Counted | null | undefined): [number, number] {
  const given = (att?.given ?? 0) + PROBE_STEP;
  return [given, att?.answered ?? 0];
}
