// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Reconciles OpenSSF Scorecard code-scanning alerts on main against the
// intentional two-maintainer ruleset trade-offs and initial 90-day window
// documented in GOVERNANCE.md#branch-protection-and-scorecard (#318):
// - Dismisses open alerts whose latest instance on refs/heads/main matches the
//   exact documented warnings.
// - Reopens previously dismissed alerts if a later scan adds any undocumented
//   warning, changes the failure reason, or passes the 90-day cutoff, so
//   regressions never stay hidden behind an earlier dismissal.

import { fileURLToPath } from "node:url";

const REPO_RE = /^(?!\.+\/)[A-Za-z0-9_.-]+\/(?!\.+$)[A-Za-z0-9_.-]+$/;
const TOKEN_RE = /^[\x21-\x7e]+$/;
const MAX_PAGES = 10;
const ISSUE_MARKER = "(#318)";

// Repository opened 2026-10-27; Scorecard's 90-day Maintained window ends at
// the end of 2027-01-25 UTC (GOVERNANCE.md, SECURITY-INSIGHTS.yml).
export const MAINTAINED_CUTOFF = new Date("2027-01-26T00:00:00Z");

const BRANCH_SCORE_HEADER_RE =
  /^score is \d+: branch protection is not maximal on development and all release branches:$/;
const REMEDIATION_FOOTER = "Click Remediation section below to solve this issue";

// Scorecard's <90-day age messages ("project was created in last 90 days",
// "created less than 90 days ago", "project is N days old"), excluding
// archived/inactivity failures checked first via MAINTAINED_NON_AGE_RE.
const MAINTAINED_AGE_RE = /\b(?:90 days|\d+ days old)\b/i;
const MAINTAINED_NON_AGE_RE =
  /\b(?:archived|commit\(s\)|issue activity)\b/i;

export const DOCUMENTED_BRANCH_WARNINGS = new Set([
  "Warn: required approving review count is 1 on branch 'main'",
  "Warn: 'last push approval' is disabled on branch 'main'",
  "Warn: 'up-to-date branches' is disabled on branch 'main'",
]);

// Escape ASCII/C1 control characters, line/paragraph separators, and Unicode
// Bidi controls in a single pass before writing outside strings to logs.
export function visible(value) {
  return String(value).replace(
    /[\u0000-\u001f\u007f-\u009f\u061c\u200e\u200f\u2028-\u202e\u2066-\u2069]/g,
    (ch) => `\\u${ch.codePointAt(0).toString(16).padStart(4, "0")}`,
  );
}

export function parseApiBase(raw = "https://api.github.com") {
  let url;
  try {
    url = new URL(raw);
  } catch {
    throw new Error(`Invalid GITHUB_API_URL: ${visible(raw)}`);
  }
  if (url.username || url.password) {
    throw new Error("GITHUB_API_URL must not embed credentials");
  }
  if (url.search || url.hash) {
    throw new Error("GITHUB_API_URL must not include query or fragment");
  }
  const isLoopbackHttp =
    url.protocol === "http:" &&
    (url.hostname === "127.0.0.1" || url.hostname === "localhost");
  if (url.protocol !== "https:" && !isLoopbackHttp) {
    throw new Error(`GITHUB_API_URL must use https: ${visible(url.protocol)}`);
  }
  return new URL(url.toString().replace(/\/+$/, ""));
}

export function evaluateExemption(alert, now = new Date()) {
  if (!alert) {
    return null;
  }
  const ref = alert?.most_recent_instance?.ref;
  if (ref !== "refs/heads/main") {
    return null;
  }
  const ruleId = alert?.rule?.id;
  if (ruleId === "BranchProtectionID") {
    const text = String(alert?.most_recent_instance?.message?.text ?? "");
    const lines = text
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter((line) => line.length > 0);
    let warnCount = 0;
    for (const line of lines) {
      if (BRANCH_SCORE_HEADER_RE.test(line) || line === REMEDIATION_FOOTER) {
        continue;
      }
      if (DOCUMENTED_BRANCH_WARNINGS.has(line)) {
        warnCount += 1;
        continue;
      }
      return null;
    }
    if (warnCount === 0) {
      return null;
    }
    return {
      reason: "won't fix",
      comment:
        "Intentional two-maintainer ruleset settings on main (1 code-owner approval, last-push approval off, strict up-to-date branches off); see GOVERNANCE.md#branch-protection-and-scorecard (#318).",
    };
  }
  if (ruleId === "MaintainedID") {
    if (!(now instanceof Date) || Number.isNaN(now.getTime())) {
      return null;
    }
    if (now >= MAINTAINED_CUTOFF) {
      return null;
    }
    const text = String(alert?.most_recent_instance?.message?.text ?? "");
    if (MAINTAINED_NON_AGE_RE.test(text) || !MAINTAINED_AGE_RE.test(text)) {
      return null;
    }
    return {
      reason: "won't fix",
      comment:
        "Repository opened on 2026-10-27 (<90 days old); Maintained clears automatically after 90 days. See GOVERNANCE.md#branch-protection-and-scorecard (#318).",
    };
  }
  return null;
}

export function decideAlertAction(alert, now = new Date()) {
  if (!alert) {
    return { action: "ignore" };
  }
  const state = alert.state;
  const exemption = evaluateExemption(alert, now);
  if (state === "open") {
    if (exemption) {
      return {
        action: "dismiss",
        reason: exemption.reason,
        comment: exemption.comment,
      };
    }
    return { action: "keep_open" };
  }
  if (state === "dismissed") {
    const ref = alert?.most_recent_instance?.ref;
    const ruleId = alert?.rule?.id;
    const managedByUs =
      (ref === "refs/heads/main" &&
        (ruleId === "BranchProtectionID" || ruleId === "MaintainedID")) ||
      String(alert?.dismissed_comment ?? "").includes(ISSUE_MARKER);
    if (managedByUs && !exemption) {
      return { action: "reopen" };
    }
    return { action: "keep_dismissed" };
  }
  return { action: "ignore" };
}

// Backwards-compatible helper returning {reason, comment} when an open alert
// qualifies for dismissal, or null otherwise.
export function decideDismissal(alert, now = new Date()) {
  const decision = decideAlertAction(alert, now);
  if (decision.action !== "dismiss") {
    return null;
  }
  return { reason: decision.reason, comment: decision.comment };
}

export function nextLink(linkHeader, expectedOrigin) {
  if (!linkHeader) {
    return null;
  }
  const entryRe = /<([^>]+)>([^<]*)/g;
  for (const match of String(linkHeader).matchAll(entryRe)) {
    const rawTarget = match[1].trim();
    const params = match[2].replace(/\s*,\s*$/, "");
    if (!/;\s*rel="next"(?:\s*;|\s*$)/.test(params)) {
      continue;
    }
    let candidate;
    try {
      candidate = new URL(rawTarget);
    } catch {
      throw new Error(`Invalid pagination link URL: ${visible(rawTarget)}`);
    }
    if (candidate.username || candidate.password) {
      throw new Error("Refusing pagination link with embedded credentials");
    }
    if (candidate.origin !== expectedOrigin) {
      throw new Error(
        `Refusing cross-origin pagination link: ${visible(candidate.origin)}`,
      );
    }
    return candidate;
  }
  return null;
}

export async function dismissDocumentedAlerts({
  repo,
  token,
  apiBase = "https://api.github.com",
  now = new Date(),
  fetchImpl = globalThis.fetch,
  log = (msg) => process.stdout.write(`${msg}\n`),
}) {
  if (typeof repo !== "string" || !REPO_RE.test(repo)) {
    throw new Error(`Invalid GITHUB_REPOSITORY: ${visible(repo)}`);
  }
  if (typeof token !== "string" || !TOKEN_RE.test(token)) {
    throw new Error("Missing or invalid GITHUB_TOKEN");
  }
  const base = parseApiBase(apiBase);
  const headers = {
    Accept: "application/vnd.github+json",
    Authorization: `Bearer ${token}`,
    "X-GitHub-Api-Version": "2022-11-28",
  };

  const alerts = [];
  for (const state of ["open", "dismissed"]) {
    let pageUrl = new URL(
      `${base.pathname.replace(/\/$/, "")}/repos/${repo}/code-scanning/alerts?tool_name=Scorecard&state=${state}&per_page=100`,
      base.origin,
    );
    for (let page = 0; page < MAX_PAGES && pageUrl; page += 1) {
      const res = await fetchImpl(pageUrl, {
        method: "GET",
        headers,
        redirect: "error",
      });
      if (res.status === 404) {
        // No Scorecard analysis has been uploaded yet on a fresh branch/repo.
        log("No Scorecard code-scanning alerts found (HTTP 404).");
        return { checked: 0, dismissed: [], reopened: [], kept: [] };
      }
      if (!res.ok) {
        const body = visible((await res.text()).slice(0, 300));
        throw new Error(
          `Failed to list Scorecard alerts: HTTP ${res.status} (${body})`,
        );
      }
      const items = await res.json();
      if (!Array.isArray(items)) {
        throw new Error(
          "Expected JSON array from code-scanning alerts endpoint",
        );
      }
      alerts.push(...items);
      pageUrl = nextLink(res.headers.get("link"), base.origin);
      if (page === MAX_PAGES - 1 && pageUrl) {
        throw new Error(`Pagination exceeded ${MAX_PAGES} pages`);
      }
    }
  }

  const dismissed = [];
  const reopened = [];
  const kept = [];
  for (const alert of alerts) {
    const number = Number(alert?.number);
    const ruleId = visible(alert?.rule?.id ?? "unknown");
    if (!Number.isInteger(number) || number <= 0) {
      throw new Error(`Invalid alert number for rule ${ruleId}`);
    }
    const decision = decideAlertAction(alert, now);
    if (decision.action === "ignore" || decision.action === "keep_dismissed") {
      continue;
    }
    if (decision.action === "keep_open") {
      log(`Keeping open Scorecard alert #${number} (${ruleId}).`);
      kept.push({ number, ruleId });
      continue;
    }
    const patchUrl = new URL(
      `${base.pathname.replace(/\/$/, "")}/repos/${repo}/code-scanning/alerts/${number}`,
      base.origin,
    );
    const payload =
      decision.action === "dismiss"
        ? {
            state: "dismissed",
            dismissed_reason: decision.reason,
            dismissed_comment: decision.comment,
          }
        : { state: "open" };
    const patchRes = await fetchImpl(patchUrl, {
      method: "PATCH",
      headers: {
        ...headers,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(payload),
      redirect: "error",
    });
    if (!patchRes.ok) {
      const body = visible((await patchRes.text()).slice(0, 300));
      const verb = decision.action === "dismiss" ? "dismiss" : "reopen";
      throw new Error(
        `Failed to ${verb} Scorecard alert #${number} (${ruleId}): HTTP ${patchRes.status} (${body})`,
      );
    }
    if (decision.action === "dismiss") {
      log(`Dismissed Scorecard alert #${number} (${ruleId}).`);
      dismissed.push({ number, ruleId });
    } else {
      log(`Reopened Scorecard alert #${number} (${ruleId}).`);
      reopened.push({ number, ruleId });
    }
  }

  return { checked: alerts.length, dismissed, reopened, kept };
}

async function main() {
  await dismissDocumentedAlerts({
    repo: process.env.GITHUB_REPOSITORY,
    token: process.env.GITHUB_TOKEN || process.env.GH_TOKEN,
    apiBase: process.env.GITHUB_API_URL || "https://api.github.com",
  });
}

if (
  process.argv[1] &&
  fileURLToPath(import.meta.url) === process.argv[1]
) {
  main().catch((err) => {
    process.stderr.write(`${visible(err?.message ?? err)}\n`);
    process.exitCode = 1;
  });
}
