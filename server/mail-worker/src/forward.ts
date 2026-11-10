// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Forwards one message to every maintainer (#415). A routing rule in
// Cloudflare Email Routing reaches one destination only, so the rules for
// hello@ (and any other project address) send mail to this Worker, and the
// Worker forwards it once to each configured address. Kept free of
// Workers-only APIs so that Node's test runner can load it.

/** At most this many destinations. Each `forward()` is one more delivery
 * of the same message, so the list stays short. */
export const MAX_RECIPIENTS = 10;

/** Mail for these domains comes back to Email Routing: forwarding to them
 * would loop. */
const OWN_DOMAIN = "totaldrop.app";

/** One address: a local part and a domain with at least one dot, no
 * spaces, commas or angle brackets. Cloudflare checks the address again
 * (it must be a verified destination); this only rejects a mistyped
 * secret before any mail is sent. */
const ADDRESS = /^[^\s@,<>"]+@(?:[A-Za-z0-9](?:[A-Za-z0-9-]*[A-Za-z0-9])?\.)+[A-Za-z]{2,}$/;

/** The destinations in `FORWARD_TO`: addresses separated by commas,
 * spaces or new lines, compared without case, each kept once, in order.
 * Throws when there are none, too many, or one is not an address or is on
 * our own domain, so a broken setting fails closed instead of sending
 * mail somewhere unexpected. The message never contains an address. */
export function recipients(raw: string | undefined): string[] {
  const parts = (raw ?? "").split(/[\s,]+/).filter((p) => p !== "");
  if (parts.length === 0) throw new Error("FORWARD_TO is not set");
  const seen = new Set<string>();
  const out: string[] = [];
  for (const [i, part] of parts.entries()) {
    if (part.length > 254 || !ADDRESS.test(part)) {
      throw new Error(`FORWARD_TO entry ${i + 1} is not an email address`);
    }
    const key = part.toLowerCase();
    const domain = key.slice(key.lastIndexOf("@") + 1);
    if (domain === OWN_DOMAIN || domain.endsWith(`.${OWN_DOMAIN}`)) {
      throw new Error(`FORWARD_TO entry ${i + 1} is on ${OWN_DOMAIN} and would loop`);
    }
    if (seen.has(key)) continue;
    seen.add(key);
    out.push(part);
  }
  if (out.length > MAX_RECIPIENTS) {
    throw new Error(`FORWARD_TO has ${out.length} addresses, at most ${MAX_RECIPIENTS}`);
  }
  return out;
}

/** The part of an incoming message this module uses. */
export interface Forwardable {
  forward(rcptTo: string): Promise<unknown>;
}

/** Forward `message` to each address, all at once. One failed
 * destination (for example one not verified yet) does not stop the
 * others. When every destination fails it throws, so Email Routing
 * records an error instead of a delivery. Returns how many failed. */
export async function forwardAll(message: Forwardable, to: readonly string[]): Promise<number> {
  if (to.length === 0) throw new Error("no destination to forward to");
  const results = await Promise.allSettled(to.map((r) => message.forward(r)));
  const failed = results.filter((r) => r.status === "rejected").length;
  if (failed === to.length) {
    throw new Error(`forwarding failed for all ${failed} destinations`);
  }
  return failed;
}

/** Where a partial failure is reported. Only a count goes there, never
 * an address, sender or subject. */
export type Report = (line: string) => void;

/** What the Worker does with one message: read `FORWARD_TO` (on every
 * message, so a Worker that somehow runs without it fails closed instead
 * of forwarding anywhere), forward to each address, and report how many
 * destinations failed when some did. */
export async function handle(
  message: Forwardable,
  forwardTo: string | undefined,
  report: Report,
): Promise<void> {
  const failed = await forwardAll(message, recipients(forwardTo));
  if (failed > 0) report(`forwarding failed for ${failed} destination(s)`);
}
