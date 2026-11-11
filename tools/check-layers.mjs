// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node tools/check-layers.mjs
//
// The Rust crates depend on each other in one direction only
// (docs/ARCHITECTURE.md, "Katmanlar"): the protocol and cryptography
// libraries know nothing above them, the servers link only the wire format
// (never the cryptography or the client engine), and nothing depends on an
// app. Three things say this, and this script holds them equal:
//
// - the code: Cargo reads the manifests (`cargo metadata --no-deps`, which
//   builds nothing), so every way of writing a dependency counts;
// - the table below (ALLOWED, ALLOWED_DEV);
// - the layer diagram in docs/ARCHITECTURE.md (the Mermaid block that
//   starts with `%% layers`), solid arrows for the build and run time,
//   dotted ones for tests.
//
// A dependency the table does not allow, an allowed one the code no longer
// has, a crate missing from the table, or an arrow the diagram draws
// differently fails the check.

import { execFileSync } from "node:child_process";
import { readFileSync, realpathSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

/** Which workspace crates each crate uses, for its build and at run time:
 * no more (a new edge fails) and no fewer (a dropped one fails too). */
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

/** What tests use on top: the CLI's end-to-end tests run both servers,
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

/** Every way the code and the tables above disagree, as readable lines:
 * an edge the tables do not allow, and an allowed edge the code lacks. */
export function violations(crates) {
  const out = [];
  for (const { name, deps } of crates) {
    if (!Object.hasOwn(ALLOWED, name)) {
      out.push(`${name}: not in tools/check-layers.mjs; add it to ALLOWED`);
      continue;
    }
    const dev = ALLOWED_DEV[name] ?? [];
    for (const d of deps) {
      const ok = ALLOWED[name].includes(d.crate) || (d.dev && dev.includes(d.crate));
      if (!ok) out.push(`${name} -> ${d.crate}${d.dev ? " (dev)" : ""}: not allowed`);
    }
    for (const crate of ALLOWED[name]) {
      if (!deps.some((d) => d.crate === crate && !d.dev)) {
        out.push(`${name} -> ${crate}: in ALLOWED but not in the code; remove it from ALLOWED and the diagram`);
      }
    }
    for (const crate of dev) {
      if (!deps.some((d) => d.crate === crate && d.dev)) {
        out.push(`${name} -> ${crate} (dev): in ALLOWED_DEV but not in the code; remove it from ALLOWED_DEV and the diagram`);
      }
    }
  }
  return out;
}

/** The layer diagram: the Mermaid block in `markdown` whose first line is
 * `%% layers`. A node is a crate when the first word of its label is a name
 * in `crates`; other nodes (the Dart interface) and their arrows are left
 * out. Returns the crate nodes and the arrows between them, solid ones as
 * `dev: false`, dotted ones as `dev: true`, or `errors` for a line that
 * looks like an arrow but cannot be read. */
export function diagramEdges(markdown, crates) {
  const block = /```mermaid\n%% layers\n([\s\S]*?)```/.exec(markdown);
  if (!block) return { nodes: [], edges: [], errors: ["no ```mermaid block starting with %% layers"] };
  const known = new Set(crates);
  const crateOf = new Map();
  const errors = [];
  const lines = block[1].split("\n");
  for (const line of lines) {
    for (const m of line.matchAll(/\b(\w+)\["([^"<]*)/g)) {
      const word = m[2].trim().split(/\s+/)[0];
      if (known.has(word)) crateOf.set(m[1], word);
    }
  }
  const edges = [];
  const arrow = /^\s*(\w+)\s+(?:-->|--\s*"[^"]*"\s*-->|(-\.->|-\.\s*"[^"]*"\s*\.->))\s+(\w+)\s*$/;
  for (const line of lines) {
    if (!/--|\.->|==/.test(line)) continue;
    const m = arrow.exec(line);
    if (!m) {
      errors.push(`cannot read the arrow \`${line.trim()}\``);
      continue;
    }
    const [, from, dotted, to] = m;
    if (crateOf.has(from) && crateOf.has(to)) {
      edges.push({ from: crateOf.get(from), crate: crateOf.get(to), dev: Boolean(dotted) });
    }
  }
  return { nodes: [...new Set(crateOf.values())], edges, errors };
}

/** Every way the diagram and the tables above disagree, as readable lines. */
export function diagramViolations({ nodes, edges, errors }) {
  const out = errors.map((e) => `diagram: ${e}`);
  const key = (from, crate, dev) => `${from} -> ${crate}${dev ? " (dev)" : ""}`;
  const drawn = new Set(edges.map((e) => key(e.from, e.crate, e.dev)));
  const table = new Set();
  for (const [from, list] of Object.entries(ALLOWED)) for (const c of list) table.add(key(from, c, false));
  for (const [from, list] of Object.entries(ALLOWED_DEV)) for (const c of list) table.add(key(from, c, true));
  for (const name of Object.keys(ALLOWED)) {
    if (!nodes.includes(name)) out.push(`diagram: ${name} is not drawn`);
  }
  for (const e of drawn) if (!table.has(e)) out.push(`diagram: ${e} is drawn but not in the table`);
  for (const e of table) if (!drawn.has(e)) out.push(`diagram: ${e} is in the table but not drawn`);
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
  const doc = readFileSync(join(root, "docs", "ARCHITECTURE.md"), "utf8");
  const found = [
    ...violations(crates),
    ...diagramViolations(diagramEdges(doc, crates.map((c) => c.name))),
  ];
  if (found.length) {
    console.error('Crate layering broken (docs/ARCHITECTURE.md, "Katmanlar"):');
    for (const v of found) console.error(`  ${v}`);
    process.exit(1);
  }
  const names = crates.map((c) => c.name).sort();
  console.log(`layers ok: ${names.length} crates (${names.join(", ")}); code, table and diagram agree`);
}

// Both sides resolved: called through a symbolic link, argv[1] and the
// module URL differ and the check would silently not run.
if (process.argv[1] && realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url))) main();
