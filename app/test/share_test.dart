// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Files shared to TotalDrop from another app, the launcher shortcuts
// (#53) and TotalDrop links (#445): what the platform's events mean, what
// the screens do with them, and which screen opens.
import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/data/launch_source.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/l10n.dart' show ShownText;
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/src/rust/frb_generated.dart';
import 'package:totaldrop/ui/core/launch_router.dart';
import 'package:totaldrop/ui/core/theme.dart';
import 'package:totaldrop/ui/receive/receive_view.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

import 'fakes.dart';

class FakeLaunches implements LaunchSource {
  final controller = StreamController<Launch>.broadcast();
  final dropped = <int>[];

  @override
  Stream<Launch> get launches => controller.stream;

  @override
  Future<void> drop(Iterable<int> ids) {
    dropped.addAll(ids);
    return Future.value();
  }

  void add(Launch l) => controller.add(l);
}

PickedFile shared(String name) =>
    PickedFile(name: name, path: '/cache/shared/1/$name', size: 10);

/// The send screen asks the core whether a text holds hidden characters.
class _Rust implements RustLibApi {
  @override
  ShownText crateApiL10NShownText({required String text}) =>
      ShownText(text: text, short: false, hidden: text.contains('\u202E'));

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  setUpAll(() => RustLib.initMock(api: _Rust()));

  group('platform events', () {
    test('a link is handed on as it came', () {
      final l = parseLaunch({'link': 'totaldrop://r/7-harbor-rocket'});
      expect(l, hasLength(1));
      expect((l.single as OpenLink).link, 'totaldrop://r/7-harbor-rocket');
      expect(parseLaunch({'link': ''}), isEmpty);
      expect(parseLaunch({'link': 7}), isEmpty);
    });

    test('a copied share becomes its files', () {
      final l = parseLaunch({
        'id': 7,
        'files': [
          {'name': 'a.jpg', 'path': '/c/a.jpg', 'size': 3},
          {'name': 'b.pdf', 'path': '/c/b.pdf', 'size': 4},
        ],
        'skipped': 2,
      });
      final share = l.single as SharedFiles;
      expect(share.id, 7);
      expect(share.files.map((f) => f.name), ['a.jpg', 'b.pdf']);
      expect(share.files.first.size, 3);
      expect(share.skipped, 2);
    });

    test('a share and a shortcut', () {
      expect(
        (parseLaunch({'id': 3, 'screen': 'send', 'copying': true}).single
                as ShareCopying)
            .id,
        3,
      );
      expect(
        (parseLaunch({'id': 3, 'failed': true}).single as ShareFailed).id,
        3,
      );
      expect(
        (parseLaunch({'screen': 'send', 'text': 'a\r\nb'}).single as SharedText)
            .text,
        'a\r\nb',
      );
      expect(parseLaunch({'screen': 'send', 'text': ''}), isEmpty);
      expect(
        (parseLaunch({'screen': 'clipboard'}).single as OpenScreen).screen,
        clipboardScreen,
      );
      expect(
        (parseLaunch({'screen': 'receive'}).single as OpenScreen).screen,
        'receive',
      );
    });

    test('what is not understood is left out', () {
      expect(parseLaunch(null), isEmpty);
      expect(parseLaunch('send'), isEmpty);
      expect(parseLaunch({'screen': 'settings'}), isEmpty);
      // A share without its number cannot be matched to its copy.
      expect(parseLaunch({'copying': true}), isEmpty);
      // A relative path, a missing size: not a file the core may read.
      expect(
        parseLaunch({
          'id': 1,
          'files': [
            {'name': 'a', 'path': 'rel/a', 'size': 1},
            {'name': 'b', 'path': '/c/b'},
          ],
        }).single,
        isA<ShareFailed>(),
      );
      // Those that are left out count as skipped.
      final some =
          parseLaunch({
                'id': 1,
                'files': [
                  {'name': 'a', 'path': '/c/a', 'size': 1},
                  {'name': 'b', 'path': 'rel/b', 'size': 1},
                ],
                'skipped': 1,
              }).single
              as SharedFiles;
      expect(some.files.single.name, 'a');
      expect(some.skipped, 2);
    });
  });

  group('send view model', () {
    late FakeRepository repo;
    late FakeLaunches launches;
    late FakeFileSource picker;
    late SendViewModel vm;

    setUp(() {
      repo = FakeRepository();
      launches = FakeLaunches();
      picker = FakeFileSource();
      vm = SendViewModel(
        repository: repo,
        fileSource: picker,
        launches: launches,
      );
    });

    test('a share waits for its copy, then lists the files', () {
      expect(vm.beginShare(1), isTrue);
      expect(vm.sharing, isTrue);
      expect(vm.addShared(1, [shared('a.jpg'), shared('b.jpg')]), isTrue);
      expect(vm.sharing, isFalse);
      expect(vm.files.map((f) => f.name), ['a.jpg', 'b.jpg']);
      expect(launches.dropped, isEmpty);
    });

    test('a share adds to what is already selected', () {
      vm.addFakeFileForTest('mine.txt');
      vm.beginShare(1);
      vm.addShared(1, [shared('a.jpg')]);
      expect(vm.files, hasLength(2));
    });

    test('two shares at once: the wait ends with the last copy', () {
      vm.beginShare(1);
      vm.beginShare(2);
      vm.addShared(2, [shared('b.jpg')]);
      expect(vm.sharing, isTrue, reason: 'the first share still copies');
      vm.shareFailed(1);
      expect(vm.sharing, isFalse);
      expect(vm.addFailed, isTrue);
      expect(vm.files.single.name, 'b.jpg');
    });

    test('cannot create a code while a share copies', () async {
      vm.addFakeFileForTest('mine.txt');
      vm.beginShare(1);
      await vm.start();
      expect(vm.stage, SendStage.picking);
    });

    test(
      'is refused while a transfer runs, and its files are not added',
      () async {
        vm.addFakeFileForTest('mine.txt');
        unawaited(vm.start());
        expect(vm.busy, isTrue);
        expect(vm.beginShare(1), isFalse);
        expect(vm.addShared(1, [shared('a.jpg')]), isFalse);
        expect(vm.files.single.name, 'mine.txt');
        expect(launches.dropped, [1], reason: 'its copy is deleted');
      },
    );

    test('is taken while the picker is open', () async {
      unawaited(vm.addFiles());
      expect(vm.beginShare(1), isTrue);
      vm.addShared(1, [shared('a.jpg')]);
      expect(vm.files.single.name, 'a.jpg');
    });

    test(
      'a finished send gives way to the share, without deleting its copy',
      () async {
        vm.addFakeFileForTest('mine.txt');
        unawaited(vm.start());
        repo.prepared.complete(FakeSession());
        await pumpEventQueue();
        repo.events.add(event(EventKind.finished));
        await pumpEventQueue();
        expect(vm.stage, isNot(SendStage.picking));
        expect(vm.beginShare(4), isTrue);
        expect(vm.stage, SendStage.picking);
        expect(launches.dropped, isNot(contains(4)));
        vm.addShared(4, [shared('a.jpg')]);
        expect(vm.files.single.name, 'a.jpg');
      },
    );

    test('a reset deletes the copies, done and still copying', () {
      vm.beginShare(1);
      vm.addShared(1, [shared('a.jpg')]);
      vm.beginShare(2);
      vm.reset();
      expect(launches.dropped, unorderedEquals([1, 2]));
      expect(vm.sharing, isFalse);
      // The copy that was still running arrives after all: not added.
      expect(vm.addShared(2, [shared('b.jpg')]), isFalse);
      expect(vm.files, isEmpty);
    });

    test('a share can be cancelled, keeping what was chosen', () {
      vm.addFakeFileForTest('mine.txt');
      vm.beginShare(1);
      vm.cancelShares();
      expect(vm.sharing, isFalse);
      expect(launches.dropped, [1]);
      expect(vm.addShared(1, [shared('a.jpg')]), isFalse);
      expect(vm.files.single.name, 'mine.txt');
    });

    test('a failed copy says so, like a failed pick', () {
      vm.beginShare(1);
      vm.shareFailed(1);
      expect(vm.sharing, isFalse);
      expect(vm.addFailed, isTrue);
    });

    test('the picker waits while a share copies', () async {
      vm.beginShare(1);
      await vm.addFiles();
      expect(picker.picks, 0);
    });
  });

  group('screens', () {
    late FakeRepository repo;
    late FakeLaunches launches;
    late SendViewModel send;
    late ReceiveViewModel receive;

    Future<void> pumpApp(WidgetTester tester) async {
      final navigator = GlobalKey<NavigatorState>();
      final messenger = GlobalKey<ScaffoldMessengerState>();
      Route<void> sendRoute() => MaterialPageRoute<void>(
        settings: const RouteSettings(name: 'send'),
        builder: (_) => SendView(viewModel: send),
      );
      Route<void> receiveRoute() => MaterialPageRoute<void>(
        settings: const RouteSettings(name: 'receive'),
        builder: (_) => ReceiveView(viewModel: receive),
      );
      await tester.pumpWidget(
        MaterialApp(
          navigatorKey: navigator,
          scaffoldMessengerKey: messenger,
          theme: buildTheme(fallbackScheme(Brightness.light)),
          localizationsDelegates: const [
            AppLocalizations.delegate,
            ...GlobalMaterialLocalizations.delegates,
          ],
          supportedLocales: AppLocalizations.supportedLocales,
          builder: (context, child) => LaunchRouter(
            launches: launches,
            navigator: navigator,
            messenger: messenger,
            send: send,
            receive: receive,
            openSend: sendRoute,
            openReceive: receiveRoute,
            child: SystemMotion(child: child!),
          ),
          home: const Scaffold(body: Text('home')),
        ),
      );
    }

    Future<void> settle(WidgetTester tester) async {
      // The event, the frame that acts on it, and any message's entrance.
      for (var i = 0; i < 3; i++) {
        await tester.pump(const Duration(milliseconds: 300));
      }
    }

    setUp(() {
      repo = FakeRepository();
      launches = FakeLaunches();
      send = SendViewModel(repository: repo, launches: launches);
      receive = ReceiveViewModel(repository: repo);
    });

    testWidgets('a share opens Send with the shared files', (tester) async {
      await pumpApp(tester);
      launches.add(const ShareCopying(1));
      await settle(tester);
      expect(find.text('Preparing your files…'), findsOneWidget);
      launches.add(SharedFiles(1, [shared('holiday.jpg')]));
      await settle(tester);
      expect(find.text('holiday.jpg'), findsOneWidget);
    });

    testWidgets('a second share goes to the Send screen already open', (
      tester,
    ) async {
      await pumpApp(tester);
      launches.add(const ShareCopying(1));
      launches.add(SharedFiles(1, [shared('a.jpg')]));
      await settle(tester);
      launches.add(const ShareCopying(2));
      launches.add(SharedFiles(2, [shared('b.jpg')]));
      await settle(tester);
      expect(find.text('a.jpg'), findsOneWidget);
      expect(find.text('b.jpg'), findsOneWidget);
      expect(find.byType(SendView), findsOneWidget);
    });

    testWidgets('items that could not be read are counted', (tester) async {
      await pumpApp(tester);
      launches.add(const ShareCopying(1));
      launches.add(SharedFiles(1, [shared('a.jpg')], skipped: 2));
      await settle(tester);
      expect(find.text('a.jpg'), findsOneWidget);
      expect(
        find.text('2 shared items could not be read and were left out.'),
        findsOneWidget,
      );
    });

    testWidgets('Cancel, or leaving the wait, stops the share', (tester) async {
      await pumpApp(tester);
      launches.add(const ShareCopying(1));
      await settle(tester);
      await tester.tap(find.text('Cancel'));
      await settle(tester);
      expect(launches.dropped, [1]);
      expect(send.sharing, isFalse);
      launches.add(const ShareCopying(2));
      await settle(tester);
      final nav = tester.state<NavigatorState>(find.byType(Navigator).first);
      nav.pop();
      await settle(tester);
      expect(find.text('home'), findsOneWidget);
      expect(launches.dropped, [1, 2]);
      // Its files, if they come, are not added behind the person's back.
      launches.add(SharedFiles(2, [shared('late.jpg')]));
      await settle(tester);
      expect(send.files, isEmpty);
    });

    testWidgets('a share during a transfer is refused with a message', (
      tester,
    ) async {
      await pumpApp(tester);
      send.addFakeFileForTest('mine.txt');
      unawaited(send.start());
      await settle(tester);
      launches.add(const ShareCopying(5));
      await settle(tester);
      expect(
        find.text(
          'A transfer is still running. Wait for it to finish or stop it, '
          'then share again.',
        ),
        findsOneWidget,
      );
      expect(send.stage, SendStage.preparing, reason: 'not cut short');
      expect(launches.dropped, [5], reason: 'its copy is deleted');
      expect(find.byType(SendView), findsOneWidget, reason: 'the running one');
    });

    testWidgets('a shortcut during a transfer shows that transfer', (
      tester,
    ) async {
      await pumpApp(tester);
      send.addFakeFileForTest('mine.txt');
      unawaited(send.start());
      await settle(tester);
      expect(find.text('home'), findsOneWidget);
      launches.add(const OpenScreen('receive'));
      await settle(tester);
      expect(find.byType(ReceiveView), findsNothing);
      expect(find.byType(SendView), findsOneWidget);
      expect(send.stage, SendStage.preparing);
    });

    testWidgets('shared text opens Send with the text in its box', (
      tester,
    ) async {
      await pumpApp(tester);
      launches.add(const SharedText('https://example.org/'));
      await settle(tester);
      expect(find.byType(SendView), findsOneWidget);
      expect(send.textMode, isTrue);
      expect(find.text('Send text'), findsOneWidget);
      expect(find.text('https://example.org/'), findsOneWidget);
    });

    testWidgets('hidden characters in a shared text are named', (tester) async {
      await pumpApp(tester);
      launches.add(const SharedText('plain'));
      await settle(tester);
      const note =
          'The text holds hidden characters, such as direction marks. They are sent with it.';
      expect(find.text(note), findsNothing);
      launches.add(const SharedText('abc\u202Edef'));
      await settle(tester);
      expect(find.text(note), findsOneWidget);
    });

    testWidgets('the clipboard shortcut opens Send with the clipboard', (
      tester,
    ) async {
      send = SendViewModel(
        repository: repo,
        launches: launches,
        textClipboard: const FakeClipboard('kedi-balık-42'),
      );
      await pumpApp(tester);
      launches.add(const OpenScreen(clipboardScreen));
      await settle(tester);
      expect(find.byType(SendView), findsOneWidget);
      expect(send.text, 'kedi-balık-42');
      expect(find.text('kedi-balık-42'), findsOneWidget);
    });

    testWidgets('shared text during a transfer is refused with a message', (
      tester,
    ) async {
      await pumpApp(tester);
      send.addFakeFileForTest('a.txt');
      unawaited(send.start());
      launches.add(const SharedText('hello'));
      await settle(tester);
      expect(send.text, isEmpty);
      expect(
        find.text(
          'A transfer is still running. Wait for it to finish or stop it, then share again.',
        ),
        findsOneWidget,
      );
    });

    testWidgets('the clipboard shortcut during a transfer says why', (
      tester,
    ) async {
      send = SendViewModel(
        repository: repo,
        launches: launches,
        textClipboard: const FakeClipboard('hello'),
      );
      await pumpApp(tester);
      send.addFakeFileForTest('a.txt');
      unawaited(send.start());
      launches.add(const OpenScreen(clipboardScreen));
      await settle(tester);
      expect(send.text, isEmpty);
      expect(
        find.text(
          'A transfer is still running. Wait for it to finish or stop it, then share again.',
        ),
        findsOneWidget,
      );
    });

    testWidgets('leaving Send forgets a text being written', (tester) async {
      await pumpApp(tester);
      launches.add(const SharedText('secret'));
      await settle(tester);
      expect(send.textMode, isTrue);
      await tester.pageBack();
      await settle(tester);
      expect(find.byType(SendView), findsNothing);
      expect(send.text, isEmpty);
      expect(send.textMode, isFalse);
    });

    testWidgets('the shortcuts open Send and Receive', (tester) async {
      await pumpApp(tester);
      launches.add(const OpenScreen('receive'));
      await settle(tester);
      expect(find.byType(ReceiveView), findsOneWidget);
      launches.add(const OpenScreen('send'));
      await settle(tester);
      expect(find.byType(SendView), findsOneWidget);
      expect(find.byType(ReceiveView), findsNothing);
    });

    testWidgets('a link opens Receive with its code, not connected', (
      tester,
    ) async {
      await pumpApp(tester);
      launches.add(const OpenLink('totaldrop://r/7-harbor-rocket'));
      await settle(tester);
      expect(find.byType(ReceiveView), findsOneWidget);
      final field = tester.widget<TextField>(find.byType(TextField));
      expect(field.controller!.text, '7-harbor-rocket');
      expect(receive.stage, ReceiveStage.entering, reason: 'not connected');
    });

    testWidgets('a link replaces what was typed on the open Receive', (
      tester,
    ) async {
      await pumpApp(tester);
      launches.add(const OpenScreen('receive'));
      await settle(tester);
      await tester.enterText(find.byType(TextField), '3-old');
      launches.add(const OpenLink('totaldrop://r/7-harbor-rocket'));
      await settle(tester);
      final field = tester.widget<TextField>(find.byType(TextField));
      expect(field.controller!.text, '7-harbor-rocket');
      expect(receive.input, '7-harbor-rocket');
    });

    testWidgets('a link without a code opens an empty Receive', (tester) async {
      await pumpApp(tester);
      launches.add(const OpenLink('totaldrop://r/evil?c=7-harbor-rocket'));
      await settle(tester);
      expect(find.byType(ReceiveView), findsOneWidget);
      expect(receive.input, isEmpty);
    });

    testWidgets('a link during a transfer is refused with a message', (
      tester,
    ) async {
      await pumpApp(tester);
      send.addFakeFileForTest('mine.txt');
      unawaited(send.start());
      await settle(tester);
      launches.add(const OpenLink('totaldrop://r/7-harbor-rocket'));
      await settle(tester);
      expect(
        find.text(
          'A transfer is still running. Wait for it to finish or stop it, '
          'then open the link again.',
        ),
        findsOneWidget,
      );
      expect(send.stage, SendStage.preparing, reason: 'not cut short');
      expect(receive.input, isEmpty);
      expect(find.byType(SendView), findsOneWidget, reason: 'the running one');
    });
  });
}
