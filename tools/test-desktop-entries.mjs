// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test tools/test-desktop-entries.mjs

import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  appEntry,
  desktopLocales,
  dolphinMenu,
  escapeList,
  escapeValue,
  escapeXml,
  keywords,
  metainfoBlock,
  nautilusBlock,
  nemoAction,
  readCatalog,
  withBlock,
} from "./desktop-entries.mjs";

const here = dirname(fileURLToPath(import.meta.url));

function catalog(entries) {
  return new Map(Object.entries(entries));
}

const en = {
  desktopGenericName: "File transfer",
  desktopComment: "Send files",
  desktopKeywords: "send;files;",
  desktopSendWith: "Send with TotalDrop",
  sendTitle: "Send files",
  sendClipboard: "Send clipboard",
  receiveTitle: "Receive files",
  metainfoSummary: "Send files",
  metainfoAbout: "TotalDrop sends files.",
  metainfoVerified: "Verified",
  metainfoWords: "Words",
  metainfoTheme: "Theme",
  metainfoOnePackage: "One package at a time.",
};

test("the files in the repository are what write would write", () => {
  execFileSync(process.execPath, [join(here, "desktop-entries.mjs"), "check"], { stdio: "pipe" });
});

test("every language of l10n/app has the menu text", () => {
  const all = readCatalog();
  assert.ok(all.size >= 35);
  for (const [arb, messages] of all) {
    for (const key of ["desktopGenericName", "desktopComment", "desktopKeywords", "desktopSendWith", "sendClipboard"]) {
      assert.equal(typeof messages[key], "string", `${arb}: ${key}`);
    }
  }
});

test("ARB locales map to desktop locales", () => {
  assert.deepEqual(desktopLocales("tr"), ["tr"]);
  assert.deepEqual(desktopLocales("pt_BR"), ["pt_BR"]);
  assert.deepEqual(desktopLocales("zh_Hans"), ["zh_CN", "zh_SG"]);
  assert.deepEqual(desktopLocales("zh_Hant"), ["zh_TW", "zh_HK", "zh_MO"]);
  assert.throws(() => desktopLocales("sr_Latn"));
  assert.throws(() => desktopLocales("x]=y"));
});

test("values are escaped as the Desktop Entry Specification says", () => {
  assert.equal(escapeValue("a\\b"), "a\\\\b");
  assert.equal(escapeValue(" lead"), "\\slead");
  assert.equal(escapeValue("tab\there"), "tab\\there");
  assert.equal(escapeList(["a;b", "c"]), "a\\;b;c;");
});

test("keywords are split, trimmed, deduplicated and extended", () => {
  assert.deepEqual(keywords("send; files;;send;", "x"), [
    "send",
    "files",
    "wormhole",
    "localsend",
    "airdrop",
  ]);
  assert.throws(() => keywords(";;", "x"), /no words/);
});

test("a translation with a line break or edge space is refused", () => {
  for (const bad of ["two\nlines", "line sep", " lead", "trail ", ""]) {
    const c = catalog({ en, tr: { ...en, desktopSendWith: bad } });
    assert.throws(() => dolphinMenu(c), undefined, JSON.stringify(bad));
  }
  // A translation cannot add a key or a group of its own either: `=`, `[`
  // and `]` stay inside the value of its line.
  const c = catalog({ en, tr: { ...en, desktopComment: "x]\n[Desktop Entry]" } });
  assert.throws(() => appEntry(c));
});

test("a translation that the desktop would fall back to is left out", () => {
  const c = catalog({
    en,
    da: { ...en, desktopComment: "Send filer" },
    pt: { ...en, desktopComment: "Envie ficheiros" },
    pt_PT: { ...en, desktopComment: "Envie ficheiros" },
    pt_BR: { ...en, desktopComment: "Envie arquivos" },
    zh: { ...en, desktopComment: "发送文件" },
    zh_Hans: { ...en, desktopComment: "发送文件" },
    zh_Hant: { ...en, desktopComment: "傳送檔案" },
  });
  const lines = appEntry(c).split("\n").filter((l) => l.startsWith("Comment"));
  assert.deepEqual(lines, [
    "Comment=Send files",
    "Comment[da]=Send filer",
    "Comment[pt]=Envie ficheiros",
    "Comment[pt_BR]=Envie arquivos",
    "Comment[zh]=发送文件",
    "Comment[zh_HK]=傳送檔案",
    "Comment[zh_MO]=傳送檔案",
    "Comment[zh_TW]=傳送檔案",
  ]);
});

test("the menus open the app with URIs, not with paths in a shell", () => {
  const c = catalog({ en });
  for (const file of [appEntry(c), dolphinMenu(c), nemoAction(c)]) {
    const exec = file.split("\n").filter((l) => l.startsWith("Exec="));
    assert.ok(exec.length > 0);
    for (const l of exec) assert.match(l, /^Exec=totaldrop-desktop( %U| --screen=(send|clipboard|receive))$/);
  }
  assert.match(dolphinMenu(c), /^X-KDE-Protocols=file$/m);
  assert.match(dolphinMenu(c), /^X-KDE-Priority=TopLevel$/m);
  // Nemo pastes the URIs into a command line that GLib splits like a
  // shell: GLib leaves ' $ & ( ) raw in file URIs, so each one is quoted,
  // and only local files are offered.
  assert.match(nemoAction(c), /^Quote=double$/m);
  assert.match(nemoAction(c), /^UriScheme=file$/m);
  assert.match(appEntry(c), /^MimeType=x-scheme-handler\/totaldrop;$/m);
});

test("the Nautilus block is a Python dict of every desktop locale", () => {
  const c = catalog({
    en,
    tr: { ...en, desktopSendWith: "TotalDrop ile gönder" },
    zh_Hant: { ...en, desktopSendWith: "用 \"TotalDrop\" 傳送" },
  });
  const block = nautilusBlock(c);
  assert.match(block, /^LABEL = "Send with TotalDrop"$/m);
  assert.match(block, /^ {4}"tr": "TotalDrop ile gönder",$/m);
  assert.match(block, /^ {4}"zh_TW": "用 \\"TotalDrop\\" 傳送",$/m);
  const src = `a\n# BEGIN GENERATED by tools/desktop-entries.mjs\nold\n# END GENERATED\nb\n`;
  assert.equal(withBlock(src, "NEW\n"), "a\nNEW\nb\n");
  assert.throws(() => withBlock("no block\n", "x"));
  assert.throws(() => withBlock(src + src, "x"));
});

test("XML text is escaped", () => {
  assert.equal(escapeXml("a & <b> c"), "a &amp; &lt;b&gt; c");
});

test("the metainfo block has each text in English and in xml:lang copies", () => {
  const c = catalog({
    en,
    de: { ...en, metainfoSummary: "Dateien senden", metainfoTheme: "Design & Farben" },
    pt: { ...en, metainfoSummary: "Enviar arquivos" },
    pt_BR: { ...en, metainfoSummary: "Enviar arquivos" },
    zh_Hant: { ...en, metainfoWords: "驗證單字" },
  });
  const block = metainfoBlock(c);
  assert.match(block, /^  <!-- BEGIN GENERATED by tools\/desktop-entries\.mjs -->\n/);
  assert.match(block, /\n  <!-- END GENERATED -->\n$/);
  assert.match(block, /\n  <summary>Send files<\/summary>\n/);
  assert.match(block, /\n  <summary xml:lang="de">Dateien senden<\/summary>\n/);
  assert.match(block, /\n      <li xml:lang="de">Design &amp; Farben<\/li>\n/);
  // pt_BR says what pt says: software centres fall back to pt.
  assert.match(block, /xml:lang="pt">Enviar arquivos/);
  assert.doesNotMatch(block, /xml:lang="pt_BR"/);
  // No script subtag: Traditional Chinese goes by its regions.
  assert.match(block, /<li xml:lang="zh_TW">驗證單字<\/li>/);
  // A language that differs in one element gets the whole description,
  // the rest in what it falls back to: a centre does not mix languages.
  for (const loc of ["de", "zh_TW", "zh_HK", "zh_MO"]) {
    assert.match(block, new RegExp(`<p xml:lang="${loc}">TotalDrop sends files\\.</p>`));
    assert.match(block, new RegExp(`<li xml:lang="${loc}">Verified</li>`));
  }
  assert.match(block, /<li xml:lang="zh_TW">Theme<\/li>/);
  // pt differs only in its summary: no description of its own.
  assert.doesNotMatch(block, /<p xml:lang="pt">/);
  // The list is inside the description.
  assert.ok(block.indexOf("<ul>") > block.indexOf("<description>"));
  assert.ok(block.indexOf("</ul>") < block.indexOf("</description>"));
  // The note on packages closes the description, after the list (#351).
  assert.match(block, /<\/ul>\n    <p>One package at a time\.<\/p>\n/);
  assert.match(block, /<p xml:lang="de">One package at a time\.<\/p>/);
  assert.ok(block.indexOf("<p>One package") < block.indexOf("</description>"));
});

test("every language of l10n/app has the software-centre text", () => {
  const keys = [
    "metainfoSummary",
    "metainfoAbout",
    "metainfoVerified",
    "metainfoWords",
    "metainfoTheme",
    "metainfoOnePackage",
  ];
  for (const [arb, messages] of readCatalog()) {
    for (const key of keys) assert.ok(messages[key], `${arb}: ${key}`);
    // The desktop names stay as they are.
    for (const name of ["Plasma", "Hyprland", "Omarchy", "GNOME"]) {
      assert.ok(messages.metainfoTheme.includes(name), `${arb}: metainfoTheme names ${name}`);
    }
    // The package names stay as they are.
    for (const name of ["Flatpak", ".pkg", ".deb", ".rpm"]) {
      assert.ok(messages.metainfoOnePackage.includes(name), `${arb}: metainfoOnePackage names ${name}`);
    }
  }
});
