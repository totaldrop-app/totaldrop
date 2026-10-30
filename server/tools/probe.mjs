// Health probe after a deploy: node server/tools/probe.mjs rendezvous|relay|site
//
// rendezvous  opens a WebSocket to /v0/allocate and closes it before sending
//             anything, so no nameplate is taken
// relay       opens a WebSocket to /v0/relay/<random pair> and closes it; a
//             pair with no second side is dropped by the Worker
// site        fetches the home page and the download page
//
// Retries for up to a minute, since a new deploy can take a few seconds to
// reach every location. Exits non-zero if the target never answers.
import { randomBytes } from "node:crypto";

const target = process.argv[2];
const tries = 12;
const wait = (ms) => new Promise((r) => setTimeout(r, ms));

function handshake(url) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    const timer = setTimeout(() => {
      ws.close();
      reject(new Error("no answer in 10 s"));
    }, 10_000);
    ws.addEventListener("open", () => {
      clearTimeout(timer);
      ws.close(1000);
      resolve();
    });
    ws.addEventListener("error", () => {
      clearTimeout(timer);
      reject(new Error("handshake refused"));
    });
  });
}

async function page(url, text) {
  const res = await fetch(url, { redirect: "follow" });
  if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
  if (!(await res.text()).includes(text)) throw new Error(`${url}: "${text}" missing`);
}

function pairName() {
  const [a, b] = [randomBytes(32).toString("hex"), randomBytes(32).toString("hex")].sort();
  return `${a}-${b}`;
}

const checks = {
  rendezvous: () => handshake("wss://rendezvous.totaldrop.app/v0/allocate"),
  relay: () => handshake(`wss://relay.totaldrop.app/v0/relay/${pairName()}`),
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
