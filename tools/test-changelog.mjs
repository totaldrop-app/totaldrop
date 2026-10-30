// Tests for tools/changelog.mjs: node --test tools/test-changelog.mjs
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
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
  today,
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

test("a single-platform release takes only its platform's entries (#152)", () => {
  const files = {
    "12-core.fixed.md": "- **Core:** Fixed in the shared code (#12).\n",
    "13-android.fixed.md": "- **Android:** Fixed on Android only (#13).\n",
    "14-both.changed.md":
      "- **Android, Linux, site:** Changed on both platforms, with a line\n" +
      "  that wraps (#14).\n" +
      "- **CLI:** A second entry in the same file (#14).\n",
    "15-cli.added.md": "- **Command line:** Added to the CLI, which ships with Linux (#15).\n",
    "16-quiet.none.md": "Only a test changes.\n",
  };
  const root = repo(files);
  try {
    const result = release(root, "[linux-0.4.0-beta.1]", "2026-11-02");
    assert.deepEqual(result, { count: 4, removed: 3 });
    const expected =
      HEADER +
      "## [linux-0.4.0-beta.1] - 2026-11-02\n\n" +
      "### Added\n\n- **Command line:** Added to the CLI, which ships with Linux (#15).\n\n" +
      "### Changed\n\n" +
      "- **Linux, site:** Changed on both platforms, with a line\n  that wraps (#14).\n" +
      "- **CLI:** A second entry in the same file (#14).\n\n" +
      "### Fixed\n\n- **Core:** Fixed in the shared code (#12).\n\n" +
      OLD;
    assert.equal(readFileSync(join(root, "CHANGELOG.md"), "utf8"), expected);
    // Android's entries wait for its own release; a spanning entry keeps
    // only the platform that has not shipped it yet.
    assert.deepEqual(readdirSync(join(root, "changelog.d")).sort(), ["13-android.fixed.md", "14-both.changed.md", "README.md"]);
    assert.equal(readFileSync(join(root, "changelog.d", "13-android.fixed.md"), "utf8"), files["13-android.fixed.md"]);
    assert.equal(
      readFileSync(join(root, "changelog.d", "14-both.changed.md"), "utf8"),
      "- **Android:** Changed on both platforms, with a line\n  that wraps (#14).\n",
    );
    assert.deepEqual(checkRepo(root), [], "the result passes the check");

    // Then Android's release takes what is left.
    assert.deepEqual(release(root, "[android-0.4.0-beta.1]", "2026-11-03"), { count: 2, removed: 2 });
    assert.deepEqual(readdirSync(join(root, "changelog.d")), ["README.md"]);
    assert.ok(
      readFileSync(join(root, "CHANGELOG.md"), "utf8").startsWith(
        HEADER +
          "## [android-0.4.0-beta.1] - 2026-11-03\n\n" +
          "### Changed\n\n- **Android:** Changed on both platforms, with a line\n  that wraps (#14).\n\n" +
          "### Fixed\n\n- **Android:** Fixed on Android only (#13).\n\n" +
          "## [linux-0.4.0-beta.1] - 2026-11-02\n",
      ),
    );
  } finally {
    rmSync(root, { recursive: true });
  }
});

test("a release that is not for one known platform takes every entry", () => {
  for (const heading of ["[android-0.4.0-beta.1] and [linux-0.4.0-beta.1]", "[server-0.1.0-beta.1]", "[0.4.0-beta.1]"]) {
    const root = repo({
      "13-android.fixed.md": "- **Android:** Fixed on Android only (#13).\n",
      "14-linux.fixed.md": "- **Linux:** Fixed on Linux only (#14).\n",
    });
    try {
      assert.deepEqual(release(root, heading, "2026-11-02"), { count: 2, removed: 2 }, heading);
    } finally {
      rmSync(root, { recursive: true });
    }
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

// A throwaway git repository with history, for `require`.
function gitRepo(files) {
  const root = repo(files);
  const env = { ...process.env, GIT_CONFIG_GLOBAL: "/dev/null", GIT_CONFIG_NOSYSTEM: "1", GIT_DIR: undefined, GIT_WORK_TREE: undefined, GIT_INDEX_FILE: undefined };
  const git = (...args) => execFileSync("git", ["-c", "user.name=t", "-c", "user.email=t@t", ...args], { cwd: root, encoding: "utf8", env }).trim();
  git("init", "-q", "-b", "main");
  git("add", "-A");
  git("commit", "-qm", "base");
  const commit = (msg) => {
    git("add", "-A");
    git("commit", "-qm", msg);
    return git("rev-parse", "HEAD");
  };
  return { root, git, commit };
}

test("require on branched history counts only the branch's own files", () => {
  const { root, git, commit } = gitRepo({});
  try {
    git("switch", "-q", "-c", "topic");
    writeFileSync(join(root, "changelog.d", "20-topic.fixed.md"), ENTRY.replace("#12", "#20"));
    commit("topic");
    git("switch", "-q", "main");
    writeFileSync(join(root, "changelog.d", "21-main.fixed.md"), ENTRY.replace("#12", "#21"));
    const mainTip = commit("main moves on");
    // A branch with no file of its own, made from the older base.
    git("switch", "-q", "-c", "empty", "HEAD~1");
    writeFileSync(join(root, "code.txt"), "code\n");
    commit("code only");
    assert.deepEqual(recordsChange(root, mainTip, "HEAD").ok, false, "a file added on main does not count for the branch");
    // The pull request merge commit, as CI checks it: first parent is main.
    git("switch", "-q", "main");
    git("merge", "-q", "--no-ff", "-m", "merge topic", "topic");
    assert.deepEqual(addedFragments(root, "HEAD^1", "HEAD"), ["changelog.d/20-topic.fixed.md"]);
  } finally {
    rmSync(root, { recursive: true });
  }
});

test("require ignores dotfiles, bad names, renames and old-section edits", () => {
  const { root, git, commit } = gitRepo({ "12-old.fixed.md": ENTRY });
  try {
    const base = git("rev-parse", "HEAD");
    writeFileSync(join(root, "changelog.d", ".gitkeep"), "");
    writeFileSync(join(root, "changelog.d", "notes.md"), "x\n");
    commit("dotfile and bad name");
    assert.deepEqual(recordsChange(root, base, "HEAD").added, [], "only entry files count");
    git("mv", "changelog.d/12-old.fixed.md", "changelog.d/12-old.changed.md");
    commit("rename");
    assert.equal(recordsChange(root, base, "HEAD").ok, false, "renaming a file is not adding one");
    const changelog = readFileSync(join(root, "CHANGELOG.md"), "utf8");
    writeFileSync(join(root, "CHANGELOG.md"), changelog.replace("Old (#1).", "Older (#1)."));
    commit("fix an old section");
    assert.equal(recordsChange(root, base, "HEAD").ok, false, "editing an old release alone needs a .none.md");
  } finally {
    rmSync(root, { recursive: true });
  }
});

test("check flags dotfiles, directories, symbolic links and CRLF", () => {
  const root = repo({ ".gitkeep": "", "12-a.fixed.md": ENTRY.replace("\n", "\r\n") });
  mkdirSync(join(root, "changelog.d", "sub"));
  symlinkSync("missing.md", join(root, "changelog.d", "13-gone.fixed.md"));
  try {
    const errors = checkRepo(root);
    has(errors, ".gitkeep: name must be");
    has(errors, "sub: changelog.d/ holds only plain entry files, no directories");
    has(errors, "13-gone.fixed.md: changelog.d/ holds only plain entry files, no symbolic links");
    has(errors, "Windows line endings");
    assert.throws(() => checkChangelog((HEADER + OLD).replaceAll("\n", "\r\n")), /Windows line endings/);
  } finally {
    rmSync(root, { recursive: true });
  }
});

test("the width limit is exactly 76 columns", () => {
  const line = (n) => "- **Core:** " + "x".repeat(n - 19) + " (#12).\n";
  assert.equal([...line(76).trimEnd()].length, 76);
  assert.deepEqual(one("12-x.fixed.md", line(76)), []);
  has(one("12-x.fixed.md", line(77)), "longer than 76");
});

test("dates", () => {
  assert.equal(today(new Date(2026, 0, 5, 1, 30)), "2026-01-05");
  // 01:30 in Istanbul is still the day before in UTC: the default must be
  // the local date. Run in a child with a fixed time zone, so this holds
  // on a UTC CI runner too.
  const probe = `import { today } from ${JSON.stringify(new URL("./changelog.mjs", import.meta.url).href)};
    console.log(today(new Date("2026-01-04T22:30:00Z")));`;
  const local = spawnSync(process.execPath, ["--input-type=module", "-e", probe], {
    encoding: "utf8",
    env: { ...process.env, TZ: "Europe/Istanbul" },
  });
  assert.equal(local.stdout.trim(), "2026-01-05", local.stderr);
  const root = repo({ "12-a.fixed.md": ENTRY });
  try {
    for (const good of ["2024-02-29", "2000-02-29"]) {
      const copy = repo({ "12-a.fixed.md": ENTRY });
      try {
        release(copy, "[linux-0.4.0-beta.1]", good);
      } finally {
        rmSync(copy, { recursive: true });
      }
    }
    for (const bad of ["2026-02-29", "1900-02-29", "2026-02-30", "2026-04-31", "2026-13-01", "2026-00-10", "2026-01-00", "2026-1-05"]) {
      assert.throws(() => release(root, "[linux-0.4.0-beta.1]", bad), /real date/, bad);
    }
  } finally {
    rmSync(root, { recursive: true });
  }
});

test("the command line: exit codes, messages, and a symbolic link", () => {
  const tool = fileURLToPath(new URL("./changelog.mjs", import.meta.url));
  const run = (path, ...args) => spawnSync(process.execPath, [path, ...args], { encoding: "utf8" });
  const usage = run(tool, "nonsense");
  assert.equal(usage.status, 2);
  assert.match(usage.stderr, /usage:/);
  const ok = run(tool, "check");
  assert.equal(ok.status, 0, ok.stderr);
  assert.match(ok.stdout, /changelog ok/);
  const dir = mkdtempSync(join(tmpdir(), "changelog-link-"));
  try {
    symlinkSync(dirname(tool), join(dir, "tools"));
    const linked = run(join(dir, "tools", "changelog.mjs"), "check");
    assert.equal(linked.status, 0, linked.stderr);
    assert.match(linked.stdout, /changelog ok/, "runs, not a silent no-op");
  } finally {
    rmSync(dir, { recursive: true });
  }
});
