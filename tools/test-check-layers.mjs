// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test tools/test-check-layers.mjs (needs cargo)

import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ALLOWED, readWorkspace, violations, workspaceEdges } from "./check-layers.mjs";

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

test("the servers are allowed in the CLI's tests, not in the CLI itself", () => {
  assert.deepEqual(
    violations([{ name: "totaldrop-cli", deps: [{ crate: "totaldrop-relay", dev: true }] }]),
    [],
  );
  assert.deepEqual(
    violations([{ name: "totaldrop-cli", deps: [{ crate: "totaldrop-relay", dev: false }] }]),
    ["totaldrop-cli -> totaldrop-relay: not allowed"],
  );
});

test("l10n may use the code format in its tests only", () => {
  assert.deepEqual(
    violations([{ name: "totaldrop-l10n", deps: [{ crate: "totaldrop-proto", dev: true }] }]),
    [],
  );
  assert.deepEqual(
    violations([{ name: "totaldrop-l10n", deps: [{ crate: "totaldrop-proto", dev: false }] }]),
    ["totaldrop-l10n -> totaldrop-proto: not allowed"],
  );
});

test("a crate the table does not know is reported, not let through", () => {
  assert.deepEqual(violations([{ name: "totaldrop-new", deps: [] }]), [
    "totaldrop-new: not in tools/check-layers.mjs; add it to ALLOWED",
  ]);
  assert.deepEqual(violations([{ name: "constructor", deps: [] }]), [
    "constructor: not in tools/check-layers.mjs; add it to ALLOWED",
  ]);
});

test("this repository has exactly the crates in the table, and they follow it", () => {
  const crates = readWorkspace(root);
  assert.deepEqual(crates.map((c) => c.name).sort(), Object.keys(ALLOWED).sort());
  assert.deepEqual(violations(crates), []);
  const out = execFileSync("node", [join(here, "check-layers.mjs")], { encoding: "utf8" });
  assert.match(out, new RegExp(`^layers ok: ${Object.keys(ALLOWED).length} crates`, "m"));
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
