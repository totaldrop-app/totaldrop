// Tests for tools/changelog.mjs: node --test tools/test-changelog.mjs
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  POINTER,
  addedFragments,
  recordsChange,
  checkChangelog,
  checkRepo,
  checkText,
  parseName,
  release,
} from "./changelog.mjs";

const ENTRY = "- **Core:** A short entry that says what changed for the user (#12).\n";
const HEADER = "# Changelog\n\nIntro.\n\n## [Unreleased]\n\n" + POINTER + "\n\n";
const OLD = "## [linux-0.3.0-beta.1] - 2026-10-30\n\n### Added\n\n- **Linux:** Old (#1).\n";

function repo(files, changelog = HEADER + OLD) {
  const root = mkdtempSync(join(tmpdir(), "changelog-"));
  mkdirSync(join(root, "changelog.d"));
  writeFileSync(join(root, "changelog.d", "README.md"), "How to write an entry.\n");
  writeFileSync(join(root, "CHANGELOG.md"), changelog);
  for (const [name, text] of Object.entries(files)) writeFileSync(join(root, "changelog.d", name), text);
  return root;
}

const one = (name, text) => checkText(name, text);
const has = (errors, part) => assert.ok(errors.some((e) => e.includes(part)), `expected "${part}" in ${JSON.stringify(errors)}`);

test("file names", () => {
  assert.deepEqual(parseName("129-cli-yes.fixed.md"), { issue: 129, slug: "cli-yes", section: "fixed" });
  assert.equal(parseName("7-x.none.md").section, "none");
  for (const bad of ["cli-yes.fixed.md", "0-x.fixed.md", "12-Upper.fixed.md", "12-x.fixed.txt", "12--x.fixed.md", "12-x-.fixed.md", "12.fixed.md"]) {
    assert.ok(parseName(bad).error, bad);
  }
  has([parseName("12-x.bugfix.md").error], 'unknown section "bugfix"');
});

test("a good entry, and several in one file", () => {
  assert.deepEqual(one("12-x.fixed.md", ENTRY), []);
  const two =
    "- **Android, Linux:** The first entry wraps onto a second line, which is\n" +
    "  indented by two spaces (#12, #40).\n" +
    "- **CLI (tests):** A second entry in the same file (#12).\n";
  assert.deepEqual(one("12-x.changed.md", two), []);
});

test("entry shape", () => {
  has(one("12-x.fixed.md", "- Core: no bold area (#12).\n"), "bold area");
  has(one("12-x.fixed.md", "- **Core:** no references.\n"), "issue references");
  has(one("12-x.fixed.md", "- **Core:** no full stop (#12)\n"), "issue references");
  has(one("12-x.fixed.md", "- **Core:** names another issue (#120).\n"), "#12");
  has(one("12-x.fixed.md", "Some prose first.\n" + ENTRY), "continuation lines");
  has(one("12-x.fixed.md", "  - **Core:** indented start (#12).\n"), "must start with an entry");
  has(one("12-x.fixed.md", "- **Core:** wrapped\n   with three spaces (#12).\n"), "two spaces");
  has(one("12-x.fixed.md", ENTRY + "\n" + ENTRY), "no blank lines");
  has(one("12-x.fixed.md", ""), "no entry");
});

test("layout", () => {
  has(one("12-x.fixed.md", "- **Core:** " + "x".repeat(70) + " (#12).\n"), "longer than 76");
  assert.deepEqual(one("12-x.fixed.md", "- **Çekirdek:** " + "ş".repeat(53) + " (#12).\n"), [], "width counts characters, not bytes");
  has(one("12-x.fixed.md", ENTRY.replace(".\n", ". \n")), "trailing whitespace");
  has(one("12-x.fixed.md", ENTRY.replace(" A", "\tA")), "tab");
  has(one("12-x.fixed.md", ENTRY.trimEnd()), "exactly one newline");
  has(one("12-x.fixed.md", ENTRY + "\n"), "exactly one newline");
});

test("a change that needs no entry says why", () => {
  assert.deepEqual(one("12-x.none.md", "Only a comment in a test changes.\n"), []);
  has(one("12-x.none.md", "\n"), "say why");
  has(one("12-x.none.md", " \n"), "say why");
  has(one("12-x.none.md", ENTRY), "a reason, not an entry");
});

test("the Unreleased section holds only the pointer", () => {
  assert.deepEqual(checkChangelog(HEADER + OLD), []);
  assert.deepEqual(checkChangelog(HEADER), [], "also with no release yet");
  has(checkChangelog(HEADER.replace(POINTER, POINTER + "\n\n### Fixed\n\n" + ENTRY.trimEnd()) + OLD), "only the pointer");
  assert.throws(() => checkChangelog("# Changelog\n"), /Unreleased/);
});

test("check reports every bad file in the repository", () => {
  const root = repo({ "12-a.fixed.md": ENTRY, "13-b.fixed.md": "nonsense\n", "bad name.md": ENTRY });
  try {
    const errors = checkRepo(root);
    has(errors, "13-b.fixed.md");
    has(errors, "bad name.md");
    assert.ok(!errors.some((e) => e.includes("12-a")));
    assert.ok(!errors.some((e) => e.includes("README")));
  } finally {
    rmSync(root, { recursive: true });
  }
});

test("release moves the entries in Keep a Changelog order and removes the files", () => {
  const root = repo({
    "40-later.fixed.md": "- **Linux:** Fixed later issue (#40).\n",
    "12-early.fixed.md": "- **Core:** Fixed early issue (#12).\n",
    "30-new.added.md": "- **Site:** Added thing (#30).\n",
    "31-quiet.none.md": "Only a test changes.\n",
    "33-safe.security.md": "- **Core:** Security fix (#33).\n",
  });
  try {
    const result = release(root, "[android-0.4.0-beta.1] and [linux-0.4.0-beta.1]", "2026-11-02");
    assert.deepEqual(result, { count: 4, removed: 5 });
    assert.deepEqual(readdirSync(join(root, "changelog.d")), ["README.md"]);
    const expected =
      HEADER +
      "## [android-0.4.0-beta.1] and [linux-0.4.0-beta.1] - 2026-11-02\n\n" +
      "### Added\n\n- **Site:** Added thing (#30).\n\n" +
      "### Fixed\n\n- **Core:** Fixed early issue (#12).\n- **Linux:** Fixed later issue (#40).\n\n" +
      "### Security\n\n- **Core:** Security fix (#33).\n\n" +
      OLD;
    assert.equal(readFileSync(join(root, "CHANGELOG.md"), "utf8"), expected);
    assert.deepEqual(checkRepo(root), [], "the result passes the check");
  } finally {
    rmSync(root, { recursive: true });
  }
});

test("release into a changelog with no earlier release", () => {
  const root = repo({ "12-a.changed.md": ENTRY }, HEADER);
  try {
    release(root, "[linux-0.1.0-beta.1]", "2026-11-02");
    assert.equal(
      readFileSync(join(root, "CHANGELOG.md"), "utf8"),
      HEADER + "## [linux-0.1.0-beta.1] - 2026-11-02\n\n### Changed\n\n" + ENTRY,
    );
  } finally {
    rmSync(root, { recursive: true });
  }
});

test("release refuses bad input and changes nothing", () => {
  const cases = [
    [{ "12-a.fixed.md": "nonsense\n" }, "[linux-0.4.0-beta.1]", "2026-11-02", /12-a\.fixed\.md/],
    [{ "12-a.fixed.md": ENTRY }, "linux-0.4.0-beta.1", "2026-11-02", /heading/],
    [{ "12-a.fixed.md": ENTRY }, "[linux-0.4.0-beta.1]", "02.11.2026", /date/],
    [{ "12-a.none.md": "Only a test.\n" }, "[linux-0.4.0-beta.1]", "2026-11-02", /no entries/],
  ];
  for (const [files, heading, date, message] of cases) {
    const root = repo(files);
    try {
      assert.throws(() => release(root, heading, date), message);
      assert.equal(readFileSync(join(root, "CHANGELOG.md"), "utf8"), HEADER + OLD);
      assert.deepEqual(readdirSync(join(root, "changelog.d")).sort(), ["README.md", ...Object.keys(files)].sort());
    } finally {
      rmSync(root, { recursive: true });
    }
  }
});

test("require sees only files a branch adds", () => {
  const root = repo({ "12-old.fixed.md": ENTRY });
  const git = (...args) =>
    execFileSync("git", args, {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, GIT_CONFIG_GLOBAL: "/dev/null", GIT_CONFIG_NOSYSTEM: "1", GIT_DIR: undefined, GIT_WORK_TREE: undefined, GIT_INDEX_FILE: undefined },
    }).trim();
  try {
    git("init", "-q", "-b", "main");
    git("-c", "user.name=t", "-c", "user.email=t@t", "add", "-A");
    git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "base");
    const base = git("rev-parse", "HEAD");
    writeFileSync(join(root, "changelog.d", "12-old.fixed.md"), ENTRY.replace("short", "changed"));
    writeFileSync(join(root, "changelog.d", "README.md"), "Changed.\n");
    git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qam", "edit only");
    assert.deepEqual(addedFragments(root, base, "HEAD"), [], "editing a file or the README does not count");
    writeFileSync(join(root, "changelog.d", "13-new.none.md"), "Only a test.\n");
    git("-c", "user.name=t", "-c", "user.email=t@t", "add", "-A");
    git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "add");
    assert.deepEqual(addedFragments(root, base, "HEAD"), ["changelog.d/13-new.none.md"]);
    assert.equal(recordsChange(root, base, "HEAD").ok, true);
    const added = git("rev-parse", "HEAD");
    assert.equal(recordsChange(root, added, "HEAD").ok, false, "a branch that adds nothing fails");
    release(root, "[linux-0.4.0-beta.1]", "2026-11-02");
    git("-c", "user.name=t", "-c", "user.email=t@t", "add", "-A");
    git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "release");
    assert.deepEqual(recordsChange(root, added, "HEAD"), { ok: true, added: [], released: true }, "a release passes");
  } finally {
    rmSync(root, { recursive: true });
  }
});
