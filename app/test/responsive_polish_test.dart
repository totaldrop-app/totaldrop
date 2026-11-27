// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Wide-screen 560 dp column constraint and Send files / Send text mode switcher
// (#545, ADR 0011).

import 'dart:async';

import 'package:flutter/rendering.dart' show RenderParagraph;
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/data/background_access.dart';
import 'package:totaldrop/data/settings_repository.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/l10n.dart' show ShownText;
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/src/rust/frb_generated.dart';
import 'package:totaldrop/ui/core/adaptive.dart';
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

Future<void> _loadBundledFonts() async {
  final body = FontLoader('AtkinsonHyperlegible')
    ..addFont(rootBundle.load('assets/fonts/AtkinsonHyperlegible-Regular.ttf'))
    ..addFont(rootBundle.load('assets/fonts/AtkinsonHyperlegible-Bold.ttf'));
  final display = FontLoader('BricolageGrotesque')
    ..addFont(rootBundle.load('assets/fonts/BricolageGrotesque.ttf'));
  await body.load();
  await display.load();
}

Widget _app(
  Widget home, {
  Locale locale = const Locale('en'),
  Brightness brightness = Brightness.light,
  double textScale = 1.0,
}) => MaterialApp(
  theme: buildTheme(fallbackScheme(brightness)),
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

void _expectPaneColumn(WidgetTester tester, Finder finder) {
  final rect = tester.getRect(finder);
  expect(rect.width, lessThanOrEqualTo(maxColumnWidth + 0.5));
  // On an 840 dp tablet two-pane screens split into two 420 dp halves (#632).
  final inLeftPane = rect.left >= 0 && rect.right <= 420.5;
  final inRightPane = rect.left >= 419.5 && rect.right <= 840.5;
  expect(inLeftPane || inRightPane, isTrue, reason: 'rect=$rect');
}

double _findSegmentThresholdWidth({
  required ThemeData theme,
  required TextScaler textScaler,
  required TextDirection textDirection,
  required List<String> labels,
}) {
  var lo = 0.0;
  var hi = 1200.0;
  for (var i = 0; i < 30; i++) {
    final mid = (lo + hi) / 2;
    if (sendModeSwitchFitsSegmented(
      theme: theme,
      textScaler: textScaler,
      textDirection: textDirection,
      availableWidth: mid,
      labels: labels,
    )) {
      hi = mid;
    } else {
      lo = mid;
    }
  }
  return hi;
}

void main() {
  setUpAll(() async {
    TestWidgetsFlutterBinding.ensureInitialized();
    await _loadBundledFonts();
    RustLib.initMock(api: _Rust());
  });

  group('wide-screen 560 dp column (#545)', () {
    for (final variant in [
      (locale: const Locale('en'), brightness: Brightness.light),
      (locale: const Locale('en'), brightness: Brightness.dark),
      (locale: const Locale('ar'), brightness: Brightness.light),
      (locale: const Locale('ar'), brightness: Brightness.dark),
      (locale: const Locale('ja'), brightness: Brightness.light),
      (locale: const Locale('ja'), brightness: Brightness.dark),
    ]) {
      testWidgets(
        'HomeView, SendView, ReceiveView, SettingsView and BackgroundSetupView '
        'stay within 560 dp on a tablet in '
        '${variant.locale.languageCode} (${variant.brightness.name})',
        (tester) async {
          await tester.binding.setSurfaceSize(_tablet);
          addTearDown(() => tester.binding.setSurfaceSize(null));

          final l = lookupAppLocalizations(variant.locale);

          // HomeView (two-pane on 840 dp tablet, #632).
          await tester.pumpWidget(
            _app(
              HomeView(onSend: () {}, onReceive: () {}, onSettings: () {}),
              locale: variant.locale,
              brightness: variant.brightness,
            ),
          );
          await tester.pump();
          expect(tester.takeException(), isNull);
          final cards = find.byType(Card);
          expect(cards, findsNWidgets(2));
          _expectPaneColumn(tester, cards.first);
          _expectPaneColumn(tester, cards.last);

          // SendView: empty file picking, text mode, and populated file list.
          final sendVm = SendViewModel(
            repository: FakeRepository(),
            fileSource: FakeFileSource(),
          );
          await tester.pumpWidget(
            _app(
              SendView(viewModel: sendVm),
              locale: variant.locale,
              brightness: variant.brightness,
            ),
          );
          await tester.pump();
          expect(tester.takeException(), isNull);
          _expectPaneColumn(tester, find.byType(SegmentedButton<bool>));
          _expectPaneColumn(tester, find.byType(FilledButton));

          sendVm.setTextMode(true);
          await tester.pump();
          await tester.pump(const Duration(milliseconds: 300));
          expect(tester.takeException(), isNull);
          _expectPaneColumn(tester, find.byType(TextField));

          sendVm
            ..setTextMode(false)
            ..addFakeFileForTest('archive.tar.zst');
          await tester.pump();
          await tester.pump(const Duration(milliseconds: 300));
          expect(tester.takeException(), isNull);
          _expectPaneColumn(tester, find.byType(ListTile).first);
          _expectPaneColumn(tester, find.byType(SwitchListTile));

          // ReceiveView (two-pane entering stage + centred single-pane error).
          final repo = FakeRepository();
          final receiveVm = ReceiveViewModel(repository: repo)
            ..setInput('7-harbor-rocket');
          await tester.pumpWidget(
            _app(
              ReceiveView(viewModel: receiveVm),
              locale: variant.locale,
              brightness: variant.brightness,
            ),
          );
          await tester.pump();
          expect(tester.takeException(), isNull);
          _expectPaneColumn(tester, find.byType(TextField));

          unawaited(receiveVm.connect());
          repo.connected.completeError(
            const TransferError(kind: ErrorKind.connect, detail: '', cause: ''),
          );
          await tester.pump();
          await tester.pump(const Duration(milliseconds: 300));
          expect(tester.takeException(), isNull);
          _expectCentredColumn(
            tester,
            find.widgetWithText(FilledButton, l.tryAgain),
          );

          // SettingsView (single-column 560 dp on flat tablet, #545, #632).
          await tester.pumpWidget(
            _app(
              SettingsView(
                settings: _Settings(),
                version: '0.3.0',
                background: const _Limits(),
              ),
              locale: variant.locale,
              brightness: variant.brightness,
            ),
          );
          await tester.pump();
          expect(tester.takeException(), isNull);
          _expectCentredColumn(tester, find.byType(TextField));

          // BackgroundSetupView (two-pane on 840 dp tablet, #632).
          await tester.pumpWidget(
            _app(
              BackgroundSetupView(
                access: const _Limits(),
                onDone: ({required skipped}) {},
              ),
              locale: variant.locale,
              brightness: variant.brightness,
            ),
          );
          await tester.pump();
          expect(tester.takeException(), isNull);
          _expectPaneColumn(tester, find.byType(FilledButton).first);
        },
      );
    }

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

  group('Send files / Send text mode switcher (#545)', () {
    testWidgets(
      'keeps the AppBar TalkBack heading while picking and updates on mode switch',
      (tester) async {
        await tester.binding.setSurfaceSize(_phone);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        final l = lookupAppLocalizations(const Locale('en'));
        final vm = SendViewModel(
          repository: FakeRepository(),
          fileSource: FakeFileSource(),
        );
        await tester.pumpWidget(_app(SendView(viewModel: vm)));
        await tester.pump();

        final filesHeading = find.descendant(
          of: find.byType(AppBar),
          matching: find.text(l.sendTitle),
        );
        expect(filesHeading, findsOneWidget);
        expect(
          tester.getSemantics(filesHeading),
          isSemantics(isHeader: true, label: l.sendTitle),
        );

        await tester.tap(
          find.descendant(
            of: find.byType(SegmentedButton<bool>),
            matching: find.text(l.sendText),
          ),
        );
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));

        final textHeading = find.descendant(
          of: find.byType(AppBar),
          matching: find.text(l.sendText),
        );
        expect(textHeading, findsOneWidget);
        expect(
          tester.getSemantics(textHeading),
          isSemantics(isHeader: true, label: l.sendText),
        );
      },
    );

    testWidgets(
      'switches between Send files and Send text without losing files or draft text',
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

        final segmented = find.byType(SegmentedButton<bool>);
        expect(segmented, findsOneWidget);
        final theme = Theme.of(tester.element(segmented));
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
        await tester.tap(
          find.descendant(of: segmented, matching: find.text(l.sendText)),
        );
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));
        expect(vm.textMode, isTrue);

        await tester.enterText(find.byType(TextField), 'secret note');
        await tester.pump();
        expect(vm.text, 'secret note');

        // Switch back to Files: the picked file is still listed and the draft
        // text stays intact when switching back to Text again.
        await tester.tap(
          find.descendant(of: segmented, matching: find.text(l.sendTitle)),
        );
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));
        expect(vm.textMode, isFalse);
        expect(find.text('report.pdf'), findsOneWidget);
        expect(vm.text, 'secret note');

        await tester.tap(
          find.descendant(of: segmented, matching: find.text(l.sendText)),
        );
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));
        expect(vm.textMode, isTrue);
        expect(find.text('secret note'), findsOneWidget);
      },
    );

    testWidgets('keeps SegmentedButton on a 360 dp phone at 1.0x across short and long '
        'translations by wrapping up to 2 lines', (tester) async {
      await tester.binding.setSurfaceSize(_phone);
      addTearDown(() => tester.binding.setSurfaceSize(null));

      for (final locale in const [
        Locale('en'),
        Locale('de'),
        Locale('fr'),
        Locale('nl'),
        Locale('ar'),
        Locale('ja'),
      ]) {
        final l = lookupAppLocalizations(locale);
        final vm = SendViewModel(
          repository: FakeRepository(),
          fileSource: FakeFileSource(),
        );
        await tester.pumpWidget(
          _app(SendView(viewModel: vm), locale: locale, textScale: 1.0),
        );
        await tester.pump();
        expect(
          tester.takeException(),
          isNull,
          reason: 'locale ${locale.languageCode} overflowed at 1.0x',
        );
        final segmented = find.byType(SegmentedButton<bool>);
        expect(
          segmented,
          findsOneWidget,
          reason:
              'locale ${locale.languageCode} should stay on SegmentedButton '
              'at 360 dp / 1.0x',
        );
        for (final label in [l.sendTitle, l.sendText]) {
          final textFinder = find.descendant(
            of: segmented,
            matching: find.text(label),
          );
          expect(textFinder, findsOneWidget);
          final paragraph = tester.renderObject<RenderParagraph>(textFinder);
          expect(
            paragraph.didExceedMaxLines,
            isFalse,
            reason:
                'locale ${locale.languageCode} label "$label" clipped past maxLines',
          );
          expect(
            paragraph.size.width,
            lessThanOrEqualTo(paragraph.constraints.maxWidth),
            reason:
                'locale ${locale.languageCode} label "$label" exceeded maxWidth',
          );
        }
      }
    });

    testWidgets(
      'switches cleanly across the exact theme-derived threshold width at 1.0x '
      'and 2.0x and renders unclipped segment paragraphs at threshold + 2',
      (tester) async {
        final theme = buildTheme(fallbackScheme(Brightness.light));
        final l = lookupAppLocalizations(const Locale('en'));
        final labels = [l.sendTitle, l.sendText];

        for (final scale in const [1.0, 2.0]) {
          final threshold = _findSegmentThresholdWidth(
            theme: theme,
            textScaler: TextScaler.linear(scale),
            textDirection: TextDirection.ltr,
            labels: labels,
          );
          expect(threshold, greaterThan(100));
          expect(threshold, lessThan(1000));

          // columnPadding adds 32 dp horizontal padding under
          // coverScreenThreshold (360 dp), 48 dp up to 608 dp, and caps inner
          // width at maxColumnWidth (560 dp) above 608 dp (#545, #632).
          for (final shouldFit in const [false, true]) {
            final innerWidth = shouldFit ? threshold + 2.0 : threshold - 2.0;
            if (innerWidth > maxColumnWidth) continue;
            final viewportWidth = innerWidth + 48.0 >= coverScreenThreshold
                ? innerWidth + 48.0
                : innerWidth + 32.0;

            await tester.binding.setSurfaceSize(Size(viewportWidth, 640));
            final vm = SendViewModel(
              repository: FakeRepository(),
              fileSource: FakeFileSource(),
            );
            await tester.pumpWidget(
              _app(SendView(viewModel: vm), textScale: scale),
            );
            await tester.pump();
            expect(tester.takeException(), isNull);
            final segmented = find.byType(SegmentedButton<bool>);
            expect(
              segmented,
              shouldFit ? findsOneWidget : findsNothing,
              reason:
                  'scale=$scale innerWidth=${innerWidth.toStringAsFixed(1)} '
                  '(threshold=${threshold.toStringAsFixed(1)})',
            );
            if (shouldFit) {
              // Verify the real rendered SegmentedButton (not just the
              // measurement helper) fits both labels without clipping so any
              // future material_ui chrome drift fails this test.
              for (final label in labels) {
                final textFinder = find.descendant(
                  of: segmented,
                  matching: find.text(label),
                );
                expect(textFinder, findsOneWidget);
                final paragraph = tester.renderObject<RenderParagraph>(
                  textFinder,
                );
                expect(
                  paragraph.didExceedMaxLines,
                  isFalse,
                  reason:
                      'segment label "$label" exceeded maxLines at '
                      'scale=$scale innerWidth=${innerWidth.toStringAsFixed(1)}',
                );
                expect(
                  paragraph.size.width,
                  lessThanOrEqualTo(paragraph.constraints.maxWidth),
                  reason:
                      'segment label "$label" exceeded maxWidth at '
                      'scale=$scale innerWidth=${innerWidth.toStringAsFixed(1)}',
                );
              }
            }
          }
        }
        await tester.binding.setSurfaceSize(null);
      },
    );

    testWidgets(
      'handles 200% text scale on a narrow phone in English, German, French '
      'and Arabic with stacked TalkBack semantics and minimum spacing',
      (tester) async {
        await tester.binding.setSurfaceSize(_phone);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        for (final locale in const [
          Locale('en'),
          Locale('de'),
          Locale('fr'),
          Locale('ar'),
        ]) {
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
          expect(
            find.byType(SegmentedButton<bool>),
            findsNothing,
            reason:
                'locale ${locale.languageCode} stayed on SegmentedButton at '
                '360 dp / 2.0x',
          );

          final filesBtn = find.widgetWithText(FilledButton, l.sendTitle);
          final textBtn = find.widgetWithText(OutlinedButton, l.sendText);
          expect(filesBtn, findsOneWidget);
          expect(textBtn, findsOneWidget);

          // Gap.m (16 dp) keeps clearance even when Spacer collapses to 0.
          final switchBottom = tester.getRect(textBtn).bottom;
          final uploadTop = tester
              .getRect(find.byIcon(Icons.upload_file_outlined))
              .top;
          expect(uploadTop - switchBottom, greaterThanOrEqualTo(15.5));

          final hintBottom = tester.getRect(find.text(l.sendEmptyHint)).bottom;
          final addFilesTop = tester
              .getRect(find.widgetWithText(FilledButton, l.sendAddFiles))
              .top;
          expect(addFilesTop - hintBottom, greaterThanOrEqualTo(15.5));

          expect(
            tester.getSemantics(filesBtn),
            isSemantics(
              isButton: true,
              isSelected: true,
              isInMutuallyExclusiveGroup: true,
            ),
          );
          expect(
            tester.getSemantics(textBtn),
            isSemantics(
              isButton: true,
              isSelected: false,
              isInMutuallyExclusiveGroup: true,
            ),
          );

          await tester.tap(textBtn);
          await tester.pump();
          await tester.pump(const Duration(milliseconds: 300));
          expect(vm.textMode, isTrue);
          expect(tester.takeException(), isNull);
          expect(
            tester.getSemantics(
              find.widgetWithText(OutlinedButton, l.sendTitle),
            ),
            isSemantics(
              isButton: true,
              isSelected: false,
              isInMutuallyExclusiveGroup: true,
            ),
          );
          expect(
            tester.getSemantics(find.widgetWithText(FilledButton, l.sendText)),
            isSemantics(
              isButton: true,
              isSelected: true,
              isInMutuallyExclusiveGroup: true,
            ),
          );
        }

        // Japanese also renders cleanly at 200% on a 360 dp phone (note:
        // AtkinsonHyperlegible.ttf omits CJK glyphs in headless tests, so
        // real-device CJK wrapping is additionally verified visually).
        final jaVm = SendViewModel(
          repository: FakeRepository(),
          fileSource: FakeFileSource(),
        );
        await tester.pumpWidget(
          _app(
            SendView(viewModel: jaVm),
            locale: const Locale('ja'),
            textScale: 2.0,
          ),
        );
        await tester.pump();
        expect(tester.takeException(), isNull);
      },
    );
  });
}
