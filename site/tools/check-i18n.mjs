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
const sandbox = { Intl };
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
const pages = ["index.html", "download/index.html", "donate/index.html", "roadmap/index.html", "r/index.html"];
const fromHtml = {};
const decode = (s) => s.replace(/&#x([0-9a-f]+);/gi, (_, h) => String.fromCodePoint(parseInt(h, 16)))
  .replace(/&#([0-9]+);/g, (_, d) => String.fromCodePoint(Number(d)))
  .replace(/&amp;/g, "&").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, '"').replace(/&nbsp;/g, " ");
for (const page of pages) {
  const html = read("site/public/" + page);
  const re = /<([a-z0-9]+)\b[^>]*\bdata-i18n="([^"]+)"[^>]*>([\s\S]*?)<\/\1>/g;
  let m;
  while ((m = re.exec(html)) !== null) {
    const text = decode(m[3].replace(/<[^>]+>/g, "")).replace(/\s+/g, " ").trim();
    if (!(m[2] in fromHtml)) fromHtml[m[2]] = text;
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

const forbidden = /[\u0000-\u0008\u000b-\u001f\u007f-\u009f\u00ad\u061c\u180e\u200b\u2028-\u202e\u2060-\u206f\ufeff\ufff9-\ufffb]|[\u{e0000}-\u{e007f}]/u;
const link = /:\/\/[a-z0-9]|www\.|[a-z0-9]\.(com|org|net|app|io|co|info|xyz|link|site|online|me|ly|gl|to|dev)\b/i;
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
    // A translation may keep a link the English text already has.
    if (link.test(v) && !link.test(en[k] || "")) errors.push(`${f} \`${k}\`: contains a link`);
    if (markup.test(v)) errors.push(`${f} \`${k}\`: contains markup`);
    try { parse(v); } catch (e) { errors.push(`${f} \`${k}\`: ${e.message}`); }
    // Version numbers must be the same as in English: a stale translation
    // once showed last release's versions.
    const versions = (x) => [...String(x || "").matchAll(/\d+\.\d+\.\d+(?:-[a-z]+\.\d+)?/g)].map((m) => m[0]).sort().join(" ");
    if (k in en && versions(v) !== versions(en[k])) errors.push(`${f} \`${k}\`: versions differ from English (${versions(v)} vs ${versions(en[k])})`);
  }
}

if (errors.length) {
  console.error(errors.join("\n"));
  process.exit(1);
}
console.log(`site translations ok: ${tags.join(", ")}; ${vectors.cases.length} shared vectors pass`);
