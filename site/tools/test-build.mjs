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
import { mkdtempSync, readFileSync, readdirSync, existsSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import vm from "node:vm";
import { build, pub, PAGES, TAGS, REVIEWED, translate, withAlternates, addressIn, canonicalOf, isRtl } from "./build.mjs";
import { onRequest } from "../functions/_middleware.js";

const out = mkdtempSync(join(tmpdir(), "totaldrop-site-"));
test.after(() => rmSync(out, { recursive: true, force: true }));
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
    const lines = read(out, page).split("\n").filter((l) => !l.includes('<link rel="alternate" hreflang="'));
    assert.equal(lines.join("\n"), source, `${page}: only the hreflang lines are new`);
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

test("the build refuses to empty its own source", () => {
  for (const bad of [pub, join(pub, ".."), join(pub, "i18n")]) assert.throws(() => build(bad), /refusing to write/);
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

// app.js itself, with just enough of a browser around it to see where it
// sends the visitor and what it asks the server for.
function open(page, pathname, { languages = ["en-US"], saved = null, langJs = true, hash = "" } = {}) {
  const calls = [];
  const el = () => ({ dataset: {}, hidden: true, value: "", append() {}, prepend() {}, setAttribute() {}, removeAttribute() {}, getAttribute: () => "", addEventListener(type, fn) { this["on" + type] = fn; } });
  const picker = el();
  const w = {
    Intl, URL, decodeURIComponent,
    document: { body: { dataset: page === "home" ? {} : { page } }, documentElement: {}, title: "", querySelectorAll: () => [], querySelector: () => null, getElementById: (id) => (id === "lang-select" ? picker : el()), createElement: el },
    location: { pathname, search: "?a=1", hash, replace: (u) => calls.push("replace:" + u), assign: (u) => calls.push("assign:" + u), reload() {} },
    navigator: { languages, userAgent: "" },
    localStorage: { getItem: () => saved, setItem: (k, v) => calls.push(`save:${v}`) },
    fetch: async (u) => { calls.push("fetch:" + u); return { ok: true, json: async () => ({}) }; },
    matchMedia: () => ({ matches: false }), addEventListener() {},
  };
  w.window = w;
  vm.createContext(w);
  if (langJs) vm.runInContext(read(pub, "lang.js"), w);
  vm.runInContext(read(pub, "app.js"), w);
  return { calls, picker };
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

test("/r and the 404 page keep their address and are translated in place", () => {
  assert.deepEqual(open("receive", "/r", { languages: ["tr-TR"], hash: "#7-harbor-rocket" }).calls, ["fetch:/i18n/tr.json"]);
  assert.deepEqual(open("notfound", "/tr/nope", { languages: ["tr-TR"] }).calls, ["fetch:/i18n/tr.json"]);
  assert.deepEqual(open("receive", "/r", { languages: ["en-US"] }).calls, []);
});
