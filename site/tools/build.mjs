// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Writes the folder that is published (ADR 0021, #513):
//   node site/tools/build.mjs                 site/public → site/dist
//   node site/tools/build.mjs --out <folder>  another folder under site/
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
import { cpSync, existsSync, lstatSync, mkdirSync, readFileSync, readdirSync, realpathSync, rmSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import vm from "node:vm";

const here = dirname(fileURLToPath(import.meta.url));
const siteRoot = realpathSync(resolve(here, ".."));
export const pub = join(siteRoot, "public");
const PROTECTED = ["public", "tools", "functions", "privacy", "team", "README.md", "wrangler.toml", ".wrangler"].map((p) => {
  const full = join(siteRoot, p);
  return existsSync(full) ? realpathSync(full) : full;
});

function within(parent, child) {
  const rel = relative(parent, child);
  return rel === "" || (rel !== ".." && !rel.startsWith(".." + sep) && !isAbsolute(rel));
}

function canonicalTarget(out) {
  let cur = resolve(out);
  const tail = [];
  while (true) {
    let st;
    try {
      st = lstatSync(cur, { throwIfNoEntry: false });
    } catch (e) {
      if (e.code === "ENOTDIR") throw new Error(`refusing to write to ${out}: not a folder`);
      throw new Error(`refusing to write to ${out}: broken or invalid path`);
    }
    if (st) {
      if (tail.length === 0) {
        if (st.isSymbolicLink()) throw new Error(`refusing to write to ${out}: output path is a symlink`);
        if (!st.isDirectory()) throw new Error(`refusing to write to ${out}: not a folder`);
      }
      let realBase;
      try {
        realBase = realpathSync(cur);
      } catch {
        throw new Error(`refusing to write to ${out}: broken or invalid path`);
      }
      if (!statSync(realBase).isDirectory()) throw new Error(`refusing to write to ${out}: not a folder`);
      return join(realBase, ...tail.reverse());
    }
    const parent = dirname(cur);
    if (parent === cur) throw new Error(`refusing to write to ${out}: broken or invalid path`);
    tail.push(basename(cur));
    cur = parent;
  }
}

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
  }
  if (has(dict, `meta.${page}.desc`)) {
    once(/(<meta name="description" content=")[^"]*(">)/, (_, a, b) => a + attr(desc) + b);
    once(/(<meta property="og:description" content=")[^"]*(">)/, (_, a, b) => a + attr(desc) + b);
  }
  once(/(<link rel="canonical" href=")[^"]*(">)/, (_, a, b) => a + own + b);
  once(/(<meta property="og:url" content=")[^"]*(">)/, (_, a, b) => a + own + b);

  // Text. A translated element holds text only (check-i18n.mjs).
  html = html.replace(/<([a-z0-9]+)\b([^>]*\bdata-i18n="([^"]+)"[^>]*)>([\s\S]*?)<\/\1>/g, (all, name, attrs, key, inner) => {
    if (has(dict, key)) return `<${name}${attrs}>${text(dict[key])}</${name}>`;
    return `<${name}${attrs} lang="en" dir="ltr">${inner}</${name}>`;
  });
  // Accessible names.
  html = html.replace(/<[a-z0-9]+\b[^>]*\bdata-i18n-label="([^"]+)"[^>]*>/g, (open, key) => {
    if (!has(dict, key)) return open;
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
  // Translated addresses never redirect (ADR 0021, #554), so move lang.js
  // out of <head> and defer it before app.js at the end of <body>.
  const headLang = /[ \t]*<script src="\/lang\.js(?:\?v=([0-9a-f]+))?" data-redirect><\/script>\n/;
  const headLangMatch = headLang.exec(html);
  if (headLangMatch) {
    const langSrc = headLangMatch[1] ? `/lang.js?v=${headLangMatch[1]}` : "/lang.js";
    html = html.replace(headLang, "");
    once(
      /([ \t]*)(<script src="\/app\.js(?:\?v=[0-9a-f]+)?" defer><\/script>)/,
      (_, appIndent, appTag) => `${appIndent}<script src="${attr(langSrc)}" defer></script>\n${appIndent}${appTag}`,
    );
  }

  return html;
}

// Any version of a page with the links to all its versions.
export function withAlternates(html, canonical) {
  const re = /(<link rel="canonical" href="[^"]*">\n)/;
  if (!re.test(html)) throw new Error("a page without a canonical link on its own line");
  return html.replace(re, (_, a) => `${a}${alternates(canonical)}\n`);
}

// A list of all 35 languages at the end of <main>, so language switching works
// without JavaScript too (as on the privacy policy pages, #513).
export function withLanguages(html, canonical, tag, dict = {}, enDict = {}) {
  const re = /\n  <\/main>/;
  if (!re.test(html)) throw new Error("a page without a closing </main> on its own line");
  const u = new URL(canonical);
  const enHeading = has(enDict, "lang.label") ? enDict["lang.label"] : "Language";
  const heading = tag === "en" || has(dict, "lang.label")
    ? `<h2 id="languages-title">${text(tag === "en" ? enHeading : dict["lang.label"])}</h2>`
    : `<h2 id="languages-title" lang="en" dir="ltr">${text(enHeading)}</h2>`;
  const items = Array.from(LANGUAGES, ([t, name]) => {
    const href = u.hostname === "download.totaldrop.app" ? addressIn(canonical, t) : address(t, u.pathname);
    const tdir = isRtl(t) === isRtl(tag) ? "" : ` dir="${isRtl(t) ? "rtl" : "ltr"}"`;
    const cur = t === tag ? ' aria-current="page"' : "";
    return `        <li><a href="${href}" lang="${t}" hreflang="${t}"${tdir}${cur}>${text(name)}</a></li>`;
  }).join("\n");
  const nav = [
    `    <nav class="lang-list" id="languages" aria-labelledby="languages-title">`,
    `      ${heading}`,
    `      <ul>`,
    items,
    `      </ul>`,
    `    </nav>`,
  ].join("\n");
  return html.replace(re, `\n\n${nav}\n  </main>`);
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
  if (typeof out !== "string" || !out.trim()) throw new Error("refusing to write: empty output path");
  // The folder is emptied first, so it must live under site/, never overlap
  // tracked files, and never wipe a non-empty folder without a prior build.
  const target = canonicalTarget(out);
  if (target === siteRoot || !within(siteRoot, target)) {
    throw new Error(`refusing to write to ${out}: output folder must be under site/`);
  }
  for (const kept of PROTECTED) {
    if (within(target, kept) || within(kept, target)) {
      throw new Error(`refusing to write to ${out}: overlaps tracked files in site/`);
    }
  }
  if (existsSync(target) && readdirSync(target).length > 0) {
    const markerSitemap = lstatSync(join(target, "sitemap.xml"), { throwIfNoEntry: false });
    const markerTr = lstatSync(join(target, "tr", "index.html"), { throwIfNoEntry: false });
    if (!markerSitemap?.isFile() || !markerTr?.isFile()) {
      throw new Error(`refusing to write to ${out}: non-empty folder has no previous site build`);
    }
  }
  for (const tag of TAGS) {
    if (tag !== "en" && existsSync(join(pub, tag))) throw new Error(`site/public/${tag} would be hidden by the language ${tag}`);
  }
  const enDict = JSON.parse(readFileSync(join(pub, "i18n", "en.json"), "utf8"));
  const dicts = new Map(TAGS.map((t) => [t, t === "en" ? enDict : JSON.parse(readFileSync(join(pub, "i18n", t + ".json"), "utf8"))]));
  const canonicals = [];
  const rendered = [];
  const errors = [];
  for (const page of PAGES) {
    const english = readFileSync(join(pub, page), "utf8");
    const canonical = canonicalOf(english);
    canonicals.push(canonical);
    for (const tag of TAGS) {
      const relFile = tag === "en" ? page : join(tag, page);
      const translated = translate(english, tag, dicts.get(tag));
      const html = withLanguages(withAlternates(translated, canonical), canonical, tag, dicts.get(tag), enDict);
      rendered.push({ relFile, html });
    }
  }
  const sitemap = readFileSync(join(pub, "sitemap.xml"), "utf8");
  if (!sitemap.includes("</urlset>")) throw new Error("sitemap.xml: no </urlset>");
  for (const c of canonicals) if (!sitemap.includes(`<loc>${c}</loc>`)) errors.push(`sitemap.xml: ${c} missing`);
  const added = addedAddresses(canonicals).map((a) => `  <url><loc>${a}</loc></url>\n`).join("");
  const builtSitemap = sitemap.replace("</urlset>", added + "</urlset>");

  rmSync(target, { recursive: true, force: true });
  cpSync(pub, target, { recursive: true });
  for (const { relFile, html } of rendered) {
    const file = join(target, relFile);
    mkdirSync(dirname(file), { recursive: true });
    writeFileSync(file, html);
  }
  writeFileSync(join(target, "sitemap.xml"), builtSitemap);

  // Checked once everything is written, since the pages link to each other.
  for (const { relFile, html } of rendered) {
    for (const ref of missingFiles(html, target)) errors.push(`${relFile} → ${ref} missing`);
  }
  if (errors.length) throw new Error(errors.join("\n"));
  return { written: rendered.length, languages: TAGS.length };
}

export function parseCliOut(args) {
  let out;
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--out") {
      if (out !== undefined || i + 1 >= args.length || !args[i + 1] || args[i + 1].startsWith("-")) {
        throw new Error("refusing to write: usage: node site/tools/build.mjs [--out <folder>]");
      }
      out = args[++i];
    } else if (arg.startsWith("--out=")) {
      const val = arg.slice("--out=".length);
      if (out !== undefined || !val) {
        throw new Error("refusing to write: usage: node site/tools/build.mjs [--out <folder>]");
      }
      out = val;
    } else {
      throw new Error(`refusing to write: unexpected argument ${arg}`);
    }
  }
  return out ?? join(siteRoot, "dist");
}

if (process.argv[1] && existsSync(process.argv[1]) && realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const out = parseCliOut(process.argv.slice(2));
    const { written, languages } = build(out);
    console.log(`site: ${written} pages in ${languages} languages written to ${out}`);
  } catch (e) {
    console.error(e.message);
    process.exit(1);
  }
}
