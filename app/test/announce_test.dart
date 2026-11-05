// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// When the screen changes under the person, the button they were on goes
// away and a screen reader says nothing. Each new state's headline is a
// live region, so TalkBack reads it out (#271). A picker that fails says
// so instead of dropping back to an empty selection without a word.
import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/src/rust/frb_generated.dart';
import 'package:totaldrop/ui/core/theme.dart';
import 'package:totaldrop/ui/receive/receive_view.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

import 'fakes.dart';

final l = lookupAppLocalizations(const Locale('en'));

/// The English text of the new message; the key is added with it.
const addFailed =
    'The files could not be added. Make sure there is enough free space '
    'on this device and try again.';

Widget app(Widget home) => MaterialApp(
  theme: buildTheme(fallbackScheme(Brightness.light)),
  localizationsDelegates: const [
    AppLocalizations.delegate,
    ...GlobalMaterialLocalizations.delegates,
  ],
  supportedLocales: AppLocalizations.supportedLocales,
  builder: (context, child) => SystemMotion(child: child!),
  home: home,
);

/// Lets the screen switch finish; the turning mark never settles.
Future<void> settle(WidgetTester tester) async {
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 300));
}

void expectAnnounced(WidgetTester tester, String text) {
  expect(find.text(text), findsOneWidget);
  expect(
    tester.getSemantics(find.text(text)),
    isSemantics(label: text, isLiveRegion: true),
    reason: '"$text" is read out when it appears',
  );
}

TransferEvent finished() => const TransferEvent(
  kind: EventKind.finished,
  words: [],
  done: 10,
  total: 10,
  summary: TransferSummary(filesOk: 1, filesFailed: 0, bytes: 10, saved: []),
  error: null,
);

TransferEvent failed() => const TransferEvent(
  kind: EventKind.failed,
  words: [],
  done: 0,
  total: 0,
  summary: null,
  error: TransferError(kind: ErrorKind.other, detail: '', cause: ''),
);

/// Stands in for the Rust library: the offer asks it to make hidden
/// characters in file names visible, which test names have none of, and
/// whether a security text must stay English, which it need not in English.
class _Rust implements RustLibApi {
  @override
  String crateApiL10NRevealControls({required String name}) => name;

  @override
  bool crateApiL10NNeedsEnglish({required String tag, required String key}) =>
      false;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  setUpAll(() => RustLib.initMock(api: _Rust()));

  group('a failed pick', () {
    late FakeFileSource source;
    late SendViewModel vm;

    setUp(() {
      source = FakeFileSource();
      vm = SendViewModel(repository: FakeRepository(), fileSource: source);
    });

    testWidgets('is said on the empty send screen', (tester) async {
      final semantics = tester.ensureSemantics();
      await tester.pumpWidget(app(SendView(viewModel: vm)));
      await tester.tap(find.text(l.sendAddFiles));
      source.startCopy();
      await settle(tester);
      source.fail(Exception('unknown_path'));
      await settle(tester);
      expect(vm.copying, isFalse);
      expect(vm.files, isEmpty);
      expect(find.text(l.sendAddFiles), findsOneWidget);
      expectAnnounced(tester, addFailed);
      expect(tester.takeException(), isNull);
      semantics.dispose();
    });

    testWidgets('is said next to files already chosen', (tester) async {
      final semantics = tester.ensureSemantics();
      vm.addFakeFileForTest('a.txt');
      await tester.pumpWidget(app(SendView(viewModel: vm)));
      await tester.tap(find.text(l.sendAddFiles));
      source.fail(Exception('out_of_memory'));
      await settle(tester);
      expect(find.text('a.txt'), findsOneWidget);
      expectAnnounced(tester, addFailed);
      semantics.dispose();
    });

    testWidgets('is cleared by the next pick and by a reset', (tester) async {
      await tester.pumpWidget(app(SendView(viewModel: vm)));
      await tester.tap(find.text(l.sendAddFiles));
      source.fail(Exception('unknown_path'));
      await settle(tester);
      expect(find.text(addFailed), findsOneWidget);

      await tester.tap(find.text(l.sendAddFiles));
      await settle(tester);
      expect(find.text(addFailed), findsNothing, reason: 'trying again');
      source.finish([pickedFile('film.mkv')]);
      await settle(tester);
      expect(find.text('film.mkv'), findsOneWidget);

      await tester.tap(find.text(l.sendAddFiles));
      source.fail(Exception('unknown_path'));
      await settle(tester);
      expect(find.text(addFailed), findsOneWidget);
      vm.reset();
      await settle(tester);
      expect(find.text(addFailed), findsNothing);
    });

    test('that returns after a reset says nothing', () async {
      final added = vm.addFiles();
      source.startCopy();
      vm.reset();
      source.fail(Exception('unknown_path'));
      await added;
      expect(vm.addFailed, isFalse);
    });

    test('a cancelled pick is not a failure', () async {
      final added = vm.addFiles();
      source.finish(const []);
      await added;
      expect(vm.addFailed, isFalse);
    });
  });

  // The fakes are made inside each test: a future made in setUp
  // completes outside the test's fake clock, and pump never sees it.
  group('the send screen reads out', () {
    late FakeFileSource source;
    late FakeRepository repo;
    late SendViewModel vm;

    void make() {
      source = FakeFileSource();
      repo = FakeRepository();
      vm = SendViewModel(repository: repo, fileSource: source);
    }

    testWidgets('the copy of the chosen files', (tester) async {
      final semantics = tester.ensureSemantics();
      make();
      await tester.pumpWidget(app(SendView(viewModel: vm)));
      await tester.tap(find.text(l.sendAddFiles));
      source.startCopy();
      await settle(tester);
      expectAnnounced(tester, l.sendPreparing);
      source.finish([pickedFile('film.mkv')]);
      await settle(tester);
      semantics.dispose();
    });

    testWidgets('each stage of a send', (tester) async {
      final semantics = tester.ensureSemantics();
      make();
      vm.addFakeFileForTest('a.txt');
      await tester.pumpWidget(app(SendView(viewModel: vm)));
      unawaited(vm.start());
      await settle(tester);
      expectAnnounced(tester, l.sendPreparing);

      repo.prepared.complete(FakeSession());
      await settle(tester);
      expectAnnounced(tester, l.sendWaiting);

      repo.events.add(event(EventKind.connected));
      await settle(tester);
      expectAnnounced(tester, l.sendSending);

      repo.events.add(finished());
      await settle(tester);
      expectAnnounced(tester, l.sendDone);
      semantics.dispose();
    });

    testWidgets('a failed send', (tester) async {
      final semantics = tester.ensureSemantics();
      make();
      vm.addFakeFileForTest('a.txt');
      await tester.pumpWidget(app(SendView(viewModel: vm)));
      unawaited(vm.start());
      repo.prepared.complete(FakeSession());
      await settle(tester);
      repo.events.add(failed());
      await settle(tester);
      expectAnnounced(tester, l.errorOther);
      semantics.dispose();
    });
  });

  group('the receive screen reads out', () {
    late FakeRepository repo;
    late ReceiveViewModel vm;

    void make() {
      repo = FakeRepository();
      vm = ReceiveViewModel(repository: repo);
      vm.setInput('7-harbor-rocket');
    }

    testWidgets('each stage of a receive', (tester) async {
      final semantics = tester.ensureSemantics();
      make();
      await tester.pumpWidget(app(ReceiveView(viewModel: vm)));
      unawaited(vm.connect());
      await settle(tester);
      expectAnnounced(tester, l.receiveConnecting);

      repo.connected.complete(FakeOffer());
      await settle(tester);
      expectAnnounced(tester, l.receiveOffer);

      unawaited(vm.accept());
      await settle(tester);
      expectAnnounced(tester, l.receiveReceiving);

      repo.events.add(finished());
      await settle(tester);
      expectAnnounced(tester, l.receiveDone);
      semantics.dispose();
    });

    testWidgets('a declined offer', (tester) async {
      final semantics = tester.ensureSemantics();
      make();
      await tester.pumpWidget(app(ReceiveView(viewModel: vm)));
      unawaited(vm.connect());
      repo.connected.complete(FakeOffer());
      await settle(tester);
      await vm.decline();
      await settle(tester);
      expectAnnounced(tester, l.receiveDeclined);
      semantics.dispose();
    });

    testWidgets('a failed receive', (tester) async {
      final semantics = tester.ensureSemantics();
      make();
      await tester.pumpWidget(app(ReceiveView(viewModel: vm)));
      unawaited(vm.connect());
      repo.connected.completeError(
        const TransferError(kind: ErrorKind.other, detail: '', cause: ''),
      );
      await settle(tester);
      expectAnnounced(tester, l.errorOther);
      semantics.dispose();
    });
  });
}
