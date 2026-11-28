// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Verifies that none of the site's 9 pages scrolls horizontally at 320 px in
// any of the 35 languages (#481):
//   node site/tools/check-layout-320.mjs           check built site/dist
//   node site/tools/check-layout-320.mjs --build   build into a temp folder and check
//   node site/tools/check-layout-320.mjs --dist <dir> [--lang <tag> ...]
//
// Zero external packages: serves the built site on 127.0.0.1 and drives
// headless Chrome/Chromium over the Chrome DevTools Protocol using Node's
// built-in WebSocket client. Tested by: node --test site/tools/test-layout-320.mjs
import { spawn } from "node:child_process";
import { accessSync, constants as fsConstants, existsSync, mkdtempSync, readFileSync, realpathSync, rmSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { delimiter, dirname, extname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { TAGS, build } from "./build.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const siteRoot = realpathSync(resolve(here, ".."));

export const VIEWPORT_WIDTH = 320;
export const VIEWPORT_HEIGHT = 800;
export const BATCH_SIZE = 5;
const PAGE_TIMEOUT_MS = 20000;
const CHROME_START_TIMEOUT_MS = 15000;

const MIME_TYPES = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".woff2": "font/woff2",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".ico": "image/x-icon",
  ".webp": "image/webp",
  ".xml": "application/xml; charset=utf-8",
  ".txt": "text/plain; charset=utf-8",
};

const STANDARD_CHROME_PATHS = [
  "/usr/bin/google-chrome",
  "/usr/bin/google-chrome-stable",
  "/usr/bin/chromium",
  "/usr/bin/chromium-browser",
  "/snap/bin/chromium",
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/Applications/Chromium.app/Contents/MacOS/Chromium",
];

const CHROME_PATH_NAMES = [
  "google-chrome",
  "google-chrome-stable",
  "chromium",
  "chromium-browser",
];

/** Builds the exact matrix of 9 pages x N languages (315 checks for 35 tags). */
export function buildChecks(tags = TAGS) {
  const checks = [];
  for (const tag of tags) {
    const prefix = tag === "en" ? "" : `/${tag}`;
    checks.push(
      { tag, page: "home", path: `${prefix}/`, dynamic: false, expectedLang: tag },
      { tag, page: "download", path: `${prefix}/download/`, dynamic: false, expectedLang: tag },
      { tag, page: "roadmap", path: `${prefix}/roadmap/`, dynamic: false, expectedLang: tag },
      { tag, page: "donate", path: `${prefix}/donate/`, dynamic: false, expectedLang: tag },
      { tag, page: "team", path: `${prefix}/team/`, dynamic: false, expectedLang: tag },
      { tag, page: "brand", path: `${prefix}/brand/`, dynamic: false, expectedLang: tag },
      { tag, page: "privacy", path: tag === "en" ? "/privacy" : `/privacy/${tag}`, dynamic: false, expectedLang: tag },
      { tag, page: "receive", path: "/r#7-harbor-rocket", dynamic: true, expectedLang: tag },
      { tag, page: "notfound", path: "/404.html", dynamic: true, expectedLang: tag },
    );
  }
  return checks;
}

/** Evaluates measured viewport and scroll widths against the 320 px limit. */
export function evaluateOverflow(measurement, viewportWidth = VIEWPORT_WIDTH) {
  const reasons = [];
  const { docScrollWidth, docClientWidth, bodyScrollWidth, expectedLang, actualLang } = measurement;

  if (!Number.isFinite(docScrollWidth) || !Number.isFinite(docClientWidth) || !Number.isFinite(bodyScrollWidth)) {
    reasons.push(`invalid layout measurement: ${JSON.stringify({ docScrollWidth, docClientWidth, bodyScrollWidth })}`);
    return { ok: false, reasons, overflowPx: Number.POSITIVE_INFINITY };
  }
  if (docClientWidth !== viewportWidth) {
    reasons.push(`clientWidth ${docClientWidth} !== ${viewportWidth}`);
  }
  if (docScrollWidth > viewportWidth) {
    reasons.push(`documentElement.scrollWidth ${docScrollWidth} > ${viewportWidth} (+${docScrollWidth - viewportWidth} px)`);
  }
  if (bodyScrollWidth > viewportWidth) {
    reasons.push(`body.scrollWidth ${bodyScrollWidth} > ${viewportWidth} (+${bodyScrollWidth - viewportWidth} px)`);
  }
  if (expectedLang != null && actualLang !== expectedLang) {
    reasons.push(`html[lang] "${actualLang}" !== "${expectedLang}"`);
  }
  const overflowPx = Math.max(0, docScrollWidth - viewportWidth, bodyScrollWidth - viewportWidth);
  return { ok: reasons.length === 0, reasons, overflowPx };
}

function isExecutableFile(filePath) {
  try {
    if (!statSync(filePath).isFile()) return false;
    accessSync(filePath, fsConstants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function resolveOnPath(name, pathValue) {
  if (!pathValue) return null;
  for (const dir of pathValue.split(delimiter)) {
    if (!dir) continue;
    const candidate = join(dir, name);
    if (isExecutableFile(candidate)) return candidate;
  }
  return null;
}

/** Finds a headless Chrome/Chromium executable or returns null if absent. */
export function findChrome(env = process.env, explicitCandidates = null) {
  if (Array.isArray(explicitCandidates)) {
    for (const candidate of explicitCandidates) {
      if (candidate && isExecutableFile(candidate)) return candidate;
    }
    return null;
  }
  const override = env && typeof env.CHROME_BIN === "string" ? env.CHROME_BIN.trim() : "";
  if (override) {
    if (isExecutableFile(override)) return override;
    if (!override.includes("/") && !override.includes("\\")) {
      const resolved = resolveOnPath(override, env.PATH || "");
      if (resolved) return resolved;
    }
    throw new Error(`CHROME_BIN is set to "${override}", but it is not an executable file`);
  }
  for (const candidate of STANDARD_CHROME_PATHS) {
    if (isExecutableFile(candidate)) return candidate;
  }
  const pathValue = env && typeof env.PATH === "string" ? env.PATH : "";
  for (const name of CHROME_PATH_NAMES) {
    const resolved = resolveOnPath(name, pathValue);
    if (resolved) return resolved;
  }
  return null;
}

/** Verifies that the built site directory exists and contains expected pages. */
export function checkDistDir(distDir) {
  if (typeof distDir !== "string" || !distDir.trim()) {
    throw new Error("dist directory path is empty");
  }
  const resolved = resolve(distDir);
  if (!existsSync(resolved) || !statSync(resolved).isDirectory()) {
    throw new Error(`built site folder not found at ${resolved}; run "node site/tools/build.mjs" or pass --build`);
  }
  const required = ["index.html", "privacy.html", join("r", "index.html"), "404.html", join("tr", "index.html")];
  const missing = required.filter((rel) => !existsSync(join(resolved, rel)));
  if (missing.length > 0) {
    throw new Error(
      `built site folder ${resolved} is incomplete (missing ${missing.join(", ")}); run "node site/tools/build.mjs" or pass --build`,
    );
  }
  return resolved;
}

function within(parent, child) {
  const rel = relative(parent, child);
  return rel === "" || (rel !== ".." && !rel.startsWith(".." + sep) && !isAbsolute(rel));
}

function resolveStaticFile(rootReal, rawPathname) {
  let pathname;
  try {
    pathname = decodeURIComponent(rawPathname);
  } catch {
    return null;
  }
  const trimmed = pathname.replace(/^\/+/, "");
  const base = resolve(rootReal, trimmed);
  if (!within(rootReal, base)) return null;

  const candidates = [];
  if (pathname.endsWith("/")) {
    candidates.push(join(base, "index.html"));
  } else {
    candidates.push(base, join(base, "index.html"), `${base}.html`);
  }
  for (const candidate of candidates) {
    if (!within(rootReal, candidate)) continue;
    try {
      const st = statSync(candidate);
      if (st.isFile()) {
        const real = realpathSync(candidate);
        if (within(rootReal, real)) return real;
      }
    } catch {
      // Try next candidate.
    }
  }
  return null;
}

/** Serves a directory over 127.0.0.1 on an ephemeral port with clean URLs. */
export async function startStaticServer(rootDir) {
  const rootReal = realpathSync(resolve(rootDir));
  const server = createServer((req, res) => {
    const method = req.method || "GET";
    if (method !== "GET" && method !== "HEAD") {
      res.writeHead(405, { "Content-Type": "text/plain; charset=utf-8" });
      res.end("Method Not Allowed");
      return;
    }
    const url = new URL(req.url || "/", "http://127.0.0.1");
    const file = resolveStaticFile(rootReal, url.pathname);
    if (!file) {
      res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
      res.end("Not Found");
      return;
    }
    const ext = extname(file).toLowerCase();
    const contentType = MIME_TYPES[ext] || "application/octet-stream";
    const body = readFileSync(file);
    res.writeHead(200, {
      "Content-Type": contentType,
      "Content-Length": body.length,
    });
    if (method === "HEAD") res.end();
    else res.end(body);
  });

  await new Promise((resolveListen, rejectListen) => {
    server.once("error", rejectListen);
    server.listen(0, "127.0.0.1", () => {
      server.off("error", rejectListen);
      resolveListen();
    });
  });

  const addr = server.address();
  const port = typeof addr === "object" && addr ? addr.port : 0;
  const origin = `http://127.0.0.1:${port}`;
  const close = () => new Promise((resolveClose) => {
    if (typeof server.closeAllConnections === "function") server.closeAllConnections();
    server.close(() => resolveClose());
  });
  return { server, origin, port, close };
}

class ChromeCrashError extends Error {
  constructor(message) {
    super(message);
    this.name = "ChromeCrashError";
  }
}

class CdpClient {
  constructor(ws) {
    this.ws = ws;
    this.nextId = 1;
    this.pending = new Map();
    this.listeners = new Set();
    this.closed = false;

    ws.addEventListener("message", (event) => {
      let msg;
      try {
        msg = JSON.parse(String(event.data));
      } catch {
        return;
      }
      if (typeof msg.id === "number") {
        const entry = this.pending.get(msg.id);
        if (!entry) return;
        this.pending.delete(msg.id);
        clearTimeout(entry.timer);
        if (msg.error) {
          const errText = msg.error.message || JSON.stringify(msg.error);
          if (/Target closed|Session closed|Inspected target navigated or closed/i.test(errText)) {
            entry.reject(new ChromeCrashError(`CDP ${entry.method} aborted: ${errText}`));
          } else {
            entry.reject(new Error(`CDP ${entry.method} failed: ${errText}`));
          }
        } else {
          entry.resolve(msg.result || {});
        }
      } else if (typeof msg.method === "string") {
        for (const listener of [...this.listeners]) {
          listener(msg);
        }
      }
    });

    ws.addEventListener("close", (e) => this._cleanup(`code ${e.code}`));
    ws.addEventListener("error", () => this._cleanup("socket error"));
  }

  _cleanup(reason) {
    if (this.closed) return;
    this.closed = true;
    const err = new ChromeCrashError(`CDP connection closed: ${reason}`);
    for (const entry of this.pending.values()) {
      clearTimeout(entry.timer);
      entry.reject(err);
    }
    this.pending.clear();
    for (const listener of [...this.listeners]) {
      listener({ method: "__closed__", params: { error: err } });
    }
    this.listeners.clear();
  }

  static async connect(wsUrl) {
    if (typeof globalThis.WebSocket !== "function") {
      throw new Error("Node global WebSocket is unavailable (Node 22+ is required)");
    }
    const ws = new globalThis.WebSocket(wsUrl);
    await new Promise((resolveOpen, rejectOpen) => {
      let done = false;
      const finish = (fn, value) => {
        if (done) return;
        done = true;
        clearTimeout(timer);
        ws.removeEventListener("open", onOpen);
        ws.removeEventListener("error", onError);
        ws.removeEventListener("close", onClose);
        fn(value);
      };
      const timer = setTimeout(() => {
        try { ws.close(); } catch { /* ignore */ }
        finish(rejectOpen, new Error(`timed out connecting to Chrome DevTools at ${wsUrl}`));
      }, CHROME_START_TIMEOUT_MS);
      const onOpen = () => finish(resolveOpen);
      const onError = () => finish(rejectOpen, new Error(`failed to connect to Chrome DevTools at ${wsUrl}`));
      const onClose = (e) => finish(rejectOpen, new Error(`Chrome DevTools closed before open (code ${e.code}) at ${wsUrl}`));
      ws.addEventListener("open", onOpen);
      ws.addEventListener("error", onError);
      ws.addEventListener("close", onClose);
    });
    return new CdpClient(ws);
  }

  send(method, params = {}, sessionId = undefined, timeoutMs = PAGE_TIMEOUT_MS) {
    if (this.closed) return Promise.reject(new ChromeCrashError(`CDP already closed before ${method}`));
    const id = this.nextId++;
    const payload = sessionId ? { id, method, params, sessionId } : { id, method, params };
    return new Promise((resolveCmd, rejectCmd) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        rejectCmd(new ChromeCrashError(`CDP ${method} timed out after ${timeoutMs} ms`));
      }, timeoutMs);
      this.pending.set(id, { method, resolve: resolveCmd, reject: rejectCmd, timer });
      try {
        this.ws.send(JSON.stringify(payload));
      } catch (err) {
        clearTimeout(timer);
        this.pending.delete(id);
        rejectCmd(new ChromeCrashError(`CDP ${method} send failed: ${err instanceof Error ? err.message : String(err)}`));
      }
    });
  }

  waitForEvent(sessionId, method, timeoutMs = PAGE_TIMEOUT_MS) {
    if (this.closed) {
      return {
        promise: Promise.reject(new ChromeCrashError(`CDP already closed waiting for ${method}`)),
        cancel: () => {},
      };
    }
    let settled = false;
    let cancel = () => {};
    const promise = new Promise((resolveEvt, rejectEvt) => {
      const cleanup = () => {
        clearTimeout(timer);
        this.listeners.delete(onMessage);
      };
      const finish = (fn, value) => {
        if (settled) return;
        settled = true;
        cleanup();
        fn(value);
      };
      const timer = setTimeout(() => {
        finish(rejectEvt, new ChromeCrashError(`timed out waiting for CDP event ${method} after ${timeoutMs} ms`));
      }, timeoutMs);
      const onMessage = (msg) => {
        if (msg.method === "__closed__") {
          finish(rejectEvt, msg.params.error);
          return;
        }
        if (msg.method === "Inspector.targetCrashed" && msg.sessionId === sessionId) {
          finish(rejectEvt, new ChromeCrashError("Chrome renderer target crashed"));
          return;
        }
        if (msg.method === method && msg.sessionId === sessionId) {
          finish(resolveEvt, msg.params || {});
        }
      };
      cancel = () => {
        if (settled) return;
        settled = true;
        cleanup();
      };
      this.listeners.add(onMessage);
    });
    return { promise, cancel };
  }

  close() {
    this._cleanup("closed");
    try {
      this.ws.close();
    } catch {
      // Ignore close errors.
    }
  }
}

const sleep = (ms) => new Promise((resolveSleep) => setTimeout(resolveSleep, ms));

function killProcessTree(proc, signal) {
  if (!proc || !proc.pid) return false;
  try {
    if (process.platform !== "win32") process.kill(-proc.pid, signal);
    else proc.kill(signal);
    return true;
  } catch {
    try {
      proc.kill(signal);
      return true;
    } catch {
      return false;
    }
  }
}

const activeChromes = new Set();
let signalHandlersInstalled = false;

function onTerminationSignal(signal) {
  for (const entry of [...activeChromes]) {
    killProcessTree(entry.proc, "SIGKILL");
    try {
      rmSync(entry.userDataDir, { recursive: true, force: true });
    } catch {
      // Best-effort cleanup on termination signal.
    }
    activeChromes.delete(entry);
  }
  uninstallSignalHandlers();
  process.kill(process.pid, signal);
}

function installSignalHandlers() {
  if (signalHandlersInstalled) return;
  signalHandlersInstalled = true;
  process.on("SIGINT", onTerminationSignal);
  process.on("SIGTERM", onTerminationSignal);
}

function uninstallSignalHandlers() {
  if (!signalHandlersInstalled) return;
  signalHandlersInstalled = false;
  process.off("SIGINT", onTerminationSignal);
  process.off("SIGTERM", onTerminationSignal);
}

function registerActiveChrome(entry) {
  activeChromes.add(entry);
  if (activeChromes.size === 1) installSignalHandlers();
}

function unregisterActiveChrome(entry) {
  activeChromes.delete(entry);
  if (activeChromes.size === 0) uninstallSignalHandlers();
}

async function launchChrome(chromeBin) {
  const userDataDir = mkdtempSync(join(tmpdir(), "td-layout-320-"));
  const args = [
    "--headless=new",
    "--no-sandbox",
    "--disable-dev-shm-usage",
    "--disable-gpu",
    "--disable-field-trial-config",
    "--disable-back-forward-cache",
    "--disable-breakpad",
    "--disable-hang-monitor",
    "--disable-ipc-flooding-protection",
    "--disable-renderer-backgrounding",
    "--disable-background-timer-throttling",
    "--disable-backgrounding-occluded-windows",
    "--disable-background-networking",
    "--disable-component-update",
    "--disable-default-apps",
    "--disable-extensions",
    "--disable-sync",
    "--disable-features=PaintHolding,OptimizationHints,MediaRouter,Translate",
    "--hide-scrollbars",
    "--metrics-recording-only",
    "--mute-audio",
    "--no-first-run",
    "--no-default-browser-check",
    "--remote-debugging-port=0",
    `--user-data-dir=${userDataDir}`,
    "about:blank",
  ];
  const proc = spawn(chromeBin, args, {
    stdio: ["ignore", "pipe", "pipe"],
    detached: process.platform !== "win32",
  });
  const activeEntry = { proc, userDataDir };
  registerActiveChrome(activeEntry);

  const exited = new Promise((resolveExit) => {
    if (proc.exitCode !== null || proc.signalCode !== null) resolveExit();
    else proc.once("exit", () => resolveExit());
  });

  let stderrTail = "";
  proc.stdout.resume();
  proc.stderr.on("data", (chunk) => {
    stderrTail += chunk.toString("utf8");
    if (stderrTail.length > 16384) stderrTail = stderrTail.slice(-8192);
  });

  let wsUrl;
  try {
    wsUrl = await new Promise((resolveUrl, rejectUrl) => {
      const timer = setTimeout(() => {
        rejectUrl(new Error(`Chrome did not publish a DevTools URL within ${CHROME_START_TIMEOUT_MS} ms:\n${stderrTail.trim()}`));
      }, CHROME_START_TIMEOUT_MS);
      const done = (fn, value) => {
        clearTimeout(timer);
        proc.off("error", onError);
        proc.off("exit", onExit);
        proc.stderr.off("data", onData);
        fn(value);
      };
      const onError = (err) => done(rejectUrl, new Error(`failed to spawn Chrome (${chromeBin}): ${err.message}`));
      const onExit = (code, signal) => done(rejectUrl, new Error(`Chrome exited early (code=${code}, signal=${signal}):\n${stderrTail.trim()}`));
      const onData = () => {
        const m = /DevTools listening on (ws:\/\/[^\s]+)/.exec(stderrTail);
        if (m) done(resolveUrl, m[1]);
      };
      proc.on("error", onError);
      proc.on("exit", onExit);
      proc.stderr.on("data", onData);
    });
  } catch (err) {
    killProcessTree(proc, "SIGKILL");
    await sleep(75);
    unregisterActiveChrome(activeEntry);
    rmSync(userDataDir, { recursive: true, force: true, maxRetries: 10, retryDelay: 100 });
    throw err;
  }

  let cdp;
  try {
    cdp = await CdpClient.connect(wsUrl);
  } catch (err) {
    killProcessTree(proc, "SIGKILL");
    await sleep(75);
    unregisterActiveChrome(activeEntry);
    rmSync(userDataDir, { recursive: true, force: true, maxRetries: 10, retryDelay: 100 });
    throw err;
  }

  let closed = false;
  const close = async () => {
    if (closed) return;
    closed = true;
    try {
      await cdp.send("Browser.close", {}, undefined, 2500).catch(() => {});
      cdp.close();
      const waitOrTimeout = (ms) => new Promise((resolveWait) => {
        let done = false;
        const timer = setTimeout(() => {
          if (done) return;
          done = true;
          resolveWait(false);
        }, ms);
        exited.then(() => {
          if (done) return;
          done = true;
          clearTimeout(timer);
          resolveWait(true);
        });
      });
      if (!(await waitOrTimeout(2000))) {
        killProcessTree(proc, "SIGTERM");
        if (!(await waitOrTimeout(1500))) {
          killProcessTree(proc, "SIGKILL");
          await waitOrTimeout(1000);
        }
      }
      // Ensure any stray helper process in the Chrome process group is gone before rmdir.
      killProcessTree(proc, "SIGKILL");
      await sleep(75);
    } finally {
      unregisterActiveChrome(activeEntry);
      rmSync(userDataDir, { recursive: true, force: true, maxRetries: 10, retryDelay: 100 });
    }
  };

  return { cdp, proc, userDataDir, close };
}

function buildMeasureExpression({ expectedLang, page, dynamic, viewportWidth }) {
  const args = [expectedLang ?? null, page ?? "", Boolean(dynamic), viewportWidth].map((v) => JSON.stringify(v)).join(", ");
  return `((expectedLang, pageName, isDynamic, viewportWidth) => (async () => {
    const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
    const deadline = Date.now() + 10000;
    while (document.readyState !== "complete" && Date.now() < deadline) {
      await sleep(25);
    }
    if (isDynamic) {
      while (Date.now() < deadline) {
        const langOk = expectedLang === null || document.documentElement.lang === expectedLang;
        const picker = document.getElementById("lang-select");
        const pickerOk = !picker || !picker.hidden;
        const receiveOk = pageName !== "receive" || (() => {
          const box = document.getElementById("r-has-code");
          const code = document.getElementById("r-code");
          return Boolean(box && !box.hidden && code && code.textContent.trim().length > 0);
        })();
        if (langOk && pickerOk && receiveOk) break;
        await sleep(25);
      }
    }
    if (document.fonts && document.fonts.ready) {
      await Promise.race([document.fonts.ready, sleep(4000)]);
    }
    await Promise.race([
      new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))),
      sleep(500),
    ]);
    const docEl = document.documentElement;
    const body = document.body;
    void docEl.offsetWidth;
    const docScrollWidth = docEl.scrollWidth;
    const docClientWidth = docEl.clientWidth;
    const bodyScrollWidth = body ? body.scrollWidth : 0;
    let widest = null;
    if (docScrollWidth > viewportWidth || bodyScrollWidth > viewportWidth || docClientWidth !== viewportWidth) {
      const isRtl = getComputedStyle(docEl).direction === "rtl";
      let bestOver = 0.5;
      for (const el of document.querySelectorAll("*")) {
        if (el === docEl || el === body) continue;
        if (el.closest && el.closest(".skip, .visually-hidden")) continue;
        const cs = getComputedStyle(el);
        if (cs.display === "none" || cs.visibility === "hidden") continue;
        let clipped = false;
        for (let p = el.parentElement; p && p !== docEl && p !== body; p = p.parentElement) {
          const pcs = getComputedStyle(p);
          if (pcs.display === "none") {
            clipped = true;
            break;
          }
          const ox = pcs.overflowX;
          if (ox === "auto" || ox === "scroll" || ox === "hidden" || ox === "clip") {
            clipped = true;
            break;
          }
        }
        if (clipped) continue;
        const r = el.getBoundingClientRect();
        const edgeOver = isRtl ? -r.left : r.right - viewportWidth;
        const over = Math.max(edgeOver, el.scrollWidth - viewportWidth);
        if (over >= bestOver - 0.5 && over > 0.5) {
          if (over > bestOver) bestOver = over;
          widest = {
            tag: el.tagName.toLowerCase(),
            id: el.id || "",
            className: typeof el.className === "string" ? el.className : "",
            left: Math.round(r.left * 10) / 10,
            right: Math.round(r.right * 10) / 10,
            scrollWidth: el.scrollWidth,
            clientWidth: el.clientWidth,
            text: (el.textContent || "").trim().replace(/\\s+/g, " ").slice(0, 60),
          };
        }
      }
    }
    return {
      docScrollWidth,
      docClientWidth,
      bodyScrollWidth,
      actualLang: docEl.lang || "",
      actualDir: docEl.dir || getComputedStyle(docEl).direction,
      url: location.href,
      widest,
    };
  })())(${args})`;
}

async function measureOnePage(cdp, sessionId, origin, check, viewportWidth) {
  const url = new URL(check.path, origin).toString();
  const loadWait = cdp.waitForEvent(sessionId, "Page.loadEventFired", PAGE_TIMEOUT_MS);
  loadWait.promise.catch(() => {});
  try {
    const nav = await cdp.send("Page.navigate", { url }, sessionId);
    if (nav.errorText) {
      if (nav.errorText === "net::ERR_ABORTED") {
        throw new ChromeCrashError(`navigation aborted for ${check.path}: ${nav.errorText}`);
      }
      throw new Error(`navigation failed for ${check.path}: ${nav.errorText}`);
    }
    await loadWait.promise;
  } finally {
    loadWait.cancel();
  }

  const evalResult = await cdp.send(
    "Runtime.evaluate",
    {
      expression: buildMeasureExpression({
        expectedLang: check.expectedLang ?? null,
        page: check.page,
        dynamic: check.dynamic,
        viewportWidth,
      }),
      awaitPromise: true,
      returnByValue: true,
    },
    sessionId,
  );
  if (evalResult.exceptionDetails) {
    throw new Error(`page evaluation failed for ${check.path}: ${evalResult.exceptionDetails.text}`);
  }
  const measurement = {
    ...(evalResult.result ? evalResult.result.value : {}),
    expectedLang: check.expectedLang ?? null,
  };
  const verdict = evaluateOverflow(measurement, viewportWidth);
  return { ...check, ...measurement, ...verdict };
}

const MAX_CHECK_CRASH_RETRIES = 2;

async function runLocaleChecks(chrome, baseUa, origin, tag, pendingChecks, viewportWidth, viewportHeight, onResult) {
  const { browserContextId } = await chrome.cdp.send("Target.createBrowserContext");
  try {
    const { targetId } = await chrome.cdp.send("Target.createTarget", {
      url: "about:blank",
      browserContextId,
    });
    try {
      await chrome.cdp.send("Target.activateTarget", { targetId }).catch(() => {});
      const { sessionId } = await chrome.cdp.send("Target.attachToTarget", {
        targetId,
        flatten: true,
      });
      await chrome.cdp.send("Page.enable", {}, sessionId);
      await chrome.cdp.send("Network.enable", {}, sessionId);
      await chrome.cdp.send("Runtime.enable", {}, sessionId);
      await chrome.cdp.send(
        "Emulation.setDeviceMetricsOverride",
        { width: viewportWidth, height: viewportHeight, deviceScaleFactor: 1, mobile: true },
        sessionId,
      );
      if (tag) {
        await chrome.cdp.send(
          "Emulation.setUserAgentOverride",
          { userAgent: baseUa, acceptLanguage: tag },
          sessionId,
        );
        await chrome.cdp.send(
          "Network.setExtraHTTPHeaders",
          { headers: { "Accept-Language": tag } },
          sessionId,
        );
        await chrome.cdp.send("Emulation.setLocaleOverride", { locale: tag }, sessionId);
      }

      let dynamicScriptInstalled = false;
      for (const check of pendingChecks) {
        if (check.dynamic && !dynamicScriptInstalled && tag) {
          await chrome.cdp.send(
            "Page.addScriptToEvaluateOnNewDocument",
            { source: `try { localStorage.setItem("td-lang", ${JSON.stringify(tag)}); } catch {}` },
            sessionId,
          );
          dynamicScriptInstalled = true;
        }
        const res = await measureOnePage(chrome.cdp, sessionId, origin, check, viewportWidth);
        onResult(check, res);
      }
    } finally {
      await chrome.cdp.send("Target.closeTarget", { targetId }).catch(() => {});
    }
  } finally {
    await chrome.cdp.send("Target.disposeBrowserContext", { browserContextId }).catch(() => {});
  }
}

/** Runs headless Chrome CDP 320 px overflow checks against a served directory. */
export async function checkPages({
  rootDir,
  checks,
  chromeBin = findChrome(),
  viewportWidth = VIEWPORT_WIDTH,
  viewportHeight = VIEWPORT_HEIGHT,
  batchSize = BATCH_SIZE,
}) {
  if (!chromeBin) {
    throw new Error(
      "No headless Chrome/Chromium binary found (checked CHROME_BIN and standard paths). Install google-chrome or chromium, or set CHROME_BIN.",
    );
  }
  const byLocale = new Map();
  for (const check of checks) {
    const key = check.tag ?? "";
    if (!byLocale.has(key)) byLocale.set(key, []);
    byLocale.get(key).push(check);
  }
  // Order static checks before dynamic checks inside each locale so localStorage stays clean on static pages.
  const localeGroups = [...byLocale.entries()].map(([tag, list]) => [
    tag,
    [...list.filter((c) => !c.dynamic), ...list.filter((c) => c.dynamic)],
  ]);

  const results = [];
  const failures = [];
  const completed = new Set();
  const crashCounts = new Map();

  const server = await startStaticServer(rootDir);
  try {
    for (let i = 0; i < localeGroups.length; i += batchSize) {
      const batch = localeGroups.slice(i, i + batchSize);
      let batchIndex = 0;

      while (batchIndex < batch.length) {
        const chrome = await launchChrome(chromeBin);
        try {
          const version = await chrome.cdp.send("Browser.getVersion");
          const baseUa = (version.userAgent || "Mozilla/5.0").replace("HeadlessChrome", "Chrome");

          while (batchIndex < batch.length) {
            const [tag, localeChecks] = batch[batchIndex];
            const remaining = localeChecks.filter((c) => !completed.has(c));
            if (remaining.length === 0) {
              batchIndex++;
              continue;
            }
            let currentCheck = remaining[0];
            try {
              await runLocaleChecks(
                chrome,
                baseUa,
                server.origin,
                tag,
                remaining,
                viewportWidth,
                viewportHeight,
                (check, res) => {
                  completed.add(check);
                  results.push(res);
                  if (!res.ok) failures.push(res);
                  const nextIdx = remaining.indexOf(check) + 1;
                  if (nextIdx < remaining.length) currentCheck = remaining[nextIdx];
                },
              );
              batchIndex++;
            } catch (err) {
              if (!(err instanceof ChromeCrashError)) throw err;
              const seen = (crashCounts.get(currentCheck) || 0) + 1;
              crashCounts.set(currentCheck, seen);
              if (seen > MAX_CHECK_CRASH_RETRIES) {
                throw new Error(
                  `Chrome crashed ${seen} times while checking ${tag || "-"} ${currentCheck.page} (${currentCheck.path}): ${err.message}`,
                );
              }
              // Break inner loop to recycle the Chrome process cleanly and resume from the crashed check.
              break;
            }
          }
        } finally {
          await chrome.close();
        }
      }
    }
  } finally {
    await server.close();
  }

  if (results.length !== checks.length) {
    throw new Error(`expected ${checks.length} measured pages, got ${results.length}`);
  }
  return {
    total: results.length,
    passed: results.length - failures.length,
    failed: failures.length,
    results,
    failures,
  };
}

export function formatFailure(f) {
  const parts = [
    `${f.tag || "-"} ${f.page} (${f.path})`,
    `doc=${f.docScrollWidth}/${f.docClientWidth}`,
    `body=${f.bodyScrollWidth}`,
    f.reasons.join("; "),
  ];
  if (f.widest) {
    const sel = `${f.widest.tag}${f.widest.id ? `#${f.widest.id}` : ""}${f.widest.className ? `.${f.widest.className.trim().split(/\s+/).join(".")}` : ""}`;
    parts.push(`widest=${sel} [L=${f.widest.left}..R=${f.widest.right}, sw=${f.widest.scrollWidth}] "${f.widest.text}"`);
  }
  return parts.join(" | ");
}

const CLI_USAGE = "usage: node site/tools/check-layout-320.mjs [--build | --dist <dir>] [--lang <tag> ...]";

/** Parses and validates CLI flags, accumulating deduplicated --lang tags. */
export function parseCliArgs(args, validTags = TAGS) {
  let doBuild = false;
  let distDir = null;
  const onlyLangs = [];
  const seenLangs = new Set();

  const addLang = (raw) => {
    if (typeof raw !== "string" || !raw.trim() || raw.startsWith("-")) {
      throw new Error(`missing value for --lang (${CLI_USAGE})`);
    }
    if (!validTags.includes(raw)) {
      throw new Error(`unknown language tag: ${raw}`);
    }
    if (!seenLangs.has(raw)) {
      seenLangs.add(raw);
      onlyLangs.push(raw);
    }
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--build") {
      doBuild = true;
    } else if (arg === "--dist") {
      if (i + 1 >= args.length || !args[i + 1] || !args[i + 1].trim() || args[i + 1].startsWith("-")) {
        throw new Error(`missing value for --dist (${CLI_USAGE})`);
      }
      distDir = args[++i];
    } else if (arg.startsWith("--dist=")) {
      const value = arg.slice("--dist=".length);
      if (!value || !value.trim()) {
        throw new Error(`missing value for --dist (${CLI_USAGE})`);
      }
      distDir = value;
    } else if (arg === "--lang") {
      if (i + 1 >= args.length) {
        throw new Error(`missing value for --lang (${CLI_USAGE})`);
      }
      addLang(args[++i]);
    } else if (arg.startsWith("--lang=")) {
      addLang(arg.slice("--lang=".length));
    } else {
      throw new Error(`unexpected argument: ${arg}`);
    }
  }
  if (doBuild && distDir !== null) {
    throw new Error("cannot pass both --build and --dist");
  }
  return { doBuild, distDir, onlyLangs };
}

async function main() {
  const { doBuild, distDir, onlyLangs } = parseCliArgs(process.argv.slice(2));
  const chromeBin = findChrome();
  if (!chromeBin) {
    throw new Error(
      "No headless Chrome/Chromium binary found (checked CHROME_BIN and standard paths). Install google-chrome or chromium, or set CHROME_BIN.",
    );
  }
  const tags = onlyLangs.length > 0 ? onlyLangs : TAGS;

  let tempBuildDir = null;
  let rootDir;
  if (doBuild) {
    tempBuildDir = mkdtempSync(join(siteRoot, ".check-layout-320-"));
    try {
      build(tempBuildDir);
    } catch (err) {
      rmSync(tempBuildDir, { recursive: true, force: true });
      throw err;
    }
    rootDir = checkDistDir(tempBuildDir);
  } else {
    rootDir = checkDistDir(distDir || join(siteRoot, "dist"));
  }

  try {
    const checks = buildChecks(tags);
    const report = await checkPages({ rootDir, checks, chromeBin });
    if (report.failed > 0) {
      console.error(`layout 320px FAILED: ${report.failed}/${report.total} checks failed:`);
      for (const f of report.failures) {
        console.error(`  - ${formatFailure(f)}`);
      }
      process.exit(1);
    }
    console.log(`layout 320px ok: ${report.passed}/${report.total} checks across ${tags.length} languages (0 overflows)`);
  } finally {
    if (tempBuildDir) rmSync(tempBuildDir, { recursive: true, force: true });
  }
}

if (process.argv[1] && existsSync(process.argv[1]) && realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url))) {
  main().catch((err) => {
    console.error(`layout 320px error: ${err instanceof Error ? err.message : String(err)}`);
    process.exit(1);
  });
}
