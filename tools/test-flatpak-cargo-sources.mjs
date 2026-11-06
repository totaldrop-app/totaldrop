// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test tools/test-flatpak-cargo-sources.mjs

import { test } from "node:test";
import assert from "node:assert/strict";
import { packages, sources } from "./flatpak-cargo-sources.mjs";

const sum = "a".repeat(64);
const io = "registry+https://github.com/rust-lang/crates.io-index";

test("reads the package tables and skips workspace members", () => {
  const lock = `version = 4

[[package]]
name = "totaldrop-core"
version = "0.2.0"
dependencies = [
 "zeta",
]

[[package]]
name = "zeta"
version = "1.0.0"
source = "${io}"
checksum = "${sum}"
`;
  const p = packages(lock);
  assert.equal(p.length, 2);
  const s = sources(p);
  // One archive and one checksum file for zeta, then the cargo config.
  assert.equal(s.length, 3);
  assert.deepEqual(s[0], {
    type: "archive",
    "archive-type": "tar-gzip",
    url: "https://static.crates.io/crates/zeta/zeta-1.0.0.crate",
    sha256: sum,
    dest: "cargo/vendor/zeta-1.0.0",
  });
  assert.equal(s[1]["dest-filename"], ".cargo-checksum.json");
  assert.equal(JSON.parse(s[1].contents).package, sum);
  assert.match(s[2].contents, /replace-with = "vendored-sources"/);
});

test("two versions of one crate are both kept, sorted", () => {
  const p = [
    { name: "b", version: "2.0.0", source: io, checksum: sum },
    { name: "b", version: "1.0.0", source: io, checksum: sum },
    { name: "a", version: "1.0.0", source: io, checksum: sum },
  ];
  const dests = sources(p).filter((x) => x.type === "archive").map((x) => x.dest);
  assert.deepEqual(dests, ["cargo/vendor/a-1.0.0", "cargo/vendor/b-1.0.0", "cargo/vendor/b-2.0.0"]);
});

test("a git source is refused, not left out", () => {
  const p = [{ name: "x", version: "1.0.0", source: "git+https://example.org/x#abc" }];
  assert.throws(() => sources(p), /only crates.io/);
});

test("a missing or malformed checksum is refused", () => {
  assert.throws(() => sources([{ name: "x", version: "1.0.0", source: io }]), /checksum/);
  assert.throws(() => sources([{ name: "x", version: "1.0.0", source: io, checksum: "zz" }]), /checksum/);
});

test("a name that could leave the vendor folder is refused", () => {
  assert.throws(
    () => sources([{ name: "../x", version: "1.0.0", source: io, checksum: sum }]),
    /unexpected characters/,
  );
  assert.throws(
    () => sources([{ name: "x", version: "1.0.0/../../y", source: io, checksum: sum }]),
    /unexpected characters/,
  );
});
