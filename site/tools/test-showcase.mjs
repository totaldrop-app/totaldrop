// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test site/tools/test-showcase.mjs
//
// Checks the Mineral theme-color meta tags, the shipped app showcase assets
// on the home page, and the repository link on "Read the code" (#529).
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const root = new URL("../../", import.meta.url);
const read = (path) => readFileSync(new URL(path, root), "utf8");
const readBytes = (path) => readFileSync(new URL(path, root));

test("_routes.json excludes /assets/* so showcase screenshots bypass Functions", () => {
  const { exclude } = JSON.parse(read("site/public/_routes.json"));
  assert.equal(exclude.filter((rule) => rule === "/assets/*").length, 1);
  assert.equal(new Set(exclude).size, exclude.length, "no rule twice");
});

test("showcase screenshots match docs/assets/ byte for byte", () => {
  const home = read("site/public/index.html");
  const srcs = [...home.matchAll(/<img\b[^>]*\bsrc="(\/assets\/[^"]+)"/g)].map((m) => m[1]);
  assert.deepEqual(srcs, ["/assets/android.png", "/assets/linux.png"]);
  for (const src of srcs) {
    const name = src.slice("/assets/".length);
    assert.ok(
      readBytes(`site/public/assets/${name}`).equals(readBytes(`docs/assets/${name}`)),
      `${src} must match docs/assets/${name}`,
    );
  }
});

test("hero.source links directly to totaldrop-app/totaldrop", () => {
  const home = read("site/public/index.html");
  const match = home.match(/<a\b[^>]*\bdata-i18n="hero\.source"[^>]*>/);
  assert.ok(match, "hero.source link is present");
  assert.match(match[0], /\bhref="https:\/\/github\.com\/totaldrop-app\/totaldrop"/);
});

test("every HTML page sets Mineral theme-color, color-scheme, and text-scale meta tags", () => {
  const pages = [
    "index.html",
    "download/index.html",
    "donate/index.html",
    "roadmap/index.html",
    "brand/index.html",
    "team/index.html",
    "r/index.html",
    "404.html",
    "privacy.html",
  ];
  for (const page of pages) {
    const html = read(`site/public/${page}`);
    assert.match(
      html,
      /<meta name="color-scheme" content="light dark">/,
      `${page}: color-scheme meta`,
    );
    assert.match(
      html,
      /<meta name="text-scale" content="scale">/,
      `${page}: text-scale meta`,
    );
    assert.match(
      html,
      /<meta name="theme-color" content="#F7F7F5" media="\(prefers-color-scheme: light\)">/,
      `${page}: light theme-color`,
    );
    assert.match(
      html,
      /<meta name="theme-color" content="#0F1116" media="\(prefers-color-scheme: dark\)">/,
      `${page}: dark theme-color`,
    );
  }
});

test("styles.css uses zoom-safe rem+vw fluid font-size clamps and metric fallback adjustment", () => {
  const css = read("site/public/styles.css");
  assert.match(css, /font-size-adjust:\s*from-font;/);
  const clamps = [...css.matchAll(/font-size:\s*clamp\(([^)]+)\)/g)].map((m) => m[1]);
  assert.ok(clamps.length >= 8, "expected fluid font-size clamp rules");
  for (const expr of clamps) {
    const parts = expr.split(",").map((s) => s.trim());
    assert.equal(parts.length, 3, `clamp(${expr}) has 3 arguments`);
    assert.match(parts[1], /rem\s*\+\s*[\d.]+vw/, `clamp(${expr}) preferred value mixes rem + vw`);
  }
});

test("team page governance copy reflects co-maintainer merges and project-lead releases", () => {
  const team = read("site/public/team/index.html");
  const en = JSON.parse(read("site/public/i18n/en.json"));
  assert.doesNotMatch(team, /only the project lead can merge/i);
  assert.doesNotMatch(team, /Only the project lead merges/i);
  assert.match(en["tm.tools.lede"], /maintainer approves or merges every change/);
  assert.match(en["tm.how.h3"], /Maintainers merge reviewed changes/);
});

