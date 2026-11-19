// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test site/tools/test-privacy.mjs
//
// Tests for site/tools/privacy.mjs and the generated privacy policy pages
// (#207, #416): link allowlist, accessible mailto labels, Bidi safety, and
// coverage of hello@, privacy@ and security@ across all 35 languages without
// exposing personal mailbox addresses.
import test from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { TAGS, file, isRtl, lint, page, shape } from "./privacy.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const privacySrc = join(here, "..", "privacy");
const PROJECT_EMAILS = new Set([
  "hello@totaldrop.app",
  "privacy@totaldrop.app",
  "security@totaldrop.app",
]);

test("lint allows the three project mailto addresses and Cloudflare's privacy policy", () => {
  const ok = [
    '<p class="eyebrow">Privacy policy</p>',
    "<h1>Heading</h1>",
    '<p class="lead">Lead.</p>',
    "<h2>When you email us</h2>",
    '<p><a href="mailto:hello@totaldrop.app">hello@totaldrop.app</a>, ' +
      '<a href="mailto:privacy@totaldrop.app">privacy@totaldrop.app</a>, ' +
      '<a href="mailto:security@totaldrop.app">security@totaldrop.app</a> and ' +
      '<a href="https://www.cloudflare.com/privacypolicy/">privacy policy</a>.</p>',
  ].join("\n");
  assert.deepEqual(lint("en", ok), []);
});

test("lint rejects external or unlisted mailto targets and mismatched or unclosed mailto links", () => {
  for (const bad of [
    "mailto:alice@example.com",
    "mailto:maintainer@gmail.com",
    "mailto:press@totaldrop.app",
    "mailto:hello@evil-totaldrop.app",
    "https://example.com/",
  ]) {
    const errs = lint("en", `<p><a href="${bad}">${bad.replace(/^mailto:/, "")}</a></p>`);
    assert.equal(errs.length, 1, bad);
    assert.match(errs[0], /attribute href=".*" is not allowed$/, bad);
  }

  for (const html of [
    '<p><a href="mailto:hello@totaldrop.app">privacy@totaldrop.app</a></p>',
    '<p><a href="mailto:security@totaldrop.app">email us</a></p>',
  ]) {
    const errs = lint("en", html);
    assert.equal(errs.length, 1, html);
    assert.match(errs[0], /mailto:.* link text must be /, html);
  }

  for (const html of [
    '<p><a href="mailto:hello@totaldrop.app">hello@totaldrop.app</p>',
    '<p><a href="mailto:hello@totaldrop.app"><code>hello@totaldrop.app</code></a></p>',
  ]) {
    const errs = lint("en", html);
    assert.equal(errs.length, 1, html);
    assert.match(errs[0], /unclosed or nested mailto link$/, html);
  }

  assert.match(lint("en", "<p><a>hello@totaldrop.app</a></p>")[0] ?? "", /attribute \(missing href\) is not allowed$/);
  assert.match(lint("en", '<p href="mailto:hello@totaldrop.app">text</p>')[0] ?? "", /attribute href="mailto:hello@totaldrop\.app" is not allowed$/);
  assert.match(lint("en", '<p><a href="mailto:hello@totaldrop.app">hello@totaldrop.app</a href="mailto:hello@totaldrop.app"></p>')[0] ?? "", /attribute href="mailto:hello@totaldrop\.app" is not allowed$/);
  assert.match(lint("en", '<h2 class="lead">Heading</h2>')[0] ?? "", /attribute class="lead" is not allowed$/);
  assert.match(lint("en", "<p>bad\u202etext</p>")[0] ?? "", /invisible direction characters$/);
  assert.match(lint("en", "<p>a & b</p>")[0] ?? "", /unescaped &$/);
});

test("page isolates mailto links with dir=ltr on RTL pages while leaving prose links untouched", () => {
  const sample =
    '<p><a href="mailto:hello@totaldrop.app">hello@totaldrop.app</a>، ' +
    '<a href="mailto:privacy@totaldrop.app">privacy@totaldrop.app</a> — ' +
    '<a href="https://www.cloudflare.com/privacypolicy/">سیاست حریم خصوصی</a>.</p>';
  const rtlOut = page("fa", sample, {});
  assert.ok(rtlOut.includes('<a dir="ltr" href="mailto:hello@totaldrop.app">hello@totaldrop.app</a>، <a dir="ltr" href="mailto:privacy@totaldrop.app">privacy@totaldrop.app</a>'));
  assert.ok(rtlOut.includes('<a href="https://www.cloudflare.com/privacypolicy/">سیاست حریم خصوصی</a>'));
  assert.ok(rtlOut.includes('<a href="https://totaldrop.app/">totaldrop.app</a> <a dir="ltr" href="mailto:privacy@totaldrop.app">privacy@totaldrop.app</a>'));
  assert.ok(!rtlOut.includes('<a href="mailto:'));

  const ltrOut = page("en", sample, {});
  assert.ok(!ltrOut.includes('<a dir="ltr" href="mailto:'));
});

test("privacy.mjs check passes and every language covers hello@, privacy@ and security@", () => {
  const out = execFileSync(process.execPath, [join(here, "privacy.mjs")], { encoding: "utf8" });
  assert.match(out, /^privacy policy ok: 35 languages\n$/);
  assert.equal(TAGS.length, 35);

  const enHtml = readFileSync(join(privacySrc, "en.html"), "utf8");
  assert.match(enHtml, /^<p class="eyebrow">Privacy policy · version 10 · 19 November 2026<\/p>\n/);
  assert.match(enHtml, /<h2>When you email us<\/h2>[\s\S]*<h2>Children<\/h2>/);
  const enShape = shape(enHtml);
  assert.equal(enShape.blocks[1], "h1");
  assert.ok(!enShape.blocks.includes("h3"), "heading hierarchy stays sequential h1 -> h2");

  for (const tag of TAGS) {
    const srcHtml = readFileSync(join(privacySrc, `${tag}.html`), "utf8");
    const pubHtml = readFileSync(file(tag), "utf8");
    const rtl = isRtl(tag);
    for (const addr of PROJECT_EMAILS) {
      const expectedLink = rtl
        ? `<a dir="ltr" href="mailto:${addr}">${addr}</a>`
        : `<a href="mailto:${addr}">${addr}</a>`;
      assert.ok(pubHtml.includes(expectedLink), `${tag}: missing ${expectedLink}`);
      assert.ok(
        pubHtml.includes(`<code>${addr}</code>`),
        `${tag}: missing <code>${addr}</code>`,
      );
    }
    assert.ok(pubHtml.includes("<code>totaldrop.app</code>"), `${tag}: missing <code>totaldrop.app</code>`);

    for (const text of [srcHtml, pubHtml]) {
      const emails = text.match(/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+/g) ?? [];
      for (const found of emails) {
        assert.ok(PROJECT_EMAILS.has(found), `${tag}: unexpected email address ${found}`);
      }
      assert.doesNotMatch(text, /[\u200e\u200f\u061c\u202a-\u202e\u2066-\u2069]/);
    }

    if (rtl) {
      assert.match(pubHtml, new RegExp(`<html lang="${tag}" dir="rtl">`));
      assert.match(pubHtml, /<em dir="ltr">/);
      assert.match(pubHtml, /<bdi dir="ltr">/);
      assert.ok(!pubHtml.includes('<a href="mailto:'), `${tag}: unisolated mailto link on RTL page`);
    } else {
      assert.ok(!pubHtml.includes('<a dir="ltr" href="mailto:'), `${tag}: unexpected dir=ltr on LTR mailto link`);
    }
  }
});
