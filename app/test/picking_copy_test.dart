// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Android's picker copies every chosen file into the app cache before it
// returns; for a large file that takes seconds. The send screen says so at
// once instead of standing still (#269).
import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/ui/core/theme.dart';
import 'package:totaldrop/ui/send/send_view.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

import 'fakes.dart';

void main() {
  late FakeFileSource source;
  late SendViewModel vm;

  setUp(() {
    source = FakeFileSource();
    vm = SendViewModel(repository: FakeRepository(), fileSource: source);
  });

  group('view model', () {
    test('is copying from the copy start until the picker returns', () async {
      unawaited(vm.addFiles());
      expect(vm.copying, isFalse, reason: 'the picker is still open');
      source.startCopy();
      expect(vm.copying, isTrue);
      source.finish([pickedFile('film.mkv')]);
      await pumpEventQueue();
      expect(vm.copying, isFalse);
      expect(vm.files.single.name, 'film.mkv');
    });

    test('does not open the picker again while it copies', () async {
      unawaited(vm.addFiles());
      source.startCopy();
      await vm.addFiles();
      expect(source.picks, 1);
      source.finish([pickedFile('film.mkv')]);
      await pumpEventQueue();
      expect(vm.files, hasLength(1));
    });

    // Back keeps the selection, so only a reset clears it; a pick that
    // returns after one must not bring files back.
    test('drops files that arrive after the selection was cleared', () async {
      unawaited(vm.addFiles());
      source.startCopy();
      vm.reset();
      final clearsAtReset = source.clears;
      source.finish([pickedFile('film.mkv')]);
      await pumpEventQueue();
      expect(vm.files, isEmpty);
      expect(vm.copying, isFalse);
      expect(
        source.clears,
        greaterThan(clearsAtReset),
        reason: 'the late copy is deleted from the cache too',
      );
    });

    test('cannot create a code while the picker copies', () async {
      vm.addFakeFileForTest('a.txt');
      unawaited(vm.addFiles());
      source.startCopy();
      await vm.start();
      expect(vm.stage, SendStage.picking);
      source.finish([pickedFile('film.mkv')]);
      await pumpEventQueue();
      expect(vm.files, hasLength(2));
    });

    test('keeps files picked before Back once the copy ends', () async {
      unawaited(vm.addFiles());
      source.startCopy();
      // Back from the screen leaves the view model as it is.
      source.finish([pickedFile('film.mkv')]);
      await pumpEventQueue();
      expect(vm.files.single.name, 'film.mkv');
      expect(source.clears, 0);
    });

    test('stops copying when the picker fails', () async {
      final added = vm.addFiles();
      source.startCopy();
      source.fail(Exception('read error'));
      await expectLater(added, throwsException);
      expect(vm.copying, isFalse);
      expect(vm.files, isEmpty);
    });
  });

  testWidgets('the send screen shows the copy at once', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: buildTheme(fallbackScheme(Brightness.light)),
        localizationsDelegates: const [
          AppLocalizations.delegate,
          ...GlobalMaterialLocalizations.delegates,
        ],
        supportedLocales: AppLocalizations.supportedLocales,
        builder: (context, child) => SystemMotion(child: child!),
        home: SendView(viewModel: vm),
      ),
    );
    await tester.tap(find.text('Add files'));
    await tester.pump();
    expect(find.text('Preparing your files…'), findsNothing);

    source.startCopy();
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));
    expect(find.text('Preparing your files…'), findsOneWidget);
    expect(find.text('Add files'), findsNothing);

    source.finish([pickedFile('film.mkv')]);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));
    expect(find.text('Preparing your files…'), findsNothing);
    expect(find.text('film.mkv'), findsOneWidget);
  });
}
