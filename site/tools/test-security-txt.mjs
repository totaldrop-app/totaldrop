// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test site/tools/test-security-txt.mjs
//
// /.well-known/security.txt (RFC 9116, #450) tells researchers where to
// report a vulnerability. It must name the same channels as SECURITY.md,
// and its Expires date must be ahead and less than a year away (RFC 9116).
// Renewing it in time is tracked in an issue, not here: a date guard in
// every pull request would turn unrelated ones red.
import test from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";

const file = new URL("../public/.well-known/security.txt", import.meta.url).pathname;
const policy = readFileSync(new URL("../../SECURITY.md", import.meta.url), "utf8");
const DAY = 24 * 60 * 60 * 1000;

function fields(text) {
  const out = new Map();
  for (const line of text.split("\n")) {
    if (line === "" || line.startsWith("#")) continue;
    const m = /^([A-Za-z-]+): (.+)$/.exec(line);
    assert.ok(m, `not a field: ${JSON.stringify(line)}`);
    const key = m[1].toLowerCase();
    out.set(key, [...(out.get(key) ?? []), m[2]]);
  }
  return out;
}

test("security.txt is published as UTF-8 text, without the site's Function", () => {
  assert.ok(existsSync(file), "site/public/.well-known/security.txt is missing");
  const headers = readFileSync(new URL("../public/_headers", import.meta.url), "utf8");
  assert.match(headers, /^\/\.well-known\/security\.txt\n  ! Content-Type\n  Content-Type: text\/plain; charset=utf-8$/m);
  const routes = JSON.parse(readFileSync(new URL("../public/_routes.json", import.meta.url), "utf8"));
  assert.ok(routes.exclude.includes("/.well-known/*"), "_routes.json must exclude /.well-known/*");
});

test("it names the channels SECURITY.md names, and only those", () => {
  const f = fields(readFileSync(file, "utf8"));
  const contacts = f.get("contact") ?? [];
  assert.deepEqual(contacts, [
    "mailto:security@totaldrop.app",
    "https://github.com/totaldrop-app/totaldrop/security/advisories/new",
  ]);
  assert.ok(policy.includes("security@totaldrop.app"), "SECURITY.md no longer names security@");
  assert.ok(policy.includes("private vulnerability reporting"), "SECURITY.md no longer names GitHub's private reporting");
  assert.deepEqual(f.get("policy"), ["https://github.com/totaldrop-app/totaldrop/blob/main/SECURITY.md"]);
  assert.deepEqual(f.get("canonical"), [
    "https://totaldrop.app/.well-known/security.txt",
    "https://download.totaldrop.app/.well-known/security.txt",
  ]);
  assert.deepEqual(f.get("preferred-languages"), ["en, tr"]);
});

test("Expires is one RFC 3339 time, ahead and less than a year away", () => {
  const f = fields(readFileSync(file, "utf8"));
  const expires = f.get("expires") ?? [];
  assert.equal(expires.length, 1, "exactly one Expires");
  assert.match(expires[0], /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/);
  const left = Date.parse(expires[0]) - Date.now();
  assert.ok(left > 0, `security.txt expired on ${expires[0]}: renew it`);
  assert.ok(left < 365 * DAY, `security.txt expires on ${expires[0]}: RFC 9116 asks for less than a year`);
});
