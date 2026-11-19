#!/usr/bin/env node
// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Checks that container base images in packaging/containers/*.Dockerfile and
// the pinned Node major in .github/workflows/ are not within 60 days of their
// end-of-life date on endoflife.date (#399, #392).
//
//   node tools/check-eol.mjs --extract
//       Parses repository files offline and prints every tracked release cycle
//       and recognized rolling/SDK image without making network calls.
//   node tools/check-eol.mjs [--now YYYY-MM-DD] [--warn-days 60] [--root DIR]
//       Queries https://endoflife.date/api/<product>.json and exits 1 if any
//       tracked cycle is already end of life or reaches end of life within
//       --warn-days days (inclusive).

import { existsSync, readFileSync, readdirSync, realpathSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const DEFAULT_WARN_DAYS = 60;
export const MAX_RESPONSE_BYTES = 1_048_576;
const DEFAULT_TIMEOUT_MS = 15_000;
const DAY_MS = 86_400_000;
const API_ORIGIN = "https://endoflife.date";

const MOVING_DEBIAN_TAGS = new Set([
  "latest",
  "stable",
  "oldstable",
  "oldoldstable",
  "testing",
  "unstable",
  "sid",
  "experimental",
  "rc-buggy",
]);

const MOVING_FEDORA_TAGS = new Set(["latest", "rawhide", "branched"]);

/** Structured error thrown by check-eol extraction, fetching, and evaluation. */
export class EolCheckError extends Error {
  constructor(code, message) {
    super(message);
    this.name = "EolCheckError";
    this.code = code;
  }
}

/**
 * Neutralizes backslashes, terminal control sequences, newlines, lone
 * surrogates, and Unicode Bidi/invisible formatting characters in one pass.
 */
export function sanitizeForLog(value, maxLen = 200) {
  const raw = String(value ?? "");
  let out = "";
  let count = 0;
  for (let i = 0; i < raw.length; ) {
    const cu = raw.charCodeAt(i);
    let cp = cu;
    let step = 1;
    if (cu >= 0xd800 && cu <= 0xdbff && i + 1 < raw.length) {
      const low = raw.charCodeAt(i + 1);
      if (low >= 0xdc00 && low <= 0xdfff) {
        cp = ((cu - 0xd800) << 10) + (low - 0xdc00) + 0x10000;
        step = 2;
      }
    }
    const unsafe =
      cp <= 0x1f ||
      (cp >= 0x7f && cp <= 0x9f) ||
      (cp >= 0xd800 && cp <= 0xdfff) ||
      cp === 0x061c ||
      (cp >= 0x200b && cp <= 0x200f) ||
      (cp >= 0x2028 && cp <= 0x202e) ||
      (cp >= 0x2060 && cp <= 0x2069) ||
      cp === 0xfeff ||
      (cp >= 0xe0000 && cp <= 0xe007f);
    let piece;
    if (cp === 0x5c) {
      piece = "\\\\";
    } else if (unsafe) {
      piece =
        cp <= 0xffff
          ? `\\u${cp.toString(16).padStart(4, "0")}`
          : `\\u{${cp.toString(16)}}`;
    } else {
      piece = raw.slice(i, i + step);
    }
    if (count + 1 > maxLen) {
      out += "...";
      break;
    }
    out += piece;
    count += 1;
    i += step;
  }
  return out;
}

/**
 * Parses a strict YYYY-MM-DD calendar date into an exact UTC epoch day integer.
 */
export function parseUtcDate(dateStr, errorCode = "INVALID_DATE", context = "date") {
  const clean = sanitizeForLog(dateStr, 64);
  if (typeof dateStr !== "string") {
    throw new EolCheckError(errorCode, `${context}: expected YYYY-MM-DD date string, got "${clean}"`);
  }
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(dateStr);
  if (!m) {
    throw new EolCheckError(errorCode, `${context}: invalid YYYY-MM-DD date "${clean}"`);
  }
  const year = Number(m[1]);
  const month = Number(m[2]);
  const day = Number(m[3]);
  const ms = Date.UTC(year, month - 1, day);
  const check = new Date(ms);
  if (
    check.getUTCFullYear() !== year ||
    check.getUTCMonth() !== month - 1 ||
    check.getUTCDate() !== day
  ) {
    throw new EolCheckError(errorCode, `${context}: non-existent calendar date "${clean}"`);
  }
  return Math.floor(ms / DAY_MS);
}

function todayUtcString() {
  return new Date().toISOString().slice(0, 10);
}

function daysLabel(n) {
  const abs = Math.abs(n);
  return `${abs} ${abs === 1 ? "day" : "days"}`;
}

function stripTrailingComment(line) {
  for (let i = 0; i < line.length; i++) {
    if (line[i] === "#" && (i === 0 || line[i - 1] === " " || line[i - 1] === "\t")) {
      return line.slice(0, i);
    }
  }
  return line;
}

function splitWhitespace(str) {
  const tokens = [];
  let start = -1;
  for (let i = 0; i < str.length; i++) {
    const code = str.charCodeAt(i);
    const isWs = code === 32 || (code >= 9 && code <= 13);
    if (isWs) {
      if (start !== -1) {
        tokens.push(str.slice(start, i));
        start = -1;
      }
    } else if (start === -1) {
      start = i;
    }
  }
  if (start !== -1) {
    tokens.push(str.slice(start));
  }
  return tokens;
}

function isValidFromFlag(token) {
  if (!token.startsWith("--")) return false;
  const eq = token.indexOf("=");
  if (eq <= 2 || eq === token.length - 1) return false;
  const name = token.slice(2, eq);
  return /^[A-Za-z][A-Za-z0-9-]*$/.test(name);
}

function parseFromTokens(tokens, where) {
  let idx = 1;
  while (idx < tokens.length && tokens[idx].startsWith("--")) {
    if (!isValidFromFlag(tokens[idx])) {
      throw new EolCheckError(
        "INVALID_DOCKERFILE_FROM",
        `${where}: invalid FROM flag "${sanitizeForLog(tokens[idx], 64)}"`,
      );
    }
    idx += 1;
  }
  if (idx >= tokens.length || tokens[idx].startsWith("-")) {
    throw new EolCheckError(
      "INVALID_DOCKERFILE_FROM",
      `${where}: malformed FROM instruction`,
    );
  }
  const imageRef = tokens[idx++];
  let alias = null;
  if (idx < tokens.length) {
    if (
      idx + 2 !== tokens.length ||
      tokens[idx].toUpperCase() !== "AS" ||
      !/^[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(tokens[idx + 1])
    ) {
      throw new EolCheckError(
        "INVALID_DOCKERFILE_FROM",
        `${where}: malformed FROM instruction`,
      );
    }
    alias = tokens[idx + 1];
  }
  return { imageRef, alias };
}

function stripDebianVariantSuffixes(tagLower) {
  let stripped = tagLower;
  while (stripped.endsWith("-slim") || stripped.endsWith("-backports")) {
    stripped = stripped.endsWith("-slim")
      ? stripped.slice(0, -"-slim".length)
      : stripped.slice(0, -"-backports".length);
  }
  return stripped;
}

function isNumericRelease(value) {
  const parts = value.split(".");
  if (parts.length === 0) return false;
  if (!/^[1-9]\d*$/.test(parts[0])) return false;
  for (let i = 1; i < parts.length; i++) {
    if (!/^\d+$/.test(parts[i])) return false;
  }
  return true;
}

/**
 * Parses a Dockerfile FROM line (or bare image reference) into an EOL target
 * or an explicitly recognized rolling/SDK exemption.
 */
export function parseContainerImage(fromOrRef, source = "<dockerfile>", lineNumber = 1) {
  const where = `${sanitizeForLog(source)}:${lineNumber}`;
  const rawInput = String(fromOrRef ?? "");
  const tokens = splitWhitespace(stripTrailingComment(rawInput));
  let rawRef;
  if (tokens.length === 0) {
    throw new EolCheckError("INVALID_DOCKERFILE_FROM", `${where}: empty image reference`);
  } else if (tokens[0].toUpperCase() === "FROM") {
    rawRef = parseFromTokens(tokens, where).imageRef;
  } else if (tokens.length === 1) {
    rawRef = tokens[0];
  } else {
    throw new EolCheckError(
      "UNRECOGNIZED_IMAGE",
      `${where}: unrecognized base image "${sanitizeForLog(rawInput.trim())}"`,
    );
  }
  const safeRef = sanitizeForLog(rawRef);

  let withoutDigest = rawRef;
  const atIdx = rawRef.indexOf("@");
  if (atIdx !== -1) {
    const digest = rawRef.slice(atIdx + 1);
    if (!/^sha256:[0-9a-f]{64}$/i.test(digest)) {
      throw new EolCheckError(
        "INVALID_IMAGE_DIGEST",
        `${where}: invalid image digest in "${safeRef}" (expected @sha256:<64 hex>)`,
      );
    }
    withoutDigest = rawRef.slice(0, atIdx);
  }

  // Locate `:tag` after the last `/` so a registry port (if any) is not confused with a tag.
  const slashIdx = withoutDigest.lastIndexOf("/");
  const colonIdx = withoutDigest.indexOf(":", slashIdx + 1);
  if (colonIdx === -1 || colonIdx === withoutDigest.length - 1) {
    throw new EolCheckError(
      "MISSING_IMAGE_TAG",
      `${where}: base image "${safeRef}" must specify an explicit release tag`,
    );
  }

  const repo = withoutDigest.slice(0, colonIdx).toLowerCase();
  const tag = withoutDigest.slice(colonIdx + 1);
  const safeTag = sanitizeForLog(tag, 80);

  if (repo === "debian" || repo === "docker.io/library/debian") {
    const stripped = stripDebianVariantSuffixes(tag.toLowerCase());
    if (MOVING_DEBIAN_TAGS.has(stripped)) {
      throw new EolCheckError(
        "MOVING_IMAGE_TAG",
        `${where}: moving or rolling Debian tag "${safeTag}" has no fixed release cycle`,
      );
    }
    if (isNumericRelease(stripped)) {
      const cycle = stripped.split(".")[0];
      return {
        kind: "eol",
        source,
        image: rawRef,
        label: `Debian ${cycle} (${safeTag})`,
        product: "debian",
        matchBy: "cycle",
        cycle,
        codename: null,
      };
    }
    if (/^[a-z][a-z0-9]*$/.test(stripped)) {
      return {
        kind: "eol",
        source,
        image: rawRef,
        label: `Debian ${stripped} (${safeTag})`,
        product: "debian",
        matchBy: "codename",
        cycle: null,
        codename: stripped,
      };
    }
    throw new EolCheckError(
      "INVALID_IMAGE_TAG",
      `${where}: unrecognized Debian release tag "${safeTag}"`,
    );
  }

  if (
    repo === "fedora" ||
    repo === "docker.io/library/fedora" ||
    repo === "registry.fedoraproject.org/fedora"
  ) {
    const lower = tag.toLowerCase();
    if (MOVING_FEDORA_TAGS.has(lower)) {
      throw new EolCheckError(
        "MOVING_IMAGE_TAG",
        `${where}: moving Fedora tag "${safeTag}" has no fixed release cycle`,
      );
    }
    const m = /^([1-9]\d*)(?:-(?:minimal|micro))?$/i.exec(tag);
    if (!m) {
      throw new EolCheckError(
        "INVALID_IMAGE_TAG",
        `${where}: unrecognized Fedora release tag "${safeTag}"`,
      );
    }
    const cycle = m[1];
    return {
      kind: "eol",
      source,
      image: rawRef,
      label: `Fedora ${cycle}`,
      product: "fedora",
      matchBy: "cycle",
      cycle,
      codename: null,
    };
  }

  if (repo === "archlinux" || repo === "docker.io/library/archlinux") {
    if (!/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(tag)) {
      throw new EolCheckError(
        "INVALID_IMAGE_TAG",
        `${where}: invalid Arch Linux tag "${safeTag}"`,
      );
    }
    return {
      kind: "exempt",
      source,
      image: rawRef,
      label: `Arch Linux (${safeTag})`,
      reason: "rolling release (no release-cycle EOL)",
    };
  }

  if (repo === "ghcr.io/flathub-infra/flatpak-github-actions") {
    if (!/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(tag)) {
      throw new EolCheckError(
        "INVALID_IMAGE_TAG",
        `${where}: invalid Flathub build image tag "${safeTag}"`,
      );
    }
    return {
      kind: "exempt",
      source,
      image: rawRef,
      label: `Flathub SDK (${safeTag})`,
      reason: "Flathub SDK image (not tracked on endoflife.date)",
    };
  }

  throw new EolCheckError(
    "UNRECOGNIZED_IMAGE",
    `${where}: unrecognized base image "${safeRef}"; add an endoflife.date mapping in tools/check-eol.mjs`,
  );
}

function parseDockerfile(content, relPath) {
  const rawLines = content.split("\n");
  const logicalLines = [];
  let buf = "";
  let startLine = 1;

  for (let i = 0; i < rawLines.length; i++) {
    const line = rawLines[i].endsWith("\r") ? rawLines[i].slice(0, -1) : rawLines[i];
    // Docker BuildKit skips `#` comment lines both outside and inside `\` continuations.
    if (line.trimStart().startsWith("#")) continue;
    if (buf === "") startLine = i + 1;
    if (line.endsWith("\\")) {
      buf += line.slice(0, -1) + " ";
    } else {
      buf += line;
      logicalLines.push({ lineNumber: startLine, text: buf });
      buf = "";
    }
  }
  if (buf !== "") {
    logicalLines.push({ lineNumber: startLine, text: buf });
  }

  const stages = new Set(["scratch"]);
  const extracted = [];

  for (const { lineNumber, text } of logicalLines) {
    const tokens = splitWhitespace(stripTrailingComment(text));
    if (tokens.length === 0 || tokens[0].toUpperCase() !== "FROM") continue;
    const where = `${sanitizeForLog(relPath)}:${lineNumber}`;
    const { imageRef, alias } = parseFromTokens(tokens, where);
    if (!imageRef.includes(":") && !imageRef.includes("@") && stages.has(imageRef.toLowerCase())) {
      if (alias) stages.add(alias.toLowerCase());
      continue;
    }
    if (alias) stages.add(alias.toLowerCase());
    extracted.push(parseContainerImage(imageRef, relPath, lineNumber));
  }

  if (extracted.length === 0) {
    throw new EolCheckError(
      "MISSING_DOCKERFILE_FROM",
      `${sanitizeForLog(relPath)}: no external FROM instruction found`,
    );
  }
  return extracted;
}

function extractNodeTarget(root) {
  const wfDir = join(root, ".github", "workflows");
  if (!existsSync(wfDir)) {
    throw new EolCheckError(
      "MISSING_NODE_VERSION",
      ".github/workflows/: directory does not exist",
    );
  }
  const files = readdirSync(wfDir)
    .filter((f) => f.endsWith(".yml") || f.endsWith(".yaml"))
    .sort();

  const majors = new Map();
  for (const file of files) {
    const rel = `.github/workflows/${file}`;
    const lines = readFileSync(join(wfDir, file), "utf8").split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].endsWith("\r") ? lines[i].slice(0, -1) : lines[i];
      if (line.trimStart().startsWith("#")) continue;
      const cleaned = stripTrailingComment(line).trim();
      if (!cleaned.startsWith("node-version:")) continue;
      let val = cleaned.slice("node-version:".length).trim();
      if (
        val.length >= 2 &&
        ((val.startsWith('"') && val.endsWith('"')) ||
          (val.startsWith("'") && val.endsWith("'")))
      ) {
        val = val.slice(1, -1).trim();
      }
      if (!/^[1-9]\d*(?:\.\d+\.\d+)?$/.test(val)) {
        throw new EolCheckError(
          "INVALID_NODE_VERSION",
          `${sanitizeForLog(rel)}:${i + 1}: node-version "${sanitizeForLog(val, 64)}" is not a fixed Node major or X.Y.Z release`,
        );
      }
      const major = val.split(".")[0];
      if (!majors.has(major)) majors.set(major, rel);
    }
  }

  if (majors.size === 0) {
    throw new EolCheckError(
      "MISSING_NODE_VERSION",
      ".github/workflows/: no node-version found in workflow files",
    );
  }
  if (majors.size > 1) {
    const detail = [...majors.entries()]
      .map(([maj, file]) => `${maj} (${sanitizeForLog(file)})`)
      .join(", ");
    throw new EolCheckError(
      "NODE_MAJOR_MISMATCH",
      `.github/workflows/: multiple Node majors found: ${detail}`,
    );
  }

  const [major] = majors.keys();
  return {
    kind: "eol",
    source: ".github/workflows/",
    image: null,
    label: `Node ${major}`,
    product: "nodejs",
    matchBy: "cycle",
    cycle: major,
    codename: null,
  };
}

/**
 * Extracts all container base-image targets from packaging/containers/*.Dockerfile
 * and the pinned Node major from .github/workflows/*.{yml,yaml}.
 */
export function extractTargets(root) {
  const containersDir = join(root, "packaging", "containers");
  if (!existsSync(containersDir)) {
    throw new EolCheckError(
      "MISSING_CONTAINERS_DIR",
      "packaging/containers: directory does not exist",
    );
  }

  const dockerfiles = readdirSync(containersDir)
    .filter((f) => f.endsWith(".Dockerfile"))
    .sort();
  if (dockerfiles.length === 0) {
    throw new EolCheckError(
      "MISSING_CONTAINERS_DIR",
      "packaging/containers: no *.Dockerfile files found",
    );
  }

  const targets = [];
  for (const file of dockerfiles) {
    const rel = `packaging/containers/${file}`;
    const content = readFileSync(join(containersDir, file), "utf8");
    targets.push(...parseDockerfile(content, rel));
  }
  targets.push(extractNodeTarget(root));
  return targets;
}

async function readBoundedText(response, maxBytes, product) {
  const clHeader = response.headers?.get?.("content-length");
  if (clHeader !== null && clHeader !== undefined && clHeader !== "") {
    const declared = Number(clHeader);
    if (Number.isFinite(declared) && declared > maxBytes) {
      await response.body?.cancel?.().catch(() => {});
      throw new EolCheckError(
        "RESPONSE_TOO_LARGE",
        `endoflife.date/${product}: Content-Length ${declared} exceeds ${maxBytes} bytes`,
      );
    }
  }

  if (!response.body || typeof response.body.getReader !== "function") {
    const buf = new Uint8Array(await response.arrayBuffer());
    if (buf.byteLength > maxBytes) {
      throw new EolCheckError(
        "RESPONSE_TOO_LARGE",
        `endoflife.date/${product}: response body exceeds ${maxBytes} bytes`,
      );
    }
    return new TextDecoder("utf-8", { fatal: true }).decode(buf);
  }

  const reader = response.body.getReader();
  const chunks = [];
  let total = 0;
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      const bytes = value instanceof Uint8Array ? value : new Uint8Array(value);
      total += bytes.byteLength;
      if (total > maxBytes) {
        await reader.cancel().catch(() => {});
        throw new EolCheckError(
          "RESPONSE_TOO_LARGE",
          `endoflife.date/${product}: response body exceeds ${maxBytes} bytes`,
        );
      }
      chunks.push(bytes);
    }
  } finally {
    reader.releaseLock();
  }

  const merged = new Uint8Array(total);
  let offset = 0;
  for (const c of chunks) {
    merged.set(c, offset);
    offset += c.byteLength;
  }
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(merged);
  } catch {
    throw new EolCheckError(
      "INVALID_API_PAYLOAD",
      `endoflife.date/${product}: response is not valid UTF-8`,
    );
  }
}

/**
 * Fetches and validates release cycle metadata for a single product from
 * https://endoflife.date/api/<product>.json.
 */
export async function fetchProductReleases(product, options = {}) {
  if (typeof product !== "string" || !/^[a-z0-9-]+$/.test(product)) {
    throw new EolCheckError(
      "INVALID_PRODUCT",
      `invalid endoflife.date product slug "${sanitizeForLog(product, 64)}"`,
    );
  }

  const {
    fetchImpl = globalThis.fetch,
    timeoutMs = DEFAULT_TIMEOUT_MS,
    maxBytes = MAX_RESPONSE_BYTES,
    retries = 0,
    retryDelayMs = 1_500,
  } = options;

  const url = new URL(`/api/${encodeURIComponent(product)}.json`, API_ORIGIN);
  if (url.origin !== API_ORIGIN || url.protocol !== "https:") {
    throw new EolCheckError("INVALID_PRODUCT", "unexpected API origin");
  }

  let lastErr;
  for (let attempt = 0; attempt <= retries; attempt++) {
    if (attempt > 0 && retryDelayMs > 0) {
      await new Promise((r) => setTimeout(r, retryDelayMs));
    }

    let response;
    try {
      response = await fetchImpl(url.href, {
        method: "GET",
        redirect: "error",
        headers: {
          Accept: "application/json",
          "User-Agent": "totaldrop-check-eol/1",
        },
        signal: AbortSignal.timeout(timeoutMs),
      });
    } catch (err) {
      lastErr = new EolCheckError(
        "HTTP_ERROR",
        `${url.href}: request failed: ${sanitizeForLog(err?.message ?? err, 160)}`,
      );
      continue;
    }

    if (!response.ok) {
      await response.body?.cancel?.().catch(() => {});
      const httpErr = new EolCheckError(
        "HTTP_ERROR",
        `${url.href}: HTTP ${Number(response.status)}`,
      );
      if ((response.status === 429 || response.status >= 500) && attempt < retries) {
        lastErr = httpErr;
        continue;
      }
      throw httpErr;
    }

    const rawText = await readBoundedText(response, maxBytes, product);
    let parsed;
    try {
      parsed = JSON.parse(rawText);
    } catch {
      throw new EolCheckError(
        "INVALID_API_PAYLOAD",
        `${url.href}: response is not valid JSON`,
      );
    }

    if (!Array.isArray(parsed) || parsed.length === 0) {
      throw new EolCheckError(
        "INVALID_API_PAYLOAD",
        `${url.href}: expected non-empty JSON array of release cycles`,
      );
    }

    for (let i = 0; i < parsed.length; i++) {
      const entry = parsed[i];
      if (
        typeof entry !== "object" ||
        entry === null ||
        Array.isArray(entry) ||
        typeof entry.cycle !== "string" ||
        entry.cycle.trim() === ""
      ) {
        throw new EolCheckError(
          "INVALID_API_PAYLOAD",
          `${url.href}: invalid release entry at index ${i}`,
        );
      }
    }

    return parsed;
  }

  throw lastErr;
}

/**
 * Compares extracted targets against fetched endoflife.date release lists.
 */
export function evaluateTargets(targets, releasesByProduct, options = {}) {
  const { now = todayUtcString(), warnDays = DEFAULT_WARN_DAYS } = options;
  if (!Number.isInteger(warnDays) || warnDays < 0) {
    throw new EolCheckError("USAGE_ERROR", `--warn-days must be a non-negative integer, got "${sanitizeForLog(warnDays)}"`);
  }
  const nowEpochDay = parseUtcDate(now, "INVALID_DATE", "--now");

  const results = [];
  const failures = [];

  for (const target of targets) {
    const safeSource = sanitizeForLog(target.source);
    const safeLabel = sanitizeForLog(target.label);

    if (target.kind === "exempt") {
      const safeReason = sanitizeForLog(target.reason);
      results.push({
        target,
        status: "exempt",
        eol: null,
        daysRemaining: null,
        message: `${safeLabel} (${safeSource}): ${safeReason}`,
      });
      continue;
    }

    const releases = releasesByProduct.get(target.product);
    if (!Array.isArray(releases)) {
      throw new EolCheckError(
        "INVALID_API_PAYLOAD",
        `missing release data for product "${sanitizeForLog(target.product)}"`,
      );
    }

    const match =
      target.matchBy === "codename"
        ? releases.find(
            (r) =>
              typeof r.codename === "string" &&
              r.codename.toLowerCase() === target.codename,
          )
        : releases.find((r) => r.cycle === target.cycle);

    if (!match) {
      const wanted =
        target.matchBy === "codename"
          ? `codename "${sanitizeForLog(target.codename)}"`
          : `cycle "${sanitizeForLog(target.cycle)}"`;
      throw new EolCheckError(
        "UNKNOWN_RELEASE_CYCLE",
        `${safeLabel} (${safeSource}): ${wanted} was not found in endoflife.date/${sanitizeForLog(target.product)}`,
      );
    }

    const cycleTag = sanitizeForLog(match.cycle, 40);
    const codenameNote =
      typeof match.codename === "string" && match.codename.trim() !== ""
        ? ` (${sanitizeForLog(match.codename, 40)})`
        : "";
    const subject = `${sanitizeForLog(target.product)} ${cycleTag}${codenameNote} [${safeSource}]`;

    // Check standard `eol` intentionally rather than `extendedSupport` (e.g.
    // Debian paid/extended ELTS) so upgrades are planned before regular
    // distribution support ends.
    if (match.eol === false) {
      results.push({
        target,
        status: "ok",
        eol: false,
        daysRemaining: null,
        message: `${subject}: supported (no end-of-life date scheduled yet)`,
      });
      continue;
    }

    if (match.eol === true) {
      const item = {
        target,
        status: "expired",
        eol: true,
        daysRemaining: null,
        message: `${subject}: already end of life (eol: true)`,
      };
      results.push(item);
      failures.push(item);
      continue;
    }

    if (typeof match.eol !== "string") {
      throw new EolCheckError(
        "INVALID_EOL_FIELD",
        `${subject}: unexpected "eol" field "${sanitizeForLog(match.eol, 64)}"`,
      );
    }

    const eolEpochDay = parseUtcDate(match.eol, "INVALID_EOL_FIELD", `${subject} eol`);
    const safeEol = sanitizeForLog(match.eol, 32);
    const daysRemaining = eolEpochDay - nowEpochDay;

    if (daysRemaining < 0) {
      const item = {
        target,
        status: "expired",
        eol: match.eol,
        daysRemaining,
        message: `${subject}: reached end of life ${daysLabel(daysRemaining)} ago on ${safeEol}`,
      };
      results.push(item);
      failures.push(item);
    } else if (daysRemaining <= warnDays) {
      const item = {
        target,
        status: "warning",
        eol: match.eol,
        daysRemaining,
        message: `${subject}: reaches end of life in ${daysLabel(daysRemaining)} on ${safeEol} (warning threshold: ${warnDays} days)`,
      };
      results.push(item);
      failures.push(item);
    } else {
      results.push({
        target,
        status: "ok",
        eol: match.eol,
        daysRemaining,
        message: `${subject}: eol ${safeEol} (${daysLabel(daysRemaining)} remaining)`,
      });
    }
  }

  return {
    ok: failures.length === 0,
    now,
    warnDays,
    results,
    failures,
  };
}

/**
 * Full check pipeline: extracts targets from `root`, fetches unique products
 * from endoflife.date, and evaluates the 60-day warning threshold.
 */
export async function runCheck(options = {}) {
  const {
    root = join(dirname(fileURLToPath(import.meta.url)), ".."),
    now = todayUtcString(),
    warnDays = DEFAULT_WARN_DAYS,
    fetchImpl,
    timeoutMs,
    maxBytes,
    retries = fetchImpl ? 0 : 2,
    retryDelayMs,
  } = options;

  // Validate --now and --warn-days before making any network calls.
  parseUtcDate(now, "INVALID_DATE", "--now");
  if (!Number.isInteger(warnDays) || warnDays < 0) {
    throw new EolCheckError(
      "USAGE_ERROR",
      `--warn-days must be a non-negative integer, got "${sanitizeForLog(warnDays)}"`,
    );
  }

  const targets = extractTargets(root);
  const products = [...new Set(targets.filter((t) => t.kind === "eol").map((t) => t.product))].sort();
  const releasesByProduct = new Map();

  for (const product of products) {
    const releases = await fetchProductReleases(product, {
      fetchImpl,
      timeoutMs,
      maxBytes,
      retries,
      retryDelayMs,
    });
    releasesByProduct.set(product, releases);
  }

  return evaluateTargets(targets, releasesByProduct, { now, warnDays });
}

const USAGE =
  "usage: node tools/check-eol.mjs [--extract | --offline-parse-only] [--now YYYY-MM-DD] [--warn-days N] [--root DIR]";

function parseArgs(argv) {
  let extractOnly = false;
  let now = todayUtcString();
  let warnDays = DEFAULT_WARN_DAYS;
  let root = join(dirname(fileURLToPath(import.meta.url)), "..");
  let help = false;

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--extract" || arg === "--offline-parse-only") {
      extractOnly = true;
    } else if (arg === "--help" || arg === "-h") {
      help = true;
    } else if (arg === "--now") {
      if (i + 1 >= argv.length) {
        throw new EolCheckError("USAGE_ERROR", `${USAGE}\nmissing value for --now`);
      }
      now = argv[++i];
    } else if (arg === "--warn-days") {
      if (i + 1 >= argv.length) {
        throw new EolCheckError("USAGE_ERROR", `${USAGE}\nmissing value for --warn-days`);
      }
      const raw = argv[++i];
      if (!/^\d+$/.test(raw)) {
        throw new EolCheckError(
          "USAGE_ERROR",
          `${USAGE}\ninvalid --warn-days "${sanitizeForLog(raw, 32)}"`,
        );
      }
      warnDays = Number(raw);
    } else if (arg === "--root") {
      if (i + 1 >= argv.length) {
        throw new EolCheckError("USAGE_ERROR", `${USAGE}\nmissing value for --root`);
      }
      root = argv[++i];
    } else {
      throw new EolCheckError(
        "USAGE_ERROR",
        `${USAGE}\nunknown option "${sanitizeForLog(arg, 64)}"`,
      );
    }
  }

  return { extractOnly, now, warnDays, root, help };
}

function formatExtractedTarget(target) {
  const src = sanitizeForLog(target.source);
  if (target.kind === "exempt") {
    return `exempt  ${sanitizeForLog(target.label)} [${src}]: ${sanitizeForLog(target.reason)}`;
  }
  const selector =
    target.matchBy === "codename"
      ? `codename=${sanitizeForLog(target.codename)}`
      : `cycle=${sanitizeForLog(target.cycle)}`;
  return `track   ${sanitizeForLog(target.product)} (${selector}) <- ${sanitizeForLog(target.label)} [${src}]`;
}

/**
 * Runs the command-line interface and returns an exit code (0 = ok, 1 = check
 * failed, 2 = invalid CLI usage).
 */
export async function runCli(argv, io = {}) {
  const stdout = io.stdout ?? ((line) => console.log(line));
  const stderr = io.stderr ?? ((line) => console.error(line));

  let opts;
  try {
    opts = parseArgs(argv);
  } catch (err) {
    stderr(err instanceof EolCheckError ? err.message : sanitizeForLog(err?.message ?? err));
    return 2;
  }

  if (opts.help) {
    stdout(USAGE);
    return 0;
  }

  try {
    if (opts.extractOnly) {
      const targets = extractTargets(opts.root);
      for (const target of targets) {
        stdout(formatExtractedTarget(target));
      }
      return 0;
    }

    const report = await runCheck({
      root: opts.root,
      now: opts.now,
      warnDays: opts.warnDays,
      fetchImpl: io.fetchImpl,
      retries: io.retries,
      retryDelayMs: io.retryDelayMs,
    });

    for (const item of report.results) {
      if (item.status === "ok" || item.status === "exempt") {
        stdout(`ok      ${item.message}`);
      } else {
        stderr(`FAIL    ${item.message}`);
      }
    }
    return report.ok ? 0 : 1;
  } catch (err) {
    const msg = err instanceof EolCheckError ? err.message : sanitizeForLog(err?.message ?? err);
    stderr(`error: ${msg}`);
    return err instanceof EolCheckError && err.code === "USAGE_ERROR" ? 2 : 1;
  }
}

const isMain =
  process.argv[1] &&
  realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url));

if (isMain) {
  const code = await runCli(process.argv.slice(2));
  process.exit(code);
}
