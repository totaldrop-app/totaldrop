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

import type { RateLimiter } from "./limit";

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

/** Count one request of `client` at `now`; whether it is allowed. While
 * any client is counted the alarm stays set, so its counter is forgotten
 * in time even when no nameplate is in use. `Directory.allocate` and
 * `Directory.lookup` count through this. */
export async function admitCounted(
  storage: AlarmStorage,
  limiter: RateLimiter,
  client: string,
  now: number,
): Promise<boolean> {
  const allowed = await limiter.admit(client, now);
  if (limiter.size > 0) await ensureAlarm(storage, now);
  return allowed;
}

/** The rate limit's part of the alarm that runs at `now`, after the
 * directory has expired and freed its nameplates: forget the clients
 * back at a full burst, then set the alarm again while nameplates are
 * active or resting (`nameplatesLeft`) or clients are still counted. */
export async function finishAlarm(
  storage: AlarmStorage,
  limiter: RateLimiter,
  nameplatesLeft: boolean,
  now: number,
): Promise<void> {
  limiter.sweep(now);
  if (nameplatesLeft || limiter.size > 0) await storage.setAlarm(now + ALARM_EVERY_MS);
}
