// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Reads text out of the site's HTML the way a browser would, for
// check-i18n.mjs. Tested by: node --test site/tools/test-html-text.mjs

// The named entities the pages may use.
export const NAMED = { amp: "&", lt: "<", gt: ">", quot: '"', apos: "'", nbsp: " ", rsquo: "’", lsquo: "‘", rdquo: "”", ldquo: "“", mdash: "—", ndash: "–", hellip: "…", middot: "·", copy: "©" };

// Decodes every character reference in one pass, so the text a reference
// produces is never read again: "&#x26;lt;" is "&lt;", not "<". An
// unknown name or a number that is no Unicode code point is passed to
// onBad and left as it is.
export function decodeEntities(s, onBad = () => {}) {
  return s.replace(/&(?:#x([0-9a-f]+)|#([0-9]+)|([a-z][a-z0-9]*));/gi, (all, hex, dec, name) => {
    if (name !== undefined) {
      if (Object.hasOwn(NAMED, name)) return NAMED[name];
      onBad(all);
      return all;
    }
    const cp = hex !== undefined ? parseInt(hex, 16) : Number(dec);
    if (!(cp <= 0x10ffff)) { onBad(all); return all; }
    return String.fromCodePoint(cp);
  });
}

// Removes comments as a browser reads them: each runs from "<!--" to the
// next "-->" ("<!-->" and "<!--->" are empty comments). Text on either
// side of a comment is joined, so the result must not form a new "<!--";
// that, and a comment that never ends, go to onBad.
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
      const close = html.indexOf("-->", open + 4);
      if (close < 0) { onBad("a comment that never ends"); return out; }
      end = close + 3;
    }
    at = end;
  }
  out += html.slice(at);
  if (out.includes("<!--")) onBad("text around a comment that joins into another \"<!--\"");
  return out;
}
