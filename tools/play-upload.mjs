#!/usr/bin/env node
// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Uploads a verified Android App Bundle (.aab) to Google Play Console through
// the Google Play Android Developer API v3 (`androidpublisher`) with zero
// third-party dependencies.
//
// Credentials come only from the `GOOGLE_PLAY_SERVICE_ACCOUNT_JSON`
// environment variable (never from command-line arguments) so secrets never
// appear in process listings or shell traces.
//
// Usage:
//   GOOGLE_PLAY_SERVICE_ACCOUNT_JSON='...' node tools/play-upload.mjs \
//     --bundle dist/TotalDrop-0.4.0-beta.1.aab \
//     --tag android-v0.4.0-beta.1 \
//     [--package app.totaldrop] \
//     [--track internal] \
//     [--status draft] \
//     [--user-fraction 0.1] \
//     [--changes-not-sent-for-review]

import { createHash, createPrivateKey, sign } from "node:crypto";
import { existsSync, openAsBlob, readFileSync, statSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const GOOGLE_OAUTH_TOKEN_URL = "https://oauth2.googleapis.com/token";
export const GOOGLE_PLAY_API_BASE_URL = "https://androidpublisher.googleapis.com";
export const ANDROID_PUBLISHER_SCOPE = "https://www.googleapis.com/auth/androidpublisher";
export const MAX_PLAY_VERSION_CODE = 2_100_000_000;

const DEFAULT_PACKAGE_NAME = "app.totaldrop";
const DEFAULT_TRACK = "internal";
const DEFAULT_RELEASE_STATUS = "draft";
const VALID_STATUSES = new Set(["draft", "completed", "halted", "inProgress"]);
const PACKAGE_NAME_RE = /^[A-Za-z][A-Za-z0-9_]*(?:\.[A-Za-z][A-Za-z0-9_]*)+$/;
const TRACK_NAME_RE = /^[A-Za-z0-9][A-Za-z0-9 _.-]{0,63}$/;
const EDIT_ID_RE = /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/;
const SHA256_HEX_RE = /^[0-9a-f]{64}$/;
const UNSAFE_TERMINAL_CHAR_RE = /[\p{Cc}\p{Cf}\p{Zl}\p{Zp}\p{Cs}]/u;

/**
 * Escapes control, bidi/format, line-separator, and lone-surrogate code points
 * in a single pass without splitting UTF-16 surrogate pairs before logging.
 */
export function sanitizeExternalText(input, maxCodePoints = 240) {
  const str = String(input ?? "");
  let out = "";
  let count = 0;
  for (const ch of str) {
    if (count >= maxCodePoints) break;
    count++;
    if (UNSAFE_TERMINAL_CHAR_RE.test(ch)) {
      const cp = ch.codePointAt(0);
      out += cp <= 0xff ? `\\x${cp.toString(16).padStart(2, "0")}` : `\\u{${cp.toString(16)}}`;
    } else {
      out += ch;
    }
  }
  return out;
}

/** Strictly parses a boolean string flag or environment value, failing closed on typos. */
export function parseStrictBoolean(raw, label = "boolean") {
  if (raw === undefined || raw === null || raw === "") return false;
  if (typeof raw === "boolean") return raw;
  const norm = String(raw).trim().toLowerCase();
  if (norm === "1" || norm === "true" || norm === "yes") return true;
  if (norm === "0" || norm === "false" || norm === "no") return false;
  throw new Error(`invalid ${label} value "${sanitizeExternalText(raw)}" (expected true/false, 1/0, or yes/no)`);
}

/** Validates and parses a Google Cloud service-account JSON key string without echoing secrets. */
export function parseServiceAccountJson(rawJson) {
  if (typeof rawJson !== "string" || rawJson.trim() === "") {
    throw new Error("GOOGLE_PLAY_SERVICE_ACCOUNT_JSON is empty or unset");
  }
  let parsed;
  try {
    parsed = JSON.parse(rawJson);
  } catch {
    throw new Error("GOOGLE_PLAY_SERVICE_ACCOUNT_JSON is not valid JSON");
  }
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("GOOGLE_PLAY_SERVICE_ACCOUNT_JSON must be a JSON object");
  }
  if (parsed.type !== "service_account") {
    throw new Error('GOOGLE_PLAY_SERVICE_ACCOUNT_JSON must have "type": "service_account"');
  }
  if (
    typeof parsed.client_email !== "string" ||
    parsed.client_email.length > 254 ||
    !/^[^\s@]+@[^\s@.]+\.[^\s@]+$/.test(parsed.client_email)
  ) {
    throw new Error("GOOGLE_PLAY_SERVICE_ACCOUNT_JSON is missing a valid client_email");
  }
  if (typeof parsed.private_key !== "string" || parsed.private_key.trim() === "") {
    throw new Error("GOOGLE_PLAY_SERVICE_ACCOUNT_JSON is missing private_key");
  }
  if (parsed.token_uri !== undefined && parsed.token_uri !== GOOGLE_OAUTH_TOKEN_URL) {
    throw new Error(`GOOGLE_PLAY_SERVICE_ACCOUNT_JSON token_uri must be ${GOOGLE_OAUTH_TOKEN_URL}`);
  }
  let keyObject;
  try {
    keyObject = createPrivateKey(parsed.private_key);
  } catch {
    throw new Error("GOOGLE_PLAY_SERVICE_ACCOUNT_JSON contains an unreadable private_key");
  }
  if (keyObject.asymmetricKeyType !== "rsa") {
    throw new Error("GOOGLE_PLAY_SERVICE_ACCOUNT_JSON private_key must be an RSA key");
  }
  const keyId =
    typeof parsed.private_key_id === "string" && parsed.private_key_id.trim() !== ""
      ? parsed.private_key_id.trim()
      : null;

  return {
    clientEmail: parsed.client_email,
    privateKey: keyObject,
    privateKeyId: keyId,
  };
}

/** Creates a 5-minute RS256 OAuth 2.0 JWT assertion for the Android Publisher scope. */
export function createServiceAccountJwt(credentials, { nowSeconds = Math.floor(Date.now() / 1000) } = {}) {
  const header = {
    alg: "RS256",
    typ: "JWT",
    ...(credentials.privateKeyId ? { kid: credentials.privateKeyId } : {}),
  };
  const payload = {
    iss: credentials.clientEmail,
    scope: ANDROID_PUBLISHER_SCOPE,
    aud: GOOGLE_OAUTH_TOKEN_URL,
    iat: nowSeconds,
    exp: nowSeconds + 300,
  };
  const encodedHeader = Buffer.from(JSON.stringify(header)).toString("base64url");
  const encodedPayload = Buffer.from(JSON.stringify(payload)).toString("base64url");
  const signingInput = `${encodedHeader}.${encodedPayload}`;
  const signature = sign("RSA-SHA256", Buffer.from(signingInput, "utf8"), credentials.privateKey).toString(
    "base64url",
  );
  return `${signingInput}.${signature}`;
}

function resolveEndpoints(testEndpoints) {
  if (!testEndpoints) {
    return {
      tokenUrl: GOOGLE_OAUTH_TOKEN_URL,
      apiBaseUrl: GOOGLE_PLAY_API_BASE_URL,
    };
  }
  const checkLoopback = (raw, label) => {
    const u = new URL(raw);
    if (u.protocol !== "http:" || u.hostname !== "127.0.0.1") {
      throw new Error(`${label} test endpoint must use http://127.0.0.1`);
    }
    return u.toString().replace(/\/$/, "");
  };
  return {
    tokenUrl: checkLoopback(testEndpoints.tokenUrl, "tokenUrl"),
    apiBaseUrl: checkLoopback(testEndpoints.apiBaseUrl, "apiBaseUrl"),
  };
}

function buildEndpointUrl(baseUrl, pathname, queryParams = null) {
  const expected = new URL(baseUrl);
  const u = new URL(baseUrl);
  u.pathname = pathname;
  u.search = "";
  if (queryParams) {
    for (const [k, v] of Object.entries(queryParams)) {
      u.searchParams.set(k, String(v));
    }
  }
  if (u.origin !== expected.origin) {
    throw new Error("refusing cross-origin endpoint URL");
  }
  return u.toString();
}

async function readJsonBody(res, label) {
  let raw;
  try {
    raw = await res.text();
  } catch {
    throw new Error(`${label} response stream failed (HTTP ${res.status})`);
  }
  try {
    return JSON.parse(raw);
  } catch {
    throw new Error(`${label} returned invalid JSON (HTTP ${res.status})`);
  }
}

async function readErrorDetail(res) {
  let raw = "";
  try {
    raw = await res.text();
  } catch {
    return `HTTP ${res.status}`;
  }
  try {
    const parsed = JSON.parse(raw);
    const msg =
      parsed?.error?.message ??
      parsed?.error_description ??
      (typeof parsed?.error === "string" ? parsed.error : null);
    if (typeof msg === "string" && msg.length > 0) {
      return `HTTP ${res.status}: ${sanitizeExternalText(msg)}`;
    }
  } catch {
    // Fall back to sanitized plain text snippet below.
  }
  const trimmed = raw.trim();
  return trimmed ? `HTTP ${res.status}: ${sanitizeExternalText(trimmed)}` : `HTTP ${res.status}`;
}

/** Reads `versions.json` and `app/pubspec.yaml` in `repoRoot` and verifies tag + versionCode agreement. */
export function resolveExpectedAndroidRelease(repoRoot, tag) {
  if (typeof tag !== "string" || !tag.startsWith("android-v") || tag.length <= "android-v".length) {
    throw new Error(`invalid Android release tag "${sanitizeExternalText(tag)}" (expected android-v<semver>)`);
  }
  const releaseVersion = tag.slice("android-v".length);
  if (!/^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$/.test(releaseVersion)) {
    throw new Error(`invalid SemVer "${sanitizeExternalText(releaseVersion)}" in tag`);
  }

  const versionsPath = join(repoRoot, "versions.json");
  if (!existsSync(versionsPath)) {
    throw new Error("versions.json is missing");
  }
  const manifest = JSON.parse(readFileSync(versionsPath, "utf8"));
  const android = manifest?.components?.android;
  if (!android || android.version !== releaseVersion) {
    throw new Error(
      `versions.json components.android.version (${sanitizeExternalText(JSON.stringify(android?.version))}) does not match tag ${tag}`,
    );
  }
  if (!Number.isSafeInteger(android.build) || android.build < 1 || android.build > MAX_PLAY_VERSION_CODE) {
    throw new Error(
      `versions.json components.android.build (${sanitizeExternalText(JSON.stringify(android.build))}) must be between 1 and ${MAX_PLAY_VERSION_CODE}`,
    );
  }
  const versionCode = android.build;

  const pubspecPath = join(repoRoot, "app", "pubspec.yaml");
  if (!existsSync(pubspecPath)) {
    throw new Error("app/pubspec.yaml is missing");
  }
  const pubspecLines = readFileSync(pubspecPath, "utf8").split("\n");
  let pubMatch = null;
  for (const line of pubspecLines) {
    const trimmed = line.trim();
    const m = /^version:[ \t]*([^+\s]+)\+([1-9][0-9]*)$/.exec(trimmed);
    if (m) {
      pubMatch = m;
      break;
    }
  }
  if (!pubMatch) {
    throw new Error("app/pubspec.yaml does not declare version: <semver>+<versionCode>");
  }
  const pubVer = pubMatch[1];
  const pubCode = Number(pubMatch[2]);
  if (pubVer !== releaseVersion) {
    throw new Error(`app/pubspec.yaml version (${sanitizeExternalText(pubVer)}) does not match tag ${tag}`);
  }
  if (pubCode !== versionCode) {
    throw new Error(`app/pubspec.yaml build (${pubCode}) does not match versions.json build (${versionCode})`);
  }

  return {
    versionName: releaseVersion,
    versionCode: pubCode,
  };
}

/** Computes the SHA-256 hex digest of `bundlePath` and cross-checks an adjacent `SHA256SUMS` file if present. */
export function verifyLocalBundleIntegrity(bundlePath) {
  if (!existsSync(bundlePath)) {
    throw new Error(`bundle file not found: ${bundlePath}`);
  }
  const st = statSync(bundlePath);
  if (!st.isFile() || st.size === 0) {
    throw new Error(`bundle file is empty or not a regular file: ${bundlePath}`);
  }
  const sha256 = createHash("sha256").update(readFileSync(bundlePath)).digest("hex");
  const sumsPath = join(dirname(bundlePath), "SHA256SUMS");
  if (existsSync(sumsPath)) {
    const fileName = basename(bundlePath);
    const lines = readFileSync(sumsPath, "utf8")
      .split("\n")
      .map((l) => l.trim())
      .filter(Boolean);
    let matched = false;
    for (const line of lines) {
      const m = /^([0-9a-fA-F]{64})[ \t]+\*?([^\s].*)$/.exec(line);
      if (m && m[2] === fileName) {
        if (m[1].toLowerCase() !== sha256) {
          throw new Error(`SHA256SUMS mismatch for ${fileName}: expected ${m[1].toLowerCase()}, got ${sha256}`);
        }
        matched = true;
      }
    }
    if (!matched) {
      throw new Error(`SHA256SUMS next to ${fileName} does not list ${fileName}`);
    }
  }
  return { sha256, sizeBytes: st.size };
}

/**
 * Uploads an `.aab` bundle to Google Play Console inside a single atomic edit
 * (`edits.insert` -> `edits.bundles.upload` -> `edits.tracks.update` -> `edits.commit`).
 */
export async function uploadPlayBundle({
  serviceAccountJson,
  bundlePath,
  packageName = DEFAULT_PACKAGE_NAME,
  track = DEFAULT_TRACK,
  status = DEFAULT_RELEASE_STATUS,
  releaseName,
  expectedVersionCode,
  userFraction = null,
  changesNotSentForReview = false,
  requestTimeoutMs = 30_000,
  uploadTimeoutMs = 300_000,
  fetchImpl = globalThis.fetch,
  warnImpl = console.error,
  testEndpoints = null,
}) {
  if (typeof packageName !== "string" || !PACKAGE_NAME_RE.test(packageName)) {
    throw new Error(`invalid Android package name "${sanitizeExternalText(packageName)}"`);
  }
  if (typeof track !== "string" || !TRACK_NAME_RE.test(track) || track.includes("..")) {
    throw new Error(`invalid Google Play track "${sanitizeExternalText(track)}"`);
  }
  if (!VALID_STATUSES.has(status)) {
    throw new Error(
      `invalid release status "${sanitizeExternalText(status)}" (expected draft, completed, halted, or inProgress)`,
    );
  }
  if (status === "inProgress") {
    if (typeof userFraction !== "number" || !Number.isFinite(userFraction) || userFraction <= 0 || userFraction >= 1) {
      throw new Error("status=inProgress requires --user-fraction strictly between 0 and 1");
    }
  } else if (userFraction !== null) {
    throw new Error("--user-fraction is only valid when --status is inProgress");
  }
  if (
    !Number.isSafeInteger(expectedVersionCode) ||
    expectedVersionCode < 1 ||
    expectedVersionCode > MAX_PLAY_VERSION_CODE
  ) {
    throw new Error(
      `expectedVersionCode must be an integer between 1 and ${MAX_PLAY_VERSION_CODE}, got ${sanitizeExternalText(JSON.stringify(expectedVersionCode))}`,
    );
  }
  if (typeof releaseName !== "string" || releaseName.trim() === "" || releaseName.length > 64) {
    throw new Error("releaseName must be a non-empty string up to 64 characters");
  }

  const credentials = parseServiceAccountJson(serviceAccountJson);
  const { sha256: localSha256, sizeBytes } = verifyLocalBundleIntegrity(bundlePath);
  const { tokenUrl, apiBaseUrl } = resolveEndpoints(testEndpoints);

  // 1. Exchange short-lived RS256 JWT assertion for an OAuth 2.0 Bearer token.
  const assertion = createServiceAccountJwt(credentials);
  const tokenRes = await fetchImpl(tokenUrl, {
    method: "POST",
    headers: {
      "content-type": "application/x-www-form-urlencoded",
      accept: "application/json",
    },
    body: new URLSearchParams({
      grant_type: "urn:ietf:params:oauth:grant-type:jwt-bearer",
      assertion,
    }).toString(),
    redirect: "error",
    signal: AbortSignal.timeout(requestTimeoutMs),
  });
  if (!tokenRes.ok) {
    throw new Error(`Google OAuth token exchange failed (${await readErrorDetail(tokenRes)})`);
  }
  const tokenPayload = await readJsonBody(tokenRes, "Google OAuth token endpoint");
  const accessToken = tokenPayload?.access_token;
  if (typeof accessToken !== "string" || accessToken.length === 0) {
    throw new Error("Google OAuth token response did not contain an access_token");
  }

  const authHeaders = {
    authorization: `Bearer ${accessToken}`,
    accept: "application/json",
  };
  const encPkg = encodeURIComponent(packageName);

  // 2. Open an atomic AppEdit transaction.
  const editsPath = `/androidpublisher/v3/applications/${encPkg}/edits`;
  const insertRes = await fetchImpl(buildEndpointUrl(apiBaseUrl, editsPath), {
    method: "POST",
    headers: {
      ...authHeaders,
      "content-type": "application/json",
    },
    body: "{}",
    redirect: "error",
    signal: AbortSignal.timeout(requestTimeoutMs),
  });
  if (!insertRes.ok) {
    throw new Error(`Google Play edits.insert failed (${await readErrorDetail(insertRes)})`);
  }
  const editObj = await readJsonBody(insertRes, "Google Play edits.insert");
  const editId = editObj?.id;
  if (typeof editId !== "string" || !EDIT_ID_RE.test(editId) || editId.includes("..")) {
    throw new Error("Google Play edits.insert returned an invalid edit id");
  }
  const encEdit = encodeURIComponent(editId);

  let committed = false;
  try {
    // 3. Stream the .aab bundle to edits.bundles.upload and verify Play's digest and versionCode.
    const bundleBlob = await openAsBlob(bundlePath, { type: "application/octet-stream" });
    const uploadUrl = buildEndpointUrl(
      apiBaseUrl,
      `/upload/androidpublisher/v3/applications/${encPkg}/edits/${encEdit}/bundles`,
      { uploadType: "media" },
    );
    const uploadRes = await fetchImpl(uploadUrl, {
      method: "POST",
      headers: {
        ...authHeaders,
        "content-type": "application/octet-stream",
      },
      body: bundleBlob,
      redirect: "error",
      signal: AbortSignal.timeout(uploadTimeoutMs),
    });
    if (!uploadRes.ok) {
      throw new Error(`Google Play edits.bundles.upload failed (${await readErrorDetail(uploadRes)})`);
    }
    const uploadedBundle = await readJsonBody(uploadRes, "Google Play edits.bundles.upload");
    const gotCode =
      typeof uploadedBundle?.versionCode === "string" && /^[1-9][0-9]*$/.test(uploadedBundle.versionCode)
        ? Number(uploadedBundle.versionCode)
        : uploadedBundle?.versionCode;
    if (gotCode !== expectedVersionCode) {
      throw new Error(
        `Google Play bundle versionCode mismatch: Play reported ${sanitizeExternalText(JSON.stringify(uploadedBundle?.versionCode))}, expected ${expectedVersionCode}`,
      );
    }
    const gotSha256 = typeof uploadedBundle?.sha256 === "string" ? uploadedBundle.sha256.toLowerCase() : "";
    if (!SHA256_HEX_RE.test(gotSha256) || gotSha256 !== localSha256) {
      throw new Error(
        `Google Play bundle SHA-256 mismatch: Play reported ${sanitizeExternalText(JSON.stringify(uploadedBundle?.sha256))}, local bundle is ${localSha256}`,
      );
    }

    // 4. Assign the uploaded versionCode to the target track.
    const releaseObj = {
      name: releaseName,
      versionCodes: [String(expectedVersionCode)],
      status,
      ...(userFraction !== null ? { userFraction } : {}),
    };
    const encTrack = encodeURIComponent(track);
    const trackUrl = buildEndpointUrl(apiBaseUrl, `${editsPath}/${encEdit}/tracks/${encTrack}`);
    const trackRes = await fetchImpl(trackUrl, {
      method: "PUT",
      headers: {
        ...authHeaders,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        track,
        releases: [releaseObj],
      }),
      redirect: "error",
      signal: AbortSignal.timeout(requestTimeoutMs),
    });
    if (!trackRes.ok) {
      throw new Error(`Google Play edits.tracks.update (${track}) failed (${await readErrorDetail(trackRes)})`);
    }

    // 5. Commit the AppEdit atomically.
    const commitUrl = buildEndpointUrl(
      apiBaseUrl,
      `${editsPath}/${encEdit}:commit`,
      changesNotSentForReview ? { changesNotSentForReview: "true" } : null,
    );
    const commitRes = await fetchImpl(commitUrl, {
      method: "POST",
      headers: authHeaders,
      redirect: "error",
      signal: AbortSignal.timeout(requestTimeoutMs),
    });
    if (!commitRes.ok) {
      throw new Error(`Google Play edits.commit failed (${await readErrorDetail(commitRes)})`);
    }
    committed = true;

    return {
      packageName,
      editId,
      track,
      status,
      releaseName,
      versionCode: expectedVersionCode,
      sha256: localSha256,
      sizeBytes,
    };
  } finally {
    if (!committed) {
      try {
        const delRes = await fetchImpl(buildEndpointUrl(apiBaseUrl, `${editsPath}/${encEdit}`), {
          method: "DELETE",
          headers: authHeaders,
          redirect: "error",
          signal: AbortSignal.timeout(10_000),
        });
        if (!delRes.ok) {
          warnImpl(
            `play-upload warning: failed to delete uncommitted edit ${editId} (${await readErrorDetail(delRes)})`,
          );
        }
      } catch (delErr) {
        warnImpl(
          `play-upload warning: failed to delete uncommitted edit ${editId} (${sanitizeExternalText(delErr?.message ?? delErr)})`,
        );
      }
    }
  }
}

const ALLOWED_FLAGS = new Set([
  "bundle",
  "tag",
  "repo-root",
  "package",
  "track",
  "status",
  "user-fraction",
  "changes-not-sent-for-review",
  "help",
]);

function parseCliArgs(argv) {
  const flags = {};
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "-h" || arg === "--help") {
      flags.help = true;
      continue;
    }
    if (!arg.startsWith("--")) {
      throw new Error(`unexpected positional argument "${sanitizeExternalText(arg)}"`);
    }
    const eq = arg.indexOf("=");
    const key = eq >= 0 ? arg.slice(2, eq) : arg.slice(2);
    if (!ALLOWED_FLAGS.has(key)) {
      throw new Error(`unknown flag "--${sanitizeExternalText(key)}"`);
    }
    if (key === "changes-not-sent-for-review" && eq < 0) {
      flags[key] = "true";
      continue;
    }
    const val = eq >= 0 ? arg.slice(eq + 1) : argv[++i];
    if (val === undefined) {
      throw new Error(`flag "--${key}" requires a value`);
    }
    flags[key] = val;
  }
  return flags;
}

function printUsage() {
  console.error(
    [
      "usage:",
      "  GOOGLE_PLAY_SERVICE_ACCOUNT_JSON='...' node tools/play-upload.mjs \\",
      "    --bundle <path/to/bundle.aab> \\",
      "    --tag android-v<semver> \\",
      "    [--package app.totaldrop] \\",
      "    [--track internal] \\",
      "    [--status draft|completed|halted|inProgress] \\",
      "    [--user-fraction <0..1>] \\",
      "    [--changes-not-sent-for-review]",
    ].join("\n"),
  );
}

async function main(argv) {
  let flags;
  try {
    flags = parseCliArgs(argv);
  } catch (e) {
    console.error(`play-upload: ${e.message}`);
    printUsage();
    process.exit(2);
  }
  if (flags.help) {
    printUsage();
    return;
  }
  if (!flags.bundle || !flags.tag) {
    console.error("play-upload: --bundle and --tag are required");
    printUsage();
    process.exit(2);
  }

  const repoRoot = flags["repo-root"]
    ? resolve(flags["repo-root"])
    : resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const bundlePath = resolve(flags.bundle);
  const track = flags.track ?? DEFAULT_TRACK;
  const status = flags.status ?? DEFAULT_RELEASE_STATUS;
  const packageName = flags.package ?? DEFAULT_PACKAGE_NAME;
  const userFraction = flags["user-fraction"] !== undefined ? Number(flags["user-fraction"]) : null;
  const changesNotSentForReview = parseStrictBoolean(
    flags["changes-not-sent-for-review"],
    "--changes-not-sent-for-review",
  );

  const { versionName, versionCode } = resolveExpectedAndroidRelease(repoRoot, flags.tag);
  const result = await uploadPlayBundle({
    serviceAccountJson: process.env.GOOGLE_PLAY_SERVICE_ACCOUNT_JSON ?? "",
    bundlePath,
    packageName,
    track,
    status,
    releaseName: versionName,
    expectedVersionCode: versionCode,
    userFraction,
    changesNotSentForReview,
  });

  console.log(
    `play-upload ok: ${result.packageName} ${result.releaseName} (versionCode ${result.versionCode}, sha256 ${result.sha256}) -> track "${result.track}" (${result.status}), edit ${result.editId}`,
  );
}

const invokedPath = process.argv[1] ? resolve(process.argv[1]) : "";
if (invokedPath === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((e) => {
    console.error(`play-upload failed: ${sanitizeExternalText(e?.message ?? e)}`);
    process.exit(1);
  });
}
