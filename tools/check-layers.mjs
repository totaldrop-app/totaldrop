// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node tools/check-layers.mjs
//
// The Rust crates depend on each other in one direction only
// (docs/ARCHITECTURE.md, "Katmanlar"): the protocol and cryptography
// libraries know nothing above them, the servers link only the wire format
// (never the cryptography or the client engine), and nothing depends on an
// app. This asks Cargo for the workspace (`cargo metadata --no-deps`, which
// reads the manifests only and builds nothing) and fails when a dependency
// on another workspace crate is not in the table below, so the diagram in
// the document cannot drift from the code unnoticed. Cargo, not a parser of
// our own, reads the manifests, so every way of writing a dependency counts.

import { execFileSync } from "node:child_process";
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

/** The workspace crates and their dependencies on each other, from the
 * output of `cargo metadata --no-deps --format-version 1`. A dependency
 * counts whatever its kind (normal, build, dev) and target; only dev ones
 * are marked `dev`. A renamed dependency counts by its real name. */
export function workspaceEdges(metadata) {
  const members = new Set(metadata.workspace_members);
  const packages = metadata.packages.filter((p) => members.has(p.id));
  const names = new Set(packages.map((p) => p.name));
  return packages.map((p) => ({
    name: p.name,
    deps: p.dependencies
      .filter((d) => names.has(d.name))
      .map((d) => ({ crate: d.name, dev: d.kind === "dev" })),
  }));
}

/** Every edge that the tables above do not allow, as readable lines. */
export function violations(crates) {
  const out = [];
  for (const { name, deps } of crates) {
    if (!Object.hasOwn(ALLOWED, name)) {
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

/** The workspace as Cargo reads it, from the repository at `root`. */
export function readWorkspace(root) {
  const json = execFileSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--offline"],
    { cwd: root, encoding: "utf8", maxBuffer: 64 << 20 },
  );
  return workspaceEdges(JSON.parse(json));
}

function main() {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const crates = readWorkspace(root);
  const found = violations(crates);
  if (found.length) {
    console.error('Crate layering broken (docs/ARCHITECTURE.md, "Katmanlar"):');
    for (const v of found) console.error(`  ${v}`);
    process.exit(1);
  }
  const names = crates.map((c) => c.name).sort();
  console.log(`layers ok: ${names.length} crates (${names.join(", ")})`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main();
