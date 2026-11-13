// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Writes the folder that is published (ADR 0021, #513):
//   node site/tools/build.mjs                 site/public → site/dist
//   node site/tools/build.mjs --out <folder>  somewhere else
//
// site/public is copied as it is, so every address that worked keeps its
// content. Then each of PAGES is written once per language under /<tag>/,
// from the English page and site/public/i18n/<tag>.json, with the language
// in the HTML the server sends: lang, dir, title, description, og: tags,
// its own canonical. Every version of a page, English included, lists all
// of them with hreflang, and the sitemap lists every address.
//
// A search engine indexes what an address returns. Before this, every
// address returned English and the browser swapped the text in place, so
// no other language could be found by a search.
//
// The translation rule is the one app.js applies in the browser (it still
// does on /r and the 404 page, which keep a single address): text of every
// data-i18n element, aria-label of every data-i18n-label element; a missing
// or empty translation stays English and is marked lang="en" dir="ltr".
// Tested by: node --test site/tools/test-build.mjs
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import vm from "node:vm";

const here = dirname(fileURLToPath(import.meta.url));
export const pub = join(here, "..", "public");

// The pages that get an address per language. Not here: the privacy policy
// (its own pages, site/tools/privacy.mjs), /r (the code after "#" must keep
// working from every link ever shared) and the 404 page.
export const PAGES = ["index.html", "download/index.html", "roadmap/index.html", "donate/index.html", "team/index.html", "brand/index.html"];

// The language list and the address rules, loaded the way a browser would.
const box = { Intl };
vm.runInNewContext(readFileSync(join(pub, "lang.js"), "utf8"), box);
export const { LANGUAGES, REVIEWED, address } = box.TDLang;
// (Array.from, so the list is an array of this realm, not of the one lang.js ran in.)
export const TAGS = Array.from(LANGUAGES, (l) => l[0]);

// Right to left: the same rule as app.js and privacy.mjs.
const RTL_SCRIPTS = ["Arab", "Hebr", "Thaa", "Syrc", "Nkoo", "Adlm", "Rohg", "Mand", "Samr"];
export const isRtl = (tag) => RTL_SCRIPTS.includes(new Intl.Locale(tag).maximize().script);

const text = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const attr = (s) => text(s).replace(/"/g, "&quot;");
const has = (d, key) => typeof d[key] === "string" && d[key] !== "";

// A page's English address, from its canonical link.
export function canonicalOf(html) {
  const m = /<link rel="canonical" href="(https:\/\/[^"]+)">/.exec(html);
  if (!m) throw new Error("a page without a canonical link");
  return m[1];
}

// The address of that page in a language: download.totaldrop.app/ becomes
// download.totaldrop.app/tr/, totaldrop.app/roadmap/ totaldrop.app/tr/roadmap/.
export function addressIn(canonical, tag) {
  const u = new URL(canonical);
  return u.origin + address(tag, u.pathname);
}

// One hreflang line per language and x-default (English), the same block
// on every version of the page, so each names all the others and itself.
export function alternates(canonical) {
  return TAGS.map((t) => `  <link rel="alternate" hreflang="${t}" href="${addressIn(canonical, t)}">`)
    .concat(`  <link rel="alternate" hreflang="x-default" href="${canonical}">`).join("\n");
}

// Links to the pages that exist in every language follow the language;
// everything else (files, the mark, other sites, "#part") stays.
const PATHS = PAGES.map((p) => "/" + p.replace(/index\.html$/, ""));
function linkIn(href, tag) {
  const m = /^(https:\/\/(?:download\.)?totaldrop\.app)?(\/[^#?]*)?(#.*)?$/.exec(href);
  if (!m || (m[1] === undefined && m[2] === undefined)) return href;
  const path = m[2] || "/";
  if (path === "/privacy") return (m[1] || "") + "/privacy/" + tag + (m[3] || "");
  if (!PATHS.includes(path)) return href;
  return (m[1] || "") + address(tag, path) + (m[3] || "");
}

// The page in one language. `dict` is that language's i18n file.
export function translate(html, tag, dict) {
  if (tag === "en") return html;
  const canonical = canonicalOf(html);
  const own = addressIn(canonical, tag);
  const page = (/<body[^>]*\bdata-page="([^"]+)"/.exec(html) || [])[1] || "home";
  let missing = 0;
  const once = (re, to) => {
    if (!re.test(html)) throw new Error(`${page}: expected ${re}`);
    html = html.replace(re, to);
  };

  once(/<html lang="en">/, `<html lang="${tag}" dir="${isRtl(tag) ? "rtl" : "ltr"}">`);
  const title = dict[`meta.${page}.title`];
  const desc = dict[`meta.${page}.desc`];
  if (has(dict, `meta.${page}.title`)) {
    once(/<title>[^<]*<\/title>/, () => `<title>${text(title)}</title>`);
    once(/(<meta property="og:title" content=")[^"]*(">)/, (_, a, b) => a + attr(title) + b);
  } else missing += 1;
  if (has(dict, `meta.${page}.desc`)) {
    once(/(<meta name="description" content=")[^"]*(">)/, (_, a, b) => a + attr(desc) + b);
    once(/(<meta property="og:description" content=")[^"]*(">)/, (_, a, b) => a + attr(desc) + b);
  } else missing += 1;
  once(/(<link rel="canonical" href=")[^"]*(">)/, (_, a, b) => a + own + b);
  once(/(<meta property="og:url" content=")[^"]*(">)/, (_, a, b) => a + own + b);

  // Text. A translated element holds text only (check-i18n.mjs).
  html = html.replace(/<([a-z0-9]+)\b([^>]*\bdata-i18n="([^"]+)"[^>]*)>([\s\S]*?)<\/\1>/g, (all, name, attrs, key, inner) => {
    if (has(dict, key)) return `<${name}${attrs}>${text(dict[key])}</${name}>`;
    missing += 1;
    return `<${name}${attrs} lang="en" dir="ltr">${inner}</${name}>`;
  });
  // Accessible names.
  html = html.replace(/<[a-z0-9]+\b[^>]*\bdata-i18n-label="([^"]+)"[^>]*>/g, (open, key) => {
    if (!has(dict, key)) { missing += 1; return open; }
    return open.replace(/(\baria-label=")[^"]*(")/, (_, a, b) => a + attr(dict[key]) + b);
  });
  // Links between the pages.
  html = html.replace(/(<a\b[^>]*\bhref=")([^"]*)(")/g, (_, a, href, b) => a + linkIn(href, tag) + b);

  // What app.js says after a copy button is pressed.
  if (html.includes("data-copy=") && has(dict, "dl.copied")) {
    once(/<body\b[^>]*/, (open) => `${open} data-copied="${attr(dict["dl.copied"])}"`);
  }
  // A language no native speaker has checked yet says so before the text.
  if (!REVIEWED.includes(tag)) {
    const note = has(dict, "lang.draft")
      ? `<p class="lang-note" role="note">${text(dict["lang.draft"])}</p>`
      : `<p class="lang-note" role="note" lang="en" dir="ltr">This translation is a draft and is being reviewed.</p>`;
    once(/(<main id="main">\n)/, (_, a) => `${a}    ${note}\n`);
  }
  return html;
}

// Any version of a page with the links to all its versions.
export function withAlternates(html, canonical) {
  const re = /(<link rel="canonical" href="[^"]*">\n)/;
  if (!re.test(html)) throw new Error("a page without a canonical link on its own line");
  return html.replace(re, (_, a) => `${a}${alternates(canonical)}\n`);
}

// Every address the build adds, for the sitemap.
export function addedAddresses(canonicals) {
  return canonicals.flatMap((c) => TAGS.filter((t) => t !== "en").map((t) => addressIn(c, t)));
}

// Every /path a page asks for must exist in the published folder.
export function missingFiles(html, out) {
  const bad = [];
  for (const m of html.matchAll(/(?:href|src)="\/([^"#?]*)[^"]*"/g)) {
    const ref = m[1];
    if (ref === "" || existsSync(join(out, ref)) || existsSync(join(out, ref, "index.html")) || existsSync(join(out, ref + ".html"))) continue;
    bad.push("/" + ref);
  }
  return bad;
}

export function build(out) {
  rmSync(out, { recursive: true, force: true });
  cpSync(pub, out, { recursive: true });
  for (const tag of TAGS) {
    if (tag !== "en" && existsSync(join(pub, tag))) throw new Error(`site/public/${tag} would be hidden by the language ${tag}`);
  }
  const dicts = new Map(TAGS.filter((t) => t !== "en").map((t) => [t, JSON.parse(readFileSync(join(pub, "i18n", t + ".json"), "utf8"))]));
  const canonicals = [];
  const errors = [];
  let written = 0;
  for (const page of PAGES) {
    const english = readFileSync(join(pub, page), "utf8");
    const canonical = canonicalOf(english);
    canonicals.push(canonical);
    for (const tag of TAGS) {
      const file = tag === "en" ? join(out, page) : join(out, tag, page);
      const html = withAlternates(translate(english, tag, dicts.get(tag)), canonical);
      mkdirSync(dirname(file), { recursive: true });
      writeFileSync(file, html);
      written += 1;
    }
  }
  // Checked once everything is written, since the pages link to each other.
  for (const page of PAGES) {
    for (const tag of TAGS) {
      const file = tag === "en" ? join(out, page) : join(out, tag, page);
      for (const ref of missingFiles(readFileSync(file, "utf8"), out)) errors.push(`${tag === "en" ? "" : tag + "/"}${page} → ${ref} missing`);
    }
  }
  const sitemap = readFileSync(join(pub, "sitemap.xml"), "utf8");
  if (!sitemap.includes("</urlset>")) throw new Error("sitemap.xml: no </urlset>");
  for (const c of canonicals) if (!sitemap.includes(`<loc>${c}</loc>`)) errors.push(`sitemap.xml: ${c} missing`);
  const added = addedAddresses(canonicals).map((a) => `  <url><loc>${a}</loc></url>\n`).join("");
  writeFileSync(join(out, "sitemap.xml"), sitemap.replace("</urlset>", added + "</urlset>"));
  if (errors.length) throw new Error(errors.join("\n"));
  return { written, languages: TAGS.length };
}

if (import.meta.url === pathToFileURL(process.argv[1] || "").href) {
  const at = process.argv.indexOf("--out");
  const out = at > 0 ? process.argv[at + 1] : join(here, "..", "dist");
  try {
    const { written, languages } = build(out);
    console.log(`site: ${written} pages in ${languages} languages written to ${out}`);
  } catch (e) {
    console.error(e.message);
    process.exit(1);
  }
}
