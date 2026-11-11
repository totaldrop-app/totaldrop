// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The team page (site/public/team/, #288). Who is listed, with which role
// and areas, is curated in site/team/team.json; profile names, bios,
// websites, pictures and every figure come from GitHub, so no share of the
// work is typed by hand (team.json may give a person's own display name or
// a translated one-line bio instead of the profile's, #408):
//   node site/tools/team.mjs           check the page and the avatars match
//                                      the snapshot (CI, offline)
//   node site/tools/team.mjs --write   rewrite the page from the snapshot
//   node site/tools/team.mjs --fetch   take a fresh snapshot from GitHub's
//                                      public API, copy the avatars, and
//                                      rewrite the page
// Who is listed and how is decided in site/team/team.json; GitHub supplies
// only public profile fields (name, bio, website, avatar) and the commit
// history, unless team.json gives its own name or bio. The snapshot is
// site/team/github.json. The page never loads anything from GitHub:
// visitors' browsers talk only to totaldrop.app.
// --fetch reads GITHUB_TOKEN if set (a higher rate limit), never prints it.
import { readFileSync, writeFileSync, existsSync, mkdirSync, readdirSync, unlinkSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createHash } from "node:crypto";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const CONFIG = join(root, "site", "team", "team.json");
const SNAPSHOT = join(root, "site", "team", "github.json");
const PAGE = join(root, "site", "public", "team", "index.html");
const AVATARS = join(root, "site", "public", "team", "avatars");
const I18N = join(root, "site", "public", "i18n");
const API = "https://api.github.com/";
const AVATAR_MAX = 256 * 1024;
const PAGES_MAX = 50; // 5,000 commits per path

const errors = [];
const fail = (msg) => { errors.push(msg); };

// --- Input rules --------------------------------------------------------

export const LOGIN = /^[A-Za-z0-9](?:[A-Za-z0-9]|-(?=[A-Za-z0-9])){0,38}(?:\[bot\])?$/;
const KINDS = ["person", "ai", "bot"];
// Roles with a translated label (tm.role.<role>); add the key in all 35
// languages before adding a role here.
const ROLES = ["maintainer", "principal", "contributor", "tester"];
// Disciplines, in page order, with their English text (translations:
// site/public/i18n/<tag>.json, keys tm.a.<key> and tm.a.<key>.d).
const AREAS = {
  core: ["Core and cryptography", "The Rust library every app shares: the transfer protocol, encryption and the code that reads untrusted input, with its fuzz tests."],
  formal: ["Formal verification", "Tamarin models that check the security of the account design before it is built."],
  android: ["Android", "The Flutter app for phones and tablets, in Material 3 and in the system's colours."],
  // iOS and macOS have no code in the repository yet (#309): for now they
  // only tag people. They get a description, in all 35 languages, when
  // they become disciplines with paths.
  ios: ["iOS", ""],
  macos: ["macOS", ""],
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
  const t = s.replace(/[\u0000-\u001f\u007f-\u009f\u061c\u200b-\u200f\u2028-\u202e\u2060-\u2069\ufeff\u{e0000}-\u{e007f}]/gu, " ")
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

// True when team.json's bio for a person is set but not one clean line.
export const badBio = (b) => b !== undefined && (typeof b !== "string" || b === "" || b !== clean(b, 240));

// The language files in an i18n folder (<tag>.json), sorted.
const langFiles = (dir) => readdirSync(dir).filter((f) => /^[a-z]{2,3}(-[A-Za-z0-9]+)*\.json$/.test(f)).sort();

// The language files of an i18n folder, read once, as { tag: messages }.
// A file that cannot be read, is not valid JSON or is not an object is
// left out and reported to the error list (see collectErrors).
export function readLangs(dir) {
  const langs = {};
  for (const f of langFiles(dir)) {
    const shownAs = relative(root, join(dir, f));
    let text, v;
    try { text = readFileSync(join(dir, f), "utf8"); } catch (e) { fail(`${shownAs}: cannot be read (${e.code || e.message})`); continue; }
    try { v = JSON.parse(text); } catch { fail(`${shownAs}: not valid JSON`); continue; }
    if (v === null || typeof v !== "object" || Array.isArray(v)) { fail(`${shownAs}: not a JSON object`); continue; }
    langs[f.slice(0, -5)] = v;
  }
  return langs;
}

// The languages (from readLangs) that lack a non-empty tm.bio.<login>: a
// bio is shown only once all 35 have it.
export function missingBio(langs, login) {
  return Object.keys(langs).filter((tag) => {
    const v = langs[tag][`tm.bio.${login}`];
    return typeof v !== "string" || v.trim() === "";
  });
}

// The bio line of a person's card: team.json's own line, translated, or
// else the GitHub bio as the person wrote it.
export function bioLine(p, pr) {
  if (p.bio !== undefined) return `<p class="bio" data-i18n="tm.bio.${esc(p.login)}">${esc(p.bio)}</p>`;
  return pr.bio ? `<p class="bio" dir="auto">${esc(pr.bio)}</p>` : "";
}

const loadConfig = () => validateConfig(JSON.parse(readFileSync(CONFIG, "utf8")));

// The errors team.json's rules find in a parsed config, for the tests.
export const configErrors = (c, i18n = I18N) => collectErrors(() => validateConfig(c, i18n));

// The errors reported while fn runs, taken out of the list, for the tests.
export function collectErrors(fn) {
  const before = errors.length;
  fn();
  return errors.splice(before);
}

function validateConfig(c, i18n = I18N) {
  if (!/^[A-Za-z0-9-]+\/[A-Za-z0-9._-]+$/.test(c.repo)) fail(`team.json: bad repo ${c.repo}`);
  const seen = new Set();
  // Read once, so a broken language file is reported once, not per bio.
  const langs = c.people.some((p) => p.bio !== undefined) ? readLangs(i18n) : {};
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
    // A line about the person's work on TotalDrop, in English, used instead
    // of their GitHub bio (#408). It is translated under tm.bio.<login> in
    // site/public/i18n/<tag>.json; add the key in all 35 languages.
    if (badBio(p.bio)) fail(`team.json: ${p.login}: bad bio`);
    else if (p.bio !== undefined && p.kind !== "person") fail(`team.json: ${p.login}: only people have a bio`);
    else if (p.bio !== undefined) for (const tag of missingBio(langs, p.login)) fail(`team.json: ${p.login}: tm.bio.${p.login} is not translated in ${tag}.json`);
    // A round coffee button to the team's donation page on the card (#486).
    // Donations go to the team, not to the person, so it takes no address.
    if (p.donate !== undefined && p.kind !== "person") fail(`team.json: ${p.login}: only people have a donate link`);
    else if (p.donate !== undefined && p.donate !== true) fail(`team.json: ${p.login}: donate must be true`);
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
    // anon=1 counts commits without a linked account too, under "" (their
    // names and e-mail addresses are not kept), so the total matches the
    // discipline tables.
    const list = await api(`repos/${c.repo}/contributors?anon=1&per_page=100&page=${page}`);
    if (!Array.isArray(list) || list.length === 0) break;
    for (const x of list) {
      if (!Number.isSafeInteger(x.contributions)) continue;
      if (x.type === "Anonymous") snap.totals[""] = (snap.totals[""] || 0) + x.contributions;
      else if (LOGIN.test(x.login)) snap.totals[x.login] = x.contributions;
    }
    if (list.length < 100) break;
    if (page === PAGES_MAX) throw new Error(`more than ${PAGES_MAX * 100} contributors, raise PAGES_MAX`);
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
        if (list.length < 100) break;
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
  for (const d of c.disciplines) if (!(d.key in s.areas)) fail(`github.json: no figures for ${d.key}, run --fetch`);
  for (const k of Object.keys(s.areas)) if (!c.disciplines.some((d) => d.key === k)) fail(`github.json: ${k} is not in team.json, run --fetch`);
  for (const [login, n] of Object.entries(s.totals || {})) if ((login !== "" && !LOGIN.test(login)) || !Number.isSafeInteger(n) || n < 1) fail(`github.json: totals: bad entry ${login}`);
  for (const [k, counts] of Object.entries(s.areas)) {
    if (!(k in AREAS)) fail(`github.json: unknown area ${k}`);
    for (const [login, n] of Object.entries(counts)) if ((login !== "" && !LOGIN.test(login)) || !Number.isSafeInteger(n) || n < 1) fail(`github.json: ${k}: bad entry ${login}`);
  }
}

// Icons for profile links. The GitHub mark is GitHub's octicon
// "mark-github" (MIT); the globe is drawn for this page. Both are
// decorative: the link's name is the visually hidden text beside them.
const ICON_GITHUB = `<svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path fill="currentColor" d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z"/></svg>`;
const ICON_WEB = `<svg class="icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round"><circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c2.5 2.6 3.75 5.6 3.75 9S14.5 18.4 12 21M12 3C9.5 5.6 8.25 8.6 8.25 12S9.5 18.4 12 21"/></svg>`;

// A cup with a handle and steam, drawn for this page in the globe's stroke.
const ICON_COFFEE = `<svg class="icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9h14v4.5a6.5 6.5 0 0 1-6.5 6.5h-1A6.5 6.5 0 0 1 3 13.5V9Z"/><path d="M17 10.5h1.25a3 3 0 0 1 0 6H16.6"/><path d="M7.5 2.5c-1 1.1-1 2.4 0 3.5M12.5 2.5c-1 1.1-1 2.4 0 3.5"/></svg>`;
const DONATE = "/donate/";

export const donateLink = () => `<li><a class="icon-link" href="${DONATE}">${ICON_COFFEE}<span class="visually-hidden" data-i18n="foot.donate">Donate</span></a></li>`;
const githubLink = (href, shownAs) => `<li><a class="icon-link" href="${esc(href)}">${ICON_GITHUB}<span class="visually-hidden">GitHub: <bdi>${esc(shownAs)}</bdi></span></a></li>`;
const webLink = (href) => `<li><a class="icon-link" href="${esc(href)}" rel="nofollow ugc">${ICON_WEB}<span class="visually-hidden"><span data-i18n="tm.web">Web site</span>: <bdi>${esc(shown(href))}</bdi></span></a></li>`;

// The colour of a login in the share bars and their legends, by role: the
// maintainer in the accent, the AI assistant in ink, other people, bots and
// everyone else in tints. Classes: s-maintainer, s-contributor, s-ai,
// s-bot, s-other.
export function series(c, login) {
  const p = c.people.find((x) => x.login === login);
  if (!p) return "s-other";
  if (p.kind === "person") return p.role === "maintainer" ? "s-maintainer" : "s-contributor";
  return `s-${p.kind}`;
}

// How a GitHub login is named on the page.
function who(c, s, login) {
  const p = c.people.find((x) => x.login === login);
  if (login === "") return `<span data-i18n="tm.others">Others without a linked GitHub account</span>`;
  if (p && p.kind === "ai") return "Claude";
  if (p && p.kind === "bot") return "Dependabot";
  if (p && s.profiles[login]) return `<bdi>${esc(p.name || s.profiles[login].name)}</bdi>`;
  const bot = /^(.+)\[bot\]$/.exec(login);
  const href = bot ? `https://github.com/apps/${bot[1]}` : `https://github.com/${login}`;
  return `<a href="${esc(href)}"><bdi>${esc(login)}</bdi></a>`;
}

const stat = (n) => (n ? `<dl class="stat"><dt data-i18n="tm.col.changes">Changes</dt><dd>${esc(String(n))}</dd></dl>` : "");

function personCard(c, s, p) {
  const pr = s.profiles[p.login];
  const role = { maintainer: "Founder and project lead", principal: "Principal engineer", contributor: "Developer", tester: "QA specialist" }[p.role];
  const areas = (p.areas || []).map((a) => `<li data-i18n="tm.a.${a}">${esc(AREAS[a][0])}</li>`).join("");
  const links = [githubLink(`https://github.com/${p.login}`, p.login)];
  if (pr.website) links.push(webLink(pr.website));
  if (p.donate) links.push(donateLink());
  const lines = [
    `        <li class="person">`,
    `          <img class="avatar" src="/team/avatars/${esc(pr.avatar)}" alt="" width="88" height="88" decoding="async">`,
    `          <div class="person-body">`,
    `            <h3 dir="auto">${esc(p.name || pr.name)}</h3>`,
    `            <p class="role" data-i18n="tm.role.${p.role}">${role}</p>`,
  ];
  const bio = bioLine(p, pr);
  if (bio) lines.push(`            ${bio}`);
  if (areas) lines.push(`            <ul class="tags">${areas}</ul>`);
  lines.push(`            <div class="person-foot">`);
  const st = stat(s.totals[p.login]);
  if (st) lines.push(`              ${st}`);
  lines.push(`              <ul class="links">${links.join("")}</ul>`, `            </div>`, `          </div>`, `        </li>`);
  return lines.join("\n");
}

function helperCard(c, s, p) {
  const st = stat(s.totals[p.login]);
  const ai = p.kind === "ai";
  const lines = [
    `        <li class="helper">`,
    `          <span class="avatar avatar-sign" aria-hidden="true">${ai ? "AI" : "BOT"}</span>`,
    `          <div class="person-body">`,
    `            <h3>${ai ? "Claude" : "Dependabot"}</h3>`,
    ai ? `            <p class="role" data-i18n="tm.role.ai">AI development assistant</p>` : `            <p class="role" data-i18n="tm.role.bot">Automation</p>`,
    ai ? `            <p data-i18n="tm.ai.d">Writes much of the code and runs the independent review of each pull request. It holds no role in the project: a maintainer makes or approves every merge, and the project lead approves every release and deployment.</p>`
       : `            <p data-i18n="tm.bot.d">Opens a pull request when a dependency has a new version. Each one is reviewed like any other change.</p>`,
    `            <div class="person-foot">`,
  ];
  if (st) lines.push(`              ${st}`);
  if (!ai) lines.push(`              <ul class="links">${githubLink("https://github.com/apps/dependabot", "dependabot")}</ul>`);
  lines.push(`            </div>`, `          </div>`, `        </li>`);
  return lines.join("\n");
}

function areaCard(c, s, d) {
  const counts = Object.entries(s.areas[d.key] || {}).sort(([a, x], [b, y]) => y - x || (a < b ? -1 : 1));
  const total = counts.reduce((t, [, n]) => t + n, 0);
  const lines = [
    `        <li class="area" id="${d.key}">`,
    `          <div class="area-head">`,
    `            <h3 id="${d.key}-title" data-i18n="tm.a.${d.key}">${esc(AREAS[d.key][0])}</h3>`,
  ];
  if (total) lines.push(`            ${stat(total)}`);
  lines.push(`          </div>`, `          <p data-i18n="tm.a.${d.key}.d">${esc(AREAS[d.key][1])}</p>`);
  if (total) {
    // The bar repeats the table below it for the eye; the table is what is
    // read out. Widths are SVG attributes, as the CSP allows no inline style.
    let x = 0;
    const rects = counts.map(([login, n]) => {
      const w = (n / total) * 100;
      const r = `<rect class="${series(c, login)}" x="${+x.toFixed(2)}" width="${+w.toFixed(2)}" height="8"/>`;
      x += w;
      return r;
    }).join("");
    lines.push(
      `          <svg class="share" viewBox="0 0 100 8" preserveAspectRatio="none" aria-hidden="true" focusable="false">${rects}</svg>`,
      `          <table class="area-table" aria-labelledby="${d.key}-title">`,
      `            <thead><tr><th scope="col"><span class="visually-hidden" data-i18n="tm.col.who">Who</span></th><th scope="col"><span class="visually-hidden" data-i18n="tm.col.changes">Changes</span></th></tr></thead>`,
      `            <tbody>`,
      ...counts.map(([login, n]) => `              <tr class="${series(c, login)}"><th scope="row">${who(c, s, login)}</th><td>${esc(String(n))}</td></tr>`),
      `            </tbody>`,
      `          </table>`,
    );
  }
  lines.push(`        </li>`);
  return lines.join("\n");
}

function stats(c, s) {
  const langs = langFiles(I18N).length;
  const changes = Object.values(s.totals).reduce((t, n) => t + n, 0);
  const item = (n, key, en) => `        <div><dt data-i18n="${key}">${en}</dt><dd>${esc(String(n))}</dd></div>`;
  return [
    `      <dl class="tm-stats">`,
    item(changes, "tm.col.changes", "Changes"),
    item(c.people.filter((p) => p.kind === "person").length, "tm.s.people", "People"),
    item(c.disciplines.length, "tm.s.areas", "Disciplines"),
    item(langs, "tm.s.langs", "Languages"),
    `      </dl>`,
  ].join("\n");
}

function render(c, s, html) {
  const regions = {
    stats: stats(c, s),
    people: `      <ul class="people">\n${c.people.filter((p) => p.kind === "person").map((p) => personCard(c, s, p)).join("\n")}\n      </ul>`,
    helpers: `      <ul class="helpers">\n${c.people.filter((p) => p.kind !== "person").map((p) => helperCard(c, s, p)).join("\n")}\n      </ul>`,
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
