// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The team page (site/public/team/, #288), made from GitHub data so that
// nobody's name, picture or share of the work is typed by hand:
//   node site/tools/team.mjs           check the page and the avatars match
//                                      the snapshot (CI, offline)
//   node site/tools/team.mjs --write   rewrite the page from the snapshot
//   node site/tools/team.mjs --fetch   take a fresh snapshot from GitHub's
//                                      public API, copy the avatars, and
//                                      rewrite the page
// Who is listed and how is decided in site/team/team.json; GitHub supplies
// only public profile fields (name, bio, website, avatar) and the commit
// history. The snapshot is site/team/github.json. The page never loads
// anything from GitHub: visitors' browsers talk only to totaldrop.app.
// --fetch reads GITHUB_TOKEN if set (a higher rate limit), never prints it.
import { readFileSync, writeFileSync, existsSync, mkdirSync, readdirSync, unlinkSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createHash } from "node:crypto";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const CONFIG = join(root, "site", "team", "team.json");
const SNAPSHOT = join(root, "site", "team", "github.json");
const PAGE = join(root, "site", "public", "team", "index.html");
const AVATARS = join(root, "site", "public", "team", "avatars");
const API = "https://api.github.com/";
const AVATAR_MAX = 256 * 1024;
const PAGES_MAX = 50; // 5,000 commits per path

const errors = [];
const fail = (msg) => { errors.push(msg); };

// --- Input rules --------------------------------------------------------

export const LOGIN = /^[A-Za-z0-9](?:[A-Za-z0-9]|-(?=[A-Za-z0-9])){0,38}(?:\[bot\])?$/;
const KINDS = ["person", "ai", "bot"];
const ROLES = ["maintainer", "contributor", "designer", "tester", "translator"];
// Disciplines, in page order, with their English text (translations:
// site/public/i18n/<tag>.json, keys tm.a.<key> and tm.a.<key>.d).
const AREAS = {
  core: ["Core and cryptography", "The Rust library every app shares: the transfer protocol, encryption and the code that reads untrusted input, with its fuzz tests."],
  formal: ["Formal verification", "Tamarin models that check the security of the account design before it is built."],
  android: ["Android", "The Flutter app for phones and tablets, in Material 3 and in the system's colours."],
  linux: ["Linux desktop", "The Qt 6 app for KDE, GNOME, Hyprland and Omarchy, and its deb, rpm and Arch packages."],
  cli: ["Command line", "The totaldrop command, for terminals and scripts."],
  site: ["Web site", "totaldrop.app in 35 languages, without trackers or third-party requests."],
  servers: ["Servers and infrastructure", "The rendezvous and relay servers on Cloudflare, and the automated builds, checks and deployments."],
  l10n: ["Translations", "Interface text in 35 languages and the rules for writing it."],
  docs: ["Documentation and security", "Decision records, protocol notes, the threat model and the security policy."],
};

// Strip what could change how text around it is shown: control
// characters, direction controls and isolates, line and paragraph
// separators. Then collapse spaces and cut to length.
export function clean(s, max) {
  if (typeof s !== "string") return "";
  const t = s.replace(/[\u0000-\u001f\u007f-\u009f\u061c\u200b-\u200f\u2028-\u202e\u2066-\u2069\ufeff]/gu, " ")
    .replace(/\s+/gu, " ").trim();
  const chars = Array.from(t);
  return chars.length > max ? chars.slice(0, max - 1).join("").trimEnd() + "\u2026" : t;
}

// A profile's website: https only, no user name or password, no port.
export function website(s) {
  const t = clean(s, 200);
  if (!t) return null;
  let u;
  try { u = new URL(/^[a-z][a-z0-9+.-]*:/i.test(t) ? t : "https://" + t); } catch (e) { return null; }
  if (u.protocol !== "https:" || u.username || u.password || u.port || !u.hostname.includes(".")) return null;
  return u.href;
}

export const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const shown = (href) => href.replace(/^https:\/\//, "").replace(/\/$/, "");

function loadConfig() {
  const c = JSON.parse(readFileSync(CONFIG, "utf8"));
  if (!/^[A-Za-z0-9-]+\/[A-Za-z0-9._-]+$/.test(c.repo)) fail(`team.json: bad repo ${c.repo}`);
  const seen = new Set();
  for (const p of c.people) {
    if (!LOGIN.test(p.login)) fail(`team.json: bad login ${JSON.stringify(p.login)}`);
    if (seen.has(p.login)) fail(`team.json: ${p.login} listed twice`);
    seen.add(p.login);
    if (!KINDS.includes(p.kind)) fail(`team.json: ${p.login}: kind must be one of ${KINDS.join(", ")}`);
    if (p.kind === "person" && !ROLES.includes(p.role)) fail(`team.json: ${p.login}: role must be one of ${ROLES.join(", ")}`);
    for (const a of p.areas || []) if (!(a in AREAS)) fail(`team.json: ${p.login}: unknown area ${a}`);
    // A display name, when the person prefers another spelling than their
    // GitHub profile's (for example the profile is in capitals).
    if (p.name !== undefined && (typeof p.name !== "string" || p.name !== clean(p.name, 80) || p.name === "")) fail(`team.json: ${p.login}: bad name`);
  }
  for (const d of c.disciplines) {
    if (!(d.key in AREAS)) fail(`team.json: unknown discipline ${d.key}`);
    if (!Array.isArray(d.paths) || d.paths.length === 0) fail(`team.json: ${d.key} has no paths`);
    for (const p of d.paths) if (!existsSync(join(root, p))) fail(`team.json: ${d.key}: ${p} does not exist`);
  }
  const keys = c.disciplines.map((d) => d.key);
  if (keys.join() !== Object.keys(AREAS).filter((k) => keys.includes(k)).join()) fail("team.json: disciplines out of page order");
  return c;
}

// --- Fetch --------------------------------------------------------------

async function get(url, accept = "application/vnd.github+json") {
  const headers = { Accept: accept, "User-Agent": "totaldrop-team-page", "X-GitHub-Api-Version": "2022-11-28" };
  if (process.env.GITHUB_TOKEN && url.startsWith(API)) headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  for (let attempt = 0; ; attempt++) {
    let res;
    try {
      res = await fetch(url, { headers, redirect: "follow", signal: AbortSignal.timeout(15000) });
    } catch (e) {
      if (attempt < 3) { await new Promise((r) => setTimeout(r, 1000 * 2 ** attempt)); continue; }
      throw new Error(`${url}: ${e.message}`);
    }
    if (res.status >= 500 && attempt < 3) { await new Promise((r) => setTimeout(r, 1000 * 2 ** attempt)); continue; }
    if ((res.status === 403 || res.status === 429) && res.headers.get("x-ratelimit-remaining") === "0") {
      throw new Error(`${url}: GitHub's rate limit is used up (60 requests an hour without a token); set GITHUB_TOKEN, for example GITHUB_TOKEN=$(gh auth token)`);
    }
    if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
    return res;
  }
}
const api = async (path) => (await get(API + path)).json();

async function fetchSnapshot(c) {
  const snap = { fetched: new Date().toISOString().slice(0, 10), profiles: {}, totals: {}, areas: {} };

  for (const p of c.people.filter((x) => x.kind === "person")) {
    const u = await api(`users/${encodeURIComponent(p.login)}`);
    if (u.login.toLowerCase() !== p.login.toLowerCase() || !Number.isSafeInteger(u.id)) throw new Error(`users/${p.login}: unexpected answer`);
    const res = await get(`https://avatars.githubusercontent.com/u/${u.id}?s=192&v=4`, "image/png,image/jpeg");
    const type = (res.headers.get("content-type") || "").split(";")[0].trim();
    const ext = { "image/png": "png", "image/jpeg": "jpg" }[type];
    if (!ext) throw new Error(`avatar of ${p.login}: unexpected type ${type}`);
    const bytes = Buffer.from(await res.arrayBuffer());
    if (bytes.length === 0 || bytes.length > AVATAR_MAX) throw new Error(`avatar of ${p.login}: ${bytes.length} bytes`);
    if (!magicOk(bytes, ext)) throw new Error(`avatar of ${p.login}: not a ${ext} file`);
    mkdirSync(AVATARS, { recursive: true });
    const file = `${p.login}.${ext}`;
    writeFileSync(join(AVATARS, file), bytes);
    snap.profiles[p.login] = {
      name: clean(u.name, 80) || p.login,
      bio: clean(u.bio, 240),
      website: website(u.blog),
      avatar: file,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    };
  }
  // Avatars of people no longer listed go.
  if (existsSync(AVATARS)) for (const f of readdirSync(AVATARS)) if (!Object.values(snap.profiles).some((p) => p.avatar === f)) unlinkSync(join(AVATARS, f));

  for (let page = 1; page <= PAGES_MAX; page++) {
    const list = await api(`repos/${c.repo}/contributors?per_page=100&page=${page}`);
    if (!Array.isArray(list) || list.length === 0) break;
    for (const x of list) if (LOGIN.test(x.login) && Number.isSafeInteger(x.contributions)) snap.totals[x.login] = x.contributions;
  }

  for (const d of c.disciplines) {
    const commits = new Map();
    for (const path of d.paths) {
      for (let page = 1; page <= PAGES_MAX; page++) {
        const list = await api(`repos/${c.repo}/commits?path=${encodeURIComponent(path)}&per_page=100&page=${page}`);
        if (!Array.isArray(list) || list.length === 0) break;
        for (const x of list) {
          const login = x.author && LOGIN.test(x.author.login) ? x.author.login : "";
          commits.set(x.sha, login);
        }
        if (page === PAGES_MAX) throw new Error(`${path}: more than ${PAGES_MAX * 100} commits, raise PAGES_MAX`);
      }
    }
    const counts = {};
    for (const login of commits.values()) counts[login] = (counts[login] || 0) + 1;
    snap.areas[d.key] = sortKeys(counts);
  }
  snap.totals = sortKeys(snap.totals);
  return snap;
}

function magicOk(b, ext) {
  if (ext === "png") return b.length > 8 && b.subarray(0, 8).equals(Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]));
  return b.length > 3 && b[0] === 0xff && b[1] === 0xd8 && b[2] === 0xff;
}
const sortKeys = (o) => Object.fromEntries(Object.entries(o).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)));

// --- Render -------------------------------------------------------------

function checkSnapshot(c, s) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(s.fetched)) fail("github.json: bad date");
  for (const p of c.people.filter((x) => x.kind === "person")) {
    const pr = s.profiles[p.login];
    if (!pr) { fail(`github.json: no profile for ${p.login}, run --fetch`); continue; }
    if (pr.name !== clean(pr.name, 80) || pr.bio !== clean(pr.bio, 240)) fail(`github.json: ${p.login}: text not cleaned`);
    if (pr.website !== null && pr.website !== website(pr.website)) fail(`github.json: ${p.login}: bad website`);
    if (pr.avatar !== `${p.login}.png` && pr.avatar !== `${p.login}.jpg`) { fail(`github.json: ${p.login}: bad avatar name`); continue; }
    const f = join(AVATARS, pr.avatar);
    if (!existsSync(f)) { fail(`avatar missing: ${pr.avatar}`); continue; }
    const bytes = readFileSync(f);
    if (createHash("sha256").update(bytes).digest("hex") !== pr.sha256) fail(`avatar ${pr.avatar} differs from the snapshot`);
    if (bytes.length > AVATAR_MAX || !magicOk(bytes, pr.avatar.slice(-3))) fail(`avatar ${pr.avatar}: wrong size or type`);
  }
  if (existsSync(AVATARS)) for (const f of readdirSync(AVATARS)) if (!Object.values(s.profiles).some((p) => p.avatar === f)) fail(`avatar not in the snapshot: ${f}`);
  for (const [k, counts] of Object.entries(s.areas)) {
    if (!(k in AREAS)) fail(`github.json: unknown area ${k}`);
    for (const [login, n] of Object.entries(counts)) if ((login !== "" && !LOGIN.test(login)) || !Number.isSafeInteger(n) || n < 1) fail(`github.json: ${k}: bad entry ${login}`);
  }
}

// How a GitHub login is named on the page.
function who(c, s, login) {
  const p = c.people.find((x) => x.login === login);
  if (login === "") return `<span data-i18n="tm.others">Others without a linked GitHub account</span>`;
  if (p && p.kind === "ai") return "Claude";
  if (p && p.kind === "bot") return "Dependabot";
  if (p && s.profiles[login]) return esc(p.name || s.profiles[login].name);
  return `<a href="https://github.com/${esc(login)}">${esc(login)}</a>`;
}

function personCard(c, s, p) {
  const total = s.totals[p.login];
  const stat = total ? `\n          <dl class="stat"><dt data-i18n="tm.col.changes">Changes</dt><dd>${total}</dd></dl>` : "";
  if (p.kind === "ai") {
    return `        <li class="person is-ai">
          <span class="avatar avatar-sign" aria-hidden="true">AI</span>
          <h3>Claude</h3>
          <p class="role" data-i18n="tm.role.ai">AI development assistant</p>
          <p data-i18n="tm.ai.d">Writes much of the code and runs the independent review of each pull request. It holds no role in the project: the maintainer approves every merge, release and deployment.</p>${stat}
        </li>`;
  }
  if (p.kind === "bot") {
    return `        <li class="person is-bot">
          <span class="avatar avatar-sign" aria-hidden="true">BOT</span>
          <h3>Dependabot</h3>
          <p class="role" data-i18n="tm.role.bot">Automation</p>
          <p data-i18n="tm.bot.d">Opens a pull request when a dependency has a new version. Each one is reviewed like any other change.</p>${stat}
          <p class="links"><a href="https://github.com/apps/dependabot">github.com/apps/dependabot</a></p>
        </li>`;
  }
  const pr = s.profiles[p.login];
  const role = { maintainer: "Founder and maintainer", contributor: "Contributor", designer: "Designer", tester: "Tester", translator: "Translator" }[p.role];
  const areas = (p.areas || []).map((a) => `<li data-i18n="tm.a.${a}">${esc(AREAS[a][0])}</li>`).join("");
  const links = [`<a href="https://github.com/${esc(p.login)}">github.com/${esc(p.login)}</a>`];
  if (pr.website) links.push(`<a href="${esc(pr.website)}">${esc(shown(pr.website))}</a>`);
  return `        <li class="person">
          <img class="avatar" src="/team/avatars/${esc(pr.avatar)}" alt="" width="96" height="96" decoding="async">
          <h3>${esc(p.name || pr.name)}</h3>
          <p class="role" data-i18n="tm.role.${p.role}">${role}</p>${pr.bio ? `\n          <p class="bio" dir="auto">${esc(pr.bio)}</p>` : ""}${areas ? `\n          <ul class="tags">${areas}</ul>` : ""}${stat}
          <p class="links">${links.join(" ")}</p>
        </li>`;
}

function areaCard(c, s, d) {
  const counts = Object.entries(s.areas[d.key] || {}).sort(([a, x], [b, y]) => y - x || (a < b ? -1 : 1));
  const rows = counts.map(([login, n]) => `              <tr><th scope="row">${who(c, s, login)}</th><td>${n}</td></tr>`).join("\n");
  const table = counts.length
    ? `
          <table class="area-table">
            <thead><tr><th scope="col" data-i18n="tm.col.who">Who</th><th scope="col" data-i18n="tm.col.changes">Changes</th></tr></thead>
            <tbody>
${rows}
            </tbody>
          </table>`
    : `\n          <p class="note" data-i18n="tm.none">No changes yet.</p>`;
  return `        <li class="area" id="${d.key}">
          <h3 data-i18n="tm.a.${d.key}">${esc(AREAS[d.key][0])}</h3>
          <p data-i18n="tm.a.${d.key}.d">${esc(AREAS[d.key][1])}</p>${table}
        </li>`;
}

function render(c, s, html) {
  const regions = {
    people: `      <ul class="people">\n${c.people.map((p) => personCard(c, s, p)).join("\n")}\n      </ul>`,
    areas: `      <ul class="areas">\n${c.disciplines.map((d) => areaCard(c, s, d)).join("\n")}\n      </ul>`,
    date: `<time datetime="${s.fetched}">${s.fetched}</time>`,
  };
  let out = html;
  for (const [name, body] of Object.entries(regions)) {
    const re = new RegExp(`(<!-- team:${name} -->)[\\s\\S]*?(<!-- /team:${name} -->)`);
    if (!re.test(out)) { fail(`team/index.html: markers for ${name} missing`); continue; }
    const inline = name === "date";
    out = out.replace(re, (_, a, b) => (inline ? `${a}${body}${b}` : `${a}\n${body}\n      ${b}`));
  }
  return out;
}

// --- Main ---------------------------------------------------------------

async function main() {

  const args = process.argv.slice(2);
  const config = loadConfig();
  let snapshot;
  if (args.includes("--fetch") && errors.length === 0) {
    snapshot = await fetchSnapshot(config);
    writeFileSync(SNAPSHOT, JSON.stringify(snapshot, null, 2) + "\n");
  } else {
    snapshot = JSON.parse(readFileSync(SNAPSHOT, "utf8"));
  }
  checkSnapshot(config, snapshot);
  const html = readFileSync(PAGE, "utf8");
  const want = errors.length ? html : render(config, snapshot, html);
  if (errors.length === 0) {
    if (args.includes("--write") || args.includes("--fetch")) {
      if (want !== html) writeFileSync(PAGE, want);
    } else if (want !== html) {
      fail("team/index.html is not up to date with site/team/github.json: run node site/tools/team.mjs --write");
    }
  }
  if (errors.length) {
    for (const e of errors) console.error("error: " + e);
    process.exit(1);
  }
  console.log(`team page: ${config.people.length} people, ${config.disciplines.length} disciplines, data from ${snapshot.fetched}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();
