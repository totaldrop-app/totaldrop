// Checks the site's translations (ADR 0008), run in CI:
//   node site/tools/check-i18n.mjs
// - every site/public/i18n/<tag>.json is valid, names its own @@locale and
//   is listed in app.js's LANGUAGES (and the other way round);
// - en.json matches the English text in the pages;
// - no translation has keys English lacks, direction or invisible control
//   characters, links or markup, or a message the formatter rejects;
// - the formatter passes the shared vectors in l10n/tests/messages.json.
// Missing translations are reported, not errors.
import { readFileSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const pub = join(root, "site", "public");
const errors = [];
const read = (p) => readFileSync(join(root, p), "utf8");

// The formatter, loaded the way the browser would.
const sandbox = { Intl, TextEncoder };
vm.runInNewContext(read("site/public/msg.js"), sandbox);
const { parse, format } = sandbox.TDMessage;

// Shared vectors.
const vectors = JSON.parse(read("l10n/tests/messages.json"));
for (const c of vectors.cases) {
  let got;
  try { got = format(c.message, c.args, c.locale); } catch (e) { got = "ERR " + e.message; }
  if (got !== c.expect) errors.push(`vector ${c.locale} ${JSON.stringify(c.message)}: got ${JSON.stringify(got)}, want ${JSON.stringify(c.expect)}`);
}
for (const bad of vectors.invalid) {
  try { parse(bad); errors.push(`vector should be rejected: ${JSON.stringify(bad)}`); } catch (e) { /* expected */ }
}

// English from the pages: textContent of every data-i18n element.
const pages = ["index.html", "download/index.html", "donate/index.html", "roadmap/index.html", "r/index.html", "404.html"];
const fromHtml = {};
const NAMED = { amp: "&", lt: "<", gt: ">", quot: '"', apos: "'", nbsp: "\u00a0", rsquo: "\u2019", lsquo: "\u2018", rdquo: "\u201d", ldquo: "\u201c", mdash: "\u2014", ndash: "\u2013", hellip: "\u2026", middot: "\u00b7", copy: "\u00a9" };
const decode = (s) => s.replace(/&([a-z]+);/gi, (all, n) => {
  if (!(n in NAMED)) { errors.push(`unknown entity ${all}`); return all; }
  return "\u0000" + n + ";";
}).replace(/&#x([0-9a-f]+);/gi, (_, h) => String.fromCodePoint(parseInt(h, 16)))
  .replace(/&#([0-9]+);/g, (_, d) => String.fromCodePoint(Number(d)))
  .replace(/&amp;/g, "&").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, '"').replace(/&nbsp;/g, " ");
for (const page of pages) {
  const html = read("site/public/" + page).replace(/<!--[\s\S]*?-->/g, "");
  if (/data-i18n=\'|<[A-Z]+[^>]*data-i18n/.test(html)) errors.push(`${page}: data-i18n must be a double-quoted attribute on a lowercase tag`);
  const re = /<([a-z0-9]+)\b[^>]*\bdata-i18n="([^"]+)"[^>]*>([\s\S]*?)<\/\1>/g;
  let m;
  while ((m = re.exec(html)) !== null) {
    // apply() replaces textContent, so a translated element may hold no
    // other elements (they would be lost in every other language).
    if (/<[a-z]/i.test(m[3])) errors.push(`${page} \`${m[2]}\`: a translated element has child elements`);
    const text = decode(m[3]).replace(/\s+/g, " ").trim();
    if (!(m[2] in fromHtml)) fromHtml[m[2]] = text;
  }
  // Accessible names: aria-label="…" data-i18n-label="key".
  for (const l of html.matchAll(/aria-label="([^"]*)"\s+data-i18n-label="([^"]+)"/g)) {
    if (l[2] !== "lang.label" && !(l[2] in fromHtml)) fromHtml[l[2]] = decode(l[1]);
  }
}

const files = readdirSync(join(pub, "i18n")).filter((f) => f.endsWith(".json")).sort();
const en = JSON.parse(readFileSync(join(pub, "i18n", "en.json"), "utf8"));
for (const [k, v] of Object.entries(fromHtml)) {
  if (en[k] !== v) errors.push(`en.json \`${k}\` differs from the page: ${JSON.stringify(v)}`);
}

const app = read("site/public/app.js");
const block = app.slice(app.indexOf("const LANGUAGES"), app.indexOf("];", app.indexOf("const LANGUAGES")));
const listed = [...block.matchAll(/\["([a-zA-Z-]+)", "[^"]+"\]/g)].map((m) => m[1]).sort();
const tags = files.map((f) => f.replace(/\.json$/, "")).sort();
if (JSON.stringify(listed) !== JSON.stringify(tags)) errors.push(`app.js LANGUAGES ${listed} differ from i18n files ${tags}`);

// Controls, format characters and separators (as the Rust check), except
// ZWNJ/ZWJ and the left-to-right and right-to-left marks.
const forbidden = /(?![\u200c\u200d\u200e\u200f])[\p{Cc}\p{Cf}\p{Zl}\p{Zp}]/u;
// Anything that looks like a link or a domain: a translation may contain
// exactly the links its English text has, no others.
const links = (x) => [...String(x || "").matchAll(/[a-z][a-z0-9+.-]*:\/\/[^\s"'<>]+|www\.[^\s"'<>]+|[\p{L}\p{N}-]+(?:\.[\p{L}\p{N}-]+)*\.\p{L}{2,}(?:\/[^\s"'<>]*)?/giu)]
  .map((m) => m[0].toLowerCase().replace(/[.,;:!?)]+$/, "")).filter((l) => !/^\p{L}\.\p{L}/u.test(l)).sort();
const markup = /<[A-Za-z/!]/;
for (const f of files) {
  const tag = f.replace(/\.json$/, "");
  let d;
  try { d = JSON.parse(readFileSync(join(pub, "i18n", f), "utf8")); } catch (e) { errors.push(`${f}: ${e.message}`); continue; }
  if (d["@@locale"] !== tag) errors.push(`${f}: @@locale is ${d["@@locale"]}`);
  let missing = 0;
  for (const k of Object.keys(en)) if (!k.startsWith("@") && !(k in d)) missing++;
  if (missing) console.log(`${tag}: ${missing} strings not translated yet (shown in English)`);
  for (const [k, v] of Object.entries(d)) {
    if (k.startsWith("@")) continue;
    if (!(k in en)) errors.push(`${f} \`${k}\`: not in en.json`);
    if (typeof v !== "string") { errors.push(`${f} \`${k}\`: not text`); continue; }
    if (forbidden.test(v)) errors.push(`${f} \`${k}\`: invisible or control character`);
    const extra = links(v).filter((l) => !links(en[k]).includes(l));
    if (extra.length) errors.push(`${f} \`${k}\`: contains a link not in English: ${extra.join(" ")}`);
    if (markup.test(v)) errors.push(`${f} \`${k}\`: contains markup`);
    try { parse(v); } catch (e) { errors.push(`${f} \`${k}\`: ${e.message}`); }
    // Version numbers must be the same as in English: a stale translation
    // once showed last release's versions.
    const versions = (x) => [...String(x || "").matchAll(/\d+\.\d+\.\d+(?:-[a-z]+\.\d+)?/g)].map((m) => m[0])
      .filter((x) => !/^\d{1,3}(\.\d{3})+$/.test(x)).sort().join(" "); // 1.000.000 is a number
    if (k in en && versions(v) !== versions(en[k])) errors.push(`${f} \`${k}\`: versions differ from English (${versions(v)} vs ${versions(en[k])})`);
  }
}

if (errors.length) {
  console.error(errors.join("\n"));
  process.exit(1);
}
console.log(`site translations ok: ${tags.join(", ")}; ${vectors.cases.length} shared vectors pass`);
