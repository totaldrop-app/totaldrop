// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test tools/test-check-layers.mjs

import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { parseManifest, violations } from "./check-layers.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const workspace = new Set([
  "totaldrop-proto",
  "totaldrop-crypto",
  "totaldrop-core",
  "totaldrop-relay",
  "totaldrop-cli",
]);

test("reads the package name, not a [[bin]] name after it", () => {
  const m = parseManifest(
    '[package]\nname = "totaldrop-cli"\n\n[[bin]]\nname = "totaldrop"\npath = "src/main.rs"\n',
    workspace,
  );
  assert.equal(m.name, "totaldrop-cli");
});

test("finds workspace crates in every dependency table, and only those", () => {
  const m = parseManifest(
    [
      "[package]",
      'name = "totaldrop-cli"',
      "[dependencies]",
      'tokio = "1"',
      'totaldrop-core = { path = "../totaldrop-core" } # the engine',
      "[dev-dependencies]",
      'totaldrop-relay = { path = "../../server/totaldrop-relay" }',
      "[target.'cfg(unix)'.dependencies]",
      'proto = { package = "totaldrop-proto", path = "../totaldrop-proto" }',
      "[build-dependencies]",
      'totaldrop-crypto = { path = "../totaldrop-crypto" }',
      "[lints]",
      'totaldrop-core = "not a dependency"',
    ].join("\n"),
    workspace,
  );
  assert.deepEqual(m.deps, [
    { crate: "totaldrop-core", dev: false },
    { crate: "totaldrop-relay", dev: true },
    { crate: "totaldrop-proto", dev: false },
    { crate: "totaldrop-crypto", dev: false },
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

test("a crate the table does not know is reported, not let through", () => {
  assert.deepEqual(violations([{ name: "totaldrop-new", deps: [] }]), [
    "totaldrop-new: not in tools/check-layers.mjs; add it to ALLOWED",
  ]);
});

test("this repository's crates follow the layers", () => {
  const out = execFileSync("node", [join(here, "check-layers.mjs")], { encoding: "utf8" });
  assert.match(out, /^layers ok: \d+ crates$/m);
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
