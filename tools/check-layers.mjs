// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node tools/check-layers.mjs
//
// The Rust crates depend on each other in one direction only
// (docs/ARCHITECTURE.md, "Katmanlar"): the protocol and cryptography
// libraries know nothing above them, the servers link only the wire format
// (never the cryptography or the client engine), and nothing depends on an
// app. This reads each workspace member's Cargo.toml and fails when a path
// dependency on another workspace crate is not in the table below, so the
// diagram in the document cannot drift from the code unnoticed.

import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

/** Which workspace crates each crate may use, for its build and at run time. */
export const ALLOWED = {
  "totaldrop-proto": [],
  "totaldrop-crypto": [],
  "totaldrop-l10n": [],
  "totaldrop-core": ["totaldrop-crypto", "totaldrop-proto"],
  "totaldrop-rendezvous": ["totaldrop-proto"],
  "totaldrop-relay": ["totaldrop-proto"],
  "totaldrop-cli": ["totaldrop-core", "totaldrop-proto", "totaldrop-l10n"],
  "totaldrop-linux": ["totaldrop-core", "totaldrop-proto", "totaldrop-l10n"],
  rust_lib_totaldrop: ["totaldrop-core", "totaldrop-proto", "totaldrop-l10n"],
};

/** What tests may use on top: the CLI's end-to-end tests run both servers,
 * and l10n checks that every language's share text gives its code back. */
export const ALLOWED_DEV = {
  "totaldrop-l10n": ["totaldrop-proto"],
  "totaldrop-cli": ["totaldrop-relay", "totaldrop-rendezvous"],
};

/** The package name and the workspace dependencies of one Cargo.toml.
 * Every `[…dependencies]` table counts, `target.….dependencies` included;
 * `dev` tells whether the table is `dev-dependencies`. Only names that are
 * workspace crates are kept. */
export function parseManifest(text, workspace) {
  let name = null;
  let table = "";
  const deps = [];
  for (const raw of text.split("\n")) {
    const line = raw.replace(/#.*$/, "").trim();
    const header = /^\[\[?([^\]]+)\]\]?$/.exec(line);
    if (header) {
      table = header[1].trim();
      continue;
    }
    const kv = /^("?)([A-Za-z0-9_-]+)\1\s*=\s*(.*)$/.exec(line);
    if (!kv) continue;
    const [, , key, value] = kv;
    if (table === "package" && key === "name") {
      name = JSON.parse(value);
      continue;
    }
    if (!/(^|\.)(dev-|build-)?dependencies$/.test(table)) continue;
    // `package = "…"` renames a dependency; the real crate name decides.
    const renamed = /package\s*=\s*"([^"]+)"/.exec(value);
    const crate = renamed ? renamed[1] : key;
    if (workspace.has(crate)) {
      deps.push({ crate, dev: /(^|\.)dev-dependencies$/.test(table) });
    }
  }
  return { name, deps };
}

/** Every edge that the tables above do not allow, as readable lines. */
export function violations(manifests) {
  const out = [];
  for (const { name, deps } of manifests) {
    if (!(name in ALLOWED)) {
      out.push(`${name}: not in tools/check-layers.mjs; add it to ALLOWED`);
      continue;
    }
    for (const { crate, dev } of deps) {
      const ok =
        ALLOWED[name].includes(crate) || (dev && (ALLOWED_DEV[name] ?? []).includes(crate));
      if (!ok) out.push(`${name} -> ${crate}${dev ? " (dev)" : ""}: not allowed`);
    }
  }
  return out;
}

/** The workspace members listed in the root Cargo.toml; a `dir/*` entry
 * stands for every directory under `dir` that has a Cargo.toml. */
export function members(root) {
  const text = readFileSync(join(root, "Cargo.toml"), "utf8");
  const list = /members\s*=\s*\[([^\]]*)\]/.exec(text);
  if (!list) throw new Error("no workspace members in Cargo.toml");
  const dirs = [];
  for (const m of list[1].matchAll(/"([^"]+)"/g)) {
    if (m[1].endsWith("/*")) {
      const base = m[1].slice(0, -2);
      for (const entry of readdirSync(join(root, base), { withFileTypes: true })) {
        if (entry.isDirectory() && existsSync(join(root, base, entry.name, "Cargo.toml"))) {
          dirs.push(join(base, entry.name));
        }
      }
    } else {
      dirs.push(m[1]);
    }
  }
  return dirs;
}

function main() {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const dirs = members(root);
  const texts = dirs.map((d) => readFileSync(join(root, d, "Cargo.toml"), "utf8"));
  const workspace = new Set(texts.map((t) => parseManifest(t, new Set()).name));
  const manifests = texts.map((t) => parseManifest(t, workspace));
  const found = violations(manifests);
  if (found.length) {
    console.error("Crate layering broken (docs/ARCHITECTURE.md, \"Katmanlar\"):");
    for (const v of found) console.error(`  ${v}`);
    process.exit(1);
  }
  console.log(`layers ok: ${manifests.length} crates`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main();
