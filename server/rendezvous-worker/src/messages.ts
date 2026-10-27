// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Rendezvous messages (spec section 5), matching `ClientMessage` and
// `ServerMessage` in `totaldrop-proto/src/wire.rs`.

import { decodeMap, encodeMap, type Value } from "./cbor";

export const MAX_PHASE = 32;
export const MAX_MAILBOX_BODY = 16 * 1024;

export type Mood = "happy" | "lonely" | "scary";

export type ClientMessage =
  | { type: "allocate" }
  | { type: "claim"; nameplate: number; token: Uint8Array | null }
  | { type: "add"; phase: string; body: Uint8Array }
  | { type: "close"; mood: Mood };

export class WireError extends Error {}

function text(m: Map<number, Value>, k: number): string {
  const v = m.get(k);
  if (!v || v.kind !== "text") throw new WireError(`field ${k}`);
  return v.value;
}

function bytes(m: Map<number, Value>, k: number): Uint8Array {
  const v = m.get(k);
  if (!v || v.kind !== "bytes") throw new WireError(`field ${k}`);
  return v.value;
}

function u32(m: Map<number, Value>, k: number): number {
  const v = m.get(k);
  if (!v || v.kind !== "uint" || v.value > 0xffff_ffffn) throw new WireError(`field ${k}`);
  return Number(v.value);
}

function mood(s: string): Mood {
  if (s === "happy" || s === "lonely" || s === "scary") return s;
  throw new WireError("mood");
}

export function decodeClient(input: Uint8Array): ClientMessage {
  const m = decodeMap(input);
  const kind = text(m, 0);
  switch (kind) {
    case "allocate":
      return { type: "allocate" };
    case "claim": {
      let token: Uint8Array | null = null;
      if (m.has(2)) {
        token = bytes(m, 2);
        if (token.length !== 16) throw new WireError("field 2");
      }
      return { type: "claim", nameplate: u32(m, 1), token };
    }
    case "add": {
      const phase = text(m, 1);
      if (phase.length === 0 || new TextEncoder().encode(phase).length > MAX_PHASE)
        throw new WireError("field 1");
      const body = bytes(m, 2);
      if (body.length > MAX_MAILBOX_BODY) throw new WireError("mailbox body");
      return { type: "add", phase, body };
    }
    case "close":
      return { type: "close", mood: mood(text(m, 1)) };
    default:
      throw new WireError("unknown type");
  }
}

export const server = {
  allocated: (nameplate: number, token: Uint8Array) =>
    encodeMap([
      [0, "allocated"],
      [1, nameplate],
      [2, token],
    ]),
  claimed: (mailbox: Uint8Array) =>
    encodeMap([
      [0, "claimed"],
      [1, mailbox],
    ]),
  message: (phase: string, body: Uint8Array) =>
    encodeMap([
      [0, "message"],
      [1, phase],
      [2, body],
    ]),
  peerClosed: (m: Mood) =>
    encodeMap([
      [0, "peer_closed"],
      [1, m],
    ]),
  error: (reason: string) =>
    encodeMap([
      [0, "error"],
      [1, reason],
    ]),
};

export function hex(b: Uint8Array): string {
  return [...b].map((x) => x.toString(16).padStart(2, "0")).join("");
}

export function unhex(s: string): Uint8Array {
  if (!/^([0-9a-f]{2})*$/.test(s)) throw new WireError("hex");
  const out = new Uint8Array(s.length / 2);
  for (let i = 0; i < out.length; i++) out[i] = parseInt(s.slice(i * 2, i * 2 + 2), 16);
  return out;
}

export function randomBytes(n: number): Uint8Array {
  const b = new Uint8Array(n);
  crypto.getRandomValues(b);
  return b;
}

/** Constant-time comparison of two byte strings of equal length. */
export function sameBytes(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) diff |= a[i]! ^ b[i]!;
  return diff === 0;
}
