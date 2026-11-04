// Builds the privacy policy pages, one per site language (#207):
//   node site/tools/privacy.mjs           check (CI): pages up to date, translations in shape
//   node site/tools/privacy.mjs --write   rewrite site/public/privacy.html and privacy/<tag>.html
//
// The text of each language is site/privacy/<tag>.html, the inside of its
// <article>. English is the authoritative text and is served at /privacy;
// every other language at /privacy/<tag>, saying that the English version
// applies (and, until a native speaker has reviewed it, that it is a
// draft). The header, footer and those notes come from i18n/<tag>.json.
// The site has no build step: the pages this writes are committed, and
// the check fails when they are out of date.
//
// Each translation must have the same shape as English: the same headings,
// paragraphs and list items in the same order, the same links, the same
// <code> text and the same version numbers. A translation that drops a
// paragraph or a server name, or keeps last version's numbers, fails.
import { readFileSync, writeFileSync, mkdirSync, readdirSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const pub = join(root, "site", "public");
const src = join(root, "site", "privacy");
const SITE = "https://totaldrop.app";
const write = process.argv.includes("--write");
const errors = [];

const langbox = { Intl };
vm.runInNewContext(readFileSync(join(pub, "lang.js"), "utf8"), langbox);
const { LANGUAGES, REVIEWED } = langbox.TDLang;
const tags = LANGUAGES.map((l) => l[0]);
const nameOf = Object.fromEntries(LANGUAGES);
const RTL_SCRIPTS = ["Arab", "Hebr", "Thaa", "Syrc", "Nkoo", "Adlm", "Rohg", "Mand", "Samr"];
const isRtl = (tag) => RTL_SCRIPTS.includes(new Intl.Locale(tag).maximize().script);

const path = (tag) => (tag === "en" ? "/privacy" : "/privacy/" + tag);
const file = (tag) => (tag === "en" ? join(pub, "privacy.html") : join(pub, "privacy", tag + ".html"));
const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

const en = JSON.parse(readFileSync(join(pub, "i18n", "en.json"), "utf8"));
const KEYS = ["skip", "nav.home", "nav.label", "nav.why", "nav.security", "nav.download", "lang.label", "lang.draft",
  "foot.line", "meta.privacy.title", "meta.privacy.desc", "privacy.langs", "privacy.official", "privacy.english"];
for (const k of KEYS) if (typeof en[k] !== "string" || !en[k]) errors.push(`en.json: \`${k}\` is missing`);

// Shape of an article: its block elements in order, its links, its code
// and its version numbers.
function shape(html) {
  return {
    blocks: [...html.matchAll(/<(h1|h2|p|ul|li)\b[^>]*>/g)].map((m) => m[1]),
    classes: [...html.matchAll(/<(?:h1|h2|p|ul|li)\b[^>]*class="([^"]+)"/g)].map((m) => m[1]),
    links: [...html.matchAll(/href="([^"]*)"/g)].map((m) => m[1]),
    em: [String((html.match(/<em>/g) || []).length)],
    code: [...html.matchAll(/<code>([^<]*)<\/code>/g)].map((m) => m[1]),
    versions: [...html.matchAll(/\d+\.\d+\.\d+(?:-[a-z]+\.\d+)?/g)].map((m) => m[0]),
  };
}

function lint(tag, html) {
  // Only the markup the English text uses, written plainly.
  for (const m of html.matchAll(/<\/?([a-zA-Z0-9]+)([^>]*)>/g)) {
    if (!["h1", "h2", "p", "ul", "li", "a", "code", "em"].includes(m[1])) errors.push(`site/privacy/${tag}.html: <${m[1]}> is not allowed`);
    const attrs = m[2].trim();
    if (attrs && !/^(class="(eyebrow|lead)"|href="(mailto:privacy@totaldrop\.app|https:\/\/www\.cloudflare\.com\/privacypolicy\/)")$/.test(attrs)) errors.push(`site/privacy/${tag}.html: attribute ${attrs} is not allowed`);
  }
  if (/<script|on[a-z]+=|javascript:/i.test(html)) errors.push(`site/privacy/${tag}.html: script`);
  if (/[‪-‮⁦-⁩]/.test(html)) errors.push(`site/privacy/${tag}.html: invisible direction characters`);
  if (/&(?!amp;|lt;|gt;|quot;|nbsp;)/.test(html)) errors.push(`site/privacy/${tag}.html: unescaped &`);
}

function page(tag, body, d) {
  // A string missing from this language shows in English, marked as such.
  const has = (k) => typeof d[k] === "string" && d[k] !== "";
  const s = (k) => (has(k) ? d[k] : en[k]);
  const text = (k) => (has(k) || tag === "en" ? esc(d[k] || en[k]) : `<span lang="en" dir="ltr">${esc(en[k])}</span>`);
  const dir = isRtl(tag) ? "rtl" : "ltr";
  const url = SITE + path(tag);
  const alternates = tags.map((t) => `  <link rel="alternate" hreflang="${t}" href="${SITE + path(t)}">`).join("\n");
  const others = tags.map((t) => {
    const cur = t === tag ? ' aria-current="page"' : "";
    return `        <li><a href="${path(t)}" lang="${t}" hreflang="${t}"${cur}>${esc(nameOf[t])}</a></li>`;
  }).join("\n");
  let note = "";
  if (tag !== "en") {
    const draft = REVIEWED.includes(tag) ? "" : text("lang.draft") + " ";
    note = `      <p class="lang-note" role="note">${draft}${text("privacy.official")} <a href="/privacy" hreflang="en">${text("privacy.english")}</a></p>\n`;
  }
  // In right-to-left text the bidi algorithm reorders the pieces of a Latin
  // version number or code ("0.3.0-beta.2" shows as "beta.2-0.3.0"), so they
  // and the file paths in <em> are isolated as left-to-right runs. <code> is
  // isolated by styles.css. Only text between tags is touched.
  if (dir === "rtl") {
    body = body
      .replace(/<em>/g, '<em dir="ltr">')
      .replace(/>([^<]*)</g, (_, t) => ">" + t.replace(/\d+(?:\.\d+){2}(?:-beta\.\d+)?|\d-[a-z]+-[a-z]+/g, '<bdi dir="ltr">$&</bdi>') + "<");
  }
  const indented = body.trim().split("\n").map((l) => (l ? "      " + l.replace(/^ {6}/, "") : l)).join("\n");
  return `<!doctype html>
<!-- Generated by site/tools/privacy.mjs from site/privacy/${tag}.html; edit those, then run it with --write. -->
<html lang="${tag}" dir="${dir}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${esc(s("meta.privacy.title"))}</title>
  <meta name="description" content="${esc(s("meta.privacy.desc"))}">
  <meta name="theme-color" content="#F4F2FB" media="(prefers-color-scheme: light)">
  <meta name="theme-color" content="#140E33" media="(prefers-color-scheme: dark)">
  <link rel="canonical" href="${url}">
${alternates}
  <link rel="alternate" hreflang="x-default" href="${SITE}/privacy">
  <meta property="og:title" content="${esc(s("meta.privacy.title"))}">
  <meta property="og:description" content="${esc(s("meta.privacy.desc"))}">
  <meta property="og:url" content="${url}">
  <meta property="og:type" content="website">
  <link rel="icon" href="/favicon.svg" type="image/svg+xml">
  <link rel="stylesheet" href="/styles.css">
</head>
<body>
  <a class="skip" href="#main">${text("skip")}</a>

  <header class="top">
    <a class="brand" href="${SITE}/" aria-label="${esc(s("nav.home"))}">
      <svg class="mark" viewBox="0 0 112 112" aria-hidden="true"><path class="mk" d="M55 15H97V57A42 42 0 1 1 55 15Z"/></svg>
      <span>TotalDrop</span>
    </a>
    <nav class="top-nav top-nav-links" aria-label="${esc(s("nav.label"))}">
      <a href="${SITE}/#why">${text("nav.why")}</a>
      <a href="${SITE}/#security">${text("nav.security")}</a>
      <a href="https://download.totaldrop.app/">${text("nav.download")}</a>
    </nav>
    <nav class="top-nav top-nav-keep" aria-label="${esc(s("lang.label"))}">
      <a href="#languages">${text("lang.label")}</a>
    </nav>
  </header>

  <main id="main" class="prose">
    <article lang="${tag}">
${note}${indented}
    </article>

    <nav class="lang-list" id="languages" aria-labelledby="languages-title">
      <h2 id="languages-title">${text("privacy.langs")}</h2>
      <ul>
${others}
      </ul>
    </nav>
  </main>

  <footer class="foot">
    <p>${text("foot.line")}</p>
    <p class="foot-links"><a href="${SITE}/">totaldrop.app</a> <a href="mailto:privacy@totaldrop.app">privacy@totaldrop.app</a></p>
  </footer>
</body>
</html>
`;
}

const sources = existsSync(src) ? readdirSync(src).filter((f) => f.endsWith(".html")).map((f) => f.replace(/\.html$/, "")).sort() : [];
if (JSON.stringify(sources) !== JSON.stringify([...tags].sort())) errors.push(`site/privacy/ has ${sources.join(" ")}; lang.js lists ${[...tags].sort().join(" ")}`);

const enBody = existsSync(join(src, "en.html")) ? readFileSync(join(src, "en.html"), "utf8") : "";
const want = shape(enBody);
const stale = [];
for (const tag of tags) {
  const p = join(src, tag + ".html");
  if (!existsSync(p)) continue;
  const body = readFileSync(p, "utf8");
  lint(tag, body);
  const got = shape(body);
  for (const k of Object.keys(want)) {
    if (JSON.stringify(got[k]) !== JSON.stringify(want[k])) errors.push(`site/privacy/${tag}.html: ${k} differ from English\n  en: ${want[k].join(" | ")}\n  ${tag}: ${got[k].join(" | ")}`);
  }
  let d = {};
  try { d = JSON.parse(readFileSync(join(pub, "i18n", tag + ".json"), "utf8")); } catch (e) { errors.push(`${tag}.json: ${e.message}`); }
  const html = page(tag, body, d);
  const out = file(tag);
  if (write) {
    mkdirSync(dirname(out), { recursive: true });
    writeFileSync(out, html);
  } else if (!existsSync(out) || readFileSync(out, "utf8") !== html) stale.push(out.slice(root.length + 1));
}
// Nothing else in public/privacy/: a page left behind by a removed language.
if (existsSync(join(pub, "privacy"))) {
  for (const f of readdirSync(join(pub, "privacy"))) if (!tags.includes(f.replace(/\.html$/, ""))) errors.push(`site/public/privacy/${f}: no such language`);
}
if (stale.length) errors.push(`out of date, run node site/tools/privacy.mjs --write: ${stale.join(" ")}`);
// Every page is in the sitemap.
const sitemap = readFileSync(join(pub, "sitemap.xml"), "utf8");
for (const tag of tags) if (!sitemap.includes(`<loc>${SITE + path(tag)}</loc>`)) errors.push(`sitemap.xml: ${SITE + path(tag)} missing`);

if (errors.length) {
  console.error(errors.join("\n"));
  process.exit(1);
}
console.log(`privacy policy ${write ? "written" : "ok"}: ${tags.length} languages`);
