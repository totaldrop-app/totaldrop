// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Check server/relay-worker/README.md's "## Edge rate limit" table, and with
// --live compare it against the live Cloudflare zone http_ratelimit ruleset
// without sending burst traffic to relay.totaldrop.app (#526).
//
//   node server/tools/check-relay-ratelimit.mjs
//   node server/tools/check-relay-ratelimit.mjs --live
//
// Environment for --live:
//   CLOUDFLARE_WAF_READ_TOKEN
//       Read-only token with Zone -> Zone WAF: Read on totaldrop.app (plus
//       Zone -> Zone: Read only when CLOUDFLARE_ZONE_ID is omitted). Kept
//       separate from CLOUDFLARE_API_TOKEN, which stays in deploy.yml only.
//   CLOUDFLARE_ZONE_ID
//       32-hex-digit zone ID for totaldrop.app. When omitted, totaldrop.app
//       (or CLOUDFLARE_ZONE_NAME) is looked up via GET /zones?name=...

import { readFileSync, realpathSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const DEFAULT_README = join(
  dirname(fileURLToPath(import.meta.url)),
  "../relay-worker/README.md",
);
const DEFAULT_API_BASE = "https://api.cloudflare.com/client/v4";
const DEFAULT_ZONE_NAME = "totaldrop.app";
const RELAY_HOST = "relay.totaldrop.app";
const RELAY_HOST_RE = /(^|[^A-Za-z0-9.-])relay\.totaldrop\.app([^A-Za-z0-9.-]|$)/;
const CHARACTERISTIC_RE = /^[a-z][a-z0-9_.]*$/;
const ZONE_ID_RE = /^[0-9a-f]{32}$/i;
const TOKEN_RE = /^[\x21-\x7e]+$/;
const TIMEOUT_MS = 10_000;

/** Make control and bidi-override characters visible before writing outside text to a terminal or CI log. */
export function visible(value) {
  return String(value ?? "").replace(
    /[\u0000-\u001f\u007f-\u009f\u061c\u200e\u200f\u202a-\u202e\u2066-\u2069]/gu,
    (ch) => `\\u{${ch.codePointAt(0).toString(16)}}`,
  );
}

/** Strip surrounding whitespace, balanced outer parentheses, and unquoted multi-space runs. */
export function normalizeExpression(raw) {
  let s = String(raw ?? "").trim();
  while (s.startsWith("(") && s.endsWith(")") && outerParensWrapAll(s)) {
    s = s.slice(1, -1).trim();
  }
  return collapseOutsideQuotes(s);
}

function collapseOutsideQuotes(s) {
  let out = "";
  let inQuotes = false;
  let pendingSpace = false;
  for (let i = 0; i < s.length; i++) {
    const ch = s[i];
    if (inQuotes) {
      out += ch;
      if (ch === "\\" && i + 1 < s.length) {
        out += s[++i];
        continue;
      }
      if (ch === '"') inQuotes = false;
      continue;
    }
    if (/\s/.test(ch)) {
      pendingSpace = out.length > 0;
      continue;
    }
    if (pendingSpace) {
      out += " ";
      pendingSpace = false;
    }
    out += ch;
    if (ch === '"') inQuotes = true;
  }
  return out;
}

function outerParensWrapAll(s) {
  let depth = 0;
  let inQuotes = false;
  for (let i = 0; i < s.length; i++) {
    const ch = s[i];
    if (inQuotes) {
      if (ch === "\\" && i + 1 < s.length) {
        i++;
        continue;
      }
      if (ch === '"') inQuotes = false;
      continue;
    }
    if (ch === '"') {
      inQuotes = true;
    } else if (ch === "(") {
      depth++;
    } else if (ch === ")") {
      depth--;
      if (depth === 0 && i < s.length - 1) return false;
      if (depth < 0) return false;
    }
  }
  return depth === 0 && !inQuotes;
}

function stripInlineCode(cell) {
  const s = cell.trim();
  return s.startsWith("`") && s.endsWith("`") && s.length >= 2
    ? s.slice(1, -1).trim()
    : s;
}

/** Parse the `## Edge rate limit` Markdown table in `server/relay-worker/README.md`. */
export function parseReadmeRule(markdown) {
  const problems = [];
  const lines = String(markdown ?? "").split(/\r?\n/);
  const headingIdx = lines.findIndex((l) => /^##\s+Edge rate limit\s*$/.test(l));
  if (headingIdx === -1) {
    return { rule: null, problems: ['missing "## Edge rate limit" section'] };
  }

  let endIdx = lines.length;
  for (let i = headingIdx + 1; i < lines.length; i++) {
    if (/^##\s+/.test(lines[i])) {
      endIdx = i;
      break;
    }
  }

  const rows = new Map();
  let inTable = false;
  for (let i = headingIdx + 1; i < endIdx; i++) {
    const line = lines[i].trim();
    if (!line.startsWith("|")) {
      if (inTable) break;
      continue;
    }
    inTable = true;
    if (!line.endsWith("|")) {
      problems.push(`malformed table row: ${visible(line)}`);
      continue;
    }
    const cells = line
      .slice(1, -1)
      .split("|")
      .map((c) => c.trim());
    if (cells.length !== 2) {
      problems.push(
        `expected 2 columns in rate-limit table, got ${cells.length}: ${visible(line)}`,
      );
      continue;
    }
    const [key, value] = cells;
    if (key === "Setting" && value === "Value") continue;
    if (/^-+$/.test(key) && /^-+$/.test(value)) continue;
    if (rows.has(key)) {
      problems.push(`duplicate "${visible(key)}" row in rate-limit table`);
      continue;
    }
    rows.set(key, value);
  }

  const required = ["Name", "Expression", "Counted by", "Rate", "Action"];
  for (const key of required) {
    if (!rows.has(key)) problems.push(`missing "${key}" row in rate-limit table`);
  }
  for (const key of rows.keys()) {
    if (!required.includes(key)) {
      problems.push(`unexpected "${visible(key)}" row in rate-limit table`);
    }
  }
  if (problems.length) return { rule: null, problems };

  const name = rows.get("Name").trim();
  if (!name) problems.push("Name must not be empty");

  const expression = stripInlineCode(rows.get("Expression"));
  if (!expression || !RELAY_HOST_RE.test(expression)) {
    problems.push(
      `Expression must target "${RELAY_HOST}", got: ${visible(expression)}`,
    );
  }

  const countedBy = rows.get("Counted by");
  const rawCharacteristics = [...countedBy.matchAll(/`([^`]+)`/g)].map((m) =>
    m[1].trim(),
  );
  if (!rawCharacteristics.length) {
    problems.push(
      `Counted by must name at least one backticked Cloudflare field (e.g. \`ip.src\`), got: ${visible(countedBy)}`,
    );
  } else {
    for (const field of rawCharacteristics) {
      if (!CHARACTERISTIC_RE.test(field)) {
        problems.push(
          `Counted by has invalid Cloudflare characteristic \`${visible(field)}\` in: ${visible(countedBy)}`,
        );
      }
    }
  }
  const characteristics = [
    ...new Set(rawCharacteristics.filter((c) => c !== "cf.colo.id")),
  ].sort();
  if (rawCharacteristics.length > 0 && characteristics.length === 0 && problems.length === 0) {
    problems.push(
      `Counted by must include at least one non-colo Cloudflare characteristic (e.g. \`ip.src\`), got: ${visible(countedBy)}`,
    );
  }

  const rateText = rows.get("Rate");
  const rateMatch = /^(\d+)\s+requests?\s+in\s+(\d+)\s+seconds?$/i.exec(
    rateText,
  );
  let requestsPerPeriod = 0;
  let period = 0;
  if (!rateMatch) {
    problems.push(
      `Rate must look like "<N> requests in <P> seconds", got: ${visible(rateText)}`,
    );
  } else {
    requestsPerPeriod = Number(rateMatch[1]);
    period = Number(rateMatch[2]);
    if (requestsPerPeriod <= 0 || period <= 0) {
      problems.push(
        `Rate values must be positive integers, got: ${visible(rateText)}`,
      );
    }
  }

  const actionText = rows.get("Action");
  const actionMatch = /^([A-Za-z_ ]+?)\s+for\s+(\d+)\s+seconds?$/i.exec(
    actionText,
  );
  let action = "";
  let mitigationTimeout = 0;
  if (!actionMatch) {
    problems.push(
      `Action must look like "Block for <M> seconds", got: ${visible(actionText)}`,
    );
  } else {
    action = actionMatch[1].trim().toLowerCase().replace(/\s+/g, "_");
    mitigationTimeout = Number(actionMatch[2]);
    if (action !== "block" || mitigationTimeout <= 0) {
      problems.push(
        `Action must be "Block" with a positive duration in seconds, got: ${visible(actionText)}`,
      );
    }
  }

  if (problems.length) return { rule: null, problems };
  return {
    rule: {
      name,
      expression,
      characteristics,
      requestsPerPeriod,
      period,
      action,
      mitigationTimeout,
    },
    problems: [],
  };
}

/** Compare the parsed README rule with Cloudflare's `http_ratelimit` entrypoint ruleset. */
export function compareRuleset(expected, ruleset) {
  if (ruleset === null) {
    return [
      "live zone has no http_ratelimit entrypoint ruleset (rule missing or deleted)",
    ];
  }
  if (!ruleset || typeof ruleset !== "object") {
    return ["live ruleset payload is not an object"];
  }
  // When all rules in an existing phase entrypoint ruleset are deleted,
  // Cloudflare's Go serializer (`rules,omitempty`) may omit `rules`.
  const rules =
    ruleset.rules === undefined &&
    (typeof ruleset.id === "string" || ruleset.phase === "http_ratelimit")
      ? []
      : ruleset.rules;
  if (!Array.isArray(rules)) {
    return ["live ruleset payload has no rules array"];
  }

  const wantExpr = normalizeExpression(expected.expression);
  const candidates = rules.filter((r) => {
    const desc = String(r?.description ?? "").trim();
    const expr = normalizeExpression(r?.expression ?? "");
    return (
      desc === expected.name ||
      expr === wantExpr ||
      RELAY_HOST_RE.test(expr)
    );
  });

  if (candidates.length === 0) {
    return [
      `no live rate-limiting rule matches "${visible(expected.name)}" or \`${visible(expected.expression)}\``,
    ];
  }

  const problems = [];
  if (candidates.length > 1) {
    problems.push(
      `expected 1 live rate-limiting rule for ${RELAY_HOST}, found ${candidates.length}`,
    );
  }

  const live =
    candidates.find((r) => String(r?.description ?? "").trim() === expected.name) ??
    candidates[0];

  if (live.enabled !== undefined && live.enabled !== true) {
    problems.push(
      `live rule "${visible(live.description ?? live.id ?? "")}" is disabled`,
    );
  }
  const liveDesc = String(live.description ?? "").trim();
  if (liveDesc !== expected.name) {
    problems.push(
      `Name: README says "${visible(expected.name)}", live rule says "${visible(liveDesc)}"`,
    );
  }
  const liveExpr = String(live.expression ?? "").trim();
  if (normalizeExpression(liveExpr) !== wantExpr) {
    problems.push(
      `Expression: README says \`${visible(expected.expression)}\`, live rule says \`${visible(liveExpr)}\``,
    );
  }
  if (live.action !== expected.action) {
    problems.push(
      `Action: README says "${visible(expected.action)}", live rule says "${visible(live.action ?? "")}"`,
    );
  }
  const customStatus = live.action_parameters?.response?.status_code;
  if (customStatus !== undefined && customStatus !== 429) {
    problems.push(
      `Status code: expected HTTP 429, live rule sets ${visible(customStatus)}`,
    );
  }

  const rl = live.ratelimit;
  if (!rl || typeof rl !== "object") {
    problems.push("live rule has no ratelimit object");
    return problems;
  }

  if (!Array.isArray(rl.characteristics)) {
    problems.push("live rule ratelimit.characteristics is missing");
  } else {
    // Cloudflare adds cf.colo.id behind the scenes to every dashboard rule.
    const nonColo = [
      ...new Set(
        rl.characteristics.map(String).filter((c) => c !== "cf.colo.id"),
      ),
    ].sort();
    const wantChars = [
      ...new Set(
        expected.characteristics.map(String).filter((c) => c !== "cf.colo.id"),
      ),
    ].sort();
    if (
      nonColo.length !== wantChars.length ||
      nonColo.some((c, i) => c !== wantChars[i])
    ) {
      problems.push(
        `Counted by: README says ${visible(wantChars.join(", "))}, live rule says ${visible(nonColo.join(", ") || "(none)")}`,
      );
    }
  }

  if (
    rl.requests_per_period !== expected.requestsPerPeriod ||
    rl.period !== expected.period
  ) {
    problems.push(
      `Rate: README says ${expected.requestsPerPeriod} requests in ${expected.period} seconds, live rule says ${visible(rl.requests_per_period)} requests in ${visible(rl.period)} seconds`,
    );
  }

  if (rl.mitigation_timeout !== expected.mitigationTimeout) {
    problems.push(
      `Action duration: README says ${expected.mitigationTimeout} seconds, live rule says ${visible(rl.mitigation_timeout)} seconds`,
    );
  }

  if (rl.requests_to_origin === true) {
    problems.push(
      "requests_to_origin: live rule sets true (must count edge requests too)",
    );
  }

  if (
    rl.counting_expression &&
    normalizeExpression(rl.counting_expression) !== wantExpr
  ) {
    problems.push(
      `counting_expression: live rule overrides counting with \`${visible(rl.counting_expression)}\``,
    );
  }

  if (rl.score_per_period != null || Boolean(rl.score_response_header_name)) {
    problems.push(
      "live rule uses complexity scoring instead of request counting",
    );
  }

  return problems;
}

async function apiGet(url, token, fetchImpl) {
  return fetchImpl(url, {
    method: "GET",
    redirect: "error",
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/json",
    },
    signal: AbortSignal.timeout(TIMEOUT_MS),
  });
}

function formatApiErrors(body) {
  if (!Array.isArray(body?.errors) || body.errors.length === 0) return "";
  return body.errors
    .map(
      (e) =>
        `${visible(e?.code ?? "?")}: ${visible(e?.message ?? "unknown error")}`,
    )
    .join("; ");
}

function validateApiBase(rawBase) {
  const base = String(rawBase ?? "").trim().replace(/\/+$/, "");
  let parsed;
  try {
    parsed = new URL(base);
  } catch {
    throw new Error(`invalid CLOUDFLARE_API_BASE URL: ${visible(base)}`);
  }
  if (parsed.username || parsed.password) {
    throw new Error("CLOUDFLARE_API_BASE must not contain URL credentials");
  }
  const isLoopbackHttp =
    parsed.protocol === "http:" &&
    (parsed.hostname === "127.0.0.1" ||
      parsed.hostname === "[::1]" ||
      parsed.hostname === "localhost");
  if (parsed.protocol !== "https:" && !isLoopbackHttp) {
    throw new Error(
      "CLOUDFLARE_API_BASE must use https:// (or http:// on loopback for tests)",
    );
  }
  return base;
}

/** Fetch the live `http_ratelimit` entrypoint ruleset from Cloudflare's API. */
export async function fetchLiveRuleset({
  token,
  zoneId,
  zoneName = DEFAULT_ZONE_NAME,
  apiBase = DEFAULT_API_BASE,
  fetchImpl = fetch,
}) {
  const cleanToken = String(token ?? "").trim();
  if (!cleanToken) {
    throw new Error(
      "missing Cloudflare read token (set CLOUDFLARE_WAF_READ_TOKEN)",
    );
  }
  if (!TOKEN_RE.test(cleanToken)) {
    throw new Error(
      "invalid CLOUDFLARE_WAF_READ_TOKEN (contains whitespace or non-ASCII characters)",
    );
  }

  const base = validateApiBase(apiBase);
  try {
    let resolvedZoneId = String(zoneId ?? "").trim();
    if (!resolvedZoneId) {
      const zonesUrl = `${base}/zones?name=${encodeURIComponent(zoneName)}`;
      const zRes = await apiGet(zonesUrl, cleanToken, fetchImpl);
      const zBody = await zRes.json().catch(() => null);
      if (!zRes.ok || zBody?.success !== true) {
        const detail = formatApiErrors(zBody);
        throw new Error(
          `GET /zones?name=${visible(zoneName)}: HTTP ${zRes.status}${detail ? ` (${detail})` : ""}`,
        );
      }
      const found = Array.isArray(zBody.result)
        ? zBody.result.find((z) => z?.name === zoneName)
        : null;
      if (!found?.id) {
        throw new Error(
          `zone "${visible(zoneName)}" not found in Cloudflare account`,
        );
      }
      resolvedZoneId = String(found.id).trim();
    }
    if (!ZONE_ID_RE.test(resolvedZoneId)) {
      throw new Error(
        "invalid CLOUDFLARE_ZONE_ID (expected 32 hex characters)",
      );
    }

    const url = `${base}/zones/${resolvedZoneId}/rulesets/phases/http_ratelimit/entrypoint`;
    const res = await apiGet(url, cleanToken, fetchImpl);
    if (res.status === 404) return null;
    const body = await res.json().catch(() => null);
    if (!res.ok || body?.success !== true || !body?.result) {
      const detail = formatApiErrors(body);
      throw new Error(
        `GET /zones/<id>/rulesets/phases/http_ratelimit/entrypoint: HTTP ${res.status}${detail ? ` (${detail})` : ""}`,
      );
    }
    return body.result;
  } catch (err) {
    const rawMsg = err instanceof Error ? err.message : String(err);
    const redacted = visible(rawMsg.replaceAll(cleanToken, "[REDACTED]"));
    throw new Error(redacted);
  }
}

function usageError() {
  console.error(
    "usage: node server/tools/check-relay-ratelimit.mjs [--live] [--readme <path>]",
  );
  process.exit(2);
}

async function main() {
  const args = process.argv.slice(2);
  let live = false;
  let readmePath = DEFAULT_README;
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--live") {
      live = true;
    } else if (args[i] === "--readme") {
      const next = args[i + 1];
      if (!next || next.startsWith("-")) usageError();
      readmePath = next;
      i++;
    } else {
      usageError();
    }
  }

  let markdown;
  try {
    markdown = readFileSync(readmePath, "utf8");
  } catch (e) {
    console.error(`${visible(readmePath)}: ${visible(e.message)}`);
    process.exit(1);
  }

  const { rule, problems } = parseReadmeRule(markdown);
  if (problems.length) {
    console.error(`${visible(readmePath)}: invalid Edge rate limit table:`);
    for (const p of problems) console.error(`  - ${p}`);
    process.exit(1);
  }

  if (!live) {
    console.log(
      `relay rate-limit README ok: "${visible(rule.name)}" (${rule.requestsPerPeriod} req / ${rule.period}s per ${rule.characteristics.join("+")}, ${rule.action} ${rule.mitigationTimeout}s)`,
    );
    return;
  }

  let ruleset;
  try {
    ruleset = await fetchLiveRuleset({
      token: process.env.CLOUDFLARE_WAF_READ_TOKEN,
      zoneId: process.env.CLOUDFLARE_ZONE_ID,
      zoneName: process.env.CLOUDFLARE_ZONE_NAME || DEFAULT_ZONE_NAME,
      apiBase: process.env.CLOUDFLARE_API_BASE || DEFAULT_API_BASE,
    });
  } catch (e) {
    console.error(`relay rate-limit API check failed: ${visible(e.message)}`);
    process.exit(1);
  }

  const drift = compareRuleset(rule, ruleset);
  if (drift.length) {
    console.error(
      "relay rate-limit drift between live Cloudflare zone and server/relay-worker/README.md:",
    );
    for (const d of drift) console.error(`  - ${d}`);
    process.exit(1);
  }
  console.log(
    "relay rate-limit live ok: Cloudflare rule matches server/relay-worker/README.md",
  );
}

if (
  process.argv[1] &&
  realpathSync(fileURLToPath(import.meta.url)) === realpathSync(process.argv[1])
) {
  await main();
}
