// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test tools/test-check-doc-refs.mjs

import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { applyMoves, check, findRefs, locate } from "./check-doc-refs.mjs";

const here = dirname(fileURLToPath(import.meta.url));

const code = ["fn a() {", "    one();", "}", "", "fn b() {", "    two();", "}"];
const files = { "src/x.rs": code };
const read = (path) => files[path] ?? null;

test("references are full paths with a line or a range; addresses are not references", () => {
  const refs = findRefs("see `src/x.rs:1` and `src/x.rs:5-7`, not `127.0.0.1:4000` or `x.rs`");
  assert.deepEqual(
    refs.map((r) => [r.key, r.first, r.last]),
    [
      ["src/x.rs:1", 1, 1],
      ["src/x.rs:5-7", 5, 7],
    ],
  );
});

test("a reference on its recorded line passes", () => {
  const r = check("`src/x.rs:5-7`", { "src/x.rs:5-7": ["fn b() {", "}"] }, read);
  assert.deepEqual(r.problems, []);
});

test("a short name, a line past the end and an unrecorded reference are reported", () => {
  const r = check("`x.rs:1` `src/x.rs:9` `src/x.rs:2`", {}, read);
  assert.deepEqual(r.problems, [
    "x.rs:1: no such file; write the path from the repository root",
    "src/x.rs:9: the file has 7 lines",
    "src/x.rs:2: not in docs/ARCHITECTURE.refs.json; check the claim, then run with --accept",
  ]);
});

test("code that only moved is followed, and --fix rewrites the document", () => {
  files["src/y.rs"] = ["// new line", ...code];
  const doc = "a `src/y.rs:5-7` b `src/y.rs:2`";
  const r = check(doc, { "src/y.rs:5-7": ["fn b() {", "}"], "src/y.rs:2": ["one();"] }, read);
  assert.deepEqual(r.problems, [
    "src/y.rs:5-7: moved to src/y.rs:6-8; run with --fix",
    "src/y.rs:2: moved to src/y.rs:3; run with --fix",
  ]);
  assert.equal(applyMoves(doc, r.moves), "a `src/y.rs:6-8` b `src/y.rs:3`");
});

test("a changed line is reported, not followed", () => {
  const r = check("`src/x.rs:2`", { "src/x.rs:2": ["three();"] }, read);
  assert.deepEqual(r.problems, [
    "src/x.rs:2: the cited line changed (was `three();`); read the claim again, correct it, then run with --accept",
  ]);
});

test("two equally near copies of the text are not guessed between", () => {
  const lines = ["x", "}", "y", "}"];
  assert.equal(locate(lines, { first: 3, last: 3 }, ["}"]), null);
  assert.deepEqual(locate(lines, { first: 1, last: 1 }, ["}"]), { first: 2, last: 2 });
});

test("this repository's document points at its recorded lines", () => {
  const out = execFileSync("node", [join(here, "check-doc-refs.mjs")], { encoding: "utf8" });
  assert.match(out, /^doc refs ok: \d+ references in docs\/ARCHITECTURE\.md$/m);
});

test("--fix rewrites the document and the record, then the check passes", () => {
  const dir = mkdtempSync(join(tmpdir(), "doc-refs-"));
  try {
    mkdirSync(join(dir, "tools"));
    mkdirSync(join(dir, "docs"));
    mkdirSync(join(dir, "src"));
    cpSync(join(here, "check-doc-refs.mjs"), join(dir, "tools", "check-doc-refs.mjs"));
    writeFileSync(join(dir, "src", "x.rs"), code.join("\n"));
    writeFileSync(join(dir, "docs", "ARCHITECTURE.md"), "see `src/x.rs:6`\n");
    const run = (...args) =>
      execFileSync("node", [join(dir, "tools", "check-doc-refs.mjs"), ...args], {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      });
    run("--accept");
    writeFileSync(join(dir, "src", "x.rs"), ["// moved", ...code].join("\n"));
    assert.throws(() => run(), /moved to src\/x\.rs:7/);
    assert.throws(() => run("--fix"), (e) => e.status === 1);
    assert.equal(readFileSync(join(dir, "docs", "ARCHITECTURE.md"), "utf8"), "see `src/x.rs:7`\n");
    assert.match(run(), /doc refs ok: 1 references/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

// A throwaway repository with the checker in tools/ and `doc` as the map.
function sandbox(doc, lines) {
  const dir = mkdtempSync(join(tmpdir(), "doc-refs-"));
  for (const d of ["tools", "docs", "src"]) mkdirSync(join(dir, d));
  cpSync(join(here, "check-doc-refs.mjs"), join(dir, "tools", "check-doc-refs.mjs"));
  writeFileSync(join(dir, "src", "x.rs"), lines.join("\n"));
  writeFileSync(join(dir, "docs", "ARCHITECTURE.md"), doc);
  const run = (script, ...args) =>
    execFileSync("node", [script, ...args], { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
  return { dir, run: (...args) => run(join(dir, "tools", "check-doc-refs.mjs"), ...args), runAt: run };
}

test("--fix keeps each record when neighbouring references move together", () => {
  const { dir, run } = sandbox("`src/x.rs:2` and `src/x.rs:3`\n", ["a", "b", "c", "d"]);
  try {
    run("--accept");
    writeFileSync(join(dir, "src", "x.rs"), ["new", "a", "b", "c", "d"].join("\n"));
    assert.throws(() => run("--fix"), (e) => e.status === 1);
    assert.equal(readFileSync(join(dir, "docs", "ARCHITECTURE.md"), "utf8"), "`src/x.rs:3` and `src/x.rs:4`\n");
    assert.deepEqual(JSON.parse(readFileSync(join(dir, "docs", "ARCHITECTURE.refs.json"), "utf8")), {
      "src/x.rs:3": ["b"],
      "src/x.rs:4": ["c"],
    });
    assert.match(run(), /doc refs ok: 2 references/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("the check runs when it is called through a symbolic link", () => {
  const { dir, run, runAt } = sandbox("`src/x.rs:1`\n", ["a"]);
  const link = mkdtempSync(join(tmpdir(), "doc-refs-link-"));
  try {
    run("--accept");
    symlinkSync(realpathSync(dir), join(link, "repo"));
    assert.match(runAt(join(link, "repo", "tools", "check-doc-refs.mjs")), /doc refs ok: 1 references/);
  } finally {
    rmSync(link, { recursive: true, force: true });
    rmSync(dir, { recursive: true, force: true });
  }
});
