// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// End-to-end check of the mail Worker inside workerd (#415): starts
// `wrangler dev --local` on loopback and posts a raw RFC 5322 message to
// `/cdn-cgi/handler/email`, checking multi-recipient forwarding and
// fail-closed rejection without a Cloudflare account.

import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { request as httpRequest } from "node:http";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const wrangler = join(root, "node_modules", ".bin", "wrangler");

const SAMPLE_MAIL = [
  "From: Sender <sender@example.net>",
  "To: hello@totaldrop.app",
  "Subject: Hello",
  "Message-ID: <mail-worker-test@example.net>",
  "Date: Sat, 14 Nov 2026 15:00:00 +0000",
  "",
  "Hello maintainers.",
  "",
].join("\r\n");

async function waitFor<T>(fn: () => T | null, timeoutMs: number, label: string): Promise<T> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const value = fn();
    if (value !== null) return value;
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error(`timed out waiting for ${label}`);
}

function postEmail(port: number, from: string, to: string, raw: string): Promise<{ status: number; body: string }> {
  return new Promise((resolve, reject) => {
    const path = `/cdn-cgi/handler/email?from=${encodeURIComponent(from)}&to=${encodeURIComponent(to)}`;
    const req = httpRequest(
      {
        host: "127.0.0.1",
        port,
        path,
        method: "POST",
        headers: {
          "content-type": "message/rfc822",
          "content-length": Buffer.byteLength(raw),
        },
      },
      (res) => {
        const chunks: Buffer[] = [];
        res.on("data", (c: Buffer) => chunks.push(c));
        res.on("end", () =>
          resolve({ status: res.statusCode ?? 0, body: Buffer.concat(chunks).toString("utf8") }),
        );
      },
    );
    req.on("error", reject);
    req.end(raw);
  });
}

async function withWorker<T>(
  vars: Record<string, string>,
  run: (port: number, output: () => string) => Promise<T>,
): Promise<T> {
  const args = ["dev", "--local", "--ip", "127.0.0.1", "--port", "0", "--inspector-port", "0"];
  for (const [k, v] of Object.entries(vars)) args.push("--var", `${k}:${v}`);

  let log = "";
  let exited: number | null = null;
  const child = spawn(wrangler, args, {
    cwd: root,
    detached: true,
    stdio: ["ignore", "pipe", "pipe"],
    env: {
      ...process.env,
      WRANGLER_SEND_METRICS: "false",
      NO_PROXY: "127.0.0.1,localhost",
    },
  });
  child.stdout?.on("data", (chunk: Buffer) => {
    log += chunk.toString("utf8");
  });
  child.stderr?.on("data", (chunk: Buffer) => {
    log += chunk.toString("utf8");
  });
  child.on("exit", (code) => {
    exited = code ?? -1;
  });

  try {
    const port = await waitFor(() => {
      if (exited !== null) throw new Error(`wrangler exited (${exited}):\n${log}`);
      const m = /Ready on http:\/\/127\.0\.0\.1:(\d+)/.exec(log);
      return m?.[1] ? Number(m[1]) : null;
    }, 30_000, "wrangler dev Ready");
    return await run(port, () => log);
  } finally {
    if (child.pid !== undefined && exited === null) {
      try {
        process.kill(-child.pid, "SIGTERM");
      } catch {
        child.kill("SIGTERM");
      }
      await waitFor(() => (exited !== null ? true : null), 5_000, "wrangler exit").catch(() => {
        if (child.pid !== undefined) {
          try {
            process.kill(-child.pid, "SIGKILL");
          } catch {
            child.kill("SIGKILL");
          }
        }
      });
    }
  }
}

test("wrangler dev forwards one incoming email to every configured recipient", async () => {
  await withWorker({ FORWARD_TO: "alice@example.com, bob@example.org" }, async (port, output) => {
    const res = await postEmail(port, "sender@example.net", "hello@totaldrop.app", SAMPLE_MAIL);
    assert.equal(res.status, 200, res.body);
    assert.match(res.body, /Worker successfully processed email/);
    await waitFor(
      () =>
        output().includes("rcptTo: alice@example.com") && output().includes("rcptTo: bob@example.org")
          ? true
          : null,
      5_000,
      "both forwarded recipients in wrangler log",
    );
  });
});

test("wrangler dev fails closed without leaking the address when FORWARD_TO loops or is unset", async () => {
  await withWorker({ FORWARD_TO: "secret-loop@totaldrop.app" }, async (port, output) => {
    const res = await postEmail(port, "sender@example.net", "hello@totaldrop.app", SAMPLE_MAIL);
    assert.equal(res.status, 500);
    assert.match(res.body, /on totaldrop\.app and would loop/);
    assert.ok(!res.body.includes("secret-loop@totaldrop.app"));
    assert.ok(!output().includes("secret-loop@totaldrop.app"));
    assert.ok(!output().includes("Email handler forwarded message"));
  });

  await withWorker({}, async (port, output) => {
    const res = await postEmail(port, "sender@example.net", "hello@totaldrop.app", SAMPLE_MAIL);
    assert.equal(res.status, 500);
    assert.match(res.body, /FORWARD_TO is not set/);
    assert.ok(!output().includes("Email handler forwarded message"));
  });
});
