// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node tools/check-state-machines.mjs
//
// Checks that every ChangeNotifier in the Android app (app/lib/) keeps its
// state machine explicit and guards async work across `await` points
// (docs/surec.md, "Örtük durum makinesi"; #342, #399):
//
// 1. Stage fields and getters (`stage`, `state`, `phase`, `status`, and their
//    private `_` forms) carry a declared `enum` type defined in `app/lib/`,
//    never `String`, `bool`, `int`, `dynamic`, `Object`, or an untyped `var`.
// 2. Every instance `async` method that suspends at `await` and mutates state
//    or calls `notifyListeners()` (directly or through a mutating helper such
//    as `setText` or `_fail`) on any path after `await` — including `catch`
//    and `finally` — snapshots `_generation` or `_resets` into a `final`
//    local before the first `await` and checks that snapshot (or `_disposed`)
//    after `await` before acting, unless listed in `ALLOWLIST` with a reason.
// 3. Every `ChangeNotifier` that declares `_generation` or `_resets` (or
//    `_disposed`) invalidates them inside `dispose()` (`_generation++`,
//    `_resets++`, `_disposed = true`), so navigating away while an `await` is
//    in flight never mutates state or calls `notifyListeners()` after disposal.

import { existsSync, readdirSync, readFileSync, realpathSync } from "node:fs";
import { dirname, join, posix, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

/** Methods on a `ChangeNotifier` that legitimately `await` and call
 * `notifyListeners()` without a transfer generation counter. Every entry must
 * carry a non-empty `reason`, and an entry that is no longer needed fails the
 * check so this table never goes stale. */
export const ALLOWLIST = {
  "SettingsRepository.setServer": {
    reason:
      "SettingsRepository is an app-lifetime SharedPreferencesWithCache store that is never reset or disposed; notifyListeners() carries no stale payload and getters read the live cache.",
  },
  "SettingsRepository.setPrivacyMode": {
    reason:
      "SettingsRepository is an app-lifetime SharedPreferencesWithCache store that is never reset or disposed; notifyListeners() carries no stale payload and getters read the live cache.",
  },
  "SettingsRepository.resetServer": {
    reason:
      "SettingsRepository is an app-lifetime SharedPreferencesWithCache store that is never reset or disposed; notifyListeners() carries no stale payload and getters read the live cache.",
  },
};

const STAGE_MEMBER_NAMES = new Set([
  "stage",
  "_stage",
  "state",
  "_state",
  "phase",
  "_phase",
  "status",
  "_status",
]);

const COUNTER_FIELDS = ["_generation", "_resets"];

const COLLECTION_MUTATORS = new Set([
  "add",
  "addAll",
  "clear",
  "fillRange",
  "insert",
  "insertAll",
  "putIfAbsent",
  "remove",
  "removeAt",
  "removeLast",
  "removeRange",
  "removeWhere",
  "replaceRange",
  "retainWhere",
  "setAll",
  "setRange",
  "shuffle",
  "sort",
  "update",
  "updateAll",
]);

const ASSIGNMENT_OPS = new Set([
  "=",
  "+=",
  "-=",
  "*=",
  "/=",
  "~/=",
  "%=",
  "&=",
  "|=",
  "^=",
  "<<=",
  ">>=",
  ">>>=",
  "??=",
]);

const MEMBER_MODIFIERS = new Set([
  "abstract",
  "covariant",
  "const",
  "external",
  "factory",
  "final",
  "late",
  "static",
]);

const COLLECTION_TYPES = new Set([
  "Iterable",
  "List",
  "LinkedHashMap",
  "LinkedHashSet",
  "Map",
  "Queue",
  "Set",
]);

const CLASS_MODIFIERS = new Set([
  "abstract",
  "base",
  "final",
  "interface",
  "mixin",
  "sealed",
]);

/** Single-pass injective escaping of backslashes, ASCII/C1 control codes, and
 * Unicode Bidi/format controls before printing source fragments to logs. */
export function sanitizeForLog(text, maxLength = 160) {
  const input = String(text);
  let out = "";
  let count = 0;
  for (const ch of input) {
    if (count >= maxLength) {
      out += "...";
      break;
    }
    count++;
    const cp = ch.codePointAt(0);
    if (cp === 0x5c) {
      out += "\\\\";
    } else if (isControlOrBidi(cp)) {
      out += `\\u{${cp.toString(16).toUpperCase().padStart(4, "0")}}`;
    } else {
      out += ch;
    }
  }
  return out;
}

function isControlOrBidi(cp) {
  return (
    cp <= 0x1f ||
    (cp >= 0x7f && cp <= 0x9f) ||
    cp === 0x061c ||
    (cp >= 0x200b && cp <= 0x200f) ||
    (cp >= 0x2028 && cp <= 0x202f) ||
    (cp >= 0x2060 && cp <= 0x206f) ||
    (cp >= 0xd800 && cp <= 0xdfff) ||
    cp === 0xfeff ||
    (cp >= 0xfff9 && cp <= 0xfffb)
  );
}

function isIdentStart(code) {
  return (
    (code >= 65 && code <= 90) ||
    (code >= 97 && code <= 122) ||
    code === 95 ||
    code === 36
  );
}

function isIdentPart(code) {
  return isIdentStart(code) || (code >= 48 && code <= 57);
}

function isDigit(code) {
  return code >= 48 && code <= 57;
}

const THREE_CHAR_OPS = new Set([">>>=", ">>>", "<<=", ">>=", "~/=", "??=", "?.."]);
const TWO_CHAR_OPS = new Set([
  "++",
  "--",
  "==",
  "!=",
  "<=",
  ">=",
  "&&",
  "||",
  "??",
  "?.",
  "..",
  "=>",
  "+=",
  "-=",
  "*=",
  "/=",
  "%=",
  "&=",
  "|=",
  "^=",
  "<<",
  ">>",
  "~/",
]);
const PUNCT_CHARS = new Set(["{", "}", "(", ")", "[", "]", ";", ",", ".", ":", "@", "?"]);

/** Linear-time Dart lexer that strips line and nested block comments, handles
 * raw/multiline strings and `${...}` string interpolations without regex
 * backtracking, and tags every token with its 1-based line number. */
export function tokenizeDart(source) {
  const tokens = [];
  const interpStack = [];
  const len = source.length;
  let i = 0;
  let line = 1;

  function readStringTail(delim, raw, startLine) {
    const dLen = delim.length;
    while (i < len) {
      if (source.startsWith(delim, i)) {
        i += dLen;
        tokens.push({ kind: "string", value: '""', line: startLine });
        return;
      }
      const ch = source.charCodeAt(i);
      if (!raw && ch === 92 /* \ */) {
        i++;
        if (i < len) {
          if (source.charCodeAt(i) === 10) line++;
          i++;
        }
        continue;
      }
      if (!raw && ch === 36 /* $ */ && i + 1 < len && source.charCodeAt(i + 1) === 123 /* { */) {
        tokens.push({ kind: "string", value: '""', line: startLine });
        tokens.push({ kind: "punct", value: "(", line });
        interpStack.push({ braceDepth: 0, delim });
        i += 2;
        return;
      }
      if (ch === 10) line++;
      i++;
    }
    tokens.push({ kind: "string", value: '""', line: startLine });
  }

  while (i < len) {
    const ch = source.charCodeAt(i);

    if (ch === 10 /* \n */) {
      line++;
      i++;
      continue;
    }
    if (ch === 32 || ch === 9 || ch === 13 || ch === 12) {
      i++;
      continue;
    }

    // Line comment `// ...`
    if (ch === 47 /* / */ && i + 1 < len && source.charCodeAt(i + 1) === 47) {
      i += 2;
      while (i < len && source.charCodeAt(i) !== 10) i++;
      continue;
    }

    // Nested block comment `/* ... */`
    if (ch === 47 /* / */ && i + 1 < len && source.charCodeAt(i + 1) === 42 /* * */) {
      i += 2;
      let depth = 1;
      while (i < len && depth > 0) {
        const c = source.charCodeAt(i);
        if (c === 10) {
          line++;
          i++;
        } else if (c === 47 && i + 1 < len && source.charCodeAt(i + 1) === 42) {
          depth++;
          i += 2;
        } else if (c === 42 && i + 1 < len && source.charCodeAt(i + 1) === 47) {
          depth--;
          i += 2;
        } else {
          i++;
        }
      }
      continue;
    }

    // Raw string `r'...'`, `r"..."`, `r'''...'''`, `r"""..."""`
    if (
      ch === 114 /* r */ &&
      i + 1 < len &&
      (source.charCodeAt(i + 1) === 39 || source.charCodeAt(i + 1) === 34)
    ) {
      const q = source[i + 1];
      const startLine = line;
      const triple = q + q + q;
      if (source.startsWith(triple, i + 1)) {
        i += 4;
        readStringTail(triple, true, startLine);
      } else {
        i += 2;
        readStringTail(q, true, startLine);
      }
      continue;
    }

    // String `'...'`, `"..."`, `'''...'''`, `"""..."""`
    if (ch === 39 /* ' */ || ch === 34 /* " */) {
      const q = source[i];
      const startLine = line;
      const triple = q + q + q;
      if (source.startsWith(triple, i)) {
        i += 3;
        readStringTail(triple, false, startLine);
      } else {
        i += 1;
        readStringTail(q, false, startLine);
      }
      continue;
    }

    // Closing `}` of a `${...}` string interpolation
    if (ch === 123 /* { */ && interpStack.length > 0) {
      interpStack[interpStack.length - 1].braceDepth++;
      tokens.push({ kind: "punct", value: "{", line });
      i++;
      continue;
    }
    if (ch === 125 /* } */ && interpStack.length > 0) {
      const top = interpStack[interpStack.length - 1];
      if (top.braceDepth > 0) {
        top.braceDepth--;
        tokens.push({ kind: "punct", value: "}", line });
        i++;
      } else {
        interpStack.pop();
        tokens.push({ kind: "punct", value: ")", line });
        i++;
        readStringTail(top.delim, false, line);
      }
      continue;
    }

    // Identifier
    if (isIdentStart(ch)) {
      const start = i;
      i++;
      while (i < len && isIdentPart(source.charCodeAt(i))) i++;
      tokens.push({ kind: "ident", value: source.slice(start, i), line });
      continue;
    }

    // Number literal
    if (isDigit(ch)) {
      const start = i;
      i++;
      while (i < len && (isIdentPart(source.charCodeAt(i)) || source.charCodeAt(i) === 46)) {
        i++;
      }
      tokens.push({ kind: "number", value: source.slice(start, i), line });
      continue;
    }

    // Multi-character operators
    if (i + 3 <= len) {
      const three = source.slice(i, i + 3);
      if (THREE_CHAR_OPS.has(three)) {
        tokens.push({ kind: "op", value: three, line });
        i += 3;
        continue;
      }
    }
    if (i + 2 <= len) {
      const two = source.slice(i, i + 2);
      if (TWO_CHAR_OPS.has(two)) {
        tokens.push({ kind: "op", value: two, line });
        i += 2;
        continue;
      }
    }

    const one = source[i];
    tokens.push({
      kind: PUNCT_CHARS.has(one) ? "punct" : "op",
      value: one,
      line,
    });
    i++;
  }

  return tokens;
}

/** Collects every top-level `enum <Name>` declared in a token stream. */
export function collectEnumsFromTokens(tokens, out = new Set()) {
  let braceDepth = 0;
  for (let i = 0; i < tokens.length; i++) {
    const t = tokens[i];
    if (t.value === "{") {
      braceDepth++;
    } else if (t.value === "}") {
      if (braceDepth > 0) braceDepth--;
    } else if (
      braceDepth === 0 &&
      t.kind === "ident" &&
      t.value === "enum" &&
      i + 1 < tokens.length &&
      tokens[i + 1].kind === "ident"
    ) {
      out.add(tokens[i + 1].value);
    }
  }
  return out;
}

function findMatchingDelim(tokens, openIndex, openVal, closeVal) {
  let depth = 0;
  for (let i = openIndex; i < tokens.length; i++) {
    if (tokens[i].value === openVal) depth++;
    else if (tokens[i].value === closeVal) {
      depth--;
      if (depth === 0) return i;
    }
  }
  return tokens.length - 1;
}

function skipTypeArgs(tokens, index) {
  if (index >= tokens.length || tokens[index].value !== "<") return index;
  let depth = 0;
  for (let i = index; i < tokens.length; i++) {
    const v = tokens[i].value;
    if (v === "<") depth++;
    else if (v === ">") {
      depth--;
      if (depth === 0) return i + 1;
    } else if (v === ">>") {
      depth -= 2;
      if (depth <= 0) return i + 1;
    } else if (v === ">>>") {
      depth -= 3;
      if (depth <= 0) return i + 1;
    } else if (v === "{" || v === ";" || v === "(") {
      return index;
    }
  }
  return index;
}

/** Checks whether a class header (`extends ... with ... implements ...`)
 * derives from `ChangeNotifier` via `extends` or `with`. */
function headerUsesChangeNotifier(headerTokens) {
  let clause = "";
  let angleDepth = 0;
  for (let i = 0; i < headerTokens.length; i++) {
    const t = headerTokens[i];
    if (t.value === "<") {
      angleDepth++;
      continue;
    }
    if (angleDepth > 0) {
      if (t.value === ">") angleDepth--;
      else if (t.value === ">>") angleDepth = Math.max(0, angleDepth - 2);
      else if (t.value === ">>>") angleDepth = Math.max(0, angleDepth - 3);
      continue;
    }
    if (t.kind === "ident" && (t.value === "extends" || t.value === "with" || t.value === "implements" || t.value === "on")) {
      clause = t.value;
      continue;
    }
    if ((clause === "extends" || clause === "with") && t.kind === "ident" && t.value === "ChangeNotifier") {
      return true;
    }
  }
  return false;
}

/** Finds every `class <Name> extends/with ChangeNotifier` in `tokens`. */
export function findChangeNotifiers(tokens) {
  const classes = [];
  let braceDepth = 0;
  for (let i = 0; i < tokens.length; i++) {
    const t = tokens[i];
    if (t.value === "{") {
      braceDepth++;
      continue;
    }
    if (t.value === "}") {
      if (braceDepth > 0) braceDepth--;
      continue;
    }
    if (braceDepth !== 0 || t.kind !== "ident" || t.value !== "class") continue;
    if (i > 0 && tokens[i - 1].kind === "ident" && !CLASS_MODIFIERS.has(tokens[i - 1].value)) {
      continue;
    }
    if (i + 1 >= tokens.length || tokens[i + 1].kind !== "ident") continue;
    const className = tokens[i + 1].value;
    const classLine = t.line;
    let cursor = skipTypeArgs(tokens, i + 2);
    const headerStart = cursor;
    while (cursor < tokens.length && tokens[cursor].value !== "{" && tokens[cursor].value !== ";") {
      cursor++;
    }
    const headerTokens = tokens.slice(headerStart, cursor);
    if (cursor < tokens.length && tokens[cursor].value === "{") {
      const closeBrace = findMatchingDelim(tokens, cursor, "{", "}");
      if (headerUsesChangeNotifier(headerTokens)) {
        classes.push({
          name: className,
          line: classLine,
          bodyTokens: tokens.slice(cursor + 1, closeBrace),
        });
      }
      i = closeBrace;
    } else {
      if (headerUsesChangeNotifier(headerTokens)) {
        classes.push({ name: className, line: classLine, bodyTokens: [] });
      }
      i = cursor;
    }
  }
  return classes;
}

function skipLeadingAnnotations(tokens, pos) {
  let i = pos;
  while (i < tokens.length && tokens[i].value === "@") {
    i++;
    if (i < tokens.length && tokens[i].kind === "ident") {
      i++;
      while (
        i + 1 < tokens.length &&
        tokens[i].value === "." &&
        tokens[i + 1].kind === "ident"
      ) {
        i += 2;
      }
    }
    i = skipTypeArgs(tokens, i);
    if (i < tokens.length && tokens[i].value === "(") {
      i = findMatchingDelim(tokens, i, "(", ")") + 1;
    }
  }
  return i;
}

function formatDeclaredType(typeTokens) {
  if (typeTokens.length === 0) return "";
  if (typeTokens.length === 1 && typeTokens[0].value === "var") return "";
  return typeTokens.map((t) => t.value).join("");
}

function baseTypeName(declaredType) {
  if (!declaredType) return "";
  let s = declaredType;
  if (s.endsWith("?")) s = s.slice(0, -1);
  const lt = s.indexOf("<");
  if (lt !== -1) s = s.slice(0, lt);
  const dot = s.lastIndexOf(".");
  if (dot !== -1) s = s.slice(dot + 1);
  return s;
}

/** Parses fields, getters, setters, constructors, and methods declared at the
 * top level of a Dart class body. */
function parseClassMembers(className, bodyTokens) {
  const fields = [];
  const getters = [];
  const methods = [];
  const len = bodyTokens.length;
  let i = 0;

  while (i < len) {
    while (i < len && bodyTokens[i].value === ";") i++;
    if (i >= len) break;
    i = skipLeadingAnnotations(bodyTokens, i);
    if (i >= len) break;

    const modifiers = new Set();
    while (i < len && bodyTokens[i].kind === "ident" && MEMBER_MODIFIERS.has(bodyTokens[i].value)) {
      modifiers.add(bodyTokens[i].value);
      i++;
    }
    if (i >= len) break;

    // Scan signature tokens up to `(`, `{`, `=>`, `=`, `,`, or `;` at angleDepth 0.
    const sigStart = i;
    let angleDepth = 0;
    while (i < len) {
      const v = bodyTokens[i].value;
      if (v === "<") {
        const after = skipTypeArgs(bodyTokens, i);
        if (after > i) {
          i = after;
          continue;
        }
      }
      if (angleDepth === 0 && (v === "(" || v === "{" || v === "=>" || v === "=" || v === "," || v === ";")) {
        break;
      }
      i++;
    }
    if (i >= len) break;

    const sigTokens = bodyTokens.slice(sigStart, i);
    const stopVal = bodyTokens[i].value;
    const isStatic = modifiers.has("static") || modifiers.has("factory");

    if (stopVal === "(") {
      const closeParen = findMatchingDelim(bodyTokens, i, "(", ")");
      let cursor = closeParen + 1;

      const isCtor =
        (sigTokens.length === 1 && sigTokens[0].value === className) ||
        (sigTokens.length === 3 &&
          sigTokens[0].value === className &&
          sigTokens[1].value === "." &&
          sigTokens[2].kind === "ident");

      if (isCtor) {
        // Skip optional `: super(...)` / initializer list up to `{` or `;`.
        while (cursor < len && bodyTokens[cursor].value !== "{" && bodyTokens[cursor].value !== ";") {
          if (bodyTokens[cursor].value === "(") {
            cursor = findMatchingDelim(bodyTokens, cursor, "(", ")") + 1;
          } else {
            cursor++;
          }
        }
        if (cursor < len && bodyTokens[cursor].value === "{") {
          cursor = findMatchingDelim(bodyTokens, cursor, "{", "}") + 1;
        } else if (cursor < len && bodyTokens[cursor].value === ";") {
          cursor++;
        }
        i = cursor;
        continue;
      }

      let isAsync = false;
      while (
        cursor < len &&
        bodyTokens[cursor].value !== "{" &&
        bodyTokens[cursor].value !== "=>" &&
        bodyTokens[cursor].value !== ";"
      ) {
        if (bodyTokens[cursor].kind === "ident" && bodyTokens[cursor].value === "async") {
          isAsync = true;
        }
        cursor++;
      }

      const nameTok = sigTokens.length > 0 ? sigTokens[sigTokens.length - 1] : null;
      const isSetter =
        sigTokens.length >= 2 && sigTokens[sigTokens.length - 2].value === "set";

      let bodyKind = "none";
      let methodBody = [];
      if (cursor < len && bodyTokens[cursor].value === "{") {
        const closeBrace = findMatchingDelim(bodyTokens, cursor, "{", "}");
        bodyKind = "block";
        methodBody = bodyTokens.slice(cursor + 1, closeBrace);
        cursor = closeBrace + 1;
      } else if (cursor < len && bodyTokens[cursor].value === "=>") {
        cursor++;
        const exprStart = cursor;
        cursor = skipExpressionUntil(bodyTokens, cursor, new Set([";"]));
        bodyKind = "arrow";
        methodBody = bodyTokens.slice(exprStart, cursor);
        if (cursor < len && bodyTokens[cursor].value === ";") cursor++;
      } else if (cursor < len && bodyTokens[cursor].value === ";") {
        cursor++;
      }

      if (nameTok && nameTok.kind === "ident") {
        methods.push({
          name: nameTok.value,
          line: nameTok.line,
          isStatic,
          isAsync,
          isSetter,
          bodyKind,
          bodyTokens: methodBody,
        });
      }
      i = cursor;
      continue;
    }

    if (stopVal === "{" || stopVal === "=>") {
      // Getter: `[Type] get <name> => ...;` or `[Type] get <name> { ... }`
      const getIdx = sigTokens.findIndex((t) => t.kind === "ident" && t.value === "get");
      if (getIdx !== -1 && getIdx + 1 < sigTokens.length) {
        const nameTok = sigTokens[getIdx + 1];
        const declaredType = formatDeclaredType(sigTokens.slice(0, getIdx));
        getters.push({
          name: nameTok.value,
          line: nameTok.line,
          isStatic,
          declaredType,
        });
      }
      if (stopVal === "{") {
        i = findMatchingDelim(bodyTokens, i, "{", "}") + 1;
      } else {
        i = skipExpressionUntil(bodyTokens, i + 1, new Set([";"]));
        if (i < len && bodyTokens[i].value === ";") i++;
      }
      continue;
    }

    // Field declaration (may declare multiple comma-separated names)
    if (sigTokens.length > 0) {
      const firstFieldTok = sigTokens[sigTokens.length - 1];
      const declaredType = formatDeclaredType(sigTokens.slice(0, -1));
      const baseType = baseTypeName(declaredType);
      const isCollectionByType = COLLECTION_TYPES.has(baseType);
      const initIsCollection = (eqPos) => {
        if (eqPos >= len || bodyTokens[eqPos].value !== "=") return false;
        let p = eqPos + 1;
        if (p < len && bodyTokens[p].value === "const") p++;
        p = skipTypeArgs(bodyTokens, p);
        return p < len && (bodyTokens[p].value === "[" || bodyTokens[p].value === "{");
      };
      if (firstFieldTok.kind === "ident") {
        fields.push({
          name: firstFieldTok.value,
          line: firstFieldTok.line,
          isStatic,
          isFinal: modifiers.has("final") || modifiers.has("const"),
          declaredType,
          isCollection: isCollectionByType || (!declaredType && initIsCollection(i)),
        });
      }
      let cursor = i;
      while (cursor < len) {
        if (bodyTokens[cursor].value === "=") {
          cursor = skipExpressionUntil(bodyTokens, cursor + 1, new Set([",", ";"]));
        }
        if (cursor < len && bodyTokens[cursor].value === ",") {
          cursor++;
          if (cursor < len && bodyTokens[cursor].kind === "ident") {
            const nameTok = bodyTokens[cursor];
            cursor++;
            fields.push({
              name: nameTok.value,
              line: nameTok.line,
              isStatic,
              isFinal: modifiers.has("final") || modifiers.has("const"),
              declaredType,
              isCollection: isCollectionByType || (!declaredType && initIsCollection(cursor)),
            });
          }
          continue;
        }
        if (cursor < len && bodyTokens[cursor].value === ";") {
          cursor++;
        }
        break;
      }
      i = cursor;
    } else {
      i++;
    }
  }

  return { fields, getters, methods };
}

function skipExpressionUntil(tokens, start, stopValues) {
  let paren = 0;
  let bracket = 0;
  let brace = 0;
  let i = start;
  while (i < tokens.length) {
    const v = tokens[i].value;
    if (paren === 0 && bracket === 0 && brace === 0 && stopValues.has(v)) {
      return i;
    }
    if (v === "(") paren++;
    else if (v === ")" && paren > 0) paren--;
    else if (v === "[") bracket++;
    else if (v === "]" && bracket > 0) bracket--;
    else if (v === "{") brace++;
    else if (v === "}" && brace > 0) brace--;
    i++;
  }
  return i;
}

/** Checks whether tokens inside `dispose()` increment `counter` (at outer
 * scope, outside nested closures). */
function bodyIncrementsCounter(bodyTokens, counter) {
  let braceDepth = 0;
  for (let i = 0; i < bodyTokens.length; i++) {
    const v = bodyTokens[i].value;
    if (v === "{") {
      braceDepth++;
      continue;
    }
    if (v === "}") {
      if (braceDepth > 0) braceDepth--;
      continue;
    }
    if (braceDepth > 0) continue;

    // `counter++` or `this.counter++` or `counter += ...`
    if (bodyTokens[i].kind === "ident" && v === counter) {
      const prev = i > 0 ? bodyTokens[i - 1].value : "";
      const prevPrev = i > 1 ? bodyTokens[i - 2].value : "";
      const onThis = prev !== "." || prevPrev === "this";
      if (!onThis) continue;
      const next = i + 1 < bodyTokens.length ? bodyTokens[i + 1].value : "";
      if (next === "++" || next === "+=") return true;
      if (prev === "++") return true;
      if (prev === "." && prevPrev === "this" && i > 2 && bodyTokens[i - 3].value === "++") {
        return true;
      }
    }
  }
  return false;
}

/** Checks whether `dispose()` sets `_disposed = true`. */
function bodySetsDisposedTrue(bodyTokens) {
  let braceDepth = 0;
  for (let i = 0; i < bodyTokens.length; i++) {
    const v = bodyTokens[i].value;
    if (v === "{") {
      braceDepth++;
      continue;
    }
    if (v === "}") {
      if (braceDepth > 0) braceDepth--;
      continue;
    }
    if (braceDepth > 0) continue;
    if (
      bodyTokens[i].kind === "ident" &&
      v === "_disposed" &&
      i + 2 < bodyTokens.length &&
      bodyTokens[i + 1].value === "=" &&
      bodyTokens[i + 2].value === "true"
    ) {
      const prev = i > 0 ? bodyTokens[i - 1].value : "";
      const prevPrev = i > 1 ? bodyTokens[i - 2].value : "";
      if (prev !== "." || prevPrev === "this") return true;
    }
  }
  return false;
}

const CONTROL_PARENS = new Set(["if", "for", "while", "switch", "catch"]);
const CLOSURE_PREFIX_KEYWORDS = new Set([
  "await",
  "case",
  "return",
  "throw",
  "when",
  "yield",
]);

function buildDelimMatchMap(tokens) {
  const match = new Int32Array(tokens.length).fill(-1);
  const parenStack = [];
  const bracketStack = [];
  const braceStack = [];
  for (let i = 0; i < tokens.length; i++) {
    const v = tokens[i].value;
    if (v === "(") parenStack.push(i);
    else if (v === ")" && parenStack.length > 0) {
      const open = parenStack.pop();
      match[open] = i;
    } else if (v === "[") bracketStack.push(i);
    else if (v === "]" && bracketStack.length > 0) {
      const open = bracketStack.pop();
      match[open] = i;
    } else if (v === "{") braceStack.push(i);
    else if (v === "}" && braceStack.length > 0) {
      const open = braceStack.pop();
      match[open] = i;
    }
  }
  return match;
}

function skipClosureModifier(tokens, pos) {
  let p = pos;
  if (p < tokens.length && tokens[p].kind === "ident") {
    if (tokens[p].value === "async") {
      p++;
      if (p < tokens.length && tokens[p].value === "*") p++;
    } else if (
      tokens[p].value === "sync" &&
      p + 1 < tokens.length &&
      tokens[p + 1].value === "*"
    ) {
      p += 2;
    }
  }
  return p;
}

function canPrecedeClosure(prevTok) {
  if (!prevTok) return true;
  if (prevTok.kind === "ident") {
    return CLOSURE_PREFIX_KEYWORDS.has(prevTok.value);
  }
  return (
    prevTok.value !== ")" &&
    prevTok.value !== "]" &&
    prevTok.value !== "." &&
    prevTok.value !== "?." &&
    prevTok.value !== ".." &&
    prevTok.value !== "?.."
  );
}

/** Strips nested closure bodies `(params) [async[*]|sync*] { ... }` and
 * `(params) [async] => expr` in linear time while preserving Dart 3 `switch`
 * expressions (`switch (x) { p => expr }`) and map/set literals (`{k: v}`). */
function stripClosureBodies(tokens) {
  const match = buildDelimMatchMap(tokens);
  const out = [];
  const switchStack = [];
  const len = tokens.length;

  for (let i = 0; i < len; i++) {
    const t = tokens[i];
    const v = t.value;

    const topSwitch = switchStack.length > 0 ? switchStack[switchStack.length - 1] : null;
    if (topSwitch && i === topSwitch.closeBrace) {
      switchStack.pop();
      out.push(t);
      continue;
    }

    // Track `switch (expr) { ... }` expressions so neither their `{ ... }` body
    // nor a record-pattern arm `(a, b) => expr` is mistaken for a closure.
    if (
      t.kind === "ident" &&
      v === "switch" &&
      i + 1 < len &&
      tokens[i + 1].value === "("
    ) {
      const closeParen = match[i + 1];
      if (closeParen !== -1 && closeParen + 1 < len && tokens[closeParen + 1].value === "{") {
        const closeBrace = match[closeParen + 1];
        if (closeBrace !== -1) {
          for (let k = i; k <= closeParen + 1; k++) out.push(tokens[k]);
          switchStack.push({
            closeBrace,
            inArmRhs: false,
            paren: 0,
            bracket: 0,
            brace: 0,
          });
          i = closeParen + 1;
          continue;
        }
      }
    }

    // Check for `[<TypeParams>](params) [async[*]|sync*] { ... } | => expr`
    const openParen = v === "<" ? skipTypeArgs(tokens, i) : i;
    if (
      openParen < len &&
      tokens[openParen].value === "(" &&
      match[openParen] !== -1
    ) {
      const prevTok = i > 0 ? tokens[i - 1] : null;
      const isControl =
        prevTok !== null && prevTok.kind === "ident" && CONTROL_PARENS.has(prevTok.value);
      if (!isControl && canPrecedeClosure(prevTok)) {
        const closeParen = match[openParen];
        const bodyPos = skipClosureModifier(tokens, closeParen + 1);
        if (bodyPos < len && tokens[bodyPos].value === "{" && match[bodyPos] !== -1) {
          out.push(tokens[openParen], tokens[closeParen]);
          i = match[bodyPos];
          continue;
        }
        if (bodyPos < len && tokens[bodyPos].value === "=>") {
          const isSwitchPatternArrow =
            topSwitch !== null &&
            topSwitch.paren === 0 &&
            topSwitch.bracket === 0 &&
            topSwitch.brace === 0 &&
            !topSwitch.inArmRhs;
          if (!isSwitchPatternArrow) {
            out.push(tokens[openParen], tokens[closeParen]);
            const exprEnd = skipExpressionUntil(
              tokens,
              bodyPos + 1,
              new Set([",", ")", "]", "}", ";"]),
            );
            i = exprEnd - 1;
            continue;
          }
        }
      }
    }

    if (topSwitch) {
      if (v === "(") topSwitch.paren++;
      else if (v === ")" && topSwitch.paren > 0) topSwitch.paren--;
      else if (v === "[") topSwitch.bracket++;
      else if (v === "]" && topSwitch.bracket > 0) topSwitch.bracket--;
      else if (v === "{") topSwitch.brace++;
      else if (v === "}" && topSwitch.brace > 0) topSwitch.brace--;
      else if (topSwitch.paren === 0 && topSwitch.bracket === 0 && topSwitch.brace === 0) {
        if (v === "=>") topSwitch.inArmRhs = true;
        else if (v === ",") topSwitch.inArmRhs = false;
      }
    }

    out.push(t);
  }

  return out;
}

/** Scans a slice of expression/statement tokens (with nested closure bodies
 * stripped, while keeping Dart 3 `switch` expressions and map/set literals)
 * for direct instance field mutations, `notifyListeners(...)`, or calls to
 * `knownMutatingMethods`. */
function findEffectsInFlatTokens(tokens, instanceFields, collectionFields, knownMutatingMethods, localVars) {
  const effects = [];
  const scanned = stripClosureBodies(tokens);

  for (let i = 0; i < scanned.length; i++) {
    const t = scanned[i];
    const v = t.value;

    if (t.kind !== "ident") continue;

    const prev = i > 0 ? scanned[i - 1].value : "";
    const prevPrev = i > 1 ? scanned[i - 2].value : "";
    const isMemberOnOther =
      (prev === "." || prev === "?." || prev === ".." || prev === "?..") &&
      prevPrev !== "this";
    const isExplicitThis = prev === "." && prevPrev === "this";
    const isLocal = !isExplicitThis && localVars.has(v);

    // 1. Direct assignment or ++/-- on an instance field
    if (!isMemberOnOther && !isLocal && instanceFields.has(v)) {
      const next = i + 1 < scanned.length ? scanned[i + 1].value : "";
      if (ASSIGNMENT_OPS.has(next) || next === "++" || next === "--") {
        effects.push({ line: t.line, description: `${v} ${next}` });
        continue;
      }
      if (prev === "++" || prev === "--") {
        effects.push({ line: t.line, description: `${prev}${v}` });
        continue;
      }
      if (isExplicitThis && i > 2 && (scanned[i - 3].value === "++" || scanned[i - 3].value === "--")) {
        effects.push({ line: t.line, description: `${scanned[i - 3].value}this.${v}` });
        continue;
      }
      // Index assignment `_files[idx] = ...`
      if (next === "[") {
        const closeBracket = findMatchingDelim(scanned, i + 1, "[", "]");
        const afterBracket = closeBracket + 1 < scanned.length ? scanned[closeBracket + 1].value : "";
        if (ASSIGNMENT_OPS.has(afterBracket)) {
          effects.push({ line: t.line, description: `${v}[] ${afterBracket}` });
          continue;
        }
      }
      // Collection mutator `_files.add(...)` / `_pendingShares.clear()`
      if (collectionFields.has(v) && (next === "." || next === "?." || next === "..") && i + 2 < scanned.length) {
        const memberTok = scanned[i + 2];
        const afterMember = i + 3 < scanned.length ? scanned[i + 3].value : "";
        if (
          memberTok.kind === "ident" &&
          COLLECTION_MUTATORS.has(memberTok.value) &&
          (afterMember === "(" || afterMember === "<")
        ) {
          effects.push({ line: memberTok.line, description: `${v}.${memberTok.value}()` });
          continue;
        }
      }
    }

    // 2. Call to `notifyListeners(...)` or a mutating helper on `this`
    if (!isMemberOnOther && !isLocal && (v === "notifyListeners" || knownMutatingMethods.has(v))) {
      const afterName = skipTypeArgs(scanned, i + 1);
      if (afterName < scanned.length && scanned[afterName].value === "(") {
        effects.push({ line: t.line, description: `${v}()` });
      }
    }
  }
  return effects;
}

/** Fixed-point computation of every instance method on the class that mutates
 * instance state or calls `notifyListeners()`. */
function computeMutatingMethods(instanceFields, collectionFields, methods) {
  const mutating = new Set(["notifyListeners"]);
  const instanceMethods = methods.filter((m) => !m.isStatic && m.name !== "dispose");
  let changed = true;
  while (changed) {
    changed = false;
    for (const m of instanceMethods) {
      if (mutating.has(m.name)) continue;
      const effects = findEffectsInFlatTokens(
        m.bodyTokens,
        instanceFields,
        collectionFields,
        mutating,
        new Set(),
      );
      if (effects.length > 0) {
        mutating.add(m.name);
        changed = true;
      }
    }
  }
  return mutating;
}

/** Parses a block of tokens into structured AST statements so Check 2 can
 * propagate `seenAwait`, `guarded`, and `terminates` across `if`, `try`,
 * `catch`, `finally`, loops, and `switch`. */
function parseStatements(tokens) {
  const stmts = [];
  let pos = 0;

  function parseOne() {
    while (pos < tokens.length && tokens[pos].value === ";") pos++;
    if (pos >= tokens.length) return null;

    const first = tokens[pos];

    if (first.value === "{") {
      const close = findMatchingDelim(tokens, pos, "{", "}");
      const inner = tokens.slice(pos + 1, close);
      pos = close + 1;
      return { kind: "block", stmts: parseStatements(inner), line: first.line };
    }

    if (first.kind === "ident" && first.value === "if" && pos + 1 < tokens.length && tokens[pos + 1].value === "(") {
      const closeCond = findMatchingDelim(tokens, pos + 1, "(", ")");
      const condTokens = tokens.slice(pos + 2, closeCond);
      pos = closeCond + 1;
      const thenStmt = parseOne() ?? { kind: "empty", line: first.line };
      let elseStmt = null;
      if (pos < tokens.length && tokens[pos].kind === "ident" && tokens[pos].value === "else") {
        pos++;
        elseStmt = parseOne() ?? { kind: "empty", line: first.line };
      }
      return { kind: "if", condTokens, thenStmt, elseStmt, line: first.line };
    }

    if (first.kind === "ident" && first.value === "try" && pos + 1 < tokens.length && tokens[pos + 1].value === "{") {
      const closeTry = findMatchingDelim(tokens, pos + 1, "{", "}");
      const tryStmts = parseStatements(tokens.slice(pos + 2, closeTry));
      pos = closeTry + 1;
      const catches = [];
      while (
        pos < tokens.length &&
        tokens[pos].kind === "ident" &&
        (tokens[pos].value === "on" || tokens[pos].value === "catch")
      ) {
        const catchLine = tokens[pos].line;
        if (tokens[pos].value === "on") {
          pos++;
          while (
            pos < tokens.length &&
            tokens[pos].value !== "{" &&
            !(tokens[pos].kind === "ident" && tokens[pos].value === "catch")
          ) {
            pos++;
          }
        }
        if (pos < tokens.length && tokens[pos].kind === "ident" && tokens[pos].value === "catch") {
          pos++;
          if (pos < tokens.length && tokens[pos].value === "(") {
            pos = findMatchingDelim(tokens, pos, "(", ")") + 1;
          }
        }
        if (pos < tokens.length && tokens[pos].value === "{") {
          const closeCatch = findMatchingDelim(tokens, pos, "{", "}");
          catches.push({
            line: catchLine,
            stmts: parseStatements(tokens.slice(pos + 1, closeCatch)),
          });
          pos = closeCatch + 1;
        } else {
          break;
        }
      }
      let finallyStmts = null;
      if (
        pos < tokens.length &&
        tokens[pos].kind === "ident" &&
        tokens[pos].value === "finally" &&
        pos + 1 < tokens.length &&
        tokens[pos + 1].value === "{"
      ) {
        const closeFin = findMatchingDelim(tokens, pos + 1, "{", "}");
        finallyStmts = parseStatements(tokens.slice(pos + 2, closeFin));
        pos = closeFin + 1;
      }
      return { kind: "try", tryStmts, catches, finallyStmts, line: first.line };
    }

    // `for (...) stmt` or `await for (...) stmt`
    if (
      (first.kind === "ident" && first.value === "for" && pos + 1 < tokens.length && tokens[pos + 1].value === "(") ||
      (first.kind === "ident" &&
        first.value === "await" &&
        pos + 2 < tokens.length &&
        tokens[pos + 1].kind === "ident" &&
        tokens[pos + 1].value === "for" &&
        tokens[pos + 2].value === "(")
    ) {
      const isAwaitFor = first.value === "await";
      const openParen = isAwaitFor ? pos + 2 : pos + 1;
      const closeParen = findMatchingDelim(tokens, openParen, "(", ")");
      const headerTokens = tokens.slice(openParen + 1, closeParen);
      pos = closeParen + 1;
      const bodyStmt = parseOne() ?? { kind: "empty", line: first.line };
      return { kind: "loop", isAwaitFor, headerTokens, bodyStmt, line: first.line };
    }

    if (first.kind === "ident" && first.value === "while" && pos + 1 < tokens.length && tokens[pos + 1].value === "(") {
      const closeParen = findMatchingDelim(tokens, pos + 1, "(", ")");
      const headerTokens = tokens.slice(pos + 2, closeParen);
      pos = closeParen + 1;
      const bodyStmt = parseOne() ?? { kind: "empty", line: first.line };
      return { kind: "loop", isAwaitFor: false, headerTokens, bodyStmt, line: first.line };
    }

    if (first.kind === "ident" && first.value === "do") {
      pos++;
      const bodyStmt = parseOne() ?? { kind: "empty", line: first.line };
      let headerTokens = [];
      if (pos < tokens.length && tokens[pos].kind === "ident" && tokens[pos].value === "while") {
        pos++;
        if (pos < tokens.length && tokens[pos].value === "(") {
          const closeParen = findMatchingDelim(tokens, pos, "(", ")");
          headerTokens = tokens.slice(pos + 1, closeParen);
          pos = closeParen + 1;
        }
      }
      if (pos < tokens.length && tokens[pos].value === ";") pos++;
      return { kind: "do", headerTokens, bodyStmt, line: first.line };
    }

    if (
      first.kind === "ident" &&
      first.value === "switch" &&
      pos + 1 < tokens.length &&
      tokens[pos + 1].value === "("
    ) {
      const closeParen = findMatchingDelim(tokens, pos + 1, "(", ")");
      const exprTokens = tokens.slice(pos + 2, closeParen);
      if (closeParen + 1 < tokens.length && tokens[closeParen + 1].value === "{") {
        const closeBrace = findMatchingDelim(tokens, closeParen + 1, "{", "}");
        const cases = parseSwitchCases(tokens.slice(closeParen + 2, closeBrace));
        pos = closeBrace + 1;
        return { kind: "switch", exprTokens, cases, line: first.line };
      }
    }

    // Simple statement up to matching `;` at depth 0
    const end = skipExpressionUntil(tokens, pos, new Set([";"]));
    const stmtTokens = tokens.slice(pos, end);
    pos = end < tokens.length && tokens[end].value === ";" ? end + 1 : end;
    return { kind: "simple", tokens: stmtTokens, line: first.line };
  }

  while (pos < tokens.length) {
    const s = parseOne();
    if (!s) break;
    if (s.kind !== "empty") stmts.push(s);
  }
  return stmts;
}

function parseSwitchCases(caseBodyTokens) {
  const cases = [];
  let current = [];
  let paren = 0;
  let bracket = 0;
  let brace = 0;
  for (let i = 0; i < caseBodyTokens.length; i++) {
    const t = caseBodyTokens[i];
    const v = t.value;
    if (paren === 0 && bracket === 0 && brace === 0 && t.kind === "ident" && (v === "case" || v === "default")) {
      if (current.length > 0) {
        cases.push(parseStatements(current));
        current = [];
      }
      // Advance past the `:` of `case ...:` or `default:`
      i = skipExpressionUntil(caseBodyTokens, i + 1, new Set([":"]));
      continue;
    }
    if (v === "(") paren++;
    else if (v === ")" && paren > 0) paren--;
    else if (v === "[") bracket++;
    else if (v === "]" && bracket > 0) bracket--;
    else if (v === "{") brace++;
    else if (v === "}" && brace > 0) brace--;
    current.push(t);
  }
  if (current.length > 0) cases.push(parseStatements(current));
  return cases;
}

function flatTokensHaveAwait(tokens) {
  const scanned = stripClosureBodies(tokens);
  return scanned.some((t) => t.kind === "ident" && t.value === "await");
}

function astHasAwait(stmts) {
  for (const s of stmts) {
    if (stmtHasAwait(s)) return true;
  }
  return false;
}

function stmtHasAwait(s) {
  switch (s.kind) {
    case "simple":
      return flatTokensHaveAwait(s.tokens);
    case "block":
      return astHasAwait(s.stmts);
    case "if":
      return (
        flatTokensHaveAwait(s.condTokens) ||
        stmtHasAwait(s.thenStmt) ||
        (s.elseStmt !== null && stmtHasAwait(s.elseStmt))
      );
    case "try":
      return (
        astHasAwait(s.tryStmts) ||
        s.catches.some((c) => astHasAwait(c.stmts)) ||
        (s.finallyStmts !== null && astHasAwait(s.finallyStmts))
      );
    case "loop":
      return s.isAwaitFor || flatTokensHaveAwait(s.headerTokens) || stmtHasAwait(s.bodyStmt);
    case "do":
      return stmtHasAwait(s.bodyStmt) || flatTokensHaveAwait(s.headerTokens);
    case "switch":
      return flatTokensHaveAwait(s.exprTokens) || s.cases.some((c) => astHasAwait(c));
    default:
      return false;
  }
}

function splitTopLevelBinary(tokens, opValue) {
  const parts = [];
  let start = 0;
  let paren = 0;
  let bracket = 0;
  let brace = 0;
  for (let i = 0; i < tokens.length; i++) {
    const v = tokens[i].value;
    if (v === "(") paren++;
    else if (v === ")" && paren > 0) paren--;
    else if (v === "[") bracket++;
    else if (v === "]" && bracket > 0) bracket--;
    else if (v === "{") brace++;
    else if (v === "}" && brace > 0) brace--;
    else if (paren === 0 && bracket === 0 && brace === 0 && v === opValue) {
      parts.push(tokens.slice(start, i));
      start = i + 1;
    }
  }
  parts.push(tokens.slice(start));
  return parts;
}

function stripOuterParens(tokens) {
  let slice = tokens;
  while (
    slice.length >= 2 &&
    slice[0].value === "(" &&
    findMatchingDelim(slice, 0, "(", ")") === slice.length - 1
  ) {
    slice = slice.slice(1, -1);
  }
  return slice;
}

function normalizeThisField(tokens) {
  if (tokens.length === 3 && tokens[0].value === "this" && tokens[1].value === "." && tokens[2].kind === "ident") {
    return tokens[2].value;
  }
  if (tokens.length === 1 && tokens[0].kind === "ident") {
    return tokens[0].value;
  }
  return null;
}

/** Evaluates whether an `if (cond)` condition acts as:
 * - `exitGuard`: `if (generation != _generation || _disposed || ...) return;`
 *   (De Morgan: at least one `||`-clause is a negative snapshot/disposed check)
 * - `enterGuard`: `if (generation == _generation && ...) { ... }` or `if (!_disposed)`
 *   (De Morgan: at least one `&&`-clause is a positive snapshot/disposed check) */
function evaluateGuardCondition(condTokens, snapshots, hasDisposedField) {
  const stripped = stripOuterParens(condTokens);

  let exitGuard = false;
  let exitUsedSnapshot = false;
  for (const rawClause of splitTopLevelBinary(stripped, "||")) {
    const c = stripOuterParens(rawClause);
    const snapMatch = matchComparison(c, "!=", snapshots);
    if (snapMatch) {
      exitGuard = true;
      exitUsedSnapshot = true;
    } else if (hasDisposedField && isPositiveDisposed(c)) {
      exitGuard = true;
    }
  }

  let enterGuard = false;
  let enterUsedSnapshot = false;
  for (const rawClause of splitTopLevelBinary(stripped, "&&")) {
    const c = stripOuterParens(rawClause);
    const snapMatch = matchComparison(c, "==", snapshots);
    if (snapMatch) {
      enterGuard = true;
      enterUsedSnapshot = true;
    } else if (hasDisposedField && isNegatedDisposed(c)) {
      enterGuard = true;
    }
  }

  return { exitGuard, exitUsedSnapshot, enterGuard, enterUsedSnapshot };
}

function matchComparison(clauseTokens, op, snapshots) {
  const parts = splitTopLevelBinary(clauseTokens, op);
  if (parts.length !== 2) return false;
  const left = normalizeThisField(stripOuterParens(parts[0]));
  const right = normalizeThisField(stripOuterParens(parts[1]));
  if (!left || !right) return false;
  return snapshots.get(left) === right || snapshots.get(right) === left;
}

function isPositiveDisposed(clauseTokens) {
  const field = normalizeThisField(clauseTokens);
  if (field === "_disposed") return true;
  const parts = splitTopLevelBinary(clauseTokens, "==");
  if (parts.length === 2) {
    const l = normalizeThisField(stripOuterParens(parts[0]));
    const r = stripOuterParens(parts[1]);
    if (l === "_disposed" && r.length === 1 && r[0].value === "true") return true;
  }
  return false;
}

function isNegatedDisposed(clauseTokens) {
  if (clauseTokens.length >= 2 && clauseTokens[0].value === "!") {
    const rest = normalizeThisField(stripOuterParens(clauseTokens.slice(1)));
    if (rest === "_disposed") return true;
  }
  const parts = splitTopLevelBinary(clauseTokens, "==");
  if (parts.length === 2) {
    const l = normalizeThisField(stripOuterParens(parts[0]));
    const r = stripOuterParens(parts[1]);
    if (l === "_disposed" && r.length === 1 && r[0].value === "false") return true;
  }
  return false;
}

function extractDeclaredLocal(stmtTokens) {
  if (stmtTokens.length < 2) return null;
  let idx = 0;
  if (stmtTokens[idx].kind === "ident" && (stmtTokens[idx].value === "final" || stmtTokens[idx].value === "const" || stmtTokens[idx].value === "var")) {
    const kw = stmtTokens[idx].value;
    idx++;
    if (idx < stmtTokens.length && stmtTokens[idx].kind === "ident") {
      // `final x = ...` or `final Type x = ...` or `final SendSession session`
      let cursor = idx + 1;
      cursor = skipTypeArgs(stmtTokens, cursor);
      if (cursor < stmtTokens.length && stmtTokens[cursor].value === "?") cursor++;
      if (cursor < stmtTokens.length && stmtTokens[cursor].kind === "ident") {
        return { name: stmtTokens[cursor].value, isFinal: kw === "final" || kw === "const", eqIdx: cursor + 1 };
      }
      return { name: stmtTokens[idx].value, isFinal: kw === "final" || kw === "const", eqIdx: idx + 1 };
    }
    return null;
  }
  // Typed local `String? clip` or `int count = 0`
  if (stmtTokens[0].kind === "ident") {
    let cursor = 1;
    cursor = skipTypeArgs(stmtTokens, cursor);
    if (cursor < stmtTokens.length && stmtTokens[cursor].value === "?") cursor++;
    if (
      cursor < stmtTokens.length &&
      stmtTokens[cursor].kind === "ident" &&
      (cursor + 1 === stmtTokens.length || stmtTokens[cursor + 1].value === "=")
    ) {
      return { name: stmtTokens[cursor].value, isFinal: false, eqIdx: cursor + 1 };
    }
  }
  return null;
}

function extractCounterSnapshot(stmtTokens, declaredCounters) {
  if (stmtTokens.length < 4 || stmtTokens[0].value !== "final") return null;
  const decl = extractDeclaredLocal(stmtTokens);
  if (!decl || !decl.isFinal) return null;
  if (decl.eqIdx >= stmtTokens.length || stmtTokens[decl.eqIdx].value !== "=") return null;
  const rhs = stripOuterParens(stmtTokens.slice(decl.eqIdx + 1));
  if (rhs.length === 0) return null;
  let rhsTokens = rhs;
  if (rhsTokens[0].value === "++") rhsTokens = rhsTokens.slice(1);
  const counter = normalizeThisField(rhsTokens);
  if (counter && declaredCounters.has(counter)) {
    return { snapName: decl.name, counter };
  }
  return null;
}

function joinStates(states) {
  const live = states.filter((s) => !s.terminates);
  if (live.length === 0) {
    const anyAwait = states.some((s) => s.seenAwait);
    return { seenAwait: anyAwait, guarded: true, terminates: true };
  }
  return {
    seenAwait: live.some((s) => s.seenAwait),
    guarded: live.every((s) => !s.seenAwait || s.guarded),
    terminates: false,
  };
}

/** Analyzes an `async` method body for Check 2. */
function analyzeAsyncMethod(
  method,
  declaredCounters,
  hasDisposedField,
  instanceFields,
  collectionFields,
  mutatingMethods,
) {
  const bodyTokens =
    method.bodyKind === "arrow"
      ? [{ kind: "ident", value: "return", line: method.line }, ...method.bodyTokens, { kind: "punct", value: ";", line: method.line }]
      : method.bodyTokens;
  const stmts = parseStatements(bodyTokens);
  if (!astHasAwait(stmts)) {
    return { hasAwait: false };
  }

  const snapshots = new Map();
  const localVars = new Set();
  const unguardedEffects = [];
  let anyPostAwaitEffect = false;
  let checkedSnapshotPostAwait = false;

  function recordEffects(tokens, state) {
    const effs = findEffectsInFlatTokens(
      tokens,
      instanceFields,
      collectionFields,
      mutatingMethods,
      localVars,
    );
    if (effs.length > 0 && state.seenAwait) {
      anyPostAwaitEffect = true;
      if (!state.guarded) {
        for (const e of effs) unguardedEffects.push(e);
      }
    }
  }

  function runBlock(blockStmts, inState) {
    let cur = { ...inState };
    for (const s of blockStmts) {
      if (cur.terminates) break;
      cur = runStmt(s, cur);
    }
    return cur;
  }

  function runStmt(s, state) {
    switch (s.kind) {
      case "block":
        return runBlock(s.stmts, state);

      case "simple": {
        const toks = s.tokens;
        if (toks.length === 0) return state;

        const decl = extractDeclaredLocal(toks);
        if (decl) localVars.add(decl.name);

        const hasAwait = flatTokensHaveAwait(toks);
        if (!state.seenAwait && !hasAwait) {
          const snap = extractCounterSnapshot(toks, declaredCounters);
          if (snap) snapshots.set(snap.snapName, snap.counter);
        }

        const nextState = hasAwait
          ? { seenAwait: true, guarded: false, terminates: false }
          : { ...state };

        recordEffects(toks, nextState);

        const firstKw = toks[0].kind === "ident" ? toks[0].value : "";
        if (
          firstKw === "return" ||
          firstKw === "throw" ||
          firstKw === "rethrow" ||
          firstKw === "break" ||
          firstKw === "continue"
        ) {
          nextState.terminates = true;
        }
        return nextState;
      }

      case "if": {
        const condHasAwait = flatTokensHaveAwait(s.condTokens);
        const afterCond = condHasAwait
          ? { seenAwait: true, guarded: false, terminates: false }
          : { ...state };
        recordEffects(s.condTokens, afterCond);

        const guard = afterCond.seenAwait
          ? evaluateGuardCondition(s.condTokens, snapshots, hasDisposedField)
          : { exitGuard: false, exitUsedSnapshot: false, enterGuard: false, enterUsedSnapshot: false };

        if (guard.enterUsedSnapshot) checkedSnapshotPostAwait = true;

        const thenIn = {
          ...afterCond,
          guarded: afterCond.guarded || (afterCond.seenAwait && guard.enterGuard),
        };
        const thenOut = runStmt(s.thenStmt, thenIn);

        if (guard.exitUsedSnapshot && (thenOut.terminates || s.elseStmt !== null)) {
          checkedSnapshotPostAwait = true;
        }

        const elseIn = {
          ...afterCond,
          guarded: afterCond.guarded || (afterCond.seenAwait && guard.exitGuard),
        };
        const elseOut = s.elseStmt ? runStmt(s.elseStmt, elseIn) : elseIn;
        return joinStates([thenOut, elseOut]);
      }

      case "try": {
        const tryOut = runBlock(s.tryStmts, state);
        const anyAwaitInTry = state.seenAwait || astHasAwait(s.tryStmts);
        const catchOuts = s.catches.map((c) =>
          runBlock(c.stmts, {
            seenAwait: anyAwaitInTry,
            guarded: anyAwaitInTry ? false : state.guarded,
            terminates: false,
          }),
        );
        const afterTryCatch = joinStates([tryOut, ...catchOuts]);
        if (!s.finallyStmts) return afterTryCatch;

        const anyAwaitBeforeFinally =
          anyAwaitInTry || s.catches.some((c) => astHasAwait(c.stmts));
        const finallyOut = runBlock(s.finallyStmts, {
          seenAwait: anyAwaitBeforeFinally,
          guarded: anyAwaitBeforeFinally ? false : afterTryCatch.guarded,
          terminates: false,
        });
        if (finallyOut.terminates) return finallyOut;
        if (afterTryCatch.terminates) {
          return {
            seenAwait: afterTryCatch.seenAwait || finallyOut.seenAwait,
            guarded: true,
            terminates: true,
          };
        }
        return {
          seenAwait: afterTryCatch.seenAwait || finallyOut.seenAwait,
          guarded:
            finallyOut.seenAwait && !astHasAwait(s.finallyStmts)
              ? afterTryCatch.guarded
              : finallyOut.guarded,
          terminates: false,
        };
      }

      case "loop": {
        const loopHasAwait =
          s.isAwaitFor || flatTokensHaveAwait(s.headerTokens) || stmtHasAwait(s.bodyStmt);
        const bodyIn = s.isAwaitFor || flatTokensHaveAwait(s.headerTokens)
          ? { seenAwait: true, guarded: false, terminates: false }
          : { ...state };
        recordEffects(s.headerTokens, bodyIn);
        runStmt(s.bodyStmt, bodyIn);
        if (loopHasAwait) {
          // A second iteration after an `await` inside the loop enters unguarded.
          runStmt(s.bodyStmt, { seenAwait: true, guarded: false, terminates: false });
          return { seenAwait: true, guarded: false, terminates: false };
        }
        return state;
      }

      case "do": {
        const firstOut = runStmt(s.bodyStmt, state);
        const condAwait = flatTokensHaveAwait(s.headerTokens);
        const afterCond = condAwait
          ? { seenAwait: true, guarded: false, terminates: false }
          : firstOut;
        recordEffects(s.headerTokens, afterCond);
        if (afterCond.seenAwait && (stmtHasAwait(s.bodyStmt) || condAwait)) {
          runStmt(s.bodyStmt, { seenAwait: true, guarded: false, terminates: false });
        }
        return afterCond;
      }

      case "switch": {
        const exprAwait = flatTokensHaveAwait(s.exprTokens);
        const afterExpr = exprAwait
          ? { seenAwait: true, guarded: false, terminates: false }
          : { ...state };
        recordEffects(s.exprTokens, afterExpr);
        if (s.cases.length === 0) return afterExpr;
        const caseOuts = s.cases.map((c) => runBlock(c, afterExpr));
        return joinStates(caseOuts);
      }

      default:
        return state;
    }
  }

  runBlock(stmts, { seenAwait: false, guarded: true, terminates: false });

  return {
    hasAwait: true,
    anyPostAwaitEffect,
    snapshots,
    checkedSnapshotPostAwait,
    unguardedEffects,
  };
}

/** Checks a map of `{ [relPath]: dartSource }` against all three state-machine
 * invariants. Exported so unit tests can run on synthetic Dart files in memory. */
export function checkStateMachines(files, { allowlist = ALLOWLIST, extraEnums = [] } = {}) {
  const problems = [];
  const knownEnums = new Set(extraEnums);
  const tokenizedFiles = new Map();

  // Validate allowlist entries up front (fail-closed on missing reasons).
  for (const [key, entry] of Object.entries(allowlist)) {
    const safeKey = sanitizeForLog(key);
    if (!entry || typeof entry.reason !== "string" || entry.reason.trim().length === 0) {
      problems.push(`ALLOWLIST[${safeKey}]: allowlist entry must carry a non-empty reason`);
    }
  }

  const sortedPaths = Object.keys(files).sort();
  for (const relPath of sortedPaths) {
    const tokens = tokenizeDart(files[relPath]);
    tokenizedFiles.set(relPath, tokens);
    collectEnumsFromTokens(tokens, knownEnums);
  }

  const usedAllowlistKeys = new Set();
  let checkedClasses = 0;

  for (const relPath of sortedPaths) {
    if (isGeneratedAppPath(relPath)) continue;
    const safePath = sanitizeForLog(relPath);
    const tokens = tokenizedFiles.get(relPath);
    const notifiers = findChangeNotifiers(tokens);

    for (const cls of notifiers) {
      checkedClasses++;
      const safeClass = sanitizeForLog(cls.name);
      const { fields, getters, methods } = parseClassMembers(cls.name, cls.bodyTokens);

      // Check 1: Stage/state/phase/status fields and getters must be a known enum.
      for (const member of [...fields, ...getters]) {
        if (member.isStatic || !STAGE_MEMBER_NAMES.has(member.name)) continue;
        const base = baseTypeName(member.declaredType);
        if (!base || !knownEnums.has(base)) {
          const shownType = member.declaredType ? sanitizeForLog(member.declaredType) : "untyped";
          problems.push(
            `${safePath}:${member.line}: ${safeClass}.${sanitizeForLog(member.name)} has type \`${shownType}\`; stage/state/phase/status on a ChangeNotifier must be an enum defined in app/lib/`,
          );
        }
      }

      // Check 3: Declared `_generation` / `_resets` (and `_disposed`) must be invalidated in `dispose()`.
      const nonStaticFields = fields.filter((f) => !f.isStatic);
      const instanceFieldSet = new Set(nonStaticFields.map((f) => f.name));
      const collectionFieldSet = new Set(
        nonStaticFields.filter((f) => f.isCollection).map((f) => f.name),
      );
      const declaredCounters = new Set(COUNTER_FIELDS.filter((c) => instanceFieldSet.has(c)));
      const hasDisposedField = instanceFieldSet.has("_disposed");
      const disposeMethod = methods.find((m) => !m.isStatic && m.name === "dispose");

      if (declaredCounters.size > 0 || hasDisposedField) {
        if (!disposeMethod) {
          const names = [...declaredCounters, ...(hasDisposedField ? ["_disposed"] : [])].join(", ");
          problems.push(
            `${safePath}:${cls.line}: ${safeClass} declares ${names} but defines no dispose() method to invalidate in-flight async calls`,
          );
        } else {
          for (const counter of declaredCounters) {
            if (!bodyIncrementsCounter(disposeMethod.bodyTokens, counter)) {
              problems.push(
                `${safePath}:${disposeMethod.line}: ${safeClass}.dispose() must increment ${counter} (e.g. ${counter}++) so in-flight async calls see a changed generation after disposal`,
              );
            }
          }
          if (hasDisposedField && !bodySetsDisposedTrue(disposeMethod.bodyTokens)) {
            problems.push(
              `${safePath}:${disposeMethod.line}: ${safeClass}.dispose() must set _disposed = true`,
            );
          }
        }
      }

      // Check 2: Async instance methods with post-`await` mutations or notifications.
      const mutatingMethods = computeMutatingMethods(instanceFieldSet, collectionFieldSet, methods);
      for (const method of methods) {
        if (method.isStatic || !method.isAsync) continue;
        const methodKey = `${cls.name}.${method.name}`;
        const safeMethod = `${safeClass}.${sanitizeForLog(method.name)}`;
        const result = analyzeAsyncMethod(
          method,
          declaredCounters,
          hasDisposedField,
          instanceFieldSet,
          collectionFieldSet,
          mutatingMethods,
        );
        if (!result.hasAwait || !result.anyPostAwaitEffect) continue;

        const methodProblems = [];
        if (result.snapshots.size === 0) {
          methodProblems.push(
            `${safePath}:${method.line}: ${safeMethod}() awaits and mutates state or calls notifyListeners() afterward without snapshotting _generation or _resets before the first await`,
          );
        } else if (!result.checkedSnapshotPostAwait) {
          const snaps = [...result.snapshots.keys()].map((k) => sanitizeForLog(k)).join(", ");
          methodProblems.push(
            `${safePath}:${method.line}: ${safeMethod}() snapshots \`${snaps}\` before await but never checks it after await`,
          );
        }
        for (const eff of result.unguardedEffects) {
          methodProblems.push(
            `${safePath}:${eff.line}: ${safeMethod}() reaches \`${sanitizeForLog(eff.description)}\` after await without a generation/reset/_disposed guard`,
          );
        }

        if (methodProblems.length > 0) {
          if (Object.hasOwn(allowlist, methodKey)) {
            usedAllowlistKeys.add(methodKey);
          } else {
            problems.push(...methodProblems);
          }
        }
      }
    }
  }

  for (const key of Object.keys(allowlist)) {
    if (!usedAllowlistKeys.has(key)) {
      problems.push(
        `ALLOWLIST[${sanitizeForLog(key)}]: stale allowlist entry (method no longer exists or already passes Check 2); remove it from ALLOWLIST`,
      );
    }
  }

  return { problems, checkedClasses, knownEnums };
}

function isGeneratedAppPath(relPath) {
  const norm = relPath.split(sep).join("/");
  return (
    norm.startsWith("app/lib/l10n/") ||
    norm.startsWith("app/lib/src/rust/") ||
    norm.startsWith("lib/l10n/") ||
    norm.startsWith("lib/src/rust/")
  );
}

/** Reads every `.dart` file under `<root>/app/lib`, keyed by POSIX path from `root`. */
export function loadAppDartFiles(root) {
  const appLib = join(root, "app", "lib");
  const out = {};
  function walk(dir) {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const full = join(dir, entry.name);
      if (entry.isDirectory()) {
        walk(full);
      } else if (entry.isFile() && entry.name.endsWith(".dart")) {
        const rel = relative(root, full).split(sep).join(posix.sep);
        out[rel] = readFileSync(full, "utf8");
      }
    }
  }
  walk(appLib);
  return out;
}

function main() {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const files = loadAppDartFiles(root);
  const { problems, checkedClasses } = checkStateMachines(files);
  if (problems.length > 0) {
    console.error('Android state-machine check failed (docs/surec.md, "Örtük durum makinesi"):');
    for (const p of problems) console.error(`  ${p}`);
    process.exit(1);
  }
  if (checkedClasses === 0) {
    console.error("Android state-machine check found 0 ChangeNotifier classes under app/lib/");
    process.exit(1);
  }
  console.log(
    `state machines ok: ${checkedClasses} ChangeNotifier classes in app/lib/ (enum stages, post-await guards, dispose invalidation)`,
  );
}

if (
  process.argv[1] &&
  existsSync(process.argv[1]) &&
  realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url))
) {
  main();
}
