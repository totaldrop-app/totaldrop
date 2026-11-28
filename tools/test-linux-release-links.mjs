// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test tools/test-linux-release-links.mjs

import { test } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  cpSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  checkRepoConsistency,
  parseSha256Sums,
  parseTag,
  resolvePackages,
  rewriteDownloadHtml,
  rewriteReadme,
  safeText,
  specFromReadme,
  updateRepo,
} from "./linux-release-links.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..");

// Synthetic sentinel versions so the test suite stays valid across all real
// future Linux and Android releases.
const SAMPLE_TAG = "linux-v99.88.77-beta.1";
const SAMPLE_VERSION = "99.88.77-beta.1";
const SAMPLE_PACKAGES = [
  "totaldrop-99.88.77-beta.1-aarch64.flatpak",
  "totaldrop-99.88.77-beta.1-x86_64.flatpak",
  "totaldrop-99.88.77.beta.1-1.fc99.aarch64.rpm",
  "totaldrop-99.88.77.beta.1-1.fc99.x86_64.rpm",
  "totaldrop-99.88.77_beta.1-1-x86_64.pkg.tar.zst",
  "totaldrop_99.88.77.beta.1-1_amd64.deb",
  "totaldrop_99.88.77.beta.1-1_arm64.deb",
];

function makeSums(names, { binary = false, crlf = false } = {}) {
  const sep = binary ? " *" : "  ";
  const eol = crlf ? "\r\n" : "\n";
  return (
    names
      .map((name, i) => {
        const hex = createHash("sha256")
          .update(`${i}:${name}`)
          .digest("hex");
        return `${hex}${sep}${name}`;
      })
      .join(eol) + eol
  );
}

function makeTempRepo(fn) {
  const outer = mkdtempSync(join(tmpdir(), "td-linux-links-"));
  const dir = join(outer, "repo");
  const symlinkedDir = join(outer, "via-symlink");
  mkdirSync(dir);
  symlinkSync(dir, symlinkedDir);
  try {
    // Copy every file the script or check-readmes.mjs reads/writes, and
    // symlink read-only repository targets so relative links in READMEs
    // resolve inside the sandbox.
    for (const entry of readdirSync(repoRoot)) {
      if (
        entry === ".git" ||
        entry === "target" ||
        entry === "node_modules" ||
        entry === "README.md" ||
        entry === "docs" ||
        entry === "site" ||
        entry === "tools"
      ) {
        continue;
      }
      symlinkSync(join(repoRoot, entry), join(dir, entry));
    }
    cpSync(join(repoRoot, "README.md"), join(dir, "README.md"));

    mkdirSync(join(dir, "docs"), { recursive: true });
    for (const entry of readdirSync(join(repoRoot, "docs"))) {
      if (entry === "readme") {
        cpSync(join(repoRoot, "docs/readme"), join(dir, "docs/readme"), {
          recursive: true,
        });
      } else {
        symlinkSync(join(repoRoot, "docs", entry), join(dir, "docs", entry));
      }
    }

    mkdirSync(join(dir, "site/public/download"), { recursive: true });
    mkdirSync(join(dir, "site/tools"), { recursive: true });
    cpSync(
      join(repoRoot, "site/public/lang.js"),
      join(dir, "site/public/lang.js"),
    );
    cpSync(
      join(repoRoot, "site/public/download/index.html"),
      join(dir, "site/public/download/index.html"),
    );
    cpSync(
      join(repoRoot, "site/tools/check-readmes.mjs"),
      join(dir, "site/tools/check-readmes.mjs"),
    );

    mkdirSync(join(dir, "tools"), { recursive: true });
    cpSync(
      join(repoRoot, "tools/linux-release-links.mjs"),
      join(dir, "tools/linux-release-links.mjs"),
    );

    return fn({ dir, symlinkedDir });
  } finally {
    rmSync(outer, { recursive: true, force: true });
  }
}

function runNode(args) {
  return spawnSync(process.execPath, args, { encoding: "utf8" });
}

test("checked-in READMEs and download page are consistent", () => {
  // site/public/lang.js lists 35 languages (README.md + 34 docs/readme/*.md),
  // plus site/public/download/index.html = 36 files in total.
  const { drifted, totalFiles } = checkRepoConsistency(repoRoot);
  assert.deepEqual(drifted, []);
  assert.equal(totalFiles, 36);

  const cli = runNode([join(here, "linux-release-links.mjs"), "check"]);
  assert.equal(cli.status, 0, cli.stderr);
  assert.match(cli.stdout, /across 36 files/);
});

test("writes all 35 READMEs and the download page from a sample SHA256SUMS", () => {
  makeTempRepo(({ dir }) => {
    const enBefore = readFileSync(join(dir, "README.md"), "utf8");
    const oldSpec = specFromReadme(enBefore);
    const oldLinuxRows = enBefore
      .split("\n")
      .filter((l) => l.includes("releases/download/linux"));
    assert.equal(oldLinuxRows.length, 2);
    const oldNotesTail = oldLinuxRows[0]
      .split("|")[3]
      .replace(new RegExp(`^\\s*${oldSpec.version.replaceAll(".", "\\.")}`), "");
    const androidRow = enBefore
      .split("\n")
      .find((l) => l.startsWith("| **Android** |"));
    assert.ok(androidRow, "README.md should have an Android row");

    const dlBefore = readFileSync(
      join(dir, "site/public/download/index.html"),
      "utf8",
    );
    const reqCellMatch = /<td data-i18n="dl\.req\.linux">[^<]*<\/td>/.exec(
      dlBefore,
    );
    assert.ok(reqCellMatch, "download/index.html should have dl.req.linux");

    // Mark one translation as already older than README.md before running the
    // link script: its stamp must stay untouched while up-to-date translations
    // advance with README.md.
    const frPath = join(dir, "docs/readme/README.fr.md");
    const frBefore = readFileSync(frPath, "utf8").replace(
      /^<!-- Translated from README\.md [0-9a-f]{12}\. -->/,
      "<!-- Translated from README.md 000000000000. -->",
    );
    writeFileSync(frPath, frBefore);

    const sumsText = makeSums(SAMPLE_PACKAGES);
    const result = updateRepo(dir, SAMPLE_TAG, sumsText, { write: true });
    assert.equal(result.totalFiles, 36);
    assert.equal(result.drifted.length, 36);

    const en = readFileSync(join(dir, "README.md"), "utf8");
    const expectedX64Row =
      "| **Linux** (x86-64) | " +
      `[Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/${SAMPLE_TAG}/totaldrop-99.88.77_beta.1-1-x86_64.pkg.tar.zst) · ` +
      `[Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/${SAMPLE_TAG}/totaldrop_99.88.77.beta.1-1_amd64.deb) · ` +
      `[Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/${SAMPLE_TAG}/totaldrop-99.88.77.beta.1-1.fc99.x86_64.rpm) · ` +
      `[Flatpak](https://github.com/totaldrop-app/totaldrop/releases/download/${SAMPLE_TAG}/totaldrop-99.88.77-beta.1-x86_64.flatpak) |` +
      ` ${SAMPLE_VERSION}${oldNotesTail}|`;
    const expectedArmRow =
      "| **Linux** (ARM64) | " +
      `[Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/${SAMPLE_TAG}/totaldrop_99.88.77.beta.1-1_arm64.deb) · ` +
      `[Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/${SAMPLE_TAG}/totaldrop-99.88.77.beta.1-1.fc99.aarch64.rpm) · ` +
      `[Flatpak](https://github.com/totaldrop-app/totaldrop/releases/download/${SAMPLE_TAG}/totaldrop-99.88.77-beta.1-aarch64.flatpak) |` +
      ` ${SAMPLE_VERSION}${oldNotesTail}|`;
    assert.ok(en.includes(expectedX64Row));
    assert.ok(en.includes(expectedArmRow));
    assert.ok(en.includes(androidRow));
    assert.ok(
      en.includes(
        "gh attestation verify totaldrop_99.88.77.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop",
      ),
    );
    assert.ok(!en.includes(oldSpec.tag));
    assert.ok(!en.includes(oldSpec.packages.rpmX64));

    // Localized punctuation in the Notes column (Arabic comma, Japanese
    // ideographic comma, full-width Chinese comma, ASCII comma) is preserved.
    assert.match(
      readFileSync(join(dir, "docs/readme/README.ar.md"), "utf8"),
      /\| 99\.88\.77-beta\.1، /,
    );
    assert.match(
      readFileSync(join(dir, "docs/readme/README.ja.md"), "utf8"),
      /\| 99\.88\.77-beta\.1、/,
    );
    assert.match(
      readFileSync(join(dir, "docs/readme/README.zh-Hans.md"), "utf8"),
      /\| 99\.88\.77-beta\.1，/,
    );
    assert.match(
      readFileSync(join(dir, "docs/readme/README.tr.md"), "utf8"),
      /\| 99\.88\.77-beta\.1, /,
    );

    // Every README has the 7 new links and updated attestation command.
    const enStamp = createHash("sha256").update(en).digest("hex").slice(0, 12);
    for (const f of readdirSync(join(dir, "docs/readme")).sort()) {
      const text = readFileSync(join(dir, "docs/readme", f), "utf8");
      for (const pkg of SAMPLE_PACKAGES) {
        assert.ok(
          text.includes(
            `https://github.com/totaldrop-app/totaldrop/releases/download/${SAMPLE_TAG}/${pkg}`,
          ),
          `${f}: missing ${pkg}`,
        );
      }
      assert.ok(
        text.includes(
          "gh attestation verify totaldrop_99.88.77.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop",
        ),
        `${f}: attestation command not updated`,
      );
      if (f === "README.fr.md") {
        assert.match(
          text,
          /^<!-- Translated from README\.md 000000000000\. -->\n/,
        );
      } else {
        assert.ok(
          text.startsWith(`<!-- Translated from README.md ${enStamp}. -->\n`),
          `${f}: stamp did not advance to ${enStamp}`,
        );
      }
    }

    // check-readmes.mjs passes on the updated tree and warns only about fr.
    const readmeCheck = runNode([join(dir, "site/tools/check-readmes.mjs")]);
    assert.equal(readmeCheck.status, 0, readmeCheck.stderr);
    assert.match(
      readmeCheck.stdout,
      /READMEs ok: 35 languages, 1 translations to update/,
    );

    // Download page links all 7 new packages; dl.req.linux stays manual (#414).
    const dl = readFileSync(join(dir, "site/public/download/index.html"), "utf8");
    for (const pkg of SAMPLE_PACKAGES) {
      assert.ok(
        dl.includes(
          `href="https://github.com/totaldrop-app/totaldrop/releases/download/${SAMPLE_TAG}/${pkg}"`,
        ),
        `download/index.html: missing ${pkg}`,
      );
    }
    assert.ok(!dl.includes(oldSpec.tag));
    assert.ok(dl.includes(reqCellMatch[0]));

    // Idempotent second run and repo-wide consistency check.
    const second = updateRepo(dir, SAMPLE_TAG, sumsText, { write: true });
    assert.deepEqual(second.drifted, []);
    assert.deepEqual(checkRepoConsistency(dir).drifted, []);
  });
});

test("supports stable tags, binary SHA256SUMS, CRLF, and HTML attribute edge cases", () => {
  const stableTag = "linux-v99.88.78";
  const stableNames = [
    "totaldrop-99.88.78-3-x86_64.pkg.tar.zst",
    "totaldrop_99.88.78-2_amd64.deb",
    "totaldrop-99.88.78-2.fc99.x86_64.rpm",
    "totaldrop-99.88.78-x86_64.flatpak",
    "totaldrop_99.88.78-2_arm64.deb",
    "totaldrop-99.88.78-2.fc99.aarch64.rpm",
    "totaldrop-99.88.78-aarch64.flatpak",
  ];
  const sums = makeSums(stableNames, { binary: true, crlf: true });
  const parsed = parseSha256Sums(sums);
  assert.deepEqual(parsed, stableNames);
  const spec = resolvePackages(stableTag, parsed);
  assert.equal(spec.version, "99.88.78");

  const enBefore = readFileSync(join(repoRoot, "README.md"), "utf8");
  const androidRow = enBefore
    .split("\n")
    .find((l) => l.startsWith("| **Android** |"));
  const enAfter = rewriteReadme(enBefore, spec);
  assert.match(enAfter, /\| 99\.88\.78, /);
  // Android's independent version row is untouched.
  assert.ok(androidRow && enAfter.includes(androidRow));
  assert.deepEqual(specFromReadme(enAfter).packages, spec.packages);

  // Reordered attributes and lookalike custom elements / hyphenated attributes
  // (`<a-custom>`, `data-href=`, `x-data-i18n=`) do not confuse HTML rewriting.
  const dlBefore =
    readFileSync(join(repoRoot, "site/public/download/index.html"), "utf8")
      .replace(
        /<a class="btn btn-primary" href="([^"]+)" ([^>]+ data-i18n="dl\.linux\.arch")>/,
        '<a class="btn btn-primary" data-href="https://example.invalid/ignore" $2 href="$1">',
      ) +
    '\n<a-custom data-i18n="dl.linux.arch" href="https://example.invalid/a"></a-custom>' +
    '\n<a x-data-i18n="dl.linux.arch" href="https://example.invalid/b">x</a>\n';
  const dlAfter = rewriteDownloadHtml(dlBefore, spec);
  assert.ok(
    dlAfter.includes(
      'data-i18n="dl.linux.arch" href="https://github.com/totaldrop-app/totaldrop/releases/download/linux-v99.88.78/totaldrop-99.88.78-3-x86_64.pkg.tar.zst"',
    ),
  );
  assert.ok(dlAfter.includes('data-href="https://example.invalid/ignore"'));
});

test("rejects missing, extra, duplicate, mismatched, swapped, or malformed inputs", () => {
  // Missing package (6 instead of 7).
  assert.throws(
    () => resolvePackages(SAMPLE_TAG, SAMPLE_PACKAGES.slice(1)),
    /missing package\(s\) for linux-v99\.88\.77-beta\.1: flatpakArm64/,
  );

  // Extra non-package file (such as the Sigstore bundle).
  assert.throws(
    () =>
      resolvePackages(SAMPLE_TAG, [
        ...SAMPLE_PACKAGES,
        "totaldrop-linux-99.88.77-beta.1.sigstore.json",
      ]),
    /unexpected package\(s\).*sigstore\.json/,
  );

  // Two packages matching the same slot (.fc98 and .fc99 together).
  assert.throws(
    () =>
      resolvePackages(SAMPLE_TAG, [
        ...SAMPLE_PACKAGES,
        "totaldrop-99.88.77.beta.1-1.fc98.x86_64.rpm",
      ]),
    /multiple packages match rpmX64/,
  );

  // Version in SHA256SUMS does not match tag.
  assert.throws(
    () => resolvePackages("linux-v99.88.77-beta.2", SAMPLE_PACKAGES),
    /missing package\(s\) for linux-v99\.88\.77-beta\.2/,
  );

  // Debian amd64 vs arm64 revision mismatch.
  assert.throws(
    () =>
      resolvePackages(
        SAMPLE_TAG,
        SAMPLE_PACKAGES.map((p) =>
          p.endsWith("_arm64.deb")
            ? "totaldrop_99.88.77.beta.1-2_arm64.deb"
            : p,
        ),
      ),
    /Debian revision mismatch/,
  );

  // RPM x86_64 vs aarch64 distro/release mismatch.
  assert.throws(
    () =>
      resolvePackages(
        SAMPLE_TAG,
        SAMPLE_PACKAGES.map((p) =>
          p.endsWith(".aarch64.rpm")
            ? "totaldrop-99.88.77.beta.1-1.fc98.aarch64.rpm"
            : p,
        ),
      ),
    /RPM release\/distro mismatch/,
  );

  // Duplicate entry, malformed checksum line, path traversal, empty file.
  const oneLine = makeSums([SAMPLE_PACKAGES[0]]).trimEnd();
  assert.throws(
    () => parseSha256Sums(`${oneLine}\n${oneLine}\n`),
    /duplicate package/,
  );
  assert.throws(() => parseSha256Sums("not-a-sha256  file.deb\n"), /malformed/);
  assert.throws(
    () => parseSha256Sums(`${"a".repeat(64)}  ../evil.deb\n`),
    /malformed/,
  );
  assert.throws(() => parseSha256Sums("\n\n"), /no package entries/);

  // Invalid release tags.
  for (const badTag of [
    "",
    "0.3.0-beta.7",
    "v0.3.0-beta.7",
    "android-v0.3.0-beta.7",
    "linux-v01.0.0",
    "linux-v0.3.0-beta.01",
    "linux-v0.3.0-rc-1",
  ]) {
    assert.throws(() => parseTag(badTag), /invalid/, badTag);
  }

  // Derive mutations from the current checked-in README so these negative
  // tests never depend on a specific live release number.
  const enCurrent = readFileSync(join(repoRoot, "README.md"), "utf8");
  const curSpec = specFromReadme(enCurrent);
  const spec = resolvePackages(SAMPLE_TAG, SAMPLE_PACKAGES);

  // Malformed pre-release suffix in Notes column does not backtrack onto X.Y.Z.
  const corruptNotes = enCurrent.replace(
    `| ${curSpec.version},`,
    `| ${curSpec.version}_bad,`,
  );
  assert.notEqual(corruptNotes, enCurrent);
  assert.throws(
    () => rewriteReadme(corruptNotes, spec),
    /notes cell does not start with a version/,
  );

  // Swapped Markdown link labels are rejected instead of overwritten by index.
  const swappedLabels = enCurrent
    .replace("[Debian / Ubuntu](", "[SWAP_TMP](")
    .replace("[Fedora](", "[Debian / Ubuntu](")
    .replace("[SWAP_TMP](", "[Fedora](");
  assert.notEqual(swappedLabels, enCurrent);
  assert.throws(
    () => rewriteReadme(swappedLabels, spec),
    /expected link label "\[Debian \/ Ubuntu\]" for debX64, found "\[Fedora\]"/,
  );

  // Extra path segment or typo in Linux release URL is caught in check mode.
  const extraSeg = enCurrent.replace(
    `${curSpec.packages.debX64})`,
    `${curSpec.packages.debX64}/extra)`,
  );
  assert.notEqual(extraSeg, enCurrent);
  assert.throws(() => specFromReadme(extraSeg), /unexpected package/);

  const badPrefix = enCurrent.replaceAll(
    curSpec.tag,
    curSpec.tag.replace("linux-v", "linux-"),
  );
  assert.notEqual(badPrefix, enCurrent);
  assert.throws(() => specFromReadme(badPrefix), /invalid Linux release tag/);
});

test("escapes control characters and stays linear on adversarial inputs", () => {
  // Control, bidi, lone-surrogate, and ESC sequences are escaped visibly, and
  // truncation slices on full code points rather than splitting astral pairs.
  const escaped = safeText(
    `bad\x1b[31mred\u202ertl\ud800\0\r\n${"🚀".repeat(180)}`,
  );
  assert.ok(!/[\p{Cc}\p{Cf}\p{Cs}]/u.test(escaped));
  assert.match(escaped, /\\u001b\[31mred\\u202ertl\\ud800\\u0000\\r\\n/);
  assert.ok(escaped.endsWith("🚀…"));

  assert.throws(
    () => parseSha256Sums("binary\x1b[2J\u202eheader\0"),
    (err) => {
      assert.ok(!/[\p{Cc}\p{Cf}\p{Cs}]/u.test(err.message));
      assert.match(err.message, /\\u001b\[2J\\u202eheader\\u0000/);
      return true;
    },
  );

  // Adversarial long inputs complete in linear time (no polynomial ReDoS).
  const spec = resolvePackages(SAMPLE_TAG, SAMPLE_PACKAGES);
  const unclosedRowLinks =
    "| a | " +
    "[Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux".repeat(
      5000,
    ) +
    " | 1.0.0, Qt |\n" +
    "| b | https://github.com/totaldrop-app/totaldrop/releases/download/linux-v1.0.0/x | 1.0.0, Qt |\n";
  const t0 = performance.now();
  assert.throws(
    () => parseTag(`linux-v0.3.0-${"a".repeat(30000)}!`),
    /invalid version/,
  );
  assert.throws(
    () => rewriteDownloadHtml('<a href="x" '.repeat(20000), spec),
    /expected 1 <a \.\.\. data-i18n=/,
  );
  assert.throws(
    () => rewriteReadme(unclosedRowLinks, spec),
    /expected 4 labeled links in row, found 0/,
  );
  const elapsedMs = performance.now() - t0;
  assert.ok(elapsedMs < 250, `regex took too long: ${elapsedMs}ms`);
});

test("CLI exit codes (0/1/2), symlinked invocation, and atomic failure", () => {
  makeTempRepo(({ dir, symlinkedDir }) => {
    // Invoke through `symlinkedDir` so the entry-point guard is exercised
    // when argv[1] and realpath differ (as on macOS /var -> /private/var).
    const script = join(symlinkedDir, "tools/linux-release-links.mjs");
    const sumsFile = join(symlinkedDir, "SHA256SUMS");
    writeFileSync(sumsFile, makeSums(SAMPLE_PACKAGES));

    // Usage errors exit with code 2.
    for (const badArgs of [
      [],
      ["unknown"],
      ["write"],
      ["write", SAMPLE_TAG],
      ["check", SAMPLE_TAG],
      ["check", SAMPLE_TAG, sumsFile, "extra"],
    ]) {
      const u = runNode([script, ...badArgs]);
      assert.equal(u.status, 2, `args=${JSON.stringify(badArgs)}`);
      assert.match(u.stderr, /^Usage:/);
    }

    // Binary/control-char file passed as SHA256SUMS never leaks raw ESC to stderr.
    const badSums = join(symlinkedDir, "BAD_SUMS");
    writeFileSync(badSums, "\x1b[31m\u202ebinary\0\n");
    const badSumsRun = runNode([script, "write", SAMPLE_TAG, badSums]);
    assert.equal(badSumsRun.status, 1);
    assert.ok(
      !/[\p{Cc}\p{Cf}]/u.test(badSumsRun.stderr.replaceAll("\n", "")),
    );
    assert.match(badSumsRun.stderr, /\\u001b\[31m\\u202ebinary\\u0000/);

    // Before `write`, `check <tag> <SHA256SUMS>` fails with exit code 1.
    const preCheck = runNode([script, "check", SAMPLE_TAG, sumsFile]);
    assert.equal(preCheck.status, 1);
    assert.match(preCheck.stderr, /files out of date for linux-v99\.88\.77-beta\.1/);

    // If one README is corrupt, `write` validates everything in memory first
    // and leaves README.md on disk untouched.
    const enOriginal = readFileSync(join(dir, "README.md"), "utf8");
    const trPath = join(dir, "docs/readme/README.tr.md");
    const trOriginal = readFileSync(trPath, "utf8");
    writeFileSync(
      trPath,
      trOriginal.replace("gh attestation verify ", "gh attestation broken "),
    );
    const failedWrite = runNode([script, "write", SAMPLE_TAG, sumsFile]);
    assert.equal(failedWrite.status, 1);
    assert.match(failedWrite.stderr, /README\.tr\.md.*gh attestation verify/);
    assert.equal(readFileSync(join(dir, "README.md"), "utf8"), enOriginal);

    // Restore README.tr.md and run `write`: exits 0 and updates all 36 files.
    writeFileSync(trPath, trOriginal);
    const okWrite = runNode([script, "write", SAMPLE_TAG, sumsFile]);
    assert.equal(okWrite.status, 0, okWrite.stderr);
    assert.match(
      okWrite.stdout,
      /Updated 36 of 36 files for linux-v99\.88\.77-beta\.1\./,
    );

    // Both `check <tag> <SHA256SUMS>` and bare `check` now exit 0 via symlink.
    assert.equal(runNode([script, "check", SAMPLE_TAG, sumsFile]).status, 0);
    assert.equal(runNode([script, "check"]).status, 0);

    // Swapping labels inside README.md itself is caught by bare `check`.
    const enUpdated = readFileSync(join(dir, "README.md"), "utf8");
    writeFileSync(
      join(dir, "README.md"),
      enUpdated
        .replace("[Arch / Omarchy](", "[Debian / Ubuntu](")
        .replace(/\[Debian \/ Ubuntu\]\(([^)]+_amd64\.deb)\)/, "[Arch / Omarchy]($1)"),
    );
    const labelCheck = runNode([script, "check"]);
    assert.equal(labelCheck.status, 1);
    assert.match(labelCheck.stderr, /expected link label "\[Arch \/ Omarchy\]"/);
    writeFileSync(join(dir, "README.md"), enUpdated);

    // Drift in site/public/download/index.html is caught by bare `check`.
    const dlPath = join(dir, "site/public/download/index.html");
    writeFileSync(
      dlPath,
      readFileSync(dlPath, "utf8").replace(
        "totaldrop-99.88.77.beta.1-1.fc99.x86_64.rpm",
        "totaldrop-99.88.77.beta.1-1.fc98.x86_64.rpm",
      ),
    );
    const driftCheck = runNode([script, "check"]);
    assert.equal(driftCheck.status, 1);
    assert.match(
      driftCheck.stderr,
      /disagree with README\.md.*site\/public\/download\/index\.html/,
    );
  });
});
