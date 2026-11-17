// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Unit and loopback HTTP end-to-end tests for tools/play-upload.mjs.
// Keys are generated ephemerally at runtime so no PEM material lives in git.

import assert from "node:assert/strict";
import { createHash, generateKeyPairSync, verify } from "node:crypto";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  ANDROID_PUBLISHER_SCOPE,
  GOOGLE_OAUTH_TOKEN_URL,
  MAX_PLAY_VERSION_CODE,
  createServiceAccountJwt,
  parseServiceAccountJson,
  parseStrictBoolean,
  resolveExpectedAndroidRelease,
  sanitizeExternalText,
  uploadPlayBundle,
  verifyLocalBundleIntegrity,
} from "./play-upload.mjs";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const SCRIPT_PATH = join(REPO_ROOT, "tools", "play-upload.mjs");

const { privateKey: TEST_RSA_PRIV, publicKey: TEST_RSA_PUB } = generateKeyPairSync("rsa", {
  modulusLength: 2048,
});
const TEST_RSA_PEM = TEST_RSA_PRIV.export({ type: "pkcs8", format: "pem" });

function makeServiceAccountJson(overrides = {}) {
  return JSON.stringify({
    type: "service_account",
    project_id: "totaldrop-test",
    private_key_id: "key-123",
    private_key: TEST_RSA_PEM,
    client_email: "play-uploader@totaldrop-test.iam.gserviceaccount.com",
    token_uri: GOOGLE_OAUTH_TOKEN_URL,
    ...overrides,
  });
}

function withTempDir(fn) {
  const dir = mkdtempSync(join(tmpdir(), "td-play-upload-"));
  return Promise.resolve()
    .then(() => fn(dir))
    .finally(() => rmSync(dir, { recursive: true, force: true }));
}

function startMockPlayServer(handler) {
  return new Promise((resolveServer, rejectServer) => {
    const server = createServer(async (req, res) => {
      const chunks = [];
      for await (const chunk of req) chunks.push(chunk);
      const body = Buffer.concat(chunks);
      await handler(req, body, res);
    });
    server.on("error", rejectServer);
    server.listen(0, "127.0.0.1", () => {
      const addr = server.address();
      const baseUrl = `http://127.0.0.1:${addr.port}`;
      resolveServer({
        baseUrl,
        close: () => new Promise((r) => server.close(() => r())),
      });
    });
  });
}

test("resolveExpectedAndroidRelease matches the live repository dynamically and catches manifest drift", async () => {
  const liveManifest = JSON.parse(readFileSync(join(REPO_ROOT, "versions.json"), "utf8"));
  const liveVer = liveManifest.components.android.version;
  const liveBuild = liveManifest.components.android.build;

  const live = resolveExpectedAndroidRelease(REPO_ROOT, `android-v${liveVer}`);
  assert.deepEqual(live, { versionName: liveVer, versionCode: liveBuild });

  assert.throws(() => resolveExpectedAndroidRelease(REPO_ROOT, "linux-v0.3.0-beta.6"), /invalid Android release tag/);
  assert.throws(
    () => resolveExpectedAndroidRelease(REPO_ROOT, "android-v999.999.999"),
    /does not match tag android-v999\.999\.999/,
  );

  await withTempDir((dir) => {
    mkdirSync(join(dir, "app"), { recursive: true });
    writeFileSync(join(dir, "app", "pubspec.yaml"), "name: totaldrop\nversion: 0.4.0-beta.1+15\n");
    // Missing versions.json must fail closed.
    assert.throws(() => resolveExpectedAndroidRelease(dir, "android-v0.4.0-beta.1"), /versions\.json is missing/);

    // Mismatched build between versions.json and pubspec.yaml.
    writeFileSync(
      join(dir, "versions.json"),
      JSON.stringify({
        schema: 1,
        stage: "pre-1.0",
        components: { android: { version: "0.4.0-beta.1", build: 15 } },
      }),
    );
    writeFileSync(join(dir, "app", "pubspec.yaml"), "name: totaldrop\nversion: 0.4.0-beta.1+16\n");
    assert.throws(() => resolveExpectedAndroidRelease(dir, "android-v0.4.0-beta.1"), /does not match versions\.json/);

    // Out-of-bounds Google Play versionCode (> 2_100_000_000).
    writeFileSync(
      join(dir, "versions.json"),
      JSON.stringify({
        schema: 1,
        stage: "pre-1.0",
        components: { android: { version: "0.4.0-beta.1", build: MAX_PLAY_VERSION_CODE + 1 } },
      }),
    );
    assert.throws(() => resolveExpectedAndroidRelease(dir, "android-v0.4.0-beta.1"), /must be between 1 and/);
  });
});

test("parseServiceAccountJson, parseStrictBoolean, and createServiceAccountJwt validate inputs cleanly", () => {
  assert.equal(parseStrictBoolean(undefined), false);
  assert.equal(parseStrictBoolean(""), false);
  assert.equal(parseStrictBoolean("FALSE"), false);
  assert.equal(parseStrictBoolean("0"), false);
  assert.equal(parseStrictBoolean("true"), true);
  assert.equal(parseStrictBoolean("YES"), true);
  assert.equal(parseStrictBoolean("1"), true);
  assert.throws(() => parseStrictBoolean("ture", "GOOGLE_PLAY_CHANGES_NOT_SENT_FOR_REVIEW"), /invalid GOOGLE_PLAY/);

  const creds = parseServiceAccountJson(makeServiceAccountJson());
  assert.equal(creds.clientEmail, "play-uploader@totaldrop-test.iam.gserviceaccount.com");
  assert.equal(creds.privateKeyId, "key-123");

  const jwt = createServiceAccountJwt(creds, { nowSeconds: 1_760_000_000 });
  const parts = jwt.split(".");
  assert.equal(parts.length, 3);
  const header = JSON.parse(Buffer.from(parts[0], "base64url").toString("utf8"));
  const payload = JSON.parse(Buffer.from(parts[1], "base64url").toString("utf8"));
  assert.deepEqual(header, { alg: "RS256", typ: "JWT", kid: "key-123" });
  assert.deepEqual(payload, {
    iss: "play-uploader@totaldrop-test.iam.gserviceaccount.com",
    scope: ANDROID_PUBLISHER_SCOPE,
    aud: GOOGLE_OAUTH_TOKEN_URL,
    iat: 1_760_000_000,
    exp: 1_760_000_300,
  });
  const validSig = verify(
    "RSA-SHA256",
    Buffer.from(`${parts[0]}.${parts[1]}`, "utf8"),
    TEST_RSA_PUB,
    Buffer.from(parts[2], "base64url"),
  );
  assert.equal(validSig, true);

  assert.throws(
    () => parseServiceAccountJson(makeServiceAccountJson({ token_uri: "https://evil.example/token" })),
    /token_uri must be https:\/\/oauth2\.googleapis\.com\/token/,
  );

  const { privateKey: ecKey } = generateKeyPairSync("ec", { namedCurve: "prime256v1" });
  const ecPem = ecKey.export({ type: "pkcs8", format: "pem" });
  assert.throws(
    () => parseServiceAccountJson(makeServiceAccountJson({ private_key: ecPem })),
    (err) => err.message.includes("must be an RSA key") && !err.message.includes("PRIVATE KEY"),
  );

  const secretCanary = "SUPER_SECRET_CANARY_VALUE_991";
  assert.throws(
    () => parseServiceAccountJson(`not-json-${secretCanary}`),
    (err) => err.message === "GOOGLE_PLAY_SERVICE_ACCOUNT_JSON is not valid JSON" && !err.message.includes(secretCanary),
  );
});

test("verifyLocalBundleIntegrity checks bundle bytes, missing entries, and adjacent SHA256SUMS", async () => {
  await withTempDir((dir) => {
    const bundlePath = join(dir, "TotalDrop-0.4.0-beta.1.aab");
    const payload = Buffer.from("fake-aab-zip-payload-for-digest-test", "utf8");
    const digest = createHash("sha256").update(payload).digest("hex");
    writeFileSync(bundlePath, payload);

    const direct = verifyLocalBundleIntegrity(bundlePath);
    assert.equal(direct.sha256, digest);
    assert.equal(direct.sizeBytes, payload.length);

    writeFileSync(join(dir, "SHA256SUMS"), `${digest}  TotalDrop-0.4.0-beta.1.aab\n`);
    assert.equal(verifyLocalBundleIntegrity(bundlePath).sha256, digest);

    writeFileSync(join(dir, "SHA256SUMS"), `${digest}  OtherFile.aab\n`);
    assert.throws(() => verifyLocalBundleIntegrity(bundlePath), /does not list TotalDrop-0\.4\.0-beta\.1\.aab/);

    writeFileSync(join(dir, "SHA256SUMS"), `${"0".repeat(64)}  TotalDrop-0.4.0-beta.1.aab\n`);
    assert.throws(() => verifyLocalBundleIntegrity(bundlePath), /SHA256SUMS mismatch/);
  });
});

test("uploadPlayBundle runs atomic Play edits including string versionCode normalization and staged rollouts", async () => {
  await withTempDir(async (dir) => {
    const bundleBytes = Buffer.from("PK\x03\x04-signed-totaldrop-bundle-bytes", "utf8");
    const bundleSha = createHash("sha256").update(bundleBytes).digest("hex");
    const bundlePath = join(dir, "TotalDrop-0.4.0-beta.1.aab");
    writeFileSync(bundlePath, bundleBytes);
    writeFileSync(join(dir, "SHA256SUMS"), `${bundleSha}  TotalDrop-0.4.0-beta.1.aab\n`);

    const calls = [];
    const mock = await startMockPlayServer((req, body, res) => {
      calls.push({ method: req.method, url: req.url, body });

      if (req.method === "POST" && req.url === "/token") {
        const params = new URLSearchParams(body.toString("utf8"));
        assert.equal(params.get("grant_type"), "urn:ietf:params:oauth:grant-type:jwt-bearer");
        const jwtParts = params.get("assertion").split(".");
        const ok = verify(
          "RSA-SHA256",
          Buffer.from(`${jwtParts[0]}.${jwtParts[1]}`, "utf8"),
          TEST_RSA_PUB,
          Buffer.from(jwtParts[2], "base64url"),
        );
        assert.equal(ok, true);
        res.writeHead(200, { "content-type": "application/json" });
        res.end(JSON.stringify({ access_token: "ya29.mock-token", expires_in: 3600, token_type: "Bearer" }));
        return;
      }
      if (req.method === "POST" && req.url === "/androidpublisher/v3/applications/app.totaldrop/edits") {
        assert.equal(req.headers.authorization, "Bearer ya29.mock-token");
        res.writeHead(200, { "content-type": "application/json" });
        res.end(JSON.stringify({ id: "edit-42", expiryTimeSeconds: "1760003600" }));
        return;
      }
      if (
        req.method === "POST" &&
        req.url === "/upload/androidpublisher/v3/applications/app.totaldrop/edits/edit-42/bundles?uploadType=media"
      ) {
        assert.deepEqual(body, bundleBytes);
        res.writeHead(200, { "content-type": "application/json" });
        // Return versionCode as numeric string to test Google Discovery int64 string normalization.
        res.end(JSON.stringify({ versionCode: "15", sha256: bundleSha.toUpperCase() }));
        return;
      }
      if (
        req.method === "PUT" &&
        req.url === "/androidpublisher/v3/applications/app.totaldrop/edits/edit-42/tracks/production"
      ) {
        const trackPayload = JSON.parse(body.toString("utf8"));
        assert.deepEqual(trackPayload, {
          track: "production",
          releases: [
            {
              name: "0.4.0-beta.1",
              versionCodes: ["15"],
              status: "inProgress",
              userFraction: 0.25,
            },
          ],
        });
        res.writeHead(200, { "content-type": "application/json" });
        res.end(JSON.stringify(trackPayload));
        return;
      }
      if (
        req.method === "POST" &&
        req.url ===
          "/androidpublisher/v3/applications/app.totaldrop/edits/edit-42:commit?changesNotSentForReview=true"
      ) {
        res.writeHead(200, { "content-type": "application/json" });
        res.end(JSON.stringify({ id: "edit-42" }));
        return;
      }

      res.writeHead(404, { "content-type": "application/json" });
      res.end(JSON.stringify({ error: { message: `unexpected ${req.method} ${req.url}` } }));
    });

    try {
      const result = await uploadPlayBundle({
        serviceAccountJson: makeServiceAccountJson(),
        bundlePath,
        packageName: "app.totaldrop",
        track: "production",
        status: "inProgress",
        userFraction: 0.25,
        releaseName: "0.4.0-beta.1",
        expectedVersionCode: 15,
        changesNotSentForReview: true,
        testEndpoints: {
          tokenUrl: `${mock.baseUrl}/token`,
          apiBaseUrl: mock.baseUrl,
        },
      });

      assert.equal(result.editId, "edit-42");
      assert.equal(result.versionCode, 15);
      assert.equal(result.sha256, bundleSha);
      assert.deepEqual(
        calls.map((c) => `${c.method} ${c.url}`),
        [
          "POST /token",
          "POST /androidpublisher/v3/applications/app.totaldrop/edits",
          "POST /upload/androidpublisher/v3/applications/app.totaldrop/edits/edit-42/bundles?uploadType=media",
          "PUT /androidpublisher/v3/applications/app.totaldrop/edits/edit-42/tracks/production",
          "POST /androidpublisher/v3/applications/app.totaldrop/edits/edit-42:commit?changesNotSentForReview=true",
        ],
      );
    } finally {
      await mock.close();
    }
  });
});

test("uploadPlayBundle validates parameters, escapes bidi/control chars, rejects dot-segment editId, and logs cleanup warnings", async () => {
  // ASCII control + Unicode RLO bidi override + astral emoji preserved without surrogate splitting.
  assert.equal(
    sanitizeExternalText("bad\x1b[31m\u202eevil😀\nnext"),
    "bad\\x1b[31m\\u{202e}evil😀\\x0anext",
  );

  await withTempDir(async (dir) => {
    const bundleBytes = Buffer.from("PK\x03\x04-bundle-v15", "utf8");
    const bundleSha = createHash("sha256").update(bundleBytes).digest("hex");
    const bundlePath = join(dir, "TotalDrop-0.4.0-beta.1.aab");
    writeFileSync(bundlePath, bundleBytes);

    // Parameter validation: packageName, track, userFraction bounds.
    await assert.rejects(
      () =>
        uploadPlayBundle({
          serviceAccountJson: makeServiceAccountJson(),
          bundlePath,
          packageName: "../bad",
          releaseName: "0.4.0-beta.1",
          expectedVersionCode: 15,
        }),
      /invalid Android package name/,
    );
    await assert.rejects(
      () =>
        uploadPlayBundle({
          serviceAccountJson: makeServiceAccountJson(),
          bundlePath,
          track: "../production",
          releaseName: "0.4.0-beta.1",
          expectedVersionCode: 15,
        }),
      /invalid Google Play track/,
    );
    await assert.rejects(
      () =>
        uploadPlayBundle({
          serviceAccountJson: makeServiceAccountJson(),
          bundlePath,
          status: "inProgress",
          userFraction: 1,
          releaseName: "0.4.0-beta.1",
          expectedVersionCode: 15,
        }),
      /strictly between 0 and 1/,
    );
    await assert.rejects(
      () =>
        uploadPlayBundle({
          serviceAccountJson: makeServiceAccountJson(),
          bundlePath,
          status: "draft",
          userFraction: 0.5,
          releaseName: "0.4.0-beta.1",
          expectedVersionCode: 15,
        }),
      /only valid when --status is inProgress/,
    );

    const methods = [];
    const warnings = [];
    let mode = "dot-edit-id";
    const mock = await startMockPlayServer((req, _body, res) => {
      methods.push(`${req.method} ${req.url}`);
      if (req.method === "POST" && req.url === "/token") {
        if (mode === "truncated-token-json") {
          res.writeHead(200, { "content-type": "application/json" });
          res.end('{"access_token":"SECRET_PARTIAL_TOKEN_');
          return;
        }
        res.writeHead(200, { "content-type": "application/json" });
        res.end(JSON.stringify({ access_token: "ya29.mock" }));
        return;
      }
      if (req.method === "POST" && req.url === "/androidpublisher/v3/applications/app.totaldrop/edits") {
        const id = mode === "dot-edit-id" ? ".." : "edit-bad";
        res.writeHead(200, { "content-type": "application/json" });
        res.end(JSON.stringify({ id }));
        return;
      }
      if (
        req.method === "POST" &&
        req.url === "/upload/androidpublisher/v3/applications/app.totaldrop/edits/edit-bad/bundles?uploadType=media"
      ) {
        if (mode === "wrong-version-code") {
          res.writeHead(200, { "content-type": "application/json" });
          res.end(JSON.stringify({ versionCode: 99, sha256: bundleSha }));
          return;
        }
        if (mode === "commit-fails-and-delete-fails") {
          res.writeHead(200, { "content-type": "application/json" });
          res.end(JSON.stringify({ versionCode: 15, sha256: bundleSha }));
          return;
        }
        res.writeHead(403, { "content-type": "application/json" });
        res.end(JSON.stringify({ error: { message: "permission denied \x1b[2J\u202e" } }));
        return;
      }
      if (
        req.method === "PUT" &&
        req.url === "/androidpublisher/v3/applications/app.totaldrop/edits/edit-bad/tracks/internal"
      ) {
        res.writeHead(200, { "content-type": "application/json" });
        res.end("{}");
        return;
      }
      if (
        req.method === "POST" &&
        req.url === "/androidpublisher/v3/applications/app.totaldrop/edits/edit-bad:commit"
      ) {
        res.writeHead(409, { "content-type": "application/json" });
        res.end(JSON.stringify({ error: { message: "edit conflict" } }));
        return;
      }
      if (req.method === "DELETE" && req.url === "/androidpublisher/v3/applications/app.totaldrop/edits/edit-bad") {
        if (mode === "commit-fails-and-delete-fails") {
          res.writeHead(500, { "content-type": "text/plain" });
          res.end("cleanup boom");
          return;
        }
        res.writeHead(204);
        res.end();
        return;
      }
      res.writeHead(500);
      res.end();
    });

    const runMode = () =>
      uploadPlayBundle({
        serviceAccountJson: makeServiceAccountJson(),
        bundlePath,
        releaseName: "0.4.0-beta.1",
        expectedVersionCode: 15,
        warnImpl: (msg) => warnings.push(msg),
        testEndpoints: { tokenUrl: `${mock.baseUrl}/token`, apiBaseUrl: mock.baseUrl },
      });

    try {
      // 1. ".." editId rejected before any bundle upload or path-normalized DELETE.
      await assert.rejects(runMode, /invalid edit id/);
      assert.equal(methods.some((m) => m.startsWith("DELETE ")), false);

      // 2. Truncated 200 OK JSON on /token does not echo partial access_token.
      methods.length = 0;
      mode = "truncated-token-json";
      await assert.rejects(
        runMode,
        (err) =>
          err.message.includes("Google OAuth token endpoint returned invalid JSON") &&
          !err.message.includes("SECRET_PARTIAL_TOKEN_"),
      );

      // 3. Mismatched versionCode aborts and deletes uncommitted edit.
      methods.length = 0;
      mode = "wrong-version-code";
      await assert.rejects(runMode, /versionCode mismatch: Play reported 99, expected 15/);
      assert.equal(methods.at(-1), "DELETE /androidpublisher/v3/applications/app.totaldrop/edits/edit-bad");

      // 4. Commit failure triggers DELETE; failed DELETE logs sanitized warning without masking primary error.
      methods.length = 0;
      warnings.length = 0;
      mode = "commit-fails-and-delete-fails";
      await assert.rejects(runMode, /edits\.commit failed \(HTTP 409: edit conflict\)/);
      assert.equal(methods.at(-1), "DELETE /androidpublisher/v3/applications/app.totaldrop/edits/edit-bad");
      assert.equal(warnings.length, 1);
      assert.match(warnings[0], /failed to delete uncommitted edit edit-bad \(HTTP 500: cleanup boom\)/);

      // 5. Control and bidi characters in remote error messages are escaped.
      methods.length = 0;
      mode = "api-403";
      await assert.rejects(
        runMode,
        (err) =>
          err.message.includes("HTTP 403: permission denied \\x1b[2J\\u{202e}") &&
          !err.message.includes("\x1b") &&
          !err.message.includes("\u202e"),
      );
    } finally {
      await mock.close();
    }
  });
});

test("CLI entry point validates flags and fails closed on empty values", () => {
  const helpRun = spawnSync(process.execPath, [SCRIPT_PATH, "--help"], { encoding: "utf8" });
  assert.equal(helpRun.status, 0);
  assert.match(helpRun.stderr, /usage:/);

  const badFlag = spawnSync(process.execPath, [SCRIPT_PATH, "--unknown=1"], { encoding: "utf8" });
  assert.equal(badFlag.status, 2);
  assert.match(badFlag.stderr, /unknown flag/);

  const missingArgs = spawnSync(process.execPath, [SCRIPT_PATH], { encoding: "utf8" });
  assert.equal(missingArgs.status, 2);
  assert.match(missingArgs.stderr, /--bundle and --tag are required/);

  const liveManifest = JSON.parse(readFileSync(join(REPO_ROOT, "versions.json"), "utf8"));
  const liveTag = `android-v${liveManifest.components.android.version}`;
  const emptyTrack = spawnSync(
    process.execPath,
    [SCRIPT_PATH, `--bundle=${SCRIPT_PATH}`, `--tag=${liveTag}`, "--track="],
    { encoding: "utf8" },
  );
  assert.equal(emptyTrack.status, 1);
  assert.match(emptyTrack.stderr, /invalid Google Play track/);
});
