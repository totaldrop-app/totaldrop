// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test site/tools/test-layout-320.mjs
//
// Unit tests, static CSS/HTML guards, and a real headless Chrome anti-regression
// check for the 320 px horizontal overflow verifier (#481).
import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { TAGS } from "./build.mjs";
import {
  VIEWPORT_WIDTH,
  buildChecks,
  checkDistDir,
  checkPages,
  evaluateOverflow,
  findChrome,
  formatFailure,
  parseCliArgs,
} from "./check-layout-320.mjs";

const pub = new URL("../public/", import.meta.url).pathname;
const css = readFileSync(join(pub, "styles.css"), "utf8").replace(/\/\*[\s\S]*?\*\//g, "");
const downloadHtml = readFileSync(join(pub, "download/index.html"), "utf8");
const checkerModuleUrl = pathToFileURL(new URL("./check-layout-320.mjs", import.meta.url).pathname).href;

function parseDeclarations(body) {
  const found = {};
  for (const decl of body.split(";")) {
    const i = decl.indexOf(":");
    if (i > 0) found[decl.slice(0, i).trim()] = decl.slice(i + 1).trim();
  }
  return found;
}

function collectRules(blockText) {
  const rules = [];
  let i = 0;
  while (i < blockText.length) {
    const open = blockText.indexOf("{", i);
    if (open === -1) break;
    const header = blockText.slice(i, open).trim();
    let depth = 1;
    let pos = open + 1;
    while (pos < blockText.length && depth > 0) {
      if (blockText[pos] === "{") depth++;
      else if (blockText[pos] === "}") depth--;
      pos++;
    }
    rules.push({ header, body: blockText.slice(open + 1, pos - 1) });
    i = pos;
  }
  return rules;
}

function declarationsIn(rules, selector) {
  const found = {};
  for (const { header, body } of rules) {
    if (header.startsWith("@")) continue;
    const selectors = header.split(",").map((s) => s.trim());
    if (header !== selector && !selectors.includes(selector)) continue;
    Object.assign(found, parseDeclarations(body));
  }
  return found;
}

const topLevelRules = collectRules(css);

function declarations(selector) {
  return declarationsIn(topLevelRules, selector);
}

function mediaDeclarations(query, selector) {
  const found = {};
  const target = `@media ${query}`.replace(/\s+/g, " ");
  for (const { header, body } of topLevelRules) {
    if (header.replace(/\s+/g, " ") !== target) continue;
    Object.assign(found, declarationsIn(collectRules(body), selector));
  }
  return found;
}

test("buildChecks produces exact 35 x 9 = 315 checks covering all tags and pages (#481)", () => {
  assert.equal(TAGS.length, 35, "expected 35 languages in site/public/lang.js");
  const checks = buildChecks(TAGS);
  assert.equal(checks.length, 315, "expected 35 languages x 9 pages = 315 checks");

  const expectedPages = ["home", "download", "roadmap", "donate", "team", "brand", "privacy", "receive", "notfound"];
  for (const tag of TAGS) {
    const forTag = checks.filter((c) => c.tag === tag);
    assert.deepEqual(
      forTag.map((c) => c.page),
      expectedPages,
      `${tag}: expected all 9 pages in order`,
    );
    for (const c of forTag) {
      assert.equal(c.expectedLang, tag);
      assert.equal(c.dynamic, c.page === "receive" || c.page === "notfound");
    }
  }

  const en = Object.fromEntries(checks.filter((c) => c.tag === "en").map((c) => [c.page, c.path]));
  assert.deepEqual(en, {
    home: "/",
    download: "/download/",
    roadmap: "/roadmap/",
    donate: "/donate/",
    team: "/team/",
    brand: "/brand/",
    privacy: "/privacy",
    receive: "/r#7-harbor-rocket",
    notfound: "/404.html",
  });

  const ptBr = Object.fromEntries(checks.filter((c) => c.tag === "pt-BR").map((c) => [c.page, c.path]));
  assert.equal(ptBr.home, "/pt-BR/");
  assert.equal(ptBr.privacy, "/privacy/pt-BR");
  assert.equal(ptBr.receive, "/r#7-harbor-rocket");
  assert.equal(ptBr.notfound, "/404.html");
});

test("evaluateOverflow catches scrollWidth, bodyScrollWidth, clientWidth, and lang mismatches", () => {
  const pass = evaluateOverflow({
    docScrollWidth: 320,
    docClientWidth: 320,
    bodyScrollWidth: 320,
    expectedLang: "ru",
    actualLang: "ru",
  });
  assert.equal(pass.ok, true);
  assert.equal(pass.overflowPx, 0);
  assert.deepEqual(pass.reasons, []);

  const docOver = evaluateOverflow({
    docScrollWidth: 372,
    docClientWidth: 320,
    bodyScrollWidth: 320,
    expectedLang: "ru",
    actualLang: "ru",
  });
  assert.equal(docOver.ok, false);
  assert.equal(docOver.overflowPx, 52);
  assert.match(docOver.reasons.join(" "), /documentElement\.scrollWidth 372 > 320/);

  const bodyOver = evaluateOverflow({
    docScrollWidth: 320,
    docClientWidth: 320,
    bodyScrollWidth: 350,
    expectedLang: "fr",
    actualLang: "fr",
  });
  assert.equal(bodyOver.ok, false);
  assert.equal(bodyOver.overflowPx, 30);
  assert.match(bodyOver.reasons.join(" "), /body\.scrollWidth 350 > 320/);

  const wrongViewport = evaluateOverflow({
    docScrollWidth: 320,
    docClientWidth: 300,
    bodyScrollWidth: 320,
    expectedLang: "en",
    actualLang: "en",
  });
  assert.equal(wrongViewport.ok, false);
  assert.match(wrongViewport.reasons.join(" "), /clientWidth 300 !== 320/);

  const wrongLang = evaluateOverflow({
    docScrollWidth: 320,
    docClientWidth: 320,
    bodyScrollWidth: 320,
    expectedLang: "zh-Hant",
    actualLang: "en",
  });
  assert.equal(wrongLang.ok, false);
  assert.match(wrongLang.reasons.join(" "), /html\[lang\] "en" !== "zh-Hant"/);
});

test("findChrome and checkDistDir fail loudly on missing binaries or incomplete builds", () => {
  assert.throws(
    () => findChrome({ CHROME_BIN: "/nonexistent/totaldrop-chrome-binary" }),
    /CHROME_BIN/,
  );
  assert.equal(findChrome({}, ["/nonexistent/a", "/nonexistent/b"]), null);

  assert.throws(() => checkDistDir("/nonexistent/totaldrop-missing-dist"), /built site folder not found/);

  const incomplete = mkdtempSync(join(tmpdir(), "td-test-incomplete-dist-"));
  try {
    writeFileSync(join(incomplete, "index.html"), "<!doctype html><html lang=\"en\"></html>");
    assert.throws(() => checkDistDir(incomplete), /incomplete/);
  } finally {
    rmSync(incomplete, { recursive: true, force: true });
  }
});

test("styles.css guards prevent 320 px horizontal overflow without regressing desktop layout (#481)", () => {
  const headings = declarations("h1, h2, h3");
  assert.equal(headings["overflow-wrap"], "break-word");
  assert.equal(headings["hyphens"], undefined, "hyphens: auto must not apply globally on desktop viewports");

  const mobileHeadings = mediaDeclarations("(max-width: 40rem)", "h1, h2, h3");
  assert.equal(mobileHeadings["hyphens"], "auto");

  const h1 = declarations("h1");
  assert.equal(h1["font-size"], "clamp(2.6rem, 6.4vw, 4.75rem)");

  assert.equal(declarations(".prose")["grid-template-columns"], "minmax(0, 1fr)");
  assert.equal(declarations(".prose article")["grid-template-columns"], "minmax(0, 1fr)");

  const areaHead = declarations(".area-head");
  assert.equal(areaHead["flex-wrap"], "wrap");
  const areaHeadTitle = declarations(".area-head h3");
  assert.equal(areaHeadTitle["flex"], "1 1 8rem");
  assert.equal(areaHeadTitle["min-width"], "0");
  assert.equal(declarations(".area-head .stat")["margin-inline-start"], "auto");
});

test("parseCliArgs accumulates deduplicated --lang tags and rejects empty or invalid flags (#481)", () => {
  assert.deepEqual(parseCliArgs([]), { doBuild: false, distDir: null, onlyLangs: [] });
  assert.deepEqual(parseCliArgs(["--lang", "cs", "--lang", "fr", "--lang", "ru"]), {
    doBuild: false,
    distDir: null,
    onlyLangs: ["cs", "fr", "ru"],
  });
  assert.deepEqual(parseCliArgs(["--lang", "cs", "--lang=fr", "--lang", "cs", "--lang=ru"]), {
    doBuild: false,
    distDir: null,
    onlyLangs: ["cs", "fr", "ru"],
  });
  assert.deepEqual(parseCliArgs(["--build", "--lang", "pt-BR"]), {
    doBuild: true,
    distDir: null,
    onlyLangs: ["pt-BR"],
  });
  assert.deepEqual(parseCliArgs(["--dist=/tmp/site-dist", "--lang=ar"]), {
    doBuild: false,
    distDir: "/tmp/site-dist",
    onlyLangs: ["ar"],
  });

  for (const badArgs of [
    ["--lang="],
    ["--lang", ""],
    ["--lang", "   "],
    ["--lang"],
    ["--lang", "--build"],
    ["--dist="],
    ["--dist", ""],
    ["--dist", "   "],
    ["--dist"],
    ["--dist", "--lang", "en"],
    ["--lang", "zz"],
    ["--lang", "cs", "--lang", "zz"],
    ["--build", "--dist", "/tmp/site-dist"],
    ["--unknown-flag"],
  ]) {
    assert.throws(() => parseCliArgs(badArgs), `expected parseCliArgs(${JSON.stringify(badArgs)}) to throw`);
  }
});

test("download page Requirements table wrapper is a keyboard-focusable named region (#481)", () => {
  const wrappers = [...downloadHtml.matchAll(/<div class="compare-wrap"([^>]*)>/g)];
  assert.equal(wrappers.length, 1, "expected Requirements table wrapper on /download/");
  const attrs = wrappers[0][1];
  assert.match(attrs, /\btabindex="0"/);
  assert.match(attrs, /\brole="region"/);
  assert.match(attrs, /\baria-labelledby="dl-req"/);
  assert.ok(downloadHtml.includes('id="dl-req"'), "aria-labelledby target #dl-req not found");
});

test("CDP checker detects overflowing LTR/RTL fixtures without blaming .skip and passes clean 320 px (#481)", async () => {
  const chromeBin = findChrome();
  assert.ok(chromeBin, "expected headless Chrome/Chromium binary to be installed");

  const fixtureDir = mkdtempSync(join(tmpdir(), "td-test-layout-320-"));
  try {
    writeFileSync(
      join(fixtureDir, "overflow.html"),
      '<!doctype html><html lang="en"><head><meta charset="utf-8">' +
      '<meta name="viewport" content="width=device-width, initial-scale=1">' +
      "<style>body{margin:0}</style></head><body>" +
      '<a class="skip" href="#wide" style="position:absolute;inset-inline-start:-999px;top:0">Skip to content</a>' +
      '<span class="visually-hidden" style="position:absolute;width:1px;height:1px;overflow:hidden">hidden</span>' +
      '<div id="wide" style="width:400px;height:16px">overflowing box</div>' +
      "</body></html>",
    );
    writeFileSync(
      join(fixtureDir, "rtl-overflow.html"),
      '<!doctype html><html lang="ar" dir="rtl"><head><meta charset="utf-8">' +
      '<meta name="viewport" content="width=device-width, initial-scale=1">' +
      "<style>body{margin:0}</style></head><body>" +
      '<a class="skip" href="#rtlwide" style="position:absolute;inset-inline-start:-999px;top:0">Skip to content</a>' +
      '<div id="rtlwide" style="width:360px;height:16px">rtl overflowing box</div>' +
      "</body></html>",
    );
    writeFileSync(
      join(fixtureDir, "clean.html"),
      '<!doctype html><html lang="en"><head><meta charset="utf-8">' +
      '<meta name="viewport" content="width=device-width, initial-scale=1">' +
      "<style>body{margin:0}</style></head><body>" +
      '<a class="skip" href="#fit" style="position:absolute;inset-inline-start:-999px;top:0">Skip to content</a>' +
      '<div id="fit" style="width:100%;height:16px">fits at 320px</div>' +
      "</body></html>",
    );

    const report = await checkPages({
      rootDir: fixtureDir,
      chromeBin,
      checks: [
        { tag: "en", page: "overflow-fixture", path: "/overflow.html", dynamic: false, expectedLang: "en" },
        { tag: "en", page: "clean-fixture", path: "/clean.html", dynamic: false, expectedLang: "en" },
        { tag: "ar", page: "rtl-overflow-fixture", path: "/rtl-overflow.html", dynamic: false, expectedLang: "ar" },
      ],
    });

    assert.equal(report.total, 3);
    assert.equal(report.passed, 1);
    assert.equal(report.failed, 2);

    const fail = report.failures.find((f) => f.page === "overflow-fixture");
    assert.ok(fail, "expected LTR overflow failure");
    assert.equal(fail.docClientWidth, VIEWPORT_WIDTH);
    assert.equal(fail.docScrollWidth, 400);
    assert.equal(fail.overflowPx, 80);
    assert.equal(fail.widest?.id, "wide");
    assert.match(formatFailure(fail), /doc=400\/320/);
    assert.match(formatFailure(fail), /#wide/);

    const rtlFail = report.failures.find((f) => f.page === "rtl-overflow-fixture");
    assert.ok(rtlFail, "expected RTL overflow failure");
    assert.equal(rtlFail.docScrollWidth, 360);
    assert.equal(rtlFail.overflowPx, 40);
    assert.equal(rtlFail.widest?.id, "rtlwide");
    assert.match(formatFailure(rtlFail), /#rtlwide/);

    const pass = report.results.find((r) => r.page === "clean-fixture");
    assert.equal(pass?.ok, true);
    assert.equal(pass?.docScrollWidth, 320);
    assert.equal(pass?.bodyScrollWidth, 320);
    assert.equal(pass?.docClientWidth, 320);
  } finally {
    rmSync(fixtureDir, { recursive: true, force: true });
  }
});

function runMockCdpScenario(mode) {
  const workDir = mkdtempSync(join(tmpdir(), "td-test-mock-cdp-"));
  try {
    writeFileSync(join(workDir, "launches.txt"), "0");
    writeFileSync(
      join(workDir, "ok.html"),
      '<!doctype html><html lang="en"><head><meta charset="utf-8"></head><body>ok</body></html>',
    );

    const mockServerScript = join(workDir, "mock-cdp.mjs");
    writeFileSync(
      mockServerScript,
      `import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";

const mode = ${JSON.stringify(mode)};
const countFile = ${JSON.stringify(join(workDir, "launches.txt"))};
const attempt = Number(readFileSync(countFile, "utf8").trim()) + 1;
writeFileSync(countFile, String(attempt));

function sendFrame(socket, opcode, payloadBuf) {
  const len = payloadBuf.length;
  let header;
  if (len < 126) {
    header = Buffer.from([0x80 | opcode, len]);
  } else if (len < 65536) {
    header = Buffer.alloc(4);
    header[0] = 0x80 | opcode;
    header[1] = 126;
    header.writeUInt16BE(len, 2);
  } else {
    header = Buffer.alloc(10);
    header[0] = 0x80 | opcode;
    header[1] = 127;
    header.writeBigUInt64BE(BigInt(len), 2);
  }
  socket.write(Buffer.concat([header, payloadBuf]));
}

function sendJson(socket, obj) {
  sendFrame(socket, 0x1, Buffer.from(JSON.stringify(obj), "utf8"));
}

const server = createServer((_req, res) => {
  res.writeHead(404);
  res.end();
});

function handleMessage(socket, msg) {
  const { id, method, sessionId } = msg;
  if (method === "Browser.getVersion") {
    sendJson(socket, { id, result: { userAgent: "HeadlessChrome/mock" } });
    return;
  }
  if (method === "Target.createBrowserContext") {
    sendJson(socket, { id, result: { browserContextId: "ctx-1" } });
    return;
  }
  if (method === "Target.createTarget") {
    sendJson(socket, { id, result: { targetId: "tgt-1" } });
    return;
  }
  if (method === "Target.attachToTarget") {
    sendJson(socket, { id, result: { sessionId: "sess-1" } });
    return;
  }
  if (method === "Browser.close") {
    sendJson(socket, { id, result: {} });
    socket.end();
    server.close();
    setTimeout(() => process.exit(0), 10);
    return;
  }
  if (method === "Page.navigate") {
    if (mode === "drop_on_navigate" && attempt === 1) {
      socket.destroy();
      server.close();
      process.exit(1);
    }
    if (mode === "target_closed" && attempt === 1) {
      sendJson(socket, { id, error: { code: -32000, message: "Target closed" } });
      return;
    }
    if (mode === "err_aborted") {
      sendJson(socket, { id, result: { frameId: "f1", errorText: "net::ERR_ABORTED" } });
      return;
    }
    sendJson(socket, { id, result: { frameId: "f1" } });
    sendJson(socket, { method: "Page.loadEventFired", sessionId, params: {} });
    return;
  }
  if (method === "Runtime.evaluate") {
    sendJson(socket, {
      id,
      result: {
        result: {
          value: {
            docScrollWidth: 320,
            docClientWidth: 320,
            bodyScrollWidth: 320,
            actualLang: "en",
            actualDir: "ltr",
            url: "http://127.0.0.1/ok.html",
            widest: null,
          },
        },
      },
    });
    return;
  }
  sendJson(socket, { id, result: {} });
}

server.on("upgrade", (req, socket) => {
  const key = req.headers["sec-websocket-key"];
  const accept = createHash("sha1")
    .update(key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11")
    .digest("base64");
  socket.write(
    "HTTP/1.1 101 Switching Protocols\\r\\n" +
    "Upgrade: websocket\\r\\n" +
    "Connection: Upgrade\\r\\n" +
    "Sec-WebSocket-Accept: " + accept + "\\r\\n\\r\\n",
  );
  let buf = Buffer.alloc(0);
  socket.on("data", (chunk) => {
    buf = Buffer.concat([buf, chunk]);
    while (buf.length >= 2) {
      const opcode = buf[0] & 0x0f;
      const masked = (buf[1] & 0x80) !== 0;
      let len = buf[1] & 0x7f;
      let offset = 2;
      if (len === 126) {
        if (buf.length < 4) return;
        len = buf.readUInt16BE(2);
        offset = 4;
      } else if (len === 127) {
        if (buf.length < 10) return;
        len = Number(buf.readBigUInt64BE(2));
        offset = 10;
      }
      const maskLen = masked ? 4 : 0;
      if (buf.length < offset + maskLen + len) return;
      const mask = masked ? buf.subarray(offset, offset + 4) : null;
      const payload = Buffer.from(buf.subarray(offset + maskLen, offset + maskLen + len));
      buf = buf.subarray(offset + maskLen + len);
      if (mask) {
        for (let i = 0; i < payload.length; i++) payload[i] ^= mask[i & 3];
      }
      if (opcode === 0x8) {
        socket.end();
        return;
      }
      if (opcode === 0x9) {
        sendFrame(socket, 0xA, payload);
        continue;
      }
      if (opcode === 0x1) {
        handleMessage(socket, JSON.parse(payload.toString("utf8")));
      }
    }
  });
});

server.listen(0, "127.0.0.1", () => {
  const { port } = server.address();
  process.stderr.write("DevTools listening on ws://127.0.0.1:" + port + "/devtools/browser/mock\\n");
});
`,
    );

    const mockChromeBin = join(workDir, "mock-chrome");
    writeFileSync(
      mockChromeBin,
      `#!/bin/sh\nexec "${process.execPath}" "${mockServerScript}" "$@"\n`,
    );
    chmodSync(mockChromeBin, 0o755);

    const runnerScript = join(workDir, "runner.mjs");
    writeFileSync(
      runnerScript,
      `import { readFileSync } from "node:fs";
import { checkPages } from ${JSON.stringify(checkerModuleUrl)};

const unhandled = [];
process.on("unhandledRejection", (reason) => {
  unhandled.push(reason instanceof Error ? reason.message : String(reason));
});

let report = null;
let caughtError = null;
try {
  report = await checkPages({
    rootDir: ${JSON.stringify(workDir)},
    chromeBin: ${JSON.stringify(mockChromeBin)},
    checks: [{ tag: "en", page: "ok", path: "/ok.html", dynamic: false, expectedLang: "en" }],
  });
} catch (err) {
  caughtError = err instanceof Error ? err.message : String(err);
}

const launches = Number(readFileSync(${JSON.stringify(join(workDir, "launches.txt"))}, "utf8").trim());
process.stdout.write(JSON.stringify({
  passed: report ? report.passed : 0,
  failed: report ? report.failed : 0,
  caughtError,
  launches,
  unhandled,
}) + "\\n");
`,
    );

    const start = Date.now();
    const res = spawnSync(process.execPath, [runnerScript], {
      encoding: "utf8",
      timeout: 10000,
    });
    const elapsedMs = Date.now() - start;
    assert.equal(res.error, undefined, `mock CDP scenario ${mode} runner error: ${res.error?.message}`);
    assert.equal(res.signal, null, `mock CDP scenario ${mode} killed by signal ${res.signal} (stderr: ${res.stderr})`);
    assert.equal(res.status, 0, `mock CDP scenario ${mode} exited ${res.status} (stderr: ${res.stderr})`);
    assert.ok(elapsedMs < 5000, `mock CDP scenario ${mode} stalled for ${elapsedMs} ms (leaked timer?)`);
    return { ...JSON.parse(res.stdout.trim()), elapsedMs };
  } finally {
    rmSync(workDir, { recursive: true, force: true });
  }
}

test("mock CDP WebSocket drop mid-Page.navigate retries cleanly without UnhandledPromiseRejection (#481)", () => {
  const result = runMockCdpScenario("drop_on_navigate");
  assert.deepEqual(result.unhandled, []);
  assert.equal(result.caughtError, null);
  assert.equal(result.launches, 2);
  assert.equal(result.passed, 1);
  assert.equal(result.failed, 0);
});

test("mock CDP Target closed and net::ERR_ABORTED cancel waitForEvent timers promptly without event-loop stall (#481)", () => {
  const targetClosed = runMockCdpScenario("target_closed");
  assert.deepEqual(targetClosed.unhandled, []);
  assert.equal(targetClosed.caughtError, null);
  assert.equal(targetClosed.launches, 2);
  assert.equal(targetClosed.passed, 1);
  assert.equal(targetClosed.failed, 0);

  const errAborted = runMockCdpScenario("err_aborted");
  assert.deepEqual(errAborted.unhandled, []);
  assert.equal(errAborted.launches, 3);
  assert.match(errAborted.caughtError || "", /Chrome crashed 3 times.*net::ERR_ABORTED/);
});
