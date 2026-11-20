// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Hibernation wake, socket replacement, delayed close/message events, alarm
// cleanup, and rolling-deploy wake tests for PairCore (#190).

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import {
  AUTH_CONTEXT,
  AUTH_HEADER,
  CONNECT_CONTEXT,
  MAGIC,
  MAX_SOCKETS,
  MAX_WAITING,
  OK,
  OPEN,
  PairCore,
  handleRelayRequest,
  parseAuthHeader,
  type Attachment,
  type PairEnv,
  type PairHost,
  type PairNamespace,
  type RelaySocket,
} from "../src/pair-core.ts";
import type { TotalsKv } from "../src/totals.ts";

const CLOSING = 2;
const CLOSED = 3;

function hex(b: Uint8Array): string {
  return Array.from(b, (x) => x.toString(16).padStart(2, "0")).join("");
}

function unhex(s: string): Uint8Array {
  const out = new Uint8Array(s.length / 2);
  for (let i = 0; i < out.length; i++) out[i] = parseInt(s.slice(2 * i, 2 * i + 2), 16);
  return out;
}

interface StoredSocketState {
  readyState: number;
  tags: string[];
  attachment: unknown;
  sent: Uint8Array[];
  closeCalls: { code?: number; reason?: string }[];
}

class FakeSocket implements RelaySocket {
  readonly backing: StoredSocketState;

  constructor(backing?: StoredSocketState) {
    this.backing = backing ?? {
      readyState: OPEN,
      tags: [],
      attachment: null,
      sent: [],
      closeCalls: [],
    };
  }

  get readyState(): number {
    return this.backing.readyState;
  }

  set readyState(v: number) {
    this.backing.readyState = v;
  }

  send(message: ArrayBuffer | ArrayBufferView | string): void {
    if (typeof message === "string") throw new Error("unexpected text frame");
    const view = ArrayBuffer.isView(message)
      ? new Uint8Array(message.buffer, message.byteOffset, message.byteLength)
      : new Uint8Array(message);
    this.backing.sent.push(new Uint8Array(view));
  }

  close(code?: number, reason?: string): void {
    this.backing.closeCalls.push({ code, reason });
    if (this.backing.readyState === OPEN) this.backing.readyState = CLOSING;
  }

  serializeAttachment(value: unknown): void {
    this.backing.attachment = structuredClone(value);
  }

  deserializeAttachment(): unknown {
    return structuredClone(this.backing.attachment);
  }
}

class FakeDurableObjectEnv {
  private kvMap = new Map<string, unknown>();
  private socketBackings: StoredSocketState[] = [];
  private currentSockets = new Map<StoredSocketState, FakeSocket>();
  alarmAt: number | null = null;
  deleteAlarmCalls = 0;
  deleteAllCalls = 0;
  onDeleteAll?: () => Promise<void>;
  onDeleteAllSync?: () => void;
  onDeleteAlarm?: () => Promise<void>;
  onKvDelete?: (key: string) => void;
  onGetWebSockets?: () => void;
  env: PairEnv;

  readonly kv: TotalsKv = {
    get: <T = unknown>(key: string): T | undefined => {
      const v = this.kvMap.get(key);
      return v === undefined ? undefined : structuredClone(v as T);
    },
    put: <T>(key: string, value: T): void => {
      this.kvMap.set(key, structuredClone(value));
    },
    delete: (key: string): boolean => {
      if (this.onKvDelete) this.onKvDelete(key);
      return this.kvMap.delete(key);
    },
  };

  constructor(env: PairEnv = {}) {
    this.env = env;
  }

  get kvSize(): number {
    return this.kvMap.size;
  }

  addSocket(ws: FakeSocket, tags: string[]): void {
    ws.backing.tags = [...tags];
    this.socketBackings.push(ws.backing);
    this.currentSockets.set(ws.backing, ws);
  }

  private wrap(b: StoredSocketState): FakeSocket {
    let s = this.currentSockets.get(b);
    if (!s) {
      s = new FakeSocket(b);
      this.currentSockets.set(b, s);
    }
    return s;
  }

  /** Simulate Durable Object hibernation: return a brand-new PairCore whose
   * WeakSet/WeakMap/Map memory is empty and whose getWebSockets() returns
   * fresh wrapper instances over the persisted attachments & KV storage. */
  wake(): { core: PairCore<FakeSocket>; socketOf: (b: StoredSocketState) => FakeSocket } {
    this.currentSockets = new Map();
    const host: PairHost<FakeSocket> = {
      getWebSockets: () => {
        if (this.onGetWebSockets) this.onGetWebSockets();
        return this.socketBackings.filter((b) => b.readyState !== CLOSED).map((b) => this.wrap(b));
      },
      acceptWebSocket: (ws, tags) => {
        ws.backing.tags = [...tags];
        this.socketBackings.push(ws.backing);
        this.currentSockets.set(ws.backing, ws);
      },
      getTags: (ws) => ws.backing.tags,
      storage: {
        kv: this.kv,
        setAlarm: async (t) => {
          this.alarmAt = t;
        },
        deleteAlarm: async () => {
          if (this.onDeleteAlarm) await this.onDeleteAlarm();
          this.deleteAlarmCalls++;
          this.alarmAt = null;
        },
        deleteAll: (): Promise<void> => {
          if (this.onDeleteAllSync) this.onDeleteAllSync();
          return (async () => {
            if (this.onDeleteAll) await this.onDeleteAll();
            this.deleteAllCalls++;
            this.kvMap.clear();
          })();
        },
      },
      env: this.env,
    };
    return {
      core: new PairCore(host),
      socketOf: (b) => this.wrap(b),
    };
  }
}

interface Identity {
  pub: Uint8Array;
  hex: string;
  priv: import("node:crypto").webcrypto.CryptoKey;
}

async function generatePairIdentities(): Promise<[Identity, Identity, string]> {
  const make = async (): Promise<Identity> => {
    const kp = await crypto.subtle.generateKey("Ed25519", true, ["sign", "verify"]);
    if (!("publicKey" in kp)) throw new Error("expected Ed25519 key pair");
    const raw = new Uint8Array(await crypto.subtle.exportKey("raw", kp.publicKey));
    return { pub: raw, hex: hex(raw), priv: kp.privateKey };
  };
  const x = await make();
  const y = await make();
  const [a, b] = x.hex < y.hex ? [x, y] : [y, x];
  return [a, b, `${a.hex}-${b.hex}`];
}

async function signHello(id: Identity, challengeFrame: Uint8Array): Promise<ArrayBuffer> {
  assert.equal(challengeFrame.length, 36);
  const challenge = challengeFrame.subarray(4);
  const signed = new Uint8Array([...AUTH_CONTEXT, ...challenge]);
  const sig = new Uint8Array(await crypto.subtle.sign({ name: "Ed25519" }, id.priv, signed));
  const out = new Uint8Array(4 + 32 + 64);
  out.set(MAGIC, 0);
  out.set(id.pub, 4);
  out.set(sig, 36);
  return out.buffer;
}

async function signConnectHeader(id: Identity, pairName: string): Promise<string> {
  const enc = new TextEncoder();
  const signed = new Uint8Array([...CONNECT_CONTEXT, ...enc.encode(pairName)]);
  const sig = new Uint8Array(await crypto.subtle.sign({ name: "Ed25519" }, id.priv, signed));
  return `${id.hex}.${hex(sig)}`;
}

function makeDatagramFrame(destHex: string, payloadLen: number, fill: number): ArrayBuffer {
  const out = new Uint8Array(32 + 2 + payloadLen);
  out.set(unhex(destHex), 0);
  out[32] = (payloadLen >> 8) & 0xff;
  out[33] = payloadLen & 0xff;
  out.fill(fill, 34);
  return out.buffer;
}

test("hibernation between socket replacement and delayed close/message keeps replacement and peer alive (#190)", async () => {
  const [a, b, pairName] = await generatePairIdentities();
  const doEnv = new FakeDurableObjectEnv({ SESSION_BYTES: String(10 * 1024 * 1024) });
  let { core, socketOf } = doEnv.wake();

  let sA1!: FakeSocket;
  let sB!: FakeSocket;
  assert.equal(
    await core.accept(
      pairName,
      () => {
        sA1 = new FakeSocket();
        return sA1;
      },
      1_000,
    ),
    101,
  );
  assert.equal(
    await core.accept(
      pairName,
      () => {
        sB = new FakeSocket();
        return sB;
      },
      1_000,
    ),
    101,
  );

  await core.onMessage(sA1, await signHello(a, sA1.backing.sent[0]!), 1_010);
  await core.onMessage(sB, await signHello(b, sB.backing.sent[0]!), 1_010);
  assert.deepEqual(sA1.backing.sent[1], OK);
  assert.deepEqual(sB.backing.sent[1], OK);

  // Key A sends 600 B on sA1, then sA1 dies unnoticed and Key A reconnects on sA2.
  await core.onMessage(sA1, makeDatagramFrame(b.hex, 600, 0xaa), 1_020);
  let sA2!: FakeSocket;
  assert.equal(
    await core.accept(
      pairName,
      () => {
        sA2 = new FakeSocket();
        return sA2;
      },
      1_050,
    ),
    101,
  );
  await core.onMessage(sA2, await signHello(a, sA2.backing.sent[0]!), 1_060);
  assert.deepEqual(sA2.backing.sent[1], OK);
  assert.equal((sA1.deserializeAttachment() as Attachment).closed, true);

  // sA2 sends 200 B so side 0 has unwritten bytes in memory, then DO hibernates!
  await core.onMessage(sA2, makeDatagramFrame(b.hex, 200, 0xbb), 1_070);

  ({ core, socketOf } = doEnv.wake());
  const wokenA1 = socketOf(sA1.backing);
  const wokenA2 = socketOf(sA2.backing);
  const wokenB = socketOf(sB.backing);

  // Even if wokenA1 reports OPEN briefly before tearing down, its delayed
  // message and close must NOT forward or tear down wokenA2 / wokenB.
  wokenA1.readyState = OPEN;
  const bBefore = wokenB.backing.sent.length;
  await core.onMessage(wokenA1, makeDatagramFrame(b.hex, 300, 0xcc), 1_100);
  assert.equal(wokenB.backing.sent.length, bBefore, "replaced socket frame must be ignored");

  wokenA1.readyState = CLOSED;
  await core.end(wokenA1);

  assert.equal(wokenA2.readyState, OPEN, "replacement socket must stay open");
  assert.equal(wokenB.readyState, OPEN, "peer socket must stay open");
  assert.equal((wokenA2.deserializeAttachment() as Attachment).closed, undefined);
  assert.equal((wokenB.deserializeAttachment() as Attachment).closed, undefined);

  // Both directions still forward between wokenA2 and wokenB after the delayed close.
  await core.onMessage(wokenA2, makeDatagramFrame(b.hex, 150, 0xdd), 1_120);
  assert.ok(wokenB.backing.sent.length > bBefore, "wokenA2 -> wokenB still forwards");
  const a2Before = wokenA2.backing.sent.length;
  await core.onMessage(wokenB, makeDatagramFrame(a.hex, 150, 0xee), 1_130);
  assert.ok(wokenA2.backing.sent.length > a2Before, "wokenB -> wokenA2 still forwards");
});

test("replacing a socket already in CLOSING persists closed: true and does not cascade", async () => {
  const [a, b, pairName] = await generatePairIdentities();
  const doEnv = new FakeDurableObjectEnv();
  let { core, socketOf } = doEnv.wake();

  let sA1!: FakeSocket;
  let sB!: FakeSocket;
  await core.accept(pairName, () => (sA1 = new FakeSocket()), 2_000);
  await core.accept(pairName, () => (sB = new FakeSocket()), 2_000);
  await core.onMessage(sA1, await signHello(a, sA1.backing.sent[0]!), 2_010);
  await core.onMessage(sB, await signHello(b, sB.backing.sent[0]!), 2_010);

  // sA1 enters CLOSING right before sA2 signs in.
  sA1.readyState = CLOSING;
  let sA2!: FakeSocket;
  await core.accept(pairName, () => (sA2 = new FakeSocket()), 2_020);
  await core.onMessage(sA2, await signHello(a, sA2.backing.sent[0]!), 2_030);

  // Hibernate, then deliver webSocketClose(sA1).
  ({ core, socketOf } = doEnv.wake());
  sA1.readyState = CLOSED;
  await core.end(socketOf(sA1.backing));

  assert.equal(socketOf(sA2.backing).readyState, OPEN);
  assert.equal(socketOf(sB.backing).readyState, OPEN);
});

test("rolling-deploy wake on pre-existing signed-in sockets seeds totals and expireSession deletes alarm", async () => {
  const [a, b, pairName] = await generatePairIdentities();
  const doEnv = new FakeDurableObjectEnv({ SESSION_BYTES: "1500", SESSION_TIME_MS: "900" });
  let { core, socketOf } = doEnv.wake();

  let sA!: FakeSocket;
  let sB!: FakeSocket;
  await core.accept(pairName, () => (sA = new FakeSocket()), 10_000);
  await core.accept(pairName, () => (sB = new FakeSocket()), 10_000);
  await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 10_010);
  await core.onMessage(sB, await signHello(b, sB.backing.sent[0]!), 10_010);

  // Simulate pre-#190 state: wipe KV `"totals"`, set legacy `att.sent = 1000` on sA, and wake.
  doEnv.kv.delete("totals");
  const legacyAtt = sA.deserializeAttachment() as Attachment;
  legacyAtt.sent = 1_000;
  sA.serializeAttachment(legacyAtt);

  ({ core, socketOf } = doEnv.wake());
  const wokenA = socketOf(sA.backing);
  const wokenB = socketOf(sB.backing);

  // First datagram (300 B -> 1,300 + 34 = 1,334 <= 1,500) initializes totals instead of failing.
  await core.onMessage(wokenA, makeDatagramFrame(b.hex, 300, 0x11), 10_200);
  assert.equal(wokenA.readyState, OPEN);
  assert.equal(wokenB.readyState, OPEN);

  // Second datagram (300 B -> > 1,500) hits the seeded limit cleanly.
  await core.onMessage(wokenA, makeDatagramFrame(b.hex, 300, 0x22), 10_250);
  assert.notEqual(wokenA.readyState, OPEN);

  // Alarm at pair deadline (10_000 + 900 = 10_900) wipes storage and deletes the alarm.
  await core.alarm(10_900);
  assert.equal(doEnv.kvSize, 0);
  assert.equal(doEnv.alarmAt, null);
  assert.ok(doEnv.deleteAllCalls >= 1);
  assert.ok(doEnv.deleteAlarmCalls >= 1);
});

test("alarm follows sign-in start time rather than accept time, and post-expiry accept succeeds before alarm (#190)", async () => {
  const [a, b, pairName] = await generatePairIdentities();
  const sessionTime = 900;
  const doEnv = new FakeDurableObjectEnv({ SESSION_TIME_MS: String(sessionTime) });
  let { core } = doEnv.wake();

  // Accept at t = 1_000, sign in at t = 1_500: deadline must be 1_500 + 900 = 2_400, not 1_900.
  let sA!: FakeSocket;
  assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
  await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_500);
  assert.deepEqual(sA.backing.sent[1], OK);
  assert.equal(doEnv.alarmAt, 1_500 + sessionTime);

  // Firing the scheduled alarm once expires the session cleanly with no past-alarm spin-loop.
  await core.alarm(doEnv.alarmAt!);
  assert.equal(doEnv.kvSize, 0);
  assert.equal(doEnv.alarmAt, null);

  // Start a second window at t = 5_000 (deadline 5_900), then connect at t = 6_000 BEFORE alarm() runs.
  let sB1!: FakeSocket;
  assert.equal(await core.accept(pairName, () => (sB1 = new FakeSocket()), 4_800), 101);
  await core.onMessage(sB1, await signHello(b, sB1.backing.sent[0]!), 5_000);
  assert.equal(doEnv.alarmAt, 5_000 + sessionTime);

  let sA2!: FakeSocket;
  assert.equal(await core.accept(pairName, () => (sA2 = new FakeSocket()), 6_000), 101);
  await core.onMessage(sA2, await signHello(a, sA2.backing.sent[0]!), 6_010);
  assert.equal(sA2.readyState, OPEN, "post-expiry sign-in before alarm() must succeed");
  assert.deepEqual(sA2.backing.sent[1], OK);
  assert.equal(doEnv.alarmAt, 6_010 + sessionTime);
});

test("production wrangler.jsonc never overrides SESSION_BYTES or SESSION_TIME_MS (#190)", () => {
  const wrangler = readFileSync(new URL("../wrangler.jsonc", import.meta.url), "utf8");
  assert.doesNotMatch(
    wrangler,
    /\bSESSION_BYTES\b|\bSESSION_TIME_MS\b/,
    "production wrangler.jsonc must not override relay session limits",
  );
});

test("edge handler rejects bad X-TotalDrop-Auth with 403 before touching Pair DO (#78)", async () => {
  const [a, b, pairName] = await generatePairIdentities();
  const [outsider] = await generatePairIdentities();
  const otherPair = a.hex < outsider.hex ? `${a.hex}-${outsider.hex}` : `${outsider.hex}-${a.hex}`;
  const validHeader = await signConnectHeader(a, pairName);
  const wrongKeyHeader = await signConnectHeader(outsider, pairName);
  const wrongPairHeader = await signConnectHeader(a, otherPair);
  const badSigHeader = `${a.hex}.${"00".repeat(64)}`;

  let lookups = 0;
  let lastName = "";
  const ns: PairNamespace = {
    getByName(name: string) {
      lookups++;
      lastName = name;
      return {
        async fetch() {
          return new Response("forwarded", { status: 200 });
        },
      };
    },
  };

  const url = `https://relay.totaldrop.app/v0/relay/${pairName}`;

  // Rejections that must NEVER call env.PAIR.getByName:
  const badCases: [string, Record<string, string>][] = [
    ["malformed", { Upgrade: "websocket", [AUTH_HEADER]: "not-valid" }],
    ["empty auth", { Upgrade: "websocket", [AUTH_HEADER]: "" }],
    ["uppercase hex", { Upgrade: "websocket", [AUTH_HEADER]: validHeader.toUpperCase() }],
    ["wrong key", { Upgrade: "websocket", [AUTH_HEADER]: wrongKeyHeader }],
    ["wrong pair", { Upgrade: "websocket", [AUTH_HEADER]: wrongPairHeader }],
    ["bad sig", { Upgrade: "websocket", [AUTH_HEADER]: badSigHeader }],
  ];

  for (const [label, headers] of badCases) {
    const res = await handleRelayRequest(new Request(url, { headers }), ns);
    assert.equal(res.status, 403, `${label} must return 403`);
    assert.equal(lookups, 0, `${label} must not invoke PAIR.getByName`);
  }

  // Duplicate X-TotalDrop-Auth headers join with ", " and must also fail with 403.
  const dupHeaders = new Headers({ Upgrade: "websocket" });
  dupHeaders.append(AUTH_HEADER, validHeader);
  dupHeaders.append(AUTH_HEADER, validHeader);
  const dupRes = await handleRelayRequest(new Request(url, { headers: dupHeaders }), ns);
  assert.equal(dupRes.status, 403, "duplicate auth headers must return 403");
  assert.equal(lookups, 0, "duplicate auth headers must not invoke PAIR.getByName");

  // Non-GET, Origin, non-prefix, and bad pair name also stay handled at the edge.
  assert.equal(
    (await handleRelayRequest(new Request(url, { method: "POST" }), ns)).status,
    405,
  );
  assert.equal(
    (
      await handleRelayRequest(
        new Request(url, { headers: { Upgrade: "websocket", Origin: "https://evil.example" } }),
        ns,
      )
    ).status,
    403,
  );
  assert.equal(
    (await handleRelayRequest(new Request("https://relay.totaldrop.app/"), ns)).status,
    200,
  );
  assert.equal(
    (
      await handleRelayRequest(
        new Request("https://relay.totaldrop.app/v0/relay/bad", {
          headers: { Upgrade: "websocket" },
        }),
        ns,
      )
    ).status,
    400,
  );
  assert.equal(lookups, 0);

  // Valid X-TotalDrop-Auth forwards to PAIR.getByName(pairName) regardless of
  // hop-by-hop Sec-WebSocket-Key rewriting by proxies/workerd.
  const okRes = await handleRelayRequest(
    new Request(url, {
      headers: { Upgrade: "websocket", "Sec-WebSocket-Key": "proxy-rewritten", [AUTH_HEADER]: validHeader },
    }),
    ns,
  );
  assert.equal(okRes.status, 200);
  assert.equal(lookups, 1);
  assert.equal(lastName, pairName);

  // Legacy client omitting X-TotalDrop-Auth also forwards to PAIR.getByName(pairName).
  const legacyRes = await handleRelayRequest(
    new Request(url, { headers: { Upgrade: "websocket" } }),
    ns,
  );
  assert.equal(legacyRes.status, 200);
  assert.equal(lookups, 2);
  assert.equal(lastName, pairName);
  assert.ok(parseAuthHeader(validHeader) !== null);
  assert.equal(parseAuthHeader(`${validHeader}.extra`), null);
  assert.equal(b.hex.length, 64);
});

test("unsigned squatters filling MAX_WAITING block 3rd unsigned with 409, while pre-auth peer gets 101 and signs in (#78)", async () => {
  const [a, b, pairName] = await generatePairIdentities();
  const doEnv = new FakeDurableObjectEnv();
  const { core } = doEnv.wake();

  // Two unsigned squatters fill MAX_WAITING = 2.
  const squatters: FakeSocket[] = [];
  for (let i = 0; i < MAX_WAITING; i++) {
    let s!: FakeSocket;
    assert.equal(
      await core.accept(pairName, () => (s = new FakeSocket()), 1_000 + i, {
        header: null,
      }),
      101,
    );
    squatters.push(s);
  }

  // Third unsigned caller gets 409 Conflict.
  assert.equal(
    await core.accept(pairName, () => new FakeSocket(), 1_010, {
      header: null,
    }),
    409,
  );

  // Real peers with valid X-TotalDrop-Auth bypass MAX_WAITING, get 101, sign in, and exchange frames.
  let sA!: FakeSocket;
  let sB!: FakeSocket;
  assert.equal(
    await core.accept(pairName, () => (sA = new FakeSocket()), 1_020, {
      header: await signConnectHeader(a, pairName),
    }),
    101,
  );
  assert.equal(
    await core.accept(pairName, () => (sB = new FakeSocket()), 1_021, {
      header: await signConnectHeader(b, pairName),
    }),
    101,
  );

  await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_030);
  await core.onMessage(sB, await signHello(b, sB.backing.sent[0]!), 1_031);
  assert.deepEqual(sA.backing.sent[1], OK);
  assert.deepEqual(sB.backing.sent[1], OK);

  const bBefore = sB.backing.sent.length;
  await core.onMessage(sA, makeDatagramFrame(b.hex, 128, 0x5a), 1_040);
  assert.ok(sB.backing.sent.length > bBefore, "datagram must be forwarded to sB");
  const delivered = sB.backing.sent[bBefore]!;
  assert.equal(delivered.length, 32 + 2 + 128);
  assert.deepEqual(delivered.subarray(0, 32), a.pub);
  assert.equal(squatters.length, MAX_WAITING);
});

test("unsigned squatters filling MAX_SOCKETS (both live and lingering CLOSING) never cause 409 for valid pre-auth (#78)", async () => {
  const [a, , pairName] = await generatePairIdentities();
  const doEnv = new FakeDurableObjectEnv();
  const { core } = doEnv.wake();

  // Case 1: 30 lingering CLOSING unsigned sockets (via malformed hello) + 2 live unsigned waiting sockets = MAX_SOCKETS.
  for (let i = 0; i < MAX_SOCKETS - MAX_WAITING; i++) {
    let s!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (s = new FakeSocket()), 1_000 + i), 101);
    // Malformed hello closes s -> readyState becomes CLOSING, lingering in getWebSockets().
    await core.onMessage(s, new Uint8Array([1, 2, 3]).buffer, 1_000 + i);
    assert.equal(s.readyState, CLOSING);
  }
  let liveOlder!: FakeSocket;
  let liveNewer!: FakeSocket;
  assert.equal(await core.accept(pairName, () => (liveOlder = new FakeSocket()), 2_000), 101);
  assert.equal(await core.accept(pairName, () => (liveNewer = new FakeSocket()), 2_500), 101);

  // Another unsigned client is blocked by MAX_SOCKETS / MAX_WAITING.
  assert.equal(await core.accept(pairName, () => new FakeSocket(), 2_600), 409);

  // Valid pre-auth evicts the OLDEST live unsigned waiting socket with (1008, "evicted") and gets 101!
  let sA1!: FakeSocket;
  assert.equal(
    await core.accept(pairName, () => (sA1 = new FakeSocket()), 3_000, {
      header: await signConnectHeader(a, pairName),
    }),
    101,
  );
  assert.equal(liveOlder.readyState, CLOSING, "oldest unsigned waiting socket must be evicted");
  assert.deepEqual(liveOlder.backing.closeCalls.at(-1), { code: 1008, reason: "evicted" });
  assert.equal(liveNewer.readyState, OPEN, "newer unsigned waiting socket stays open");
  await core.onMessage(sA1, await signHello(a, sA1.backing.sent[0]!), 3_010);
  assert.deepEqual(sA1.backing.sent[1], OK);

  // Case 2: All MAX_SOCKETS (32) are lingering CLOSING unsigned sockets (0 live sockets).
  const doEnv2 = new FakeDurableObjectEnv();
  const { core: core2 } = doEnv2.wake();
  for (let i = 0; i < MAX_SOCKETS; i++) {
    let s!: FakeSocket;
    assert.equal(await core2.accept(pairName, () => (s = new FakeSocket()), 4_000 + i), 101);
    await core2.onMessage(s, new Uint8Array([9, 9]).buffer, 4_000 + i);
    assert.equal(s.readyState, CLOSING);
  }
  // Unsigned client gets 409 because getWebSockets().length === MAX_SOCKETS.
  assert.equal(await core2.accept(pairName, () => new FakeSocket(), 4_500), 409);
  // Valid pre-auth STILL gets 101 because authenticatedSockets === 0 < MAX_SOCKETS!
  let sA2!: FakeSocket;
  assert.equal(
    await core2.accept(pairName, () => (sA2 = new FakeSocket()), 4_600, {
      header: await signConnectHeader(a, pairName),
    }),
    101,
  );
  await core2.onMessage(sA2, await signHello(a, sA2.backing.sent[0]!), 4_610);
  assert.deepEqual(sA2.backing.sent[1], OK);
});

test("MAX_SOCKETS authenticated sockets return 409, and same-key waiting sockets deduplicate (#78)", async () => {
  const [a, b, pairName] = await generatePairIdentities();
  const doEnv = new FakeDurableObjectEnv();
  const { core } = doEnv.wake();

  // Same-key pre-auth waiting deduplication closes older waiting socket with (1000, "replaced")
  // without touching the other key's waiting socket.
  const authA = await signConnectHeader(a, pairName);
  const authB = await signConnectHeader(b, pairName);
  let wA1!: FakeSocket;
  let wB1!: FakeSocket;
  let wA2!: FakeSocket;
  assert.equal(
    await core.accept(pairName, () => (wA1 = new FakeSocket()), 1_000, { header: authA }),
    101,
  );
  assert.equal(
    await core.accept(pairName, () => (wB1 = new FakeSocket()), 1_001, { header: authB }),
    101,
  );
  assert.equal(
    await core.accept(pairName, () => (wA2 = new FakeSocket()), 1_002, { header: authA }),
    101,
  );
  assert.equal(wA1.readyState, CLOSING);
  assert.deepEqual(wA1.backing.closeCalls.at(-1), { code: 1000, reason: "replaced" });
  assert.equal(wB1.readyState, OPEN);
  assert.equal(wA2.readyState, OPEN);

  // Fill remaining slots up to MAX_SOCKETS with authenticated sockets (replaced ones linger as CLOSING).
  for (let i = 3; i < MAX_SOCKETS; i++) {
    let s!: FakeSocket;
    assert.equal(
      await core.accept(pairName, () => (s = new FakeSocket()), 1_000 + i, { header: authA }),
      101,
    );
    await core.onMessage(s, await signHello(a, s.backing.sent[0]!), 1_000 + i);
  }

  // Once 32 authenticated sockets exist in getWebSockets(), the 33rd returns 409
  // without closing live waiting sockets.
  assert.equal(
    await core.accept(pairName, () => new FakeSocket(), 2_000, { header: authB }),
    409,
  );
  assert.equal(wB1.readyState, OPEN, "409 rejection must not close live same-key waiter wB1");
  assert.equal(wB1.backing.closeCalls.length, 0);
});

test("409 when authenticatedSockets >= MAX_SOCKETS never closes pre-auth or unsigned waiters (#78)", async () => {
  const [a, , pairName] = await generatePairIdentities();
  const authA = await signConnectHeader(a, pairName);

  // Case 1: 31 CLOSING authenticated sockets + 1 OPEN pre-authenticated waiting socket (wA).
  {
    const doEnv = new FakeDurableObjectEnv();
    const { core } = doEnv.wake();
    for (let i = 0; i < MAX_SOCKETS - 1; i++) {
      let s!: FakeSocket;
      assert.equal(
        await core.accept(pairName, () => (s = new FakeSocket()), 1_000 + i, { header: authA }),
        101,
      );
      await core.onMessage(s, new Uint8Array([0]).buffer, 1_000 + i);
      assert.equal(s.readyState, CLOSING);
    }
    let wA!: FakeSocket;
    assert.equal(
      await core.accept(pairName, () => (wA = new FakeSocket()), 2_000, { header: authA }),
      101,
    );
    assert.equal(wA.readyState, OPEN);
    assert.equal(wA.backing.closeCalls.length, 0);

    // 33rd pre-auth returns 409 and MUST leave wA untouched.
    assert.equal(
      await core.accept(pairName, () => new FakeSocket(), 2_100, { header: authA }),
      409,
    );
    assert.equal(wA.readyState, OPEN, "409 must not close live same-key pre-auth waiter");
    assert.equal(wA.backing.closeCalls.length, 0);
  }

  // Case 2: 32 CLOSING authenticated sockets + 1 OPEN unsigned waiting socket (u1).
  {
    const doEnv = new FakeDurableObjectEnv();
    const { core } = doEnv.wake();
    for (let i = 0; i < MAX_SOCKETS; i++) {
      let s!: FakeSocket;
      assert.equal(
        await core.accept(pairName, () => (s = new FakeSocket()), 1_100 + i, { header: authA }),
        101,
      );
      await core.onMessage(s, new Uint8Array([0]).buffer, 1_100 + i);
      assert.equal(s.readyState, CLOSING);
    }
    const u1 = new FakeSocket();
    u1.serializeAttachment({ challenge: "00".repeat(32), key: null, since: 1_000 });
    doEnv.addSocket(u1, [pairName]);
    assert.equal(u1.readyState, OPEN);

    // 33rd authenticated attempt returns 409 and MUST leave u1 untouched (zero eviction).
    assert.equal(
      await core.accept(pairName, () => new FakeSocket(), 2_100, { header: authA }),
      409,
    );
    assert.equal(u1.readyState, OPEN, "409 must not evict live unsigned waiter u1");
    assert.equal(u1.backing.closeCalls.length, 0);
  }
});

test("invalid X-TotalDrop-Auth in PairCore.accept returns 403 without evicting any waiting socket (#78)", async () => {
  const [a, , pairName] = await generatePairIdentities();
  const [outsider] = await generatePairIdentities();
  const otherPair = a.hex < outsider.hex ? `${a.hex}-${outsider.hex}` : `${outsider.hex}-${a.hex}`;
  const doEnv = new FakeDurableObjectEnv();
  const { core } = doEnv.wake();

  // Fill with 30 CLOSING + 2 OPEN unsigned waiting sockets so a valid pre-auth would evict.
  for (let i = 0; i < MAX_SOCKETS - MAX_WAITING; i++) {
    let s!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (s = new FakeSocket()), 1_000 + i), 101);
    await core.onMessage(s, new Uint8Array([0]).buffer, 1_000 + i);
  }
  let u1!: FakeSocket;
  let u2!: FakeSocket;
  assert.equal(await core.accept(pairName, () => (u1 = new FakeSocket()), 2_000), 101);
  assert.equal(await core.accept(pairName, () => (u2 = new FakeSocket()), 2_001), 101);

  const valid = await signConnectHeader(a, pairName);
  const badAuths = [
    { header: "" },
    { header: "malformed" },
    { header: valid.toUpperCase() },
    { header: await signConnectHeader(outsider, pairName) },
    { header: await signConnectHeader(a, otherPair) },
    { header: `${a.hex}.${"ff".repeat(64)}` },
    { header: `${valid}, ${valid}` },
  ];

  for (const auth of badAuths) {
    assert.equal(await core.accept(pairName, () => new FakeSocket(), 3_000, auth), 403);
  }
  assert.equal(u1.readyState, OPEN, "invalid auth must not evict u1");
  assert.equal(u2.readyState, OPEN, "invalid auth must not evict u2");
});

test("pre-authenticated socket cannot sign in as the other pair key, even after hibernation (#78)", async () => {
  const [a, b, pairName] = await generatePairIdentities();
  const doEnv = new FakeDurableObjectEnv();
  let { core, socketOf } = doEnv.wake();

  let sA!: FakeSocket;
  assert.equal(
    await core.accept(pairName, () => (sA = new FakeSocket()), 1_000, {
      header: await signConnectHeader(a, pairName),
    }),
    101,
  );

  // Hibernate between HTTP 101 and TDR0 hello: preauth must persist in attachment.
  ({ core, socketOf } = doEnv.wake());
  const wokenA = socketOf(sA.backing);
  assert.equal((wokenA.deserializeAttachment() as Attachment).preauth, a.hex);

  // Attempting to sign in with key B's valid hello signature must fail with (1008, "not this pair").
  await core.onMessage(wokenA, await signHello(b, wokenA.backing.sent[0]!), 1_010);
  assert.equal(wokenA.readyState, CLOSING);
  assert.deepEqual(wokenA.backing.closeCalls.at(-1), { code: 1008, reason: "not this pair" });
});

test("over-limit pair rejects pre-auth with 403 cold without evicting, while expired pair expires both peers and admits new window (#78)", async () => {
  const [a, b, pairName] = await generatePairIdentities();
  const sessionTime = 1_000;
  const doEnv = new FakeDurableObjectEnv({
    SESSION_BYTES: "500",
    SESSION_TIME_MS: String(sessionTime),
  });
  let { core } = doEnv.wake();

  // Sign in both peers and exceed SESSION_BYTES.
  let sA!: FakeSocket;
  let sB!: FakeSocket;
  assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 10_000), 101);
  assert.equal(await core.accept(pairName, () => (sB = new FakeSocket()), 10_000), 101);
  await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 10_100);
  await core.onMessage(sB, await signHello(b, sB.backing.sent[0]!), 10_100);
  await core.onMessage(sA, makeDatagramFrame(b.hex, 600, 0x77), 10_200);
  assert.equal(sA.readyState, CLOSING);
  assert.equal(sB.readyState, CLOSING);

  // Cold wake after hibernation: an unsigned waiting socket connects, then valid pre-auth tries while over-limit.
  ({ core } = doEnv.wake());
  let waiter!: FakeSocket;
  assert.equal(await core.accept(pairName, () => (waiter = new FakeSocket()), 10_300), 101);
  assert.equal(
    await core.accept(pairName, () => new FakeSocket(), 10_400, {
      header: await signConnectHeader(a, pairName),
    }),
    403,
    "over-limit pair must reject pre-auth with 403 even on cold wake",
  );
  assert.equal(waiter.readyState, OPEN, "over-limit 403 must not evict waiter");

  // After the session window expires (10_100 + 1_000 = 11_100), valid pre-auth triggers expireSession()
  // closing waiter with (1008, "session over") and admits a fresh session window.
  let sA2!: FakeSocket;
  assert.equal(
    await core.accept(pairName, () => (sA2 = new FakeSocket()), 11_100, {
      header: await signConnectHeader(a, pairName),
    }),
    101,
  );
  assert.equal(waiter.readyState, CLOSING);
  assert.deepEqual(waiter.backing.closeCalls.at(-1), { code: 1008, reason: "session over" });
  await core.onMessage(sA2, await signHello(a, sA2.backing.sent[0]!), 11_110);
  assert.deepEqual(sA2.backing.sent[1], OK);
});

test("session over closes both signed-in peers simultaneously on post-deadline message and on alarm (#78)", async () => {
  const [a, b, pairName] = await generatePairIdentities();
  const sessionTime = 1_000;

  // Path 1: post-deadline message on sA closes BOTH sA and sB with (1008, "session over") immediately.
  {
    const doEnv = new FakeDurableObjectEnv({ SESSION_TIME_MS: String(sessionTime) });
    const { core } = doEnv.wake();
    let sA!: FakeSocket;
    let sB!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    assert.equal(await core.accept(pairName, () => (sB = new FakeSocket()), 1_400), 101);
    await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_050);
    await core.onMessage(sB, await signHello(b, sB.backing.sent[0]!), 1_450);

    // At 1_050 + 1_000 = 2_050, sB is well within 1_450 + 1_000, yet BOTH close together.
    await core.onMessage(sA, makeDatagramFrame(b.hex, 100, 0x01), 2_050);
    assert.equal(sA.readyState, CLOSING);
    assert.equal(sB.readyState, CLOSING);
    assert.deepEqual(sA.backing.closeCalls.at(-1), { code: 1008, reason: "session over" });
    assert.deepEqual(sB.backing.closeCalls.at(-1), { code: 1008, reason: "session over" });
    assert.equal(doEnv.kvSize, 0);
    assert.equal(doEnv.alarmAt, null);
  }

  // Path 2: alarm firing at deadline closes BOTH sA and sB with (1008, "session over") simultaneously.
  {
    const doEnv = new FakeDurableObjectEnv({ SESSION_TIME_MS: String(sessionTime) });
    const { core } = doEnv.wake();
    let sA!: FakeSocket;
    let sB!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 5_000), 101);
    assert.equal(await core.accept(pairName, () => (sB = new FakeSocket()), 5_400), 101);
    await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 5_050);
    await core.onMessage(sB, await signHello(b, sB.backing.sent[0]!), 5_450);
    assert.equal(doEnv.alarmAt, 5_050 + sessionTime);

    await core.alarm(doEnv.alarmAt!);
    assert.equal(sA.readyState, CLOSING);
    assert.equal(sB.readyState, CLOSING);
    assert.deepEqual(sA.backing.closeCalls.at(-1), { code: 1008, reason: "session over" });
    assert.deepEqual(sB.backing.closeCalls.at(-1), { code: 1008, reason: "session over" });
    assert.equal(doEnv.kvSize, 0);
    assert.equal(doEnv.alarmAt, null);
  }
});

test("concurrent accept and hello serialize behind in-flight expireSession() across deleteAll and deleteAlarm (#78)", async () => {
  const [a, b, pairName] = await generatePairIdentities();
  const authA = await signConnectHeader(a, pairName);
  const sessionTime = 1_000;

  const makeBarrier = () => {
    let enteredResolve!: () => void;
    let releaseResolve!: () => void;
    const entered = new Promise<void>((r) => {
      enteredResolve = r;
    });
    const release = new Promise<void>((r) => {
      releaseResolve = r;
    });
    return {
      entered,
      release: () => releaseResolve(),
      hook: async () => {
        enteredResolve();
        await release;
      },
    };
  };

  // Variant A: alarm() paused inside deleteAll() while concurrent accept() + hello run.
  {
    const doEnv = new FakeDurableObjectEnv({ SESSION_TIME_MS: String(sessionTime) });
    const { core } = doEnv.wake();
    let sA!: FakeSocket;
    let sB!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    assert.equal(await core.accept(pairName, () => (sB = new FakeSocket()), 1_000), 101);
    await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_000);
    await core.onMessage(sB, await signHello(b, sB.backing.sent[0]!), 1_000);
    assert.equal(doEnv.alarmAt, 2_000);

    const barrier = makeBarrier();
    doEnv.onDeleteAll = barrier.hook;

    const alarmP = core.alarm(2_000);
    await barrier.entered;
    // Both old peers were closed synchronously before deleteAll() suspended.
    assert.equal(sA.readyState, CLOSING);
    assert.equal(sB.readyState, CLOSING);

    // Launch a concurrent pre-authenticated accept while deleteAll() is suspended.
    const accepted: FakeSocket[] = [];
    const acceptP = core.accept(
      pairName,
      () => {
        const ws = new FakeSocket();
        accepted.push(ws);
        return ws;
      },
      2_010,
      { header: authA },
    );
    // Allow verifyConnectAuth() WebCrypto promise to settle while deleteAll() is still blocked.
    await new Promise((r) => setTimeout(r, 20));
    assert.equal(accepted.length, 0, "accept must wait for in-flight expireSession() before admitting");

    // Release deleteAll(): expireSession() finishes, then accept() admits sA2 cleanly.
    doEnv.onDeleteAll = undefined;
    barrier.release();
    const [, status] = await Promise.all([alarmP, acceptP]);
    assert.equal(status, 101);
    assert.equal(accepted.length, 1);
    const sA2 = accepted[0]!;
    assert.equal(sA2.readyState, OPEN);
    assert.equal(doEnv.alarmAt, 12_010);

    // New session sign-in persists KV totals and sets its own session alarm intact.
    await core.onMessage(sA2, await signHello(a, sA2.backing.sent[0]!), 2_020);
    assert.deepEqual(sA2.backing.sent[1], OK);
    assert.equal(doEnv.kvSize, 1, "new session KV totals must survive expireSession()");
    assert.equal(doEnv.alarmAt, 2_020 + sessionTime, "new session alarm must survive expireSession()");
  }

  // Variant B: onMessage() paused inside deleteAlarm() while concurrent accept() runs.
  {
    const doEnv = new FakeDurableObjectEnv({ SESSION_TIME_MS: String(sessionTime) });
    const { core } = doEnv.wake();
    let sA!: FakeSocket;
    let sB!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    assert.equal(await core.accept(pairName, () => (sB = new FakeSocket()), 1_000), 101);
    await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_000);
    await core.onMessage(sB, await signHello(b, sB.backing.sent[0]!), 1_000);

    const barrier = makeBarrier();
    doEnv.onDeleteAlarm = barrier.hook;

    const msgP = core.onMessage(sA, makeDatagramFrame(b.hex, 50, 0xaa), 2_000);
    await barrier.entered;

    const accepted: FakeSocket[] = [];
    const acceptP = core.accept(
      pairName,
      () => {
        const ws = new FakeSocket();
        accepted.push(ws);
        return ws;
      },
      2_010,
      { header: authA },
    );
    await new Promise((r) => setTimeout(r, 20));
    assert.equal(accepted.length, 0, "accept must wait for deleteAlarm() inside expireSession()");

    doEnv.onDeleteAlarm = undefined;
    barrier.release();
    const [, status] = await Promise.all([msgP, acceptP]);
    assert.equal(status, 101);
    assert.equal(accepted.length, 1);
    assert.equal(accepted[0]!.readyState, OPEN);
    assert.equal(doEnv.alarmAt, 12_010, "admitted socket alarm must not be wiped by deleteAlarm()");
  }

  // Variant C: overlapping expireSession() calls coalesce (single deleteAll/deleteAlarm)
  // and a waiting socket's hello arriving during deleteAll() waits and is ignored once closed.
  {
    const doEnv = new FakeDurableObjectEnv({ SESSION_TIME_MS: String(sessionTime) });
    const { core } = doEnv.wake();
    let sA!: FakeSocket;
    let waiter!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_000);
    assert.equal(await core.accept(pairName, () => (waiter = new FakeSocket()), 1_500), 101);
    const waiterHello = await signHello(b, waiter.backing.sent[0]!);

    const barrier = makeBarrier();
    doEnv.onDeleteAll = barrier.hook;

    const alarm1 = core.alarm(2_000);
    await barrier.entered;
    const expire2 = core.expireSession();
    const helloP = core.onMessage(waiter, waiterHello, 2_005);

    doEnv.onDeleteAll = undefined;
    barrier.release();
    await Promise.all([alarm1, expire2, helloP]);
    assert.equal(doEnv.deleteAllCalls, 1, "overlapping expirations must coalesce");
    assert.equal(doEnv.deleteAlarmCalls, 1, "coalesced expiration must deleteAlarm only once");
    assert.equal(waiter.readyState, CLOSING);
    assert.equal(waiter.backing.sent.length, 1, "expired waiter must not receive OK");
    assert.equal(doEnv.kvSize, 0);
    assert.equal(doEnv.alarmAt, null);
  }
});

test("expireSession() never wedges waitExpiring() on sync throws (kv.delete, deleteAll, getWebSockets) or async rejection (#78)", async () => {
  const [a, b, pairName] = await generatePairIdentities();
  const authA = await signConnectHeader(a, pairName);
  const sessionTime = 1_000;

  // Case 1: synchronous throw in kv.delete("totals") leaves totals consistent with KV and does not wedge PairCore.
  {
    const doEnv = new FakeDurableObjectEnv({
      SESSION_BYTES: "500",
      SESSION_TIME_MS: String(sessionTime),
    });
    const { core } = doEnv.wake();
    let sA!: FakeSocket;
    let sB!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    assert.equal(await core.accept(pairName, () => (sB = new FakeSocket()), 1_000), 101);
    await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_000);
    await core.onMessage(sB, await signHello(b, sB.backing.sent[0]!), 1_000);
    await core.onMessage(sA, makeDatagramFrame(b.hex, 600, 0x33), 1_100);
    assert.equal(core.totals.isOver(500), true);

    doEnv.onKvDelete = () => {
      throw new Error("sync kv.delete fault");
    };
    await assert.rejects(() => core.alarm(2_000), /sync kv\.delete fault/);
    assert.equal(doEnv.kvSize, 1, "KV must retain totals after failed kv.delete");
    assert.equal(core.totals.isOver(500), true, "totals.isOver() must remain consistent with KV");
    assert.equal(core.totals.isExpired(2_000, sessionTime), true);

    // Clear fault: subsequent schedule() and accept() complete immediately and retry expiry cleanly.
    doEnv.onKvDelete = undefined;
    await core.schedule(2_005);
    let sA2!: FakeSocket;
    assert.equal(
      await core.accept(pairName, () => (sA2 = new FakeSocket()), 2_010, { header: authA }),
      101,
    );
    assert.equal(doEnv.kvSize, 0, "retried expireSession() must clear stale KV totals");
    await core.onMessage(sA2, await signHello(a, sA2.backing.sent[0]!), 2_020);
    assert.deepEqual(sA2.backing.sent[1], OK);
    assert.equal(doEnv.kvSize, 1);
    assert.equal(doEnv.alarmAt, 2_020 + sessionTime);
  }

  // Case 2: synchronous throw in storage.deleteAll() rejects cleanly without wedging subsequent accept()/schedule().
  {
    const doEnv = new FakeDurableObjectEnv({ SESSION_TIME_MS: String(sessionTime) });
    const { core } = doEnv.wake();
    let sA!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_000);

    doEnv.onDeleteAllSync = () => {
      throw new Error("sync deleteAll fault");
    };
    await assert.rejects(() => core.alarm(2_000), /sync deleteAll fault/);
    assert.equal(sA.readyState, CLOSING);

    doEnv.onDeleteAllSync = undefined;
    await core.schedule(2_005);
    let sA2!: FakeSocket;
    assert.equal(
      await core.accept(pairName, () => (sA2 = new FakeSocket()), 2_010, { header: authA }),
      101,
    );
    await core.onMessage(sA2, await signHello(a, sA2.backing.sent[0]!), 2_020);
    assert.deepEqual(sA2.backing.sent[1], OK);
  }

  // Case 3: synchronous throw in getWebSockets() rejects cleanly without wedging PairCore.
  {
    const doEnv = new FakeDurableObjectEnv({ SESSION_TIME_MS: String(sessionTime) });
    const { core } = doEnv.wake();
    let sA!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_000);

    doEnv.onGetWebSockets = () => {
      throw new Error("sync getWebSockets fault");
    };
    await assert.rejects(() => core.alarm(2_000), /sync getWebSockets fault/);

    doEnv.onGetWebSockets = undefined;
    assert.equal(doEnv.kvSize, 1);
    let sA2!: FakeSocket;
    assert.equal(
      await core.accept(pairName, () => (sA2 = new FakeSocket()), 2_010, { header: authA }),
      101,
    );
    assert.equal(sA.readyState, CLOSING);
    await core.onMessage(sA2, await signHello(a, sA2.backing.sent[0]!), 2_020);
    assert.deepEqual(sA2.backing.sent[1], OK);
  }

  // Case 4: async rejection in storage.deleteAll() unblocks concurrent waiting accept() cleanly.
  {
    const doEnv = new FakeDurableObjectEnv({ SESSION_TIME_MS: String(sessionTime) });
    const { core } = doEnv.wake();
    let sA!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_000);

    let enteredResolve!: () => void;
    let rejectDeleteAll!: (err: Error) => void;
    const entered = new Promise<void>((r) => {
      enteredResolve = r;
    });
    const gate = new Promise<void>((_res, rej) => {
      rejectDeleteAll = rej;
    });
    doEnv.onDeleteAll = async () => {
      enteredResolve();
      await gate;
    };

    const alarmP = core.alarm(2_000);
    await entered;

    let sA2!: FakeSocket;
    const acceptP = core.accept(pairName, () => (sA2 = new FakeSocket()), 2_010, { header: authA });
    await new Promise((r) => setTimeout(r, 15));

    doEnv.onDeleteAll = undefined;
    rejectDeleteAll(new Error("async deleteAll fault"));
    await assert.rejects(alarmP, /async deleteAll fault/);
    assert.equal(await acceptP, 101);
    assert.equal(sA2.readyState, OPEN);
    await core.onMessage(sA2, await signHello(a, sA2.backing.sent[0]!), 2_020);
    assert.deepEqual(sA2.backing.sent[1], OK);
  }
});

test("client-initiated CLOSING or CLOSED Session 1 socket before or during expireSession() never kills Session 2 peer (#78)", async () => {
  const [a, b, pairName] = await generatePairIdentities();
  const sessionTime = 1_000;

  // Case 1: sA enters client-initiated CLOSING right before expireSession() -> Session 2 sB2 signs in -> delayed end(sA).
  {
    const doEnv = new FakeDurableObjectEnv({ SESSION_TIME_MS: String(sessionTime) });
    const { core } = doEnv.wake();
    let sA!: FakeSocket;
    let sB!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    assert.equal(await core.accept(pairName, () => (sB = new FakeSocket()), 1_000), 101);
    await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_000);
    await core.onMessage(sB, await signHello(b, sB.backing.sent[0]!), 1_000);

    // Client starts closing sA before webSocketClose fires, then session expires.
    sA.readyState = CLOSING;
    await core.alarm(2_000);
    assert.equal((sA.deserializeAttachment() as Attachment).closed, true);
    assert.equal(sB.readyState, CLOSING);
    assert.deepEqual(sB.backing.closeCalls.at(-1), { code: 1008, reason: "session over" });

    // Session 2: peer B signs in cleanly.
    let sB2!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sB2 = new FakeSocket()), 2_010), 101);
    await core.onMessage(sB2, await signHello(b, sB2.backing.sent[0]!), 2_020);
    assert.deepEqual(sB2.backing.sent[1], OK);

    // Delayed close callback for Session 1's sA must NOT cascade "peer left" onto Session 2's sB2.
    sA.readyState = CLOSED;
    await core.end(sA);
    assert.equal(sB2.readyState, OPEN, "Session 1 delayed end(sA) must not close Session 2 peer sB2");
    assert.equal(sB2.backing.closeCalls.length, 0);
  }

  // Case 2: same as Case 1 across a Durable Object hibernation wake between expiry and delayed end(sA).
  {
    const doEnv = new FakeDurableObjectEnv({ SESSION_TIME_MS: String(sessionTime) });
    let { core, socketOf } = doEnv.wake();
    let sA!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_000);

    sA.readyState = CLOSING;
    await core.alarm(2_000);

    ({ core, socketOf } = doEnv.wake());
    let sB2!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sB2 = new FakeSocket()), 2_010), 101);
    await core.onMessage(sB2, await signHello(b, sB2.backing.sent[0]!), 2_020);

    const wokenA = socketOf(sA.backing);
    wokenA.readyState = CLOSED;
    await core.end(wokenA);
    assert.equal(sB2.readyState, OPEN, "post-hibernation delayed end(sA) must not close sB2");
    assert.equal(sB2.backing.closeCalls.length, 0);
  }

  // Case 3: sA already transitioned to CLOSED (dropped from getWebSockets(), still in byKey) before expireSession().
  {
    const doEnv = new FakeDurableObjectEnv({ SESSION_TIME_MS: String(sessionTime) });
    const { core } = doEnv.wake();
    let sA!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_000);

    sA.readyState = CLOSED;
    await core.alarm(2_000);

    let sB2!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sB2 = new FakeSocket()), 2_010), 101);
    await core.onMessage(sB2, await signHello(b, sB2.backing.sent[0]!), 2_020);

    await core.end(sA);
    assert.equal(sB2.readyState, OPEN, "CLOSED byKey socket end(sA) must not close sB2");
    assert.equal(sB2.backing.closeCalls.length, 0);
  }

  // Case 4: end(sA) arrives WHILE expireSession() is suspended in deleteAll(), even if sA wasn't in getWebSockets()/byKey.
  {
    const doEnv = new FakeDurableObjectEnv({ SESSION_TIME_MS: String(sessionTime) });
    let { core, socketOf } = doEnv.wake();
    let sA!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    await core.onMessage(sA, await signHello(a, sA.backing.sent[0]!), 1_000);

    // Hibernate so byKey is empty, and mark sA CLOSED so getWebSockets() omits it too.
    ({ core, socketOf } = doEnv.wake());
    const wokenA = socketOf(sA.backing);
    wokenA.readyState = CLOSED;

    let enteredResolve!: () => void;
    let releaseResolve!: () => void;
    const entered = new Promise<void>((r) => {
      enteredResolve = r;
    });
    const release = new Promise<void>((r) => {
      releaseResolve = r;
    });
    doEnv.onDeleteAll = async () => {
      enteredResolve();
      await release;
    };

    const alarmP = core.alarm(2_000);
    await entered;

    // Queue end(wokenA) and Session 2 accept(sB2) while expireSession() is in flight.
    const endP = core.end(wokenA);
    let sB2!: FakeSocket;
    const acceptP = core.accept(pairName, () => (sB2 = new FakeSocket()), 2_010);

    doEnv.onDeleteAll = undefined;
    releaseResolve();
    const [, , st] = await Promise.all([alarmP, endP, acceptP]);
    assert.equal(st, 101);
    await core.onMessage(sB2, await signHello(b, sB2.backing.sent[0]!), 2_020);
    assert.equal(sB2.readyState, OPEN);
    assert.equal(sB2.backing.closeCalls.length, 0);
    assert.equal(doEnv.alarmAt, 2_020 + sessionTime);
  }
});

test("pipelined concurrent hello frames emit strictly one OK and never corrupt byKey or kill an already signed-in socket (#78)", async () => {
  const [a, b, pairName] = await generatePairIdentities();

  // Case 1: duplicate valid hello(A) frames pipelined on the same socket.
  {
    const doEnv = new FakeDurableObjectEnv();
    const { core } = doEnv.wake();
    let sA!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    const helloA = await signHello(a, sA.backing.sent[0]!);

    await Promise.all([
      core.onMessage(sA, helloA, 1_010),
      core.onMessage(sA, helloA, 1_010),
    ]);
    assert.equal(sA.readyState, OPEN);
    assert.equal(sA.backing.sent.length, 2, "must send strictly 1 challenge + 1 OK");
    assert.deepEqual(sA.backing.sent[1], OK);
    assert.equal(core.socketFor(a.hex), sA);
  }

  // Case 2: unsigned socket pipelining valid hello(A) and valid hello(B) concurrently.
  {
    const doEnv = new FakeDurableObjectEnv();
    const { core } = doEnv.wake();
    let ws!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (ws = new FakeSocket()), 1_000), 101);
    const helloA = await signHello(a, ws.backing.sent[0]!);
    const helloB = await signHello(b, ws.backing.sent[0]!);

    await Promise.all([
      core.onMessage(ws, helloA, 1_010),
      core.onMessage(ws, helloB, 1_010),
    ]);
    assert.equal(ws.readyState, OPEN);
    assert.equal(ws.backing.sent.length, 2, "must send strictly 1 OK");
    assert.deepEqual(ws.backing.sent[1], OK);
    const aMapped = core.socketFor(a.hex) === ws;
    const bMapped = core.socketFor(b.hex) === ws;
    assert.notEqual(aMapped, bMapped, "socket must map to strictly one key in byKey");
  }

  // Case 3: valid hello(A) completes while a pipelined bad-signature hello(A) is still in flight
  // (deterministically ordered so the trailing bad hello finishes strictly after sA has signed in).
  {
    const doEnv = new FakeDurableObjectEnv();
    const { core } = doEnv.wake();
    let sA!: FakeSocket;
    let sB!: FakeSocket;
    assert.equal(await core.accept(pairName, () => (sA = new FakeSocket()), 1_000), 101);
    assert.equal(await core.accept(pairName, () => (sB = new FakeSocket()), 1_000), 101);
    await core.onMessage(sB, await signHello(b, sB.backing.sent[0]!), 1_005);

    const goodHello = await signHello(a, sA.backing.sent[0]!);
    const badBytes = new Uint8Array(goodHello.slice(0));
    badBytes[badBytes.length - 1] = (badBytes[badBytes.length - 1] ?? 0) ^ 0xff;

    let releaseBadVerify!: () => void;
    const badVerifyGate = new Promise<void>((r) => {
      releaseBadVerify = r;
    });
    const origVerify = crypto.subtle.verify;
    let verifyCall = 0;
    crypto.subtle.verify = async function (
      this: typeof crypto.subtle,
      algorithm: Parameters<typeof origVerify>[0],
      key: Parameters<typeof origVerify>[1],
      signature: Parameters<typeof origVerify>[2],
      data: Parameters<typeof origVerify>[3],
    ): Promise<boolean> {
      verifyCall++;
      const result = await origVerify.call(this, algorithm, key, signature, data);
      if (!result) await badVerifyGate;
      return result;
    };

    try {
      const pGood = core.onMessage(sA, goodHello, 1_010);
      const pBad = core.onMessage(sA, badBytes.buffer, 1_010);
      await pGood;
      releaseBadVerify();
      await pBad;
      assert.equal(verifyCall, 2, "both pipelined hello frames must have invoked verify()");
      assert.equal(sA.readyState, OPEN, "trailing bad hello must not kill already signed-in sA");
      assert.equal(sA.backing.sent.length, 2);
      assert.deepEqual(sA.backing.sent[1], OK);
      assert.equal(sA.backing.closeCalls.length, 0);
      assert.equal(core.socketFor(a.hex), sA);
      assert.equal(sB.readyState, OPEN);
    } finally {
      crypto.subtle.verify = origVerify;
    }
  }
});
