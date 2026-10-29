// Checks the README and its translations in docs/readme/, run in CI:
//   node site/tools/check-readmes.mjs          check
//   node site/tools/check-readmes.mjs --write  rewrite the language lists
//                                              (and the source marker of
//                                              translations given as args)
// - every site language (site/public/lang.js) has docs/readme/README.<tag>.md;
// - every README carries the same language list, its own language in bold;
// - every relative link and image resolves to a file in the repository;
// - no direction or invisible control characters (ZWNJ only in Persian);
// - a translation older than README.md is reported, not an error: its first
//   line names the README.md it was translated from.
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { createHash } from "node:crypto";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const read = (p) => readFileSync(join(root, p), "utf8");
const box = { Intl };
vm.runInNewContext(read("site/public/lang.js"), box);
const { LANGUAGES } = box.TDLang;
const write = process.argv.includes("--write");
const stamp = process.argv.slice(2).filter((a) => !a.startsWith("--"));
const errors = [];
const warnings = [];

const fileOf = (tag) => (tag === "en" ? "README.md" : `docs/readme/README.${tag}.md`);
const source = createHash("sha256").update(read("README.md")).digest("hex").slice(0, 12);
const marker = (s) => `<!-- Translated from README.md ${s}. -->`;

function list(tag) {
  const from = dirname(fileOf(tag));
  const rel = (to) => {
    const up = from === "." ? "" : "../../";
    return to === "README.md" ? up + "README.md" : (from === "." ? to : to.replace("docs/readme/", ""));
  };
  const items = LANGUAGES.map(([t, name]) => (t === tag ? `**${name}**` : `[${name}](${rel(fileOf(t))})`));
  return `<!-- languages -->\n<sub>${items.join(" · ")}</sub>\n<!-- /languages -->`;
}

const block = /<!-- languages -->[\s\S]*?<!-- \/languages -->/;
const forbidden = /[\p{Cc}\p{Cf}\p{Zl}\p{Zp}]/u;

for (const [tag] of LANGUAGES) {
  const path = fileOf(tag);
  if (!existsSync(join(root, path))) { errors.push(`${path}: missing`); continue; }
  let text = read(path);
  if (!block.test(text)) { errors.push(`${path}: no <!-- languages --> list`); continue; }
  if (write) {
    text = text.replace(block, list(tag));
    if (tag !== "en" && stamp.includes(tag)) {
      text = text.replace(/^<!-- Translated from README\.md [0-9a-f]+\. -->\n/, "");
      text = marker(source) + "\n" + text;
    }
    writeFileSync(join(root, path), text);
  } else if (text.match(block)[0] !== list(tag)) {
    errors.push(`${path}: the language list differs (run with --write)`);
  }
  if (tag !== "en") {
    const m = /^<!-- Translated from README\.md ([0-9a-f]{12})\. -->/.exec(text);
    if (!m) errors.push(`${path}: the first line must name the README.md it was translated from`);
    else if (m[1] !== source) warnings.push(`${path}: translated from an older README.md (${m[1]}, now ${source})`);
  }
  const lines = text.split("\n");
  lines.forEach((line, i) => {
    // Persian spelling needs ZWNJ; nothing else may hide in the text.
    const bad = (tag === "fa" ? line.replace(/\u200c/g, "") : line).replace(/\t/g, "");
    if (forbidden.test(bad)) errors.push(`${path}:${i + 1}: invisible or control character`);
  });
  // Relative links and images: [..](path), src="path", srcset="path".
  const dir = dirname(join(root, path));
  const refs = [...text.matchAll(/\]\(([^)\s]+)\)|(?:src|srcset)="([^"]+)"/g)].map((x) => x[1] || x[2]);
  for (const ref of refs) {
    if (/^(https?:|mailto:|#)/.test(ref)) continue;
    const file = ref.split("#")[0];
    if (file && !existsSync(join(dir, file))) errors.push(`${path}: link to ${ref} does not exist`);
  }
}

for (const w of warnings) console.log(`::warning::${w}`);
if (errors.length) {
  console.error(errors.join("\n"));
  process.exit(1);
}
console.log(`READMEs ok: ${LANGUAGES.length} languages${warnings.length ? `, ${warnings.length} translations to update` : ""}`);
