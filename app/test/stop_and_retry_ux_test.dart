// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// In-screen Cancel / Stop controls across active stages, Receive code
// preservation on retry, calm stop iconography, and text decline copy (#591).
import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/l10n.dart' show ShownText;
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/src/rust/frb_generated.dart';
import 'package:totaldrop/ui/core/common.dart';
import 'package:totaldrop/ui/core/theme.dart';
import 'package:totaldrop/ui/receive/receive_view.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

import 'fakes.dart';

class _Rust implements RustLibApi {
  @override
  String crateApiL10NRevealControls({required String name}) => name;

  @override
  bool crateApiL10NNeedsEnglish({required String tag, required String key}) =>
      false;

  @override
  ShownText crateApiL10NShownText({required String text}) =>
      ShownText(text: text, short: false, hidden: false);

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

Widget _wrap(Widget home) => MaterialApp(
  theme: buildTheme(fallbackScheme(Brightness.light)),
  locale: const Locale('en'),
  localizationsDelegates: const [
    AppLocalizations.delegate,
    ...GlobalMaterialLocalizations.delegates,
  ],
  supportedLocales: AppLocalizations.supportedLocales,
  home: home,
);

/// Advances past AnimatedSwitcher and dialog transitions without hanging on
/// TotalDropMark's continuous transfer animation.
Future<void> _settle(WidgetTester tester) async {
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 350));
}

/// Scrolls the stage's ListView until [target] is built and visible, then taps it.
Future<void> _scrollAndTap(WidgetTester tester, Finder target) async {
  await tester.scrollUntilVisible(
    target,
    200,
    scrollable: find.byType(Scrollable).first,
  );
  await _settle(tester);
  await tester.tap(target);
  await _settle(tester);
}

void main() {
  setUpAll(() => RustLib.initMock(api: _Rust()));

  group('SendView in-screen Cancel and Stop (#591)', () {
    testWidgets('Cancel while preparing stops and keeps chosen files', (
      tester,
    ) async {
      final repo = FakeRepository();
      final keep = FakeKeepAlive();
      final vm = SendViewModel(
        repository: repo,
        keepAlive: keep,
        fileSource: FakeFileSource(),
      )..addFakeFileForTest('notes.pdf');

      await tester.pumpWidget(_wrap(SendView(viewModel: vm)));
      final l = await AppLocalizations.delegate.load(const Locale('en'));

      unawaited(vm.start());
      await _settle(tester);
      expect(vm.stage, SendStage.preparing);

      await tester.tap(find.widgetWithText(TextButton, l.cancel));
      await _settle(tester);

      expect(vm.stage, SendStage.picking);
      expect(vm.files.map((f) => f.name), ['notes.pdf']);
      expect(repo.stoppers.single.stopped, isTrue);
      expect(keep.calls, ['begin sending', 'end']);

      final late = FakeSession();
      repo.prepared.complete(late);
      await tester.pump();
      expect(late.disposed, isTrue);
      expect(vm.stage, SendStage.picking);
    });

    testWidgets('Stop while waiting returns directly to chosen text', (
      tester,
    ) async {
      await tester.binding.setSurfaceSize(const Size(360, 520));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      final repo = FakeRepository();
      final vm = SendViewModel(repository: repo, fileSource: FakeFileSource())
        ..takeText('hello world');

      await tester.pumpWidget(_wrap(SendView(viewModel: vm)));
      final l = await AppLocalizations.delegate.load(const Locale('en'));

      unawaited(vm.start());
      repo.prepared.complete(FakeSession());
      await _settle(tester);
      expect(vm.stage, SendStage.waiting);

      await _scrollAndTap(tester, find.widgetWithText(OutlinedButton, l.stop));

      expect(vm.stage, SendStage.picking);
      expect(vm.textMode, isTrue);
      expect(vm.text, 'hello world');
      expect(repo.stoppers.single.stopped, isTrue);
    });

    testWidgets(
      'Stop while connected or transferring asks first and keeps selection',
      (tester) async {
        final repo = FakeRepository();
        final vm = SendViewModel(repository: repo, fileSource: FakeFileSource())
          ..addFakeFileForTest('archive.tar');

        await tester.pumpWidget(_wrap(SendView(viewModel: vm)));
        final l = await AppLocalizations.delegate.load(const Locale('en'));

        unawaited(vm.start());
        repo.prepared.complete(FakeSession());
        await tester.pump();
        repo.events.add(
          const TransferEvent(
            kind: EventKind.connected,
            words: ['alpha', 'bravo', 'charlie', 'delta'],
            done: 0,
            total: 0,
            summary: null,
            error: null,
          ),
        );
        await _settle(tester);
        expect(vm.stage, SendStage.connected);

        await _scrollAndTap(
          tester,
          find.widgetWithText(OutlinedButton, l.stop),
        );
        expect(find.text(l.cancelTransferTitle), findsOneWidget);

        // Choosing "Keep going" keeps the connected session alive.
        await tester.tap(find.text(l.keepGoing));
        await _settle(tester);
        expect(vm.stage, SendStage.connected);
        expect(repo.stoppers.single.stopped, isFalse);

        // Move to transferring and confirm Stop.
        repo.events.add(
          const TransferEvent(
            kind: EventKind.progress,
            words: [],
            done: 50,
            total: 100,
            summary: null,
            error: null,
          ),
        );
        await _settle(tester);
        expect(vm.stage, SendStage.transferring);

        await _scrollAndTap(
          tester,
          find.widgetWithText(OutlinedButton, l.stop),
        );
        await tester.tap(find.widgetWithText(FilledButton, l.stop));
        await _settle(tester);

        expect(vm.stage, SendStage.picking);
        expect(vm.files.map((f) => f.name), ['archive.tar']);
        expect(repo.stoppers.single.stopped, isTrue);
      },
    );
  });

  group(
    'ReceiveView Cancel, Stop, retry code preservation, and text decline',
    () {
      testWidgets(
        'Cancel while connecting keeps typed code and declines late offer',
        (tester) async {
          final repo = FakeRepository();
          final keep = FakeKeepAlive();
          final vm = ReceiveViewModel(repository: repo, keepAlive: keep)
            ..setInput('7-harbor-rocket');

          await tester.pumpWidget(_wrap(ReceiveView(viewModel: vm)));
          final l = await AppLocalizations.delegate.load(const Locale('en'));

          unawaited(vm.connect());
          await _settle(tester);
          expect(vm.stage, ReceiveStage.connecting);

          await tester.tap(find.widgetWithText(TextButton, l.cancel));
          await _settle(tester);

          expect(vm.stage, ReceiveStage.entering);
          expect(vm.input, '7-harbor-rocket');
          expect(
            tester.widget<TextField>(find.byType(TextField)).controller?.text,
            '7-harbor-rocket',
          );
          expect(repo.stoppers.single.stopped, isTrue);
          expect(keep.calls, ['begin receiving', 'end']);

          final late = FakeOffer();
          repo.connected.complete(late);
          await tester.pump();
          expect(repo.declined, [late]);
        },
      );

      testWidgets('Stop while receiving asks first and resets to empty input', (
        tester,
      ) async {
        final repo = FakeRepository();
        final vm = ReceiveViewModel(repository: repo)
          ..setInput('7-harbor-rocket');

        await tester.pumpWidget(_wrap(ReceiveView(viewModel: vm)));
        final l = await AppLocalizations.delegate.load(const Locale('en'));

        unawaited(vm.connect());
        repo.connected.complete(FakeOffer());
        await _settle(tester);
        await vm.accept();
        await _settle(tester);
        expect(vm.stage, ReceiveStage.transferring);

        await _scrollAndTap(
          tester,
          find.widgetWithText(OutlinedButton, l.stop),
        );
        await tester.tap(find.widgetWithText(FilledButton, l.stop));
        await _settle(tester);

        expect(vm.stage, ReceiveStage.entering);
        expect(vm.input, isEmpty);
        expect(
          tester.widget<TextField>(find.byType(TextField)).controller?.text,
          isEmpty,
        );
        expect(repo.stoppers.single.stopped, isTrue);
      });

      testWidgets(
        'confirming Stop after transfer finishes keeps the Done screen',
        (tester) async {
          final repo = FakeRepository();
          final vm = ReceiveViewModel(repository: repo)
            ..setInput('7-harbor-rocket');

          await tester.pumpWidget(_wrap(ReceiveView(viewModel: vm)));
          final l = await AppLocalizations.delegate.load(const Locale('en'));

          unawaited(vm.connect());
          repo.connected.complete(FakeOffer());
          await _settle(tester);
          await vm.accept();
          await _settle(tester);
          expect(vm.stage, ReceiveStage.transferring);

          await _scrollAndTap(
            tester,
            find.widgetWithText(OutlinedButton, l.stop),
          );
          expect(find.text(l.cancelTransferTitle), findsOneWidget);

          // Transfer finishes while the confirmation dialog is open.
          repo.events.add(
            const TransferEvent(
              kind: EventKind.finished,
              words: [],
              done: 100,
              total: 100,
              summary: TransferSummary(
                filesOk: 1,
                filesFailed: 0,
                bytes: 100,
                saved: ['/tmp/doc.pdf'],
              ),
              error: null,
            ),
          );
          await _settle(tester);
          expect(vm.stage, ReceiveStage.done);

          // Tapping Stop must not wipe the finished result screen.
          await tester.tap(find.widgetWithText(FilledButton, l.stop));
          await _settle(tester);
          expect(vm.stage, ReceiveStage.done);
          expect(find.text(l.receiveDone), findsOneWidget);
        },
      );

      testWidgets(
        'Try again keeps unspent Receive code and clears spent Receive code',
        (tester) async {
          final l = await AppLocalizations.delegate.load(const Locale('en'));

          for (final reusable in const [
            ErrorKind.invalidCode,
            ErrorKind.codeNotInUse,
            ErrorKind.badServer,
            ErrorKind.server,
          ]) {
            final repo = FakeRepository();
            final vm = ReceiveViewModel(repository: repo)
              ..setInput('7-harbor-rocket');
            await tester.pumpWidget(_wrap(ReceiveView(viewModel: vm)));

            unawaited(vm.connect());
            repo.connected.completeError(
              TransferError(kind: reusable, detail: '', cause: ''),
            );
            await _settle(tester);
            expect(vm.stage, ReceiveStage.failed);

            await tester.tap(find.widgetWithText(FilledButton, l.tryAgain));
            await _settle(tester);
            expect(vm.stage, ReceiveStage.entering);
            expect(vm.input, '7-harbor-rocket', reason: '$reusable');
            expect(
              tester.widget<TextField>(find.byType(TextField)).controller?.text,
              '7-harbor-rocket',
              reason: '$reusable field text',
            );
          }

          for (final spent in const [
            ErrorKind.connect,
            ErrorKind.wrongCode,
            ErrorKind.codeExpired,
            ErrorKind.peerLeft,
          ]) {
            final repo = FakeRepository();
            final vm = ReceiveViewModel(repository: repo)
              ..setInput('7-harbor-rocket');
            await tester.pumpWidget(_wrap(ReceiveView(viewModel: vm)));

            unawaited(vm.connect());
            repo.connected.completeError(
              TransferError(kind: spent, detail: '', cause: ''),
            );
            await _settle(tester);
            expect(vm.stage, ReceiveStage.failed);

            await tester.tap(find.widgetWithText(FilledButton, l.tryAgain));
            await _settle(tester);
            expect(vm.stage, ReceiveStage.entering);
            expect(vm.input, isEmpty, reason: '$spent');
            expect(
              tester.widget<TextField>(find.byType(TextField)).controller?.text,
              isEmpty,
              reason: '$spent field text',
            );
          }
        },
      );

      testWidgets(
        'declining a text offer says text, declining files says files',
        (tester) async {
          final l = await AppLocalizations.delegate.load(const Locale('en'));

          final textRepo = FakeRepository();
          final textVm = ReceiveViewModel(repository: textRepo)
            ..setInput('7-harbor-rocket');
          await tester.pumpWidget(_wrap(ReceiveView(viewModel: textVm)));
          unawaited(textVm.connect());
          textRepo.connected.complete(FakeOffer(isText: true));
          await _settle(tester);
          await textVm.decline();
          await _settle(tester);
          expect(find.text(l.receiveTextDeclined), findsOneWidget);
          expect(find.text(l.receiveDeclined), findsNothing);

          final fileRepo = FakeRepository();
          final fileVm = ReceiveViewModel(repository: fileRepo)
            ..setInput('7-harbor-rocket');
          await tester.pumpWidget(_wrap(ReceiveView(viewModel: fileVm)));
          unawaited(fileVm.connect());
          fileRepo.connected.complete(FakeOffer(isText: false));
          await _settle(tester);
          await fileVm.decline();
          await _settle(tester);
          expect(find.text(l.receiveDeclined), findsOneWidget);
          expect(find.text(l.receiveTextDeclined), findsNothing);
        },
      );
    },
  );

  testWidgets('ErrorPanel renders ErrorKind.stopped with a neutral stop icon', (
    tester,
  ) async {
    final scheme = fallbackScheme(Brightness.light);
    await tester.pumpWidget(
      _wrap(
        Scaffold(
          body: ErrorPanel(
            error: const TransferError(
              kind: ErrorKind.stopped,
              detail: '',
              cause: '',
            ),
            onRetry: () {},
          ),
        ),
      ),
    );
    await _settle(tester);

    final stopIcon = tester.widget<Icon>(
      find.byIcon(Icons.stop_circle_outlined),
    );
    expect(stopIcon.color, scheme.onSurfaceVariant);

    await tester.pumpWidget(
      _wrap(
        Scaffold(
          body: ErrorPanel(
            error: const TransferError(
              kind: ErrorKind.connect,
              detail: '',
              cause: '',
            ),
            onRetry: () {},
          ),
        ),
      ),
    );
    await _settle(tester);

    final errorIcon = tester.widget<Icon>(find.byIcon(Icons.error_outline));
    expect(errorIcon.color, scheme.error);
  });
}
