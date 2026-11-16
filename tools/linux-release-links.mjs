#!/usr/bin/env node
// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Updates or checks the Linux package download links in README.md, the 34
// translated docs/readme/README.<tag>.md files (35 languages total in
// site/public/lang.js), and site/public/download/index.html (36 files in all)
// from a release tag (`linux-vX.Y.Z[-pre]`) and the CI-produced SHA256SUMS
// file (#414, #392):
//
//   node tools/linux-release-links.mjs write <linux-v...> <SHA256SUMS>
//   node tools/linux-release-links.mjs check [<linux-v...> <SHA256SUMS>]
//
// Without arguments after `check` (CI), it checks that every translated
// README and site/public/download/index.html link the exact same tag,
// version and seven package file names as README.md, and that those seven
// names follow the release naming rules.
//
// Design note (#414): the system requirement text (`dl.req.linux` across the
// 35 site/public/i18n/*.json files and site/public/download/index.html) stays
// manual rather than derived from the RPM `.fcNN` build suffix, because it is
// translated editorial copy covering Qt, Debian, Ubuntu, Fedora and Arch
// together and only changes when a build base moves (#392), alongside the
// manual `dl.version`, `dl.date` and `dl.new.*` release notes.

import { createHash } from "node:crypto";
import {
  existsSync,
  readFileSync,
  readdirSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const DEFAULT_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const RELEASE_BASE = "https://github.com/totaldrop-app/totaldrop/releases/download/";
const LINUX_URL_RE =
  /https:\/\/github\.com\/totaldrop-app\/totaldrop\/releases\/download\/(linux[^\s)"'<>/]*)(?:\/([^\s)"'<>]*))?/g;
const ROW_LINK_RE =
  /\[([^\[\]\r\n]+)\]\((https:\/\/github\.com\/totaldrop-app\/totaldrop\/releases\/download\/linux[^\s()"'<>\[\]]*)\)/g;
const ATTEST_RE = /^gh attestation verify ([^\s]+) --repo totaldrop-app\/totaldrop$/gm;
const SEMVER_CORE = "(?:0|[1-9]\\d*)\\.(?:0|[1-9]\\d*)\\.(?:0|[1-9]\\d*)";
// RPM `Version:` forbids `-`, and packaging/check-versions.sh turns only the
// first `-` into `~`, so pre-release identifiers are dot-separated without `-`.
// `\d*[A-Za-z][0-9A-Za-z]*` avoids overlapping `*` quantifiers (O(N), no ReDoS).
const SEMVER_PRE_ID = "(?:0|[1-9]\\d*|\\d*[A-Za-z][0-9A-Za-z]*)";
const VERSION_RE = new RegExp(
  `^${SEMVER_CORE}(?:-${SEMVER_PRE_ID}(?:\\.${SEMVER_PRE_ID})*)?$`,
);
const NOTE_VERSION_RE = new RegExp(
  `^([ \\t]*)${SEMVER_CORE}(?:-${SEMVER_PRE_ID}(?:\\.${SEMVER_PRE_ID})*)?(?=[,،、，])`,
);
const HAS_UNSAFE_RE = /[\p{Cc}\p{Cf}\p{Cs}\p{Zl}\p{Zp}]/u;
const UNSAFE_CHAR_RE = /[\p{Cc}\p{Cf}\p{Cs}\p{Zl}\p{Zp}\\]/gu;
const UNSAFE_LINE_RE = /[\p{Cc}\p{Cf}\p{Cs}\p{Zl}\p{Zp}]/gu;

const SLOTS = [
  "archX64",
  "debX64",
  "rpmX64",
  "flatpakX64",
  "debArm64",
  "rpmArm64",
  "flatpakArm64",
];

const SLOT_LABELS = {
  archX64: "Arch / Omarchy",
  debX64: "Debian / Ubuntu",
  rpmX64: "Fedora",
  flatpakX64: "Flatpak",
  debArm64: "Debian / Ubuntu",
  rpmArm64: "Fedora",
  flatpakArm64: "Flatpak",
};

const SITE_LINK_KEYS = [
  ["dl.linux.arch", "archX64"],
  ["dl.linux.deb", "debX64"],
  ["dl.linux.rpm", "rpmX64"],
  ["dl.linux.flatpak", "flatpakX64"],
  ["dl.linux.deb.arm", "debArm64"],
  ["dl.linux.rpm.arm", "rpmArm64"],
  ["dl.linux.flatpak.arm", "flatpakArm64"],
];

function escapeRegExp(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function escapeCodePoint(ch) {
  if (ch === "\n") return "\\n";
  if (ch === "\r") return "\\r";
  if (ch === "\t") return "\\t";
  const cp = ch.codePointAt(0);
  return cp <= 0xffff
    ? `\\u${cp.toString(16).padStart(4, "0")}`
    : `\\u{${cp.toString(16)}}`;
}

// Make control, format/bidi, lone-surrogate, and line-separator characters
// visible and bound length by code point before including external text in
// terminal error messages (docs/surec.md, security rules).
export function safeText(value, maxLen = 160) {
  const chars = [...String(value)];
  const sliced =
    chars.length > maxLen ? `${chars.slice(0, maxLen).join("")}…` : chars.join("");
  return sliced.replace(UNSAFE_CHAR_RE, (ch) =>
    ch === "\\" ? "\\\\" : escapeCodePoint(ch),
  );
}

function safeTerminalMessage(value) {
  return String(value).replace(UNSAFE_LINE_RE, (ch) =>
    ch === "\n" ? "\n" : escapeCodePoint(ch),
  );
}

export function parseTag(tag) {
  if (typeof tag !== "string" || !tag.startsWith("linux-v")) {
    throw new Error(
      `invalid Linux release tag "${safeText(tag)}": must start with "linux-v"`,
    );
  }
  const version = tag.slice("linux-v".length);
  if (!VERSION_RE.test(version)) {
    throw new Error(
      `invalid version "${safeText(version)}" in tag "${safeText(tag)}"`,
    );
  }
  return version;
}

function slotMatchers(version) {
  const archVer = escapeRegExp(version.replaceAll("-", "_"));
  // Debian and RPM write pre-releases with `~`, which linux-release.yml
  // renames to `.` before writing SHA256SUMS so checksums match GitHub's
  // uploaded asset names.
  const debRpmVer = escapeRegExp(version.replace("-", "."));
  const flatpakVer = escapeRegExp(version);
  return {
    archX64: new RegExp(`^totaldrop-${archVer}-([1-9]\\d*)-x86_64\\.pkg\\.tar\\.zst$`),
    debX64: new RegExp(`^totaldrop_${debRpmVer}-([1-9]\\d*)_amd64\\.deb$`),
    rpmX64: new RegExp(`^totaldrop-${debRpmVer}-([1-9]\\d*\\.fc[1-9]\\d*)\\.x86_64\\.rpm$`),
    flatpakX64: new RegExp(`^totaldrop-${flatpakVer}-x86_64\\.flatpak$`),
    debArm64: new RegExp(`^totaldrop_${debRpmVer}-([1-9]\\d*)_arm64\\.deb$`),
    rpmArm64: new RegExp(`^totaldrop-${debRpmVer}-([1-9]\\d*\\.fc[1-9]\\d*)\\.aarch64\\.rpm$`),
    flatpakArm64: new RegExp(`^totaldrop-${flatpakVer}-aarch64\\.flatpak$`),
  };
}

export function parseSha256Sums(sumsText) {
  const names = [];
  const seen = new Set();
  const lines = sumsText.replace(/\r\n/g, "\n").split("\n");
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line === "") continue;
    const m = /^([0-9a-f]{64}) [ *]([^\s/\\]+)$/.exec(line);
    if (!m) {
      throw new Error(
        `SHA256SUMS:${i + 1}: malformed checksum line "${safeText(line)}"`,
      );
    }
    const name = m[2];
    if (HAS_UNSAFE_RE.test(name)) {
      throw new Error(
        `SHA256SUMS:${i + 1}: control or invisible character in "${safeText(name)}"`,
      );
    }
    if (seen.has(name)) {
      throw new Error(
        `SHA256SUMS:${i + 1}: duplicate package "${safeText(name)}"`,
      );
    }
    seen.add(name);
    names.push(name);
  }
  if (names.length === 0) {
    throw new Error("SHA256SUMS: no package entries found");
  }
  return names;
}

export function resolvePackages(tag, names) {
  const version = parseTag(tag);
  const matchers = slotMatchers(version);
  const packages = {};
  const extras = [];
  const suffixes = {};

  for (const name of names) {
    let matchedSlot = null;
    for (const slot of SLOTS) {
      const m = matchers[slot].exec(name);
      if (m) {
        if (packages[slot]) {
          throw new Error(
            `multiple packages match ${slot}: "${safeText(packages[slot])}" and "${safeText(name)}"`,
          );
        }
        packages[slot] = name;
        suffixes[slot] = m[1] ?? null;
        matchedSlot = slot;
        break;
      }
    }
    if (!matchedSlot) extras.push(safeText(name));
  }

  const missing = SLOTS.filter((s) => !packages[s]);
  if (missing.length || extras.length) {
    const parts = [];
    if (missing.length) parts.push(`missing package(s) for ${tag}: ${missing.join(", ")}`);
    if (extras.length) parts.push(`unexpected package(s) for ${tag}: ${extras.join(", ")}`);
    throw new Error(parts.join("; "));
  }
  if (suffixes.debX64 !== suffixes.debArm64) {
    throw new Error(
      `Debian revision mismatch between amd64 (${safeText(packages.debX64)}) and arm64 (${safeText(packages.debArm64)})`,
    );
  }
  if (suffixes.rpmX64 !== suffixes.rpmArm64) {
    throw new Error(
      `RPM release/distro mismatch between x86_64 (${safeText(packages.rpmX64)}) and aarch64 (${safeText(packages.rpmArm64)})`,
    );
  }
  return { tag, version, packages, nameSet: new Set(names) };
}

function assertLinksMatchPackageList(fileLabel, text, spec) {
  const urls = [...text.matchAll(LINUX_URL_RE)];
  if (urls.length !== SLOTS.length) {
    throw new Error(
      `${safeText(fileLabel)}: expected ${SLOTS.length} Linux release links, found ${urls.length}`,
    );
  }
  const seenNames = new Set();
  for (const [, linkTag, rawName] of urls) {
    const fileName = rawName ?? "";
    if (linkTag !== spec.tag) {
      throw new Error(
        `${safeText(fileLabel)}: link tag "${safeText(linkTag)}" does not match "${spec.tag}"`,
      );
    }
    if (!spec.nameSet.has(fileName)) {
      throw new Error(
        `${safeText(fileLabel)}: linked file "${safeText(fileName)}" is not in the package list`,
      );
    }
    if (seenNames.has(fileName)) {
      throw new Error(
        `${safeText(fileLabel)}: duplicate link to "${safeText(fileName)}"`,
      );
    }
    seenNames.add(fileName);
  }
}

function rewriteRowLinks(fileLabel, line, slotOrder, spec) {
  const cells = line.split("|");
  if (cells.length !== 5) {
    throw new Error(
      `${safeText(fileLabel)}: unexpected Linux table row layout: "${safeText(line)}"`,
    );
  }
  const allUrls = [...cells[2].matchAll(LINUX_URL_RE)];
  const labeledLinks = [...cells[2].matchAll(ROW_LINK_RE)];
  if (
    allUrls.length !== slotOrder.length ||
    labeledLinks.length !== slotOrder.length
  ) {
    throw new Error(
      `${safeText(fileLabel)}: expected ${slotOrder.length} labeled links in row, found ${labeledLinks.length} (${allUrls.length} URLs)`,
    );
  }
  let idx = 0;
  cells[2] = cells[2].replace(ROW_LINK_RE, (_full, label) => {
    const slot = slotOrder[idx++];
    const expectedLabel = SLOT_LABELS[slot];
    if (label !== expectedLabel) {
      throw new Error(
        `${safeText(fileLabel)}: expected link label "[${expectedLabel}]" for ${slot}, found "[${safeText(label)}]"`,
      );
    }
    const fileName = spec.packages[slot];
    if (!spec.nameSet.has(fileName)) {
      throw new Error(
        `${safeText(fileLabel)}: "${safeText(fileName)}" is not in the package list`,
      );
    }
    return `[${label}](${RELEASE_BASE}${spec.tag}/${fileName})`;
  });
  if (!NOTE_VERSION_RE.test(cells[3])) {
    throw new Error(
      `${safeText(fileLabel)}: Linux table notes cell does not start with a version: "${safeText(cells[3])}"`,
    );
  }
  cells[3] = cells[3].replace(NOTE_VERSION_RE, (_, ws) => `${ws}${spec.version}`);
  return cells.join("|");
}

export function rewriteReadme(text, spec, fileLabel = "README.md") {
  const lines = text.split("\n");
  const rowIndices = [];
  for (let i = 0; i < lines.length; i++) {
    if (lines[i].includes(RELEASE_BASE + "linux")) rowIndices.push(i);
  }
  if (rowIndices.length !== 2) {
    throw new Error(
      `${safeText(fileLabel)}: expected 2 Linux download rows, found ${rowIndices.length}`,
    );
  }
  lines[rowIndices[0]] = rewriteRowLinks(
    fileLabel,
    lines[rowIndices[0]],
    ["archX64", "debX64", "rpmX64", "flatpakX64"],
    spec,
  );
  lines[rowIndices[1]] = rewriteRowLinks(
    fileLabel,
    lines[rowIndices[1]],
    ["debArm64", "rpmArm64", "flatpakArm64"],
    spec,
  );

  let updated = lines.join("\n");
  const attestMatches = [...updated.matchAll(ATTEST_RE)];
  if (attestMatches.length !== 1) {
    throw new Error(
      `${safeText(fileLabel)}: expected 1 "gh attestation verify" command, found ${attestMatches.length}`,
    );
  }
  if (!spec.nameSet.has(spec.packages.debX64)) {
    throw new Error(
      `${safeText(fileLabel)}: "${safeText(spec.packages.debX64)}" is not in the package list`,
    );
  }
  updated = updated.replace(
    ATTEST_RE,
    () => `gh attestation verify ${spec.packages.debX64} --repo totaldrop-app/totaldrop`,
  );

  assertLinksMatchPackageList(fileLabel, updated, spec);
  return updated;
}

export function rewriteDownloadHtml(html, spec, fileLabel = "site/public/download/index.html") {
  let updated = html;
  const hrefRe =
    /(?<=\s)href="https:\/\/github\.com\/totaldrop-app\/totaldrop\/releases\/download\/linux[^\s"<>]*"/g;
  for (const [i18nKey, slot] of SITE_LINK_KEYS) {
    const fileName = spec.packages[slot];
    if (!spec.nameSet.has(fileName)) {
      throw new Error(
        `${safeText(fileLabel)}: "${safeText(fileName)}" is not in the package list`,
      );
    }
    const keyEsc = escapeRegExp(i18nKey);
    const tagRe = new RegExp(
      `<a(?=\\s)[^<>]*\\sdata-i18n="${keyEsc}"[^<>]*>`,
      "g",
    );
    const matches = [...updated.matchAll(tagRe)];
    if (matches.length !== 1) {
      throw new Error(
        `${safeText(fileLabel)}: expected 1 <a ... data-i18n="${i18nKey}"> element, found ${matches.length}`,
      );
    }
    updated = updated.replace(tagRe, (openTag) => {
      const hrefs = [...openTag.matchAll(hrefRe)];
      if (hrefs.length !== 1) {
        throw new Error(
          `${safeText(fileLabel)}: expected 1 Linux release href on data-i18n="${i18nKey}", found ${hrefs.length}`,
        );
      }
      return openTag.replace(
        hrefRe,
        () => `href="${RELEASE_BASE}${spec.tag}/${fileName}"`,
      );
    });
  }
  assertLinksMatchPackageList(fileLabel, updated, spec);
  return updated;
}

function siteReadmePaths(root) {
  const langJs = readFileSync(join(root, "site/public/lang.js"), "utf8");
  const box = { Intl };
  vm.runInNewContext(langJs, box);
  const languages = box.TDLang?.LANGUAGES;
  if (!Array.isArray(languages) || languages.length === 0) {
    throw new Error("site/public/lang.js: TDLang.LANGUAGES missing");
  }
  const expected = languages.map(([tag]) =>
    tag === "en" ? "README.md" : `docs/readme/README.${tag}.md`,
  );
  const readmeDir = join(root, "docs/readme");
  if (existsSync(readmeDir)) {
    const known = new Set(expected);
    for (const f of readdirSync(readmeDir).sort()) {
      if (!/^README\..+\.md$/.test(f)) continue;
      const rel = `docs/readme/${f}`;
      if (!known.has(rel)) {
        throw new Error(
          `${safeText(rel)}: unexpected README not listed in site/public/lang.js`,
        );
      }
    }
  }
  return expected;
}

function readmeStamp(text) {
  return createHash("sha256").update(text).digest("hex").slice(0, 12);
}

export function specFromReadme(readmeText, fileLabel = "README.md") {
  const urls = [...readmeText.matchAll(LINUX_URL_RE)];
  if (urls.length !== SLOTS.length) {
    throw new Error(
      `${safeText(fileLabel)}: expected ${SLOTS.length} Linux release links, found ${urls.length}`,
    );
  }
  const tags = new Set(urls.map((m) => m[1]));
  if (tags.size !== 1) {
    throw new Error(
      `${safeText(fileLabel)}: Linux links mix multiple tags: ${[...tags].map((t) => safeText(t)).join(", ")}`,
    );
  }
  const [tag] = tags;
  const names = urls.map((m) => m[2] ?? "");
  return resolvePackages(tag, names);
}

export function updateRepo(root, tag, sumsText, { write = false } = {}) {
  const names = parseSha256Sums(sumsText);
  const spec = resolvePackages(tag, names);
  return applySpec(root, spec, { write });
}

export function applySpec(root, spec, { write = false } = {}) {
  const paths = siteReadmePaths(root);
  const downloadRel = "site/public/download/index.html";
  for (const rel of [...paths, downloadRel]) {
    if (!existsSync(join(root, rel))) {
      throw new Error(`${safeText(rel)}: missing`);
    }
  }

  const oldEn = readFileSync(join(root, "README.md"), "utf8");
  const oldStamp = readmeStamp(oldEn);
  const newEn = rewriteReadme(oldEn, spec, "README.md");
  const newStamp = readmeStamp(newEn);

  const planned = new Map();
  planned.set("README.md", { before: oldEn, after: newEn });

  for (const rel of paths) {
    if (rel === "README.md") continue;
    const before = readFileSync(join(root, rel), "utf8");
    let after = rewriteReadme(before, spec, rel);
    // If a translation was in step with README.md before this link-only
    // update, keep its source stamp in step with the new README.md; leave
    // translations that were already behind untouched.
    if (newStamp !== oldStamp) {
      const m = /^<!-- Translated from README\.md ([0-9a-f]{12})\. -->\n/.exec(after);
      if (m && m[1] === oldStamp) {
        after =
          `<!-- Translated from README.md ${newStamp}. -->\n` +
          after.slice(m[0].length);
      }
    }
    planned.set(rel, { before, after });
  }

  const dlBefore = readFileSync(join(root, downloadRel), "utf8");
  const dlAfter = rewriteDownloadHtml(dlBefore, spec, downloadRel);
  planned.set(downloadRel, { before: dlBefore, after: dlAfter });

  const drifted = [];
  for (const [rel, { before, after }] of planned) {
    if (before !== after) {
      drifted.push(rel);
      if (write) writeFileSync(join(root, rel), after);
    }
  }
  return { spec, drifted, totalFiles: planned.size };
}

export function checkRepoConsistency(root) {
  const enPath = join(root, "README.md");
  if (!existsSync(enPath)) throw new Error("README.md: missing");
  const spec = specFromReadme(readFileSync(enPath, "utf8"), "README.md");
  return applySpec(root, spec, { write: false });
}

function usage() {
  return (
    "Usage:\n" +
    "  node tools/linux-release-links.mjs write <linux-vX.Y.Z[-pre]> <SHA256SUMS>\n" +
    "  node tools/linux-release-links.mjs check [<linux-vX.Y.Z[-pre]> <SHA256SUMS>]"
  );
}

function main() {
  const [cmd, tag, sumsPath, ...extra] = process.argv.slice(2);
  if (
    extra.length > 0 ||
    (cmd !== "write" && cmd !== "check") ||
    (cmd === "write" && (!tag || !sumsPath)) ||
    (cmd === "check" && Boolean(tag) !== Boolean(sumsPath))
  ) {
    console.error(usage());
    process.exit(2);
  }
  try {
    if (cmd === "write") {
      const sumsText = readFileSync(sumsPath, "utf8");
      const { spec, drifted, totalFiles } = updateRepo(
        DEFAULT_ROOT,
        tag,
        sumsText,
        { write: true },
      );
      console.log(
        `Updated ${drifted.length} of ${totalFiles} files for ${spec.tag}.`,
      );
    } else if (tag) {
      const sumsText = readFileSync(sumsPath, "utf8");
      const { spec, drifted, totalFiles } = updateRepo(
        DEFAULT_ROOT,
        tag,
        sumsText,
        { write: false },
      );
      if (drifted.length > 0) {
        throw new Error(
          `files out of date for ${spec.tag} (run "write"): ${drifted.join(", ")}`,
        );
      }
      console.log(
        `Linux release links ok: ${spec.tag} across ${totalFiles} files`,
      );
    } else {
      const { spec, drifted, totalFiles } = checkRepoConsistency(DEFAULT_ROOT);
      if (drifted.length > 0) {
        throw new Error(
          `Linux release links disagree with README.md (${spec.tag}): ${drifted.join(", ")}`,
        );
      }
      console.log(
        `Linux release links ok: ${spec.tag} across ${totalFiles} files`,
      );
    }
  } catch (err) {
    const raw = err instanceof Error ? err.message : String(err);
    console.error(safeTerminalMessage(raw));
    process.exit(1);
  }
}

// Resolve both sides through symlinks (macOS's /var -> /private/var is one)
// so invoking the CLI through a symlinked path never silently skips main().
if (
  import.meta.main ??
  (process.argv[1] !== undefined &&
    existsSync(process.argv[1]) &&
    realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url)))
) {
  main();
}
