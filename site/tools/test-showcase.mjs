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

test("every HTML page sets Mineral light and dark theme-color meta tags", () => {
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
