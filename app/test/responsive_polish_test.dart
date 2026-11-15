// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Wide-screen 560 dp column constraint and Send Files / Text mode switcher
// (#545, ADR 0011).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/data/background_access.dart';
import 'package:totaldrop/data/settings_repository.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/l10n.dart' show ShownText;
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/src/rust/frb_generated.dart';
import 'package:totaldrop/ui/core/common.dart';
import 'package:totaldrop/ui/core/theme.dart';
import 'package:totaldrop/ui/home/home_view.dart';
import 'package:totaldrop/ui/receive/receive_view.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';
import 'package:totaldrop/ui/settings/settings_view.dart';
import 'package:totaldrop/ui/setup/background_setup_view.dart';

import 'fakes.dart';

class _Rust implements RustLibApi {
  @override
  String crateApiL10NRevealControls({required String name}) => name;

  @override
  bool crateApiL10NNeedsEnglish({required String tag, required String key}) =>
      false;

  @override
  bool crateApiL10NHasDraftTranslation({required String tag}) => false;

  @override
  ShownText crateApiL10NShownText({required String text}) =>
      ShownText(text: text, short: false, hidden: false);

  @override
  String crateApiTransferDefaultServer() => 'wss://rendezvous.totaldrop.app';

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Settings extends ChangeNotifier implements SettingsRepository {
  @override
  String get defaultServer => 'wss://rendezvous.totaldrop.app';
  @override
  String get server => defaultServer;
  @override
  bool get usesDefaultServer => true;
  @override
  bool get privacyMode => false;
  @override
  Future<void> setServer(String url) async {}
  @override
  Future<void> setPrivacyMode(bool on) async {}
  @override
  Future<void> resetServer() async {}
  @override
  bool get backgroundSetupSeen => true;
  @override
  bool get backgroundSkipped => false;
  @override
  bool get backgroundReminded => false;
  @override
  Future<void> setBackgroundSetupSeen({required bool skipped}) async {}
  @override
  Future<void> setBackgroundReminded() async {}
}

class _Limits implements BackgroundAccess {
  const _Limits();

  @override
  Future<BackgroundState> state() async =>
      const BackgroundState(exempt: false, maker: 'Xiaomi', autostart: true);

  @override
  Future<bool> open(BackgroundPage page) async => true;
}

const _tablet = Size(840, 600);
const _phone = Size(360, 640);

Widget _app(
  Widget home, {
  Locale locale = const Locale('en'),
  double textScale = 1.0,
}) => MaterialApp(
  theme: buildTheme(fallbackScheme(Brightness.light)),
  locale: locale,
  localizationsDelegates: const [
    AppLocalizations.delegate,
    ...GlobalMaterialLocalizations.delegates,
  ],
  supportedLocales: AppLocalizations.supportedLocales,
  builder: (context, child) => MediaQuery(
    data: MediaQuery.of(context)
        .copyWith(textScaler: TextScaler.linear(textScale)),
    child: SystemMotion(child: child!),
  ),
  home: home,
);

void _expectCentredColumn(WidgetTester tester, Finder finder) {
  final rect = tester.getRect(finder);
  expect(rect.width, lessThanOrEqualTo(maxColumnWidth + 0.5));
  // 840 dp viewport with a 560 dp centred column leaves 140 dp on each side.
  expect(rect.left, greaterThanOrEqualTo(139.5));
  expect(rect.right, lessThanOrEqualTo(700.5));
}

void main() {
  setUpAll(() => RustLib.initMock(api: _Rust()));

  group('wide-screen 560 dp column (#545)', () {
    testWidgets('HomeView centres action cards within 560 dp on a tablet', (
      tester,
    ) async {
      await tester.binding.setSurfaceSize(_tablet);
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await tester.pumpWidget(
        _app(HomeView(onSend: () {}, onReceive: () {}, onSettings: () {})),
      );
      await tester.pump();

      final cards = find.byType(Card);
      expect(cards, findsNWidgets(2));
      _expectCentredColumn(tester, cards.first);
      _expectCentredColumn(tester, cards.last);
    });

    testWidgets('SendView, ReceiveView, SettingsView and BackgroundSetupView '
        'stay within 560 dp on a tablet', (tester) async {
      await tester.binding.setSurfaceSize(_tablet);
      addTearDown(() => tester.binding.setSurfaceSize(null));

      // SendView: empty file-picking state (SliverFillRemaining), Text mode
      // (SliverFillRemaining), and populated file list + pinned footer.
      final sendVm = SendViewModel(
        repository: FakeRepository(),
        fileSource: FakeFileSource(),
      );
      await tester.pumpWidget(_app(SendView(viewModel: sendVm)));
      await tester.pump();
      _expectCentredColumn(tester, find.byType(SegmentedButton<bool>));
      _expectCentredColumn(tester, find.byType(FilledButton));

      sendVm.setTextMode(true);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));
      _expectCentredColumn(tester, find.byType(TextField));

      sendVm
        ..setTextMode(false)
        ..addFakeFileForTest('archive.tar.zst');
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));
      _expectCentredColumn(tester, find.byType(ListTile).first);
      _expectCentredColumn(tester, find.byType(SwitchListTile));

      // ReceiveView (entering stage + error stage via CenteredScroll).
      final repo = FakeRepository();
      final receiveVm = ReceiveViewModel(repository: repo)
        ..setInput('7-harbor-rocket');
      await tester.pumpWidget(_app(ReceiveView(viewModel: receiveVm)));
      await tester.pump();
      _expectCentredColumn(tester, find.byType(TextField));

      unawaited(receiveVm.connect());
      repo.connected.completeError(
        const TransferError(kind: ErrorKind.connect, detail: '', cause: ''),
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));
      final l = lookupAppLocalizations(const Locale('en'));
      _expectCentredColumn(
        tester,
        find.widgetWithText(FilledButton, l.tryAgain),
      );

      // SettingsView.
      await tester.pumpWidget(
        _app(
          SettingsView(
            settings: _Settings(),
            version: '0.3.0',
            background: const _Limits(),
          ),
        ),
      );
      await tester.pump();
      _expectCentredColumn(tester, find.byType(TextField));

      // BackgroundSetupView.
      await tester.pumpWidget(
        _app(
          BackgroundSetupView(
            access: const _Limits(),
            onDone: ({required skipped}) {},
          ),
        ),
      );
      await tester.pump();
      _expectCentredColumn(tester, find.byType(FilledButton).first);
    });

    testWidgets('phones keep 24 dp margins without shrinking the column', (
      tester,
    ) async {
      await tester.binding.setSurfaceSize(_phone);
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await tester.pumpWidget(
        _app(HomeView(onSend: () {}, onReceive: () {}, onSettings: () {})),
      );
      await tester.pump();

      final rect = tester.getRect(find.byType(Card).first);
      expect(rect.left, closeTo(24, 0.5));
      expect(rect.width, closeTo(312, 0.5));
    });
  });

  group('Send Files / Text mode switcher (#545)', () {
    testWidgets(
      'switches between Files and Text without losing files or draft text',
      (tester) async {
        await tester.binding.setSurfaceSize(_tablet);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        final l = lookupAppLocalizations(const Locale('en'));
        final vm = SendViewModel(
          repository: FakeRepository(),
          fileSource: FakeFileSource(),
        )..addFakeFileForTest('report.pdf');

        await tester.pumpWidget(_app(SendView(viewModel: vm)));
        await tester.pump();

        expect(find.byType(SegmentedButton<bool>), findsOneWidget);
        final theme = Theme.of(
          tester.element(find.byType(SegmentedButton<bool>)),
        );
        expect(
          theme.segmentedButtonTheme.style?.elevation?.resolve(const {}),
          0,
        );
        expect(
          theme.segmentedButtonTheme.style?.surfaceTintColor?.resolve(const {}),
          Colors.transparent,
        );
        expect(find.text('report.pdf'), findsOneWidget);

        // Switch to Text mode even though a file is already picked.
        await tester.tap(find.text(l.sendText));
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));
        expect(vm.textMode, isTrue);

        await tester.enterText(find.byType(TextField), 'secret note');
        await tester.pump();
        expect(vm.text, 'secret note');

        // Switch back to Files: the picked file is still listed and the draft
        // text stays intact when switching back to Text again.
        await tester.tap(find.text(l.sendTitle));
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));
        expect(vm.textMode, isFalse);
        expect(find.text('report.pdf'), findsOneWidget);
        expect(vm.text, 'secret note');

        await tester.tap(find.text(l.sendText));
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));
        expect(vm.textMode, isTrue);
        expect(find.text('secret note'), findsOneWidget);
      },
    );

    testWidgets(
      'handles 200% text scale on a narrow phone in German and Arabic with '
      'TalkBack selection semantics',
      (tester) async {
        await tester.binding.setSurfaceSize(_phone);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        for (final locale in [const Locale('de'), const Locale('ar')]) {
          final l = lookupAppLocalizations(locale);
          final vm = SendViewModel(
            repository: FakeRepository(),
            fileSource: FakeFileSource(),
          );
          await tester.pumpWidget(
            _app(SendView(viewModel: vm), locale: locale, textScale: 2.0),
          );
          await tester.pump();
          expect(tester.takeException(), isNull);
          expect(find.byType(SegmentedButton<bool>), findsNothing);
          expect(
            tester.getSemantics(find.text(l.sendTitle)),
            isSemantics(
              isButton: true,
              isSelected: true,
              isInMutuallyExclusiveGroup: true,
            ),
          );
          expect(
            tester.getSemantics(find.text(l.sendText)),
            isSemantics(
              isButton: true,
              isSelected: false,
              isInMutuallyExclusiveGroup: true,
            ),
          );

          await tester.tap(find.text(l.sendText));
          await tester.pump();
          await tester.pump(const Duration(milliseconds: 300));
          expect(vm.textMode, isTrue);
          expect(tester.takeException(), isNull);
          expect(
            tester.getSemantics(find.text(l.sendTitle)),
            isSemantics(
              isButton: true,
              isSelected: false,
              isInMutuallyExclusiveGroup: true,
            ),
          );
          expect(
            tester.getSemantics(find.text(l.sendText)),
            isSemantics(
              isButton: true,
              isSelected: true,
              isInMutuallyExclusiveGroup: true,
            ),
          );
        }
      },
    );
  });
}
