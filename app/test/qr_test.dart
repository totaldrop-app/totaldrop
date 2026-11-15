// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Send QR code rendering and Receive camera QR scanning (#29).
import 'dart:async';

import 'package:flutter/semantics.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/data/qr_scanner.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/src/rust/frb_generated.dart';
import 'package:totaldrop/ui/core/qr_code_view.dart';
import 'package:totaldrop/ui/core/theme.dart';
import 'package:totaldrop/ui/receive/receive_view.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

import 'fakes.dart';

final l = lookupAppLocalizations(const Locale('en'));

class _Rust implements RustLibApi {
  @override
  String crateApiL10NRevealControls({required String name}) => name;

  @override
  bool crateApiL10NNeedsEnglish({required String tag, required String key}) =>
      false;

  @override
  QrGrid crateApiTransferQrForCode({required String code}) =>
      QrGrid(modules: '${'10' * 220}1', width: 21);

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _FakeQrScanner extends QrScanner {
  _FakeQrScanner(this.next);

  QrScanResult next;
  int calls = 0;

  @override
  bool get available => true;

  @override
  Future<QrScanResult> scan() async {
    calls++;
    return next;
  }
}

Widget _app(Widget home) => MaterialApp(
  theme: buildTheme(fallbackScheme(Brightness.light)),
  localizationsDelegates: const [
    AppLocalizations.delegate,
    ...GlobalMaterialLocalizations.delegates,
  ],
  supportedLocales: AppLocalizations.supportedLocales,
  builder: (context, child) => SystemMotion(child: child!),
  home: home,
);

void main() {
  setUpAll(() => RustLib.initMock(api: _Rust()));

  group('AndroidQrScanner channel', () {
    const channel = MethodChannel('app.totaldrop/qr_scan');

    tearDown(() {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, null);
    });

    testWidgets('maps native scanner outcomes cleanly', (tester) async {
      Map<String, Object?>? response;
      PlatformException? failure;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            expect(call.method, 'scan');
            if (failure case final e?) throw e;
            return response;
          });

      const scanner = AndroidQrScanner();
      expect(scanner.available, isTrue);

      response = {
        'outcome': 'scanned',
        'link': 'https://totaldrop.app/r#7-harbor-rocket',
      };
      final scanned = await scanner.scan();
      expect(scanned, isA<QrScanned>());
      expect(
        (scanned as QrScanned).link,
        'https://totaldrop.app/r#7-harbor-rocket',
      );

      response = {'outcome': 'denied'};
      expect(await scanner.scan(), isA<QrCameraDenied>());

      response = {'outcome': 'unavailable'};
      expect(await scanner.scan(), isA<QrCameraUnavailable>());

      response = {'outcome': 'cancelled'};
      expect(await scanner.scan(), isA<QrScanCancelled>());

      response = {'outcome': 'scanned'};
      expect(await scanner.scan(), isA<QrScanCancelled>());

      failure = PlatformException(code: 'NO_ACTIVITY');
      expect(await scanner.scan(), isA<QrCameraUnavailable>());
    });
  });

  group('Send QR code', () {
    testWidgets(
      'waiting screen renders high-contrast QR code and enlarges on tap',
      (tester) async {
        final semantics = tester.ensureSemantics();
        final repo = FakeRepository();
        final vm = SendViewModel(repository: repo, fileSource: FakeFileSource())
          ..addFakeFileForTest('photo.jpg');

        await tester.pumpWidget(_app(SendView(viewModel: vm)));
        unawaited(vm.start());
        repo.prepared.complete(FakeSession());
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));

        final list = tester.state<ScrollableState>(
          find.byType(Scrollable).first,
        );
        list.position.jumpTo(list.position.maxScrollExtent);
        await tester.pump();

        expect(find.byType(QrCodeView), findsOneWidget);
        expect(find.text(l.sendQrHint), findsOneWidget);

        await tester.ensureVisible(find.byType(QrCodeView));
        await tester.pump();
        await tester.tap(find.byType(QrCodeView));
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));

        final dialog = tester.widget<Dialog>(find.byType(Dialog));
        expect(dialog.elevation, 0);
        expect(dialog.surfaceTintColor, Colors.transparent);
        expect(find.byType(CloseButton), findsOneWidget);

        final dialogCode = find.descendant(
          of: find.byType(Dialog),
          matching: find.text('7-harbor-rocket'),
        );
        final node = tester.getSemantics(dialogCode);
        expect(
          node.attributedLabel.attributes.whereType<LocaleStringAttribute>(),
          isNotEmpty,
        );

        await tester.tap(find.byType(CloseButton));
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));
        expect(find.byType(Dialog), findsNothing);
        semantics.dispose();
      },
    );

    testWidgets(
      'enlarged QR dialog auto-closes when receiver connects with verification words',
      (tester) async {
        final repo = FakeRepository();
        final vm = SendViewModel(repository: repo, fileSource: FakeFileSource())
          ..addFakeFileForTest('photo.jpg');

        await tester.pumpWidget(_app(SendView(viewModel: vm)));
        unawaited(vm.start());
        repo.prepared.complete(FakeSession());
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));

        final list = tester.state<ScrollableState>(
          find.byType(Scrollable).first,
        );
        list.position.jumpTo(list.position.maxScrollExtent);
        await tester.pump();

        await tester.ensureVisible(find.byType(QrCodeView));
        await tester.pump();
        await tester.tap(find.byType(QrCodeView));
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));
        expect(find.byType(Dialog), findsOneWidget);

        repo.events.add(
          const TransferEvent(
            kind: EventKind.connected,
            words: ['alpha', 'bravo', 'charlie', 'delta'],
            done: 0,
            total: 10,
            summary: null,
            error: null,
          ),
        );
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));

        expect(vm.stage, SendStage.connected);
        expect(find.byType(Dialog), findsNothing);
        expect(find.text('alpha'), findsOneWidget);
        expect(find.text('delta'), findsOneWidget);
      },
    );
  });

  group('Receive QR scanning', () {
    testWidgets('hides Scan QR code when no scanner is wired', (tester) async {
      final vm = ReceiveViewModel(repository: FakeRepository());
      await tester.pumpWidget(_app(ReceiveView(viewModel: vm)));
      expect(find.text(l.receiveScanQr), findsNothing);
    });

    testWidgets(
      'fills valid scanned QR code into input without auto-connecting (T19)',
      (tester) async {
        final repo = FakeRepository();
        final vm = ReceiveViewModel(repository: repo);
        final scanner = _FakeQrScanner(
          const QrScanned('https://totaldrop.app/r#7-harbor-rocket'),
        );

        await tester.pumpWidget(
          _app(ReceiveView(viewModel: vm, qrScanner: scanner)),
        );
        expect(find.text(l.receiveScanQr), findsOneWidget);

        await tester.tap(find.text(l.receiveScanQr));
        await tester.pump();

        expect(scanner.calls, 1);
        expect(vm.stage, ReceiveStage.entering);
        expect(vm.input, '7-harbor-rocket');
        expect(vm.showInvalid, isFalse);
        // Nothing connected automatically; the person still taps Connect.
        expect(repo.stoppers, isEmpty);

        await tester.tap(find.text(l.receiveConnect));
        await tester.pump();
        expect(vm.stage, ReceiveStage.connecting);
        expect(repo.stoppers, hasLength(1));
      },
    );

    testWidgets(
      'shows invalid-code error when scanned link code is rejected by Rust',
      (tester) async {
        final vm = ReceiveViewModel(repository: FakeRepository())
          ..setInput('7-harbor-rocket');
        final scanner = _FakeQrScanner(
          const QrScanned('https://totaldrop.app/r#not-a-valid-code'),
        );

        await tester.pumpWidget(
          _app(ReceiveView(viewModel: vm, qrScanner: scanner)),
        );
        await tester.tap(find.text(l.receiveScanQr));
        await tester.pump();

        expect(vm.input, isEmpty);
        expect(vm.showInvalid, isTrue);
        expect(find.text(l.receiveCodeInvalid), findsOneWidget);
      },
    );

    testWidgets(
      'camera permission denial informs calmly and keeps typing functional',
      (tester) async {
        final semantics = tester.ensureSemantics();
        final repo = FakeRepository();
        final vm = ReceiveViewModel(repository: repo);
        final scanner = _FakeQrScanner(const QrCameraDenied());

        await tester.pumpWidget(
          _app(ReceiveView(viewModel: vm, qrScanner: scanner)),
        );
        await tester.tap(find.text(l.receiveScanQr));
        await tester.pump();

        expect(find.text(l.receiveCameraDenied), findsOneWidget);
        expect(
          tester.getSemantics(find.text(l.receiveCameraDenied)),
          isSemantics(label: l.receiveCameraDenied, isLiveRegion: true),
        );

        // Typing clears the camera notice and allows connecting normally.
        await tester.enterText(find.byType(TextField), '7-harbor-rocket');
        await tester.pump();
        expect(find.text(l.receiveCameraDenied), findsNothing);

        await tester.tap(find.text(l.receiveConnect));
        await tester.pump();
        expect(vm.stage, ReceiveStage.connecting);
        semantics.dispose();
      },
    );

    testWidgets(
      'missing camera notice announces cleanly and cancelled scan keeps input',
      (tester) async {
        final vm = ReceiveViewModel(repository: FakeRepository())
          ..setInput('7-harbor');
        final scanner = _FakeQrScanner(const QrScanCancelled());

        await tester.pumpWidget(
          _app(ReceiveView(viewModel: vm, qrScanner: scanner)),
        );
        await tester.tap(find.text(l.receiveScanQr));
        await tester.pump();
        expect(vm.input, '7-harbor');

        scanner.next = const QrCameraUnavailable();
        await tester.tap(find.text(l.receiveScanQr));
        await tester.pump();
        expect(find.text(l.receiveCameraUnavailable), findsOneWidget);
      },
    );

    test('takeQr accepts trailing-slash web link and refuses active stage', () {
      final repo = FakeRepository();
      final vm = ReceiveViewModel(repository: repo);
      expect(vm.takeQr('https://totaldrop.app/r/#7-harbor-rocket'), isTrue);
      expect(vm.input, '7-harbor-rocket');

      unawaited(vm.connect());
      expect(vm.stage, ReceiveStage.connecting);
      expect(vm.takeQr('https://totaldrop.app/r#9-garden-window'), isFalse);
      expect(vm.connectingCode, '7-harbor-rocket');
    });
  });
}
