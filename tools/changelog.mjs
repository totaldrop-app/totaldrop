#!/usr/bin/env node
// Changelog entries live in changelog.d/, one file per change, and are moved
// into CHANGELOG.md when a release is made (docs/surec.md). Pull requests
// never edit the same lines, so they do not conflict with each other.
//
//   node tools/changelog.mjs check
//       Every file in changelog.d/ follows the rules below, and the
//       Unreleased section of CHANGELOG.md holds only its pointer.
//   node tools/changelog.mjs require <base> <head>
//       The commits between base and head add at least one file to
//       changelog.d/ (an entry, or a .none.md file that says why none is
//       needed), or change CHANGELOG.md, which only a release does.
//   node tools/changelog.mjs release "<heading>" [YYYY-MM-DD]
//       Moves every entry into a new "## <heading> - <date>" section under
//       Unreleased, in Keep a Changelog order, and deletes the files. Within
//       a section, entries are ordered by issue number, then file name.
//
// A file is named <issue>-<slug>.<section>.md, where <section> is added,
// changed, deprecated, removed, fixed, security, or none. It holds one or
// more entries exactly as they will appear:
//
//   - **Area:** What changed and why it matters to a user, wrapped at 76
//     columns, continuation lines indented by two spaces (#123).
//
// Every entry starts with a bold area, ends with its issue references in
// parentheses and a full stop, and names the file's issue. A .none.md file
// holds one sentence saying why the change needs no entry.

import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, unlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

export const SECTIONS = ["added", "changed", "deprecated", "removed", "fixed", "security"];
export const WIDTH = 76;
export const POINTER =
  "Entries for the next release are in [changelog.d/](changelog.d/), one file\n" +
  "per change, and move here when it is released (docs/surec.md).";

const NAME = /^([1-9][0-9]*)-([a-z0-9]+(?:-[a-z0-9]+)*)\.([a-z]+)\.md$/;
const UNRELEASED = "## [Unreleased]";

/** Splits a file name into its issue, slug and section, or returns an error. */
export function parseName(name) {
  const m = NAME.exec(name);
  if (!m) return { error: `${name}: name must be <issue>-<slug>.<section>.md (e.g. 129-cli-yes.fixed.md)` };
  const [, issue, slug, section] = m;
  if (section !== "none" && !SECTIONS.includes(section)) {
    return { error: `${name}: unknown section "${section}" (${[...SECTIONS, "none"].join(", ")})` };
  }
  return { issue: Number(issue), slug, section };
}

/** Problems with one file's text; an empty list means it is fine. */
export function checkText(name, text) {
  const parsed = parseName(name);
  if (parsed.error) return [parsed.error];
  const errors = [];
  const at = (n, msg) => errors.push(`${name}:${n}: ${msg}`);
  if (!text.endsWith("\n") || text.endsWith("\n\n")) at(0, "must end with exactly one newline");
  const lines = text.replace(/\n$/, "").split("\n");
  lines.forEach((line, i) => {
    if (/\s$/.test(line)) at(i + 1, "trailing whitespace");
    if ([...line].length > WIDTH) at(i + 1, `longer than ${WIDTH} columns`);
    if (line.includes("\t")) at(i + 1, "tab character");
  });
  if (parsed.section === "none") {
    if (lines.length === 0 || lines.join("").trim() === "") at(1, "say why this change needs no entry");
    else if (lines.some((l) => l.startsWith("- "))) at(1, "a .none.md file holds a reason, not an entry");
    return errors;
  }
  const entries = [];
  lines.forEach((line, i) => {
    if (line.startsWith("- ")) entries.push({ start: i + 1, lines: [line] });
    else if (line === "") at(i + 1, "no blank lines; entries follow each other directly");
    else if (!/^ {2}\S/.test(line)) at(i + 1, 'continuation lines start with exactly two spaces; an entry starts with "- "');
    else if (entries.length === 0) at(i + 1, 'the file must start with an entry ("- **Area:** …")');
    else entries.at(-1).lines.push(line);
  });
  if (entries.length === 0) at(1, 'no entry ("- **Area:** … (#n).")');
  for (const e of entries) {
    const body = e.lines.map((l) => l.trim()).join(" ");
    if (!/^- \*\*[^*]+:\*\* \S/.test(body)) at(e.start, 'an entry starts with a bold area: "- **Area:** …"');
    const refs = /\(([^()]*#[0-9][^()]*)\)\.$/.exec(body);
    if (!refs) at(e.start, 'an entry ends with its issue references and a full stop: "… (#123)."');
    else if (!new RegExp(`#${parsed.issue}(?![0-9])`).test(refs[1])) {
      at(e.start, `the entry's references do not name the file's issue #${parsed.issue}`);
    }
  }
  return errors;
}

/** The Unreleased section of a CHANGELOG.md text: [start, end) offsets. */
function unreleased(changelog) {
  const start = changelog.indexOf(`${UNRELEASED}\n`);
  if (start < 0) throw new Error(`CHANGELOG.md has no "${UNRELEASED}" heading`);
  const next = changelog.indexOf("\n## ", start + UNRELEASED.length);
  return [start, next < 0 ? changelog.length : next + 1];
}

/** Problems with CHANGELOG.md itself: Unreleased must hold only the pointer. */
export function checkChangelog(changelog) {
  const [start, end] = unreleased(changelog);
  const body = changelog.slice(start + UNRELEASED.length, end).trim();
  if (body !== POINTER) {
    return [
      "CHANGELOG.md: the Unreleased section must hold only the pointer to changelog.d/; " +
        "put new entries in a file there instead (tools/changelog.mjs)",
    ];
  }
  return [];
}

function fragments(dir) {
  return readdirSync(dir)
    .filter((n) => n !== "README.md" && !n.startsWith("."))
    .sort((a, b) => {
      const pa = parseName(a);
      const pb = parseName(b);
      return (pa.issue ?? 0) - (pb.issue ?? 0) || a.localeCompare(b);
    });
}

/** Every problem in a repository checkout. */
export function checkRepo(root) {
  const dir = join(root, "changelog.d");
  const errors = [];
  for (const name of fragments(dir)) {
    errors.push(...checkText(name, readFileSync(join(dir, name), "utf8")));
  }
  errors.push(...checkChangelog(readFileSync(join(root, "CHANGELOG.md"), "utf8")));
  return errors;
}

/** The new release section built from the files, in Keep a Changelog order. */
export function buildSection(root, heading, date) {
  const dir = join(root, "changelog.d");
  const bySection = new Map(SECTIONS.map((s) => [s, []]));
  const used = [];
  for (const name of fragments(dir)) {
    const { section } = parseName(name);
    used.push(name);
    if (section === "none") continue;
    bySection.get(section).push(readFileSync(join(dir, name), "utf8").replace(/\n$/, ""));
  }
  const parts = [`## ${heading} - ${date}`];
  for (const s of SECTIONS) {
    const entries = bySection.get(s);
    if (entries.length) parts.push(`### ${s[0].toUpperCase()}${s.slice(1)}`, entries.join("\n"));
  }
  return { text: parts.join("\n\n") + "\n", used, count: [...bySection.values()].flat().length };
}

/** Moves every entry into a new section and deletes the files. */
export function release(root, heading, date) {
  const errors = checkRepo(root);
  if (errors.length) throw new Error(errors.join("\n"));
  if (!/^\[[^\]]+\]( and \[[^\]]+\])*$/.test(heading)) {
    throw new Error(`heading must look like "[linux-0.4.0-beta.1]" or "[android-…] and [linux-…]", not "${heading}"`);
  }
  if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) throw new Error(`date must be YYYY-MM-DD, not "${date}"`);
  const { text, used, count } = buildSection(root, heading, date);
  if (count === 0) throw new Error("changelog.d/ holds no entries to release");
  const path = join(root, "CHANGELOG.md");
  const changelog = readFileSync(path, "utf8");
  const [, end] = unreleased(changelog);
  // The Unreleased section keeps its pointer; the new section follows it,
  // one blank line on each side, above the previous release.
  const head = changelog.slice(0, end).replace(/\n+$/, "\n\n");
  const rest = changelog.slice(end);
  writeFileSync(path, rest ? `${head}${text}\n${rest}` : `${head}${text}`);
  for (const name of used) unlinkSync(join(root, "changelog.d", name));
  return { count, removed: used.length };
}

function changedPaths(root, base, head, filter, path) {
  const out = execFileSync(
    "git",
    ["-c", "core.quotePath=false", "diff", "--no-renames", "--name-only", `--diff-filter=${filter}`, `${base}...${head}`, "--", path],
    { cwd: root, encoding: "utf8" },
  );
  return out.split("\n").filter(Boolean);
}

/** Files added to changelog.d/ between two commits. */
export function addedFragments(root, base, head) {
  return changedPaths(root, base, head, "A", "changelog.d/").filter((p) => p !== "changelog.d/README.md");
}

/**
 * Whether a branch records its change: it adds a file to changelog.d/, or it
 * changes CHANGELOG.md, which `check` allows only outside Unreleased, so only
 * a release (or a correction to an old one) does it.
 */
export function recordsChange(root, base, head) {
  const added = addedFragments(root, base, head);
  const released = changedPaths(root, base, head, "AM", "CHANGELOG.md").length > 0;
  return { ok: added.length > 0 || released, added, released };
}

function main(argv) {
  const root = fileURLToPath(new URL("..", import.meta.url));
  const [cmd, ...args] = argv;
  if (cmd === "check" && args.length === 0) {
    const errors = checkRepo(root);
    if (errors.length) {
      for (const e of errors) console.error(e);
      return 1;
    }
    console.log(`changelog ok: ${fragments(join(root, "changelog.d")).length} files in changelog.d/`);
    return 0;
  }
  if (cmd === "require" && args.length === 2) {
    const { ok, added, released } = recordsChange(root, args[0], args[1]);
    if (!ok) {
      console.error(
        "This pull request adds no file to changelog.d/. Add the entry as " +
          "changelog.d/<issue>-<slug>.<section>.md, or, if the change needs none, " +
          "changelog.d/<issue>-<slug>.none.md with one sentence saying why (docs/surec.md).",
      );
      return 1;
    }
    console.log(added.length ? `changelog files added: ${added.join(", ")}` : "CHANGELOG.md changed (a release)");
    return 0;
  }
  if (cmd === "release" && (args.length === 1 || args.length === 2)) {
    const date = args[1] ?? new Date().toISOString().slice(0, 10);
    const { count, removed } = release(root, args[0], date);
    console.log(`moved ${count} entries into "## ${args[0]} - ${date}" and removed ${removed} files from changelog.d/`);
    return 0;
  }
  console.error('usage: changelog.mjs check | require <base> <head> | release "<heading>" [YYYY-MM-DD]');
  return 2;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  try {
    process.exitCode = main(process.argv.slice(2));
  } catch (e) {
    console.error(e.message);
    process.exitCode = 1;
  }
}
