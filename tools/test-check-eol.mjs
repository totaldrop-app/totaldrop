// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test tools/test-check-eol.mjs

import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  DEFAULT_WARN_DAYS,
  EolCheckError,
  MAX_RESPONSE_BYTES,
  evaluateTargets,
  extractTargets,
  fetchProductReleases,
  parseContainerImage,
  parseUtcDate,
  runCheck,
  runCli,
  sanitizeForLog,
} from "./check-eol.mjs";

const DIGEST_A = "a".repeat(64);
const DIGEST_B = "b".repeat(64);
const DIGEST_C = "c".repeat(64);
const DIGEST_D = "d".repeat(64);

async function withTempRepo(files, fn) {
  const root = mkdtempSync(join(tmpdir(), "td-eol-test-"));
  try {
    for (const [rel, content] of Object.entries(files)) {
      const full = join(root, rel);
      mkdirSync(dirname(full), { recursive: true });
      writeFileSync(full, content, "utf8");
    }
    return await fn(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

function mockJsonFetch(routes) {
  return async (url, init = {}) => {
    assert.equal(init.redirect, "error");
    assert.ok(init.signal instanceof AbortSignal);
    const handler = routes[url];
    if (!handler) {
      return new Response("not found", { status: 404 });
    }
    return typeof handler === "function" ? handler(url, init) : handler;
  };
}

function jsonResponse(payload, status = 200, headers = {}) {
  return new Response(JSON.stringify(payload), {
    status,
    headers: { "content-type": "application/json", ...headers },
  });
}

test("extracts Debian codename/numeric/slim/digest, Fedora, Arch rolling, Flatpak KDE, and Node major", async () => {
  await withTempRepo(
    {
      "packaging/containers/arch.Dockerfile": `# Arch build container\nFROM archlinux:latest@sha256:${DIGEST_A}\nRUN pacman -Syu --noconfirm\n`,
      "packaging/containers/debian.Dockerfile": `FROM debian:trixie-slim@sha256:${DIGEST_B}\n`,
      "packaging/containers/debian-num.Dockerfile": `FROM --platform=linux/amd64 docker.io/library/debian:12.15-slim@sha256:${DIGEST_C} AS build\nFROM build AS final\n`,
      "packaging/containers/fedora.Dockerfile": `FROM fedora:44@sha256:${DIGEST_D}\n`,
      "packaging/containers/flatpak.Dockerfile": `FROM ghcr.io/flathub-infra/flatpak-github-actions:kde-6.11@sha256:${DIGEST_A}\n`,
      ".github/workflows/ci.yml": `jobs:\n  changelog:\n    steps:\n      - uses: actions/setup-node@0\n        with:\n          node-version: "24"\n`,
      ".github/workflows/site.yml": `jobs:\n  site:\n    steps:\n      - uses: actions/setup-node@0\n        with:\n          node-version: "24.11.1"\n`,
    },
    (root) => {
      const targets = extractTargets(root);
      assert.deepEqual(
        targets.map((t) => ({
          kind: t.kind,
          source: t.source,
          product: t.product ?? null,
          matchBy: t.matchBy ?? null,
          cycle: t.cycle ?? null,
          codename: t.codename ?? null,
        })),
        [
          {
            kind: "exempt",
            source: "packaging/containers/arch.Dockerfile",
            product: null,
            matchBy: null,
            cycle: null,
            codename: null,
          },
          {
            kind: "eol",
            source: "packaging/containers/debian-num.Dockerfile",
            product: "debian",
            matchBy: "cycle",
            cycle: "12",
            codename: null,
          },
          {
            kind: "eol",
            source: "packaging/containers/debian.Dockerfile",
            product: "debian",
            matchBy: "codename",
            cycle: null,
            codename: "trixie",
          },
          {
            kind: "eol",
            source: "packaging/containers/fedora.Dockerfile",
            product: "fedora",
            matchBy: "cycle",
            cycle: "44",
            codename: null,
          },
          {
            kind: "exempt",
            source: "packaging/containers/flatpak.Dockerfile",
            product: null,
            matchBy: null,
            cycle: null,
            codename: null,
          },
          {
            kind: "eol",
            source: ".github/workflows/",
            product: "nodejs",
            matchBy: "cycle",
            cycle: "24",
            codename: null,
          },
        ],
      );
    },
  );
});

test("extracts real repository targets cleanly without network calls", () => {
  const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
  const targets = extractTargets(repoRoot);
  const products = targets.filter((t) => t.kind === "eol").map((t) => t.product);
  assert.deepEqual(products, ["debian", "fedora", "nodejs"]);
  const exempt = targets.filter((t) => t.kind === "exempt");
  assert.ok(exempt.some((t) => t.source === "packaging/containers/arch.Dockerfile"));
});

test("passes when all EOL dates are strictly more than 60 days ahead", async () => {
  await withTempRepo(
    {
      "packaging/containers/arch.Dockerfile": `FROM archlinux:latest@sha256:${DIGEST_A}\n`,
      "packaging/containers/debian.Dockerfile": `FROM debian:trixie@sha256:${DIGEST_B}\n`,
      "packaging/containers/fedora.Dockerfile": `FROM fedora:44@sha256:${DIGEST_C}\n`,
      "packaging/containers/flatpak.Dockerfile": `FROM ghcr.io/flathub-infra/flatpak-github-actions:kde-6.11@sha256:${DIGEST_D}\n`,
      ".github/workflows/ci.yml": `jobs:\n  a:\n    steps:\n      - uses: actions/setup-node@0\n        with:\n          node-version: "24"\n`,
    },
    async (root) => {
      const fetchImpl = mockJsonFetch({
        "https://endoflife.date/api/debian.json": () =>
          jsonResponse([
            { cycle: "13", codename: "Trixie", eol: "2028-08-09", extendedSupport: "2030-06-30" },
          ]),
        "https://endoflife.date/api/fedora.json": () =>
          jsonResponse([{ cycle: "44", eol: "2027-06-02" }]),
        "https://endoflife.date/api/nodejs.json": () =>
          jsonResponse([{ cycle: "24", eol: "2028-04-30" }]),
      });

      const report = await runCheck({
        root,
        now: "2026-11-19",
        warnDays: DEFAULT_WARN_DAYS,
        fetchImpl,
      });
      assert.equal(report.ok, true);
      assert.equal(report.failures.length, 0);
      assert.equal(report.results.length, 5);
      const fedora = report.results.find((r) => r.target.product === "fedora");
      assert.equal(fedora.daysRemaining, 195);
      assert.equal(fedora.status, "ok");
    },
  );
});

test("fails at 60-day boundary, 13-day Fedora #392 case, and already-expired -5 days", () => {
  const targets = [
    parseContainerImage("FROM debian:bookworm-slim", "packaging/containers/debian.Dockerfile", 1),
    parseContainerImage("FROM fedora:43", "packaging/containers/fedora.Dockerfile", 1),
    {
      kind: "eol",
      source: ".github/workflows/",
      label: "Node 20",
      product: "nodejs",
      matchBy: "cycle",
      cycle: "20",
    },
  ];
  const releasesByProduct = new Map([
    // Exactly 60 days ahead of 2026-11-18 -> 2027-01-17: boundary must warn/fail.
    ["debian", [{ cycle: "12", codename: "Bookworm", eol: "2027-01-17" }]],
    // 13 days ahead of 2026-11-18 -> 2026-12-01 (Fedora 43 in #392).
    ["fedora", [{ cycle: "43", eol: "2026-12-01" }]],
    // Expired 5 days ago -> 2026-11-13.
    ["nodejs", [{ cycle: "20", eol: "2026-11-13" }]],
  ]);

  const report = evaluateTargets(targets, releasesByProduct, {
    now: "2026-11-18",
    warnDays: 60,
  });

  assert.equal(report.ok, false);
  assert.equal(report.failures.length, 3);

  assert.equal(report.failures[0].status, "warning");
  assert.equal(report.failures[0].daysRemaining, 60);
  assert.ok(report.failures[0].message.includes("60 days"));
  assert.ok(report.failures[0].message.includes("2027-01-17"));

  assert.equal(report.failures[1].status, "warning");
  assert.equal(report.failures[1].daysRemaining, 13);
  assert.ok(report.failures[1].message.includes("13 days"));
  assert.ok(report.failures[1].message.includes("2026-12-01"));

  assert.equal(report.failures[2].status, "expired");
  assert.equal(report.failures[2].daysRemaining, -5);
  assert.ok(report.failures[2].message.includes("5 days ago"));
  assert.ok(report.failures[2].message.includes("2026-11-13"));

  // Verify 61 days ahead passes.
  const passReport = evaluateTargets(
    [parseContainerImage("FROM debian:bookworm", "packaging/containers/debian.Dockerfile", 1)],
    new Map([["debian", [{ cycle: "12", codename: "Bookworm", eol: "2027-01-18" }]]]),
    { now: "2026-11-18", warnDays: 60 },
  );
  assert.equal(passReport.ok, true);
  assert.equal(passReport.results[0].daysRemaining, 61);

  // Standard `eol` governs the warning even when Debian `extendedSupport` is years away.
  const debianExtendedReport = evaluateTargets(
    [parseContainerImage("FROM debian:trixie", "packaging/containers/debian.Dockerfile", 1)],
    new Map([
      [
        "debian",
        [{ cycle: "13", codename: "Trixie", eol: "2028-08-09", extendedSupport: "2030-06-30" }],
      ],
    ]),
    { now: "2028-07-01", warnDays: 60 },
  );
  assert.equal(debianExtendedReport.ok, false);
  assert.equal(debianExtendedReport.failures.length, 1);
  assert.equal(debianExtendedReport.failures[0].status, "warning");
  assert.equal(debianExtendedReport.failures[0].daysRemaining, 39);
});

test("handles boolean eol: true (expired) and eol: false (supported without scheduled date)", () => {
  const targets = [
    parseContainerImage("FROM fedora:41", "packaging/containers/fedora.Dockerfile", 1),
    parseContainerImage("FROM debian:forky", "packaging/containers/debian.Dockerfile", 1),
  ];
  const releasesByProduct = new Map([
    ["fedora", [{ cycle: "41", eol: true }]],
    ["debian", [{ cycle: "14", codename: "Forky", eol: false }]],
  ]);

  const report = evaluateTargets(targets, releasesByProduct, {
    now: "2026-11-19",
    warnDays: 60,
  });

  assert.equal(report.ok, false);
  assert.equal(report.failures.length, 1);
  assert.equal(report.failures[0].status, "expired");
  assert.match(report.failures[0].message, /already end of life/i);

  const debianResult = report.results.find((r) => r.target.product === "debian");
  assert.equal(debianResult.status, "ok");
  assert.equal(debianResult.daysRemaining, null);
});

test("rejects unrecognized, moving, untagged, and malformed-digest base images", () => {
  assert.throws(
    () => parseContainerImage("FROM alpine:3.20", "packaging/containers/alpine.Dockerfile", 2),
    (err) =>
      err instanceof EolCheckError &&
      err.code === "UNRECOGNIZED_IMAGE" &&
      err.message.includes("alpine.Dockerfile:2") &&
      err.message.includes("alpine:3.20"),
  );

  for (const moving of ["FROM debian:stable", "FROM debian:sid-slim", "FROM fedora:rawhide", "FROM fedora:latest"]) {
    assert.throws(
      () => parseContainerImage(moving, "packaging/containers/test.Dockerfile", 1),
      (err) => err instanceof EolCheckError && err.code === "MOVING_IMAGE_TAG",
    );
  }

  assert.throws(
    () => parseContainerImage("FROM debian", "packaging/containers/debian.Dockerfile", 1),
    (err) => err instanceof EolCheckError && err.code === "MISSING_IMAGE_TAG",
  );

  assert.throws(
    () => parseContainerImage("FROM fedora:44@sha256:nothex", "packaging/containers/fedora.Dockerfile", 1),
    (err) => err instanceof EolCheckError && err.code === "INVALID_IMAGE_DIGEST",
  );
});

test("rejects missing FROM, missing containers dir, and mismatched or moving workflow node-version", async () => {
  await withTempRepo(
    {
      "packaging/containers/empty.Dockerfile": "# comment only\nRUN true\n",
      ".github/workflows/ci.yml": 'node-version: "24"\n',
    },
    (root) => {
      assert.throws(
        () => extractTargets(root),
        (err) => err instanceof EolCheckError && err.code === "MISSING_DOCKERFILE_FROM",
      );
    },
  );

  await withTempRepo(
    {
      ".github/workflows/ci.yml": 'node-version: "24"\n',
    },
    (root) => {
      assert.throws(
        () => extractTargets(root),
        (err) => err instanceof EolCheckError && err.code === "MISSING_CONTAINERS_DIR",
      );
    },
  );

  await withTempRepo(
    {
      "packaging/containers/arch.Dockerfile": "FROM archlinux:latest\n",
      ".github/workflows/ci.yml": 'node-version: "24"\n',
      ".github/workflows/site.yml": 'node-version: "22"\n',
    },
    (root) => {
      assert.throws(
        () => extractTargets(root),
        (err) => err instanceof EolCheckError && err.code === "NODE_MAJOR_MISMATCH",
      );
    },
  );

  await withTempRepo(
    {
      "packaging/containers/arch.Dockerfile": "FROM archlinux:latest\n",
      ".github/workflows/ci.yml": 'node-version: "lts/*"\n',
    },
    (root) => {
      assert.throws(
        () => extractTargets(root),
        (err) => err instanceof EolCheckError && err.code === "INVALID_NODE_VERSION",
      );
    },
  );

  // BuildKit skips `#` comment lines inside `\`-continued FROM instructions.
  await withTempRepo(
    {
      "packaging/containers/debian.Dockerfile": [
        "FROM \\",
        "  # comment inside backslash continuation",
        `  --platform=linux/amd64 \\`,
        `  debian:trixie-backports-slim@sha256:${DIGEST_B} \\`,
        "  AS builder",
        "",
      ].join("\n"),
      ".github/workflows/ci.yml": 'node-version: "24" # LTS\n',
    },
    (root) => {
      const targets = extractTargets(root);
      assert.equal(targets[0].product, "debian");
      assert.equal(targets[0].codename, "trixie");
    },
  );

  // Fail-closed: a malformed FROM line must fail even when another valid FROM exists.
  for (const badFrom of [
    "FROM debian:trixie\nFROM\n",
    "FROM debian:trixie\nFROM fedora:44 extra-token\n",
    "FROM debian:trixie\nFROM --platform fedora:44\n",
    "FROM debian:trixie\nFROM fedora:44 AS\n",
  ]) {
    await withTempRepo(
      {
        "packaging/containers/bad.Dockerfile": badFrom,
        ".github/workflows/ci.yml": 'node-version: "24"\n',
      },
      (root) => {
        assert.throws(
          () => extractTargets(root),
          (err) => err instanceof EolCheckError && err.code === "INVALID_DOCKERFILE_FROM",
        );
      },
    );
  }
});

test("handles HTTP errors, oversized bodies, non-array JSON, malformed eol fields, and missing cycles", async () => {
  await assert.rejects(
    () =>
      fetchProductReleases("fedora", {
        fetchImpl: mockJsonFetch({
          "https://endoflife.date/api/fedora.json": () => new Response("server down", { status: 503 }),
        }),
      }),
    (err) => err instanceof EolCheckError && err.code === "HTTP_ERROR" && /HTTP 503/.test(err.message),
  );

  await assert.rejects(
    () =>
      fetchProductReleases("fedora", {
        maxBytes: 64,
        fetchImpl: mockJsonFetch({
          "https://endoflife.date/api/fedora.json": () =>
            new Response("[" + " ".repeat(200) + "]", { status: 200 }),
        }),
      }),
    (err) => err instanceof EolCheckError && err.code === "RESPONSE_TOO_LARGE" && MAX_RESPONSE_BYTES > 64,
  );

  for (const badPayload of [{ cycle: "44" }, [], "not-an-array", [null], [{ noCycle: true }]]) {
    await assert.rejects(
      () =>
        fetchProductReleases("fedora", {
          fetchImpl: mockJsonFetch({
            "https://endoflife.date/api/fedora.json": () => jsonResponse(badPayload),
          }),
        }),
      (err) => err instanceof EolCheckError && err.code === "INVALID_API_PAYLOAD",
    );
  }

  assert.throws(
    () =>
      evaluateTargets(
        [parseContainerImage("FROM fedora:99", "packaging/containers/fedora.Dockerfile", 1)],
        new Map([["fedora", [{ cycle: "44", eol: "2027-06-02" }]]]),
        { now: "2026-11-19", warnDays: 60 },
      ),
    (err) => err instanceof EolCheckError && err.code === "UNKNOWN_RELEASE_CYCLE" && /44|99/.test(err.message),
  );

  assert.throws(
    () =>
      evaluateTargets(
        [parseContainerImage("FROM fedora:44", "packaging/containers/fedora.Dockerfile", 1)],
        new Map([["fedora", [{ cycle: "44", eol: "2027-02-30" }]]]),
        { now: "2026-11-19", warnDays: 60 },
      ),
    (err) => err instanceof EolCheckError && err.code === "INVALID_EOL_FIELD",
  );

  assert.throws(
    () => parseUtcDate("2026-02-29"),
    (err) => err instanceof EolCheckError && err.code === "INVALID_DATE",
  );
});

test("sanitizes backslash, control, ANSI escape, newline, and Bidi characters in log output", () => {
  assert.equal(sanitizeForLog("a\\b\x1b"), "a\\\\b\\u001b");
  assert.notEqual(sanitizeForLog("\\u001b"), sanitizeForLog("\x1b"));

  const dirty = "trixie\x1b[31m\n::error::spoof\u202eevil\u0000";
  const cleaned = sanitizeForLog(dirty);
  assert.ok(!cleaned.includes("\x1b"));
  assert.ok(!cleaned.includes("\n"));
  assert.ok(!cleaned.includes("\u202e"));
  assert.ok(!cleaned.includes("\u0000"));
  assert.ok(cleaned.includes("\\u001b"));
  assert.ok(cleaned.includes("\\u000a"));
  assert.ok(cleaned.includes("\\u202e"));
});

test("CLI --extract runs offline and CLI live mode sets non-zero exit code on warning", async () => {
  await withTempRepo(
    {
      "packaging/containers/arch.Dockerfile": `FROM archlinux:latest@sha256:${DIGEST_A}\n`,
      "packaging/containers/debian.Dockerfile": `FROM debian:trixie@sha256:${DIGEST_B}\n`,
      "packaging/containers/fedora.Dockerfile": `FROM fedora:43@sha256:${DIGEST_C}\n`,
      "packaging/containers/flatpak.Dockerfile": `FROM ghcr.io/flathub-infra/flatpak-github-actions:kde-6.11@sha256:${DIGEST_D}\n`,
      ".github/workflows/ci.yml": `jobs:\n  a:\n    steps:\n      - uses: actions/setup-node@0\n        with:\n          node-version: "24"\n`,
    },
    async (root) => {
      const out = [];
      const err = [];
      const extractCode = await runCli(["--extract", "--root", root], {
        stdout: (s) => out.push(s),
        stderr: (s) => err.push(s),
        fetchImpl: async () => {
          throw new Error("network must not be called in --extract mode");
        },
      });
      assert.equal(extractCode, 0);
      assert.equal(err.length, 0);
      const outText = out.join("\n").toLowerCase();
      assert.ok(outText.includes("debian") && outText.includes("trixie"));
      assert.ok(out.join("\n").includes("flatpak.Dockerfile"));

      const warnOut = [];
      const warnErr = [];
      const liveCode = await runCli(["--root", root, "--now", "2026-11-18"], {
        stdout: (s) => warnOut.push(s),
        stderr: (s) => warnErr.push(s),
        fetchImpl: mockJsonFetch({
          "https://endoflife.date/api/debian.json": () =>
            jsonResponse([
              { cycle: "13", codename: "Trixie", eol: "2028-08-09", extendedSupport: "2030-06-30" },
            ]),
          "https://endoflife.date/api/fedora.json": () =>
            jsonResponse([{ cycle: "43", eol: "2026-12-01" }]),
          "https://endoflife.date/api/nodejs.json": () =>
            jsonResponse([{ cycle: "24", eol: "2028-04-30" }]),
        }),
      });
      assert.equal(liveCode, 1);
      const errText = warnErr.join("\n").toLowerCase();
      assert.ok(errText.includes("fedora 43") && errText.includes("13 days"));

      const aliasCode = await runCli(["--offline-parse-only", "--root", root], {
        stdout: () => {},
        stderr: (s) => err.push(s),
        fetchImpl: async () => {
          throw new Error("network must not be called in --offline-parse-only mode");
        },
      });
      assert.equal(aliasCode, 0);

      const badFlagErr = [];
      const usageCode = await runCli(["--unknown-flag"], {
        stdout: () => {},
        stderr: (s) => badFlagErr.push(s),
      });
      assert.equal(usageCode, 2);
      assert.match(badFlagErr.join("\n"), /usage:/i);
    },
  );
});

test("recovers from transient HTTP 503 on retry and rejects oversized Content-Length immediately", async () => {
  let attempts = 0;
  const recovered = await fetchProductReleases("fedora", {
    retries: 1,
    retryDelayMs: 0,
    fetchImpl: mockJsonFetch({
      "https://endoflife.date/api/fedora.json": () => {
        attempts += 1;
        if (attempts === 1) return new Response("bad gateway", { status: 503 });
        return jsonResponse([{ cycle: "44", eol: "2027-06-02" }]);
      },
    }),
  });
  assert.equal(attempts, 2);
  assert.equal(recovered[0].cycle, "44");

  await assert.rejects(
    () =>
      fetchProductReleases("fedora", {
        maxBytes: 128,
        fetchImpl: mockJsonFetch({
          "https://endoflife.date/api/fedora.json": () =>
            new Response("[]", { status: 200, headers: { "content-length": "999999" } }),
        }),
      }),
    (err) => err instanceof EolCheckError && err.code === "RESPONSE_TOO_LARGE",
  );
});

test("parses pathological repeated inputs in linear time without backtracking", async () => {
  const start = performance.now();
  assert.throws(
    () => parseContainerImage("debian:trixie" + "-slim".repeat(5000) + "!"),
    (err) => err instanceof EolCheckError && err.code === "INVALID_IMAGE_TAG",
  );
  assert.throws(
    () => parseContainerImage("FROM " + "--platform=linux/amd64 ".repeat(5000) + "extra bad"),
    (err) => err instanceof EolCheckError && err.code === "INVALID_DOCKERFILE_FROM",
  );
  await withTempRepo(
    {
      "packaging/containers/arch.Dockerfile": "FROM archlinux:latest\n",
      ".github/workflows/ci.yml": "node-version: " + "24 ".repeat(5000) + "!\n",
    },
    (root) => {
      assert.throws(
        () => extractTargets(root),
        (err) => err instanceof EolCheckError && err.code === "INVALID_NODE_VERSION",
      );
    },
  );
  assert.ok(performance.now() - start < 1000);
});
