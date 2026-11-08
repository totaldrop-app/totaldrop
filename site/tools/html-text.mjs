// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Reads text out of the site's HTML for check-i18n.mjs. Where a browser
// would repair or remap something, these functions report it instead.
// Tested by: node --test site/tools/test-html-text.mjs

// The named entities the pages may use.
export const NAMED = { amp: "&", lt: "<", gt: ">", quot: '"', apos: "'", nbsp: "\u00a0", rsquo: "\u2019", lsquo: "\u2018", rdquo: "\u201d", ldquo: "\u201c", mdash: "\u2014", ndash: "\u2013", hellip: "\u2026", middot: "\u00b7", copy: "\u00a9" };

// Decodes every character reference in one pass, so the text a reference
// produces is never read again: "&#x26;lt;" is "&lt;", not "<". An
// unknown name is passed to onBad and left as it is. So is a number a
// browser would not show as itself: 0, a surrogate, a C1 control (shown
// as windows-1252) or anything past U+10FFFF.
export function decodeEntities(s, onBad = () => {}) {
  return s.replace(/&(?:#x([0-9a-f]+)|#([0-9]+)|([a-z][a-z0-9]*));/gi, (all, hex, dec, name) => {
    if (name !== undefined) {
      if (Object.hasOwn(NAMED, name)) return NAMED[name];
      onBad(all);
      return all;
    }
    const cp = hex !== undefined ? parseInt(hex, 16) : Number(dec);
    if (!(cp > 0 && cp <= 0x10ffff) || (cp >= 0xd800 && cp <= 0xdfff) || (cp >= 0x80 && cp <= 0x9f)) {
      onBad(all);
      return all;
    }
    return String.fromCodePoint(cp);
  });
}

// Removes comments as a browser reads them: each runs from "<!--" to the
// next "-->" or "--!>" ("<!-->" and "<!--->" are empty comments). Text on
// either side of a comment is joined, so the result must not form a new
// "<!--"; that, and a comment that never ends, go to onBad.
export function stripComments(html, onBad = () => {}) {
  let out = "";
  let at = 0;
  for (;;) {
    const open = html.indexOf("<!--", at);
    if (open < 0) break;
    out += html.slice(at, open);
    let end;
    if (html.startsWith(">", open + 4)) end = open + 5;
    else if (html.startsWith("->", open + 4)) end = open + 6;
    else {
      const ends = [["-->", 3], ["--!>", 4]]
        .map(([mark, len]) => [html.indexOf(mark, open + 4), len])
        .filter(([i]) => i >= 0);
      if (ends.length === 0) { onBad("a comment that never ends"); return out; }
      const [close, len] = ends.reduce((a, b) => (b[0] < a[0] ? b : a));
      end = close + len;
    }
    at = end;
  }
  out += html.slice(at);
  if (out.includes("<!--")) onBad("text around a comment that joins into another \"<!--\"");
  return out;
}
