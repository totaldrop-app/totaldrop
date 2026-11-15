// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Tests for tools/version.mjs: node --test tools/test-version.mjs
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import {
  bumpComponent,
  checkRepo,
  checkTag,
  compareSemver,
  formatSemver,
  main,
  nextSemver,
  parseSemver,
  planComponent,
  projectVersion,
  readManifest,
  validateStageSemver,
} from "./version.mjs";

const REPO_ROOT = fileURLToPath(new URL("..", import.meta.url));
const SCRIPT_PATH = fileURLToPath(new URL("./version.mjs", import.meta.url));

const GIT_ENV = {
  ...process.env,
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_AUTHOR_NAME: "test",
  GIT_AUTHOR_EMAIL: "test@example.invalid",
  GIT_COMMITTER_NAME: "test",
  GIT_COMMITTER_EMAIL: "test@example.invalid",
};
delete GIT_ENV.GIT_DIR;
delete GIT_ENV.GIT_WORK_TREE;
delete GIT_ENV.GIT_INDEX_FILE;
delete GIT_ENV.GIT_OBJECT_DIRECTORY;
delete GIT_ENV.GIT_COMMON_DIR;

function has(errors, part) {
  assert.ok(
    errors.some((e) => e.includes(part)),
    `expected "${part}" in ${JSON.stringify(errors)}`,
  );
}

function writeFiles(root, files) {
  for (const [rel, content] of Object.entries(files)) {
    const abs = join(root, rel);
    mkdirSync(join(abs, ".."), { recursive: true });
    writeFileSync(abs, content);
  }
}

/** Creates a complete synthetic monorepo fixture matching the live TotalDrop layout. */
function makeFixture(overrides = {}) {
  const root = mkdtempSync(join(tmpdir(), "version-test-"));
  const baseFiles = {
    "versions.json": `${JSON.stringify(
      {
        schema: 1,
        stage: "pre-1.0",
        components: {
          core: { version: "0.2.0-beta.5" },
          android: { version: "0.3.0-beta.7", build: 14 },
          linux: { version: "0.3.0-beta.6" },
          server: { version: "0.1.0-beta.1" },
        },
      },
      null,
      2,
    )}\n`,
    "Cargo.toml":
      '[workspace]\nmembers = ["crates/*", "linux", "app/rust", "server/totaldrop-rendezvous", "server/totaldrop-relay"]\n\n' +
      '[workspace.package]\nversion = "0.2.0-beta.5"\nedition = "2024"\n',
    "crates/totaldrop-crypto/Cargo.toml": '[package]\nname = "totaldrop-crypto"\nversion.workspace = true\n',
    "crates/totaldrop-proto/Cargo.toml":
      '[package]\nname = "totaldrop-proto"\nversion.workspace = true\n\n[dependencies]\n' +
      'totaldrop-crypto = { path = "../totaldrop-crypto", version = "0.2.0-beta.5" }\n',
    "crates/totaldrop-l10n/Cargo.toml": '[package]\nname = "totaldrop-l10n"\nversion.workspace = true\n',
    "crates/totaldrop-core/Cargo.toml":
      '[package]\nname = "totaldrop-core"\nversion.workspace = true\n\n[dependencies]\n' +
      'totaldrop-crypto = { path = "../totaldrop-crypto", version = "0.2.0-beta.5" }\n' +
      'totaldrop-proto = { path = "../totaldrop-proto", version = "0.2.0-beta.5" }\n' +
      'totaldrop-l10n = { path = "../totaldrop-l10n", version = "0.2.0-beta.5" }\n',
    "crates/totaldrop-cli/Cargo.toml":
      '[package]\nname = "totaldrop-cli"\nversion.workspace = true\n\n[dependencies]\n' +
      'totaldrop-core = { path = "../totaldrop-core", version = "0.2.0-beta.5" }\n',
    "app/pubspec.yaml": "name: totaldrop\nversion: 0.3.0-beta.7+14\n",
    "app/rust/Cargo.toml":
      '[package]\nname = "rust_lib_totaldrop"\nversion = "0.3.0-beta.7"\n\n[dependencies]\n' +
      'totaldrop-core = { path = "../../crates/totaldrop-core", version = "0.2.0-beta.5" }\n',
    "app/lib/main.dart":
      "const appVersion = String.fromEnvironment(\n  'TOTALDROP_VERSION',\n  defaultValue: '0.3.0-beta.7',\n);\n",
    "linux/Cargo.toml":
      '[package]\nname = "totaldrop-linux"\nversion = "0.3.0-beta.6"\n\n[dependencies]\n' +
      'totaldrop-core = { path = "../crates/totaldrop-core", version = "0.2.0-beta.5" }\n',
    "packaging/arch/PKGBUILD": "pkgname=totaldrop\n_ver=0.3.0-beta.6\npkgver=${_ver//-/_}\npkgrel=2\n",
    "packaging/rpm/totaldrop.spec":
      "%global upstream_version 0.3.0-beta.6\nName:           totaldrop\nVersion:        0.3.0~beta.6\nRelease:        3%{?dist}\n\n" +
      "%changelog\n* Sat Nov 07 2026 TotalDrop <hello@totaldrop.app> - 0.3.0~beta.6-1\n- Release 0.3.0-beta.6.\n",
    "packaging/debian/changelog":
      "totaldrop (0.3.0~beta.6-1) unstable; urgency=medium\n\n  * Release 0.3.0-beta.6.\n\n" +
      " -- TotalDrop <hello@totaldrop.app>  Thu, 12 Nov 2026 12:00:00 +0000\n\n",
    "packaging/linux/app.totaldrop.TotalDrop.metainfo.xml":
      '<?xml version="1.0" encoding="UTF-8"?>\n<component type="desktop-application">\n  <releases>\n' +
      '    <release version="0.3.0~beta.6" date="2026-11-12" type="development">\n' +
      "      <description><p>TotalDrop 0.3.0-beta.6.</p></description>\n    </release>\n  </releases>\n</component>\n",
    "server/totaldrop-rendezvous/Cargo.toml":
      '[package]\nname = "totaldrop-rendezvous"\nversion = "0.1.0-beta.1"\n\n[dependencies]\n' +
      'totaldrop-proto = { path = "../../crates/totaldrop-proto", version = "0.2.0-beta.5" }\n',
    "server/totaldrop-relay/Cargo.toml":
      '[package]\nname = "totaldrop-relay"\nversion = "0.1.0-beta.1"\n\n[dependencies]\n' +
      'totaldrop-rendezvous = { path = "../totaldrop-rendezvous", version = "0.1.0-beta.1" }\n',
    "server/rendezvous-worker/package.json": '{\n  "name": "totaldrop-rendezvous-worker",\n  "version": "0.1.0-beta.1"\n}\n',
    "server/rendezvous-worker/package-lock.json":
      '{\n  "name": "totaldrop-rendezvous-worker",\n  "version": "0.1.0-beta.1",\n  "lockfileVersion": 3,\n' +
      '  "packages": {\n    "": {\n      "name": "totaldrop-rendezvous-worker",\n      "version": "0.1.0-beta.1"\n    }\n  }\n}\n',
    "server/relay-worker/package.json": '{\n  "name": "totaldrop-relay-worker",\n  "version": "0.1.0-beta.1"\n}\n',
    "server/relay-worker/package-lock.json":
      '{\n  "name": "totaldrop-relay-worker",\n  "version": "0.1.0-beta.1",\n  "lockfileVersion": 3,\n' +
      '  "packages": {\n    "": {\n      "name": "totaldrop-relay-worker",\n      "version": "0.1.0-beta.1"\n    }\n  }\n}\n',
    "Cargo.lock":
      [
        ["totaldrop-crypto", "0.2.0-beta.5"],
        ["totaldrop-proto", "0.2.0-beta.5"],
        ["totaldrop-l10n", "0.2.0-beta.5"],
        ["totaldrop-core", "0.2.0-beta.5"],
        ["totaldrop-cli", "0.2.0-beta.5"],
        ["rust_lib_totaldrop", "0.3.0-beta.7"],
        ["totaldrop-linux", "0.3.0-beta.6"],
        ["totaldrop-rendezvous", "0.1.0-beta.1"],
        ["totaldrop-relay", "0.1.0-beta.1"],
      ]
        .map(([n, v]) => `[[package]]\nname = "${n}"\nversion = "${v}"\n`)
        .join("\n") + "\n",
    "fuzz/Cargo.lock":
      [
        ["totaldrop-proto", "0.2.0-beta.5"],
        ["totaldrop-l10n", "0.2.0-beta.5"],
      ]
        .map(([n, v]) => `[[package]]\nname = "${n}"\nversion = "${v}"\n`)
        .join("\n") + "\n",
    "docs/ARCHITECTURE.refs.json": `${JSON.stringify(
      {
        "app/rust/Cargo.toml:4": ['totaldrop-core = { path = "../../crates/totaldrop-core", version = "0.2.0-beta.5" }'],
        "server/totaldrop-relay/Cargo.toml:4": [
          'totaldrop-rendezvous = { path = "../totaldrop-rendezvous", version = "0.1.0-beta.1" }',
        ],
      },
      null,
      2,
    )}\n`,
    "changelog.d/README.md": "Fragments live here.\n",
  };
  writeFiles(root, { ...baseFiles, ...overrides });
  return root;
}

function gitRun(cwd, args) {
  return execFileSync("git", args, { cwd, env: GIT_ENV, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
}

test("the live repository passes checkRepo", () => {
  assert.deepEqual(checkRepo(REPO_ROOT), []);
});

test("SemVer parsing, comparison, and stage rules", () => {
  assert.deepEqual(parseSemver("0.3.0-beta.7"), {
    raw: "0.3.0-beta.7",
    major: 0,
    minor: 3,
    patch: 0,
    channel: "beta",
    pre: 7,
  });
  assert.deepEqual(parseSemver("1.2.3"), {
    raw: "1.2.3",
    major: 1,
    minor: 2,
    patch: 3,
    channel: null,
    pre: null,
  });
  assert.equal(formatSemver({ major: 1, minor: 0, patch: 0, channel: "rc", pre: 2 }), "1.0.0-rc.2");
  assert.equal(formatSemver({ major: 1, minor: 0, patch: 0 }), "1.0.0");

  for (const bad of [
    "",
    "v0.3.0-beta.7",
    "01.0.0",
    "0.02.0-beta.1",
    "0.3.0-beta.0",
    "1.0.0-rc.0",
    "1.0.0-gamma.1",
    "0.3.0-beta.7+14",
    "1.2",
  ]) {
    assert.equal(parseSemver(bad), null, `expected null for "${bad}"`);
  }

  const ordered = [
    "0.1.0-beta.1",
    "0.1.0-beta.9",
    "0.1.0-beta.10",
    "0.2.0-beta.1",
    "0.3.0-beta.7",
    "1.0.0-alpha.1",
    "1.0.0-alpha.9",
    "1.0.0-alpha.10",
    "1.0.0-beta.1",
    "1.0.0-rc.1",
    "1.0.0-rc.2",
    "1.0.0",
    "1.0.1-alpha.1",
    "1.0.1",
    "1.1.0-rc.1",
    "2.0.0",
  ];
  for (let i = 0; i < ordered.length; i++) {
    assert.equal(compareSemver(ordered[i], ordered[i]), 0);
    for (let j = i + 1; j < ordered.length; j++) {
      assert.equal(compareSemver(ordered[i], ordered[j]), -1, `${ordered[i]} < ${ordered[j]}`);
      assert.equal(compareSemver(ordered[j], ordered[i]), 1, `${ordered[j]} > ${ordered[i]}`);
    }
  }

  assert.equal(validateStageSemver("0.3.0-beta.7", "pre-1.0"), null);
  assert.match(validateStageSemver("0.3.0", "pre-1.0"), /0\.MINOR\.PATCH-beta\.N/);
  assert.match(validateStageSemver("0.3.0-rc.1", "pre-1.0"), /0\.MINOR\.PATCH-beta\.N/);
  assert.match(validateStageSemver("1.0.0-rc.1", "pre-1.0"), /0\.MINOR\.PATCH-beta\.N/);

  assert.equal(validateStageSemver("1.0.0-rc.1", "stable"), null);
  assert.equal(validateStageSemver("1.0.0", "stable"), null);
  assert.equal(validateStageSemver("0.1.0-beta.1", "stable"), null);
  assert.match(validateStageSemver("0.1.0", "stable"), /0\.MINOR\.PATCH-beta\.N/);
  assert.match(validateStageSemver("0.2.0-rc.1", "stable"), /0\.MINOR\.PATCH-beta\.N/);
});

test("native package and store projections", () => {
  const beta = projectVersion("0.3.0-beta.7", 14);
  assert.equal(beta.android, "0.3.0-beta.7+14");
  assert.equal(beta.androidVersionName, "0.3.0-beta.7");
  assert.equal(beta.androidVersionCode, 14);
  assert.equal(beta.deb, "0.3.0~beta.7");
  assert.equal(beta.debChangelog, "0.3.0~beta.7-1");
  assert.equal(beta.rpm, "0.3.0~beta.7");
  assert.equal(beta.arch, "0.3.0_beta.7");
  assert.equal(beta.appstream, "0.3.0~beta.7");
  assert.equal(beta.appstreamType, "development");
  assert.equal(beta.msix, "0.3.0.14");
  assert.deepEqual(beta.apple, {
    shortVersion: "0.3.0",
    bundleVersion: "14",
    displayVersion: "0.3.0-beta.7",
  });

  const stable = projectVersion("1.0.0", 25);
  assert.equal(stable.deb, "1.0.0");
  assert.equal(stable.rpm, "1.0.0");
  assert.equal(stable.arch, "1.0.0");
  assert.equal(stable.appstream, "1.0.0");
  assert.equal(stable.appstreamType, "stable");
  assert.equal(stable.msix, "1.0.0.25");
  assert.deepEqual(stable.apple, {
    shortVersion: "1.0.0",
    bundleVersion: "25",
    displayVersion: "1.0.0",
  });

  const hugeBuild = projectVersion("1.0.0", 70_000);
  assert.equal(hugeBuild.msix, null, "MSIX uint16 overflow yields null projection");
  assert.throws(() => projectVersion("0.3.0-beta.7", 0), /positive integer/);
});

test("readManifest validates schema, stage, components, and store build bounds", () => {
  const root = makeFixture({
    "versions.json": JSON.stringify({
      schema: 2,
      stage: "alpha",
      components: {
        unknown: { version: "0.1.0-beta.1" },
        core: { version: "0.2.0-beta.5", build: 3 },
        android: { version: "0.3.0-beta.7", build: 2_100_000_001 },
        windows: { version: "0.1.0-beta.1", build: 65_536 },
      },
    }),
  });
  try {
    const { errors } = readManifest(root);
    has(errors, "unsupported schema 2");
    has(errors, 'stage must be "pre-1.0" or "stable"');
    has(errors, 'unknown component "unknown"');
    has(errors, '"core" does not use an integer build number');
    has(errors, "exceeds Google Play limit");
    has(errors, "MSIX version parts must each be <= 65535");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("checkRepo catches drift across every manifest, lockfile, recipe, and internal dependency", () => {
  const clean = makeFixture();
  try {
    assert.deepEqual(checkRepo(clean), []);
  } finally {
    rmSync(clean, { recursive: true, force: true });
  }

  const driftCases = [
    [
      "app/lib/main.dart",
      "const appVersion = String.fromEnvironment('TOTALDROP_VERSION', defaultValue: '0.3.0-beta.6');\n",
      "app/lib/main.dart (TOTALDROP_VERSION)",
    ],
    ["app/pubspec.yaml", "name: totaldrop\nversion: 0.3.0-beta.7+13\n", "app/pubspec.yaml"],
    [
      "app/rust/Cargo.toml",
      '[package]\nname = "rust_lib_totaldrop"\nversion.workspace = true\n',
      "app/rust/Cargo.toml must name its own version",
    ],
    [
      "crates/totaldrop-cli/Cargo.toml",
      '[package]\nname = "totaldrop-cli"\nversion.workspace = true\n\n[dependencies]\n' +
        'totaldrop-core = { path = "../totaldrop-core", version = "0.2.0-beta.4" }\n',
      "crates/totaldrop-cli/Cargo.toml:6 (totaldrop-core)",
    ],
    [
      "Cargo.lock",
      '[[package]]\nname = "rust_lib_totaldrop"\nversion = "0.3.0-beta.6"\n',
      "Cargo.lock (rust_lib_totaldrop)",
    ],
    [
      "fuzz/Cargo.lock",
      '[[package]]\nname = "totaldrop-proto"\nversion = "0.2.0-beta.4"\n',
      "fuzz/Cargo.lock (totaldrop-proto)",
    ],
    ["packaging/arch/PKGBUILD", "_ver=0.3.0-beta.5\n", "packaging/arch/PKGBUILD (_ver)"],
    [
      "packaging/rpm/totaldrop.spec",
      "%global upstream_version 0.3.0-beta.6\nVersion:        0.3.0-beta.6\n",
      "packaging/rpm/totaldrop.spec (Version)",
    ],
    [
      "packaging/debian/changelog",
      "totaldrop (0.3.0~beta.5-1) unstable; urgency=medium\n",
      "packaging/debian/changelog",
    ],
    [
      "packaging/linux/app.totaldrop.TotalDrop.metainfo.xml",
      '<component><releases><release version="0.3.0~beta.6" date="2026-11-12" type="stable"/></releases></component>\n',
      "metainfo.xml (type)",
    ],
    [
      "server/relay-worker/package-lock.json",
      '{\n  "name": "totaldrop-relay-worker",\n  "version": "0.1.0-beta.1",\n' +
        '  "packages": { "": { "version": "0.0.9-beta.1" } }\n}\n',
      "server/relay-worker/package-lock.json (root package)",
    ],
    [
      "docs/ARCHITECTURE.refs.json",
      `${JSON.stringify(
        { "app/rust/Cargo.toml:4": ['totaldrop-core = { path = "../../crates/totaldrop-core", version = "0.2.0-beta.4" }'] },
        null,
        2,
      )}\n`,
      "docs/ARCHITECTURE.refs.json (app/rust/Cargo.toml:4, totaldrop-core)",
    ],
    [
      "server/mail-worker/package.json",
      '{\n  "name": "totaldrop-mail-worker",\n  "version": "0.1.0-beta.2"\n}\n',
      "server/mail-worker/package-lock.json: missing",
    ],
    [
      "server/mail-worker/package.json",
      '{\n  "name": "totaldrop-mail-worker"\n}\n',
      "server/mail-worker/package.json: \"null\" is not a valid TotalDrop SemVer",
    ],
  ];

  for (const [rel, badContent, expectedPart] of driftCases) {
    const root = makeFixture({ [rel]: badContent });
    try {
      has(checkRepo(root), expectedPart);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  }
});

test("modular single-component repository passes checkRepo and bumpComponent", () => {
  const root = mkdtempSync(join(tmpdir(), "version-modular-"));
  try {
    writeFiles(root, {
      "versions.json": `${JSON.stringify(
        { schema: 1, stage: "pre-1.0", components: { android: { version: "0.3.0-beta.7", build: 14 } } },
        null,
        2,
      )}\n`,
      "app/pubspec.yaml": "name: totaldrop\nversion: 0.3.0-beta.7+14\n",
      "app/rust/Cargo.toml": '[package]\nname = "rust_lib_totaldrop"\nversion = "0.3.0-beta.7"\n',
      "app/lib/main.dart":
        "const appVersion = String.fromEnvironment('TOTALDROP_VERSION', defaultValue: '0.3.0-beta.7');\n",
      "changelog.d/540-share-text.added.md": "- **Android:** Share received text (#540).\n",
    });
    assert.deepEqual(checkRepo(root), []);
    const bumped = bumpComponent(root, "android");
    assert.equal(bumped.version, "0.4.0-beta.1");
    assert.equal(bumped.build, 15);
    assert.deepEqual(checkRepo(root), []);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("planComponent and bumpComponent update every component atomically, including 1.0 promotion", () => {
  const root = makeFixture({
    "changelog.d/10-android-feat.added.md": "- **Android:** Added quick share action (#10).\n",
    "changelog.d/11-linux-fix.fixed.md": "- **Linux:** Fixed tray icon padding (#11).\n",
    "changelog.d/12-site-only.added.md": "- **Site:** Updated download page (#12).\n",
    "changelog.d/13-releng.none.md": "CI script cleanup only.\n",
  });
  try {
    const androidPlan = planComponent(root, "android");
    assert.equal(androidPlan.recommendedLevel, "minor");
    assert.equal(androidPlan.proposedVersion, "0.4.0-beta.1");
    assert.equal(androidPlan.proposedBuild, 15);
    assert.equal(androidPlan.sameSeriesBeta, "0.3.0-beta.8");

    const linuxPlan = planComponent(root, "linux");
    assert.equal(linuxPlan.recommendedLevel, "patch");
    assert.equal(linuxPlan.proposedVersion, "0.3.1-beta.1");

    const serverPlan = planComponent(root, "server");
    assert.equal(serverPlan.recommendedLevel, "prerelease");
    assert.equal(serverPlan.proposedVersion, "0.1.0-beta.2");

    // Failed bumps (rollover dates, conflicting flags, Play store overflow) leave zero mutated files on disk.
    const beforeVersionsJson = readFileSync(join(root, "versions.json"), "utf8");
    assert.throws(() => bumpComponent(root, "linux", { date: "2026-02-29" }), /invalid calendar date/);
    assert.throws(() => bumpComponent(root, "linux", { level: "patch", setVersion: "0.3.1-beta.1" }), /cannot pass both/);
    assert.throws(() => bumpComponent(root, "android", { build: 2_100_000_001 }), /exceeds Google Play limit/);
    assert.equal(readFileSync(join(root, "versions.json"), "utf8"), beforeVersionsJson);

    // Atomic bumps across all four components (including docs/ARCHITECTURE.refs.json, Debian Maintainer, RPM %changelog).
    bumpComponent(root, "core", { level: "patch" });
    bumpComponent(root, "android");
    bumpComponent(root, "linux", { date: "2026-11-15" });
    bumpComponent(root, "server");
    assert.deepEqual(checkRepo(root), []);
    const updatedRefs = JSON.parse(readFileSync(join(root, "docs/ARCHITECTURE.refs.json"), "utf8"));
    assert.deepEqual(updatedRefs["app/rust/Cargo.toml:4"], [
      'totaldrop-core = { path = "../../crates/totaldrop-core", version = "0.2.1-beta.1" }',
    ]);
    assert.deepEqual(updatedRefs["server/totaldrop-relay/Cargo.toml:4"], [
      'totaldrop-rendezvous = { path = "../totaldrop-rendezvous", version = "0.1.0-beta.2" }',
    ]);
    assert.match(
      readFileSync(join(root, "packaging/debian/changelog"), "utf8"),
      /^ -- TotalDrop <hello@totaldrop\.app> {2}Sun, 15 Nov 2026 12:00:00 \+0000$/m,
    );
    assert.match(
      readFileSync(join(root, "packaging/rpm/totaldrop.spec"), "utf8"),
      /^\* Sun Nov 15 2026 TotalDrop <hello@totaldrop\.app> - 0\.3\.1~beta\.1-1$/m,
    );

    // Reject non-increasing Android build numbers and non-increasing SemVer.
    assert.throws(() => bumpComponent(root, "android", { level: "prerelease", build: 15 }), /above current/);
    assert.throws(() => bumpComponent(root, "core", { setVersion: "0.2.0-beta.5" }), /higher than current/);

    // Promote Linux from 0.x -> 1.0.0-rc.1 -> 1.0.0 while other components stay valid on 0.x-beta.N.
    const rc = bumpComponent(root, "linux", { stage: "stable", level: "rc", date: "2026-12-01" });
    assert.equal(rc.version, "1.0.0-rc.1");
    assert.equal(planComponent(root, "linux").recommendedLevel, "prerelease");
    assert.match(readFileSync(join(root, "packaging/rpm/totaldrop.spec"), "utf8"), /^Version:\s+1\.0\.0~rc\.1$/m);
    assert.match(
      readFileSync(join(root, "packaging/linux/app.totaldrop.TotalDrop.metainfo.xml"), "utf8"),
      /version="1\.0\.0~rc\.1" date="2026-12-01" type="development"/,
    );

    const ga = bumpComponent(root, "linux", { level: "release", date: "2026-12-10" });
    assert.equal(ga.version, "1.0.0");
    assert.match(readFileSync(join(root, "packaging/rpm/totaldrop.spec"), "utf8"), /^Version:\s+1\.0\.0$/m);
    assert.match(
      readFileSync(join(root, "packaging/linux/app.totaldrop.TotalDrop.metainfo.xml"), "utf8"),
      /version="1\.0\.0" date="2026-12-10" type="stable"/,
    );
    assert.deepEqual(checkRepo(root), []);

    // Post-1.0 channel progressions and pre-release stripping via nextSemver.
    assert.equal(nextSemver("1.0.0-rc.1", "stable", "patch"), "1.0.0");
    assert.equal(nextSemver("1.1.0-rc.1", "stable", "minor"), "1.1.0");
    assert.equal(nextSemver("2.0.0-rc.1", "stable", "major"), "2.0.0");
    assert.equal(nextSemver("1.0.0", "stable", "alpha"), "1.1.0-alpha.1");
    assert.equal(nextSemver("1.1.0-alpha.1", "stable", "alpha"), "1.1.0-alpha.2");
    assert.equal(nextSemver("1.1.0-alpha.2", "stable", "beta"), "1.1.0-beta.1");
    assert.equal(nextSemver("1.1.0-beta.1", "stable", "rc"), "1.1.0-rc.1");
    assert.throws(() => nextSemver("1.1.0-rc.1", "stable", "beta"), /cannot move backwards/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("checkTag enforces origin/main ancestry, SemVer monotonicity, and highest buildCode across all merge parents", () => {
  const root = makeFixture();
  const shallow = `${root}-shallow`;
  try {
    gitRun(root, ["init", "-q", "-b", "main"]);
    gitRun(root, ["add", "-A"]);
    gitRun(root, ["commit", "-q", "-m", "initial 0.3.0-beta.7+14"]);
    gitRun(root, ["tag", "-a", "-m", "android-v0.3.0-beta.7", "android-v0.3.0-beta.7"]);

    // Missing origin/main fails closed.
    has(checkTag(root, "android-v0.3.0-beta.7"), 'main reference "origin/main" not found');

    // Point origin/main to main: first release tag passes.
    gitRun(root, ["update-ref", "refs/remotes/origin/main", "HEAD"]);
    assert.deepEqual(checkTag(root, "android-v0.3.0-beta.7"), []);

    // Plan sees that HEAD equals the last release tag (not alreadyAhead).
    assert.equal(planComponent(root, "android").alreadyAhead, false);

    // Side branch with a HIGHER version + build (0.3.0-beta.9+16) merged as HEAD^2 (second parent!)
    // while main's tree stays at 0.3.0-beta.8+15: checkTag MUST inspect HEAD^2 and reject!
    gitRun(root, ["checkout", "-q", "-b", "side-higher", "HEAD"]);
    bumpComponent(root, "android", { setVersion: "0.3.0-beta.9", build: 16 });
    gitRun(root, ["add", "-A"]);
    gitRun(root, ["commit", "-q", "-m", "side higher 0.3.0-beta.9+16"]);
    gitRun(root, ["tag", "android-v0.3.0-beta.9"]);

    gitRun(root, ["checkout", "-q", "main"]);
    bumpComponent(root, "android", { level: "prerelease" }); // 0.3.0-beta.8+15
    gitRun(root, ["add", "-A"]);
    gitRun(root, ["commit", "-q", "-m", "main 0.3.0-beta.8+15"]);
    gitRun(root, ["merge", "-q", "--strategy=ours", "--no-edit", "side-higher"]);
    gitRun(root, ["update-ref", "refs/remotes/origin/main", "HEAD"]);

    const secondParentErrors = checkTag(root, "android-v0.3.0-beta.8");
    has(secondParentErrors, "android version 0.3.0-beta.8 (android-v0.3.0-beta.8) is not higher than 0.3.0-beta.9");
    has(secondParentErrors, "android build number 15 (android-v0.3.0-beta.8) is not above 16");

    // Bumping SemVer to 0.3.0-beta.10 with build 17 on main passes, and plan sees alreadyAhead before tagging.
    bumpComponent(root, "android", { setVersion: "0.3.0-beta.10", build: 17 });
    gitRun(root, ["add", "-A"]);
    gitRun(root, ["commit", "-q", "-m", "bump android 0.3.0-beta.10+17"]);
    assert.equal(planComponent(root, "android").alreadyAhead, true);
    assert.throws(() => bumpComponent(root, "android"), /already ahead/);

    // Before updating origin/main, HEAD is ahead of origin/main -> off-main commit rejected!
    has(checkTag(root, "android-v0.3.0-beta.10"), "is not an ancestor of origin/main");

    // Once merged into origin/main, checkTag passes cleanly, and checking an existing tag pointing elsewhere fails!
    gitRun(root, ["update-ref", "refs/remotes/origin/main", "HEAD"]);
    gitRun(root, ["tag", "android-v0.3.0-beta.10"]);
    assert.deepEqual(checkTag(root, "android-v0.3.0-beta.10"), []);
    has(checkTag(root, "android-v0.3.0-beta.7"), "points to");

    // Shallow clone fails checkTag and sets shallowClone=true in planComponent.
    execFileSync("git", ["clone", "-q", "--depth", "1", `file://${root}`, shallow], { env: GIT_ENV });
    has(checkTag(shallow, "android-v0.3.0-beta.10"), "shallow clone");
    assert.equal(planComponent(shallow, "android").shallowClone, true);
  } finally {
    rmSync(root, { recursive: true, force: true });
    rmSync(shallow, { recursive: true, force: true });
  }
});

test("checkTag delegates linux-v*, cli-v*, and core-v* tags to packaging/check-cli-version.sh", () => {
  const root = makeFixture({
    "packaging/check-cli-version.sh": readFileSync(join(REPO_ROOT, "packaging/check-cli-version.sh"), "utf8"),
    "crates/totaldrop-cli/src/main.rs": "fn main() {}\n",
    "l10n/app/en.arb": "{}\n",
    "l10n/cli/en.arb": "{}\n",
    "l10n/status.json": "{}\n",
  });
  try {
    gitRun(root, ["init", "-q", "-b", "main"]);
    gitRun(root, ["add", "-A"]);
    gitRun(root, ["commit", "-q", "-m", "linux 0.3.0-beta.6 with core 0.2.0-beta.5"]);
    gitRun(root, ["tag", "linux-v0.3.0-beta.6"]);

    // Modify CLI source and bump ONLY linux (not core): checkTag must reject both linux-v* and cli-v*/core-v*!
    writeFiles(root, { "crates/totaldrop-cli/src/main.rs": 'fn main() { println!("changed"); }\n' });
    bumpComponent(root, "linux", { level: "prerelease", date: "2026-11-15" });
    gitRun(root, ["add", "-A"]);
    gitRun(root, ["commit", "-q", "-m", "linux 0.3.0-beta.7 without bumping core"]);
    gitRun(root, ["update-ref", "refs/remotes/origin/main", "HEAD"]);
    gitRun(root, ["tag", "linux-v0.3.0-beta.7"]);
    has(checkTag(root, "linux-v0.3.0-beta.7"), "CLI version 0.2.0-beta.5 shipped in linux-v0.3.0-beta.6");
    has(checkTag(root, "cli-v0.2.0-beta.5"), "CLI version 0.2.0-beta.5 shipped in linux-v0.3.0-beta.6");
    has(checkTag(root, "core-v0.2.0-beta.5"), "CLI version 0.2.0-beta.5 shipped in linux-v0.3.0-beta.6");

    // Bumping core as well makes linux-v0.3.0-beta.8 and core-v0.2.1-beta.1 pass cleanly.
    bumpComponent(root, "core", { level: "patch" });
    bumpComponent(root, "linux", { level: "prerelease", date: "2026-11-15" });
    gitRun(root, ["add", "-A"]);
    gitRun(root, ["commit", "-q", "-m", "linux 0.3.0-beta.8 with core 0.2.1-beta.1"]);
    gitRun(root, ["update-ref", "refs/remotes/origin/main", "HEAD"]);
    gitRun(root, ["tag", "linux-v0.3.0-beta.8"]);
    gitRun(root, ["tag", "core-v0.2.1-beta.1"]);
    assert.deepEqual(checkTag(root, "linux-v0.3.0-beta.8"), []);
    assert.deepEqual(checkTag(root, "core-v0.2.1-beta.1"), []);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("CLI entry point handles check, plan --json, project --format, and strict per-subcommand flags", () => {
  const run = (args, cwd = REPO_ROOT) =>
    spawnSync(process.execPath, [SCRIPT_PATH, ...args], { cwd, encoding: "utf8" });

  const chk = run(["check"]);
  assert.equal(chk.status, 0);
  assert.match(chk.stdout, /version check ok:.*android 0\.3\.0-beta\.7\+14/);

  const proj = run(["project", "android", "--format", "msix"]);
  assert.equal(proj.status, 0);
  assert.equal(proj.stdout.trim(), "0.3.0.14");

  const apple = run(["project", "android", "--format", "apple"]);
  assert.equal(apple.status, 0);
  assert.deepEqual(JSON.parse(apple.stdout), {
    shortVersion: "0.3.0",
    bundleVersion: "14",
    displayVersion: "0.3.0-beta.7",
  });

  const plan = run(["plan", "android", "--json"]);
  assert.equal(plan.status, 0);
  const parsed = JSON.parse(plan.stdout);
  assert.equal(parsed.length, 1);
  assert.equal(parsed[0].component, "android");
  assert.equal(parsed[0].currentVersion, "0.3.0-beta.7");
  assert.equal(parsed[0].currentBuild, 14);

  assert.equal(main(["unknown-subcommand"], REPO_ROOT), 2);
  const badFlag = run(["check", "--nope"]);
  assert.equal(badFlag.status, 1);
  assert.match(badFlag.stderr, /unknown flag/);

  const wrongSubcmdFlag = run(["check", "--level", "minor"]);
  assert.equal(wrongSubcmdFlag.status, 1);
  assert.match(wrongSubcmdFlag.stderr, /unknown flag "--level" for subcommand "check"/);

  const nonDecimalBuild = run(["bump", "android", "--build", "0x10"]);
  assert.equal(nonDecimalBuild.status, 1);
  assert.match(nonDecimalBuild.stderr, /--build must be a positive integer/);
});
