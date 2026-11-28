// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test site/tools/test-build.mjs
//
// The published folder (site/tools/build.mjs, ADR 0021, #513): a page per
// language whose HTML says its language without any script, the addresses
// that existed before unchanged, and the address rules the pages and the
// download host name follow.
import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { lstatSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, existsSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { join, parse } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import * as buildMod from "./build.mjs";
import { build, pub, PAGES, TAGS, REVIEWED, translate, withAlternates, addressIn, canonicalOf, isRtl } from "./build.mjs";
import * as middlewareMod from "../functions/_middleware.js";
import { onRequest } from "../functions/_middleware.js";

const siteDir = join(pub, "..");
const out = mkdtempSync(join(siteDir, ".test-build-"));
const cleanupOut = () => rmSync(out, { recursive: true, force: true });
process.on("exit", cleanupOut);
test.after(cleanupOut);
const built = build(out);
const read = (...p) => readFileSync(join(...p), "utf8");
const dict = (tag) => JSON.parse(read(pub, "i18n", tag + ".json"));
const others = TAGS.filter((t) => t !== "en");
const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
// The file that answers an address: download.totaldrop.app/tr/ is
// tr/download/index.html (the middleware picks it on that host name).
function fileOf(address) {
  const u = new URL(address);
  const path = u.hostname === "download.totaldrop.app" ? u.pathname + "download/" : u.pathname;
  return join(out, path, "index.html");
}

test("35 languages, six pages each", () => {
  assert.equal(TAGS.length, 35);
  assert.equal(built.written, 35 * PAGES.length);
  for (const tag of others) for (const page of PAGES) assert.ok(existsSync(join(out, tag, page)), `${tag}/${page}`);
});

test("the HTML says its language and direction without a script", () => {
  assert.deepEqual(TAGS.filter(isRtl), ["ar", "fa", "he", "ur"]);
  for (const tag of others) {
    for (const page of PAGES) {
      const html = read(out, tag, page);
      assert.ok(html.includes(`<html lang="${tag}" dir="${isRtl(tag) ? "rtl" : "ltr"}">`), `${tag}/${page}`);
    }
  }
});

test("title, description and link preview are the language's own", () => {
  for (const tag of others) {
    const d = dict(tag);
    for (const page of PAGES) {
      const html = read(out, tag, page);
      const name = (/data-page="([^"]+)"/.exec(html) || [])[1] || "home";
      const title = d[`meta.${name}.title`];
      const desc = d[`meta.${name}.desc`];
      assert.ok(title && desc, `${tag}: meta.${name} is translated`);
      assert.ok(html.includes(`<title>${esc(title)}</title>`), `${tag}/${page} title`);
      for (const [key, value] of [['name="description"', desc], ['property="og:title"', title], ['property="og:description"', desc]]) {
        assert.ok(html.includes(`<meta ${key} content="${esc(value).replace(/"/g, "&quot;")}">`), `${tag}/${page} ${key}`);
      }
    }
  }
});

test("no translated text is left in English, and what is missing is marked as English", () => {
  for (const tag of others) {
    const d = dict(tag);
    for (const page of PAGES) {
      const html = read(out, tag, page);
      let seen = 0;
      for (const m of html.matchAll(/<([a-z0-9]+)\b([^>]*)\bdata-i18n="([^"]+)"([^>]*)>([\s\S]*?)<\/\1>/g)) {
        seen += 1;
        const translated = typeof d[m[3]] === "string" && d[m[3]] !== "";
        if (translated) assert.equal(m[5], esc(d[m[3]]), `${tag}/${page} ${m[3]}`);
        else assert.ok(/ lang="en" dir="ltr"/.test(m[2] + m[4]), `${tag}/${page} ${m[3]} is marked English`);
      }
      assert.ok(seen > 20, `${tag}/${page} has translated elements`);
      // Counted another way, so an element the pattern cannot read is noticed.
      assert.equal(seen, html.split(' data-i18n="').length - 1, `${tag}/${page}: every data-i18n element was read`);
    }
  }
});

test("every version names itself as canonical and lists all 35 and x-default", () => {
  for (const page of PAGES) {
    const english = canonicalOf(read(pub, page));
    let block = null;
    for (const tag of TAGS) {
      const own = addressIn(english, tag);
      const html = read(fileOf(own));
      assert.equal(canonicalOf(html), own, `${tag}/${page} canonical`);
      assert.ok(html.includes(`<meta property="og:url" content="${own}">`), `${tag}/${page} og:url`);
      const links = [...html.matchAll(/<link rel="alternate" hreflang="([^"]+)" href="([^"]+)">/g)].map((m) => [m[1], m[2]]);
      assert.deepEqual(links.map((l) => l[0]), [...TAGS, "x-default"], `${tag}/${page} hreflang`);
      // The same list on every version: if A names B, B names A.
      block ??= JSON.stringify(links);
      assert.equal(JSON.stringify(links), block, `${tag}/${page} lists the same versions`);
      for (const [lang, href] of links) {
        assert.equal(href, lang === "x-default" ? english : addressIn(english, lang));
        assert.ok(existsSync(fileOf(href)), `${href} exists`);
      }
    }
  }
});

test("addresses that existed before answer as before", () => {
  for (const page of PAGES) {
    const source = read(pub, page);
    const withoutGenerated = read(out, page)
      .replace(/\n\n    <nav class="lang-list" id="languages"[\s\S]*?<\/nav>/, "")
      .split("\n")
      .filter((l) => !l.includes('<link rel="alternate" hreflang="'))
      .join("\n");
    assert.equal(withoutGenerated, source, `${page}: only hreflang and #languages are generated`);
  }
  // Everything else, /r, the 404 page and the privacy pages among it, is
  // copied as it is.
  const files = readdirSync(pub, { recursive: true, withFileTypes: true }).filter((e) => e.isFile()).map((e) => join(e.parentPath, e.name).slice(pub.length + 1));
  assert.ok(files.includes(join("r", "index.html")) && files.includes("404.html") && files.includes(join("privacy", "tr.html")));
  for (const same of files) {
    if (PAGES.includes(same.split("\\").join("/")) || same === "sitemap.xml") continue;
    assert.ok(readFileSync(join(out, same)).equals(readFileSync(join(pub, same))), `${same} is copied as it is`);
  }
});

test("every addressed page has a header jump link and a 35-language list that works without JavaScript", () => {
  const nameOf = Object.fromEntries(buildMod.LANGUAGES || []);
  for (const page of PAGES) {
    const english = canonicalOf(read(pub, page));
    const u = new URL(english);
    for (const tag of TAGS) {
      const d = dict(tag);
      const html = read(fileOf(addressIn(english, tag)));
      assert.ok(
        html.includes(`<a class="lang-jump" href="#languages" data-i18n="lang.label">${esc(d["lang.label"])}</a>`),
        `${tag}/${page} header jump link`,
      );
      const navMatch = /<nav class="lang-list" id="languages" aria-labelledby="languages-title">\n      <h2 id="languages-title">([^<]+)<\/h2>\n      <ul>\n([\s\S]*?)\n      <\/ul>\n    <\/nav>\n  <\/main>/.exec(html);
      assert.ok(navMatch, `${tag}/${page} #languages nav before </main>`);
      assert.equal(navMatch[1], esc(d["lang.label"]), `${tag}/${page} #languages heading`);
      const items = [...navMatch[2].matchAll(/<li><a href="([^"]+)" lang="([^"]+)" hreflang="([^"]+)"( dir="(rtl|ltr)")?( aria-current="page")?>([^<]+)<\/a><\/li>/g)];
      assert.equal(items.length, 35, `${tag}/${page} lists 35 languages`);
      assert.deepEqual(items.map((m) => m[2]), TAGS, `${tag}/${page} language order`);
      for (const m of items) {
        const [, href, ltag, hlang, , dirAttr, curAttr, label] = m;
        assert.equal(hlang, ltag);
        assert.equal(label, esc(nameOf[ltag]));
        const wantHref = u.hostname === "download.totaldrop.app"
          ? addressIn(english, ltag)
          : (ltag === "en" ? u.pathname : `/${ltag}${u.pathname}`);
        assert.equal(href, wantHref, `${tag}/${page} -> ${ltag} href`);
        assert.ok(existsSync(fileOf(addressIn(english, ltag))), `${wantHref} file exists`);
        const wantDir = isRtl(ltag) === isRtl(tag) ? undefined : (isRtl(ltag) ? "rtl" : "ltr");
        assert.equal(dirAttr, wantDir, `${tag}/${page} -> ${ltag} dir`);
        assert.equal(Boolean(curAttr), ltag === tag, `${tag}/${page} -> ${ltag} aria-current`);
      }
    }
  }
  // Unit check: when a draft language lacks lang.label, the fallback heading is marked as English.
  const minimal = '<!doctype html>\n<html lang="ar" dir="rtl">\n<head><link rel="canonical" href="https://totaldrop.app/"></head>\n<body>\n  <main id="main">\n    <p>x</p>\n  </main>\n</body>\n</html>\n';
  assert.equal(typeof buildMod.withLanguages, "function");
  const withNav = buildMod.withLanguages(minimal, "https://totaldrop.app/", "ar", {}, { "lang.label": "Language" });
  assert.ok(withNav.includes('<h2 id="languages-title" lang="en" dir="ltr">Language</h2>'));
});

test("/i18n/* carries X-Robots-Tag: noindex in _headers", () => {
  const headers = read(out, "_headers");
  assert.match(headers, /(?:^|\n)\/i18n\/\*\n(?:  [^\n]+\n)*  X-Robots-Tag: noindex(?:\n|$)/);
});

test("the build refuses paths outside site/, tracked paths, symlinks, or non-empty unbuilt folders", () => {
  const bad = [
    "",
    "   ",
    parse(pub).root,
    tmpdir(),
    join(homedir(), "Documents"),
    join(siteDir, ".."),
    siteDir,
    pub,
    join(pub, "i18n"),
    join(pub, "..foo"),
    join(siteDir, "tools"),
    join(siteDir, "tools", "..foo"),
    join(siteDir, "functions"),
    join(siteDir, "privacy"),
    join(siteDir, "team"),
    join(siteDir, "README.md"),
    join(siteDir, "wrangler.toml"),
    join(siteDir, ".wrangler"),
  ];
  for (const p of bad) assert.throws(() => build(p), /refusing to write/, String(p));

  const scratch = mkdtempSync(join(siteDir, ".test-guard-"));
  try {
    // Non-empty unbuilt folder is preserved.
    const nonempty = join(scratch, "nonempty");
    mkdirSync(nonempty);
    writeFileSync(join(nonempty, "keep.txt"), "do not delete");
    assert.throws(() => build(nonempty), /non-empty folder has no previous site build/);
    assert.equal(readFileSync(join(nonempty, "keep.txt"), "utf8"), "do not delete");

    // Unprotected regular file (and path beneath it) is refused as not a folder.
    const plainFile = join(scratch, "plain.txt");
    writeFileSync(plainFile, "stay");
    assert.throws(() => build(plainFile), /not a folder/);
    assert.throws(() => build(join(plainFile, "sub")), /not a folder/);
    assert.equal(readFileSync(plainFile, "utf8"), "stay");

    // Symlink to outside site/, symlink to tracked folder, and broken symlink.
    const outsideTarget = mkdtempSync(join(tmpdir(), "td-outside-"));
    try {
      writeFileSync(join(outsideTarget, "outside.txt"), "safe");
      const linkOutside = join(scratch, "link-outside");
      const linkTools = join(scratch, "link-tools");
      const linkBroken = join(scratch, "link-broken");
      symlinkSync(outsideTarget, linkOutside);
      symlinkSync(join(siteDir, "tools"), linkTools);
      symlinkSync(join(scratch, "does-not-exist"), linkBroken);

      for (const p of [linkOutside, join(linkOutside, "sub"), linkTools, join(linkTools, "sub"), linkBroken, join(linkBroken, "sub")]) {
        assert.throws(() => build(p), /refusing to write/, String(p));
      }
      assert.ok(lstatSync(linkOutside).isSymbolicLink());
      assert.ok(lstatSync(linkTools).isSymbolicLink());
      assert.ok(lstatSync(linkBroken).isSymbolicLink());
      assert.equal(readFileSync(join(outsideTarget, "outside.txt"), "utf8"), "safe");
    } finally {
      rmSync(outsideTarget, { recursive: true, force: true });
    }

    // A ..* folder directly under site/ (not overlapping tracked files) is allowed, and rebuilding succeeds.
    const dotdotOut = join(scratch, "..preview");
    assert.equal(build(dotdotOut).written, 35 * PAGES.length);
    assert.equal(build(dotdotOut).written, 35 * PAGES.length);
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }

  // CLI flag validation (--out, --out=, and stray arguments).
  const script = fileURLToPath(new URL("./build.mjs", import.meta.url));
  for (const cliArgs of [["--out"], ["--out", "--help"], ["--out="], ["--out=/"], [parse(pub).root], ["--unknown"], ["--out", out, "--out", out]]) {
    const res = spawnSync(process.execPath, [script, ...cliArgs], { encoding: "utf8" });
    assert.notEqual(res.status, 0, `expected failure for ${JSON.stringify(cliArgs)}`);
    assert.match(res.stderr, /refusing to write/);
  }

  assert.ok(existsSync(join(pub, "index.html")));
});

test("links between the pages follow the language; files and other sites do not", () => {
  const html = read(out, "tr", "index.html");
  assert.ok(html.includes('href="/tr/roadmap/"'));
  assert.ok(html.includes('href="https://download.totaldrop.app/tr/"'));
  assert.ok(html.includes('href="/privacy/tr"'));
  assert.ok(html.includes('href="#why"'));
  assert.ok(!/href="(https:\/\/totaldrop\.app)?\/(roadmap|donate|team|brand)\/"/.test(html));
  assert.ok(html.includes('href="https://github.com/totaldrop-app/totaldrop'));
  assert.match(html, /href="\/styles\.css\?v=[0-9a-f]{10}"/);
  const brand = read(out, "ja", "brand", "index.html");
  assert.ok(brand.includes('href="/brand/files/'), "downloads stay where they are");
});

test("a draft language says so in the page; a reviewed one does not", () => {
  assert.deepEqual([...REVIEWED], ["en", "tr"]);
  assert.ok(!read(out, "tr", "index.html").includes('class="lang-note"'));
  const de = read(out, "de", "index.html");
  assert.ok(de.includes(`<main id="main">\n    <p class="lang-note" role="note">${esc(dict("de")["lang.draft"])}</p>`));
});

test("the copy button's answer is in the page's language", () => {
  assert.ok(read(out, "tr", "download", "index.html").includes(`data-copied="${dict("tr")["dl.copied"]}"`));
  assert.ok(!read(out, "tr", "index.html").includes("data-copied="));
});

test("text is escaped, and a missing sentence stays English and says so", () => {
  const page = '<!doctype html>\n<html lang="en">\n<head>\n  <title>T</title>\n  <meta name="description" content="D">\n  <link rel="canonical" href="https://totaldrop.app/roadmap/">\n  <meta property="og:title" content="T">\n  <meta property="og:description" content="D">\n  <meta property="og:url" content="https://totaldrop.app/roadmap/">\n</head>\n<body data-page="roadmap">\n  <main id="main">\n    <h1 data-i18n="a">One</h1>\n    <p data-i18n="b">Two</p>\n    <a href="/" aria-label="Home" data-i18n-label="c">x</a>\n  </main>\n</body>\n</html>\n';
  const html = withAlternates(translate(page, "ar", { "meta.roadmap.title": 'A "b" <c>', "meta.roadmap.desc": "d & e", a: "<script>1</script> & 2", b: "", c: 'say "hi"' }), "https://totaldrop.app/roadmap/");
  assert.ok(html.includes('<html lang="ar" dir="rtl">'));
  assert.ok(html.includes("<title>A \"b\" &lt;c&gt;</title>"));
  assert.ok(html.includes('<meta property="og:title" content="A &quot;b&quot; &lt;c&gt;">'));
  assert.ok(html.includes('<meta name="description" content="d &amp; e">'));
  assert.ok(html.includes('<h1 data-i18n="a">&lt;script&gt;1&lt;/script&gt; &amp; 2</h1>'));
  assert.ok(html.includes('<p data-i18n="b" lang="en" dir="ltr">Two</p>'));
  assert.ok(html.includes('<a href="/ar/" aria-label="say &quot;hi&quot;" data-i18n-label="c">x</a>'));
  assert.ok(html.includes('<link rel="canonical" href="https://totaldrop.app/ar/roadmap/">'));
  assert.ok(html.includes('role="note" lang="en" dir="ltr">This translation is a draft'));
  assert.equal(translate(page, "en", {}), page);
});

test("the sitemap lists every address once", () => {
  const locs = [...read(out, "sitemap.xml").matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => m[1]);
  assert.equal(new Set(locs).size, locs.length);
  for (const page of PAGES) for (const tag of TAGS) assert.ok(locs.includes(addressIn(canonicalOf(read(pub, page)), tag)));
  assert.equal(locs.length, [...read(pub, "sitemap.xml").matchAll(/<loc>/g)].length + 34 * PAGES.length);
});

// The address rules, as the browser loads them.
const box = { Intl };
vm.runInNewContext(read(pub, "lang.js"), box);
const { locate, address, redirect, ADDRESSED } = box.TDLang;

test("the pages the build writes are the pages the script treats as having addresses", () => {
  const names = PAGES.map((page) => (/data-page="([^"]+)"/.exec(read(pub, page)) || [])[1] || "home");
  assert.deepEqual(names.sort(), [...ADDRESSED].sort());
  for (const other of ["r/index.html", "404.html"]) assert.ok(!ADDRESSED.includes(/data-page="([^"]+)"/.exec(read(pub, other))[1]), other);
});

test("an address names its language; English keeps the address it had", () => {
  assert.deepEqual({ ...locate("/tr/roadmap/") }, { tag: "tr", rest: "/roadmap/" });
  assert.deepEqual({ ...locate("/zh-Hans/") }, { tag: "zh-Hans", rest: "/" });
  assert.deepEqual({ ...locate("/roadmap/") }, { tag: "en", rest: "/roadmap/" });
  assert.deepEqual({ ...locate("/") }, { tag: "en", rest: "/" });
  // Not languages: /en/ has no pages, /r is the receive link, /tr alone is no folder.
  for (const p of ["/en/", "/r", "/r/", "/tr", "/xx/roadmap/", "/TR/"]) assert.equal(locate(p).tag, "en", p);
  assert.equal(address("tr", "/roadmap/"), "/tr/roadmap/");
  assert.equal(address("en", "/roadmap/"), "/roadmap/");
  assert.equal(address("xx", "/"), "/");
  for (const tag of TAGS) assert.deepEqual({ ...locate(address(tag, "/team/")) }, { tag, rest: "/team/" });
  // No part ever starts with "//", which as an address is another site.
  for (const p of ["//evil.example/", "/tr//evil.example/", "/\\evil.example/", "///x"]) {
    const here = locate(p);
    for (const tag of ["en", "tr"]) assert.match(address(tag, here.rest), /^\/(?![/\\])/, `${p} → ${address(tag, here.rest)}`);
  }
});

test("only an English address moves, and only for a visitor who did not choose English", () => {
  assert.equal(redirect("/", null, ["tr-TR", "en"]), "/tr/");
  assert.equal(redirect("/roadmap/", null, ["ar"]), "/ar/roadmap/");
  assert.equal(redirect("/", null, ["zh-HK"]), "/zh-Hant/");
  assert.equal(redirect("/", "de", ["tr"]), "/de/", "the saved choice wins over the browser");
  assert.equal(redirect("/", "en", ["tr"]), null, "English was chosen");
  assert.equal(redirect("/", null, ["en-US"]), null, "what a crawler sends");
  assert.equal(redirect("/", null, []), null);
  assert.equal(redirect("/", "nonsense", ["xx"]), null);
  assert.equal(redirect("/tr/", null, ["de"]), null, "a translated address never moves");
  assert.equal(redirect("/tr/", "de", ["de"]), null);
  // Nothing in the address can send the visitor to another site.
  for (const p of ["//evil.example/", "/\\evil.example/", "https://evil.example/"]) {
    const to = redirect(p, null, ["tr"]);
    assert.ok(to === null || to.startsWith("/tr/"), `${p} → ${to}`);
  }
});

// The middleware, with the published folder as its assets.
function ask(url, headers = {}) {
  const calls = [];
  const env = { ASSETS: { fetch: async (req) => {
    const path = new URL(req.url).pathname;
    calls.push(path);
    if (!existsSync(join(out, path, "index.html"))) return new Response("not found", { status: 404 });
    // As a real asset server does for a browser that already has the page.
    if (req.headers.get("if-none-match") === '"same"') return new Response(null, { status: 304 });
    return new Response(read(out, path, "index.html"));
  } } };
  return onRequest({ request: new Request(url, { headers }), env, next: async () => new Response("next") }).then(async (res) => ({ res, body: await res.text(), calls }));
}

test("download.totaldrop.app/<tag>/ is the download page in that language", async () => {
  const tr = await ask("https://download.totaldrop.app/tr/");
  assert.equal(tr.body, read(out, "tr", "download", "index.html"));
  assert.equal((await ask("https://download.totaldrop.app/zh-Hant/")).body, read(out, "zh-Hant", "download", "index.html"));
  assert.equal((await ask("https://download.totaldrop.app/")).body, read(out, "download", "index.html"));
  // Everything else on either host name is answered as before.
  for (const url of ["https://download.totaldrop.app/xx/", "https://download.totaldrop.app/r/", "https://download.totaldrop.app/tr/roadmap/", "https://download.totaldrop.app/styles.css", "https://totaldrop.app/tr/", "https://totaldrop.app/"]) {
    assert.equal((await ask(url)).body, "next", url);
  }
  // A browser asking again must not be handed the home page instead.
  const again = await ask("https://download.totaldrop.app/tr/", { "if-none-match": '"same"' });
  assert.equal(again.res.status, 304);
  assert.equal(again.body, "");
  const www = await ask("https://www.totaldrop.app/tr/?a=1");
  assert.equal(www.res.status, 301);
  assert.equal(www.res.headers.get("location"), "https://totaldrop.app/tr/?a=1");
});

test("middleware redirects /en/ and non-canonical language tags with 301 without open redirects", async () => {
  assert.deepEqual(middlewareMod.TAGS, TAGS);
  const subpages = PAGES.filter((p) => p !== "index.html").map((p) => p.replace(/\/index\.html$/, "")).sort();
  assert.deepEqual([...(middlewareMod.SUBPAGES || [])].sort(), subpages);

  const redirects = [
    // download.totaldrop.app (#554 item 2 and #513 PR 2)
    ["https://download.totaldrop.app/en/", "https://download.totaldrop.app/"],
    ["https://download.totaldrop.app/en", "https://download.totaldrop.app/"],
    ["https://download.totaldrop.app/en/index.html", "https://download.totaldrop.app/"],
    ["https://download.totaldrop.app/EN/?a=1", "https://download.totaldrop.app/?a=1"],
    ["https://download.totaldrop.app/pt-br/", "https://download.totaldrop.app/pt-BR/"],
    ["https://download.totaldrop.app/PT-BR", "https://download.totaldrop.app/pt-BR/"],
    ["https://download.totaldrop.app/zh-hans/?x=1", "https://download.totaldrop.app/zh-Hans/?x=1"],
    ["https://download.totaldrop.app/zh-hant", "https://download.totaldrop.app/zh-Hant/"],
    ["https://download.totaldrop.app/TR/", "https://download.totaldrop.app/tr/"],
    ["https://download.totaldrop.app/tr?k=2", "https://download.totaldrop.app/tr/?k=2"],
    // totaldrop.app
    ["https://totaldrop.app/en", "https://totaldrop.app/"],
    ["https://totaldrop.app/en/", "https://totaldrop.app/"],
    ["https://totaldrop.app/en/index.html", "https://totaldrop.app/"],
    ["https://totaldrop.app/EN/?a=1", "https://totaldrop.app/?a=1"],
    ["https://totaldrop.app/en/roadmap", "https://totaldrop.app/roadmap/"],
    ["https://totaldrop.app/en/roadmap/", "https://totaldrop.app/roadmap/"],
    ["https://totaldrop.app/en/roadmap/index.html", "https://totaldrop.app/roadmap/"],
    ["https://totaldrop.app/EN/donate/?s=1", "https://totaldrop.app/donate/?s=1"],
    ["https://totaldrop.app/en/download/", "https://totaldrop.app/download/"],
    ["https://totaldrop.app/en/team/", "https://totaldrop.app/team/"],
    ["https://totaldrop.app/en/brand/", "https://totaldrop.app/brand/"],
    ["https://totaldrop.app/pt-br", "https://totaldrop.app/pt-BR/"],
    ["https://totaldrop.app/pt-br/", "https://totaldrop.app/pt-BR/"],
    ["https://totaldrop.app/PT-BR/donate", "https://totaldrop.app/pt-BR/donate/"],
    ["https://totaldrop.app/zh-hans/", "https://totaldrop.app/zh-Hans/"],
    ["https://totaldrop.app/zh-hant/team?q=1", "https://totaldrop.app/zh-Hant/team/?q=1"],
    ["https://totaldrop.app/TR/brand/", "https://totaldrop.app/tr/brand/"],
    ["https://totaldrop.app/tr", "https://totaldrop.app/tr/"],
    ["https://totaldrop.app/tr/roadmap", "https://totaldrop.app/tr/roadmap/"],
    ["https://totaldrop.app/tr/index.html", "https://totaldrop.app/tr/"],
    ["https://www.totaldrop.app/en/roadmap?a=1", "https://totaldrop.app/roadmap/?a=1"],
  ];
  for (const [from, to] of redirects) {
    const { res } = await ask(from);
    assert.equal(res.status, 301, `${from} status`);
    assert.equal(res.headers.get("location"), to, `${from} -> ${to}`);
  }

  // Unknown suffixes and protocol-relative lookalikes must fall through, never redirect.
  const passthrough = [
    "https://totaldrop.app/en/nope",
    "https://totaldrop.app/en/nope/",
    "https://totaldrop.app/en/r",
    "https://totaldrop.app/en/privacy",
    "https://totaldrop.app/pt-br/nope",
    "https://totaldrop.app/tr/nope",
    "https://totaldrop.app//evil.example/",
    "https://totaldrop.app//en/",
    "https://totaldrop.app/en//roadmap/",
    "https://download.totaldrop.app//evil.example/",
    "https://download.totaldrop.app//en/",
    "https://download.totaldrop.app/en/roadmap/",
  ];
  for (const url of passthrough) {
    const { res, body } = await ask(url);
    assert.equal(res.status, 200, `${url} falls through`);
    assert.equal(body, "next", `${url} body`);
  }
});

// app.js itself, with just enough of a browser around it to see where it
// sends the visitor and what it asks the server for.
function open(page, pathname, { hostname, languages = ["en-US"], saved = null, langJs = true, dataRedirect = false, search = "?a=1", hash = "", storageThrows = false, hrefs = [], langLinks = [] } = {}) {
  const calls = [];
  const makeAnchor = (spec) => {
    const init = typeof spec === "string" ? { href: spec } : spec;
    let cur = init.href;
    const attrs = { ...(init.hreflang ? { hreflang: init.hreflang } : {}), ...(init.lang ? { lang: init.lang } : {}) };
    return {
      getAttribute: (k) => (k === "href" ? cur : (attrs[k] ?? null)),
      hasAttribute: (k) => k === "href" || Object.hasOwn(attrs, k),
      setAttribute: (k, v) => { if (k === "href") cur = v; else attrs[k] = v; },
      removeAttribute: (k) => { delete attrs[k]; },
      addEventListener(type, fn) { this["on" + type] = fn; },
      get href() { return cur; },
    };
  };
  const anchors = hrefs.map(makeAnchor);
  const languageAnchors = langLinks.map(makeAnchor);
  const allAnchors = [...anchors, ...languageAnchors];
  const el = () => ({ dataset: {}, hidden: true, value: "", append() {}, prepend() {}, setAttribute() {}, removeAttribute() {}, hasAttribute: () => false, getAttribute: () => "", addEventListener(type, fn) { this["on" + type] = fn; } });
  const picker = el();
  const jump = { ...el(), hidden: false };
  let stored = saved;
  const localStorage = storageThrows
    ? { getItem() { throw new Error("storage blocked"); }, setItem() { throw new Error("storage blocked"); } }
    : { getItem: () => stored, setItem: (k, v) => { stored = v; calls.push(`save:${v}`); } };
  const host = hostname || (page === "download" && !pathname.endsWith("/download/") ? "download.totaldrop.app" : "totaldrop.app");
  const w = {
    Intl, URL, decodeURIComponent,
    history: { replaceState: (_s, _t, u) => calls.push("replaceState:" + u) },
    document: {
      currentScript: dataRedirect ? { hasAttribute: (k) => k === "data-redirect" } : null,
      body: { dataset: page === "home" ? {} : { page } },
      documentElement: {},
      title: "",
      querySelectorAll: (sel) => {
        if (sel === "a[href]") return allAnchors;
        if (sel === "#languages a[hreflang]") return languageAnchors;
        if (sel === 'a[href="/privacy"], a[href="https://totaldrop.app/privacy"]') {
          return allAnchors.filter((a) => a.href === "/privacy" || a.href === "https://totaldrop.app/privacy");
        }
        return [];
      },
      querySelector: (sel) => (sel === ".lang-jump" ? jump : null),
      getElementById: (id) => (id === "lang-select" ? picker : el()),
      createElement: el,
    },
    location: { hostname: host, origin: "https://" + host, pathname, search, hash, replace: (u) => calls.push("replace:" + u), assign: (u) => calls.push("assign:" + u), reload() {} },
    navigator: { languages, userAgent: "" },
    localStorage,
    fetch: async (u) => { calls.push("fetch:" + u); return { ok: true, json: async () => ({}) }; },
    matchMedia: () => ({ matches: false }), addEventListener() {},
  };
  w.window = w;
  vm.createContext(w);
  if (langJs) vm.runInContext(read(pub, "lang.js"), w);
  w.document.currentScript = null;
  vm.runInContext(read(pub, "app.js"), w);
  return { calls, picker, jump, anchors, languageAnchors };
}

test("app.js sends a visitor on an English address to their language, with the part after #", () => {
  assert.deepEqual(open("home", "/", { languages: ["tr-TR"], hash: "#why" }).calls, ["replace:/tr/?a=1#why"]);
  assert.deepEqual(open("download", "/", { languages: ["ar"] }).calls, ["replace:/ar/?a=1"]);
  assert.deepEqual(open("roadmap", "/roadmap/", { languages: ["tr"], saved: "en" }).calls, [], "English was chosen");
  assert.deepEqual(open("home", "/", { languages: ["en-US"] }).calls, [], "what a crawler sends");
  assert.deepEqual(open("home", "/tr/", { languages: ["de"] }).calls, [], "the address decides");
  assert.deepEqual(open("home", "/", { languages: ["tr"], langJs: false }).calls, [], "without lang.js the page stays as it was sent");
});

test("the picker goes to the same page in the chosen language and remembers the choice", () => {
  const { calls, picker } = open("donate", "/tr/donate/", { languages: ["tr"], hash: "#x" });
  assert.equal(picker.value, "tr");
  picker.value = "ja";
  picker.onchange();
  assert.deepEqual(calls, ["save:ja", "assign:/ja/donate/?a=1#x"]);
  const english = open("donate", "/tr/donate/", { languages: ["tr"] });
  english.picker.value = "en";
  english.picker.onchange();
  assert.deepEqual(english.calls, ["save:en", "assign:/donate/?a=1"]);
  assert.equal(open("home", "/tr/", { langJs: false }).picker.hidden, true, "no list, no picker");
});

test("with localStorage throwing, a visitor can still pick English and stay on English pages", () => {
  const onTr = open("home", "/tr/", { languages: ["tr-TR"], search: "", storageThrows: true });
  onTr.picker.value = "en";
  onTr.picker.onchange();
  assert.deepEqual(onTr.calls, ["assign:/?lang=en"]);

  const onEn = open("home", "/", {
    languages: ["tr-TR"],
    search: "?lang=en",
    storageThrows: true,
    hrefs: ["/roadmap/", "https://download.totaldrop.app/", "/donate/#ways", "#why", "/privacy", "https://github.com/totaldrop-app/totaldrop"],
  });
  assert.deepEqual(onEn.calls, [], "stays on English instead of bouncing back to /tr/");
  assert.deepEqual(
    onEn.anchors.map((a) => a.href),
    ["/roadmap/?lang=en", "https://download.totaldrop.app/?lang=en", "/donate/?lang=en#ways", "#why", "/privacy", "https://github.com/totaldrop-app/totaldrop"],
  );

  // Picking another language drops the ?lang=en query marker.
  onEn.picker.value = "de";
  onEn.picker.onchange();
  assert.deepEqual(onEn.calls, ["assign:/de/"]);
});

test("/r and the 404 page keep their address and are translated in place", () => {
  assert.deepEqual(open("receive", "/r", { languages: ["tr-TR"], hash: "#7-harbor-rocket" }).calls, ["fetch:/i18n/tr.json"]);
  assert.deepEqual(open("notfound", "/tr/nope", { languages: ["tr-TR"] }).calls, ["fetch:/i18n/tr.json"]);
  assert.deepEqual(open("notfound", "/ja/nope", { languages: ["tr-TR"], saved: "tr" }).calls, ["fetch:/i18n/ja.json"], "404 under /<tag>/ uses the address language");
  assert.deepEqual(open("notfound", "/nope", { languages: ["tr-TR"] }).calls, ["fetch:/i18n/tr.json"]);
  assert.deepEqual(open("receive", "/r", { languages: ["en-US"] }).calls, []);
});

test("every public page declares the social preview image and omits cancelled browser-receive cards", () => {
  const repoRoot = join(siteDir, "..");
  const canonical = readFileSync(join(repoRoot, "docs", "assets", "social-preview.png"));
  const published = readFileSync(join(out, "social-preview.png"));
  assert.ok(published.equals(canonical), "site/public/social-preview.png matches docs/assets/social-preview.png");
  for (const tag of TAGS) {
    for (const page of PAGES) {
      const html = read(out, tag === "en" ? page : join(tag, page));
      assert.ok(html.includes('<meta property="og:image" content="https://totaldrop.app/social-preview.png">'), `${tag}/${page} og:image`);
      assert.ok(html.includes('<meta property="og:image:type" content="image/png">'), `${tag}/${page} og:image:type`);
      assert.ok(html.includes('<meta name="twitter:card" content="summary_large_image">'), `${tag}/${page} twitter:card`);
      assert.ok(!html.includes('property="og:image:alt"'), `${tag}/${page} has no untranslated og:image:alt`);
    }
    const dl = read(out, tag === "en" ? "download/index.html" : join(tag, "download/index.html"));
    assert.ok(!dl.includes('data-platform="web"') && !dl.includes('data-i18n="dl.web"'), `${tag}/download omits cancelled web card`);
    const rm = read(out, tag === "en" ? "roadmap/index.html" : join(tag, "roadmap/index.html"));
    assert.ok(!rm.includes('data-i18n="rm.x2.t"'), `${tag}/roadmap omits cancelled browser-receive item`);
  }
});

test("header jump link hides only when the picker is ready, and clicking #languages saves the choice", () => {
  const ready = open("home", "/tr/", {
    languages: ["tr-TR"],
    search: "",
    langLinks: [
      { href: "/", hreflang: "en", lang: "en" },
      { href: "/de/", hreflang: "de", lang: "de" },
    ],
  });
  assert.equal(ready.picker.hidden, false);
  assert.equal(ready.jump.hidden, true);

  const noLangJs = open("home", "/tr/", { langJs: false });
  assert.equal(noLangJs.picker.hidden, true);
  assert.equal(noLangJs.jump.hidden, false, "fallback jump link stays visible if lang.js fails");

  // Modified click does not hijack browser new-tab behaviour.
  let prevented = false;
  ready.languageAnchors[0].onclick({ button: 0, metaKey: true, preventDefault() { prevented = true; } });
  assert.equal(prevented, false);
  assert.deepEqual(ready.calls, []);

  // Normal click on English from /tr/ saves "en" so landing on / does not bounce back.
  ready.languageAnchors[0].onclick({ button: 0, preventDefault() { prevented = true; } });
  assert.equal(prevented, false);
  assert.deepEqual(ready.calls, ["save:en"]);

  // With blocked storage, clicking English in #languages appends ?lang=en.
  const noStorage = open("home", "/tr/", {
    languages: ["tr-TR"],
    search: "",
    storageThrows: true,
    langLinks: [{ href: "/", hreflang: "en", lang: "en" }],
  });
  let blockedPrevented = false;
  noStorage.languageAnchors[0].onclick({ button: 0, preventDefault() { blockedPrevented = true; } });
  assert.equal(blockedPrevented, true);
  assert.deepEqual(noStorage.calls, ["assign:/?lang=en"]);
});

test("chosen language is carried between totaldrop.app and download.totaldrop.app and cleaned on arrival", () => {
  // On totaldrop.app with saved "en": cross-host download link carries ?lang=en; same-host links stay clean.
  const onHomeEn = open("home", "/", {
    hostname: "totaldrop.app",
    languages: ["tr-TR"],
    saved: "en",
    search: "",
    hrefs: ["/roadmap/", "https://totaldrop.app/team/", "https://download.totaldrop.app/", "#why", "/privacy"],
    langLinks: [{ href: "/tr/", hreflang: "tr", lang: "tr" }],
  });
  assert.deepEqual(onHomeEn.calls, []);
  assert.deepEqual(
    onHomeEn.anchors.map((a) => a.href),
    ["/roadmap/", "https://totaldrop.app/team/", "https://download.totaldrop.app/?lang=en", "#why", "/privacy"],
  );
  assert.equal(onHomeEn.languageAnchors[0].href, "/tr/", "#languages links keep their own target language");

  // On totaldrop.app/de/ with saved "de": cross-host download link carries /de/?lang=de.
  const onHomeDe = open("home", "/de/", {
    hostname: "totaldrop.app",
    languages: ["tr-TR"],
    saved: "de",
    search: "",
    hrefs: ["/de/roadmap/", "https://download.totaldrop.app/de/"],
  });
  assert.deepEqual(onHomeDe.anchors.map((a) => a.href), ["/de/roadmap/", "https://download.totaldrop.app/de/?lang=de"]);

  // Arriving on download.totaldrop.app/?lang=en&utm=1#top saves "en" (overriding stale "de"), strips lang, and rewrites links back to totaldrop.app.
  const onDownloadEn = open("download", "/", {
    hostname: "download.totaldrop.app",
    languages: ["tr-TR"],
    saved: "de",
    search: "?lang=en&utm=1",
    hash: "#top",
    hrefs: ["https://totaldrop.app/", "https://totaldrop.app/#why", "https://totaldrop.app/roadmap/", "https://download.totaldrop.app/", "https://totaldrop.app/privacy"],
    langLinks: [{ href: "https://download.totaldrop.app/tr/", hreflang: "tr", lang: "tr" }],
  });
  assert.deepEqual(onDownloadEn.calls, ["save:en", "replaceState:/?utm=1#top"]);
  assert.deepEqual(
    onDownloadEn.anchors.map((a) => a.href),
    ["https://totaldrop.app/?lang=en", "https://totaldrop.app/?lang=en#why", "https://totaldrop.app/roadmap/?lang=en", "https://download.totaldrop.app/", "https://totaldrop.app/privacy"],
  );
  assert.equal(onDownloadEn.languageAnchors[0].href, "https://download.totaldrop.app/tr/");

  // Invalid ?lang= value is ignored.
  const badQuery = open("home", "/", { hostname: "totaldrop.app", languages: ["en-US"], search: "?lang=xx" });
  assert.deepEqual(badQuery.calls, []);
});

test("changing #lang-select on /r re-syncs cross-host and storage-blocked links", async () => {
  // 1. Empty localStorage on /r: picking "de" then "en" updates the Download link each time.
  const firstVisit = open("receive", "/r", {
    hostname: "totaldrop.app",
    languages: ["en-US"],
    search: "",
    hash: "#7-harbor-rocket",
    hrefs: ["https://download.totaldrop.app/", "https://totaldrop.app/roadmap/", "https://totaldrop.app/privacy"],
  });
  assert.deepEqual(
    firstVisit.anchors.map((a) => a.href),
    ["https://download.totaldrop.app/", "https://totaldrop.app/roadmap/", "https://totaldrop.app/privacy"],
  );
  firstVisit.picker.value = "de";
  await firstVisit.picker.onchange();
  assert.deepEqual(
    firstVisit.anchors.map((a) => a.href),
    ["https://download.totaldrop.app/de/?lang=de", "https://totaldrop.app/de/roadmap/", "https://totaldrop.app/privacy/de"],
  );
  firstVisit.picker.value = "en";
  await firstVisit.picker.onchange();
  assert.deepEqual(
    firstVisit.anchors.map((a) => a.href),
    ["https://download.totaldrop.app/?lang=en", "https://totaldrop.app/roadmap/", "https://totaldrop.app/privacy"],
  );

  // 2. Returning visit with saved "fr": switching #lang-select to "de" replaces stale /fr/?lang=fr.
  const returning = open("receive", "/r", {
    hostname: "totaldrop.app",
    languages: ["en-US"],
    saved: "fr",
    search: "",
    hrefs: ["https://download.totaldrop.app/", "https://totaldrop.app/roadmap/"],
  });
  assert.deepEqual(
    returning.anchors.map((a) => a.href),
    ["https://download.totaldrop.app/fr/?lang=fr", "https://totaldrop.app/fr/roadmap/"],
  );
  returning.picker.value = "de";
  await returning.picker.onchange();
  assert.deepEqual(
    returning.anchors.map((a) => a.href),
    ["https://download.totaldrop.app/de/?lang=de", "https://totaldrop.app/de/roadmap/"],
  );

  // 3. Storage-blocked on /r: picking "tr" carries ?lang=tr on both cross-host and same-host addressed links.
  const blocked = open("receive", "/r", {
    hostname: "totaldrop.app",
    languages: ["en-US"],
    storageThrows: true,
    search: "",
    hrefs: ["https://download.totaldrop.app/", "https://totaldrop.app/roadmap/"],
  });
  blocked.picker.value = "tr";
  await blocked.picker.onchange();
  assert.deepEqual(
    blocked.anchors.map((a) => a.href),
    ["https://download.totaldrop.app/tr/?lang=tr", "https://totaldrop.app/tr/roadmap/?lang=tr"],
  );
});

test("English addressed pages run lang.js with data-redirect in <head> after styles.css; translated and single-address pages keep it deferred (#554)", () => {
  const assertNoInlineScripts = (html, label) => {
    const lower = html.replace(/[A-Z]/g, (c) => c.toLowerCase());
    let pos = 0;
    while ((pos = lower.indexOf("<script", pos)) !== -1) {
      const next = lower[pos + 7];
      if (next && !/[\s/>]/.test(next)) {
        pos += 7;
        continue;
      }
      const gt = html.indexOf(">", pos);
      assert.notEqual(gt, -1, `${label}: unclosed <script> tag`);
      const attrs = html.slice(pos + 7, gt);
      const close = lower.indexOf("</script", gt + 1);
      assert.notEqual(close, -1, `${label}: missing </script> closing tag`);
      const body = html.slice(gt + 1, close);
      assert.match(attrs, /\bsrc="\/[^"]+"/, `${label}: external script only (CSP script-src 'self')`);
      assert.equal(body, "", `${label}: no inline script body`);
      pos = close + 8;
    }
  };
  const splitParts = (html, label) => {
    const head = /<head>([\s\S]*?)<\/head>/.exec(html);
    const body = /<body\b[^>]*>([\s\S]*?)<\/body>/.exec(html);
    assert.ok(head && body, `${label}: has <head> and <body>`);
    return { head: head[1], body: body[1] };
  };

  let expectedLangSrc = null;
  let expectedAppSrc = null;
  for (const page of PAGES) {
    for (const [label, html] of [[`public/${page}`, read(pub, page)], [`dist/${page}`, read(out, page)]]) {
      assertNoInlineScripts(html, label);
      const { head, body } = splitParts(html, label);
      const cssMatch = /<link rel="stylesheet" href="\/styles\.css\?v=[0-9a-f]{10}">/.exec(head);
      const langMatch = /<script src="(\/lang\.js\?v=[0-9a-f]{10})" data-redirect><\/script>/.exec(head);
      assert.ok(cssMatch, `${label}: stylesheet in <head>`);
      assert.ok(langMatch, `${label}: non-deferred lang.js with data-redirect in <head>`);
      assert.ok(langMatch.index > cssMatch.index, `${label}: lang.js comes after styles.css in <head>`);
      assert.equal((head.match(/<script\b/gi) || []).length, 1, `${label}: only lang.js script in <head>`);
      assert.ok(!/\bdefer\b/.test(langMatch[0]), `${label}: head lang.js is not deferred`);
      assert.ok(!body.includes("lang.js"), `${label}: no duplicate lang.js in <body>`);
      assert.ok(!body.includes("data-redirect"), `${label}: no data-redirect in <body>`);
      const appMatch = /<script src="(\/app\.js\?v=[0-9a-f]{10})" defer><\/script>/.exec(body);
      assert.ok(appMatch, `${label}: deferred app.js in <body>`);
      expectedLangSrc ??= langMatch[1];
      expectedAppSrc ??= appMatch[1];
      assert.equal(langMatch[1], expectedLangSrc, `${label}: consistent lang.js version`);
      assert.equal(appMatch[1], expectedAppSrc, `${label}: consistent app.js version`);
    }
  }

  for (const tag of others) {
    for (const page of PAGES) {
      const label = `dist/${tag}/${page}`;
      const html = read(out, tag, page);
      assertNoInlineScripts(html, label);
      const { head, body } = splitParts(html, label);
      assert.ok(!head.includes("<script"), `${label}: no head-blocking scripts on translated page`);
      assert.ok(!html.includes("data-redirect"), `${label}: no data-redirect on translated page`);
      assert.ok(
        body.includes(`<script src="${expectedLangSrc}" defer></script>\n  <script src="${expectedAppSrc}" defer></script>`),
        `${label}: deferred lang.js immediately before app.js in <body>`,
      );
    }
  }

  for (const single of ["r/index.html", "404.html"]) {
    for (const [label, html] of [[`public/${single}`, read(pub, single)], [`dist/${single}`, read(out, single)]]) {
      assertNoInlineScripts(html, label);
      const { head, body } = splitParts(html, label);
      assert.ok(!head.includes("<script"), `${label}: no scripts in <head>`);
      assert.ok(!html.includes("data-redirect"), `${label}: no data-redirect`);
      assert.ok(
        body.includes(`<script src="${expectedLangSrc}" defer></script>\n  <script src="${expectedAppSrc}" defer></script>`),
        `${label}: deferred lang.js before app.js in <body>`,
      );
    }
  }
});

test("lang.js with data-redirect redirects early and preserves app.js ?lang= cleanup (#554)", () => {
  function runLang({
    pathname = "/",
    hostname = "totaldrop.app",
    languages = ["en-US"],
    language,
    saved = null,
    search = "",
    hash = "",
    storageThrows = false,
    storageReadOnly = false,
    storageGetterThrows = false,
    dataRedirect = true,
  } = {}) {
    const calls = [];
    let stored = saved;
    const w = {
      Intl, URL,
      history: { replaceState: (_s, _t, u) => calls.push("replaceState:" + u) },
      document: {
        currentScript: { hasAttribute: (k) => k === "data-redirect" && dataRedirect },
      },
      location: {
        hostname,
        origin: "https://" + hostname,
        pathname,
        search,
        hash,
        replace: (u) => calls.push("replace:" + u),
      },
      navigator: language !== undefined ? { languages, language } : { languages },
    };
    if (storageGetterThrows) {
      Object.defineProperty(w, "localStorage", {
        get() { throw new Error("SecurityError: storage blocked"); },
      });
    } else if (storageThrows) {
      w.localStorage = {
        getItem() { throw new Error("storage blocked"); },
        setItem() { throw new Error("storage blocked"); },
      };
    } else if (storageReadOnly) {
      w.localStorage = {
        getItem: () => stored,
        setItem() { throw new Error("QuotaExceededError"); },
      };
    } else {
      w.localStorage = {
        getItem: () => stored,
        setItem: (_k, v) => { stored = v; calls.push(`save:${v}`); },
      };
    }
    w.window = w;
    vm.createContext(w);
    vm.runInContext(read(pub, "lang.js"), w);
    return { calls, stored: () => stored, TDLang: w.TDLang };
  }

  // Redirects non-English visitors on /, /download/, and download.totaldrop.app/, preserving query and hash.
  assert.deepEqual(
    runLang({ pathname: "/", languages: ["tr-TR"], search: "?foo=1", hash: "#why" }).calls,
    ["replace:/tr/?foo=1#why"],
  );
  assert.deepEqual(
    runLang({ pathname: "/download/", languages: ["tr-TR"], search: "?foo=1", hash: "#linux" }).calls,
    ["replace:/tr/download/?foo=1#linux"],
  );
  assert.deepEqual(
    runLang({ hostname: "download.totaldrop.app", pathname: "/", languages: ["tr-TR"], search: "?foo=1", hash: "#android" }).calls,
    ["replace:/tr/?foo=1#android"],
  );
  // Falls back to navigator.language when navigator.languages is empty.
  assert.deepEqual(
    runLang({ pathname: "/", languages: [], language: "tr-TR" }).calls,
    ["replace:/tr/"],
  );

  // Saved "en" stays on English without redirecting or touching history.
  assert.deepEqual(runLang({ pathname: "/", languages: ["tr-TR"], saved: "en" }).calls, []);

  // ?lang=en stays on English and NEVER calls history.replaceState in lang.js
  // (whether storage works, throws, getter throws, or is read-only).
  const enWritable = runLang({ pathname: "/", languages: ["tr-TR"], saved: "de", search: "?lang=en&foo=1", hash: "#top" });
  assert.deepEqual(enWritable.calls, ["save:en"]);
  assert.equal(enWritable.stored(), "en");
  assert.deepEqual(
    runLang({ pathname: "/", languages: ["tr-TR"], search: "?lang=en&foo=1", hash: "#top", storageThrows: true }).calls,
    [],
  );
  assert.deepEqual(
    runLang({ pathname: "/", languages: ["tr-TR"], search: "?lang=en", storageGetterThrows: true }).calls,
    [],
  );
  assert.deepEqual(
    runLang({ pathname: "/", languages: ["tr-TR"], saved: "de", search: "?lang=en", storageReadOnly: true }).calls,
    [],
    "valid ?lang=en wins over stale saved language even if setItem throws",
  );

  // ?lang=de saves choice and strips lang from query on redirect when storage works; keeps it when storage throws.
  const deWithOther = runLang({ pathname: "/", languages: ["en-US"], search: "?lang=de&foo=1", hash: "#sec" });
  assert.deepEqual(deWithOther.calls, ["save:de", "replace:/de/?foo=1#sec"]);
  const deOnly = runLang({ pathname: "/roadmap/", languages: ["en-US"], search: "?lang=de" });
  assert.deepEqual(deOnly.calls, ["save:de", "replace:/de/roadmap/"]);
  const deBlocked = runLang({ pathname: "/", languages: ["en-US"], search: "?lang=de&foo=1", hash: "#sec", storageThrows: true });
  assert.deepEqual(deBlocked.calls, ["replace:/de/?lang=de&foo=1#sec"]);

  // Invalid ?lang=xx is ignored (not saved, does not strip query).
  assert.deepEqual(runLang({ pathname: "/", languages: ["tr-TR"], search: "?lang=xx" }).calls, ["replace:/tr/?lang=xx"]);
  assert.deepEqual(runLang({ pathname: "/", languages: ["en-US"], search: "?lang=xx" }).calls, []);

  // Translated addresses never redirect even with data-redirect, and without data-redirect lang.js never auto-redirects.
  assert.deepEqual(runLang({ pathname: "/tr/", languages: ["de"], saved: "de", dataRedirect: true }).calls, []);
  const noAttr = runLang({ pathname: "/", languages: ["tr-TR"], dataRedirect: false });
  assert.deepEqual(noAttr.calls, []);
  // Direct helper call still works deterministically for tests.
  const directCalls = [];
  const directTarget = noAttr.TDLang.earlyRedirect({
    URL,
    location: { pathname: "/", search: "?foo=1", hash: "#why", replace: (u) => directCalls.push(u) },
    navigator: { languages: ["tr-TR"] },
    localStorage: { getItem: () => null, setItem() {} },
  });
  assert.equal(directTarget, "/tr/?foo=1#why");
  assert.deepEqual(directCalls, ["/tr/?foo=1#why"]);

  // Full two-script lifecycle: lang.js (data-redirect) followed by deferred app.js on /?lang=en.
  const bothWritable = open("home", "/", {
    hostname: "totaldrop.app",
    languages: ["tr-TR"],
    saved: "de",
    dataRedirect: true,
    search: "?lang=en&utm=1",
    hash: "#top",
    hrefs: ["/roadmap/", "https://download.totaldrop.app/"],
  });
  assert.ok(bothWritable.calls.includes("replaceState:/?utm=1#top"), "app.js strips ?lang=en after lang.js stayed");
  assert.ok(!bothWritable.calls.some((c) => c.startsWith("replace:")), "neither script redirects");
  assert.deepEqual(bothWritable.anchors.map((a) => a.href), ["/roadmap/", "https://download.totaldrop.app/?lang=en"]);

  const bothBlocked = open("home", "/", {
    hostname: "totaldrop.app",
    languages: ["tr-TR"],
    dataRedirect: true,
    search: "?lang=en",
    storageThrows: true,
    hrefs: ["/roadmap/", "https://download.totaldrop.app/"],
  });
  assert.deepEqual(bothBlocked.calls, [], "no redirect and no replaceState when storage throws");
  assert.deepEqual(
    bothBlocked.anchors.map((a) => a.href),
    ["/roadmap/?lang=en", "https://download.totaldrop.app/?lang=en"],
    "app.js still rewrites links via syncChosenLinks when storage throws",
  );
});
