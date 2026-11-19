// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test tools/test-check-test-patterns.mjs

import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  ALLOWLIST,
  checkPatterns,
  checkRepo,
  maskCode,
  safeText,
  scanFile,
} from "./check-test-patterns.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");

test("clean Rust, Dart, JS and QML tests pass without findings", () => {
  const clean = [
    [
      "crates/totaldrop-core/src/example.rs",
      [
        "pub async fn production_sleep() {",
        "    tokio::time::sleep(std::time::Duration::from_secs(1)).await;",
        "}",
        "",
        "#[cfg(test)]",
        "mod tests {",
        "    use std::time::Duration;",
        "",
        "    #[tokio::test]",
        "    async fn waits_on_channel_and_polls_with_deadline() {",
        "        let (tx, rx) = tokio::sync::oneshot::channel::<u8>();",
        "        tokio::spawn(async move { let _ = tx.send(7); });",
        "        let got = tokio::time::timeout(Duration::from_secs(2), rx).await.unwrap().unwrap();",
        "        assert_eq!(got, 7);",
        "        let start = std::time::Instant::now();",
        "        let ready = std::sync::atomic::AtomicBool::new(true);",
        "        while !ready.load(std::sync::atomic::Ordering::SeqCst) {",
        "            tokio::time::sleep(Duration::from_millis(10)).await;",
        "            assert!(start.elapsed() < Duration::from_secs(2), \"timed out\");",
        "        }",
        "        for _ in 0..20 {",
        "            if ready.load(std::sync::atomic::Ordering::SeqCst) { break; }",
        "            tokio::time::sleep(Duration::from_millis(10)).await;",
        "        }",
        "        assert!(ready.load(std::sync::atomic::Ordering::SeqCst));",
        "    }",
        "",
        "    #[tokio::test(start_paused = true)]",
        "    async fn virtual_clock_sleep_is_deterministic() {",
        "        tokio::time::sleep(Duration::from_secs(30)).await;",
        "        assert_eq!(1, 1);",
        "    }",
        "",
        "    #[tokio::test]",
        "    async fn explicit_pause_before_sleep_is_deterministic() {",
        "        tokio::time::pause();",
        "        tokio::time::sleep(Duration::from_secs(30)).await;",
        "        assert_eq!(2, 2);",
        "    }",
        "}",
      ].join("\n"),
    ],
    [
      "app/test/clean_widget_test.dart",
      [
        "import 'package:flutter/widgets.dart';",
        "import 'package:flutter_test/flutter_test.dart';",
        "",
        "void main() {",
        "  testWidgets('specific error matchers and synthetic pump pass', (tester) async {",
        "    await tester.pumpAndSettle();",
        "    await tester.pump(const Duration(milliseconds: 300));",
        "    expect(tester.takeException(), isNull);",
        "    expect(",
        "      tester.takeException(),",
        "      isA<FlutterError>().having((e) => e.toString(), 'message', contains('overflowed')),",
        "    );",
        "    expect(() => throw StateError('bad'), throwsStateError);",
        "    expect(() => throw ArgumentError('x'), throwsA(isA<ArgumentError>()));",
        "  });",
        "}",
      ].join("\n"),
    ],
    [
      "tools/test-clean.mjs",
      [
        'import { test } from "node:test";',
        'import assert from "node:assert/strict";',
        'import { setTimeout } from "node:timers/promises";',
        "",
        'test("specific error checks and deadline-guarded while loop pass", async () => {',
        '  assert.throws(() => { throw new Error("bad input"); }, /bad input/);',
        '  assert.throws(() => { throw new TypeError("kind"); }, TypeError);',
        '  await assert.rejects(async () => { throw new Error("boom"); }, (e) => e.message === "boom");',
        "  const deadline = Date.now() + 2000;",
        "  let ready = true;",
        "  while (!ready) {",
        "    await setTimeout(10);",
        "    assert.ok(Date.now() < deadline);",
        "  }",
        "  assert.equal(ready, true);",
        "});",
      ].join("\n"),
    ],
    [
      "linux/tests/qml/tst_clean.qml",
      [
        "import QtQuick",
        "import QtTest",
        "TestCase {",
        "    function test_ready() {",
        "        tryCompare(item, 'opacity', 1.0, 1000)",
        "        verify(item.visible)",
        "    }",
        "}",
      ].join("\n"),
    ],
  ];
  const result = checkPatterns(clean, []);
  assert.deepEqual(result.problems, []);
  assert.deepEqual(result.findings, []);
});

test("catches catch-all error matchers in Dart unit and integration tests (#316)", () => {
  const dart = [
    "import 'package:flutter_test/flutter_test.dart';",
    "void main() {",
    "  testWidgets('overflow check', (tester) async {",
    "    expect(",
    "      tester.takeException(),",
    "      isNotNull,",
    "    );",
    "  });",
    "  it('any thrown error', () {",
    "    expect(() => doWork(), throwsA(anything));",
    "    expect(() => doWork(), throwsA(isNotNull));",
    "    expect(() => doWork(), throwsException);",
    "    expect(() => doWork(), throwsA(isException));",
    "    expect(() => doWork(), throwsA(isA<Exception>()));",
    "    expect(() => doWork(), throwsA(isA<Error>()));",
    "  });",
    "}",
  ].join("\n");

  const findings = scanFile("app/integration_test/weak_error_test.dart", dart);
  assert.deepEqual(
    findings.map((f) => [f.test, f.rule, f.line]),
    [
      ["overflow check", "any-error-matcher", 4],
      ["any thrown error", "any-error-matcher", 10],
      ["any thrown error", "any-error-matcher", 11],
      ["any thrown error", "any-error-matcher", 12],
      ["any thrown error", "any-error-matcher", 13],
      ["any thrown error", "any-error-matcher", 14],
      ["any thrown error", "any-error-matcher", 15],
    ],
  );
  const { problems } = checkPatterns([["app/integration_test/weak_error_test.dart", dart]], []);
  assert.equal(problems.length, 7);
  assert.match(
    problems[0],
    /app\/integration_test\/weak_error_test\.dart:4 \(overflow check\): \[any-error-matcher\]/,
  );
});

test("catches bare, null, trivial-regex and catch-all assert.throws/rejects in JS/TS tests", () => {
  const js = [
    'import { it } from "node:test";',
    'import assert from "node:assert/strict";',
    'it("weak js throws", async () => {',
    "  assert.throws(() => parse('x'));",
    "  assert.throws(() => parse('y'), undefined, 'context');",
    "  assert.throws(() => parse('z'), null);",
    "  assert.throws(() => parse('w'), /.*/);",
    "  assert.throws(() => parse('v'), /(?:)/);",
    "  await assert.rejects(async () => load(), () => true);",
    "});",
  ].join("\n");

  const findings = scanFile("tools/test-weak.mjs", js);
  assert.deepEqual(
    findings.map((f) => [f.test, f.rule, f.line]),
    [
      ["weak js throws", "any-error-matcher", 4],
      ["weak js throws", "any-error-matcher", 5],
      ["weak js throws", "any-error-matcher", 6],
      ["weak js throws", "any-error-matcher", 7],
      ["weak js throws", "any-error-matcher", 8],
      ["weak js throws", "any-error-matcher", 9],
    ],
  );
});

test("catches fixed sleep followed by assert or panic in Rust, Dart, JS and QML tests (#395)", () => {
  const rustUnit = [
    "#[cfg(test)]",
    "mod tests {",
    "    use std::time::Duration;",
    "",
    "    #[tokio::test]",
    "    async fn waits_with_tokio_sleep() {",
    "        tokio::time::sleep(Duration::from_secs(5)).await;",
    "        assert_eq!(1 + 1, 2);",
    "    }",
    "",
    "    #[test]",
    "    fn waits_with_thread_sleep() {",
    "        std::thread::sleep(Duration::from_millis(50));",
    "        assert!(true);",
    "    }",
    "",
    "    #[tokio::test]",
    "    async fn spawned_fixed_sleep_before_assert() {",
    "        let moved = tokio::spawn(async move {",
    "            tokio::time::sleep(Duration::from_secs(5)).await;",
    "            true",
    "        });",
    "        assert!(moved.await.unwrap());",
    "    }",
    "",
    "    #[test]",
    "    fn table_for_loop_without_break_is_caught_even_when_assert_follows_loop() {",
    "        let mut got = Vec::new();",
    "        for n in [1, 2] {",
    "            thread::sleep(Duration::from_millis(20));",
    "            got.push(n);",
    "        }",
    "        assert_eq!(got, vec![1, 2]);",
    "    }",
    "",
    "    #[test]",
    "    fn while_true_without_real_condition_is_caught() {",
    "        let start = std::time::Instant::now();",
    "        while true {",
    "            thread::sleep(Duration::from_millis(10));",
    "            assert!(start.elapsed() < Duration::from_secs(1));",
    "        }",
    "    }",
    "",
    "    #[tokio::test]",
    "    async fn sleep_before_pause_is_still_caught() {",
    "        tokio::time::sleep(Duration::from_millis(25)).await;",
    "        tokio::time::pause();",
    "        tokio::time::sleep(Duration::from_secs(30)).await;",
    "        if false { panic!(\"unreachable\"); }",
    "    }",
    "",
    "    #[test]",
    "    fn impl_for_inside_test_is_not_mistaken_for_a_polling_loop() {",
    "        struct Held;",
    "        impl Drop for Held {",
    "            fn drop(&mut self) {",
    "                std::thread::sleep(Duration::from_millis(20));",
    "            }",
    "        }",
    "        let _h = Held;",
    "        assert!(true);",
    "    }",
    "}",
  ].join("\n");

  const rustFindings = scanFile("crates/totaldrop-core/src/sample.rs", rustUnit);
  assert.deepEqual(
    rustFindings.map((f) => [f.test, f.rule, f.line]),
    [
      ["waits_with_tokio_sleep", "sleep-before-assert", 7],
      ["waits_with_thread_sleep", "sleep-before-assert", 13],
      ["spawned_fixed_sleep_before_assert", "sleep-before-assert", 20],
      ["table_for_loop_without_break_is_caught_even_when_assert_follows_loop", "sleep-before-assert", 30],
      ["while_true_without_real_condition_is_caught", "sleep-before-assert", 40],
      ["sleep_before_pause_is_still_caught", "sleep-before-assert", 47],
      ["impl_for_inside_test_is_not_mistaken_for_a_polling_loop", "sleep-before-assert", 58],
    ],
  );

  const dartSleep = [
    "import 'package:flutter_test/flutter_test.dart';",
    "void main() {",
    "  test('waits with Future.delayed', () async {",
    "    await Future<void>.delayed(const Duration(milliseconds: 250));",
    "    expect(ready, isTrue);",
    "  });",
    "}",
  ].join("\n");
  const dartFindings = scanFile("app/test/timing_test.dart", dartSleep);
  assert.deepEqual(
    dartFindings.map((f) => [f.test, f.rule, f.line]),
    [["waits with Future.delayed", "sleep-before-assert", 4]],
  );

  const qmlWait = [
    "import QtTest",
    "TestCase {",
    "    function test_delayed() {",
    "        wait(200)",
    "        compare(count, 1)",
    "    }",
    "}",
  ].join("\n");
  const qmlFindings = scanFile("linux/tests/qml/tst_wait.qml", qmlWait);
  assert.deepEqual(
    qmlFindings.map((f) => [f.test, f.rule, f.line]),
    [["test_delayed", "sleep-before-assert", 4]],
  );
});

test("comments, quote-bearing char/RegExp literals, C-strings, emojis and lifetimes do not flip quote polarity", () => {
  const rust = [
    "#[cfg(test)]",
    "mod tests {",
    "    // 🚀 tokio::time::sleep(Duration::from_secs(5)).await;",
    "    /* outer /* nested std::thread::sleep(Duration::from_secs(1)); */ still comment */",
    "    fn helper<'a>(text: &'a str) -> &'a str { text.trim_matches('\"') }",
    "",
    "    #[test]",
    "    fn quote_chars_c_strings_and_emojis_preserve_masking() {",
    "        let emoji = \"🚀\";",
    "        let dq = '\"';",
    "        let bdq = b'\"';",
    "        let cstr = c\"tokio::time::sleep(Duration::from_secs(5)).await;\";",
    "        let crstr = cr#\"std::thread::sleep(Duration::from_millis(10));\"#;",
    "        let quote = '\\'';",
    "        assert_eq!(helper(emoji), \"🚀\");",
    "        assert_eq!((dq, bdq as char, quote), ('\"', '\"', '\\''));",
    "        assert!(!cstr.to_bytes().is_empty() && !crstr.to_bytes().is_empty());",
    "    }",
    "",
    "    #[test]",
    "    fn violation_after_quote_char_literal_is_still_detected() {",
    "        let trimmed = \"\\\"hi\\\"\".trim_matches('\"');",
    "        std::thread::sleep(std::time::Duration::from_millis(10));",
    "        assert_eq!(trimmed, \"hi\");",
    "    }",
    "}",
  ].join("\n");

  const rustFindings = scanFile("crates/totaldrop-core/src/quotes.rs", rust);
  assert.deepEqual(
    rustFindings.map((f) => [f.test, f.rule, f.line]),
    [["violation_after_quote_char_literal_is_still_detected", "sleep-before-assert", 23]],
  );
  assert.equal(maskCode(rust, "rs").length, rust.length);

  const jsWithOddQuoteRegexes = [
    'import { test } from "node:test";',
    'import assert from "node:assert/strict";',
    'test("regexes with odd quotes, apostrophes and parens keep polarity", () => {',
    '  const icon = "🚀";',
    '  const re1 = /applicationId\\s*=\\s*"([^"]+)"/;',
    "  const re2 = /can't [/] break/gi;",
    "  const re3 = /unclosed \\(/;",
    "  assert.ok(icon.length > 0 && re1 && re2 && re3);",
    "  assert.throws(() => broken());",
    "});",
  ].join("\n");
  assert.equal(maskCode(jsWithOddQuoteRegexes, "js").length, jsWithOddQuoteRegexes.length);
  const jsFindings = scanFile("site/tools/test-quotes.mjs", jsWithOddQuoteRegexes);
  assert.deepEqual(
    jsFindings.map((f) => [f.test, f.rule, f.line]),
    [["regexes with odd quotes, apostrophes and parens keep polarity", "any-error-matcher", 9]],
  );

  const dart = [
    "import 'package:flutter_test/flutter_test.dart';",
    "void main() {",
    "  /* outer /* nested Future.delayed(Duration(seconds: 1)); */ comment */",
    "  test('strings only', () {",
    "    const note = '🚀 Future.delayed(Duration(seconds: 1)); expect(tester.takeException(), isNotNull);';",
    "    const block = '''throwsA(anything)''';",
    "    expect(note.isNotEmpty, isTrue);",
    "    expect(block.isNotEmpty, isTrue);",
    "  });",
    "}",
  ].join("\n");
  assert.deepEqual(scanFile("app/test/comments_test.dart", dart), []);
});

test("cfg(test) on a struct field or statement does not leak into surrounding production code", () => {
  const rust = [
    "pub struct Counter {",
    "    #[cfg(test)]",
    "    pub seen: usize,",
    "    pub live: usize,",
    "}",
    "",
    "pub async fn run_work() {",
    "    #[cfg(test)]",
    "    let _ = 1;",
    "    if true {",
    "        tokio::time::sleep(std::time::Duration::from_millis(50)).await;",
    "        assert!(true);",
    "    }",
    "}",
  ].join("\n");
  assert.deepEqual(scanFile("crates/totaldrop-core/src/prod_only.rs", rust), []);
});

test("allowlisted occurrence with a valid reason passes, and extra unapproved occurrences still fail", () => {
  const rust = [
    "#[tokio::test]",
    "async fn stays_quiet_over_window() {",
    "    tokio::time::sleep(std::time::Duration::from_millis(200)).await;",
    "    assert!(true);",
    "}",
  ].join("\n");
  const allow = [
    {
      file: "crates/totaldrop-core/tests/negative.rs",
      test: "stays_quiet_over_window",
      rule: "sleep-before-assert",
      reason: "Negative check: proves no packet is sent during the 200 ms quiet window.",
    },
  ];
  assert.deepEqual(
    checkPatterns([["crates/totaldrop-core/tests/negative.rs", rust]], allow).problems,
    [],
  );

  const twoSleeps = rust.replace(
    "    assert!(true);",
    "    tokio::time::sleep(std::time::Duration::from_millis(50)).await;\n    assert!(true);",
  );
  const { problems } = checkPatterns([["crates/totaldrop-core/tests/negative.rs", twoSleeps]], allow);
  assert.equal(problems.length, 1);
  assert.match(problems[0], /allowlisted 1, found 2/);
});

test("allowlist entry with an empty or missing reason fails", () => {
  const rust = [
    "#[test]",
    "fn waits_briefly() {",
    "    std::thread::sleep(std::time::Duration::from_millis(10));",
    "    assert!(true);",
    "}",
  ].join("\n");
  for (const badReason of ["", "   ", undefined]) {
    const { problems } = checkPatterns(
      [["crates/totaldrop-core/tests/bad.rs", rust]],
      [
        {
          file: "crates/totaldrop-core/tests/bad.rs",
          test: "waits_briefly",
          rule: "sleep-before-assert",
          reason: badReason,
        },
      ],
    );
    assert.ok(
      problems.some((p) => /reason/.test(p)),
      `expected reason error for ${JSON.stringify(badReason)}, got ${JSON.stringify(problems)}`,
    );
  }
});

test("stale allowlist entry (no longer present in code) fails", () => {
  const cleanRust = [
    "#[test]",
    "fn now_uses_channel() {",
    "    assert_eq!(2 + 2, 4);",
    "}",
  ].join("\n");
  const { problems } = checkPatterns(
    [["crates/totaldrop-core/tests/clean.rs", cleanRust]],
    [
      {
        file: "crates/totaldrop-core/tests/clean.rs",
        test: "now_uses_channel",
        rule: "sleep-before-assert",
        reason: "Used to sleep before switching to a channel.",
      },
    ],
  );
  assert.equal(problems.length, 1);
  assert.match(problems[0], /stale/);
});

test("safeText escapes backslashes, control/Bidi chars, and truncates without splitting surrogates", () => {
  const dirty = "a\\b\x1b[31mname\u202eevil\nnext";
  const cleaned = safeText(dirty);
  assert.ok(!/[\p{Cc}\p{Cf}\p{Zl}\p{Zp}]/u.test(cleaned));
  assert.match(cleaned, /^a\\\\b\\x1b\[31mname\\u\{202e\}evil\\x0anext$/);

  const astral = "x🚀y";
  const truncated = safeText(astral, 2);
  assert.equal(truncated, "x🚀…");
});

test("checkRepo walks scratch directories without .git and fails closed on oversized files", () => {
  const scratch = mkdtempSync(join(tmpdir(), "test-patterns-scratch-"));
  try {
    mkdirSync(join(scratch, "tools"), { recursive: true });
    writeFileSync(
      join(scratch, "tools", "test-big.mjs"),
      'import assert from "node:assert/strict";\nassert.equal(1, 1);\n',
    );
    assert.deepEqual(checkRepo(scratch, []).problems, []);
    const oversized = checkRepo(scratch, [], { maxFileBytes: 10 });
    assert.equal(oversized.problems.length, 1);
    assert.match(oversized.problems[0], /exceeds MAX_FILE_BYTES/);
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
});

test("this repository passes with every ALLOWLIST entry justified and active, including via symlink", () => {
  for (const entry of ALLOWLIST) {
    assert.equal(typeof entry.reason, "string");
    assert.ok(entry.reason.trim().length > 0, `empty reason on ${entry.file}:${entry.test}`);
  }
  const repoResult = checkRepo(root);
  assert.deepEqual(repoResult.problems, []);
  assert.ok(repoResult.scanned > 40, `expected >40 test files, scanned ${repoResult.scanned}`);

  const dir = mkdtempSync(join(tmpdir(), "test-patterns-link-"));
  try {
    symlinkSync(realpathSync(root), join(dir, "repo"));
    const out = execFileSync("node", [join(dir, "repo", "tools", "check-test-patterns.mjs")], {
      encoding: "utf8",
    });
    assert.match(out, /^test patterns ok: \d+ files scanned, \d+ allowlisted$/m);

    const sb = join(dir, "sandbox");
    mkdirSync(join(sb, "tools"), { recursive: true });
    mkdirSync(join(sb, "app", "test"), { recursive: true });
    cpSync(join(here, "check-test-patterns.mjs"), join(sb, "tools", "check-test-patterns.mjs"));
    writeFileSync(
      join(sb, "app", "test", "bad_test.dart"),
      "void main() { test('bad', () { expect(tester.takeException(), isNotNull); }); }\n",
    );
    assert.throws(
      () =>
        execFileSync("node", [join(sb, "tools", "check-test-patterns.mjs")], {
          encoding: "utf8",
          stdio: ["ignore", "pipe", "pipe"],
        }),
      /any-error-matcher/,
    );
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
