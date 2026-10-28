// The site's formatter for the ICU message subset TotalDrop uses (ADR 0008):
// {name}, {n, plural, =0/=1/=2/zero/one/two/few/many/other{…}} and
// {x, select, …, other{…}}. It follows Flutter's gen_l10n and the Rust
// crate totaldrop-l10n exactly; l10n/tests/messages.json holds the shared
// test vectors (site/tools/check-i18n.mjs runs them in CI).
(function (root) {
  "use strict";

  const MAX_LEN = 16 * 1024;
  const MAX_DEPTH = 4;
  const SLOTS = { "=0": "zero", zero: "zero", "=1": "one", one: "one", "=2": "two", two: "two", few: "few", many: "many", other: "other" };
  const RTL = ["ar", "fa", "he", "ur", "ps", "yi"];

  function parse(src) {
    if (src.length > MAX_LEN) throw new Error("too long");
    let pos = 0;
    const peek = () => src[pos];
    const spaces = () => { while (pos < src.length && /[ \t\n\r\f]/.test(src[pos])) pos++; };
    const ident = () => {
      const start = pos;
      while (pos < src.length && /[A-Za-z0-9_]/.test(src[pos])) pos++;
      const s = src.slice(start, pos);
      if (!s || /^[0-9]/.test(s)) throw new Error("syntax at " + start);
      return s;
    };
    const eat = (c) => { if (src[pos] !== c) throw new Error("syntax at " + pos); pos++; };

    function message(depth) {
      if (depth > MAX_DEPTH) throw new Error("too deep");
      const parts = [];
      let text = "";
      while (pos < src.length) {
        const c = peek();
        if (c === "{") {
          if (text) { parts.push({ t: text }); text = ""; }
          pos++;
          parts.push(placeholder(depth));
        } else if (c === "}") {
          break;
        } else {
          text += c; pos++;
        }
      }
      if (text) parts.push({ t: text });
      return parts;
    }

    function placeholder(depth) {
      spaces();
      const arg = ident();
      spaces();
      if (peek() === "}") { pos++; return { a: arg }; }
      eat(",");
      spaces();
      const kind = ident();
      spaces();
      eat(",");
      if (kind !== "plural" && kind !== "select") throw new Error("syntax at " + pos);
      const cases = {};
      for (;;) {
        spaces();
        if (pos >= src.length) throw new Error("syntax at " + pos);
        if (peek() === "}") { pos++; break; }
        const start = pos;
        if (peek() === "=") { pos++; while (pos < src.length && /[0-9]/.test(src[pos])) pos++; } else ident();
        let key = src.slice(start, pos);
        if (key === "=") throw new Error("syntax at " + start);
        if (kind === "plural") {
          if (!(key in SLOTS)) throw new Error("syntax at " + start);
          key = SLOTS[key];
        }
        if (Object.prototype.hasOwnProperty.call(cases, key)) throw new Error("duplicate case");
        spaces();
        eat("{");
        cases[key] = message(depth + 1);
        eat("}");
      }
      if (!Object.keys(cases).length) throw new Error("syntax at " + pos);
      if (!("other" in cases)) throw new Error("no other");
      return kind === "plural" ? { p: arg, cases } : { s: arg, cases };
    }

    const m = message(0);
    if (pos !== src.length) throw new Error("syntax at " + pos);
    return m;
  }

  // Formats `src` in `locale` with `args` (numbers are counts).
  function format(src, args, locale) {
    const parts = parse(src);
    const rules = new Intl.PluralRules(locale);
    const numbers = new Intl.NumberFormat(locale);
    const rtl = RTL.includes(String(locale).split("-")[0]);
    args = args || {};
    const write = (list) => list.map((part) => {
      if ("t" in part) return part.t;
      if ("a" in part) {
        const v = args[part.a];
        if (typeof v === "number") return numbers.format(v);
        if (typeof v === "string") return rtl ? "\u2068" + v + "\u2069" : v;
        return "{" + part.a + "}";
      }
      if ("p" in part) {
        const v = args[part.p];
        const n = typeof v === "number" ? v : (typeof v === "string" && /^[0-9]+$/.test(v) ? Number(v) : null);
        let chosen = null;
        if (n !== null) {
          if (n === 0 && part.cases.zero) chosen = part.cases.zero;
          else if (n === 1 && part.cases.one) chosen = part.cases.one;
          else if (n === 2 && part.cases.two) chosen = part.cases.two;
          else chosen = part.cases[rules.select(n)] || null;
        }
        return write(chosen || part.cases.other);
      }
      const v = args[part.s];
      const key = v === undefined ? "" : String(v);
      return write(Object.prototype.hasOwnProperty.call(part.cases, key) && key !== "" ? part.cases[key] : part.cases.other);
    }).join("");
    return write(parts);
  }

  root.TDMessage = { parse, format, RTL };
})(typeof window !== "undefined" ? window : globalThis);
