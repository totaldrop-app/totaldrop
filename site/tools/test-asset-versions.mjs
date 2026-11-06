// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test site/tools/test-asset-versions.mjs
import test from "node:test";
import assert from "node:assert/strict";
import { assetRef, versioned, problems } from "./asset-versions.mjs";

const css = assetRef("styles.css");

test("the address carries 10 hex digits of the file's hash", () => {
  assert.match(css, /^\/styles\.css\?v=[0-9a-f]{10}$/);
  assert.throws(() => assetRef("other.css"));
});

test("a bare or old reference is rewritten, and rewriting twice changes nothing", () => {
  for (const html of ['<link rel="stylesheet" href="/styles.css">', '<link rel="stylesheet" href="/styles.css?v=0000000000">']) {
    const once = versioned(html);
    assert.equal(once, `<link rel="stylesheet" href="${css}">`);
    assert.equal(versioned(once), once);
    assert.deepEqual(problems(html), ["stale"]);
    assert.deepEqual(problems(once), []);
  }
  assert.equal(versioned('<script src="/app.js" defer></script>'), `<script src="${assetRef("app.js")}" defer></script>`);
});

test("references the rewrite does not recognise are reported", () => {
  for (const html of ["<link href='/styles.css'>", '<link href="styles.css">', '<link href="/styles.css?x=1">', '<script src="./app.js"></script>']) {
    assert.ok(problems(html).some((p) => p.startsWith("unrecognised reference")), html);
  }
});

test("other files and text are left alone", () => {
  const html = '<link rel="icon" href="/favicon.svg"><p>styles.css</p><a href="/brand/">x</a>';
  assert.equal(versioned(html), html);
  assert.deepEqual(problems(html), []);
});
