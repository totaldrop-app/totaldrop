// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Per-client rate limit (threat model T9), the same rule as the Rust
// server's (`server/totaldrop-rendezvous`). Kept free of Workers-only APIs
// so it runs under `node --test` as well.
//
// A client is an IPv4 address or an IPv6 /64, the block one subscriber
// usually gets. Counters live only in this object's memory, under a keyed
// hash of the client whose key is made at random when the object starts
// and is never stored. Nothing here is written to storage or logged: a
// counter is forgotten once its client has been quiet long enough to be
// back at a full burst, and all of them when the object is evicted.

/** What a client over its rate is told. */
export const TOO_MANY = "too many requests";

export type RateConfig = {
  /** Requests a client may make at once. */
  burst: number;
  /** One more request is allowed every `intervalMs` after that. */
  intervalMs: number;
  /** Most clients counted at once. A client beyond this is let through
   * uncounted: the global limits still hold, and refusing everyone new
   * would let one attacker with many addresses shut the service. */
  maxClients: number;
};

export const RATE_DEFAULTS: RateConfig = {
  burst: 60,
  intervalMs: 1_000,
  maxClients: 100_000,
};

/** The limits, with `burst` and `interval` overridden when they are
 * positive whole numbers (the end-to-end tests set them with
 * `wrangler dev --var`). Anything else keeps the default. */
export function rateConfig(burst: unknown, intervalMs: unknown): RateConfig {
  const positive = (v: unknown, d: number): number => {
    const n = typeof v === "string" && /^[1-9][0-9]{0,8}$/.test(v) ? Number(v) : NaN;
    return Number.isSafeInteger(n) ? n : d;
  };
  return {
    ...RATE_DEFAULTS,
    burst: positive(burst, RATE_DEFAULTS.burst),
    intervalMs: positive(intervalMs, RATE_DEFAULTS.intervalMs),
  };
}

/** The part of an address that names one client: an IPv4 address (also
 * when written as an IPv4-mapped IPv6 address) or the first 64 bits of an
 * IPv6 address. Anything that does not parse is one shared client, "". */
export function clientOf(ip: string | null): string {
  if (!ip) return "";
  if (/^[0-9]{1,3}(\.[0-9]{1,3}){3}$/.test(ip)) return ip;
  if (!/^[0-9a-fA-F:.]+$/.test(ip) || ip.length > 45) return "";
  const halves = ip.toLowerCase().split("::");
  if (halves.length > 2) return "";
  const groups = (s: string | undefined): string[] | null => {
    if (!s) return [];
    const out: string[] = [];
    for (const g of s.split(":")) {
      if (/^[0-9]{1,3}(\.[0-9]{1,3}){3}$/.test(g)) {
        // An embedded IPv4 address fills the last two groups.
        const b = g.split(".").map(Number);
        if (b.some((x) => x > 255)) return null;
        out.push(((b[0]! << 8) | b[1]!).toString(16), ((b[2]! << 8) | b[3]!).toString(16));
      } else if (/^[0-9a-f]{1,4}$/.test(g)) {
        out.push(g);
      } else {
        return null;
      }
    }
    return out;
  };
  const head = groups(halves[0]);
  const tail = halves.length === 2 ? groups(halves[1]) : [];
  if (!head || !tail) return "";
  const missing = 8 - head.length - tail.length;
  if (halves.length === 2 ? missing < 1 : missing !== 0) return "";
  const all = [...head, ...Array<string>(missing).fill("0"), ...tail].map((g) =>
    parseInt(g, 16),
  );
  // ::ffff:a.b.c.d is an IPv4 client.
  if (all.slice(0, 5).every((g) => g === 0) && all[5] === 0xffff) {
    return [all[6]! >> 8, all[6]! & 0xff, all[7]! >> 8, all[7]! & 0xff].join(".");
  }
  return all
    .slice(0, 4)
    .map((g) => g.toString(16))
    .join(":");
}

/** Counts requests per client with the generic cell rate algorithm: each
 * client has one number, the time its allowance is next fully spent
 * (`due`). A request is allowed while `due` stays within `burst`
 * intervals of now. */
export class RateLimiter {
  private readonly config: RateConfig;
  private readonly due = new Map<string, number>();
  /** Made at random here and never stored or exported. */
  private readonly key = crypto.subtle.importKey(
    "raw",
    crypto.getRandomValues(new Uint8Array(32)),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  private lastPrune = 0;

  constructor(config: RateConfig = RATE_DEFAULTS) {
    this.config = config;
  }

  /** Whether `client` (from `clientOf`) may make one more request now.
   * An allowed request is counted; a refused one is not. */
  async admit(client: string, now: number): Promise<boolean> {
    const id = await this.hash(client);
    this.prune(now);
    const { burst, intervalMs, maxClients } = this.config;
    const known = this.due.get(id);
    if (known === undefined && this.due.size >= maxClients) return true;
    const next = Math.max(known ?? now, now) + intervalMs;
    if (next - now > burst * intervalMs) return false;
    this.due.set(id, next);
    return true;
  }

  /** Clients counted now, for tests. */
  get size(): number {
    return this.due.size;
  }

  private async hash(client: string): Promise<string> {
    const mac = await crypto.subtle.sign("HMAC", await this.key, new TextEncoder().encode(client));
    return Array.from(new Uint8Array(mac, 0, 16), (b) => b.toString(16).padStart(2, "0")).join(
      "",
    );
  }

  /** Forget clients that are back at a full burst. Runs at most once per
   * interval, so a request costs the same however many clients there are,
   * except for that one pass. */
  private prune(now: number): void {
    if (now - this.lastPrune < this.config.intervalMs) return;
    this.lastPrune = now;
    for (const [id, due] of this.due) {
      if (due <= now) this.due.delete(id);
    }
  }
}
