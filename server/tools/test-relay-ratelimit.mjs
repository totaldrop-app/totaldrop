// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// node --test server/tools/test-relay-ratelimit.mjs

import { test } from "node:test";
import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { createServer } from "node:http";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import {
  DEFAULT_README,
  compareRuleset,
  fetchLiveRuleset,
  normalizeExpression,
  parseReadmeRule,
  visible,
} from "./check-relay-ratelimit.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const scriptPath = join(here, "check-relay-ratelimit.mjs");

const VALID_README = `# TotalDrop relay on Cloudflare Workers

## Edge rate limit

Change the two together.

| Setting | Value |
| --- | --- |
| Name | TotalDrop relay: new connections per IP |
| Expression | \`(http.host eq "relay.totaldrop.app")\` |
| Counted by | IP (\`ip.src\`, the full address) |
| Rate | 30 requests in 10 seconds |
| Action | Block for 10 seconds |

- Bullet after the table.
`;

function matchingLiveRule(overrides = {}) {
  const { ratelimit, ...rest } = overrides;
  return {
    id: "rule-1",
    enabled: true,
    description: "TotalDrop relay: new connections per IP",
    expression: '(http.host eq "relay.totaldrop.app")',
    action: "block",
    ratelimit: {
      characteristics: ["cf.colo.id", "ip.src"],
      requests_per_period: 30,
      period: 10,
      mitigation_timeout: 10,
      ...ratelimit,
    },
    ...rest,
  };
}

test("visible escapes ASCII controls, DEL, C1 controls, and bidi overrides", () => {
  assert.equal(
    visible("bad\n::error::injected\x1b[31mred\u202e"),
    "bad\\u{a}::error::injected\\u{1b}[31mred\\u{202e}",
  );
});

test("normalizeExpression strips balanced outer parens and preserves quoted spaces", () => {
  assert.equal(
    normalizeExpression('  ((http.host   eq  "relay.totaldrop.app")) '),
    'http.host eq "relay.totaldrop.app"',
  );
  assert.equal(
    normalizeExpression('(http.host eq "a") and (http.request.method eq "GET")'),
    '(http.host eq "a") and (http.request.method eq "GET")',
  );
  assert.equal(
    normalizeExpression('(http.host eq "a)  b")'),
    'http.host eq "a)  b"',
  );
});

test("parseReadmeRule parses both the fixture and the repository README", () => {
  const { rule, problems } = parseReadmeRule(VALID_README);
  assert.deepEqual(problems, []);
  assert.deepEqual(rule, {
    name: "TotalDrop relay: new connections per IP",
    expression: '(http.host eq "relay.totaldrop.app")',
    characteristics: ["ip.src"],
    requestsPerPeriod: 30,
    period: 10,
    action: "block",
    mitigationTimeout: 10,
  });

  const realReadme = readFileSync(DEFAULT_README, "utf8");
  const realParsed = parseReadmeRule(realReadme);
  assert.deepEqual(realParsed.problems, []);
  assert.equal(realParsed.rule.requestsPerPeriod, 30);

  const compound = VALID_README.replace(
    '`(http.host eq "relay.totaldrop.app")`',
    '`(http.host eq "relay.totaldrop.app" and http.request.uri.path wildcard "/v0/relay/*")`',
  ).replace(
    "IP (`ip.src`, the full address)",
    "IP (`cf.colo.id`, `ip.src`, `ip.src`)",
  );
  const compoundParsed = parseReadmeRule(compound);
  assert.deepEqual(compoundParsed.problems, []);
  assert.deepEqual(compoundParsed.rule.characteristics, ["ip.src"]);
});

test("parseReadmeRule rejects missing sections, missing rows, and invalid fields with specific errors", () => {
  assert.deepEqual(parseReadmeRule("# Nothing here").problems, [
    'missing "## Edge rate limit" section',
  ]);

  const missingRate = VALID_README.replace(
    "| Rate | 30 requests in 10 seconds |\n",
    "",
  );
  assert.deepEqual(parseReadmeRule(missingRate).problems, [
    'missing "Rate" row in rate-limit table',
  ]);

  const badValues = VALID_README.replace(
    '`(http.host eq "relay.totaldrop.app")`',
    '`(http.host eq "other-relay.totaldrop.app")`',
  )
    .replace("IP (`ip.src`, the full address)", "IP (`/64`)")
    .replace("30 requests in 10 seconds", "30 per minute")
    .replace("Block for 10 seconds", "Log for 10 seconds");
  const { problems } = parseReadmeRule(badValues);
  assert.equal(problems.length, 4);
  assert.ok(problems.some((p) => p.startsWith('Expression must target "relay.totaldrop.app"')));
  assert.ok(problems.some((p) => p.startsWith("Counted by has invalid Cloudflare characteristic")));
  assert.ok(problems.some((p) => p.startsWith('Rate must look like "<N> requests in <P> seconds"')));
  assert.ok(problems.some((p) => p.startsWith('Action must be "Block"')));
});

test("compareRuleset accepts Cloudflare defaults (omitted enabled, implicit cf.colo.id, null score)", () => {
  const { rule } = parseReadmeRule(VALID_README);
  const liveWithoutEnabled = matchingLiveRule({
    expression: 'http.host eq "relay.totaldrop.app"',
    ratelimit: {
      characteristics: ["cf.colo.id", "ip.src"],
      requests_per_period: 30,
      period: 10,
      mitigation_timeout: 10,
      score_per_period: null,
      score_response_header_name: "",
    },
  });
  delete liveWithoutEnabled.enabled;

  const problems = compareRuleset(rule, {
    rules: [
      {
        id: "unrelated-rule",
        description: "Other host rule",
        expression: '(http.host eq "other-relay.totaldrop.app")',
        action: "block",
        ratelimit: {
          characteristics: ["cf.colo.id", "ip.src"],
          requests_per_period: 5,
          period: 10,
          mitigation_timeout: 10,
        },
      },
      liveWithoutEnabled,
    ],
  });
  assert.deepEqual(problems, []);
});

test("compareRuleset detects deletion (404, empty, or omitted rules), duplicates, controls, and drift", () => {
  const { rule } = parseReadmeRule(VALID_README);

  assert.match(compareRuleset(rule, null)[0], /no http_ratelimit entrypoint/);
  assert.match(compareRuleset(rule, { rules: [] })[0], /no live rate-limiting rule/);
  assert.match(
    compareRuleset(rule, { id: "rs-empty", phase: "http_ratelimit" })[0],
    /no live rate-limiting rule/,
  );

  const duplicate = compareRuleset(rule, {
    rules: [matchingLiveRule(), matchingLiveRule({ id: "rule-2" })],
  });
  assert.ok(
    duplicate.some((p) =>
      p.includes("expected 1 live rate-limiting rule for relay.totaldrop.app, found 2"),
    ),
  );

  const drifted = compareRuleset(rule, {
    rules: [
      matchingLiveRule({
        enabled: false,
        description: "Renamed\n::error::injected",
        action: "log",
        action_parameters: { response: { status_code: 403 } },
        ratelimit: {
          characteristics: ["cf.colo.id", "cf.unique_visitor_id"],
          requests_per_period: 60,
          period: 10,
          mitigation_timeout: 60,
          requests_to_origin: true,
          counting_expression: '(http.response.code eq 500)',
          score_per_period: 100,
        },
      }),
    ],
  });
  assert.ok(drifted.some((p) => p.includes("disabled")));
  assert.ok(drifted.some((p) => p.includes("Renamed\\u{a}::error::injected")));
  assert.ok(drifted.every((p) => !p.includes("\n")));
  assert.ok(drifted.some((p) => p.startsWith("Name:")));
  assert.ok(drifted.some((p) => p.startsWith("Action:")));
  assert.ok(drifted.some((p) => p.startsWith("Status code:")));
  assert.ok(drifted.some((p) => p.startsWith("Counted by:")));
  assert.ok(drifted.some((p) => p.startsWith("Rate:")));
  assert.ok(drifted.some((p) => p.startsWith("Action duration:")));
  assert.ok(drifted.some((p) => p.startsWith("requests_to_origin:")));
  assert.ok(drifted.some((p) => p.startsWith("counting_expression:")));
  assert.ok(drifted.some((p) => p.includes("complexity scoring")));
});

test("fetchLiveRuleset validates token and HTTPS base, resolves zone ID, handles 404, and redacts secrets", async () => {
  const secret = "cf-secret-token-value-do-not-print";
  const zoneId = "0123456789abcdef0123456789abcdef";

  await assert.rejects(
    () => fetchLiveRuleset({ token: "   ", zoneId }),
    /missing Cloudflare read token/,
  );

  await assert.rejects(
    () => fetchLiveRuleset({ token: "bad token\nwith-space", zoneId }),
    /invalid CLOUDFLARE_WAF_READ_TOKEN/,
  );

  await assert.rejects(
    () =>
      fetchLiveRuleset({
        token: secret,
        zoneId,
        apiBase: "http://api.cloudflare.test/client/v4",
        fetchImpl: async () => {
          throw new Error("must not send token over cleartext http");
        },
      }),
    /CLOUDFLARE_API_BASE must use https:\/\//,
  );

  await assert.rejects(
    () =>
      fetchLiveRuleset({
        token: secret,
        zoneId: "../bad",
        fetchImpl: async () => {
          throw new Error("should not be called");
        },
      }),
    /invalid CLOUDFLARE_ZONE_ID/,
  );

  // Successful zone-name lookup -> ZONE_ID_RE validation -> entrypoint fetch (404 -> null),
  // with surrounding whitespace trimmed from `token` before sending Authorization.
  const calls = [];
  const got404 = await fetchLiveRuleset({
    token: `  ${secret}\n`,
    zoneName: "totaldrop.app",
    apiBase: "https://api.cloudflare.test/client/v4/",
    fetchImpl: async (url, init) => {
      calls.push(url);
      assert.equal(init.headers.Authorization, `Bearer ${secret}`);
      assert.equal(init.redirect, "error");
      if (calls.length === 1) {
        assert.equal(
          url,
          "https://api.cloudflare.test/client/v4/zones?name=totaldrop.app",
        );
        return new Response(
          JSON.stringify({
            success: true,
            result: [{ id: zoneId, name: "totaldrop.app" }],
          }),
          { status: 200 },
        );
      }
      assert.equal(
        url,
        `https://api.cloudflare.test/client/v4/zones/${zoneId}/rulesets/phases/http_ratelimit/entrypoint`,
      );
      return new Response("{}", { status: 404 });
    },
  });
  assert.equal(got404, null);
  assert.equal(calls.length, 2);

  // Malformed zone ID returned by `/zones?name=...` is rejected before path interpolation.
  await assert.rejects(
    () =>
      fetchLiveRuleset({
        token: secret,
        zoneName: "totaldrop.app",
        apiBase: "https://api.cloudflare.test/client/v4",
        fetchImpl: async () =>
          new Response(
            JSON.stringify({
              success: true,
              result: [{ id: "../../evil", name: "totaldrop.app" }],
            }),
            { status: 200 },
          ),
      }),
    /invalid CLOUDFLARE_ZONE_ID/,
  );

  // Token and raw control characters are never echoed on API or transport errors.
  await assert.rejects(
    async () => {
      await fetchLiveRuleset({
        token: secret,
        zoneName: "totaldrop.app",
        apiBase: "https://api.cloudflare.test/client/v4",
        fetchImpl: async () =>
          new Response(
            JSON.stringify({
              success: false,
              errors: [
                {
                  code: 10000,
                  message: `Authentication error for ${secret}\n::error::spoof`,
                },
              ],
            }),
            { status: 403 },
          ),
      });
    },
    (err) => {
      assert.ok(!err.message.includes(secret), "error must never leak token");
      assert.ok(!err.message.includes("\n"), "error must escape control characters");
      assert.match(err.message, /HTTP 403 \(10000: Authentication error for \[REDACTED\]/);
      return true;
    },
  );
});

test("CLI verifies exit codes 0 (ok), 1 (live drift or missing file), and 2 (bad flags)", async () => {
  const dir = mkdtempSync(join(tmpdir(), "td-ratelimit-test-"));
  const readme = join(dir, "README.md");
  writeFileSync(readme, VALID_README);
  const zoneId = "fedcba9876543210fedcba9876543210";
  let liveRate = 30;

  const server = createServer((req, res) => {
    if (
      req.url === `/client/v4/zones/${zoneId}/rulesets/phases/http_ratelimit/entrypoint` &&
      req.headers.authorization === "Bearer test-token"
    ) {
      res.writeHead(200, { "content-type": "application/json" });
      res.end(
        JSON.stringify({
          success: true,
          result: {
            rules: [
              matchingLiveRule({
                ratelimit: {
                  characteristics: ["cf.colo.id", "ip.src"],
                  requests_per_period: liveRate,
                  period: 10,
                  mitigation_timeout: 10,
                },
              }),
            ],
          },
        }),
      );
      return;
    }
    res.writeHead(404, { "content-type": "application/json" });
    res.end(JSON.stringify({ success: false, errors: [] }));
  });

  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const execFileAsync = promisify(execFile);
  try {
    const { port } = server.address();
    const relScript = relative(dir, scriptPath);
    const liveEnv = {
      ...process.env,
      CLOUDFLARE_WAF_READ_TOKEN: "test-token",
      CLOUDFLARE_ZONE_ID: zoneId,
      CLOUDFLARE_API_BASE: `http://127.0.0.1:${port}/client/v4`,
    };

    const offlineOut = await execFileAsync(
      process.execPath,
      [relScript, "--readme", readme],
      { cwd: dir, encoding: "utf8" },
    );
    assert.match(offlineOut.stdout, /relay rate-limit README ok/);

    const liveOut = await execFileAsync(
      process.execPath,
      [relScript, "--live", "--readme", readme],
      { cwd: dir, encoding: "utf8", env: liveEnv },
    );
    assert.match(liveOut.stdout, /relay rate-limit live ok/);

    // Drift exits 1 with a specific Rate mismatch message.
    liveRate = 60;
    await assert.rejects(
      () =>
        execFileAsync(
          process.execPath,
          [relScript, "--live", "--readme", readme],
          { cwd: dir, encoding: "utf8", env: liveEnv },
        ),
      (err) => {
        assert.equal(err.code, 1);
        assert.match(err.stderr, /Rate: README says 30 requests in 10 seconds, live rule says 60/);
        return true;
      },
    );

    // Missing README file exits 1 cleanly without an unhandled stack trace.
    await assert.rejects(
      () =>
        execFileAsync(
          process.execPath,
          [relScript, "--readme", join(dir, "missing.md")],
          { cwd: dir, encoding: "utf8" },
        ),
      (err) => {
        assert.equal(err.code, 1);
        assert.match(err.stderr, /ENOENT/);
        assert.doesNotMatch(err.stderr, /\n\s+at /);
        return true;
      },
    );

    // Invalid CLI flags (`--readme --live`, `--unknown`) exit 2.
    for (const badArgs of [
      [relScript, "--readme", "--live"],
      [relScript, "--unknown"],
    ]) {
      await assert.rejects(
        () =>
          execFileAsync(process.execPath, badArgs, {
            cwd: dir,
            encoding: "utf8",
          }),
        (err) => {
          assert.equal(err.code, 2);
          assert.match(err.stderr, /^usage: node server\/tools\/check-relay-ratelimit\.mjs/);
          return true;
        },
      );
    }
  } finally {
    server.close();
    rmSync(dir, { recursive: true, force: true });
  }
});
