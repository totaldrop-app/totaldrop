// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Calls from the Worker to a Durable Object that may be tried again. Kept
// free of Workers-only APIs so it runs under `node --test` as well.

/** Most tries at one call, and the pause before the second (it grows by
 * this much each time). */
export const ATTEMPTS = 3;
export const RETRY_DELAY = 50;

/** What a client is told when the call did not succeed. */
export const BUSY = "server busy";
export const FAILED = "server error, try again";

export type Outcome<T> = { ok: true; value: T } | { ok: false; reason: string };

type Flags = { retryable?: unknown; overloaded?: unknown; remote?: unknown; message?: unknown };

/** How workerd words an error it hides the details of. */
const INTERNAL_ERROR = "internal error; reference = ";

/** Whether a failed Durable Object call may be tried again. The runtime
 * marks its errors: `overloaded` means more calls only make it worse, so
 * never; `retryable` means a transient failure, so yes. An error without
 * `remote: true` was not thrown inside the object: it comes from the call
 * itself (the connection to the object, or building the request in this
 * Worker), and the calls passed here are cheap and safe to repeat, so yes.
 * An error from inside the object arrives with `remote: true`; if it is
 * workerd's "internal error; reference = …", it is the runtime's, never
 * ours (our code throws no such error), and it is what a new object's
 * first request failed with in #119, so yes. Anything else was thrown by
 * our own code and would fail the same way again. */
export function mayRetry(e: unknown): boolean {
  const f = (typeof e === "object" && e !== null ? e : {}) as Flags;
  if (f.overloaded === true) return false;
  if (f.retryable === true) return true;
  if (f.remote !== true) return true;
  return typeof f.message === "string" && f.message.startsWith(INTERNAL_ERROR);
}

/** Run `call`, trying again as `mayRetry` allows. Only a thrown error is
 * tried again: whatever `call` returns, even an error response, is the
 * answer. `call` must be safe to repeat. */
export async function withRetry<T>(
  call: () => Promise<T>,
  sleep: (ms: number) => Promise<void> = (ms) => new Promise((r) => setTimeout(r, ms)),
): Promise<Outcome<T>> {
  for (let attempt = 1; ; attempt++) {
    try {
      return { ok: true, value: await call() };
    } catch (e) {
      const overloaded = (e as Flags | null)?.overloaded === true;
      if (!mayRetry(e) || attempt >= ATTEMPTS) {
        return { ok: false, reason: overloaded ? BUSY : FAILED };
      }
      await sleep(RETRY_DELAY * attempt);
    }
  }
}
