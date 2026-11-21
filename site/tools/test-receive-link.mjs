// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test site/tools/test-receive-link.mjs
//
// On Android, the "Open in TotalDrop" button on /r must name our package in
// an intent:// URL so no other app that declared totaldrop:// can claim the
// transfer code (#507). Elsewhere it keeps totaldrop://r/<code>.
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";

const root = new URL("../../", import.meta.url);
const read = (path) => readFileSync(new URL(path, root), "utf8");

const appJs = read("site/public/app.js");
const gradle = read("app/android/app/build.gradle.kts");
const manifest = read("app/android/app/src/main/AndroidManifest.xml");
const mainActivity = read("app/android/app/src/main/kotlin/app/totaldrop/totaldrop/MainActivity.kt");
const assetlinks = JSON.parse(read("site/public/.well-known/assetlinks.json"));

function loadHelpers() {
  const ctx = {};
  ctx.globalThis = ctx;
  vm.runInNewContext(appJs, ctx);
  return ctx.TDApp;
}

// Runs app.js on a minimal /r page DOM and returns the state of its elements.
function runReceivePage(hash, nav) {
  const elements = new Map([
    ["r-has-code", { id: "r-has-code", hidden: true }],
    ["r-no-code", { id: "r-no-code", hidden: true }],
    ["r-code", { id: "r-code", textContent: "" }],
    ["r-open", { id: "r-open", href: "#" }],
    ["r-copy", { id: "r-copy", textContent: "Copy the code", addEventListener() {} }],
    ["lang-select", null],
    ["main", null],
  ]);
  const document = {
    title: "Receive with TotalDrop",
    documentElement: { lang: "en", dir: "ltr" },
    body: { dataset: { page: "receive" } },
    querySelector: () => null,
    querySelectorAll: () => [],
    getElementById: (id) => elements.get(id) ?? null,
    createElement: () => ({ setAttribute() {}, removeAttribute() {} }),
  };
  const window = {
    document,
    navigator: { languages: ["en"], language: "en", ...nav },
    location: { hash, reload() {} },
    addEventListener() {},
    matchMedia: () => ({ matches: false }),
    getSelection: () => ({ selectAllChildren() {} }),
  };
  window.window = window;
  window.globalThis = window;
  vm.runInNewContext(appJs, window);
  return {
    hasCode: !elements.get("r-has-code").hidden,
    noCode: !elements.get("r-no-code").hidden,
    code: elements.get("r-code").textContent,
    href: elements.get("r-open").href,
  };
}

// Reconstructs the data URI that Android's Intent.parseUri(..., URI_INTENT_SCHEME)
// builds from intent://<authority>/<path>#Intent;scheme=<scheme>;package=<pkg>;end.
function parseAndroidIntentUri(url) {
  const m = /^intent:\/\/([^/#]+)(\/[^#]*)#Intent;scheme=([^;]+);package=([^;]+);end$/.exec(url);
  assert.ok(m, `not a package-pinned intent URL: ${url}`);
  const [, authority, path, scheme, pkg] = m;
  return { dataUri: `${scheme}://${authority}${path}`, scheme, authority, pkg };
}

test("the intent URL names the Android app's package and manifest filter", () => {
  const { ANDROID_PACKAGE, receiveHref } = loadHelpers();
  const appId = gradle.match(/applicationId\s*=\s*"([^"]+)"/)?.[1];
  assert.equal(ANDROID_PACKAGE, "app.totaldrop");
  assert.equal(ANDROID_PACKAGE, appId, "matches build.gradle.kts applicationId");
  assert.equal(ANDROID_PACKAGE, assetlinks[0].target.package_name, "matches assetlinks.json");

  const parsed = parseAndroidIntentUri(receiveHref("7-harbor-rocket", { userAgent: "Mozilla/5.0 (Android 14; Mobile)" }));
  assert.equal(parsed.pkg, ANDROID_PACKAGE);
  assert.equal(parsed.dataUri, "totaldrop://r/7-harbor-rocket");
  const filters = manifest.match(/<intent-filter\b[\s\S]*?<\/intent-filter>/g) ?? [];
  const schemeFilters = filters.filter((f) => new RegExp(`android:scheme="${parsed.scheme}"`).test(f));
  assert.equal(schemeFilters.length, 1, "one totaldrop:// intent-filter");
  const [schemeFilter] = schemeFilters;
  assert.match(schemeFilter, /android\.intent\.action\.VIEW/);
  assert.match(schemeFilter, /android\.intent\.category\.DEFAULT/);
  assert.match(schemeFilter, /android\.intent\.category\.BROWSABLE/);
  const data = schemeFilter.match(/<data\b[^>]*\/>/g) ?? [];
  assert.deepEqual(data, [
    `<data android:scheme="${parsed.scheme}" android:host="${parsed.authority}"/>`,
  ]);
  assert.match(
    mainActivity,
    new RegExp(`uri\\.scheme\\s*==\\s*"${parsed.scheme}"\\s*&&\\s*uri\\.host\\s*==\\s*"${parsed.authority}"`),
    "MainActivity.kt accepts the reconstructed data URI",
  );
});

test("Android browsers get the intent URL, including desktop-site and tablet modes", () => {
  const { isAndroid, receiveHref } = loadHelpers();
  const code = "7-harbor-rocket";
  const wantIntent = "intent://r/7-harbor-rocket#Intent;scheme=totaldrop;package=app.totaldrop;end";

  const androidCases = [
    [
      "Chrome on Android",
      {
        userAgent: "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Mobile Safari/537.36",
        userAgentData: { platform: "Android" },
        platform: "Linux armv81",
      },
    ],
    [
      "Chrome on Android with 'Desktop site' on / Android tablet default (UA and client hint say Linux, platform stays Linux armv81)",
      {
        userAgent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
        userAgentData: { platform: "Linux" },
        platform: "Linux armv81",
      },
    ],
    [
      "Chromium with kAndroidDesktopUAPlatform enabled (UA says X11 Linux, client hint says Android)",
      {
        userAgent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
        userAgentData: { platform: "Android" },
      },
    ],
    [
      "Firefox for Android",
      {
        userAgent: "Mozilla/5.0 (Android 14; Mobile; rv:133.0) Gecko/133.0 Firefox/133.0",
        platform: "Linux armv81",
      },
    ],
    [
      "Firefox for Android with 'Desktop site' on (UA says X11 Linux, platform stays Linux armv81)",
      {
        userAgent: "Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0",
        platform: "Linux armv81",
      },
    ],
    [
      "Samsung Internet on Android",
      {
        userAgent: "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) SamsungBrowser/27.0 Chrome/125.0.0.0 Mobile Safari/537.36",
        platform: "Linux armv81",
      },
    ],
  ];

  for (const [label, nav] of androidCases) {
    assert.equal(isAndroid(nav), true, label);
    assert.equal(receiveHref(code, nav), wantIntent, label);
  }
});

test("non-Android browsers keep totaldrop://r/<code>", () => {
  const { isAndroid, receiveHref } = loadHelpers();
  const code = "7-harbor-rocket";
  const wantScheme = "totaldrop://r/7-harbor-rocket";

  const otherCases = [
    [
      "Linux desktop Firefox",
      {
        userAgent: "Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0",
        platform: "Linux x86_64",
      },
    ],
    [
      "Linux touchscreen laptop in Chromium",
      {
        userAgent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
        userAgentData: { platform: "Linux" },
        platform: "Linux x86_64",
        maxTouchPoints: 10,
      },
    ],
    [
      "ARM64 Linux desktop (Linux aarch64)",
      {
        userAgent: "Mozilla/5.0 (X11; Linux aarch64; rv:133.0) Gecko/20100101 Firefox/133.0",
        platform: "Linux aarch64",
      },
    ],
    [
      "32-bit ARM Linux desktop (Linux armv8l, lowercase L not digit 1)",
      {
        userAgent: "Mozilla/5.0 (X11; Linux armv8l) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
        userAgentData: { platform: "Linux" },
        platform: "Linux armv8l",
      },
    ],
    [
      "macOS Safari",
      {
        userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15",
        platform: "MacIntel",
      },
    ],
    [
      "Windows Chrome",
      {
        userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
        userAgentData: { platform: "Windows" },
        platform: "Win32",
      },
    ],
    [
      "iPhone Safari",
      {
        userAgent: "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1",
        platform: "iPhone",
      },
    ],
    [
      "ChromeOS",
      {
        userAgent: "Mozilla/5.0 (X11; CrOS x86_64 14541.0.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
        userAgentData: { platform: "Chrome OS" },
        platform: "Linux x86_64",
      },
    ],
  ];

  for (const [label, nav] of otherCases) {
    assert.equal(isAndroid(nav), false, label);
    assert.equal(receiveHref(code, nav), wantScheme, label);
  }
});

test("percent-encoding stays identical and cannot break the #Intent fragment", () => {
  const { receiveHref } = loadHelpers();
  const android = { userAgentData: { platform: "Android" } };
  const desktop = { userAgentData: { platform: "Linux" } };

  for (const sample of ["7-harbor-rocket", "12-zoo-zone-able", "7-a#b;package=evil;end", "7-café%20test"]) {
    const desktopUrl = receiveHref(sample, desktop);
    const androidUrl = receiveHref(sample, android);
    const parsed = parseAndroidIntentUri(androidUrl);
    assert.equal(parsed.pkg, "app.totaldrop");
    assert.equal(parsed.dataUri, desktopUrl, "Android delivers the exact same totaldrop://r/ URI");
    assert.equal(decodeURIComponent(desktopUrl.slice("totaldrop://r/".length)), sample);
  }
});

test("the /r page normalizes the fragment and sets #r-open for each platform", () => {
  const android = { userAgent: "Mozilla/5.0 (Android 14; Mobile; rv:133.0) Gecko/133.0 Firefox/133.0" };
  const linux = { userAgent: "Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0" };

  for (const [hash, expectedCode] of [
    ["#7-harbor-rocket", "7-harbor-rocket"],
    ["#7-HİDDEN-İSLAND", "7-hidden-island"],
    ["#%D9%A7-harbor%2Drocket", "7-harbor-rocket"],
    ["#۷-harbor-rocket-able", "7-harbor-rocket-able"],
  ]) {
    const onAndroid = runReceivePage(hash, android);
    assert.equal(onAndroid.hasCode, true, hash);
    assert.equal(onAndroid.noCode, false, hash);
    assert.equal(onAndroid.code, expectedCode, hash);
    assert.equal(
      onAndroid.href,
      `intent://r/${expectedCode}#Intent;scheme=totaldrop;package=app.totaldrop;end`,
    );

    const onLinux = runReceivePage(hash, linux);
    assert.equal(onLinux.code, expectedCode, hash);
    assert.equal(onLinux.href, `totaldrop://r/${expectedCode}`);
  }

  for (const bad of ["", "#", "#not-a-code", "#7-ab-cd", "#%E0%A4%A"]) {
    const res = runReceivePage(bad, android);
    assert.equal(res.hasCode, false, bad);
    assert.equal(res.noCode, true, bad);
    assert.equal(res.href, "#", bad);
  }
});
