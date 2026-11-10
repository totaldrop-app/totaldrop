// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Tests for the team page's handling of GitHub profile text (#288), which
// comes from people outside the project and ends up in the page's HTML:
//   node --test site/tools/test-team.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { clean, website, esc, LOGIN } from "./team.mjs";

test("clean removes control, direction and invisible characters", () => {
  assert.equal(clean("A‮B⁦C​D\u0000E F", 80), "A B C D E F");
  assert.equal(clean("a\u2060b\u{e0041}c", 80), "a b c");
  assert.equal(clean("  many   spaces\n\tand lines ", 80), "many spaces and lines");
  assert.equal(clean(null, 80), "");
  assert.equal(clean(42, 80), "");
});

test("clean cuts long text by characters, not bytes", () => {
  assert.equal(clean("çğışöü".repeat(10), 5), "çğış…");
  assert.equal(Array.from(clean("😀".repeat(300), 240)).length, 240);
  assert.equal(clean("short", 5), "short");
});

test("clean is idempotent", () => {
  const once = clean("x‮y  z".repeat(50), 80);
  assert.equal(clean(once, 80), once);
});

test("website accepts https and bare host names only", () => {
  assert.equal(website("ozanozdil.com"), "https://ozanozdil.com/");
  assert.equal(website("https://sadikkuzu.com/linktree/"), "https://sadikkuzu.com/linktree/");
  assert.equal(website("http://example.com"), null);
  assert.equal(website("javascript:alert(1)"), null);
  assert.equal(website("https://user:pw@example.com"), null);
  assert.equal(website("https://example.com:8443/"), null);
  assert.equal(website("localhost"), null);
  assert.equal(website(""), null);
  assert.equal(website(null), null);
});

test("esc makes text safe inside elements and quoted attributes", () => {
  assert.equal(esc(`<img src=x onerror="a">&`), "&lt;img src=x onerror=&quot;a&quot;&gt;&amp;");
});

test("login rule matches GitHub's", () => {
  for (const ok of ["ozdil", "sadikkuzu", "a-b", "dependabot[bot]", "x".repeat(39)]) assert.ok(LOGIN.test(ok), ok);
  for (const bad of ["", "-a", "a-", "a--b", "a b", "a/b", "../x", "x".repeat(40), "<x>"]) assert.ok(!LOGIN.test(bad), bad);
});

test("bar colours follow the role, not the order in team.json", async () => {
  const { series } = await import("./team.mjs");
  const c = { people: [
    { login: "b", kind: "person", role: "contributor" },
    { login: "a", kind: "person", role: "maintainer" },
    { login: "t", kind: "person", role: "tester" },
    { login: "s", kind: "person", role: "principal" },
    { login: "claude", kind: "ai" },
    { login: "dependabot[bot]", kind: "bot" },
  ] };
  assert.equal(series(c, "a"), "s-maintainer");
  assert.equal(series(c, "b"), "s-contributor");
  assert.equal(series(c, "t"), "s-contributor");
  assert.equal(series(c, "s"), "s-contributor");
  assert.equal(series(c, "claude"), "s-ai");
  assert.equal(series(c, "dependabot[bot]"), "s-bot");
  assert.equal(series(c, ""), "s-other");
  assert.equal(series(c, "someone"), "s-other");
});

test("a team.json bio replaces the GitHub bio, escaped, under its tm.bio key (#408)", async () => {
  const { bioLine } = await import("./team.mjs");
  const own = bioLine({ login: "ozdil", bio: "Leads product, design and releases." }, { bio: "Other projects." });
  assert.equal(own, `<p class="bio" data-i18n="tm.bio.ozdil">Leads product, design and releases.</p>`);
  assert.equal(bioLine({ login: "x" }, { bio: "QA <specialist>" }), `<p class="bio" dir="auto">QA &lt;specialist&gt;</p>`);
  assert.equal(bioLine({ login: "x" }, { bio: "" }), "");
  assert.equal(bioLine({ login: "x", bio: `A <b> & "c"` }, { bio: "" }), `<p class="bio" data-i18n="tm.bio.x">A &lt;b&gt; &amp; &quot;c&quot;</p>`);
});

test("a bio in team.json must be clean, one line and not empty (#408)", async () => {
  const { badBio } = await import("./team.mjs");
  assert.equal(badBio(undefined), false);
  assert.equal(badBio("Leads product, design and releases."), false);
  assert.equal(badBio("x".repeat(240)), false);
  for (const bad of ["", " padded ", "two  spaces", "a‮b", "x".repeat(241), 42, null]) assert.ok(badBio(bad), JSON.stringify(bad));
});

test("a team.json bio needs a translation in every language file (#408)", async (t) => {
  const { missingBio } = await import("./team.mjs");
  const dir = mkdtempSync(join(tmpdir(), "team-i18n-"));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  writeFileSync(join(dir, "en.json"), JSON.stringify({ "tm.bio.x": "Leads." }));
  writeFileSync(join(dir, "tr.json"), JSON.stringify({ "tm.bio.x": " " }));
  writeFileSync(join(dir, "pt-BR.json"), JSON.stringify({}));
  writeFileSync(join(dir, "notes.txt"), "");
  assert.deepEqual(missingBio(dir, "x"), ["pt-BR", "tr"]);
  // The real files: every language has the project lead's line.
  assert.deepEqual(missingBio(fileURLToPath(new URL("../public/i18n/", import.meta.url)), "ozdil"), []);
});

test("a broken language file is reported as invalid, not thrown or called untranslated (#410)", async (t) => {
  const { missingBio, collectErrors } = await import("./team.mjs");
  const dir = mkdtempSync(join(tmpdir(), "team-i18n-"));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  writeFileSync(join(dir, "en.json"), JSON.stringify({ "tm.bio.x": "Leads." }));
  writeFileSync(join(dir, "de.json"), "{ not json");
  writeFileSync(join(dir, "fr.json"), JSON.stringify({}));
  let missing;
  const errs = collectErrors(() => { missing = missingBio(dir, "x"); });
  assert.deepEqual(missing, ["fr"]);
  assert.equal(errs.length, 1);
  assert.match(errs[0], /(^|\/)de\.json: not valid JSON$/);
});

test("team.json rejects a bio that is bad, untranslated or on a non-person (#408)", async () => {
  const { configErrors } = await import("./team.mjs");
  const orig = JSON.parse(readFileSync(new URL("../team/team.json", import.meta.url), "utf8"));
  const errs = (edit) => { const c = structuredClone(orig); edit(c); return configErrors(c).join("\n"); };
  const person = (c, login) => c.people.find((p) => p.login === login);
  assert.equal(errs(() => {}), "");
  assert.match(errs((c) => { person(c, "ozdil").bio = " padded"; }), /ozdil: bad bio/);
  assert.match(errs((c) => { person(c, "sadikkuzu").bio = "Untranslated."; }), /sadikkuzu: tm\.bio\.sadikkuzu is not translated in [a-z]{2,3}(-[A-Za-z0-9]+)*\.json/);
  assert.match(errs((c) => { person(c, "claude").bio = "x"; }), /claude: only people have a bio/);
});
