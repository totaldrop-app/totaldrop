// SPDX-License-Identifier: AGPL-3.0-or-later
//
// TotalDrop rendezvous on Cloudflare Workers (spec section 5).
//
//   wss://rendezvous.totaldrop.app/v0/allocate       one `allocate`, one reply
//   wss://rendezvous.totaldrop.app/v0/mailbox/<n>    claim, add, close
//
// The Worker itself keeps no state and logs nothing. Allocation is decided
// by the Directory object; each mailbox is its own Mailbox object.

import { Directory } from "./directory";
import { Mailbox } from "./mailbox";
import { decodeClient, server, unhex } from "./messages";
import { BUSY, FAILED, withRetry } from "./retry";

export { Directory, Mailbox };

export interface Env {
  DIRECTORY: DurableObjectNamespace<Directory>;
  MAILBOX: DurableObjectNamespace<Mailbox>;
}

/** How long an allocation connection may take to send its request. */
const ALLOCATE_TIMEOUT = 10_000;

function isUpgrade(request: Request): boolean {
  return (
    request.method === "GET" && request.headers.get("Upgrade")?.toLowerCase() === "websocket"
  );
}

/** The bytes of a binary WebSocket message, whatever shape the runtime
 * delivers them in. Text messages are not part of the protocol. */
async function binary(data: unknown): Promise<Uint8Array> {
  if (data instanceof ArrayBuffer) return new Uint8Array(data);
  if (ArrayBuffer.isView(data)) return new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
  if (data instanceof Blob) return new Uint8Array(await data.arrayBuffer());
  throw new Error("malformed message");
}

/** Accept a WebSocket only to send one error and close it, so clients get a
 * readable reason instead of a failed handshake. */
function refuse(reason: string): Response {
  const pair = new WebSocketPair();
  const socket = pair[1];
  socket.accept();
  socket.send(server.error(reason));
  socket.close(1000, "refused");
  return new Response(null, { status: 101, webSocket: pair[0] });
}

/** The reply to an allocation request. Only fixed reasons reach the
 * client, never a runtime error's text. Allocation is not tried again: a
 * call that failed after the directory handed out a number would hold a
 * second one until it expires. */
async function answer(env: Env, data: unknown): Promise<Uint8Array> {
  let msg;
  try {
    msg = decodeClient(await binary(data));
  } catch {
    return server.error("malformed message");
  }
  if (msg.type !== "allocate") return server.error("only allocate here");
  let result;
  try {
    result = await env.DIRECTORY.getByName("directory").allocate();
  } catch (e) {
    return server.error((e as { overloaded?: unknown } | null)?.overloaded === true ? BUSY : FAILED);
  }
  if ("error" in result) return server.error(result.error);
  return server.allocated(result.nameplate, unhex(result.token));
}

function allocate(env: Env): Response {
  const pair = new WebSocketPair();
  const socket = pair[1];
  socket.accept();
  let answered = false;
  const timer = setTimeout(() => {
    if (!answered) socket.close(1000, "timeout");
  }, ALLOCATE_TIMEOUT);
  socket.addEventListener("message", async (event) => {
    if (answered) return;
    answered = true;
    clearTimeout(timer);
    try {
      socket.send(await answer(env, event.data));
      socket.close(1000, "done");
    } catch {
      // The client is gone.
    }
  });
  return new Response(null, { status: 101, webSocket: pair[0] });
}

async function mailbox(env: Env, request: Request, nameplate: number): Promise<Response> {
  // A lookup only reads, so it may be tried again too.
  const found = await withRetry(() => env.DIRECTORY.getByName("directory").lookup(nameplate));
  if (!found.ok) return refuse(found.reason);
  const record = found.value;
  if (!record) return refuse("unknown nameplate");
  // Only these headers reach the mailbox; nothing the client sent is
  // forwarded except the upgrade itself.
  const headers = new Headers({
    Upgrade: "websocket",
    "x-td-nameplate": String(nameplate),
    "x-td-mailbox": record.mailbox,
    "x-td-token": record.token,
    "x-td-created": String(record.created),
  });
  // Every code opens a brand-new mailbox object, and the platform now and
  // then fails that first request with a bare "internal error" (#119).
  // Opening is safe to repeat (the record is stored once; a socket left by
  // a failed try is never claimed and is closed at the claim deadline), so
  // it is tried again on a new stub as `mayRetry` allows. If it still
  // fails, the client gets a reason, not a 500.
  const opened = await withRetry(() =>
    env.MAILBOX.getByName(record.mailbox).fetch(
      new Request("https://mailbox.internal/", { method: "GET", headers }),
    ),
  );
  return opened.ok ? opened.value : refuse(opened.reason);
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname === "/" || url.pathname === "/v0") {
      return new Response("TotalDrop rendezvous server. https://totaldrop.app\n", {
        headers: { "content-type": "text/plain; charset=utf-8" },
      });
    }
    if (!isUpgrade(request)) return new Response("WebSocket only\n", { status: 426 });
    if (url.pathname === "/v0/allocate") return allocate(env);
    const m = /^\/v0\/mailbox\/([1-9][0-9]{0,9})$/.exec(url.pathname);
    if (m) {
      const n = Number(m[1]);
      if (n > 0xffff_ffff) return refuse("unknown nameplate");
      return mailbox(env, request, n);
    }
    return new Response("not found\n", { status: 404 });
  },
} satisfies ExportedHandler<Env>;
