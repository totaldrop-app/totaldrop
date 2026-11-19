// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// node --test tools/test-check-state-machines.mjs

import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, symlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import {
  ALLOWLIST,
  checkStateMachines,
  loadAppDartFiles,
  sanitizeForLog,
  tokenizeDart,
} from "./check-state-machines.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");

function checkSingle(dartSource, options = {}) {
  return checkStateMachines(
    { "app/lib/ui/sample_view_model.dart": dartSource },
    { allowlist: {}, ...options },
  );
}

test("clean ChangeNotifier with enum stage, pre-await snapshot, guarded catch/finally, and dispose invalidation passes", () => {
  const { problems, checkedClasses } = checkSingle(`
    enum SampleStage { idle, loading, done, failed }

    class SampleViewModel extends ChangeNotifier {
      int _generation = 0;
      int _resets = 0;
      bool _disposed = false;
      SampleStage stage = SampleStage.idle;
      final List<String> _items = [];

      Future<void> load() async {
        stage = SampleStage.loading;
        final generation = ++_generation;
        final resets = _resets;
        notifyListeners();
        try {
          final item = await repo.fetch();
          if (generation != _generation || resets != _resets) return;
          _items.add(item);
          stage = SampleStage.done;
        } catch (e) {
          if (generation != _generation) return;
          stage = SampleStage.failed;
        } finally {
          if (!_disposed) {
            notifyListeners();
          }
        }
      }

      Future<void> bestEffortTellPeer() async {
        stage = SampleStage.idle;
        notifyListeners();
        try {
          await repo.tellPeer();
        } catch (_) {}
      }

      @override
      void dispose() {
        _disposed = true;
        _generation++;
        _resets++;
        super.dispose();
      }
    }
  `);
  assert.equal(checkedClasses, 1);
  assert.deepEqual(problems, []);
});

test("class mixing in `with ChangeNotifier` is checked just like `extends ChangeNotifier`", () => {
  const { problems, checkedClasses } = checkSingle(`
    class MixinNotifier with ChangeNotifier {
      String stage = 'idle';
    }
  `);
  assert.equal(checkedClasses, 1);
  assert.equal(problems.length, 1);
  assert.match(problems[0], /MixinNotifier\.stage has type `String`/);
});

test("Check 1 rejects String, bool, dynamic, Object, untyped stage fields, and non-enum stage getters", () => {
  const { problems } = checkSingle(`
    class BadStages extends ChangeNotifier {
      String stage = 'idle';
      bool phase = false;
      dynamic state;
      Object? status;
      var _stage = 'entering';
      String get _status => 'ok';
    }
  `);
  assert.equal(problems.length, 6);
  assert.match(problems[0], /BadStages\.stage has type `String`/);
  assert.match(problems[1], /BadStages\.phase has type `bool`/);
  assert.match(problems[2], /BadStages\.state has type `dynamic`/);
  assert.match(problems[3], /BadStages\.status has type `Object\?`/);
  assert.match(problems[4], /BadStages\._stage has type `untyped`/);
  assert.match(problems[5], /BadStages\._status has type `String`/);
});

test("Check 2 catches pasteClipboard-style async mutation/setText/notifyListeners without pre-await snapshot", () => {
  const { problems } = checkSingle(`
    enum SendStage { picking, sending }
    enum TextNotice { none, clipboardEmpty, clipboardKept }

    class BuggySend extends ChangeNotifier {
      int _generation = 0;
      int _resets = 0;
      SendStage stage = SendStage.picking;
      bool textMode = false;
      String _text = '';
      TextNotice textNotice = TextNotice.none;

      void setText(String v) {
        _text = v;
        notifyListeners();
      }

      Future<void> pasteClipboard() async {
        textMode = true;
        notifyListeners();
        final clip = await clipboard.read();
        if (!textMode) return;
        if (_text.isNotEmpty) {
          textNotice = TextNotice.clipboardKept;
        } else if (clip == null) {
          textNotice = TextNotice.clipboardEmpty;
        } else {
          setText(clip);
          return;
        }
        notifyListeners();
      }

      @override
      void dispose() {
        _generation++;
        _resets++;
        super.dispose();
      }
    }
  `);
  assert.ok(
    problems.some((p) =>
      p.includes("BuggySend.pasteClipboard() awaits and mutates state or calls notifyListeners() afterward without snapshotting"),
    ),
  );
  assert.ok(problems.some((p) => p.includes("`textNotice =`")));
  assert.ok(problems.some((p) => p.includes("`setText()`")));
  assert.ok(problems.some((p) => p.includes("`notifyListeners()`")));
});

test("Check 2 catches unguarded `finally { notifyListeners(); }` even when try and catch check resets", () => {
  const { problems } = checkSingle(`
    class BuggyAddFiles extends ChangeNotifier {
      int _resets = 0;
      bool _picking = false;
      bool copying = false;
      bool addFailed = false;
      final List<String> _files = [];

      Future<void> addFiles() async {
        _picking = true;
        final resets = _resets;
        try {
          final picked = await source.pick();
          if (resets != _resets) return;
          _files.add(picked);
        } on Exception {
          if (resets == _resets) addFailed = true;
        } finally {
          _picking = false;
          copying = false;
          notifyListeners();
        }
      }

      @override
      void dispose() {
        _resets++;
        super.dispose();
      }
    }
  `);
  assert.deepEqual(problems, [
    "app/lib/ui/sample_view_model.dart:19: BuggyAddFiles.addFiles() reaches `_picking =` after await without a generation/reset/_disposed guard",
    "app/lib/ui/sample_view_model.dart:20: BuggyAddFiles.addFiles() reaches `copying =` after await without a generation/reset/_disposed guard",
    "app/lib/ui/sample_view_model.dart:21: BuggyAddFiles.addFiles() reaches `notifyListeners()` after await without a generation/reset/_disposed guard",
  ]);
});

test("Check 2 catches second await not re-checked, unguarded catch, same-statement await assignment, post-await snapshot, and && instead of ||", () => {
  const { problems } = checkSingle(`
    enum S { a, b }

    class SubtleBugs extends ChangeNotifier {
      int _generation = 0;
      S stage = S.a;
      bool busy = false;

      void _fail() {
        stage = S.b;
        notifyListeners();
      }

      Future<void> missingSecondCheck() async {
        final g = ++_generation;
        await stepOne();
        if (g != _generation) return;
        await stepTwo();
        stage = S.b;
      }

      Future<void> unguardedCatch() async {
        final g = ++_generation;
        try {
          await stepOne();
          if (g != _generation) return;
          stage = S.b;
        } catch (_) {
          _fail();
        }
      }

      Future<void> inlineMutation() async {
        final g = ++_generation;
        stage = await fetchStage();
        if (g != _generation) return;
      }

      Future<void> snapshotTooLate() async {
        await stepOne();
        final g = _generation;
        if (g != _generation) return;
        notifyListeners();
      }

      Future<void> wrongBooleanOperator() async {
        final g = ++_generation;
        await stepOne();
        if (g != _generation && busy) return;
        notifyListeners();
      }

      @override
      void dispose() {
        _generation++;
        super.dispose();
      }
    }
  `);
  assert.ok(problems.some((p) => p.includes("SubtleBugs.missingSecondCheck() reaches `stage =`")));
  assert.ok(problems.some((p) => p.includes("SubtleBugs.unguardedCatch() reaches `_fail()`")));
  assert.ok(problems.some((p) => p.includes("SubtleBugs.inlineMutation() reaches `stage =`")));
  assert.ok(
    problems.some((p) =>
      p.includes("SubtleBugs.snapshotTooLate() awaits and mutates state or calls notifyListeners() afterward without snapshotting"),
    ),
  );
  assert.ok(problems.some((p) => p.includes("SubtleBugs.wrongBooleanOperator() reaches `notifyListeners()`")));
});

test("Check 2 catches unguarded await and mutations inside Dart 3 switch expressions and map/set literals", () => {
  const { problems } = checkSingle(`
    enum Mode { text, files }
    enum Stage { idle, done }

    class SwitchAndMapBugs extends ChangeNotifier {
      int _generation = 0;
      Stage stage = Stage.idle;

      Future<void> switchAwait(Mode m) async {
        final g = ++_generation;
        final x = switch (m) {
          Mode.text => await stepText(),
          Mode.files => await stepFiles(),
        };
        use(x);
        stage = Stage.done;
      }

      Future<void> recordSwitchAwait((int, int) pair) async {
        final g = ++_generation;
        final x = switch (pair) {
          (final a, final b) => await stepPair(a, b),
        };
        use(x);
        notifyListeners();
      }

      Future<void> mapLiteralAwait() async {
        final g = ++_generation;
        await warmUp();
        if (g != _generation) return;
        final payload = {'k': await step()};
        use(payload);
        stage = Stage.done;
      }

      Future<void> mutationInsideSwitchAndMap(Mode m) async {
        final g = ++_generation;
        await warmUp();
        final _ = switch (m) {
          Mode.text => stage = Stage.done,
          Mode.files => Stage.idle,
        };
        final map = {'sideEffect': notifyListeners()};
        use(map);
        if (g != _generation) return;
      }

      Future<void> guardedSwitchPasses(Mode m) async {
        final g = ++_generation;
        final x = switch (m) {
          Mode.text => await stepText(),
          Mode.files => await stepFiles(),
        };
        if (g != _generation) return;
        use(x);
        stage = Stage.done;
        notifyListeners();
      }

      @override
      void dispose() {
        _generation++;
        super.dispose();
      }
    }
  `);
  assert.deepEqual(problems, [
    "app/lib/ui/sample_view_model.dart:9: SwitchAndMapBugs.switchAwait() snapshots `g` before await but never checks it after await",
    "app/lib/ui/sample_view_model.dart:16: SwitchAndMapBugs.switchAwait() reaches `stage =` after await without a generation/reset/_disposed guard",
    "app/lib/ui/sample_view_model.dart:19: SwitchAndMapBugs.recordSwitchAwait() snapshots `g` before await but never checks it after await",
    "app/lib/ui/sample_view_model.dart:25: SwitchAndMapBugs.recordSwitchAwait() reaches `notifyListeners()` after await without a generation/reset/_disposed guard",
    "app/lib/ui/sample_view_model.dart:34: SwitchAndMapBugs.mapLiteralAwait() reaches `stage =` after await without a generation/reset/_disposed guard",
    "app/lib/ui/sample_view_model.dart:41: SwitchAndMapBugs.mutationInsideSwitchAndMap() reaches `stage =` after await without a generation/reset/_disposed guard",
    "app/lib/ui/sample_view_model.dart:44: SwitchAndMapBugs.mutationInsideSwitchAndMap() reaches `notifyListeners()` after await without a generation/reset/_disposed guard",
  ]);
});

test("Check 2 rejects an unused pre-await snapshot that is never checked after await", () => {
  const { problems } = checkSingle(`
    class UnusedSnapshot extends ChangeNotifier {
      int _generation = 0;
      bool _disposed = false;

      Future<void> run() async {
        final g = ++_generation;
        await step();
        if (_disposed) return;
        notifyListeners();
      }

      @override
      void dispose() {
        _disposed = true;
        _generation++;
        super.dispose();
      }
    }
  `);
  assert.equal(problems.length, 1);
  assert.match(problems[0], /snapshots `g` before await but never checks it after await/);
});

test("Check 3 catches missing dispose() or missing _generation++ / _resets++ / _disposed = true in dispose()", () => {
  const { problems } = checkSingle(`
    class MissingIncrements extends ChangeNotifier {
      int _generation = 0;
      int _resets = 0;
      bool _disposed = false;

      @override
      void dispose() {
        super.dispose();
      }
    }

    class NoDisposeAtAll extends ChangeNotifier {
      int _generation = 0;
    }
  `);
  assert.deepEqual(problems, [
    "app/lib/ui/sample_view_model.dart:8: MissingIncrements.dispose() must increment _generation (e.g. _generation++) so in-flight async calls see a changed generation after disposal",
    "app/lib/ui/sample_view_model.dart:8: MissingIncrements.dispose() must increment _resets (e.g. _resets++) so in-flight async calls see a changed generation after disposal",
    "app/lib/ui/sample_view_model.dart:8: MissingIncrements.dispose() must set _disposed = true",
    "app/lib/ui/sample_view_model.dart:13: NoDisposeAtAll declares _generation but defines no dispose() method to invalidate in-flight async calls",
  ]);
});

test("ALLOWLIST requires non-empty reason and fails on stale entries", () => {
  const cleanCode = `
    class CleanPrefs extends ChangeNotifier {
      Future<void> save() async {
        await prefs.write();
        notifyListeners();
      }
    }
  `;

  const emptyReason = checkSingle(cleanCode, {
    allowlist: { "CleanPrefs.save": { reason: "   " } },
  });
  assert.deepEqual(emptyReason.problems, [
    "ALLOWLIST[CleanPrefs.save]: allowlist entry must carry a non-empty reason",
  ]);

  const stale = checkSingle(cleanCode, {
    allowlist: {
      "CleanPrefs.save": { reason: "app-lifetime preferences singleton" },
      "CleanPrefs.deletedMethod": { reason: "old method" },
    },
  });
  assert.deepEqual(stale.problems, [
    "ALLOWLIST[CleanPrefs.deletedMethod]: stale allowlist entry (method no longer exists or already passes Check 2); remove it from ALLOWLIST",
  ]);
});

test("comments, nested block comments, raw strings, and string interpolations do not fool the lexer", () => {
  const { problems, checkedClasses } = checkSingle(`
    /* Outer comment /* nested class Fake extends ChangeNotifier { String stage = 'bad'; } */ still comment */
    // class CommentedOut extends ChangeNotifier { String stage = 'bad'; }
    enum RealStage { idle, done }

    class Quoted extends ChangeNotifier {
      int _generation = 0;
      RealStage stage = RealStage.idle;

      Future<void> run(Object err) async {
        final g = ++_generation;
        final msg = 'not an await: \${err.toString() == "x" ? "a" : "b"}';
        final raw = r'''await notifyListeners(); stage = 'bad';''';
        await work(msg, raw);
        if (g != _generation) return;
        stage = RealStage.done;
        notifyListeners();
      }

      @override
      void dispose() {
        _generation++;
        super.dispose();
      }
    }
  `);
  assert.equal(checkedClasses, 1);
  assert.deepEqual(problems, []);
});

test("generated l10n and bridge files are skipped for ChangeNotifier checks while bridge enums stay usable", () => {
  const { problems, checkedClasses } = checkStateMachines(
    {
      "app/lib/src/rust/api/transfer.dart": `
        enum BridgeStage { ready, running }
        class GeneratedNotifier extends ChangeNotifier {
          String stage = 'untyped_in_generated_code';
        }
      `,
      "app/lib/ui/bridge_consumer.dart": `
        class BridgeConsumer extends ChangeNotifier {
          BridgeStage stage = BridgeStage.ready;
        }
      `,
    },
    { allowlist: {} },
  );
  assert.equal(checkedClasses, 1);
  assert.deepEqual(problems, []);
});

test("sanitizeForLog injectively escapes backslashes, ASCII controls, and Unicode Bidi characters", () => {
  const raw = "Bad\u202EStage\n\x1b[31m\\u{202E}\u061C\u2066";
  const safe = sanitizeForLog(raw);
  assert.equal(safe, "Bad\\u{202E}Stage\\u{000A}\\u{001B}[31m\\\\u{202E}\\u{061C}\\u{2066}");
  assert.notEqual(sanitizeForLog("\u202E"), sanitizeForLog("\\u{202E}"));
});

test("lexer and analyzer run in linear time on large pathological input", () => {
  const repeated =
    "  final String s = '${a ? \"x\" : \"y\"}'; // comment\n".repeat(1500);
  const source = `
    enum FastStage { a, b }
    class FastNotifier extends ChangeNotifier {
      FastStage stage = FastStage.a;
      void noop() {
        ${repeated}
      }
    }
  `;
  const start = performance.now();
  const toks = tokenizeDart(source);
  const { problems } = checkSingle(source);
  const elapsedMs = performance.now() - start;
  assert.ok(toks.length > 15000);
  assert.deepEqual(problems, []);
  assert.ok(elapsedMs < 1000, `expected linear scan < 1000ms, took ${elapsedMs.toFixed(1)}ms`);
});

test("the real repository passes check-state-machines.mjs and has no stale allowlist entries", () => {
  const files = loadAppDartFiles(root);
  const { problems, checkedClasses } = checkStateMachines(files, { allowlist: ALLOWLIST });
  assert.deepEqual(problems, []);
  assert.ok(checkedClasses >= 4, `expected at least 4 ChangeNotifiers, found ${checkedClasses}`);

  const out = execFileSync("node", [join(here, "check-state-machines.mjs")], {
    encoding: "utf8",
  });
  assert.match(out, /^state machines ok: \d+ ChangeNotifier classes in app\/lib\//m);
});

test("CLI entrypoint runs when invoked through a symbolic link", () => {
  const dir = mkdtempSync(join(tmpdir(), "state-machines-link-"));
  try {
    const link = join(dir, "repo");
    symlinkSync(root, link);
    const out = execFileSync("node", [join(link, "tools", "check-state-machines.mjs")], {
      encoding: "utf8",
    });
    assert.match(out, /^state machines ok:/m);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
