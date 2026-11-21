// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Sending and receiving a text (#326, #53): what the send screen's text box
// holds, where the clipboard and a shared text go, that a text offer is
// received as a text, and how Share text and confirmed Open link behave.
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/l10n.dart' show ShownText;
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/src/rust/frb_generated.dart';
import 'package:totaldrop/ui/core/theme.dart';
import 'package:totaldrop/ui/receive/receive_view.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

import 'fakes.dart';

class _Rust implements RustLibApi {
  @override
  ShownText crateApiL10NShownText({required String text}) => ShownText(
    text: text.length > 16384 ? text.substring(0, 16384) : text,
    short: text.length > 16384,
    hidden: text.contains('\u202E'),
  );

  @override
  String? crateApiL10NSingleWebUrl({required String text}) => switch (text) {
    'https://totaldrop.app/docs' => 'https://totaldrop.app/docs',
    'https://münchen.de/rathaus' => 'https://xn--mnchen-3ya.de/rathaus',
    _ => null,
  };

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

Widget _receiveApp(
  ReceiveViewModel vm, {
  Future<void> Function(String text)? onShareText,
}) => MaterialApp(
  theme: buildTheme(fallbackScheme(Brightness.light)),
  localizationsDelegates: const [
    AppLocalizations.delegate,
    ...GlobalMaterialLocalizations.delegates,
  ],
  supportedLocales: AppLocalizations.supportedLocales,
  home: ReceiveView(viewModel: vm, onShareText: onShareText),
);

TransferEvent finished({int ok = 1, int failed = 0, String? text}) =>
    TransferEvent(
      kind: EventKind.finished,
      words: const [],
      done: 0,
      total: 0,
      summary: TransferSummary(
        filesOk: ok,
        filesFailed: failed,
        bytes: 1,
        saved: const [],
        text: text,
      ),
      error: null,
    );

void main() {
  setUpAll(() => RustLib.initMock(api: _Rust()));

  group('send', () {
    late FakeRepository repo;

    SendViewModel model({String? clipboard}) => SendViewModel(
      repository: repo,
      textClipboard: FakeClipboard(clipboard),
    );

    setUp(() => repo = FakeRepository());

    test('the clipboard goes into an empty box exactly', () async {
      final vm = model(clipboard: 'a\r\nb c ');
      await vm.pasteClipboard();
      expect(vm.textMode, isTrue);
      expect(vm.text, 'a\r\nb c ');
      expect(vm.textNotice, TextNotice.none);
      expect(vm.canStart, isTrue);
    });

    test('typed text is kept, and the screen says so', () async {
      final vm = model(clipboard: 'other')
        ..setTextMode(true)
        ..setText('mine');
      await vm.pasteClipboard();
      expect(vm.text, 'mine');
      expect(vm.textNotice, TextNotice.clipboardKept);
    });

    test('an empty clipboard is said', () async {
      final vm = model();
      await vm.pasteClipboard();
      expect(vm.text, isEmpty);
      expect(vm.textNotice, TextNotice.clipboardEmpty);
      expect(vm.canStart, isFalse);
    });

    test('a shared text takes the place of the text in the box', () {
      final vm = model();
      expect(vm.takeText('one'), isTrue);
      expect(vm.takeText('two'), isTrue);
      expect(vm.text, 'two');
    });

    test('a shared text stops files still being copied from a share', () {
      final vm = model();
      expect(vm.beginShare(1), isTrue);
      expect(vm.takeText('hi'), isTrue);
      expect(vm.sharing, isFalse);
      expect(vm.addShared(1, [pickedFile('late.jpg')]), isFalse);
      expect(vm.textMode, isTrue);
      expect(vm.files, isEmpty);
    });

    test('shared files are shown and sent, not the text being written', () {
      final vm = model()..takeText('draft');
      expect(vm.beginShare(1), isTrue);
      expect(vm.textMode, isFalse);
      expect(vm.addShared(1, [pickedFile('a.jpg')]), isTrue);
      expect(vm.textMode, isFalse);
      expect(vm.files.single.name, 'a.jpg');
      // The text is still there for when the person goes back to it.
      expect(vm.text, 'draft');
    });

    test('leaving the screen while choosing forgets the text only', () {
      final vm = model()
        ..addFakeFileForTest('a.txt')
        ..takeText('secret');
      vm.leaveText();
      expect(vm.text, isEmpty);
      expect(vm.textMode, isFalse);
      expect(vm.files, hasLength(1));
      // Nothing old hides in front of the next shared text.
      vm.takeText('new');
      expect(vm.text, 'new');
    });

    test('a text over the limit cannot be sent', () {
      final vm = model()..setTextMode(true);
      vm.setText('é' * (repo.maxTextBytes ~/ 2));
      expect(vm.textTooLong, isFalse);
      vm.setText('${vm.text}x');
      expect(vm.textTooLong, isTrue);
      expect(vm.canStart, isFalse);
      expect(vm.bigText, isTrue);
    });

    test('the text is sent as a text, not the files listed', () async {
      final vm = model()
        ..addFakeFileForTest('a.txt')
        ..setTextMode(true)
        ..setText('hello');
      final started = vm.start();
      repo.prepared.complete(FakeSession());
      await started;
      expect(repo.sentTexts, ['hello']);
      repo.events.add(finished());
      await pumpEventQueue();
      expect(vm.stage, SendStage.done);
    });

    test('a text the receiver found damaged is a failure', () async {
      final vm = model()
        ..setTextMode(true)
        ..setText('hello');
      final started = vm.start();
      repo.prepared.complete(FakeSession());
      await started;
      repo.events.add(finished(ok: 0, failed: 1));
      await pumpEventQueue();
      expect(vm.stage, SendStage.failed);
      expect(vm.error?.kind, ErrorKind.textDamaged);
    });

    test('a reset forgets the text', () {
      final vm = model()..takeText('secret');
      vm.reset();
      expect(vm.text, isEmpty);
      expect(vm.textMode, isFalse);
    });
  });

  group('receive', () {
    test('a text offer is received as a text and kept in memory', () async {
      final repo = FakeRepository();
      final vm = ReceiveViewModel(repository: repo)..setInput('7-a-b');
      final connected = vm.connect();
      repo.connected.complete(FakeOffer(isText: true));
      await connected;
      expect(vm.isText, isTrue);
      await vm.accept();
      expect(repo.accepted, ['text']);
      repo.events.add(finished(text: 'hi\u202E'));
      await pumpEventQueue();
      expect(vm.stage, ReceiveStage.done);
      expect(vm.receivedText, 'hi\u202E');
      vm.reset();
      expect(vm.receivedText, isNull);
    });

    test('a text offer needs no folder', () async {
      final repo = FakeRepository()..folderError = Exception('no folder');
      final vm = ReceiveViewModel(repository: repo)..setInput('7-a-b');
      final connected = vm.connect();
      repo.connected.complete(FakeOffer(isText: true));
      await connected;
      expect(vm.stage, ReceiveStage.offered);
      expect(vm.folder, isNull);
      await vm.accept();
      expect(repo.accepted, ['text']);
    });

    test('a file offer is saved as files', () async {
      final repo = FakeRepository();
      final vm = ReceiveViewModel(repository: repo)..setInput('7-a-b');
      final connected = vm.connect();
      repo.connected.complete(FakeOffer());
      await connected;
      await vm.accept();
      expect(repo.accepted, ['files']);
    });

    testWidgets('plain received text shares as it came and hides Open link', (
      tester,
    ) async {
      final shared = <String>[];
      final repo = FakeRepository();
      final vm = ReceiveViewModel(repository: repo)..setInput('7-a-b');
      final connected = vm.connect();
      repo.connected.complete(FakeOffer(isText: true));
      await connected;
      await vm.accept();
      repo.events.add(finished(text: 'meeting notes\r\nline 2'));
      await tester.pump();

      await tester.pumpWidget(
        _receiveApp(vm, onShareText: (t) async => shared.add(t)),
      );
      await tester.pump();

      expect(find.text('Open link'), findsNothing);
      expect(find.text('Copy text'), findsOneWidget);
      expect(find.text('Share text'), findsOneWidget);

      await Scrollable.ensureVisible(tester.element(find.text('Share text')));
      await tester.pump();
      await tester.tap(find.text('Share text'));
      await tester.pump();
      expect(shared, ['meeting notes\r\nline 2']);
    });

    testWidgets(
      'oversized received text omits Share text and handles share PlatformException',
      (tester) async {
        final repo = FakeRepository();
        final vm = ReceiveViewModel(repository: repo)..setInput('7-a-b');
        final connected = vm.connect();
        repo.connected.complete(FakeOffer(isText: true));
        await connected;
        await vm.accept();
        repo.events.add(finished(text: 'x' * (maxSharedTextChars + 1)));
        await tester.pump();

        await tester.pumpWidget(_receiveApp(vm));
        await tester.pump();
        final outer = tester.state<ScrollableState>(
          find.byType(Scrollable).first,
        );
        outer.position.jumpTo(outer.position.maxScrollExtent);
        await tester.pump();

        expect(find.text('Copy text'), findsOneWidget);
        expect(find.text('Share text'), findsNothing);

        // A normal-sized text handles PlatformException from the share sheet cleanly.
        final repo2 = FakeRepository();
        final vm2 = ReceiveViewModel(repository: repo2)..setInput('7-a-b');
        final connected2 = vm2.connect();
        repo2.connected.complete(FakeOffer(isText: true));
        await connected2;
        await vm2.accept();
        repo2.events.add(finished(text: 'short text'));
        await tester.pump();

        await tester.pumpWidget(
          _receiveApp(
            vm2,
            onShareText: (_) => Future.error(PlatformException(code: 'FAILED')),
          ),
        );
        await tester.pump();
        await Scrollable.ensureVisible(tester.element(find.text('Share text')));
        await tester.pump();
        await tester.tap(find.text('Share text'));
        await tester.pump();
        expect(tester.takeException(), isNull);
      },
    );

    testWidgets(
      'a received web link shows its full normalized URL before opening',
      (tester) async {
        final opened = <String>[];
        var browserAvailable = true;
        const channel = MethodChannel('app.totaldrop/open_url');
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              expect(call.method, 'open');
              opened.add(call.arguments as String);
              return browserAvailable;
            });
        addTearDown(
          () => TestDefaultBinaryMessengerBinding
              .instance
              .defaultBinaryMessenger
              .setMockMethodCallHandler(channel, null),
        );

        final repo = FakeRepository();
        final vm = ReceiveViewModel(repository: repo)..setInput('7-a-b');
        final connected = vm.connect();
        repo.connected.complete(FakeOffer(isText: true));
        await connected;
        await vm.accept();
        repo.events.add(finished(text: 'https://münchen.de/rathaus'));
        await tester.pump();

        await tester.pumpWidget(_receiveApp(vm));
        await tester.pump();

        expect(find.text('Open link'), findsOneWidget);
        await Scrollable.ensureVisible(tester.element(find.text('Open link')));
        await tester.pump();
        await tester.tap(find.text('Open link'));
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));

        // Full normalized Punycode URL is shown before any external launch.
        expect(opened, isEmpty);
        expect(find.text('Open this link?'), findsOneWidget);
        expect(find.text('https://xn--mnchen-3ya.de/rathaus'), findsOneWidget);

        // Cancel leaves the browser closed.
        await tester.tap(find.text('Cancel'));
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));
        expect(opened, isEmpty);

        // Confirm launches the normalized URL.
        await tester.tap(find.text('Open link'));
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));
        await tester.tap(find.text('Open'));
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));
        expect(opened, ['https://xn--mnchen-3ya.de/rathaus']);

        // When no browser handles the intent, the person is told.
        browserAvailable = false;
        await tester.tap(find.text('Open link'));
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));
        await tester.tap(find.text('Open'));
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));
        expect(
          find.text('No app on this device could open the link.'),
          findsOneWidget,
        );
      },
    );
  });
}
