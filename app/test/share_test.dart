// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Files shared to TotalDrop from another app, and the launcher shortcuts
// (#53): what the platform's events mean, what the send screen does with
// them, and which screen opens.
import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/data/launch_source.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/ui/core/launch_router.dart';
import 'package:totaldrop/ui/core/theme.dart';
import 'package:totaldrop/ui/receive/receive_view.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

import 'fakes.dart';

class FakeLaunches implements LaunchSource {
  final controller = StreamController<Launch>.broadcast();
  int clears = 0;

  @override
  Stream<Launch> get launches => controller.stream;

  @override
  Future<void> clearShared() {
    clears++;
    return Future.value();
  }

  void add(Launch l) => controller.add(l);
}

PickedFile shared(String name) =>
    PickedFile(name: name, path: '/cache/shared/1/$name', size: 10);

void main() {
  group('platform events', () {
    test('a copied share becomes its files', () {
      final l = parseLaunch({
        'files': [
          {'name': 'a.jpg', 'path': '/c/a.jpg', 'size': 3},
          {'name': 'b.pdf', 'path': '/c/b.pdf', 'size': 4},
        ],
      });
      final files = (l.single as SharedFiles).files;
      expect(files.map((f) => f.name), ['a.jpg', 'b.pdf']);
      expect(files.first.size, 3);
    });

    test('a share and a shortcut', () {
      expect(
        parseLaunch({'screen': 'send', 'copying': true}).single,
        isA<ShareCopying>(),
      );
      expect(parseLaunch({'failed': true}).single, isA<ShareFailed>());
      expect(
        parseLaunch({'screen': 'send', 'textOnly': true}).single,
        isA<SharedTextOnly>(),
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
      // A relative path, a missing size: not a file the core may read.
      expect(
        parseLaunch({
          'files': [
            {'name': 'a', 'path': 'rel/a', 'size': 1},
            {'name': 'b', 'path': '/c/b'},
          ],
        }).single,
        isA<ShareFailed>(),
      );
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
      expect(vm.beginShare(), isTrue);
      expect(vm.sharing, isTrue);
      vm.addShared([shared('a.jpg'), shared('b.jpg')]);
      expect(vm.sharing, isFalse);
      expect(vm.files.map((f) => f.name), ['a.jpg', 'b.jpg']);
    });

    test('a share adds to what is already selected', () {
      vm.addFakeFileForTest('mine.txt');
      vm.beginShare();
      vm.addShared([shared('a.jpg')]);
      expect(vm.files, hasLength(2));
    });

    test('cannot create a code while a share copies', () async {
      vm.addFakeFileForTest('mine.txt');
      vm.beginShare();
      await vm.start();
      expect(vm.stage, SendStage.picking);
    });

    test(
      'is refused while a transfer runs, and its files are not added',
      () async {
        vm.addFakeFileForTest('mine.txt');
        unawaited(vm.start());
        expect(vm.busy, isTrue);
        expect(vm.beginShare(), isFalse);
        vm.addShared([shared('a.jpg')]);
        expect(vm.files.single.name, 'mine.txt');
      },
    );

    test('is refused while the picker is open', () {
      unawaited(vm.addFiles());
      expect(vm.beginShare(), isFalse);
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
        final clears = launches.clears;
        expect(vm.beginShare(), isTrue);
        expect(vm.stage, SendStage.picking);
        expect(launches.clears, clears, reason: 'the copy is being written');
        vm.addShared([shared('a.jpg')]);
        expect(vm.files.single.name, 'a.jpg');
      },
    );

    test('files that arrive after a reset are dropped', () {
      vm.beginShare();
      vm.reset();
      expect(launches.clears, 1, reason: 'the copies are deleted');
      vm.addShared([shared('a.jpg')]);
      expect(vm.files, isEmpty);
      expect(vm.sharing, isFalse);
    });

    test('a failed copy says so, like a failed pick', () {
      vm.beginShare();
      vm.shareFailed();
      expect(vm.sharing, isFalse);
      expect(vm.addFailed, isTrue);
    });

    test('the picker waits while a share copies', () async {
      vm.beginShare();
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
      launches.add(const ShareCopying());
      await settle(tester);
      expect(find.text('Preparing your files…'), findsOneWidget);
      launches.add(SharedFiles([shared('holiday.jpg')]));
      await settle(tester);
      expect(find.text('holiday.jpg'), findsOneWidget);
    });

    testWidgets('a second share goes to the Send screen already open', (
      tester,
    ) async {
      await pumpApp(tester);
      launches.add(const ShareCopying());
      launches.add(SharedFiles([shared('a.jpg')]));
      await settle(tester);
      launches.add(const ShareCopying());
      launches.add(SharedFiles([shared('b.jpg')]));
      await settle(tester);
      expect(find.text('a.jpg'), findsOneWidget);
      expect(find.text('b.jpg'), findsOneWidget);
      expect(find.byType(SendView), findsOneWidget);
    });

    testWidgets('a share during a transfer is refused with a message', (
      tester,
    ) async {
      await pumpApp(tester);
      send.addFakeFileForTest('mine.txt');
      unawaited(send.start());
      await settle(tester);
      launches.add(const ShareCopying());
      await settle(tester);
      expect(find.text('home'), findsOneWidget);
      expect(
        find.text(
          'A transfer is still running. Wait for it to finish or stop it, '
          'then share again.',
        ),
        findsOneWidget,
      );
      expect(send.stage, SendStage.preparing, reason: 'not cut short');
    });

    testWidgets('shared text opens Send and says it is not sent yet', (
      tester,
    ) async {
      await pumpApp(tester);
      launches.add(const SharedTextOnly());
      await settle(tester);
      expect(find.byType(SendView), findsOneWidget);
      expect(
        find.text(
          'TotalDrop does not send text yet, only files. Share a file instead.',
        ),
        findsOneWidget,
      );
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
  });
}
