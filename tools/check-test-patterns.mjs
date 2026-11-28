#!/usr/bin/env node
// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node tools/check-test-patterns.mjs
//
// Automated check for the weak test patterns recorded in docs/surec.md
// ("Zayıf test", #398, #399):
//
// 1. `any-error-matcher`: error assertions that pass on any failure without
//    checking what failed (#316), such as `expect(tester.takeException(),
//    isNotNull)`, `throwsException`, `throwsA(anything)` / `throwsA(isNotNull)`
//    in Dart tests, and `assert.throws` / `assert.rejects` with no error
//    matcher (or `undefined`, `null`, `Error`, `() => true`, `/.*/`) in JS/TS
//    tests.
// 2. `sleep-before-assert`: fixed wall-clock sleeps (`tokio::time::sleep`,
//    `std::thread::sleep`, `Future.delayed`, `sleep`, JS `setTimeout`
//    promises, QML `wait`) followed by an assertion in the same test or helper
//    function (#395), instead of waiting for a condition with a deadline
//    (`tokio::time::timeout` + poll loop, channels, synthetic
//    `tokio::time::pause` / `start_paused = true`, Flutter `tester.pump` /
//    `pumpAndSettle`, QML `tryCompare` / `tryVerify`).
//
// Legitimate bounded waits — such as negative assertions proving nothing
// happened during a quiet window, or black-box server timer expirations —
// live in `ALLOWLIST` below with a concrete `reason` beside each entry.
// Every allowlist entry must carry a non-empty `reason` and match an actual
// occurrence in the tree; stale entries fail the check so dead exceptions
// never linger.

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, realpathSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

export const RULES = ["any-error-matcher", "sleep-before-assert"];
export const MAX_FILE_BYTES = 8 * 1024 * 1024;

/** Legitimate exceptions keyed by `(file, test, rule)` (and optional `count`,
 * defaulting to 1), so line shifts elsewhere in a file never cause conflicts
 * while removed or extra occurrences fail immediately. */
export const ALLOWLIST = [
  {
    file: "crates/totaldrop-cli/src/main.rs",
    test: "a_panic_in_the_work_still_releases_what_it_held",
    rule: "sleep-before-assert",
    reason:
      "Held::drop spawns a 20 ms async task to prove run_then_release waits for async cleanup after a panic.",
  },
  {
    file: "crates/totaldrop-cli/tests/end_to_end.rs",
    test: "scary_close_retires_the_nameplate",
    rule: "sleep-before-assert",
    reason:
      "Black-box rendezvous server (Rust or Worker) retires the nameplate asynchronously after receiving CLOSE, with no client-visible event.",
  },
  {
    file: "crates/totaldrop-cli/tests/end_to_end.rs",
    test: "a_failed_code_counts_once",
    rule: "sleep-before-assert",
    reason:
      "Negative check: after polling until scary_count reaches 1, waits 300 ms to prove the second peer's scary close does not increment it again.",
  },
  {
    file: "crates/totaldrop-cli/tests/end_to_end.rs",
    test: "a_client_over_its_rate_is_refused_until_it_waits",
    rule: "sleep-before-assert",
    reason:
      "Black-box rate-limiter window: waits TEST_RATE_INTERVAL + margin so exactly one connection slot refills.",
  },
  {
    file: "crates/totaldrop-cli/tests/end_to_end.rs",
    test: "a_claimed_connection_outlives_the_claim_deadline",
    rule: "sleep-before-assert",
    reason:
      "Negative check: waits 3x the 300 ms claim_timeout to prove a claimed mailbox is not closed at the unclaimed deadline.",
  },
  {
    file: "crates/totaldrop-cli/tests/end_to_end.rs",
    test: "expired_nameplates_cannot_be_claimed",
    rule: "sleep-before-assert",
    reason:
      "Waits past unclaimed_ttl + sweep_interval before claiming, because polling claim earlier would claim the nameplate instead of letting it expire.",
  },
  {
    file: "crates/totaldrop-cli/tests/end_to_end.rs",
    test: "relay_flood",
    rule: "sleep-before-assert",
    reason:
      "Backpressure pacing loop pauses 5 ms while 16 MiB is in flight until the receiver reports progress.",
  },
  {
    file: "crates/totaldrop-cli/tests/end_to_end.rs",
    test: "stop_while_waiting_and_while_connecting",
    rule: "sleep-before-assert",
    reason:
      "Lets the spawned sender reach its idle mailbox wait before signalling Stop (no_events() emits no further event after PreparedSend::new).",
  },
  {
    file: "crates/totaldrop-core/src/path.rs",
    test: "a_moved_address_matches_in_either_family",
    rule: "sleep-before-assert",
    reason:
      "Advances Fallback's monotonic Instant past 0 ms from creation so heard_from stores a non-zero millisecond timestamp.",
  },
  {
    file: "crates/totaldrop-core/src/portmap.rs",
    test: "hints_carry_the_mapping_except_in_privacy_mode",
    rule: "sleep-before-assert",
    reason:
      "Lets both UDP release resends from the first subtest arrive and drain before asserting that privacy mode sends nothing to the router.",
  },
  {
    file: "crates/totaldrop-core/src/portmap.rs",
    test: "upnp_is_not_used_when_pcp_answers",
    rule: "sleep-before-assert",
    reason:
      "Negative check: waits RESEND * 4 after PCP answers to prove UPnP is never contacted.",
  },
  {
    file: "crates/totaldrop-core/src/relay.rs",
    test: "a_parked_link_does_not_wait_on_a_stuck_close",
    rule: "sleep-before-assert",
    count: 2,
    reason:
      "Fills the relay send buffer (no buffer-full event exists) and lets the link observe park() before set_moved(true) so it takes the stuck-close branch.",
  },
  {
    file: "crates/totaldrop-core/src/relay.rs",
    test: "a_parked_link_drops_what_waited_behind_its_gate",
    rule: "sleep-before-assert",
    reason:
      "Lets the spawned link task wake on gate.open() and discard the pre-gate datagram before queuing the post-gate datagram.",
  },
  {
    file: "crates/totaldrop-core/src/relay.rs",
    test: "recv_stun",
    rule: "sleep-before-assert",
    reason:
      "FakeStunSocket::recv_stun simulates delayed STUN replies under #[tokio::test(start_paused = true)] tests.",
  },
  {
    file: "crates/totaldrop-core/src/stop.rs",
    test: "stopped_resolves_for_every_waiter_and_after_the_fact",
    rule: "sleep-before-assert",
    reason:
      "Lets both spawned tasks park on Stop::stopped() before Stop::stop() wakes them.",
  },
  {
    file: "crates/totaldrop-core/src/transport.rs",
    test: "a_stream_can_have_several_megabytes_in_flight",
    rule: "sleep-before-assert",
    reason:
      "Spawned peer holds the QUIC connection open without reading (aborted once the writer fills the flow-control window).",
  },
  {
    file: "crates/totaldrop-core/src/transport.rs",
    test: "an_idle_direct_connection_stays_direct",
    rule: "sleep-before-assert",
    reason:
      "Negative check: waits 2 * silence so QUIC keep-alives run and proves an idle direct connection does not move to the relay.",
  },
  {
    file: "crates/totaldrop-core/src/transport.rs",
    test: "picks_up_after",
    rule: "sleep-before-assert",
    reason:
      "Fault injector holds the simulated network cut open for `gone` seconds before restoring traffic and polling with a deadline.",
  },
  {
    file: "linux/src/quit.rs",
    test: "quitting_waits_for_a_stopped_transfer_and_its_release",
    rule: "sleep-before-assert",
    count: 2,
    reason:
      "Simulates a 30 ms peer notification and a 30 ms async port-release task so finish() proves it waits for both in order.",
  },
];

// Backslash plus Unicode control (Cc), format/Bidi (Cf), and line/paragraph separators (Zl, Zp).
const UNSAFE_CHAR_RE = /[\\\p{Cc}\p{Cf}\p{Zl}\p{Zp}]/gu;

function escapeCodePoint(ch) {
  if (ch === "\\") return "\\\\";
  const cp = ch.codePointAt(0);
  if (cp <= 0xff) return `\\x${cp.toString(16).padStart(2, "0")}`;
  return `\\u{${cp.toString(16)}}`;
}

/** Neutralises backslash, control, and Bidi/format characters without splitting surrogates. */
export function safeText(value, maxLen = 160) {
  const chars = Array.from(String(value));
  const sliced = chars.length > maxLen ? `${chars.slice(0, maxLen).join("")}…` : chars.join("");
  return sliced.replace(UNSAFE_CHAR_RE, escapeCodePoint);
}

function lineStarts(text) {
  const starts = [0];
  for (let i = 0; i < text.length; i++) {
    if (text.charCodeAt(i) === 10) starts.push(i + 1);
  }
  return starts;
}

function lineOf(starts, offset) {
  let lo = 0;
  let hi = starts.length - 1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (starts[mid] <= offset) lo = mid + 1;
    else hi = mid - 1;
  }
  return hi + 1;
}

const REGEX_PRECEDING_KEYWORDS = new Set([
  "return",
  "throw",
  "case",
  "typeof",
  "void",
  "delete",
  "in",
  "instanceof",
  "of",
  "await",
  "yield",
  "new",
  "else",
  "do",
]);

/** Replaces comments and string/regex interiors with spaces while keeping
 * UTF-16 offsets, line breaks, and outer delimiters at their exact positions. */
export function maskCode(source, kind) {
  const n = source.length;
  // Split by UTF-16 code units so indices match `source[i]` even across astral code points.
  const out = source.split("");
  const blank = (from, to) => {
    for (let k = from; k < to && k < n; k++) {
      if (out[k] !== "\n") out[k] = " ";
    }
  };

  let i = 0;
  let lastSigChar = "";
  let lastWord = "";

  while (i < n) {
    const ch = source[i];
    const next = source[i + 1] ?? "";

    // Line comments in Rust, Dart, JS/TS and QML.
    if (ch === "/" && next === "/") {
      let j = i + 2;
      while (j < n && source[j] !== "\n") j++;
      blank(i, j);
      i = j;
      continue;
    }

    // Block comments (nested in Rust and Dart, non-nested in JS/TS/QML).
    if (ch === "/" && next === "*") {
      let j = i + 2;
      let depth = 1;
      const nested = kind === "rs" || kind === "dart";
      while (j < n && depth > 0) {
        if (nested && source[j] === "/" && source[j + 1] === "*") {
          depth++;
          j += 2;
        } else if (source[j] === "*" && source[j + 1] === "/") {
          depth--;
          j += 2;
        } else {
          j++;
        }
      }
      blank(i, j);
      i = j;
      continue;
    }

    if (kind === "rs") {
      // Rust char/byte literals (`'"'`, `b'"'`, `'\''`, `'\x7f'`, `'\u{10ffff}'`)
      // must be matched before `"` strings while leaving lifetimes (`'a`, `'static`) intact.
      if (
        ch === "'" ||
        (ch === "b" && next === "'" && (i === 0 || !/\w/.test(source[i - 1])))
      ) {
        const charLit =
          /^(?:b)?'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]{1,6}\}|.)|[^'\\\n\r\t])'/.exec(
            source.slice(i, i + 16),
          );
        if (charLit) {
          const qOpen = ch === "b" ? i + 1 : i;
          const qClose = i + charLit[0].length - 1;
          if (ch === "b") out[i] = " ";
          blank(qOpen + 1, qClose);
          i += charLit[0].length;
          continue;
        }
        if (ch === "'") {
          i++;
          continue;
        }
      }

      // Rust raw strings: r"...", r#"..."#, br#"..."#, cr#"..."#.
      if (
        (ch === "r" || ((ch === "b" || ch === "c") && next === "r")) &&
        (i === 0 || !/\w/.test(source[i - 1]))
      ) {
        const rIdx = ch === "r" ? i : i + 1;
        let k = rIdx + 1;
        while (k < n && source[k] === "#") k++;
        if (source[k] === '"') {
          const hashes = k - (rIdx + 1);
          blank(i, k); // blank prefix & opening hashes, keep opening `"`
          let j = k + 1;
          while (j < n) {
            if (source[j] === '"') {
              let h = 0;
              while (h < hashes && source[j + 1 + h] === "#") h++;
              if (h === hashes) {
                blank(k + 1, j); // blank interior, keep closing `"`
                blank(j + 1, j + 1 + hashes);
                i = j + 1 + hashes;
                break;
              }
            }
            j++;
          }
          if (j >= n) {
            blank(k + 1, n);
            i = n;
          }
          continue;
        }
      }

      // Rust byte/c-string prefix (`b"..."`, `c"..."`).
      if ((ch === "b" || ch === "c") && next === '"' && (i === 0 || !/\w/.test(source[i - 1]))) {
        out[i] = " ";
        i++;
        continue;
      }

      // Rust normal string `"..."`.
      if (ch === '"') {
        let j = i + 1;
        while (j < n) {
          if (source[j] === "\\") {
            j += 2;
            continue;
          }
          if (source[j] === '"') {
            blank(i + 1, j);
            j++;
            break;
          }
          j++;
        }
        if (j > n) j = n;
        i = j;
        continue;
      }

      i++;
      continue;
    }

    if (kind === "dart") {
      // Raw strings: r'''...''', r"""...""", r'...', r"...".
      const isRaw =
        ch === "r" && (next === "'" || next === '"') && (i === 0 || !/\w/.test(source[i - 1]));
      const qStart = isRaw ? i + 1 : i;
      const q = source[qStart];
      if (q === "'" || q === '"') {
        if (isRaw) out[i] = " ";
        const triple = source[qStart + 1] === q && source[qStart + 2] === q;
        const delimLen = triple ? 3 : 1;
        let j = qStart + delimLen;
        while (j < n) {
          if (!isRaw && source[j] === "\\") {
            j += 2;
            continue;
          }
          if (
            source[j] === q &&
            (!triple || (source[j + 1] === q && source[j + 2] === q))
          ) {
            blank(qStart + 1, j + delimLen - 1);
            j += delimLen;
            break;
          }
          j++;
        }
        if (j > n) j = n;
        i = j;
        continue;
      }
      i++;
      continue;
    }

    // JS / TS / QML strings, template literals, and single-line RegExp literals.
    if (ch === "'" || ch === '"' || ch === "`") {
      let j = i + 1;
      while (j < n) {
        if (source[j] === "\\") {
          j += 2;
          continue;
        }
        if (source[j] === ch) {
          blank(i + 1, j);
          j++;
          break;
        }
        j++;
      }
      if (j > n) j = n;
      lastSigChar = ch;
      lastWord = "";
      i = j;
      continue;
    }

    if (ch === "/") {
      const canStartRegex =
        lastSigChar === "" ||
        "([{,;:!=?&|^~+-*%<>".includes(lastSigChar) ||
        REGEX_PRECEDING_KEYWORDS.has(lastWord);
      if (canStartRegex) {
        let j = i + 1;
        let inClass = false;
        let foundClose = -1;
        while (j < n && source[j] !== "\n" && source[j] !== "\r") {
          if (source[j] === "\\") {
            j += 2;
            continue;
          }
          if (source[j] === "[") inClass = true;
          else if (source[j] === "]") inClass = false;
          else if (source[j] === "/" && !inClass) {
            foundClose = j;
            break;
          }
          j++;
        }
        if (foundClose > i) {
          blank(i + 1, foundClose);
          let end = foundClose + 1;
          while (end < n && /[dgimsuvy]/.test(source[end])) end++;
          lastSigChar = "/";
          lastWord = "";
          i = end;
          continue;
        }
      }
    }

    if (/[A-Za-z_$]/.test(ch)) {
      let j = i + 1;
      while (j < n && /[\w$]/.test(source[j])) j++;
      lastWord = source.slice(i, j);
      lastSigChar = source[j - 1];
      i = j;
      continue;
    }

    if (!/\s/.test(ch)) {
      lastSigChar = ch;
      lastWord = "";
    }
    i++;
  }

  return out.join("");
}

function findMatching(text, openIdx, openChar, closeChar, limit = text.length) {
  let depth = 0;
  for (let i = openIdx; i < limit; i++) {
    const c = text[i];
    if (c === openChar) depth++;
    else if (c === closeChar) {
      depth--;
      if (depth === 0) return i;
    }
  }
  return -1;
}

function splitCallArgs(masked, openParenIdx, closeParenIdx) {
  const args = [];
  let start = openParenIdx + 1;
  let p = 0;
  let b = 0;
  let c = 0;
  for (let i = openParenIdx + 1; i < closeParenIdx; i++) {
    const ch = masked[i];
    if (ch === "(") p++;
    else if (ch === ")") p = Math.max(0, p - 1);
    else if (ch === "[") b++;
    else if (ch === "]") b = Math.max(0, b - 1);
    else if (ch === "{") c++;
    else if (ch === "}") c = Math.max(0, c - 1);
    else if (ch === "," && p === 0 && b === 0 && c === 0) {
      args.push({ start, end: i });
      start = i + 1;
    }
  }
  const tail = masked.slice(start, closeParenIdx).trim();
  if (tail.length > 0) args.push({ start, end: closeParenIdx });
  return args;
}

/** Steps from `from` over balanced `(...)` and `[...]` to find the opening `{`
 * and matching `}` of a block; returns null if `;` is hit at depth 0 first. */
function findBlockAfterHeader(masked, from, limit, stopOnSemi = true) {
  let p = 0;
  let b = 0;
  for (let i = from; i < limit; i++) {
    const ch = masked[i];
    if (ch === "(") p++;
    else if (ch === ")") p = Math.max(0, p - 1);
    else if (ch === "[") b++;
    else if (ch === "]") b = Math.max(0, b - 1);
    else if (p === 0 && b === 0) {
      if (stopOnSemi && ch === ";") return null;
      if (ch === "{") {
        const close = findMatching(masked, i, "{", "}", limit);
        return close === -1 ? null : { open: i, close };
      }
    }
  }
  return null;
}

function skipRustOuterAttributes(masked, pos, limit) {
  let i = pos;
  while (i < limit) {
    while (i < limit && /\s/.test(masked[i])) i++;
    if (masked[i] === "#" && masked[i + 1] === "[") {
      const close = findMatching(masked, i + 1, "[", "]", limit);
      if (close === -1) return limit;
      i = close + 1;
      continue;
    }
    break;
  }
  return i;
}

function findRustTestRegions(masked, wholeFile) {
  if (wholeFile) return [{ start: 0, end: masked.length }];
  const regions = [];
  const cfgRe = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/g;
  for (const m of masked.matchAll(cfgRe)) {
    const attrStart = m.index;
    const afterAttrs = skipRustOuterAttributes(masked, attrStart + m[0].length, masked.length);
    const tail = masked.slice(afterAttrs);
    const modMatch = /^(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+[A-Za-z_]\w*\s*\{/.exec(tail);
    if (modMatch) {
      const braceIdx = afterAttrs + modMatch[0].length - 1;
      const close = findMatching(masked, braceIdx, "{", "}");
      if (close !== -1) regions.push({ start: braceIdx + 1, end: close });
      continue;
    }
    const fnMatch =
      /^(?:pub(?:\s*\([^)]*\))?\s+)?(?:(?:async|const|unsafe)\s+)*fn\s+[A-Za-z_]\w*/.exec(tail);
    if (fnMatch) {
      const block = findBlockAfterHeader(masked, afterAttrs + fnMatch[0].length, masked.length, true);
      if (block) regions.push({ start: attrStart, end: block.close + 1 });
    }
  }
  return regions;
}

function extractRustFunctions(masked, start, end, out = []) {
  let i = start;
  let attrStart = -1;
  while (i < end) {
    while (i < end && /\s/.test(masked[i])) i++;
    if (i >= end) break;

    // Inner `#![...]` or outer `#[...]` attribute.
    if (masked[i] === "#" && (masked[i + 1] === "[" || (masked[i + 1] === "!" && masked[i + 2] === "["))) {
      const isOuter = masked[i + 1] === "[";
      if (isOuter && attrStart === -1) attrStart = i;
      const bracketIdx = isOuter ? i + 1 : i + 2;
      const close = findMatching(masked, bracketIdx, "[", "]", end);
      i = close === -1 ? end : close + 1;
      if (!isOuter) attrStart = -1;
      continue;
    }

    const slice = masked.slice(i, end);
    const modMatch = /^(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+[A-Za-z_]\w*\s*\{/.exec(slice);
    if (modMatch) {
      const braceIdx = i + modMatch[0].length - 1;
      const close = findMatching(masked, braceIdx, "{", "}", end);
      attrStart = -1;
      if (close === -1) break;
      extractRustFunctions(masked, braceIdx + 1, close, out);
      i = close + 1;
      continue;
    }

    const implMatch = /^impl\b/.exec(slice);
    if (implMatch) {
      const block = findBlockAfterHeader(masked, i + implMatch[0].length, end, true);
      attrStart = -1;
      if (!block) {
        i += implMatch[0].length;
        continue;
      }
      extractRustFunctions(masked, block.open + 1, block.close, out);
      i = block.close + 1;
      continue;
    }

    const fnMatch =
      /^(?:pub(?:\s*\([^)]*\))?\s+)?(?:(?:async|const|unsafe)\s+)*fn\s+([A-Za-z_]\w*)/.exec(slice);
    if (fnMatch) {
      const block = findBlockAfterHeader(masked, i + fnMatch[0].length, end, true);
      const attrs = attrStart !== -1 ? masked.slice(attrStart, i) : "";
      attrStart = -1;
      if (!block) {
        const semi = masked.indexOf(";", i);
        i = semi === -1 ? end : semi + 1;
        continue;
      }
      out.push({
        name: fnMatch[1],
        attrs,
        bodyStart: block.open + 1,
        bodyEnd: block.close,
      });
      i = block.close + 1;
      continue;
    }

    attrStart = -1;
    if (masked[i] === "{") {
      const close = findMatching(masked, i, "{", "}", end);
      i = close === -1 ? end : close + 1;
    } else if (masked[i] === "(") {
      const close = findMatching(masked, i, "(", ")", end);
      i = close === -1 ? end : close + 1;
    } else {
      const word = /^[A-Za-z_]\w*/.exec(slice);
      i += word ? word[0].length : 1;
    }
  }
  return out;
}

function unwrapOuterParens(raw) {
  let t = raw.trim();
  while (t.startsWith("(") && findMatching(t, 0, "(", ")") === t.length - 1) {
    t = t.slice(1, -1).trim();
  }
  return t;
}

function isRealConditionHeader(raw) {
  const cond = unwrapOuterParens(raw);
  return cond !== "" && cond !== "true" && cond !== "1";
}

function extractAssertionSpans(bodyMasked, assertRe) {
  const out = [];
  for (const m of bodyMasked.matchAll(assertRe)) {
    const start = m.index;
    const after = start + m[0].length;
    let end = after;
    const prev = bodyMasked[after - 1];
    let k = prev === "(" || prev === "[" || prev === "{" ? after - 1 : after;
    if (k === after) {
      while (k < bodyMasked.length && /\w/.test(bodyMasked[k])) k++;
      while (k < bodyMasked.length && /\s/.test(bodyMasked[k])) k++;
    }
    const ch = bodyMasked[k];
    if (ch === "(" || ch === "[" || ch === "{") {
      const closeChar = ch === "(" ? ")" : ch === "[" ? "]" : "}";
      const close = findMatching(bodyMasked, k, ch, closeChar);
      if (close !== -1) end = close + 1;
    }
    const text = bodyMasked.slice(start, end);
    const isDeadlineGuard =
      /\b(?:elapsed\w*|deadline\w*|timeout\w*|limit\w*|Date\s*\.\s*now|Instant\s*::\s*now|SystemTime\s*::\s*now|performance\s*\.\s*now|lessThan\w*)\b/.test(
        text,
      );
    out.push({ index: start, isDeadlineGuard });
  }
  return out;
}

function findPollingLoops(bodyMasked, kind = "") {
  const loops = [];
  const loopKw = /\b(while|for|loop|do)\b/g;
  for (const m of bodyMasked.matchAll(loopKw)) {
    const kw = m[1];
    const after = m.index + kw.length;
    if (kw === "loop") {
      if (kind !== "rs") continue;
      const braceMatch = /^\s*\{/.exec(bodyMasked.slice(after));
      if (!braceMatch) continue;
      const open = after + braceMatch[0].length - 1;
      const close = findMatching(bodyMasked, open, "{", "}");
      if (close !== -1) loops.push({ open, close, hasRealCond: false });
      continue;
    }
    if (kw === "do") {
      if (kind === "rs") continue;
      const braceMatch = /^\s*\{/.exec(bodyMasked.slice(after));
      if (!braceMatch) continue;
      const open = after + braceMatch[0].length - 1;
      const close = findMatching(bodyMasked, open, "{", "}");
      if (close === -1) continue;
      const trailingWhile = /^\s*while\s*\(/.exec(bodyMasked.slice(close + 1));
      if (!trailingWhile) continue;
      const wOpen = close + 1 + trailingWhile[0].length - 1;
      const wClose = findMatching(bodyMasked, wOpen, "(", ")");
      const condRaw = wClose === -1 ? "" : bodyMasked.slice(wOpen, wClose + 1);
      loops.push({ open, close, hasRealCond: isRealConditionHeader(condRaw) });
      continue;
    }
    const block = findBlockAfterHeader(bodyMasked, after, bodyMasked.length, true);
    if (!block) continue;
    const header = bodyMasked.slice(after, block.open);
    if (kw === "for") {
      // In Rust, `impl Trait for Type { ... }` uses `for` without `in`.
      if (kind === "rs" && !/\bin\b/.test(header)) continue;
      // A `for` loop without `break` or `return` runs every iteration unconditionally
      // (e.g. a table-driven `for case in cases` loop) and is never a condition poll.
      const loopBody = bodyMasked.slice(block.open + 1, block.close);
      if (!/\b(?:break|return)\b/.test(loopBody)) continue;
      loops.push({ open: block.open, close: block.close, hasRealCond: false });
    } else {
      loops.push({
        open: block.open,
        close: block.close,
        hasRealCond: isRealConditionHeader(header),
      });
    }
  }
  return loops;
}

function isExemptByPollingLoop(loops, asserts, sleepIdx, sleepEnd) {
  return loops.some((l) => {
    if (sleepIdx <= l.open || sleepIdx >= l.close) return false;
    return !asserts.some(
      (a) => a.index > sleepEnd && a.index < l.close && !(l.hasRealCond && a.isDeadlineGuard),
    );
  });
}

function scanRust(file, source, wholeFile) {
  const masked = maskCode(source, "rs");
  const starts = lineStarts(source);
  const findings = [];
  const regions = findRustTestRegions(masked, wholeFile);

  const fns = [];
  for (const r of regions) extractRustFunctions(masked, r.start, r.end, fns);

  const baseAssertRe =
    /\b(?:(?:debug_)?assert(?:_eq|_ne)?|panic)\s*!|\bassert_[A-Za-z0-9_]*\s*\(|\.\s*(?:unwrap|expect)\s*\(/g;
  const helperNames = new Set();
  for (const fn of fns) {
    if (/\btest\b/.test(fn.attrs)) continue;
    const body = masked.slice(fn.bodyStart, fn.bodyEnd);
    baseAssertRe.lastIndex = 0;
    if (baseAssertRe.test(body)) helperNames.add(fn.name);
  }
  const helperAlt = [...helperNames]
    .sort()
    .map((n) => n.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
    .join("|");
  const assertRe = helperAlt
    ? new RegExp(`${baseAssertRe.source}|\\b(?:${helperAlt})\\s*\\(`, "g")
    : baseAssertRe;
  const sleepRe =
    /(?:(?:\bstd\s*::\s*)?\bthread\s*::\s*sleep|(?:\btokio\s*::\s*)?\btime\s*::\s*sleep|(?<![.\w])sleep)\s*\(/g;
  const pauseRe = /\b(?:tokio\s*::\s*time\s*::\s*|time\s*::\s*)pause\s*\(/;
  const selectRe = /\b(?:tokio\s*::\s*)?select\s*!\s*\{/g;

  for (const fn of fns) {
    const body = masked.slice(fn.bodyStart, fn.bodyEnd);
    const asserts = extractAssertionSpans(body, assertRe);
    if (asserts.length === 0) continue;

    const startPaused = /\bstart_paused\s*=\s*true\b/.test(fn.attrs);
    const pauseMatch = pauseRe.exec(body);
    const pauseIdx = pauseMatch ? pauseMatch.index : -1;

    const selects = [];
    for (const m of body.matchAll(selectRe)) {
      const brace = m.index + m[0].length - 1;
      const close = findMatching(body, brace, "{", "}");
      if (close !== -1) selects.push({ open: brace, close });
    }
    const loops = findPollingLoops(body, "rs");

    for (const m of body.matchAll(sleepRe)) {
      const sleepIdx = m.index;
      const openParen = sleepIdx + m[0].length - 1;
      const closeParen = findMatching(body, openParen, "(", ")");
      const sleepEnd = closeParen === -1 ? openParen + 1 : closeParen + 1;

      const isThreadSleep = /thread\s*::\s*sleep/.test(m[0]);
      if (!isThreadSleep && (startPaused || (pauseIdx !== -1 && pauseIdx < sleepIdx))) {
        continue;
      }
      if (selects.some((s) => sleepIdx > s.open && sleepIdx < s.close)) {
        continue;
      }
      if (isExemptByPollingLoop(loops, asserts, sleepIdx, sleepEnd)) {
        continue;
      }

      if (asserts.some((a) => a.index > sleepEnd)) {
        findings.push({
          file,
          test: fn.name,
          line: lineOf(starts, fn.bodyStart + sleepIdx),
          rule: "sleep-before-assert",
          detail: "fixed sleep before assertion; wait for a condition with a deadline",
        });
      }
    }
  }
  return findings;
}

function unquoteDescription(raw) {
  const t = raw.trim();
  for (const q of ["'''", '"""', "'", '"', "`"]) {
    if (t.startsWith(q) && t.endsWith(q) && t.length >= q.length * 2) {
      return t.slice(q.length, t.length - q.length);
    }
  }
  return t;
}

function findCallbackTestScopes(source, masked, callRe) {
  const scopes = [];
  for (const m of masked.matchAll(callRe)) {
    const openParen = m.index + m[0].length - 1;
    const closeParen = findMatching(masked, openParen, "(", ")");
    if (closeParen === -1) continue;
    const args = splitCallArgs(masked, openParen, closeParen);
    if (args.length < 2) continue;
    const name = unquoteDescription(source.slice(args[0].start, args[0].end));
    const cb = args[1];
    const brace = masked.indexOf("{", cb.start);
    if (brace !== -1 && brace < cb.end) {
      const closeBrace = findMatching(masked, brace, "{", "}", cb.end + 1);
      if (closeBrace !== -1) {
        scopes.push({ name, start: brace + 1, end: closeBrace });
        continue;
      }
    }
    scopes.push({ name, start: cb.start, end: cb.end });
  }
  return scopes;
}

function enclosingScopeName(scopes, pos) {
  let best = null;
  for (const s of scopes) {
    if (pos >= s.start && pos <= s.end) {
      if (!best || s.end - s.start <= best.end - best.start) best = s;
    }
  }
  return best ? best.name : "<top-level>";
}

const DART_CATCH_ALL_INNER =
  /^(?:anything|isNotNull|isException|isFlutterError|(?:isA|isInstanceOf)<(?:Object|dynamic|Exception|Error|FlutterError)>\(\)|predicate\(\([^)]*\)(?:=>true|\{returntrue;?\})\))$/;

function isWeakDartMatcher(actualCompact, matcherCompact) {
  if (/\btakeException\(\)/.test(actualCompact) && DART_CATCH_ALL_INNER.test(matcherCompact)) {
    return true;
  }
  if (matcherCompact === "throwsException" || matcherCompact === "throwsFlutterError") {
    return true;
  }
  if (matcherCompact.startsWith("throwsA(") && matcherCompact.endsWith(")")) {
    const inner = matcherCompact.slice("throwsA(".length, -1);
    if (DART_CATCH_ALL_INNER.test(inner)) return true;
  }
  return false;
}

function scanDart(file, source) {
  const masked = maskCode(source, "dart");
  const starts = lineStarts(source);
  const findings = [];
  const scopes = findCallbackTestScopes(source, masked, /\b(?:test|testWidgets|it)\s*\(/g);

  // 1. Catch-all error matchers (`#316`).
  for (const m of masked.matchAll(/\b(?:expect|expectLater)\s*\(/g)) {
    const openParen = m.index + m[0].length - 1;
    const closeParen = findMatching(masked, openParen, "(", ")");
    if (closeParen === -1) continue;
    const args = splitCallArgs(masked, openParen, closeParen);
    if (args.length < 2) continue;
    const actualCompact = masked.slice(args[0].start, args[0].end).replace(/\s+/g, "");
    const matcherCompact = masked.slice(args[1].start, args[1].end).replace(/\s+/g, "");
    if (isWeakDartMatcher(actualCompact, matcherCompact)) {
      const rawMatcher = source.slice(args[1].start, args[1].end).replace(/\s+/g, " ").trim();
      findings.push({
        file,
        test: enclosingScopeName(scopes, m.index),
        line: lineOf(starts, m.index),
        rule: "any-error-matcher",
        detail: `catch-all error matcher \`${safeText(rawMatcher, 80)}\`; check the specific error type and message`,
      });
    }
  }

  // 2. Fixed wall-clock sleeps (`Future.delayed`, `sleep`) before `expect`.
  const checkRegions = scopes.length > 0 ? scopes : [{ name: "<top-level>", start: 0, end: masked.length }];
  const assertRe = /\b(?:expect|expectLater|fail)\s*\(/g;
  const sleepRe = /\bFuture\s*(?:<[^>]*>\s*)?\.\s*delayed\s*\(|(?<![.\w])sleep\s*\(/g;

  for (const scope of checkRegions) {
    const body = masked.slice(scope.start, scope.end);
    const asserts = extractAssertionSpans(body, assertRe);
    if (asserts.length === 0) continue;
    const loops = findPollingLoops(body, "dart");
    for (const m of body.matchAll(sleepRe)) {
      const sleepIdx = m.index;
      const openParen = sleepIdx + m[0].length - 1;
      const closeParen = findMatching(body, openParen, "(", ")");
      const sleepEnd = closeParen === -1 ? openParen + 1 : closeParen + 1;
      if (isExemptByPollingLoop(loops, asserts, sleepIdx, sleepEnd)) continue;
      if (asserts.some((a) => a.index > sleepEnd)) {
        findings.push({
          file,
          test: scope.name,
          line: lineOf(starts, scope.start + sleepIdx),
          rule: "sleep-before-assert",
          detail: "fixed sleep before assertion; wait for a condition or step the widget clock with tester.pump",
        });
      }
    }
  }

  return findings;
}

const TRIVIAL_REGEX_RE = /^\/(?:\.|\.\*|\[\^\]\*|\[\\s\\S\]\*|\(\?:\)|\^|\$)?\/[dgimsuvy]*$/;
const TRIVIAL_REGEXP_CTOR_RE =
  /^(?:new\s+)?RegExp\s*\(\s*(?:""|''|``|"\.\*"|'\.\*'|`\.\*`|"\."|'\.'|`\.`)?\s*\)$/;

function isWeakJsErrorArg(errMasked, errRaw) {
  const t = errMasked.trim();
  if (t === "" || t === "undefined" || t === "null" || t === "Error" || t === "Object") {
    return true;
  }
  if (/^\{\s*\}$/.test(t)) return true;
  if (/^(?:async\b\s*)?(?:\([^)]*\)|[A-Za-z_$][\w$]*)\s*=>\s*(?:true|\{\s*return\s+true\s*(?:;\s*)?\})$/.test(t)) {
    return true;
  }
  if (/^function\s*\([^)]*\)\s*\{\s*return\s+true\s*(?:;\s*)?\}$/.test(t)) return true;
  if (
    /^(?:async\b\s*)?(?:\(\s*[A-Za-z_$][\w$]*\s*\)|[A-Za-z_$][\w$]*)\s*=>\s*(?:[A-Za-z_$][\w$]*\s+instanceof\s+(?:Error|Object)|\{\s*return\s+[A-Za-z_$][\w$]*\s+instanceof\s+(?:Error|Object)\s*(?:;\s*)?\})$/.test(
      t,
    )
  ) {
    return true;
  }
  if (
    /^function\s*\(\s*[A-Za-z_$][\w$]*\s*\)\s*\{\s*return\s+[A-Za-z_$][\w$]*\s+instanceof\s+(?:Error|Object)\s*(?:;\s*)?\}$/.test(
      t,
    )
  ) {
    return true;
  }
  const raw = errRaw.trim();
  if (TRIVIAL_REGEX_RE.test(raw) || TRIVIAL_REGEXP_CTOR_RE.test(raw)) return true;
  return false;
}

function scanJs(file, source) {
  const masked = maskCode(source, "js");
  const starts = lineStarts(source);
  const findings = [];
  const scopes = findCallbackTestScopes(source, masked, /\b(?:test|it)\s*\(/g);

  // 1. Bare or catch-all `assert.throws` / `assert.rejects`.
  for (const m of masked.matchAll(/\bassert\s*\.\s*(throws|rejects)\s*\(/g)) {
    const kind = m[1];
    const openParen = m.index + m[0].length - 1;
    const closeParen = findMatching(masked, openParen, "(", ")");
    if (closeParen === -1) continue;
    const args = splitCallArgs(masked, openParen, closeParen);
    const errMasked = args.length >= 2 ? masked.slice(args[1].start, args[1].end) : "";
    const errRaw = args.length >= 2 ? source.slice(args[1].start, args[1].end) : "";
    if (args.length < 2 || isWeakJsErrorArg(errMasked, errRaw)) {
      findings.push({
        file,
        test: enclosingScopeName(scopes, m.index),
        line: lineOf(starts, m.index),
        rule: "any-error-matcher",
        detail: `assert.${kind} without a specific error matcher; pass a RegExp, error class or predicate`,
      });
    }
  }

  // 2. Fixed `setTimeout` sleep before `assert`.
  const checkRegions = scopes.length > 0 ? scopes : [{ name: "<top-level>", start: 0, end: masked.length }];
  const assertRe = /\bassert\s*(?:\.|\()/g;
  const sleepStartRe = /\bnew\s+Promise\s*\(|\bawait\s+setTimeout\s*\(/g;

  for (const scope of checkRegions) {
    const body = masked.slice(scope.start, scope.end);
    const asserts = extractAssertionSpans(body, assertRe);
    if (asserts.length === 0) continue;
    const loops = findPollingLoops(body, "js");
    for (const m of body.matchAll(sleepStartRe)) {
      const sleepIdx = m.index;
      const openParen = sleepIdx + m[0].length - 1;
      const closeParen = findMatching(body, openParen, "(", ")");
      if (m[0].startsWith("new")) {
        if (closeParen === -1) continue;
        if (!/\bsetTimeout\b/.test(body.slice(openParen + 1, closeParen))) continue;
      }
      const sleepEnd = closeParen === -1 ? openParen + 1 : closeParen + 1;
      if (isExemptByPollingLoop(loops, asserts, sleepIdx, sleepEnd)) continue;
      if (asserts.some((a) => a.index > sleepEnd)) {
        findings.push({
          file,
          test: scope.name,
          line: lineOf(starts, scope.start + sleepIdx),
          rule: "sleep-before-assert",
          detail: "fixed setTimeout sleep before assertion; poll with a deadline",
        });
      }
    }
  }

  return findings;
}

function scanQml(file, source) {
  const masked = maskCode(source, "js");
  const starts = lineStarts(source);
  const findings = [];
  const fnRe = /\bfunction\s+(test_[A-Za-z0-9_]*)\s*\([^)]*\)\s*\{/g;
  const assertRe = /\b(?:verify|compare|tryCompare|tryVerify)\s*\(/g;
  const waitRe = /(?<![.\w])wait\s*\(/g;

  for (const m of masked.matchAll(fnRe)) {
    const name = m[1];
    const open = m.index + m[0].length - 1;
    const close = findMatching(masked, open, "{", "}");
    if (close === -1) continue;
    const bodyStart = open + 1;
    const body = masked.slice(bodyStart, close);
    const asserts = extractAssertionSpans(body, assertRe);
    if (asserts.length === 0) continue;
    const loops = findPollingLoops(body, "qml");
    for (const w of body.matchAll(waitRe)) {
      const waitIdx = w.index;
      const openParen = waitIdx + w[0].length - 1;
      const closeParen = findMatching(body, openParen, "(", ")");
      const waitEnd = closeParen === -1 ? openParen + 1 : closeParen + 1;
      if (isExemptByPollingLoop(loops, asserts, waitIdx, waitEnd)) continue;
      if (asserts.some((a) => a.index > waitEnd)) {
        findings.push({
          file,
          test: name,
          line: lineOf(starts, bodyStart + waitIdx),
          rule: "sleep-before-assert",
          detail: "fixed wait() before assertion in QML test; use tryCompare() or tryVerify()",
        });
      }
    }
  }
  return findings;
}

const IGNORED_PREFIXES = [
  "target/",
  "node_modules/",
  ".git/",
  ".dart_tool/",
  "build/",
  "fuzz/",
  "app/lib/l10n/",
  "app/lib/src/rust/",
  "app/rust_builder/cargokit/",
];

function isIgnoredPath(norm) {
  if (norm === "frb_generated.rs" || norm.endsWith("/frb_generated.rs")) return true;
  return IGNORED_PREFIXES.some((prefix) => norm.startsWith(prefix));
}

function classifyFile(path) {
  const norm = path.replaceAll("\\", "/");
  if (isIgnoredPath(norm)) return null;
  if (norm.endsWith(".rs")) {
    const wholeFile = norm.startsWith("tests/") || norm.includes("/tests/");
    return { kind: "rs", wholeFile };
  }
  if (norm.endsWith(".dart")) {
    const segments = norm.split("/");
    const inAppTest =
      norm.startsWith("app/test/") || norm.startsWith("app/integration_test/");
    const inNamedTestDir =
      norm.endsWith("_test.dart") &&
      segments.slice(0, -1).some((s) => s === "test" || s === "integration_test");
    if (inAppTest || inNamedTestDir) return { kind: "dart" };
    return null;
  }
  if (
    /(?:^|\/)test-[^/]+\.(?:mjs|js|ts)$/.test(norm) ||
    /\.(?:test|spec)\.(?:mjs|js|ts)$/.test(norm)
  ) {
    return { kind: "js" };
  }
  if (
    norm.endsWith(".qml") &&
    (norm.startsWith("tests/") || norm.includes("/tests/")) &&
    !norm.startsWith("stub/") &&
    !norm.includes("/stub/")
  ) {
    return { kind: "qml" };
  }
  return null;
}

/** Scans a single file and returns raw weak-pattern findings. */
export function scanFile(path, source) {
  const norm = path.replaceAll("\\", "/");
  const info = classifyFile(norm);
  if (!info) return [];
  if (info.kind === "rs") return scanRust(norm, source, info.wholeFile);
  if (info.kind === "dart") return scanDart(norm, source);
  if (info.kind === "js") return scanJs(norm, source);
  if (info.kind === "qml") return scanQml(norm, source);
  return [];
}

const KNOWN_ALLOWLIST_KEYS = new Set(["file", "test", "rule", "count", "reason"]);

function validateAllowlist(allowlist) {
  const problems = [];
  const map = new Map();
  if (!Array.isArray(allowlist)) {
    return { problems: ["allowlist: expected an array"], map };
  }
  for (let idx = 0; idx < allowlist.length; idx++) {
    const entry = allowlist[idx];
    const label = `allowlist[${idx}]`;
    if (!entry || typeof entry !== "object" || Array.isArray(entry)) {
      problems.push(`${label}: expected an object`);
      continue;
    }
    const extra = Object.keys(entry).filter((k) => !KNOWN_ALLOWLIST_KEYS.has(k));
    if (extra.length > 0) {
      problems.push(`${label}: unknown fields: ${extra.map((k) => safeText(k)).join(", ")}`);
      continue;
    }
    if (
      typeof entry.file !== "string" ||
      entry.file.trim() === "" ||
      entry.file.startsWith("/") ||
      entry.file.includes("..")
    ) {
      problems.push(`${label}: invalid \`file\`: ${safeText(entry.file)}`);
      continue;
    }
    if (typeof entry.test !== "string" || entry.test.trim() === "") {
      problems.push(`${label} (${safeText(entry.file)}): \`test\` must be a non-empty test/function name`);
      continue;
    }
    if (!RULES.includes(entry.rule)) {
      problems.push(`${label} (${safeText(entry.file)}): unknown \`rule\` "${safeText(entry.rule)}"`);
      continue;
    }
    const count = entry.count ?? 1;
    if (!Number.isInteger(count) || count < 1) {
      problems.push(`${label} (${safeText(entry.file)}): \`count\` must be a positive integer`);
      continue;
    }
    if (typeof entry.reason !== "string" || entry.reason.trim() === "") {
      problems.push(
        `allowlist: ${safeText(entry.file)} (${safeText(entry.test)}, ${entry.rule}): missing non-empty \`reason\``,
      );
      continue;
    }
    const key = `${entry.file}\0${entry.test}\0${entry.rule}`;
    if (map.has(key)) {
      problems.push(
        `allowlist: ${safeText(entry.file)} (${safeText(entry.test)}, ${entry.rule}): duplicate allowlist entry`,
      );
      continue;
    }
    map.set(key, { ...entry, count });
  }
  return { problems, map };
}

/** Checks `[path, source]` pairs against `allowlist`. */
export function checkPatterns(files, allowlist = ALLOWLIST) {
  const { problems, map } = validateAllowlist(allowlist);
  const findings = [];
  for (const item of files) {
    const [path, source] = Array.isArray(item) ? item : [item.path, item.source];
    findings.push(...scanFile(path, source));
  }

  const grouped = new Map();
  for (const f of findings) {
    const key = `${f.file}\0${f.test}\0${f.rule}`;
    if (!grouped.has(key)) grouped.set(key, []);
    grouped.get(key).push(f);
  }

  for (const [key, list] of grouped) {
    const allowed = map.get(key);
    if (!allowed) {
      for (const f of list) {
        problems.push(
          `${safeText(f.file)}:${f.line} (${safeText(f.test)}): [${f.rule}] ${f.detail}`,
        );
      }
    } else if (list.length > allowed.count) {
      for (const f of list.slice(allowed.count)) {
        problems.push(
          `${safeText(f.file)}:${f.line} (${safeText(f.test)}): [${f.rule}] ${f.detail} (allowlisted ${allowed.count}, found ${list.length})`,
        );
      }
    }
  }

  for (const [key, allowed] of map) {
    const actual = grouped.get(key)?.length ?? 0;
    if (actual < allowed.count) {
      problems.push(
        `allowlist: ${safeText(allowed.file)} (${safeText(allowed.test)}, ${allowed.rule}): stale entry (allowlisted ${allowed.count}, found ${actual}); remove or update it in tools/check-test-patterns.mjs`,
      );
    }
  }

  return { problems, findings };
}

function listCandidateFiles(root) {
  if (existsSync(join(root, ".git"))) {
    try {
      const out = execFileSync("git", ["-C", root, "ls-files", "-z"], {
        encoding: "utf8",
        maxBuffer: 32 << 20,
      });
      return out.split("\0").filter(Boolean);
    } catch {
      // Fall back to walking the directory tree if git refuses (e.g. container ownership).
    }
  }
  const found = [];
  const walk = (dir) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      if (
        entry.name === ".git" ||
        entry.name === "node_modules" ||
        entry.name === "target" ||
        entry.name === "build" ||
        entry.name === ".dart_tool"
      ) {
        continue;
      }
      const full = join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (entry.isFile()) found.push(relative(root, full).replaceAll("\\", "/"));
    }
  };
  walk(root);
  return found.sort();
}

/** Scans every test file in `root` and checks findings against `allowlist`. */
export function checkRepo(root, allowlist = ALLOWLIST, { maxFileBytes = MAX_FILE_BYTES } = {}) {
  const candidates = listCandidateFiles(root).filter((p) => classifyFile(p) !== null);
  const sizeProblems = [];
  const files = [];
  for (const p of candidates) {
    const full = join(root, p);
    const st = statSync(full);
    if (st.size > maxFileBytes) {
      sizeProblems.push(
        `${safeText(p)}: file size (${st.size} bytes) exceeds MAX_FILE_BYTES (${maxFileBytes} bytes)`,
      );
      continue;
    }
    files.push([p, readFileSync(full, "utf8")]);
  }
  const { problems, findings } = checkPatterns(files, allowlist);
  return { problems: [...sizeProblems, ...problems], findings, scanned: candidates.length };
}

function main() {
  const root = process.argv[2] ?? join(dirname(fileURLToPath(import.meta.url)), "..");
  const { problems, scanned } = checkRepo(root);
  if (problems.length > 0) {
    console.error('Weak test patterns found (docs/surec.md, "Zayıf test", #399):');
    for (const p of problems) console.error(`  ${p}`);
    process.exit(1);
  }
  const allowlisted = ALLOWLIST.reduce((sum, e) => sum + (e.count ?? 1), 0);
  console.log(`test patterns ok: ${scanned} files scanned, ${allowlisted} allowlisted`);
}

if (process.argv[1] && realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url))) {
  main();
}
