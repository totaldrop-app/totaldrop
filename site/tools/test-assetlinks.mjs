// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test site/tools/test-assetlinks.mjs
//
// https://totaldrop.app/r#<code> opens the Android app only while the
// site's assetlinks.json names that app (#445). These checks keep the
// file, the app's package and its manifest filter in step.
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const root = new URL("../../", import.meta.url);
const read = (path) => readFileSync(new URL(path, root), "utf8");

const statements = JSON.parse(read("site/public/.well-known/assetlinks.json"));
const gradle = read("app/android/app/build.gradle.kts");
const manifest = read("app/android/app/src/main/AndroidManifest.xml");

test("assetlinks.json lets the Android app handle the site's links", () => {
  assert.equal(statements.length, 1);
  const [s] = statements;
  assert.deepEqual(s.relation, ["delegate_permission/common.handle_all_urls"]);
  assert.equal(s.target.namespace, "android_app");
  const id = gradle.match(/applicationId\s*=\s*"([^"]+)"/)?.[1];
  assert.equal(s.target.package_name, id, "the package the app is built as");
});

test("the fingerprints are SHA-256, in the form Android compares", () => {
  const prints = statements[0].target.sha256_cert_fingerprints;
  assert.ok(prints.length >= 1);
  for (const p of prints) {
    assert.match(p, /^([0-9A-F]{2}:){31}[0-9A-F]{2}$/);
  }
});

test("the manifest asks Android to verify https://totaldrop.app/r", () => {
  const filters = manifest.match(/<intent-filter\b[\s\S]*?<\/intent-filter>/g) ?? [];
  const https = filters.filter((f) => /android:scheme="https"/.test(f));
  assert.equal(https.length, 1, "one https filter");
  const [f] = https;
  assert.match(f, /android:autoVerify="true"/);
  assert.match(f, /android\.intent\.category\.BROWSABLE/);
  // Exactly one address: another <data> in the same filter would be
  // verified along with it, and every host in it must pass, or none does.
  const data = f.match(/<data\b[^>]*\/>/g) ?? [];
  assert.deepEqual(data, [
    '<data android:scheme="https" android:host="totaldrop.app" android:path="/r"/>',
  ]);
});

// Flutter also reads every VIEW intent and pushes its path as a route, so
// totaldrop://r/ put a second Home screen over Receive. Links are handled
// only by MainActivity.kt; Flutter's own deep linking stays off.
test("Flutter's own deep linking is off, so links reach only MainActivity", () => {
  const activity = manifest.match(
    /<activity\b[^>]*android:name="\.MainActivity"[\s\S]*?<\/activity>/,
  )?.[0];
  assert.ok(activity, "MainActivity is declared");
  assert.match(
    activity,
    /<meta-data\s+android:name="flutter_deeplinking_enabled"\s+android:value="false"\s*\/>/,
  );
});
