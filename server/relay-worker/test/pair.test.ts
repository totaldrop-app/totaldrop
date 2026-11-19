// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Hibernation wake, socket replacement, delayed close/message events, alarm
// cleanup, and rolling-deploy wake tests for PairCore (#190).

import assert from "node:assert/strict";
import { test } from "node:test";

import {
  AUTH_CONTEXT,
  MAGIC,
  OK,
  OPEN,
  PairCore,
  type Attachment,
  type PairEnv,
  type PairHost,
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
  env: PairEnv;

  readonly kv: TotalsKv = {
    get: <T = unknown>(key: string): T | undefined => {
      const v = this.kvMap.get(key);
      return v === undefined ? undefined : structuredClone(v as T);
    },
    put: <T>(key: string, value: T): void => {
      this.kvMap.set(key, structuredClone(value));
    },
    delete: (key: string): boolean => this.kvMap.delete(key),
  };

  constructor(env: PairEnv = {}) {
    this.env = env;
  }

  get kvSize(): number {
    return this.kvMap.size;
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
      getWebSockets: () =>
        this.socketBackings.filter((b) => b.readyState !== CLOSED).map((b) => this.wrap(b)),
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
          this.deleteAlarmCalls++;
          this.alarmAt = null;
        },
        deleteAll: async () => {
          this.deleteAllCalls++;
          this.kvMap.clear();
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

