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

export { Directory, Mailbox };

export interface Env {
  DIRECTORY: DurableObjectNamespace<Directory>;
  MAILBOX: DurableObjectNamespace<Mailbox>;
}

/** How long an allocation connection may take to send its request. */
const ALLOCATE_TIMEOUT = 10_000;
/** Tries at opening a mailbox object, and the pause before the second
 * (it grows by this much each time). */
const MAILBOX_ATTEMPTS = 3;
const MAILBOX_RETRY_DELAY = 50;

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
      const msg = decodeClient(await binary(event.data));
      if (msg.type !== "allocate") throw new Error("only allocate here");
      const result = await env.DIRECTORY.getByName("directory").allocate();
      if ("error" in result) {
        socket.send(server.error(result.error));
      } else {
        socket.send(server.allocated(result.nameplate, unhex(result.token)));
      }
    } catch (e) {
      socket.send(server.error(e instanceof Error ? e.message : "server error"));
    }
    socket.close(1000, "done");
  });
  return new Response(null, { status: 101, webSocket: pair[0] });
}

async function mailbox(env: Env, request: Request, nameplate: number): Promise<Response> {
  const record = await env.DIRECTORY.getByName("directory").lookup(nameplate);
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
  // Opening is idempotent (the record is stored once; a socket left by a
  // failed try is never claimed and is closed at the claim deadline), so
  // try again with a new stub, as Cloudflare advises, unless the object is
  // overloaded. If it still fails, the client gets a reason, not a 500.
  for (let attempt = 1; ; attempt++) {
    const forward = new Request("https://mailbox.internal/", { method: "GET", headers });
    try {
      return await env.MAILBOX.getByName(record.mailbox).fetch(forward);
    } catch (e) {
      const overloaded = (e as { overloaded?: unknown } | null)?.overloaded === true;
      if (overloaded || attempt >= MAILBOX_ATTEMPTS) {
        return refuse(overloaded ? "server busy" : "server error, try again");
      }
      await new Promise((r) => setTimeout(r, MAILBOX_RETRY_DELAY * attempt));
    }
  }
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
