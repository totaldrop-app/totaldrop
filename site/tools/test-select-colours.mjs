// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test site/tools/test-select-colours.mjs
//
// The list a <select> opens is drawn by the browser and the system, not by the
// page. Its options inherit the page's text colour, but the list's background
// falls back to the system's (white in Chromium on Linux) while the select
// itself is transparent. In the dark theme that gave light text on
// white (#293). Every option therefore names both colours itself.
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

const root = new URL("../public/", import.meta.url).pathname;
const css = readFileSync(join(root, "styles.css"), "utf8").replace(/\/\*[\s\S]*?\*\//g, "");

function declarations(selector) {
  const rules = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)];
  const found = {};
  for (const [, sel, body] of rules) {
    if (!sel.split(",").map((s) => s.trim()).includes(selector)) continue;
    for (const decl of body.split(";")) {
      const i = decl.indexOf(":");
      if (i > 0) found[decl.slice(0, i).trim()] = decl.slice(i + 1).trim();
    }
  }
  return found;
}

test("the language list's options set their own background and text colour", () => {
  const d = declarations(".lang option");
  assert.equal(d["background-color"], "var(--bg)");
  assert.equal(d["color"], "var(--text)");
  const checked = declarations(".lang option:checked");
  assert.equal(checked["background-color"], "var(--accent)");
  assert.equal(checked["color"], "var(--on-accent)");
});

test("every select on the site is a .lang select, so the rule reaches it", () => {
  const pages = [];
  const walk = (dir) => {
    for (const name of readdirSync(dir)) {
      const p = join(dir, name);
      if (statSync(p).isDirectory()) walk(p);
      else if (name.endsWith(".html")) pages.push(p);
    }
  };
  walk(root);
  let selects = 0;
  for (const page of pages) {
    for (const [tag] of readFileSync(page, "utf8").matchAll(/<select\b[^>]*>/g)) {
      selects++;
      assert.match(tag, /class="(?:[^"]*\s)?lang(?:\s[^"]*)?"/, `${page}: ${tag}`);
    }
  }
  assert.ok(selects > 0);
});
