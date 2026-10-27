// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The small part of CBOR (RFC 8949) the rendezvous protocol needs, written
// to match `totaldrop-proto/src/wire.rs` exactly:
//
// - decoding accepts any well-formed CBOR item with definite lengths, so
//   unknown fields of any type can be skipped, but rejects trailing bytes,
//   nesting deeper than 16, maps with more than 64 keys, duplicate integer
//   keys and invalid UTF-8;
// - encoding writes maps with unsigned integer keys in ascending order and
//   shortest-form lengths: the deterministic encoding of RFC 8949 4.2.

export type Value =
  | { kind: "uint"; value: bigint }
  | { kind: "nint" }
  | { kind: "bytes"; value: Uint8Array }
  | { kind: "text"; value: string }
  | { kind: "array"; value: Value[] }
  | { kind: "map"; value: Map<number, Value> }
  | { kind: "other" };

export class CborError extends Error {}

const MAX_DEPTH = 16;
const MAX_MAP_KEYS = 64;
const utf8 = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
const utf8Encoder = new TextEncoder();

class Reader {
  pos = 0;
  readonly buf: Uint8Array;
  constructor(buf: Uint8Array) {
    this.buf = buf;
  }

  byte(): number {
    if (this.pos >= this.buf.length) throw new CborError("truncated");
    return this.buf[this.pos++]!;
  }

  take(n: number): Uint8Array {
    if (n > this.buf.length - this.pos) throw new CborError("truncated");
    const out = this.buf.subarray(this.pos, this.pos + n);
    this.pos += n;
    return out;
  }

  done(): boolean {
    return this.pos === this.buf.length;
  }

  /** The argument of a head byte; `null` means indefinite length. */
  argument(info: number): bigint | null {
    if (info < 24) return BigInt(info);
    const size = info === 24 ? 1 : info === 25 ? 2 : info === 26 ? 4 : info === 27 ? 8 : 0;
    if (info === 31) return null;
    if (size === 0) throw new CborError("reserved additional information");
    let v = 0n;
    for (const b of this.take(size)) v = (v << 8n) | BigInt(b);
    return v;
  }
}

function length(r: Reader, info: number): number {
  const n = r.argument(info);
  if (n === null) throw new CborError("indefinite lengths are not used");
  if (n > BigInt(Number.MAX_SAFE_INTEGER)) throw new CborError("length too large");
  return Number(n);
}

function item(r: Reader, depth: number): Value {
  if (depth > MAX_DEPTH) throw new CborError("nested too deeply");
  const head = r.byte();
  const major = head >> 5;
  const info = head & 0x1f;
  switch (major) {
    case 0: {
      const v = r.argument(info);
      if (v === null) throw new CborError("bad integer");
      return { kind: "uint", value: v };
    }
    case 1:
      if (r.argument(info) === null) throw new CborError("bad integer");
      return { kind: "nint" };
    case 2:
      return { kind: "bytes", value: r.take(length(r, info)).slice() };
    case 3:
      try {
        return { kind: "text", value: utf8.decode(r.take(length(r, info))) };
      } catch {
        throw new CborError("invalid UTF-8");
      }
    case 4: {
      const n = length(r, info);
      const out: Value[] = [];
      for (let i = 0; i < n; i++) out.push(item(r, depth + 1));
      return { kind: "array", value: out };
    }
    case 5: {
      const n = length(r, info);
      if (n > MAX_MAP_KEYS) throw new CborError("too many map keys");
      const out = new Map<number, Value>();
      for (let i = 0; i < n; i++) {
        const k = item(r, depth + 1);
        const v = item(r, depth + 1);
        // Non-integer and huge keys are unknown keys: ignored.
        if (k.kind !== "uint" || k.value > BigInt(Number.MAX_SAFE_INTEGER)) continue;
        const key = Number(k.value);
        if (out.has(key)) throw new CborError("duplicate key");
        out.set(key, v);
      }
      return { kind: "map", value: out };
    }
    case 6:
      if (r.argument(info) === null) throw new CborError("bad tag");
      item(r, depth + 1);
      return { kind: "other" };
    default: {
      // Major type 7: simple values and floats.
      if (info === 24) r.take(1);
      else if (info === 25) r.take(2);
      else if (info === 26) r.take(4);
      else if (info === 27) r.take(8);
      else if (info >= 28) throw new CborError("reserved simple value");
      return { kind: "other" };
    }
  }
}

/** Decode exactly one map; anything else is an error. */
export function decodeMap(input: Uint8Array): Map<number, Value> {
  const r = new Reader(input);
  const v = item(r, 0);
  if (!r.done()) throw new CborError("trailing bytes");
  if (v.kind !== "map") throw new CborError("not a map");
  return v.value;
}

// ---------------------------------------------------------------------------
// Encoding

export type Field = number | string | Uint8Array;

function head(major: number, n: number, out: number[]): void {
  const m = major << 5;
  if (n < 24) out.push(m | n);
  else if (n < 0x100) out.push(m | 24, n);
  else if (n < 0x10000) out.push(m | 25, n >> 8, n & 0xff);
  else if (n < 0x1_0000_0000)
    out.push(m | 26, (n >>> 24) & 0xff, (n >>> 16) & 0xff, (n >>> 8) & 0xff, n & 0xff);
  else throw new CborError("value too large for this encoder");
}

/** Encode a map of unsigned integer keys to unsigned integers, text or bytes. */
export function encodeMap(fields: Array<[number, Field]>): Uint8Array {
  const sorted = [...fields].sort((a, b) => a[0] - b[0]);
  const out: number[] = [];
  head(5, sorted.length, out);
  for (const [k, v] of sorted) {
    head(0, k, out);
    if (typeof v === "number") {
      if (!Number.isInteger(v) || v < 0) throw new CborError("only unsigned integers");
      head(0, v, out);
    } else if (typeof v === "string") {
      const b = utf8Encoder.encode(v);
      head(3, b.length, out);
      for (const x of b) out.push(x);
    } else {
      head(2, v.length, out);
      for (const x of v) out.push(x);
    }
  }
  return new Uint8Array(out);
}
