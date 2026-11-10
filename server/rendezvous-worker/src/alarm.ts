// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// When the nameplate directory's alarm runs (`directory.ts`). Kept free of
// Workers-only APIs so it runs under `node --test` as well.
//
// The alarm expires nameplates, frees rested numbers and forgets the
// per-client counters of `limit.ts`. It must stay set while any of these
// exist: a counter is otherwise kept until the object is evicted, which
// Cloudflare does not promise to do in time (#393). With one run a minute,
// a counter, which is due at most one burst (60 s) after its client's last
// request, is gone within about two minutes of it, as the privacy policy
// says.

/** How often the alarm runs while there is work for it. */
export const ALARM_EVERY_MS = 60_000;

/** The part of `DurableObjectStorage` the alarm needs. */
export interface AlarmStorage {
  getAlarm(): Promise<number | null>;
  setAlarm(scheduledTime: number): Promise<void>;
}

/** Set the alarm for one interval from `now`, unless one is already set. */
export async function ensureAlarm(storage: AlarmStorage, now: number): Promise<void> {
  if ((await storage.getAlarm()) === null) await storage.setAlarm(now + ALARM_EVERY_MS);
}

/** After a request the rate limit counted: keep the alarm set while any
 * client is counted, so its counter is forgotten in time even when no
 * nameplate is in use. */
export async function afterCounted(
  storage: AlarmStorage,
  counted: number,
  now: number,
): Promise<void> {
  if (counted > 0) await ensureAlarm(storage, now);
}

/** When the alarm that ran at `now` should run again, or null for not at
 * all: while nameplates are active or resting, or clients are counted. */
export function nextAlarm(now: number, nameplatesLeft: boolean, counted: number): number | null {
  return nameplatesLeft || counted > 0 ? now + ALARM_EVERY_MS : null;
}
