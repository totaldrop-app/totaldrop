// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Tests for tools/dismiss-scorecard-alerts.mjs:
//   node --test tools/test-scorecard-dismiss.mjs

import assert from "node:assert/strict";
import http from "node:http";
import test from "node:test";

import {
  MAINTAINED_CUTOFF,
  decideAlertAction,
  decideDismissal,
  dismissDocumentedAlerts,
  nextLink,
  parseApiBase,
  visible,
} from "./dismiss-scorecard-alerts.mjs";

function branchAlert(lines, overrides = {}) {
  return {
    number: 6,
    state: "open",
    rule: { id: "BranchProtectionID" },
    most_recent_instance: {
      ref: "refs/heads/main",
      message: { text: lines.join("\n") },
    },
    ...overrides,
  };
}

test("decideDismissal accepts exact documented two-maintainer Branch-Protection warnings", () => {
  const alert = branchAlert([
    "score is 4: branch protection is not maximal on development and all release branches:",
    "Warn: required approving review count is 1 on branch 'main'",
    "Warn: 'last push approval' is disabled on branch 'main'",
    "Warn: 'up-to-date branches' is disabled on branch 'main'",
    "Click Remediation section below to solve this issue",
  ]);
  const decision = decideDismissal(alert, new Date("2026-11-17T00:00:00Z"));
  assert.equal(decision?.reason, "won't fix");
  assert.match(
    decision?.comment ?? "",
    /GOVERNANCE\.md#branch-protection-and-scorecard/,
  );
});

test("decideDismissal refuses Branch-Protection if any undocumented warning appears", () => {
  const withAdminBypass = branchAlert([
    "score is 3: branch protection is not maximal on development and all release branches:",
    "Warn: 'branch protection settings apply to administrators' is disabled on branch 'main'",
    "Warn: required approving review count is 1 on branch 'main'",
    "Click Remediation section below to solve this issue",
  ]);
  assert.equal(decideDismissal(withAdminBypass), null);

  const withoutWarnings = branchAlert([
    "score is 4: branch protection is not maximal on development and all release branches:",
    "Click Remediation section below to solve this issue",
  ]);
  assert.equal(decideDismissal(withoutWarnings), null);

  const nonMainRef = branchAlert(
    [
      "score is 4: branch protection is not maximal on development and all release branches:",
      "Warn: required approving review count is 1 on branch 'main'",
    ],
    {
      most_recent_instance: {
        ref: "refs/heads/release",
        message: {
          text: "Warn: required approving review count is 1 on branch 'main'",
        },
      },
    },
  );
  assert.equal(decideDismissal(nonMainRef), null);
});

test("decideAlertAction reopens previously dismissed alerts when regressions appear", () => {
  const regressedBranch = branchAlert(
    [
      "score is 3: branch protection is not maximal on development and all release branches:",
      "Warn: 'branch protection settings apply to administrators' is disabled on branch 'main'",
      "Warn: required approving review count is 1 on branch 'main'",
    ],
    {
      state: "dismissed",
      dismissed_comment:
        "Intentional two-maintainer ruleset settings on main (#318).",
    },
  );
  assert.deepEqual(
    decideAlertAction(regressedBranch, new Date("2026-11-17T00:00:00Z")),
    { action: "reopen" },
  );

  const stillExemptBranch = branchAlert(
    [
      "score is 4: branch protection is not maximal on development and all release branches:",
      "Warn: required approving review count is 1 on branch 'main'",
      "Warn: 'last push approval' is disabled on branch 'main'",
      "Warn: 'up-to-date branches' is disabled on branch 'main'",
    ],
    { state: "dismissed" },
  );
  assert.deepEqual(
    decideAlertAction(stillExemptBranch, new Date("2026-11-17T00:00:00Z")),
    { action: "keep_dismissed" },
  );

  const expiredMaintained = {
    number: 12,
    state: "dismissed",
    dismissed_comment: "Repository opened on 2026-10-27 (#318).",
    rule: { id: "MaintainedID" },
    most_recent_instance: {
      ref: "refs/heads/main",
      message: { text: "Warn: repo was created less than 90 days ago" },
    },
  };
  assert.deepEqual(decideAlertAction(expiredMaintained, MAINTAINED_CUTOFF), {
    action: "reopen",
  });
});

test("decideDismissal handles MaintainedID only for <90-day age messages before 2027-01-26", () => {
  for (const ageText of [
    "score is 0: repo was created less than 90 days ago\nWarn: project is 20 days old",
    "score is 0: project was created in last 90 days",
    "score is 0: project is less than 90 days old",
  ]) {
    const ageAlert = {
      number: 12,
      state: "open",
      rule: { id: "MaintainedID" },
      most_recent_instance: {
        ref: "refs/heads/main",
        message: { text: ageText },
      },
    };
    assert.equal(
      decideDismissal(ageAlert, new Date("2027-01-25T23:59:59Z"))?.reason,
      "won't fix",
    );
    assert.equal(decideDismissal(ageAlert, MAINTAINED_CUTOFF), null);
  }

  const archivedAlert = {
    number: 12,
    state: "open",
    rule: { id: "MaintainedID" },
    most_recent_instance: {
      ref: "refs/heads/main",
      message: { text: "score is 0: Warn: repo is marked as archived" },
    },
  };
  assert.equal(
    decideDismissal(archivedAlert, new Date("2026-11-17T00:00:00Z")),
    null,
  );

  const inactiveAlert = {
    number: 12,
    state: "open",
    rule: { id: "MaintainedID" },
    most_recent_instance: {
      ref: "refs/heads/main",
      message: {
        text: "score is 0: 0 commit(s) and 0 issue activity found in the last 90 days",
      },
    },
  };
  assert.equal(
    decideDismissal(inactiveAlert, new Date("2026-11-17T00:00:00Z")),
    null,
  );
});

test("decideDismissal leaves unrelated Scorecard rules open and ignores unrelated dismissals", () => {
  const pinned = {
    number: 15,
    state: "open",
    rule: { id: "PinnedDependenciesID" },
    most_recent_instance: {
      ref: "refs/heads/main",
      message: { text: "Warn: GitHub-owned GitHubAction not pinned by hash" },
    },
  };
  assert.equal(decideDismissal(pinned, new Date("2026-11-17T00:00:00Z")), null);
  assert.deepEqual(
    decideAlertAction(
      {
        ...pinned,
        state: "dismissed",
        dismissed_comment: "Manual dismissal unrelated to branch rules.",
      },
      new Date("2026-11-17T00:00:00Z"),
    ),
    { action: "keep_dismissed" },
  );
});

test("parseApiBase and nextLink enforce origin, HTTPS, query/fragment, and credential rules", () => {
  assert.throws(
    () => parseApiBase("http://api.github.com"),
    /must use https/,
  );
  assert.throws(
    () => parseApiBase("https://user:secret@api.github.com"),
    /must not embed credentials/,
  );
  assert.throws(
    () => parseApiBase("https://api.github.com?foo=1"),
    /must not include query or fragment/,
  );
  assert.equal(
    parseApiBase("http://127.0.0.1:8123/").toString(),
    "http://127.0.0.1:8123/",
  );

  const next = nextLink(
    '<https://api.github.com/repos/o/r/code-scanning/alerts?filter=a,b&page=2>; rel="next", <https://api.github.com/repos/o/r/code-scanning/alerts?page=5>; rel="last"',
    "https://api.github.com",
  );
  assert.equal(
    next?.toString(),
    "https://api.github.com/repos/o/r/code-scanning/alerts?filter=a,b&page=2",
  );
  assert.throws(
    () =>
      nextLink(
        '<https://user:pass@api.github.com/repos/o/r/alerts?page=2>; rel="next"',
        "https://api.github.com",
      ),
    /embedded credentials/,
  );
});

test("visible escapes control, bidi, and line-separator characters", () => {
  assert.equal(
    visible("bad\u202e\nline\u2028\u0007"),
    "bad\\u202e\\u000aline\\u2028\\u0007",
  );
});

test("dismissDocumentedAlerts rejects dot-segment repos and malformed tokens without echoing secrets", async () => {
  for (const badRepo of ["../other", "totaldrop-app/..", ".", "a/b/c"]) {
    await assert.rejects(
      () =>
        dismissDocumentedAlerts({
          repo: badRepo,
          token: "ghs_validToken123",
          log: () => {},
        }),
      /Invalid GITHUB_REPOSITORY/,
    );
  }

  const secret = "ghs_TopSecretValue999";
  for (const badToken of ["", `${secret}\u0000`, ` ${secret}`, `${secret}\n`]) {
    await assert.rejects(
      () =>
        dismissDocumentedAlerts({
          repo: "totaldrop-app/totaldrop",
          token: badToken,
          log: () => {},
        }),
      (err) => {
        assert.match(err.message, /Missing or invalid GITHUB_TOKEN/);
        assert.ok(!err.message.includes(secret));
        return true;
      },
    );
  }
});

test("dismissDocumentedAlerts paginates, dismisses documented alerts, reopens regressed ones, and keeps unexpected ones open", async () => {
  const patched = [];
  const logs = [];
  let port = 0;
  const server = http.createServer((req, res) => {
    assert.equal(req.headers.authorization, "Bearer test-token-123");
    if (
      req.method === "GET" &&
      req.url ===
        "/repos/totaldrop-app/totaldrop/code-scanning/alerts?tool_name=Scorecard&state=open&per_page=100"
    ) {
      res.writeHead(200, {
        "Content-Type": "application/json",
        Link: `<http://127.0.0.1:${port}/repos/totaldrop-app/totaldrop/code-scanning/alerts?tool_name=Scorecard&state=open&per_page=100&page=2>; rel="next"`,
      });
      res.end(
        JSON.stringify([
          branchAlert([
            "score is 4: branch protection is not maximal on development and all release branches:",
            "Warn: required approving review count is 1 on branch 'main'",
            "Warn: 'last push approval' is disabled on branch 'main'",
            "Warn: 'up-to-date branches' is disabled on branch 'main'",
            "Click Remediation section below to solve this issue",
          ]),
        ]),
      );
      return;
    }
    if (
      req.method === "GET" &&
      req.url ===
        "/repos/totaldrop-app/totaldrop/code-scanning/alerts?tool_name=Scorecard&state=open&per_page=100&page=2"
    ) {
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(
        JSON.stringify([
          {
            number: 99,
            state: "open",
            rule: { id: "TokenPermissionsID\u202e" },
            most_recent_instance: {
              ref: "refs/heads/main",
              message: { text: "Warn: top-level permission write-all" },
            },
          },
        ]),
      );
      return;
    }
    if (
      req.method === "GET" &&
      req.url ===
        "/repos/totaldrop-app/totaldrop/code-scanning/alerts?tool_name=Scorecard&state=dismissed&per_page=100"
    ) {
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(
        JSON.stringify([
          {
            number: 12,
            state: "dismissed",
            dismissed_comment: "Repository opened on 2026-10-27 (#318).",
            rule: { id: "MaintainedID" },
            most_recent_instance: {
              ref: "refs/heads/main",
              message: { text: "Warn: repo is marked as archived" },
            },
          },
        ]),
      );
      return;
    }
    if (
      req.method === "PATCH" &&
      (req.url === "/repos/totaldrop-app/totaldrop/code-scanning/alerts/6" ||
        req.url === "/repos/totaldrop-app/totaldrop/code-scanning/alerts/12")
    ) {
      let raw = "";
      req.setEncoding("utf8");
      req.on("data", (chunk) => {
        raw += chunk;
      });
      req.on("end", () => {
        patched.push({ url: req.url, body: JSON.parse(raw) });
        res.writeHead(200, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ ok: true }));
      });
      return;
    }
    res.writeHead(404);
    res.end("unexpected route");
  });

  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  port = server.address().port;
  try {
    const summary = await dismissDocumentedAlerts({
      repo: "totaldrop-app/totaldrop",
      token: "test-token-123",
      apiBase: `http://127.0.0.1:${port}`,
      now: new Date("2026-11-17T00:00:00Z"),
      log: (line) => logs.push(line),
    });
    assert.equal(summary.checked, 3);
    assert.deepEqual(summary.dismissed, [
      { number: 6, ruleId: "BranchProtectionID" },
    ]);
    assert.deepEqual(summary.reopened, [
      { number: 12, ruleId: "MaintainedID" },
    ]);
    assert.deepEqual(summary.kept, [
      { number: 99, ruleId: "TokenPermissionsID\\u202e" },
    ]);
    assert.equal(patched.length, 2);
    assert.equal(patched[0].body.state, "dismissed");
    assert.equal(patched[0].body.dismissed_reason, "won't fix");
    assert.deepEqual(patched[1].body, { state: "open" });
    assert.ok(logs.some((l) => l.includes("TokenPermissionsID\\u202e")));
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});

test("dismissDocumentedAlerts handles HTTP 404, non-2xx GET/PATCH errors, cross-origin links, and MAX_PAGES overflow", async () => {
  // 1. HTTP 404 empty analysis returns cleanly.
  const server404 = http.createServer((_req, res) => {
    res.writeHead(404);
    res.end("no analysis found");
  });
  await new Promise((resolve) => server404.listen(0, "127.0.0.1", resolve));
  try {
    const summary = await dismissDocumentedAlerts({
      repo: "totaldrop-app/totaldrop",
      token: "test-token-123",
      apiBase: `http://127.0.0.1:${server404.address().port}`,
      log: () => {},
    });
    assert.deepEqual(summary, {
      checked: 0,
      dismissed: [],
      reopened: [],
      kept: [],
    });
  } finally {
    await new Promise((resolve) => server404.close(resolve));
  }

  // 2. HTTP 500 on GET surfaces status code.
  const server500 = http.createServer((_req, res) => {
    res.writeHead(500);
    res.end("upstream failure");
  });
  await new Promise((resolve) => server500.listen(0, "127.0.0.1", resolve));
  try {
    await assert.rejects(
      () =>
        dismissDocumentedAlerts({
          repo: "totaldrop-app/totaldrop",
          token: "test-token-123",
          apiBase: `http://127.0.0.1:${server500.address().port}`,
          log: () => {},
        }),
      /Failed to list Scorecard alerts: HTTP 500/,
    );
  } finally {
    await new Promise((resolve) => server500.close(resolve));
  }

  // 3. HTTP 403 on PATCH surfaces status code.
  const serverPatch403 = http.createServer((req, res) => {
    if (req.method === "GET") {
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(
        JSON.stringify([
          branchAlert([
            "Warn: required approving review count is 1 on branch 'main'",
          ]),
        ]),
      );
      return;
    }
    res.writeHead(403);
    res.end("Resource not accessible by integration");
  });
  await new Promise((resolve) =>
    serverPatch403.listen(0, "127.0.0.1", resolve),
  );
  try {
    await assert.rejects(
      () =>
        dismissDocumentedAlerts({
          repo: "totaldrop-app/totaldrop",
          token: "test-token-123",
          apiBase: `http://127.0.0.1:${serverPatch403.address().port}`,
          log: () => {},
        }),
      /Failed to dismiss Scorecard alert #6 \(BranchProtectionID\): HTTP 403/,
    );
  } finally {
    await new Promise((resolve) => serverPatch403.close(resolve));
  }

  // 4. Cross-origin Link header rejected.
  const serverEvilLink = http.createServer((_req, res) => {
    res.writeHead(200, {
      "Content-Type": "application/json",
      Link: '<https://evil.example/alerts?page=2>; rel="next"',
    });
    res.end(JSON.stringify([]));
  });
  await new Promise((resolve) =>
    serverEvilLink.listen(0, "127.0.0.1", resolve),
  );
  try {
    await assert.rejects(
      () =>
        dismissDocumentedAlerts({
          repo: "totaldrop-app/totaldrop",
          token: "test-token-123",
          apiBase: `http://127.0.0.1:${serverEvilLink.address().port}`,
          log: () => {},
        }),
      /Refusing cross-origin pagination link/,
    );
  } finally {
    await new Promise((resolve) => serverEvilLink.close(resolve));
  }

  // 5. Endless Link loop rejected at MAX_PAGES.
  let loopPort = 0;
  const serverLoop = http.createServer((_req, res) => {
    res.writeHead(200, {
      "Content-Type": "application/json",
      Link: `<http://127.0.0.1:${loopPort}/repos/totaldrop-app/totaldrop/code-scanning/alerts?page=next>; rel="next"`,
    });
    res.end(JSON.stringify([]));
  });
  await new Promise((resolve) => serverLoop.listen(0, "127.0.0.1", resolve));
  loopPort = serverLoop.address().port;
  try {
    await assert.rejects(
      () =>
        dismissDocumentedAlerts({
          repo: "totaldrop-app/totaldrop",
          token: "test-token-123",
          apiBase: `http://127.0.0.1:${loopPort}`,
          log: () => {},
        }),
      /Pagination exceeded 10 pages/,
    );
  } finally {
    await new Promise((resolve) => serverLoop.close(resolve));
  }
});
