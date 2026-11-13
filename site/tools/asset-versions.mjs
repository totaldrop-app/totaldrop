// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Gives the site's stylesheet and scripts a version in their address (#291):
//   node site/tools/asset-versions.mjs           check (CI): every page asks for the current version
//   node site/tools/asset-versions.mjs --write   rewrite the pages
//
// Browsers keep /styles.css and the scripts for hours (Cloudflare sets
// max-age=14400 on .css and .js), so after a deploy a visitor could see a
// new page with last week's stylesheet: unstyled. Each page therefore asks
// for /styles.css?v=<hash>, the first 10 hex digits of the SHA-256 of the
// file. When the file changes the address changes, and the browser fetches
// it again. The pages are committed with the version in them, and the
// check fails when one is out of date; the build (build.mjs) copies them
// as they are, so the translated pages carry the same version.
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync, readdirSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const pub = join(dirname(fileURLToPath(import.meta.url)), "..", "public");
export const ASSETS = ["styles.css", "app.js", "lang.js", "msg.js"];

const versions = new Map();
// The address a page uses for one of ASSETS, e.g. /styles.css?v=0123456789.
export function assetRef(name) {
  if (!ASSETS.includes(name)) throw new Error(`not a versioned asset: ${name}`);
  if (!versions.has(name)) versions.set(name, createHash("sha256").update(readFileSync(join(pub, name))).digest("hex").slice(0, 10));
  return `/${name}?v=${versions.get(name)}`;
}

const REF = new RegExp(`((?:href|src)=")/(${ASSETS.map((a) => a.replace(".", "\\.")).join("|")})(?:\\?v=[0-9a-f]*)?(")`, "g");
export const versioned = (html) => html.replace(REF, (_, a, name, b) => `${a}${assetRef(name)}${b}`);

// A reference the rewrite would not recognise (single quotes, a relative
// path, another query) would ship unversioned; the check reports it.
const ODD = new RegExp(`(?:href|src)\\s*=\\s*["']?[^"'\\s>]*?\\b(${ASSETS.map((a) => a.replace(".", "\\.")).join("|")})(?!\\?v=[0-9a-f]{10}")`, "g");

// What is wrong with a page: "stale" when a reference has an old or no
// version, the unrecognised references otherwise.
export function problems(html) {
  const out = [];
  if (versioned(html) !== html) out.push("stale");
  for (const m of versioned(html).matchAll(ODD)) out.push(`unrecognised reference: ${m[0]}`);
  return out;
}

function pages(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    const p = join(dir, e.name);
    return e.isDirectory() ? pages(p) : e.name.endsWith(".html") ? [p] : [];
  });
}

function main() {
  const write = process.argv.includes("--write");
  const stale = [];
  let count = 0;
  for (const p of pages(pub)) {
    const html = readFileSync(p, "utf8");
    const want = versioned(html);
    if (want !== html && write) writeFileSync(p, want);
    for (const problem of problems(write ? want : html)) {
      stale.push(problem === "stale"
        ? `site/public/${relative(pub, p)}: asks for an old version of a stylesheet or script; run node site/tools/asset-versions.mjs --write`
        : `site/public/${relative(pub, p)}: ${problem}; write it as href="/<file>" or src="/<file>"`);
    }
    count += (want.match(REF) || []).length;
  }
  if (stale.length) {
    for (const s of stale) console.error(`error: ${s}`);
    process.exit(1);
  }
  console.log(`asset versions: ${count} references, ${[...versions].map(([n, v]) => `${n}@${v}`).join(" ")}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
