// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test tools/test-check-layers.mjs (needs cargo)

import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  ALLOWED,
  ALLOWED_DEV,
  diagramEdges,
  diagramViolations,
  readWorkspace,
  violations,
  workspaceEdges,
} from "./check-layers.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");

test("takes only workspace members, by real name, and marks dev ones", () => {
  const edges = workspaceEdges({
    workspace_members: ["core-id", "cli-id"],
    packages: [
      {
        id: "cli-id",
        name: "totaldrop-cli",
        dependencies: [
          { name: "tokio", kind: null },
          { name: "totaldrop-core", kind: null, rename: "engine" },
          { name: "totaldrop-core", kind: "build" },
          { name: "totaldrop-core", kind: "dev" },
          { name: "totaldrop-other", kind: null },
        ],
      },
      { id: "core-id", name: "totaldrop-core", dependencies: [] },
      // In the metadata but not a member: neither checked nor a target.
      { id: "other-id", name: "totaldrop-other", dependencies: [] },
    ],
  });
  assert.deepEqual(edges, [
    {
      name: "totaldrop-cli",
      deps: [
        { crate: "totaldrop-core", dev: false },
        { crate: "totaldrop-core", dev: false },
        { crate: "totaldrop-core", dev: true },
      ],
    },
    { name: "totaldrop-core", deps: [] },
  ]);
});

test("a lower layer that uses a higher one is reported", () => {
  const found = violations([
    { name: "totaldrop-crypto", deps: [{ crate: "totaldrop-core", dev: false }] },
  ]);
  assert.deepEqual(found, ["totaldrop-crypto -> totaldrop-core: not allowed"]);
});

test("a server that links the cryptography or the client engine is reported", () => {
  const found = violations([
    {
      name: "totaldrop-relay",
      deps: [
        { crate: "totaldrop-proto", dev: false },
        { crate: "totaldrop-crypto", dev: false },
        { crate: "totaldrop-core", dev: false },
      ],
    },
  ]);
  assert.deepEqual(found, [
    "totaldrop-relay -> totaldrop-crypto: not allowed",
    "totaldrop-relay -> totaldrop-core: not allowed",
  ]);
});

// The crates exactly as the table has them, to change one edge at a time.
const asTable = () =>
  Object.keys(ALLOWED).map((name) => ({
    name,
    deps: [
      ...ALLOWED[name].map((crate) => ({ crate, dev: false })),
      ...(ALLOWED_DEV[name] ?? []).map((crate) => ({ crate, dev: true })),
    ],
  }));
const withDeps = (name, deps) => asTable().map((c) => (c.name === name ? { name, deps } : c));

test("the table itself passes", () => {
  assert.deepEqual(violations(asTable()), []);
});

test("the servers are allowed in the CLI's tests, not in the CLI itself", () => {
  const base = ALLOWED["totaldrop-cli"].map((crate) => ({ crate, dev: false }));
  assert.deepEqual(
    violations(
      withDeps("totaldrop-cli", [
        ...base,
        { crate: "totaldrop-relay", dev: false },
        { crate: "totaldrop-rendezvous", dev: true },
      ]),
    ),
    [
      "totaldrop-cli -> totaldrop-relay: not allowed",
      "totaldrop-cli -> totaldrop-relay (dev): in ALLOWED_DEV but not in the code; remove it from ALLOWED_DEV and the diagram",
    ],
  );
});

test("l10n may use the code format in its tests only", () => {
  assert.deepEqual(
    violations(withDeps("totaldrop-l10n", [{ crate: "totaldrop-proto", dev: false }])),
    [
      "totaldrop-l10n -> totaldrop-proto: not allowed",
      "totaldrop-l10n -> totaldrop-proto (dev): in ALLOWED_DEV but not in the code; remove it from ALLOWED_DEV and the diagram",
    ],
  );
});

test("an allowed edge the code no longer has is reported, so the table cannot go stale", () => {
  assert.deepEqual(violations(withDeps("totaldrop-core", [{ crate: "totaldrop-proto", dev: false }])), [
    "totaldrop-core -> totaldrop-crypto: in ALLOWED but not in the code; remove it from ALLOWED and the diagram",
  ]);
});

test("a crate the table does not know is reported, not let through", () => {
  assert.deepEqual(violations([{ name: "totaldrop-new", deps: [] }]), [
    "totaldrop-new: not in tools/check-layers.mjs; add it to ALLOWED",
  ]);
  assert.deepEqual(violations([{ name: "constructor", deps: [] }]), [
    "constructor: not in tools/check-layers.mjs; add it to ALLOWED",
  ]);
});

test("this repository has exactly the crates in the table, and code, table and diagram agree", () => {
  const crates = readWorkspace(root);
  assert.deepEqual(crates.map((c) => c.name).sort(), Object.keys(ALLOWED).sort());
  assert.deepEqual(violations(crates), []);
  const doc = readFileSync(join(root, "docs", "ARCHITECTURE.md"), "utf8");
  assert.deepEqual(diagramViolations(diagramEdges(doc, Object.keys(ALLOWED))), []);
  const out = execFileSync("node", [join(here, "check-layers.mjs")], { encoding: "utf8" });
  assert.match(out, new RegExp(`^layers ok: ${Object.keys(ALLOWED).length} crates`, "m"));
});

const crateNames = Object.keys(ALLOWED);
const diagram = (body) => `text\n\n\`\`\`mermaid\n%% layers\nflowchart TB\n${body}\n\`\`\`\n`;

test("the diagram is read by the first word of each label, solid and dotted arrows apart", () => {
  const got = diagramEdges(
    diagram(
      [
        '  DART["Flutter arayüzü<br/>app/lib"]',
        '  CLI["totaldrop-cli<br/>crates/totaldrop-cli"]',
        '  RL["totaldrop-relay"]',
        '  P["totaldrop-proto (kablo)<br/>x"]',
        '  DART -- "flutter_rust_bridge" --> CLI',
        "  CLI --> P",
        '  CLI -. "test" .-> RL',
        "  RL -.-> P",
      ].join("\n"),
    ),
    crateNames,
  );
  assert.deepEqual(got, {
    nodes: ["totaldrop-cli", "totaldrop-relay", "totaldrop-proto"],
    edges: [
      { from: "totaldrop-cli", crate: "totaldrop-proto", dev: false },
      { from: "totaldrop-cli", crate: "totaldrop-relay", dev: true },
      { from: "totaldrop-relay", crate: "totaldrop-proto", dev: true },
    ],
    errors: [],
  });
});

test("a diagram that leaves out, adds or misdraws an arrow is reported", () => {
  const doc = readFileSync(join(root, "docs", "ARCHITECTURE.md"), "utf8");
  const without = doc.replace("  CORE --> CRYPTO\n", "");
  assert.notEqual(without, doc);
  assert.deepEqual(diagramViolations(diagramEdges(without, crateNames)), [
    "diagram: totaldrop-core -> totaldrop-crypto is in the table but not drawn",
  ]);
  const extra = doc.replace("  RLR --> PROTO\n", "  RLR --> PROTO\n  RLR --> CRYPTO\n");
  assert.deepEqual(diagramViolations(diagramEdges(extra, crateNames)), [
    "diagram: totaldrop-relay -> totaldrop-crypto is drawn but not in the table",
  ]);
  const solid = doc.replace('  CLI -. "test" .-> RLR\n', "  CLI --> RLR\n");
  assert.notEqual(solid, doc);
  assert.deepEqual(diagramViolations(diagramEdges(solid, crateNames)), [
    "diagram: totaldrop-cli -> totaldrop-relay is drawn but not in the table",
    "diagram: totaldrop-cli -> totaldrop-relay (dev) is in the table but not drawn",
  ]);
  const odd = doc.replace("  CORE --> CRYPTO\n", "  CORE ==> CRYPTO\n");
  assert.deepEqual(diagramViolations(diagramEdges(odd, crateNames)), [
    "diagram: cannot read the arrow `CORE ==> CRYPTO`",
    "diagram: totaldrop-core -> totaldrop-crypto is in the table but not drawn",
  ]);
  assert.deepEqual(diagramViolations(diagramEdges("no diagram", crateNames)).slice(0, 2), [
    "diagram: no ```mermaid block starting with %% layers",
    "diagram: totaldrop-proto is not drawn",
  ]);
});

// Every way of writing a dependency reaches the check, because Cargo reads
// the manifests: here a table header `[dependencies.<name>]`, a dotted key
// and a target table, in a throwaway workspace with the same crate names.
test("dependencies written in any Cargo form are caught", () => {
  const dir = mkdtempSync(join(tmpdir(), "layers-"));
  try {
    copyFileSync(join(root, "rust-toolchain.toml"), join(dir, "rust-toolchain.toml"));
    writeFileSync(
      join(dir, "Cargo.toml"),
      '[workspace]\nresolver = "3"\nmembers = ["proto", "crypto", "relay"]\n',
    );
    const crate = (name, path, extra) => {
      mkdirSync(join(dir, path, "src"), { recursive: true });
      writeFileSync(join(dir, path, "src", "lib.rs"), "");
      writeFileSync(
        join(dir, path, "Cargo.toml"),
        `[package]\nname = "${name}"\nversion = "0.0.0"\nedition = "2024"\n${extra}`,
      );
    };
    crate("totaldrop-proto", "proto", "");
    crate("totaldrop-crypto", "crypto", "");
    crate(
      "totaldrop-relay",
      "relay",
      [
        "[dependencies.totaldrop-proto]",
        'path = "../proto"',
        "[target.'cfg(unix)'.dependencies]",
        'totaldrop-crypto.path = "../crypto"',
        "",
      ].join("\n"),
    );
    assert.deepEqual(violations(readWorkspace(dir)), [
      "totaldrop-relay -> totaldrop-crypto: not allowed",
    ]);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
