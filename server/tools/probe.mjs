// Health probe after a deploy: node server/tools/probe.mjs rendezvous|relay|site
//
// rendezvous  1. opens a WebSocket to /v0/allocate and closes it before
//                sending anything, so no nameplate is taken;
//             2. asks for mailbox 4294967295, which is never handed out, and
//                expects the directory's "unknown nameplate" answer: this
//                reaches the Directory Durable Object and only reads.
// relay       opens a WebSocket to /v0/relay/<random pair> and closes it; a
//             pair with no second side is dropped by the Worker
// site        fetches the home page and the download page
//
// Every attempt gives up after 10 s, and the probe tries 12 times, 5 s
// apart, since a new deploy can take a moment to reach every location: at
// most about three minutes. Exits 1 if the target never answers, 2 on a
// usage error.
import { randomBytes } from "node:crypto";

const target = process.argv[2];
const tries = 12;
const timeout = 10_000;
const wait = (ms) => new Promise((r) => setTimeout(r, ms));

// Opens a WebSocket and resolves once it is open, or, with `expect`, once a
// message whose bytes contain `expect` arrives.
function socket(url, expect) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    ws.binaryType = "arraybuffer";
    let settled = false;
    const done = (fn, arg) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      try {
        ws.close(1000);
      } catch {
        // Closed already.
      }
      fn(arg);
    };
    const timer = setTimeout(() => done(reject, new Error("no answer in 10 s")), timeout);
    ws.addEventListener("open", () => {
      if (!expect) done(resolve);
    });
    ws.addEventListener("message", (event) => {
      const text = Buffer.from(event.data).toString("utf8");
      if (expect && text.includes(expect)) done(resolve);
      else done(reject, new Error("unexpected answer"));
    });
    ws.addEventListener("close", () => done(reject, new Error("closed without the expected answer")));
    ws.addEventListener("error", () => done(reject, new Error("handshake refused")));
  });
}

async function page(url, text) {
  const res = await fetch(url, { redirect: "follow", signal: AbortSignal.timeout(timeout) });
  if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
  if (!(await res.text()).includes(text)) throw new Error(`${url}: "${text}" missing`);
}

function pairName() {
  const [a, b] = [randomBytes(32).toString("hex"), randomBytes(32).toString("hex")].sort();
  return `${a}-${b}`;
}

const checks = {
  rendezvous: async () => {
    await socket("wss://rendezvous.totaldrop.app/v0/allocate");
    await socket("wss://rendezvous.totaldrop.app/v0/mailbox/4294967295", "unknown nameplate");
  },
  relay: () => socket(`wss://relay.totaldrop.app/v0/relay/${pairName()}`),
  site: async () => {
    await page("https://totaldrop.app/", "TotalDrop");
    await page("https://totaldrop.app/download/", "releases/download/");
  },
};

const check = checks[target];
if (!check) {
  console.error("usage: probe.mjs rendezvous|relay|site");
  process.exit(2);
}
for (let i = 1; ; i++) {
  try {
    await check();
    console.log(`${target}: ok`);
    break;
  } catch (e) {
    console.log(`${target}: try ${i}: ${e.message}`);
    if (i === tries) process.exit(1);
    await wait(5_000);
  }
}
