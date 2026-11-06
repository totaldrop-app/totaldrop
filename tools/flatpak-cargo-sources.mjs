#!/usr/bin/env node
// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The Flatpak build runs without network (Flathub requires it), so every
// crate in Cargo.lock is a source of the manifest, fetched by flatpak-builder
// and checked against the lockfile's own checksum. This writes that list.
//
//   node tools/flatpak-cargo-sources.mjs write
//       Writes packaging/flatpak/cargo-sources.json from Cargo.lock.
//
// The file is not kept in the repository (.gitignore): CI writes it before
// every Flatpak build, so a Cargo.lock change (Dependabot's too) never
// leaves it behind. The Flathub repository gets a fresh copy with each
// release.
//
// Each crate becomes a .crate archive unpacked into cargo/vendor/<name>-<version>
// with the .cargo-checksum.json that cargo expects of a vendored crate, and
// cargo/config.toml points crates.io at that folder. The same shape that
// flatpak-cargo-generator (flatpak-builder-tools) writes, without its
// Python dependencies. Only crates.io crates are supported: a git or other
// source in Cargo.lock is an error, so it cannot be left out silently.

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const lockPath = join(root, "Cargo.lock");
const outPath = join(root, "packaging/flatpak/cargo-sources.json");
const crates = "registry+https://github.com/rust-lang/crates.io-index";

/** The [[package]] tables of a Cargo.lock: name, version, source, checksum. */
export function packages(lock) {
  const out = [];
  let cur = null;
  for (const raw of lock.split("\n")) {
    const line = raw.trim();
    if (line === "[[package]]") {
      cur = {};
      out.push(cur);
      continue;
    }
    if (line.startsWith("[")) {
      cur = null;
      continue;
    }
    const m = cur && /^(name|version|source|checksum) = "([^"]*)"$/.exec(line);
    if (m) cur[m[1]] = m[2];
  }
  return out;
}

/** The flatpak-builder sources for these packages. */
export function sources(pkgs) {
  const list = [];
  const seen = new Set();
  const remote = pkgs
    .filter((p) => p.source !== undefined)
    .sort((a, b) => (a.name + " " + a.version < b.name + " " + b.version ? -1 : 1));
  for (const p of remote) {
    if (p.source !== crates) {
      throw new Error(`${p.name} ${p.version}: only crates.io sources are supported, not ${p.source}`);
    }
    if (!/^[0-9a-f]{64}$/.test(p.checksum ?? "")) {
      throw new Error(`${p.name} ${p.version}: no sha256 checksum in Cargo.lock`);
    }
    if (!/^[A-Za-z0-9_-]+$/.test(p.name) || !/^[0-9A-Za-z.+-]+$/.test(p.version)) {
      throw new Error(`${p.name} ${p.version}: unexpected characters in name or version`);
    }
    const id = `${p.name}-${p.version}`;
    if (seen.has(id)) continue;
    seen.add(id);
    const dest = `cargo/vendor/${id}`;
    list.push({
      type: "archive",
      "archive-type": "tar-gzip",
      url: `https://static.crates.io/crates/${p.name}/${encodeURIComponent(id)}.crate`,
      sha256: p.checksum,
      dest,
    });
    list.push({
      type: "inline",
      contents: JSON.stringify({ package: p.checksum, files: {} }),
      dest,
      "dest-filename": ".cargo-checksum.json",
    });
  }
  list.push({
    type: "inline",
    contents:
      '[source.vendored-sources]\ndirectory = "cargo/vendor"\n\n' +
      '[source.crates-io]\nreplace-with = "vendored-sources"\n',
    dest: "cargo",
    "dest-filename": "config.toml",
  });
  return list;
}

function render() {
  return JSON.stringify(sources(packages(readFileSync(lockPath, "utf8"))), null, 4) + "\n";
}

const cmd = process.argv[2];
if (cmd === "write") {
  writeFileSync(outPath, render());
} else if (cmd !== undefined) {
  console.error("usage: flatpak-cargo-sources.mjs write");
  process.exit(2);
}
