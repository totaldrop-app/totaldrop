// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test site/tools/test-donate.mjs
//
// The /donate/ page sets no funding goal or yearly budget target (#636):
// donations are spent only as funds arrive, in priority order. These checks
// guard the no-goal structure, keyboard-accessible table wrappers, and
// 35-language completeness of every key on the page.
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const pub = new URL("../public/", import.meta.url).pathname;
const html = readFileSync(join(pub, "donate/index.html"), "utf8");
const css = readFileSync(join(pub, "styles.css"), "utf8");

test("donate page has no funding progress bar, yearly total row, or dead .goal styles (#636)", () => {
  assert.doesNotMatch(html, /<progress\b/i, "donate page must not render a goal progress bar");
  assert.doesNotMatch(html, /<tfoot\b/i, "donate cost table must not sum items into a yearly total");
  assert.doesNotMatch(css, /\.goal\b/, "unused .goal progress styles should stay removed");
});

test("donate summary shows the two-card raised/spent grid and no third goal card (#636)", () => {
  const m = /<dl class="costs costs-2">([\s\S]*?)<\/dl>/.exec(html);
  assert.ok(m, "expected a two-column <dl class=\"costs costs-2\"> summary");
  const cards = [...m[1].matchAll(/<div>\s*<dt\b/g)];
  assert.equal(cards.length, 2, "summary should contain only raised and spent cards");
  assert.match(css, /\.costs\.costs-2\s*\{[^}]*repeat\(2,/);
});

test("every scrollable table wrapper on /donate/ is a keyboard-focusable named region (#636)", () => {
  const wrappers = [...html.matchAll(/<div class="compare-wrap"([^>]*)>/g)];
  assert.equal(wrappers.length, 3, "expected ledger, priority costs, and relay tables");
  for (const [, attrs] of wrappers) {
    assert.match(attrs, /\btabindex="0"/, `missing tabindex="0" on ${attrs}`);
    assert.match(attrs, /\brole="region"/, `missing role="region" on ${attrs}`);
    const ref = /\baria-labelledby="([^"]+)"/.exec(attrs);
    assert.ok(ref, `missing aria-labelledby on ${attrs}`);
    assert.ok(html.includes(`id="${ref[1]}"`), `aria-labelledby target #${ref[1]} not found`);
  }
});

test("every donate page key is translated in all 35 locales and removed goal keys stay absent (#636)", () => {
  const keys = new Set(["meta.donate.title", "meta.donate.desc"]);
  for (const [, k] of html.matchAll(/\bdata-i18n="([^"]+)"/g)) keys.add(k);
  for (const [, k] of html.matchAll(/\bdata-i18n-label="([^"]+)"/g)) keys.add(k);

  const removed = [
    "don.raised.t3",
    "don.raised.d3",
    "don.raised.goal",
    "don.col.year",
    "don.m.1420",
    "don.total",
    "don.total.c",
  ];
  const i18nDir = join(pub, "i18n");
  const files = readdirSync(i18nDir).filter((f) => f.endsWith(".json")).sort();
  assert.equal(files.length, 35, "expected 35 locale catalogs");
  for (const file of files) {
    const data = JSON.parse(readFileSync(join(i18nDir, file), "utf8"));
    for (const k of keys) {
      assert.equal(typeof data[k], "string", `${file}: missing ${k}`);
      assert.ok(data[k].trim().length > 0, `${file}: empty ${k}`);
    }
    for (const old of removed) {
      assert.equal(old in data, false, `${file}: stale goal key ${old} must be removed`);
    }
  }
});
