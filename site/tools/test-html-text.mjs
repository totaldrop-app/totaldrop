// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test site/tools/test-html-text.mjs
import test from "node:test";
import assert from "node:assert/strict";
import { decodeEntities, stripComments } from "./html-text.mjs";

function collect(fn, input) {
  const bad = [];
  return { out: fn(input, (b) => bad.push(b)), bad };
}

test("named and numeric references decode to their characters", () => {
  assert.equal(decodeEntities("phone&#x27;s &amp; Apple&#39;s &mdash; &lt;b&gt; &nbsp;&copy;"), "phone's & Apple's \u2014 <b> \u00a0\u00a9");
  assert.equal(decodeEntities("&#X41;&#x6a;&#x6A;"), "Ajj", "the x and the hex digits may be either case");
});

test("a reference is decoded once: its result is never read again", () => {
  assert.equal(decodeEntities("&#x26;lt;"), "&lt;");
  assert.equal(decodeEntities("&#38;amp;"), "&amp;");
  assert.equal(decodeEntities("&amp;lt;"), "&lt;");
  assert.equal(decodeEntities("&amp;#x3c;"), "&#x3c;");
});

test("unknown names and numbers past Unicode are reported and kept", () => {
  const { out, bad } = collect(decodeEntities, "&bogus; &#x110000; &#99999999999; &constructor; &AMP; ok");
  assert.equal(out, "&bogus; &#x110000; &#99999999999; &constructor; &AMP; ok");
  assert.deepEqual(bad, ["&bogus;", "&#x110000;", "&#99999999999;", "&constructor;", "&AMP;"]);
});

test("numbers a browser replaces or remaps are reported and kept", () => {
  const input = "&#0; &#xD800; &#xdfff; &#x80; &#159; &#x7f; &#xa0;";
  const { out, bad } = collect(decodeEntities, input);
  assert.equal(out, "&#0; &#xD800; &#xdfff; &#x80; &#159; \u007f \u00a0");
  assert.deepEqual(bad, ["&#0;", "&#xD800;", "&#xdfff;", "&#x80;", "&#159;"]);
});

test("text without references is unchanged, and so is a lone ampersand", () => {
  assert.equal(decodeEntities("Tom & Jerry; a&b"), "Tom & Jerry; a&b");
});

test("comments are removed up to the first end, as a browser reads them", () => {
  assert.equal(stripComments("a<!-- x -->b<!--y-->c"), "abc");
  assert.equal(stripComments("a<!-- <!-- x -->b -->c"), "ab -->c");
  assert.equal(stripComments("a<!-->b<!--->c"), "abc");
  assert.equal(stripComments("a<!-- x --!><p>b</p><!-- y -->c"), "a<p>b</p>c");
  assert.equal(stripComments("a<!-- x -->b --!>c"), "ab --!>c");
  assert.equal(stripComments("no comments"), "no comments");
});

test("text joined into a new comment start is reported", () => {
  const { out, bad } = collect(stripComments, "<!-<!-- x -->- y -->");
  assert.equal(out, "<!-- y -->");
  assert.equal(bad.length, 1);
});

test("a comment that never ends hides the rest and is reported", () => {
  const { out, bad } = collect(stripComments, "keep<!-- open");
  assert.equal(out, "keep");
  assert.deepEqual(bad, ["a comment that never ends"]);
});
