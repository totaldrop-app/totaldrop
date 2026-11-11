// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node tools/check-doc-refs.mjs            check
// node tools/check-doc-refs.mjs --fix      follow lines that only moved
// node tools/check-doc-refs.mjs --accept   record the lines as they are now
//
// docs/ARCHITECTURE.md points into the code as `path:line` or
// `path:first-last`, with the path from the repository root. A line number
// says nothing once the code above it changes, so next to the document sits
// docs/ARCHITECTURE.refs.json: for every reference, the text of the cited
// line (and of the last line of a range) as it was when someone checked the
// claim. This fails when a reference no longer lands on that text:
//
// - the text is still in the file, elsewhere: the code only moved, and
//   `--fix` rewrites the numbers in the document;
// - the text is gone or changed: the claim has to be read again against the
//   code. Correct the document, then `--accept` records the new lines.
//
// It also fails on a reference whose file does not exist (a short name such
// as `lib.rs:12` included) or whose lines are past the end of the file, and
// on a reference the record does not know.

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const DOC = "docs/ARCHITECTURE.md";
const RECORD = "docs/ARCHITECTURE.refs.json";

/** Every `path:line` and `path:first-last` in backticks, in order. */
export function findRefs(markdown) {
  const out = [];
  const re = /`([\w./-]+\.[A-Za-z]\w*):(\d+)(?:-(\d+))?`/g;
  for (const m of markdown.matchAll(re)) {
    const first = Number(m[2]);
    const last = m[3] ? Number(m[3]) : first;
    out.push({ token: m[0], path: m[1], first, last, key: `${m[1]}:${m[2]}${m[3] ? `-${m[3]}` : ""}` });
  }
  return out;
}

/** The text a reference is checked by: its first line, and its last line
 * when it is a range, without surrounding white space. */
export function snapshot(lines, first, last) {
  const at = (n) => lines[n - 1].trim();
  return first === last ? [at(first)] : [at(first), at(last)];
}

/** Where a recorded reference is now: the same numbers, a shifted pair when
 * its text sits uniquely elsewhere (the nearest match wins only if no other
 * is as near), or null when the text is gone. */
export function locate(lines, ref, recorded) {
  const same = (first, last) =>
    first >= 1 && last <= lines.length && snapshot(lines, first, last).join("\n") === recorded.join("\n");
  if (same(ref.first, ref.last)) return { first: ref.first, last: ref.last };
  const span = ref.last - ref.first;
  const hits = [];
  for (let n = 1; n + span <= lines.length; n++) if (same(n, n + span)) hits.push(n);
  if (!hits.length) return null;
  const dist = (n) => Math.abs(n - ref.first);
  hits.sort((a, b) => dist(a) - dist(b));
  if (hits.length > 1 && dist(hits[0]) === dist(hits[1])) return null;
  return { first: hits[0], last: hits[0] + span };
}

/** Checks `markdown` against `record` with `read(path)` giving a file's
 * lines or null. Returns the problems, the moves `--fix` would make, and
 * the record `--accept` would write. */
export function check(markdown, record, read) {
  const problems = [];
  const moves = new Map();
  const fresh = {};
  for (const ref of findRefs(markdown)) {
    const lines = read(ref.path);
    if (!lines) {
      problems.push(`${ref.key}: no such file; write the path from the repository root`);
      continue;
    }
    if (ref.first < 1 || ref.last < ref.first || ref.last > lines.length) {
      problems.push(`${ref.key}: the file has ${lines.length} lines`);
      continue;
    }
    fresh[ref.key] = snapshot(lines, ref.first, ref.last);
    const recorded = record[ref.key];
    if (!recorded) {
      problems.push(`${ref.key}: not in ${RECORD}; check the claim, then run with --accept`);
      continue;
    }
    if (moves.has(ref.key)) continue;
    const now = locate(lines, ref, recorded);
    if (!now) {
      problems.push(`${ref.key}: the cited line changed (was \`${recorded[0]}\`); read the claim again, correct it, then run with --accept`);
    } else if (now.first !== ref.first) {
      const key = `${ref.path}:${now.first}${ref.first === ref.last ? "" : `-${now.last}`}`;
      moves.set(ref.key, key);
      problems.push(`${ref.key}: moved to ${key}; run with --fix`);
    }
  }
  return { problems, moves, fresh };
}

/** `markdown` with every moved reference rewritten. */
export function applyMoves(markdown, moves) {
  return markdown.replace(/`([\w./-]+\.[A-Za-z]\w*:\d+(?:-\d+)?)`/g, (all, key) =>
    moves.has(key) ? `\`${moves.get(key)}\`` : all,
  );
}

function main() {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const mode = process.argv[2] ?? "";
  if (!["", "--fix", "--accept"].includes(mode)) {
    console.error("usage: node tools/check-doc-refs.mjs [--fix | --accept]");
    process.exit(2);
  }
  const read = (path) => {
    const full = join(root, path);
    return existsSync(full) ? readFileSync(full, "utf8").split("\n") : null;
  };
  const recordPath = join(root, RECORD);
  const docPath = join(root, DOC);
  let doc = readFileSync(docPath, "utf8");
  let record = existsSync(recordPath) ? JSON.parse(readFileSync(recordPath, "utf8")) : {};
  let result = check(doc, record, read);

  if (mode === "--fix" && result.moves.size) {
    for (const [from, to] of result.moves) {
      record[to] = record[from];
      delete record[from];
    }
    doc = applyMoves(doc, result.moves);
    writeFileSync(docPath, doc);
    writeFileSync(recordPath, `${JSON.stringify(sorted(record), null, 2)}\n`);
    console.error(`${DOC}: followed ${result.moves.size} moved reference(s)`);
    result = check(doc, record, read);
    if (!result.problems.length) process.exit(1); // changed files: look, then commit
  }
  if (mode === "--accept") {
    const unreadable = result.problems.filter((p) => /no such file|lines$/.test(p));
    if (!unreadable.length) {
      writeFileSync(recordPath, `${JSON.stringify(sorted(result.fresh), null, 2)}\n`);
      console.log(`${RECORD}: recorded ${Object.keys(result.fresh).length} references`);
      return;
    }
    result.problems = unreadable;
  }
  if (result.problems.length) {
    console.error(`${DOC} points at code that changed:`);
    for (const p of result.problems) console.error(`  ${p}`);
    process.exit(1);
  }
  const unused = Object.keys(record).filter((k) => !(k in result.fresh));
  if (unused.length) {
    console.error(`${RECORD} has references the document no longer uses; run with --accept:`);
    for (const k of unused) console.error(`  ${k}`);
    process.exit(1);
  }
  console.log(`doc refs ok: ${Object.keys(result.fresh).length} references in ${DOC}`);
}

function sorted(record) {
  return Object.fromEntries(Object.entries(record).sort(([a], [b]) => a.localeCompare(b)));
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main();
