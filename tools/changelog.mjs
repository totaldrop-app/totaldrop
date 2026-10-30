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
//       needed), or are a release: change CHANGELOG.md and delete entry
//       files from changelog.d/.
//   node tools/changelog.mjs release "<heading>" [YYYY-MM-DD]
//       Moves the entries into a new "## <heading> - <date>" section under
//       Unreleased, in Keep a Changelog order, and deletes the files. Within
//       a section, entries are ordered by issue number, then file name.
//       A release of one platform ("[linux-…]" or "[android-…]") takes only
//       the entries that name it or no platform; an entry that names both
//       stays behind under the other platform's area for its release.
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
import { lstatSync, readFileSync, readdirSync, realpathSync, unlinkSync, writeFileSync } from "node:fs";
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
  if (text.includes("\r")) return [`${name}: Windows line endings (CRLF); save the file with LF only`];
  if (!text.endsWith("\n") || text.endsWith("\n\n")) at(0, "must end with exactly one newline");
  const lines = text.replace(/\n$/, "").split("\n");
  lines.forEach((line, i) => {
    if (/\s$/.test(line)) at(i + 1, "trailing whitespace");
    if ([...line].length > WIDTH) at(i + 1, `longer than ${WIDTH} columns`);
    if (line.includes("\t")) at(i + 1, "tab character");
  });
  if (parsed.section === "none") {
    if (lines.join("").trim() === "") at(1, "say why this change needs no entry");
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
  if (changelog.includes("\r")) throw new Error("CHANGELOG.md has Windows line endings (CRLF); save it with LF only");
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
    .filter((n) => n !== "README.md")
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
    const kind = lstatSync(join(dir, name));
    if (!kind.isFile()) {
      const what = kind.isSymbolicLink() ? "symbolic links" : "directories or other special files";
      errors.push(`${name}: changelog.d/ holds only plain entry files, no ${what}`);
      continue;
    }
    errors.push(...checkText(name, readFileSync(join(dir, name), "utf8")));
  }
  errors.push(...checkChangelog(readFileSync(join(root, "CHANGELOG.md"), "utf8")));
  return errors;
}

/** The platforms a release heading names, e.g. ["linux"] for "[linux-0.4.0-beta.1]". */
export function headingPlatforms(heading) {
  return [...heading.matchAll(/\[([a-z]+)-[^\]]*\]/g)].map((m) => m[1]);
}

/**
 * The platform an entry's area belongs to, or null for an area that is not
 * a platform of its own (Core, Site, CI, Build…). The command line ships
 * with the Linux packages (docs/versioning.md).
 */
export function areaPlatform(area) {
  if (/\bandroid\b/i.test(area)) return "android";
  if (/\b(linux|cli|command line)\b/i.test(area)) return "linux";
  return null;
}

/** The entries of an entry file: each a list of lines, the first "- **Area:** …". */
function entriesOf(text) {
  const entries = [];
  for (const line of text.replace(/\n$/, "").split("\n")) {
    if (line.startsWith("- ")) entries.push([line]);
    else entries.at(-1).push(line);
  }
  return entries;
}

/**
 * Splits one entry for a release of `platforms`. A release for one platform
 * takes an entry that names that platform or no platform at all, under the
 * areas that are not another platform's; an entry that also names another
 * platform stays in its file under that platform's areas only, for that
 * platform's release.
 */
function splitEntry(lines, platforms) {
  const m = /^- \*\*([^*]+):\*\*/.exec(lines[0]);
  const areas = m[1].split(/,\s*/);
  const kinds = areas.map(areaPlatform);
  const named = kinds.filter((k) => k !== null);
  const known = named.length === 0 || named.some((k) => platforms.includes(k));
  if (!known) return { take: null, keep: lines };
  const later = areas.filter((_, i) => kinds[i] !== null && !platforms.includes(kinds[i]));
  if (later.length === 0) return { take: lines, keep: null };
  const now = areas.filter((_, i) => kinds[i] === null || platforms.includes(kinds[i]));
  const withAreas = (list) => [lines[0].replace(m[0], `- **${list.join(", ")}:**`), ...lines.slice(1)];
  return { take: withAreas(now), keep: withAreas(later) };
}

/**
 * The new release section built from the files, in Keep a Changelog order.
 * A heading for exactly one known platform ("[linux-…]") takes only what
 * that platform ships (splitEntry); any other heading takes every entry.
 */
export function buildSection(root, heading, date) {
  const dir = join(root, "changelog.d");
  const named = headingPlatforms(heading);
  const single = named.length === 1 && ["android", "linux"].includes(named[0]);
  const bySection = new Map(SECTIONS.map((s) => [s, []]));
  const used = [];
  const rewritten = new Map();
  for (const name of fragments(dir)) {
    const { section } = parseName(name);
    if (section === "none") {
      used.push(name);
      continue;
    }
    const taken = [];
    const kept = [];
    for (const lines of entriesOf(readFileSync(join(dir, name), "utf8"))) {
      const { take, keep } = single ? splitEntry(lines, named) : { take: lines, keep: null };
      if (take) taken.push(take.join("\n"));
      if (keep) kept.push(keep.join("\n"));
    }
    bySection.get(section).push(...taken);
    if (kept.length === 0) used.push(name);
    else if (taken.length > 0) rewritten.set(name, kept.join("\n") + "\n");
  }
  const parts = [`## ${heading} - ${date}`];
  for (const s of SECTIONS) {
    const entries = bySection.get(s);
    if (entries.length) parts.push(`### ${s[0].toUpperCase()}${s.slice(1)}`, entries.join("\n"));
  }
  return { text: parts.join("\n\n") + "\n", used, rewritten, count: [...bySection.values()].flat().length };
}

function isDate(text) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(text)) return false;
  const d = new Date(`${text}T00:00:00Z`);
  return !Number.isNaN(d.getTime()) && d.toISOString().slice(0, 10) === text;
}

/** Today in local time, as YYYY-MM-DD. */
export function today(now = new Date()) {
  const pad = (n) => String(n).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

/** Moves every entry into a new section and deletes the files. */
export function release(root, heading, date) {
  const errors = checkRepo(root);
  if (errors.length) throw new Error(errors.join("\n"));
  if (!/^\[[^\]]+\]( and \[[^\]]+\])*$/.test(heading)) {
    throw new Error(`heading must look like "[linux-0.4.0-beta.1]" or "[android-…] and [linux-…]", not "${heading}"`);
  }
  if (!isDate(date)) throw new Error(`date must be a real date as YYYY-MM-DD, not "${date}"`);
  const { text, used, rewritten, count } = buildSection(root, heading, date);
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
  for (const [name, rest] of rewritten) writeFileSync(join(root, "changelog.d", name), rest);
  return { count, removed: used.length };
}

// Renames are detected, so moving or renaming an existing file is not
// counted as adding one.
function changedPaths(root, base, head, filter, path) {
  const out = execFileSync(
    "git",
    ["-c", "core.quotePath=false", "diff", "--find-renames", "--name-only", `--diff-filter=${filter}`, `${base}...${head}`, "--", path],
    { cwd: root, encoding: "utf8" },
  );
  return out.split("\n").filter(Boolean);
}

/** Entry files (valid names only) added to changelog.d/ between two commits. */
export function addedFragments(root, base, head) {
  return changedPaths(root, base, head, "A", "changelog.d/").filter((p) => {
    const name = p.slice("changelog.d/".length);
    return !name.includes("/") && !parseName(name).error;
  });
}

/**
 * Whether a branch records its change: it adds an entry file to
 * changelog.d/, or it is a release, which changes CHANGELOG.md and deletes
 * entry files from changelog.d/. A correction to an old release section
 * adds a .none.md file like any other change.
 */
export function recordsChange(root, base, head) {
  const added = addedFragments(root, base, head);
  const released =
    changedPaths(root, base, head, "M", "CHANGELOG.md").length > 0 &&
    changedPaths(root, base, head, "D", "changelog.d/").some((p) => !parseName(p.slice("changelog.d/".length)).error);
  return { ok: added.length > 0 || released, added, released };
}

export function main(argv) {
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
    console.log(added.length ? `changelog files added: ${added.join(", ")}` : "a release: entries moved into CHANGELOG.md");
    return 0;
  }
  if (cmd === "release" && (args.length === 1 || args.length === 2)) {
    const date = args[1] ?? today();
    const { count, removed } = release(root, args[0], date);
    console.log(`moved ${count} entries into "## ${args[0]} - ${date}" and removed ${removed} files from changelog.d/`);
    return 0;
  }
  console.error('usage: changelog.mjs check | require <base> <head> | release "<heading>" [YYYY-MM-DD]');
  return 2;
}

// Run as a script, also when called through a symbolic link.
const isScript =
  import.meta.main ??
  (process.argv[1] !== undefined && realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url)));
if (isScript) {
  try {
    process.exitCode = main(process.argv.slice(2));
  } catch (e) {
    console.error(e.message);
    process.exitCode = 1;
  }
}
