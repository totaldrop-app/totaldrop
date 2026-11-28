// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Foldable & large-screen adaptive layout tests (#632): posture detection,
// asymmetric hinge avoidance in LTR and RTL, SafeArea/AppBar origin geometry,
// tabletop keyboard focus retention, single-pane hinge avoidance, fold/unfold
// state continuity, narrow cover-screen ergonomics, and full screen/stage
// matrix across light/dark themes, English/Arabic/Japanese/German locales,
// 1.5x Medium two-pane, and 200% text scale.

import 'dart:async';
import 'dart:ui' show DisplayFeature, DisplayFeatureState, DisplayFeatureType;

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart'
    show PlatformInt64;
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/data/background_access.dart';
import 'package:totaldrop/data/settings_repository.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/l10n.dart' show ShownText;
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/src/rust/frb_generated.dart';
import 'package:totaldrop/ui/core/adaptive.dart';
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
  String? crateApiL10NSingleWebUrl({required String text}) =>
      text.startsWith('https://') ? text : null;

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
      const BackgroundState(exempt: false, maker: 'Samsung', autostart: true);
  @override
  Future<bool> open(BackgroundPage page) async => true;
}

class _MultiFileOffer extends FakeOffer {
  _MultiFileOffer();

  static final _names = List<String>.generate(
    10,
    (i) => 'Quarterly-Financial-Statement-2026-Part-${i + 1}-Final.pdf',
  );

  @override
  PlatformInt64 get files => _names.length;

  @override
  List<String> get names => _names;

  @override
  PlatformInt64 get totalBytes => 48 * 1024 * 1024;
}

const _failedEvent = TransferEvent(
  kind: EventKind.failed,
  words: [],
  done: 0,
  total: 0,
  summary: null,
  error: TransferError(
    kind: ErrorKind.connect,
    detail:
        'could not connect to the other device: timed out after 30 s; '
        'tried the local network, the direct path and the relay '
        'wss://relay.totaldrop.app',
    cause: 'TimedOut',
  ),
);

const _finishedFilesEvent = TransferEvent(
  kind: EventKind.finished,
  words: [],
  done: 0,
  total: 0,
  summary: TransferSummary(
    filesOk: 3,
    filesFailed: 0,
    bytes: 12345678,
    saved: ['/tmp/TotalDrop/report.pdf'],
    text: null,
  ),
  error: null,
);

Widget _wrap({
  required Widget home,
  required Size size,
  Locale locale = const Locale('en'),
  Brightness brightness = Brightness.light,
  double textScale = 1.0,
  List<DisplayFeature> features = const [],
  EdgeInsets viewPadding = EdgeInsets.zero,
  EdgeInsets viewInsets = EdgeInsets.zero,
}) => MaterialApp(
  theme: buildTheme(fallbackScheme(brightness)),
  locale: locale,
  localizationsDelegates: const [
    AppLocalizations.delegate,
    ...GlobalMaterialLocalizations.delegates,
  ],
  supportedLocales: AppLocalizations.supportedLocales,
  builder: (context, child) => MediaQuery(
    data: MediaQuery.of(context).copyWith(
      size: size,
      textScaler: TextScaler.linear(textScale),
      displayFeatures: features,
      padding: viewPadding,
      viewPadding: viewPadding,
      viewInsets: viewInsets,
    ),
    child: SystemMotion(child: child!),
  ),
  home: home,
);

void main() {
  setUpAll(() => RustLib.initMock(api: _Rust()));

  group('AdaptiveMetrics.fromMediaQuery', () {
    test('classifies cover, phone, medium, and expanded flat screens', () {
      final cover = AdaptiveMetrics.fromMediaQuery(
        const MediaQueryData(size: Size(320, 740)),
      );
      expect(cover.widthClass, WindowWidthClass.compact);
      expect(cover.isCoverScreen, isTrue);
      expect(cover.horizontalMargin, 16);
      expect(cover.posture, FoldPosture.flat);
      expect(cover.useTwoPane, isFalse);

      final phone = AdaptiveMetrics.fromMediaQuery(
        const MediaQueryData(size: Size(360, 740)),
      );
      expect(phone.widthClass, WindowWidthClass.compact);
      expect(phone.isCoverScreen, isFalse);
      expect(phone.horizontalMargin, 24);
      expect(phone.useTwoPane, isFalse);

      final medium = AdaptiveMetrics.fromMediaQuery(
        const MediaQueryData(size: Size(673, 841)),
      );
      expect(medium.widthClass, WindowWidthClass.medium);
      expect(medium.useTwoPane, isTrue);

      final medium200 = AdaptiveMetrics.fromMediaQuery(
        const MediaQueryData(
          size: Size(673, 841),
          textScaler: TextScaler.linear(2.0),
        ),
      );
      expect(medium200.widthClass, WindowWidthClass.medium);
      expect(medium200.useTwoPane, isFalse);

      final expanded200 = AdaptiveMetrics.fromMediaQuery(
        const MediaQueryData(
          size: Size(840, 600),
          textScaler: TextScaler.linear(2.0),
        ),
      );
      expect(expanded200.widthClass, WindowWidthClass.expanded);
      expect(expanded200.useTwoPane, isTrue);
    });

    test('detects book, tabletop, flat fold, hinge, and enforces pane floors', () {
      const verticalFoldHalf = DisplayFeature(
        bounds: Rect.fromLTWH(400, 0, 0, 600),
        type: DisplayFeatureType.fold,
        state: DisplayFeatureState.postureHalfOpened,
      );
      final book = AdaptiveMetrics.fromMediaQuery(
        const MediaQueryData(
          size: Size(800, 600),
          displayFeatures: [verticalFoldHalf],
        ),
      );
      expect(book.posture, FoldPosture.book);
      expect(book.useTwoPane, isTrue);
      expect(book.hingeBounds, verticalFoldHalf.bounds);

      const verticalFoldFlat = DisplayFeature(
        bounds: Rect.fromLTWH(400, 0, 0, 600),
        type: DisplayFeatureType.fold,
        state: DisplayFeatureState.postureFlat,
      );
      final flatFold = AdaptiveMetrics.fromMediaQuery(
        const MediaQueryData(
          size: Size(800, 600),
          displayFeatures: [verticalFoldFlat],
        ),
      );
      expect(flatFold.posture, FoldPosture.flat);
      expect(flatFold.hingeBounds, isNull);

      const physicalHinge = DisplayFeature(
        bounds: Rect.fromLTWH(406, 0, 28, 600),
        type: DisplayFeatureType.hinge,
        state: DisplayFeatureState.unknown,
      );
      final dualScreen = AdaptiveMetrics.fromMediaQuery(
        const MediaQueryData(
          size: Size(840, 600),
          displayFeatures: [physicalHinge],
        ),
      );
      expect(dualScreen.posture, FoldPosture.book);
      expect(dualScreen.hingeBounds, physicalHinge.bounds);

      const horizontalFold = DisplayFeature(
        bounds: Rect.fromLTWH(0, 420, 673, 0),
        type: DisplayFeatureType.fold,
        state: DisplayFeatureState.postureHalfOpened,
      );
      final tabletop = AdaptiveMetrics.fromMediaQuery(
        const MediaQueryData(
          size: Size(673, 841),
          displayFeatures: [horizontalFold],
        ),
      );
      expect(tabletop.posture, FoldPosture.tabletop);
      expect(tabletop.useTwoPane, isFalse);
      expect(tabletop.hingeBounds, horizontalFold.bounds);

      const cutout = DisplayFeature(
        bounds: Rect.fromLTWH(160, 0, 40, 28),
        type: DisplayFeatureType.cutout,
        state: DisplayFeatureState.unknown,
      );
      final withCutout = AdaptiveMetrics.fromMediaQuery(
        const MediaQueryData(size: Size(360, 740), displayFeatures: [cutout]),
      );
      expect(withCutout.posture, FoldPosture.flat);

      // A hinge leaving a pane narrower than minHorizontalPaneWidth (280 dp) collapses.
      const tooNarrow = DisplayFeature(
        bounds: Rect.fromLTWH(270, 0, 24, 600),
        type: DisplayFeatureType.hinge,
        state: DisplayFeatureState.unknown,
      );
      final collapsed = AdaptiveMetrics.fromMediaQuery(
        const MediaQueryData(
          size: Size(700, 600),
          displayFeatures: [tooNarrow],
        ),
      );
      expect(collapsed.posture, FoldPosture.flat);

      // Exactly 280 dp on both sides qualifies as book posture.
      const exactFloor = DisplayFeature(
        bounds: Rect.fromLTWH(280, 0, 24, 600),
        type: DisplayFeatureType.hinge,
        state: DisplayFeatureState.unknown,
      );
      final atFloor = AdaptiveMetrics.fromMediaQuery(
        const MediaQueryData(
          size: Size(584, 600),
          displayFeatures: [exactFloor],
        ),
      );
      expect(atFloor.posture, FoldPosture.book);

      // Non-zero featureOrigin translates hinge into local coordinates cleanly.
      const offsetHinge = DisplayFeature(
        bounds: Rect.fromLTWH(500, 0, 28, 600),
        type: DisplayFeatureType.hinge,
        state: DisplayFeatureState.unknown,
      );
      final withOrigin = AdaptiveMetrics.fromViewport(
        size: const Size(792, 600),
        displayFeatures: const [offsetHinge],
        textScaler: TextScaler.noScaling,
        featureOrigin: const Offset(48, 0),
      );
      expect(withOrigin.posture, FoldPosture.book);
    });
  });

  group('AdaptivePaneLayout & AdaptiveSinglePane geometry', () {
    testWidgets('avoids asymmetric vertical hinge in both LTR and RTL', (
      tester,
    ) async {
      // Asymmetric hinge: 300 dp physical left, 28 dp gap, 512 dp physical right.
      const size = Size(840, 600);
      const hinge = Rect.fromLTWH(300, 0, 28, 600);
      const feature = DisplayFeature(
        bounds: hinge,
        type: DisplayFeatureType.hinge,
        state: DisplayFeatureState.unknown,
      );
      const primaryKey = ValueKey('primary');
      const secondaryKey = ValueKey('secondary');

      Widget buildScaffold() => const Scaffold(
        body: AdaptivePaneLayout(
          primary: SizedBox.expand(key: primaryKey),
          secondary: SizedBox.expand(key: secondaryKey),
          singlePane: SizedBox.expand(),
        ),
      );

      await tester.binding.setSurfaceSize(size);
      addTearDown(() => tester.binding.setSurfaceSize(null));

      // LTR: primary on physical left (0..300), secondary on physical right (328..840).
      await tester.pumpWidget(
        _wrap(
          home: buildScaffold(),
          size: size,
          locale: const Locale('en'),
          features: const [feature],
        ),
      );
      final ltrPrimary = tester.getRect(find.byKey(primaryKey));
      final ltrSecondary = tester.getRect(find.byKey(secondaryKey));
      expect(ltrPrimary.left, 0);
      expect(ltrPrimary.right, 300);
      expect(ltrSecondary.left, 328);
      expect(ltrSecondary.right, 840);
      expect(ltrPrimary.overlaps(hinge), isFalse);
      expect(ltrSecondary.overlaps(hinge), isFalse);

      // RTL (Arabic): primary mirrored to physical right (328..840), secondary on physical left (0..300).
      await tester.pumpWidget(
        _wrap(
          home: buildScaffold(),
          size: size,
          locale: const Locale('ar'),
          features: const [feature],
        ),
      );
      final rtlPrimary = tester.getRect(find.byKey(primaryKey));
      final rtlSecondary = tester.getRect(find.byKey(secondaryKey));
      expect(rtlPrimary.left, 328);
      expect(rtlPrimary.right, 840);
      expect(rtlSecondary.left, 0);
      expect(rtlSecondary.right, 300);
      expect(rtlPrimary.overlaps(hinge), isFalse);
      expect(rtlSecondary.overlaps(hinge), isFalse);
    });

    testWidgets(
      'aligns Book hinge cleanly both with and without SafeArea when viewPadding.left > 0',
      (tester) async {
        const size = Size(840, 600);
        const hinge = Rect.fromLTWH(406, 0, 28, 600);
        const feature = DisplayFeature(
          bounds: hinge,
          type: DisplayFeatureType.hinge,
          state: DisplayFeatureState.unknown,
        );
        const primaryKey = ValueKey('primary');
        const secondaryKey = ValueKey('secondary');

        await tester.binding.setSurfaceSize(size);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        // 1) With SafeArea consuming viewPadding.left = 48.
        await tester.pumpWidget(
          _wrap(
            home: const Scaffold(
              body: SafeArea(
                child: AdaptivePaneLayout(
                  primary: SizedBox.expand(key: primaryKey),
                  secondary: SizedBox.expand(key: secondaryKey),
                  singlePane: SizedBox.expand(),
                ),
              ),
            ),
            size: size,
            features: const [feature],
            viewPadding: const EdgeInsets.only(left: 48),
          ),
        );
        final withSafePrimary = tester.getRect(find.byKey(primaryKey));
        final withSafeSecondary = tester.getRect(find.byKey(secondaryKey));
        expect(withSafePrimary.left, 48);
        expect(withSafePrimary.right, 406);
        expect(withSafeSecondary.left, 434);
        expect(withSafeSecondary.right, 840);
        expect(withSafePrimary.overlaps(hinge), isFalse);
        expect(withSafeSecondary.overlaps(hinge), isFalse);

        // 2) Without SafeArea (viewPadding.left = 48 unconsumed).
        await tester.pumpWidget(
          _wrap(
            home: const Scaffold(
              body: AdaptivePaneLayout(
                primary: SizedBox.expand(key: primaryKey),
                secondary: SizedBox.expand(key: secondaryKey),
                singlePane: SizedBox.expand(),
              ),
            ),
            size: size,
            features: const [feature],
            viewPadding: const EdgeInsets.only(left: 48),
          ),
        );
        final noSafePrimary = tester.getRect(find.byKey(primaryKey));
        final noSafeSecondary = tester.getRect(find.byKey(secondaryKey));
        expect(noSafePrimary.left, 0);
        expect(noSafePrimary.right, 406);
        expect(noSafeSecondary.left, 434);
        expect(noSafeSecondary.right, 840);
        expect(noSafePrimary.overlaps(hinge), isFalse);
        expect(noSafeSecondary.overlaps(hinge), isFalse);
      },
    );

    testWidgets(
      'aligns tabletop fold with AppBar + status bar, collapses on keyboard, and updates isTabletopOf',
      (tester) async {
        const size = Size(673, 841);
        const hinge = Rect.fromLTWH(0, 420, 673, 12);
        const feature = DisplayFeature(
          bounds: hinge,
          type: DisplayFeatureType.fold,
          state: DisplayFeatureState.postureHalfOpened,
        );
        const primaryKey = ValueKey('top');
        const secondaryKey = ValueKey('bottom');
        const singleKey = ValueKey('single');
        bool? observedTabletop;

        Widget buildScaffold() => Scaffold(
          appBar: AppBar(title: const Text('Test')),
          body: SafeArea(
            child: AdaptivePaneLayout(
              primary: Builder(
                builder: (context) {
                  observedTabletop = AdaptivePaneLayout.isTabletopOf(context);
                  return const SizedBox.expand(key: primaryKey);
                },
              ),
              secondary: const SizedBox.expand(key: secondaryKey),
              singlePane: Builder(
                builder: (context) {
                  observedTabletop = AdaptivePaneLayout.isTabletopOf(context);
                  return const SizedBox.expand(key: singleKey);
                },
              ),
            ),
          ),
        );

        await tester.binding.setSurfaceSize(size);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        await tester.pumpWidget(
          _wrap(
            home: buildScaffold(),
            size: size,
            features: const [feature],
            viewPadding: const EdgeInsets.only(top: 32),
          ),
        );
        final topRect = tester.getRect(find.byKey(primaryKey));
        final bottomRect = tester.getRect(find.byKey(secondaryKey));
        expect(observedTabletop, isTrue);
        expect(topRect.bottom, lessThanOrEqualTo(hinge.top + 0.1));
        expect(bottomRect.top, greaterThanOrEqualTo(hinge.bottom - 0.1));
        expect(topRect.overlaps(hinge), isFalse);
        expect(bottomRect.overlaps(hinge), isFalse);

        // Soft keyboard covers most of the lower pane -> gracefully collapses.
        await tester.pumpWidget(
          _wrap(
            home: buildScaffold(),
            size: size,
            features: const [feature],
            viewPadding: const EdgeInsets.only(top: 32),
            viewInsets: const EdgeInsets.only(bottom: 340),
          ),
        );
        expect(find.byKey(singleKey), findsOneWidget);
        expect(find.byKey(primaryKey), findsNothing);
        expect(observedTabletop, isFalse);
      },
    );

    testWidgets(
      'SettingsView stays single-column in Tabletop and flat tablet, splits around Book hinge',
      (tester) async {
        final settings = _Settings();
        const tabletopSize = Size(673, 841);
        const tabletopFeature = DisplayFeature(
          bounds: Rect.fromLTWH(0, 420, 673, 0),
          type: DisplayFeatureType.fold,
          state: DisplayFeatureState.postureHalfOpened,
        );
        const bookSize = Size(840, 600);
        const bookHinge = Rect.fromLTWH(406, 0, 28, 600);
        const bookFeature = DisplayFeature(
          bounds: bookHinge,
          type: DisplayFeatureType.hinge,
          state: DisplayFeatureState.unknown,
        );

        Widget buildSettings() => SettingsView(
          settings: settings,
          version: '0.3.0-beta.7',
          background: const _Limits(),
        );

        await tester.binding.setSurfaceSize(tabletopSize);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        // Tabletop: single scrollable ListView (not split into two ListViews).
        await tester.pumpWidget(
          _wrap(
            home: buildSettings(),
            size: tabletopSize,
            features: const [tabletopFeature],
          ),
        );
        await tester.pump();
        expect(find.byType(ListView), findsOneWidget);

        // Flat tablet: single scrollable ListView (allowFlatTwoPane: false).
        await tester.binding.setSurfaceSize(const Size(1280, 800));
        await tester.pumpWidget(
          _wrap(home: buildSettings(), size: const Size(1280, 800)),
        );
        await tester.pump();
        expect(find.byType(ListView), findsOneWidget);

        // Book: two ListViews strictly on opposite sides of the vertical hinge.
        await tester.binding.setSurfaceSize(bookSize);
        await tester.pumpWidget(
          _wrap(
            home: buildSettings(),
            size: bookSize,
            features: const [bookFeature],
          ),
        );
        await tester.pump();
        final lists = find.byType(ListView);
        expect(lists, findsNWidgets(2));
        final first = tester.getRect(lists.at(0));
        final second = tester.getRect(lists.at(1));
        expect(first.overlaps(bookHinge), isFalse);
        expect(second.overlaps(bookHinge), isFalse);
        expect(first.right, lessThanOrEqualTo(bookHinge.left + 0.1));
        expect(second.left, greaterThanOrEqualTo(bookHinge.right - 0.1));
      },
    );

    testWidgets(
      'single-pane Send and Receive stages avoid physical vertical hinge in Book posture',
      (tester) async {
        const size = Size(840, 600);
        const hinge = Rect.fromLTWH(406, 0, 28, 600);
        const feature = DisplayFeature(
          bounds: hinge,
          type: DisplayFeatureType.hinge,
          state: DisplayFeatureState.unknown,
        );

        await tester.binding.setSurfaceSize(size);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        Future<void> verifyAvoidsHinge(
          String label,
          Widget widget,
          Future<void> Function() reach, {
          int expectedPanes = 1,
        }) async {
          await tester.pumpWidget(
            _wrap(home: widget, size: size, features: const [feature]),
          );
          await reach();
          await tester.pump();
          await tester.pump(const Duration(milliseconds: 300));
          final scrolls = find.byType(SingleChildScrollView);
          expect(
            scrolls,
            findsNWidgets(expectedPanes),
            reason: '$label renders $expectedPanes CenteredScroll pane(s)',
          );
          final firstRect = tester.getRect(scrolls.first);
          expect(
            firstRect.overlaps(hinge),
            isFalse,
            reason: '$label primary pane must not straddle physical hinge',
          );
          expect(firstRect.right, lessThanOrEqualTo(hinge.left + 0.1));
          if (expectedPanes == 2) {
            final secondRect = tester.getRect(scrolls.last);
            expect(
              secondRect.overlaps(hinge),
              isFalse,
              reason: '$label control pane must not straddle physical hinge',
            );
            expect(secondRect.left, greaterThanOrEqualTo(hinge.right - 0.1));
          }
        }

        // SendStage.preparing
        {
          final repo = FakeRepository();
          final vm = SendViewModel(
            repository: repo,
            fileSource: FakeFileSource(),
          )..addFakeFileForTest('a.pdf');
          await verifyAvoidsHinge(
            'SendStage.preparing',
            SendView(viewModel: vm),
            () async {
              unawaited(vm.start());
              expect(vm.stage, SendStage.preparing);
            },
          );
        }

        // SendStage.done
        {
          final repo = FakeRepository();
          final vm = SendViewModel(
            repository: repo,
            fileSource: FakeFileSource(),
          )..addFakeFileForTest('a.pdf');
          await verifyAvoidsHinge(
            'SendStage.done',
            SendView(viewModel: vm),
            () async {
              unawaited(vm.start());
              repo.prepared.complete(FakeSession());
              await tester.pump();
              repo.events.add(_finishedFilesEvent);
              await tester.pump();
              expect(vm.stage, SendStage.done);
            },
            expectedPanes: 2,
          );
        }

        // SendStage.failed
        {
          final repo = FakeRepository();
          final vm = SendViewModel(
            repository: repo,
            fileSource: FakeFileSource(),
          )..addFakeFileForTest('a.pdf');
          await verifyAvoidsHinge(
            'SendStage.failed',
            SendView(viewModel: vm),
            () async {
              unawaited(vm.start());
              repo.prepared.complete(FakeSession());
              await tester.pump();
              repo.events.add(_failedEvent);
              await tester.pump();
              expect(vm.stage, SendStage.failed);
            },
            expectedPanes: 2,
          );
        }

        // ReceiveStage.connecting
        {
          final repo = FakeRepository();
          final vm = ReceiveViewModel(repository: repo)
            ..setInput('7-harbor-rocket');
          await verifyAvoidsHinge(
            'ReceiveStage.connecting',
            ReceiveView(viewModel: vm),
            () async {
              unawaited(vm.connect());
              expect(vm.stage, ReceiveStage.connecting);
            },
          );
        }

        // ReceiveStage.declined
        {
          final repo = FakeRepository();
          final vm = ReceiveViewModel(repository: repo)
            ..setInput('7-harbor-rocket');
          await verifyAvoidsHinge(
            'ReceiveStage.declined',
            ReceiveView(viewModel: vm),
            () async {
              unawaited(vm.connect());
              repo.connected.complete(FakeOffer());
              await tester.pump();
              await vm.decline();
              expect(vm.stage, ReceiveStage.declined);
            },
            expectedPanes: 2,
          );
        }

        // ReceiveStage.failed
        {
          final repo = FakeRepository();
          final vm = ReceiveViewModel(repository: repo)
            ..setInput('7-harbor-rocket');
          await verifyAvoidsHinge(
            'ReceiveStage.failed',
            ReceiveView(viewModel: vm),
            () async {
              unawaited(vm.connect());
              repo.connected.complete(FakeOffer());
              await tester.pump();
              await vm.accept();
              repo.events.add(_failedEvent);
              await tester.pump();
              expect(vm.stage, ReceiveStage.failed);
            },
            expectedPanes: 2,
          );
        }
      },
    );
  });

  group('Fold, unfold, and keyboard continuity', () {
    testWidgets(
      'preserves drafted Send text and Receive code across full cover -> book -> tabletop -> cover round-trip',
      (tester) async {
        final sendVm = SendViewModel(
          repository: FakeRepository(),
          fileSource: FakeFileSource(),
        )..setTextMode(true);
        final receiveVm = ReceiveViewModel(repository: FakeRepository());

        const coverSize = Size(344, 882);
        const bookSize = Size(840, 600);
        const tabletopSize = Size(673, 841);
        const bookFeature = DisplayFeature(
          bounds: Rect.fromLTWH(406, 0, 28, 600),
          type: DisplayFeatureType.hinge,
          state: DisplayFeatureState.unknown,
        );
        const tabletopFeature = DisplayFeature(
          bounds: Rect.fromLTWH(0, 420, 673, 0),
          type: DisplayFeatureType.fold,
          state: DisplayFeatureState.postureHalfOpened,
        );

        // Send text draft survives cover -> book -> tabletop -> cover.
        await tester.binding.setSurfaceSize(coverSize);
        addTearDown(() => tester.binding.setSurfaceSize(null));
        await tester.pumpWidget(
          _wrap(
            home: SendView(viewModel: sendVm),
            size: coverSize,
          ),
        );
        await tester.enterText(
          find.byType(TextField),
          'Foldable continuity test',
        );
        await tester.pump();
        expect(sendVm.text, 'Foldable continuity test');

        for (final (size, features) in [
          (bookSize, const [bookFeature]),
          (tabletopSize, const [tabletopFeature]),
          (coverSize, const <DisplayFeature>[]),
        ]) {
          await tester.binding.setSurfaceSize(size);
          await tester.pumpWidget(
            _wrap(
              home: SendView(viewModel: sendVm),
              size: size,
              features: features,
            ),
          );
          await tester.pump();
          expect(
            tester.widget<TextField>(find.byType(TextField)).controller?.text,
            'Foldable continuity test',
          );
          expect(tester.takeException(), isNull);
        }

        // Receive code input survives cover -> book -> tabletop -> cover.
        await tester.binding.setSurfaceSize(coverSize);
        await tester.pumpWidget(
          _wrap(
            home: ReceiveView(viewModel: receiveVm),
            size: coverSize,
          ),
        );
        await tester.enterText(find.byType(TextField), '9-candle-harbor');
        await tester.pump();
        expect(receiveVm.input, '9-candle-harbor');

        for (final (size, features) in [
          (bookSize, const [bookFeature]),
          (tabletopSize, const [tabletopFeature]),
          (coverSize, const <DisplayFeature>[]),
        ]) {
          await tester.binding.setSurfaceSize(size);
          await tester.pumpWidget(
            _wrap(
              home: ReceiveView(viewModel: receiveVm),
              size: size,
              features: features,
            ),
          );
          await tester.pump();
          expect(
            tester.widget<TextField>(find.byType(TextField)).controller?.text,
            '9-candle-harbor',
          );
          expect(tester.takeException(), isNull);
        }
      },
    );

    testWidgets(
      'keeps TextField focus and text across tabletop keyboard collapse and re-expansion',
      (tester) async {
        const size = Size(673, 841);
        const tabletopFeature = DisplayFeature(
          bounds: Rect.fromLTWH(0, 420, 673, 0),
          type: DisplayFeatureType.fold,
          state: DisplayFeatureState.postureHalfOpened,
        );

        await tester.binding.setSurfaceSize(size);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        // 1) SendView text mode
        final sendVm = SendViewModel(
          repository: FakeRepository(),
          fileSource: FakeFileSource(),
        )..setTextMode(true);
        await tester.pumpWidget(
          _wrap(
            home: SendView(viewModel: sendVm),
            size: size,
            features: const [tabletopFeature],
          ),
        );
        await tester.tap(find.byType(TextField));
        await tester.enterText(find.byType(TextField), 'draft across ime');
        await tester.pump();
        final sendNode = tester
            .widget<TextField>(find.byType(TextField))
            .focusNode!;
        expect(sendNode.hasFocus, isTrue);

        // Soft keyboard opens -> tabletop collapses to singlePane.
        await tester.pumpWidget(
          _wrap(
            home: SendView(viewModel: sendVm),
            size: size,
            features: const [tabletopFeature],
            viewInsets: const EdgeInsets.only(bottom: 340),
          ),
        );
        await tester.pump();
        expect(sendNode.hasFocus, isTrue, reason: 'focus survives collapse');
        expect(
          tester.widget<TextField>(find.byType(TextField)).controller?.text,
          'draft across ime',
        );

        // Soft keyboard closes -> tabletop re-expands without unfocusing.
        await tester.pumpWidget(
          _wrap(
            home: SendView(viewModel: sendVm),
            size: size,
            features: const [tabletopFeature],
          ),
        );
        await tester.pump();
        expect(sendNode.hasFocus, isTrue, reason: 'focus survives re-expand');
        expect(tester.takeException(), isNull);

        // 2) ReceiveView code input
        final receiveVm = ReceiveViewModel(repository: FakeRepository());
        await tester.pumpWidget(
          _wrap(
            home: ReceiveView(viewModel: receiveVm),
            size: size,
            features: const [tabletopFeature],
          ),
        );
        await tester.tap(find.byType(TextField));
        await tester.enterText(find.byType(TextField), '9-candle-harbor');
        await tester.pump();
        final receiveNode = tester
            .widget<TextField>(find.byType(TextField))
            .focusNode!;
        expect(receiveNode.hasFocus, isTrue);

        await tester.pumpWidget(
          _wrap(
            home: ReceiveView(viewModel: receiveVm),
            size: size,
            features: const [tabletopFeature],
            viewInsets: const EdgeInsets.only(bottom: 340),
          ),
        );
        await tester.pump();
        expect(receiveNode.hasFocus, isTrue);
        expect(
          tester.widget<TextField>(find.byType(TextField)).controller?.text,
          '9-candle-harbor',
        );

        await tester.pumpWidget(
          _wrap(
            home: ReceiveView(viewModel: receiveVm),
            size: size,
            features: const [tabletopFeature],
          ),
        );
        await tester.pump();
        expect(receiveNode.hasFocus, isTrue);
        expect(tester.takeException(), isNull);

        // Dismissing the keyboard (unfocus) and re-expanding Tabletop must not
        // re-trigger autofocus and bounce the keyboard back open.
        await tester.pumpWidget(
          _wrap(
            home: ReceiveView(viewModel: receiveVm),
            size: size,
            features: const [tabletopFeature],
            viewInsets: const EdgeInsets.only(bottom: 340),
          ),
        );
        await tester.pump();
        receiveNode.unfocus();
        await tester.pump();
        expect(receiveNode.hasFocus, isFalse);

        await tester.pumpWidget(
          _wrap(
            home: ReceiveView(viewModel: receiveVm),
            size: size,
            features: const [tabletopFeature],
          ),
        );
        await tester.pump();
        expect(
          receiveNode.hasFocus,
          isFalse,
          reason:
              're-expanding Tabletop must not trigger autofocus bounce loop',
        );

        // Rapid entering -> connecting -> failed -> entering within the 250 ms
        // AnimatedSwitcher transition must not collide on TextField GlobalKey.
        final rapidRepo = FakeRepository();
        final rapidVm = ReceiveViewModel(repository: rapidRepo)
          ..setInput('9-candle-harbor');
        await tester.pumpWidget(
          _wrap(
            home: ReceiveView(viewModel: rapidVm),
            size: size,
            features: const [tabletopFeature],
          ),
        );
        await tester.pump();
        unawaited(rapidVm.connect());
        await tester.pump(const Duration(milliseconds: 50));
        rapidRepo.connected.completeError(
          const TransferError(
            kind: ErrorKind.connect,
            detail: 'instant offline error',
            cause: 'NoRoute',
          ),
        );
        await tester.pump(const Duration(milliseconds: 50));
        expect(rapidVm.stage, ReceiveStage.failed);
        rapidVm.reset();
        await tester.pump(const Duration(milliseconds: 50));
        expect(rapidVm.stage, ReceiveStage.entering);
        expect(tester.takeException(), isNull);
        await tester.pump(const Duration(milliseconds: 300));
        expect(tester.takeException(), isNull);

        // 3) SettingsView server field across Book <-> single-column resize
        final settings = _Settings();
        const bookSize = Size(840, 600);
        const bookFeature = DisplayFeature(
          bounds: Rect.fromLTWH(406, 0, 28, 600),
          type: DisplayFeatureType.hinge,
          state: DisplayFeatureState.unknown,
        );
        await tester.binding.setSurfaceSize(bookSize);
        await tester.pumpWidget(
          _wrap(
            home: SettingsView(
              settings: settings,
              version: '0.3.0-beta.7',
              background: const _Limits(),
            ),
            size: bookSize,
            features: const [bookFeature],
          ),
        );
        await tester.pump();
        await tester.tap(find.byType(TextField));
        await tester.enterText(
          find.byType(TextField),
          'wss://custom.example.org',
        );
        await tester.pump();
        final settingsNode = tester
            .widget<TextField>(find.byType(TextField))
            .focusNode!;
        expect(settingsNode.hasFocus, isTrue);

        await tester.binding.setSurfaceSize(size);
        await tester.pumpWidget(
          _wrap(
            home: SettingsView(
              settings: settings,
              version: '0.3.0-beta.7',
              background: const _Limits(),
            ),
            size: size,
            features: const [tabletopFeature],
          ),
        );
        await tester.pump();
        expect(settingsNode.hasFocus, isTrue);
        expect(
          tester.widget<TextField>(find.byType(TextField)).controller?.text,
          'wss://custom.example.org',
        );
        expect(tester.takeException(), isNull);
      },
    );
  });

  group('Medium 600x800 two-pane at 1.5x text scale', () {
    const size = Size(600, 800);
    for (final locale in const [
      Locale('en'),
      Locale('ar'),
      Locale('ja'),
      Locale('de'),
    ]) {
      testWidgets(
        'renders two-pane screens cleanly in ${locale.languageCode} at 1.5x',
        (tester) async {
          await tester.binding.setSurfaceSize(size);
          addTearDown(() => tester.binding.setSurfaceSize(null));

          final cases = <String, Widget Function()>{
            'home': () =>
                HomeView(onSend: () {}, onReceive: () {}, onSettings: () {}),
            'send empty': () => SendView(
              viewModel: SendViewModel(
                repository: FakeRepository(),
                fileSource: FakeFileSource(),
              ),
            ),
            'receive entering': () => ReceiveView(
              viewModel: ReceiveViewModel(repository: FakeRepository()),
            ),
            'background setup': () => BackgroundSetupView(
              access: const _Limits(),
              firstRun: true,
              onDone: ({required skipped}) {},
            ),
          };

          for (final MapEntry(key: name, value: build) in cases.entries) {
            await tester.pumpWidget(
              _wrap(home: build(), size: size, locale: locale, textScale: 1.5),
            );
            await tester.pump();
            await tester.pump(const Duration(milliseconds: 300));
            expect(
              tester.takeException(),
              isNull,
              reason: '$name at 600x800 1.5x (${locale.languageCode})',
            );
            expect(
              find.byType(VerticalDivider),
              findsOneWidget,
              reason: '$name activates flat two-pane at 600 dp 1.5x',
            );
          }
        },
      );
    }
  });

  group('All screens & stages across foldable postures, locales, themes, and 200% text', () {
    final settings = _Settings();
    final formFactors = <String, (Size, List<DisplayFeature>)>{
      'cover 320x740': (const Size(320, 740), const []),
      'book 840x600': (
        const Size(840, 600),
        const [
          DisplayFeature(
            bounds: Rect.fromLTWH(406, 0, 28, 600),
            type: DisplayFeatureType.hinge,
            state: DisplayFeatureState.unknown,
          ),
        ],
      ),
      'tabletop 673x841': (
        const Size(673, 841),
        const [
          DisplayFeature(
            bounds: Rect.fromLTWH(0, 420, 673, 0),
            type: DisplayFeatureType.fold,
            state: DisplayFeatureState.postureHalfOpened,
          ),
        ],
      ),
      'unfolded flat 673x841': (const Size(673, 841), const []),
      'tablet 1280x800': (const Size(1280, 800), const []),
    };

    final configs = [
      (const Locale('en'), Brightness.light, 1.0),
      (const Locale('ar'), Brightness.dark, 2.0),
      (const Locale('ja'), Brightness.dark, 2.0),
      (const Locale('de'), Brightness.light, 2.0),
    ];

    final screens =
        <String, (Widget, Future<void> Function(WidgetTester)) Function()>{
          'home': () => (
            HomeView(onSend: () {}, onReceive: () {}, onSettings: () {}),
            (_) async {},
          ),
          'send empty': () {
            final vm = SendViewModel(
              repository: FakeRepository(),
              fileSource: FakeFileSource(),
            );
            return (
              SendView(viewModel: vm),
              (_) async => expect(vm.stage, SendStage.picking),
            );
          },
          'send multi-file list': () {
            final vm = SendViewModel(
              repository: FakeRepository(),
              fileSource: FakeFileSource(),
            );
            for (var i = 1; i <= 10; i++) {
              vm.addFakeFileForTest(
                'Annual-Sustainability-Report-2026-Appendix-$i-Final.pdf',
              );
            }
            return (
              SendView(viewModel: vm),
              (_) async {
                expect(vm.stage, SendStage.picking);
                expect(vm.files, hasLength(10));
              },
            );
          },
          'send text': () {
            final vm =
                SendViewModel(
                    repository: FakeRepository(),
                    fileSource: FakeFileSource(),
                  )
                  ..setTextMode(true)
                  ..setText('Sample message for foldable check');
            return (
              SendView(viewModel: vm),
              (_) async {
                expect(vm.stage, SendStage.picking);
                expect(vm.textMode, isTrue);
              },
            );
          },
          'send preparing': () {
            final repo = FakeRepository();
            final vm = SendViewModel(
              repository: repo,
              fileSource: FakeFileSource(),
            )..addFakeFileForTest('report.pdf');
            return (
              SendView(viewModel: vm),
              (_) async {
                unawaited(vm.start());
                expect(vm.stage, SendStage.preparing);
              },
            );
          },
          'send waiting': () {
            final repo = FakeRepository();
            final vm = SendViewModel(
              repository: repo,
              fileSource: FakeFileSource(),
            )..addFakeFileForTest('report.pdf');
            return (
              SendView(viewModel: vm),
              (tester) async {
                unawaited(vm.start());
                repo.prepared.complete(FakeSession());
                await tester.pump();
                expect(vm.stage, SendStage.waiting);
              },
            );
          },
          'send connected': () {
            final repo = FakeRepository();
            final vm = SendViewModel(
              repository: repo,
              fileSource: FakeFileSource(),
            )..addFakeFileForTest('report.pdf');
            return (
              SendView(viewModel: vm),
              (tester) async {
                unawaited(vm.start());
                repo.prepared.complete(FakeSession());
                await tester.pump();
                repo.events.add(
                  const TransferEvent(
                    kind: EventKind.connected,
                    words: ['abandon', 'ability', 'absorb', 'abstract'],
                    done: 0,
                    total: 0,
                    summary: null,
                    error: null,
                  ),
                );
                await tester.pump();
                expect(vm.stage, SendStage.connected);
              },
            );
          },
          'send transferring': () {
            final repo = FakeRepository();
            final vm = SendViewModel(
              repository: repo,
              fileSource: FakeFileSource(),
            )..addFakeFileForTest('report.pdf');
            return (
              SendView(viewModel: vm),
              (tester) async {
                unawaited(vm.start());
                repo.prepared.complete(FakeSession());
                await tester.pump();
                repo.events.add(
                  const TransferEvent(
                    kind: EventKind.connected,
                    words: ['abandon', 'ability', 'absorb', 'abstract'],
                    done: 0,
                    total: 0,
                    summary: null,
                    error: null,
                  ),
                );
                repo.events.add(
                  const TransferEvent(
                    kind: EventKind.progress,
                    words: [],
                    done: 123456789,
                    total: 345678901,
                    summary: null,
                    error: null,
                  ),
                );
                await tester.pump();
                expect(vm.stage, SendStage.transferring);
              },
            );
          },
          'send done': () {
            final repo = FakeRepository();
            final vm = SendViewModel(
              repository: repo,
              fileSource: FakeFileSource(),
            )..addFakeFileForTest('report.pdf');
            return (
              SendView(viewModel: vm),
              (tester) async {
                unawaited(vm.start());
                repo.prepared.complete(FakeSession());
                await tester.pump();
                repo.events.add(_finishedFilesEvent);
                await tester.pump();
                expect(vm.stage, SendStage.done);
              },
            );
          },
          'send failed': () {
            final repo = FakeRepository();
            final vm = SendViewModel(
              repository: repo,
              fileSource: FakeFileSource(),
            )..addFakeFileForTest('report.pdf');
            return (
              SendView(viewModel: vm),
              (tester) async {
                unawaited(vm.start());
                repo.prepared.complete(FakeSession());
                await tester.pump();
                repo.events.add(_failedEvent);
                await tester.pump();
                expect(vm.stage, SendStage.failed);
              },
            );
          },
          'receive entering': () {
            final vm = ReceiveViewModel(repository: FakeRepository());
            return (
              ReceiveView(viewModel: vm),
              (_) async => expect(vm.stage, ReceiveStage.entering),
            );
          },
          'receive connecting': () {
            final repo = FakeRepository();
            final vm = ReceiveViewModel(repository: repo)
              ..setInput('7-harbor-rocket');
            return (
              ReceiveView(viewModel: vm),
              (_) async {
                unawaited(vm.connect());
                expect(vm.stage, ReceiveStage.connecting);
              },
            );
          },
          'receive offered multi-file': () {
            final repo = FakeRepository();
            final vm = ReceiveViewModel(repository: repo)
              ..setInput('7-harbor-rocket');
            return (
              ReceiveView(viewModel: vm),
              (tester) async {
                unawaited(vm.connect());
                repo.connected.complete(_MultiFileOffer());
                await tester.pump();
                expect(vm.stage, ReceiveStage.offered);
              },
            );
          },
          'receive offered text': () {
            final repo = FakeRepository();
            final vm = ReceiveViewModel(repository: repo)
              ..setInput('7-harbor-rocket');
            return (
              ReceiveView(viewModel: vm),
              (tester) async {
                unawaited(vm.connect());
                repo.connected.complete(FakeOffer(isText: true));
                await tester.pump();
                expect(vm.stage, ReceiveStage.offered);
              },
            );
          },
          'receive transferring': () {
            final repo = FakeRepository();
            final vm = ReceiveViewModel(repository: repo)
              ..setInput('7-harbor-rocket');
            return (
              ReceiveView(viewModel: vm),
              (tester) async {
                unawaited(vm.connect());
                repo.connected.complete(FakeOffer());
                await tester.pump();
                await vm.accept();
                repo.events.add(
                  const TransferEvent(
                    kind: EventKind.progress,
                    words: [],
                    done: 123456789,
                    total: 345678901,
                    summary: null,
                    error: null,
                  ),
                );
                await tester.pump();
                expect(vm.stage, ReceiveStage.transferring);
              },
            );
          },
          'receive done files': () {
            final repo = FakeRepository();
            final vm = ReceiveViewModel(repository: repo)
              ..setInput('7-harbor-rocket');
            return (
              ReceiveView(viewModel: vm),
              (tester) async {
                unawaited(vm.connect());
                repo.connected.complete(FakeOffer());
                await tester.pump();
                await vm.accept();
                repo.events.add(_finishedFilesEvent);
                await tester.pump();
                expect(vm.stage, ReceiveStage.done);
              },
            );
          },
          'receive done text': () {
            final repo = FakeRepository();
            final vm = ReceiveViewModel(repository: repo)
              ..setInput('7-harbor-rocket');
            return (
              ReceiveView(viewModel: vm),
              (tester) async {
                unawaited(vm.connect());
                repo.connected.complete(FakeOffer(isText: true));
                await tester.pump();
                await vm.accept();
                repo.events.add(
                  const TransferEvent(
                    kind: EventKind.finished,
                    words: [],
                    done: 0,
                    total: 0,
                    summary: TransferSummary(
                      filesOk: 1,
                      filesFailed: 0,
                      bytes: 32,
                      saved: [],
                      text: 'Received on foldable device',
                    ),
                    error: null,
                  ),
                );
                await tester.pump();
                expect(vm.stage, ReceiveStage.done);
              },
            );
          },
          'receive declined': () {
            final repo = FakeRepository();
            final vm = ReceiveViewModel(repository: repo)
              ..setInput('7-harbor-rocket');
            return (
              ReceiveView(viewModel: vm),
              (tester) async {
                unawaited(vm.connect());
                repo.connected.complete(FakeOffer());
                await tester.pump();
                await vm.decline();
                await tester.pump();
                expect(vm.stage, ReceiveStage.declined);
              },
            );
          },
          'receive failed': () {
            final repo = FakeRepository();
            final vm = ReceiveViewModel(repository: repo)
              ..setInput('7-harbor-rocket');
            return (
              ReceiveView(viewModel: vm),
              (tester) async {
                unawaited(vm.connect());
                repo.connected.complete(FakeOffer());
                await tester.pump();
                await vm.accept();
                repo.events.add(_failedEvent);
                await tester.pump();
                expect(vm.stage, ReceiveStage.failed);
              },
            );
          },
          'settings': () => (
            SettingsView(
              settings: settings,
              version: '0.3.0-beta.7',
              background: const _Limits(),
            ),
            (_) async {},
          ),
          'background setup firstRun': () => (
            BackgroundSetupView(
              access: const _Limits(),
              firstRun: true,
              onDone: ({required skipped}) {},
            ),
            (_) async {},
          ),
          'background setup from settings': () => (
            BackgroundSetupView(
              access: const _Limits(),
              firstRun: false,
              onDone: ({required skipped}) {},
            ),
            (_) async {},
          ),
        };

    for (final MapEntry(key: ffName, value: (size, features))
        in formFactors.entries) {
      for (final (locale, brightness, scale) in configs) {
        testWidgets(
          '$ffName in ${locale.languageCode} (${brightness.name}, ${scale}x)',
          (tester) async {
            await tester.binding.setSurfaceSize(size);
            addTearDown(() => tester.binding.setSurfaceSize(null));
            for (final MapEntry(key: name, value: factory) in screens.entries) {
              final (widget, reach) = factory();
              await tester.pumpWidget(
                _wrap(
                  home: widget,
                  size: size,
                  locale: locale,
                  brightness: brightness,
                  textScale: scale,
                  features: features,
                ),
              );
              await reach(tester);
              await tester.pump();
              await tester.pump(const Duration(milliseconds: 300));
              expect(
                tester.takeException(),
                isNull,
                reason:
                    'overflow or layout error on $name ($ffName, ${locale.languageCode}, ${scale}x)',
              );
            }
          },
        );
      }
    }
  });

  group('Review hardening: narrow Book fallback, body offsets, and Tabletop controls', () {
    testWidgets(
      'Book posture with landscape SafeArea inset shrinking one side below 280 dp '
      'places AdaptivePaneLayout singlePane on the wider side clear of the hinge',
      (tester) async {
        // Window is 664x500 with a 24 dp hinge at [320, 344]. Both physical
        // sides are 320 dp >= 280 dp, so window posture is Book. A 60 dp left
        // SafeArea leaves 260 dp on the left (< 280 dp) and 320 dp on the right.
        const size = Size(664, 500);
        const hinge = Rect.fromLTWH(320, 0, 24, 500);
        const features = [
          DisplayFeature(
            bounds: hinge,
            type: DisplayFeatureType.hinge,
            state: DisplayFeatureState.postureHalfOpened,
          ),
        ];
        await tester.binding.setSurfaceSize(size);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        await tester.pumpWidget(
          _wrap(
            size: size,
            features: features,
            viewPadding: const EdgeInsets.only(left: 60),
            home: const Scaffold(
              body: SafeArea(
                child: AdaptivePaneLayout(
                  primary: ColoredBox(key: Key('primary'), color: Colors.red),
                  secondary: ColoredBox(
                    key: Key('secondary'),
                    color: Colors.blue,
                  ),
                  singlePane: ColoredBox(
                    key: Key('single'),
                    color: Colors.green,
                  ),
                ),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.byKey(const Key('primary')), findsNothing);
        expect(find.byKey(const Key('single')), findsOneWidget);
        final rect = tester.getRect(find.byKey(const Key('single')));
        expect(
          rect.left,
          greaterThanOrEqualTo(hinge.right),
          reason: 'singlePane must land on the wider right side, never across the hinge',
        );
        expect(rect.width, closeTo(320, 0.5));
      },
    );

    testWidgets(
      'Book posture with landscape SafeArea inset places AdaptiveSinglePane '
      'on the wider side clear of the hinge',
      (tester) async {
        const size = Size(664, 500);
        const hinge = Rect.fromLTWH(320, 0, 24, 500);
        const features = [
          DisplayFeature(
            bounds: hinge,
            type: DisplayFeatureType.hinge,
            state: DisplayFeatureState.postureHalfOpened,
          ),
        ];
        await tester.binding.setSurfaceSize(size);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        await tester.pumpWidget(
          _wrap(
            size: size,
            features: features,
            viewPadding: const EdgeInsets.only(left: 60),
            home: const Scaffold(
              body: SafeArea(
                child: AdaptiveSinglePane(
                  child: ColoredBox(
                    key: Key('single_only'),
                    color: Colors.amber,
                  ),
                ),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();

        final rect = tester.getRect(find.byKey(const Key('single_only')));
        expect(rect.left, greaterThanOrEqualTo(hinge.right));
        expect(rect.width, closeTo(320, 0.5));
      },
    );

    testWidgets(
      'Tabletop posture realigns via RenderBox.localToGlobal when a banner '
      'sits above AdaptivePaneLayout inside Scaffold body',
      (tester) async {
        const size = Size(800, 720);
        const hinge = Rect.fromLTWH(0, 348, 800, 24);
        const features = [
          DisplayFeature(
            bounds: hinge,
            type: DisplayFeatureType.hinge,
            state: DisplayFeatureState.postureHalfOpened,
          ),
        ];
        await tester.binding.setSurfaceSize(size);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        await tester.pumpWidget(
          _wrap(
            size: size,
            features: features,
            home: const Scaffold(
              body: Column(
                children: [
                  SizedBox(height: 60, child: Text('Top banner')),
                  Expanded(
                    child: AdaptivePaneLayout(
                      primary: ColoredBox(
                        key: Key('top_pane'),
                        color: Colors.red,
                      ),
                      secondary: ColoredBox(
                        key: Key('bottom_pane'),
                        color: Colors.blue,
                      ),
                      singlePane: ColoredBox(
                        key: Key('single_pane'),
                        color: Colors.green,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();

        final topRect = tester.getRect(find.byKey(const Key('top_pane')));
        final bottomRect = tester.getRect(find.byKey(const Key('bottom_pane')));
        expect(topRect.top, closeTo(60, 0.5));
        expect(topRect.bottom, closeTo(hinge.top, 0.5));
        expect(bottomRect.top, closeTo(hinge.bottom, 0.5));
        expect(bottomRect.bottom, closeTo(size.height, 0.5));
      },
    );

    testWidgets(
      'Tabletop posture re-measures origin after a pushed route animation '
      'completes (#635 review)',
      (tester) async {
        const size = Size(800, 720);
        const hinge = Rect.fromLTWH(0, 348, 800, 24);
        const features = [
          DisplayFeature(
            bounds: hinge,
            type: DisplayFeatureType.hinge,
            state: DisplayFeatureState.postureHalfOpened,
          ),
        ];
        await tester.binding.setSurfaceSize(size);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        await tester.pumpWidget(
          _wrap(
            size: size,
            features: features,
            home: Builder(
              builder: (context) => Scaffold(
                body: Center(
                  child: FilledButton(
                    onPressed: () {
                      Navigator.of(context).push(
                        MaterialPageRoute<void>(
                          builder: (_) => const Scaffold(
                            body: Column(
                              children: [
                                SizedBox(height: 60, child: Text('Top banner')),
                                Expanded(
                                  child: AdaptivePaneLayout(
                                    primary: ColoredBox(
                                      key: Key('pushed_top_pane'),
                                      color: Colors.red,
                                    ),
                                    secondary: ColoredBox(
                                      key: Key('pushed_bottom_pane'),
                                      color: Colors.blue,
                                    ),
                                    singlePane: ColoredBox(
                                      key: Key('pushed_single_pane'),
                                      color: Colors.green,
                                    ),
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ),
                      );
                    },
                    child: const Text('Push route'),
                  ),
                ),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();

        await tester.tap(find.text('Push route'));
        await tester.pumpAndSettle();

        final topRect = tester.getRect(
          find.byKey(const Key('pushed_top_pane')),
        );
        final bottomRect = tester.getRect(
          find.byKey(const Key('pushed_bottom_pane')),
        );
        expect(topRect.top, closeTo(60, 0.5));
        expect(topRect.bottom, closeTo(hinge.top, 0.5));
        expect(bottomRect.top, closeTo(hinge.bottom, 0.5));
        expect(bottomRect.bottom, closeTo(size.height, 0.5));
      },
    );

    testWidgets(
      'Book posture keeps settled origin after pushing and popping a route '
      '(secondaryAnimation) and after a cancelled predictive back gesture '
      '(#635 review)',
      (tester) async {
        const size = Size(840, 600);
        const hinge = Rect.fromLTWH(408, 0, 24, 600);
        const features = [
          DisplayFeature(
            bounds: hinge,
            type: DisplayFeatureType.hinge,
            state: DisplayFeatureState.postureHalfOpened,
          ),
        ];
        await tester.binding.setSurfaceSize(size);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        late NavigatorState nav;
        final leadingWidth = ValueNotifier<double>(0);
        addTearDown(leadingWidth.dispose);

        await tester.pumpWidget(
          _wrap(
            size: size,
            features: features,
            home: Builder(
              builder: (context) {
                nav = Navigator.of(context);
                return Scaffold(
                  body: Row(
                    children: [
                      const SizedBox(width: 40),
                      Expanded(
                        child: AdaptivePaneLayout(
                          primary: Center(
                            key: const Key('home_left_pane'),
                            child: FilledButton(
                              onPressed: () {
                                nav.push(
                                  MaterialPageRoute<void>(
                                    builder: (_) =>
                                        ValueListenableBuilder<double>(
                                          valueListenable: leadingWidth,
                                          builder: (_, w, _) => Scaffold(
                                            body: Row(
                                              children: [
                                                SizedBox(width: w),
                                                const Expanded(
                                                  child: AdaptivePaneLayout(
                                                    primary: ColoredBox(
                                                      key: Key(
                                                        'detail_left_pane',
                                                      ),
                                                      color: Colors.orange,
                                                    ),
                                                    secondary: ColoredBox(
                                                      key: Key(
                                                        'detail_right_pane',
                                                      ),
                                                      color: Colors.purple,
                                                    ),
                                                    singlePane: ColoredBox(
                                                      key: Key('detail_single'),
                                                      color: Colors.teal,
                                                    ),
                                                  ),
                                                ),
                                              ],
                                            ),
                                          ),
                                        ),
                                  ),
                                );
                              },
                              child: const Text('Open detail'),
                            ),
                          ),
                          secondary: const ColoredBox(
                            key: Key('home_right_pane'),
                            color: Colors.blue,
                          ),
                          singlePane: const ColoredBox(
                            key: Key('home_single_pane'),
                            color: Colors.green,
                          ),
                        ),
                      ),
                    ],
                  ),
                );
              },
            ),
          ),
        );
        await tester.pumpAndSettle();

        // Push a MaterialPageRoute (activating secondaryAnimation on home).
        await tester.tap(find.text('Open detail'));
        await tester.pumpAndSettle();

        // Simulate a cancelled predictive back gesture on the pushed route
        // while its horizontal origin shifts, then confirm it re-measures
        // origin.dx after returning to AnimationStatus.completed.
        final detailRoute = ModalRoute.of(
          tester.element(find.byKey(const Key('detail_left_pane'))),
        )!;
        detailRoute.handleStartBackGesture(progress: 0.25);
        leadingWidth.value = 48;
        await tester.pump(const Duration(milliseconds: 50));
        detailRoute.handleCancelBackGesture();
        await tester.pumpAndSettle();

        final detailLeft = tester.getRect(
          find.byKey(const Key('detail_left_pane')),
        );
        final detailRight = tester.getRect(
          find.byKey(const Key('detail_right_pane')),
        );
        expect(detailLeft.left, closeTo(48, 0.5));
        expect(detailLeft.right, closeTo(hinge.left, 0.5));
        expect(detailRight.left, closeTo(hinge.right, 0.5));
        expect(detailRight.right, closeTo(size.width, 0.5));

        // Pop back to home (exercising secondaryAnimation reverse -> dismissed)
        // and verify home's origin settles cleanly at dx = 40 without locking a
        // mid-transition slide offset.
        nav.pop();
        await tester.pumpAndSettle();

        final leftRect = tester.getRect(
          find.byKey(const Key('home_left_pane')),
        );
        final rightRect = tester.getRect(
          find.byKey(const Key('home_right_pane')),
        );
        expect(leftRect.left, closeTo(40, 0.5));
        expect(leftRect.right, closeTo(hinge.left, 0.5));
        expect(rightRect.left, closeTo(hinge.right, 0.5));
        expect(rightRect.right, closeTo(size.width, 0.5));
      },
    );

    testWidgets(
      'Tabletop posture puts Done and Try again controls below the horizontal '
      'fold on Send Done, Receive Declined, and Failed stages',
      (tester) async {
        const size = Size(800, 720);
        const hinge = Rect.fromLTWH(0, 348, 800, 24);
        const features = [
          DisplayFeature(
            bounds: hinge,
            type: DisplayFeatureType.hinge,
            state: DisplayFeatureState.postureHalfOpened,
          ),
        ];
        await tester.binding.setSurfaceSize(size);
        addTearDown(() => tester.binding.setSurfaceSize(null));
        final l = lookupAppLocalizations(const Locale('en'));

        // Send Done stage
        final sendRepo = FakeRepository();
        final sendVm = SendViewModel(
          repository: sendRepo,
          fileSource: FakeFileSource(),
        )..addFakeFileForTest('a.txt');
        await tester.pumpWidget(
          _wrap(
            size: size,
            features: features,
            home: SendView(viewModel: sendVm),
          ),
        );
        unawaited(sendVm.start());
        sendRepo.prepared.complete(FakeSession());
        await tester.pump();
        sendRepo.events.add(_finishedFilesEvent);
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));

        expect(
          tester.getRect(find.text(l.sendDone)).bottom,
          lessThanOrEqualTo(hinge.top),
        );
        expect(
          tester.getRect(find.widgetWithText(FilledButton, l.done)).top,
          greaterThanOrEqualTo(hinge.bottom),
        );

        // Send Failed stage
        final failRepo = FakeRepository();
        final failVm = SendViewModel(
          repository: failRepo,
          fileSource: FakeFileSource(),
        )..addFakeFileForTest('a.txt');
        await tester.pumpWidget(
          _wrap(
            size: size,
            features: features,
            home: SendView(viewModel: failVm),
          ),
        );
        unawaited(failVm.start());
        failRepo.prepared.complete(FakeSession());
        await tester.pump();
        failRepo.events.add(_failedEvent);
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));

        expect(
          tester.getRect(find.text(l.errorConnect)).bottom,
          lessThanOrEqualTo(hinge.top),
        );
        expect(
          tester.getRect(find.widgetWithText(FilledButton, l.tryAgain)).top,
          greaterThanOrEqualTo(hinge.bottom),
        );

        // Receive Declined stage
        final recRepo = FakeRepository();
        final recVm = ReceiveViewModel(repository: recRepo)
          ..setInput('7-harbor-rocket');
        await tester.pumpWidget(
          _wrap(
            size: size,
            features: features,
            home: ReceiveView(viewModel: recVm),
          ),
        );
        unawaited(recVm.connect());
        recRepo.connected.complete(FakeOffer());
        await tester.pump();
        await recVm.decline();
        await tester.pump();
        await tester.pump(const Duration(milliseconds: 300));

        expect(
          tester.getRect(find.text(l.receiveDeclined)).bottom,
          lessThanOrEqualTo(hinge.top),
        );
        expect(
          tester.getRect(find.widgetWithText(FilledButton, l.done)).top,
          greaterThanOrEqualTo(hinge.bottom),
        );
      },
    );

    testWidgets(
      'Tabletop falls back to the taller half instead of crossing the fold in '
      'AdaptiveSinglePane and AdaptivePaneLayout',
      (tester) async {
        const size = Size(673, 841);
        const hinge = Rect.fromLTWH(0, 420, 673, 12);
        const feature = DisplayFeature(
          bounds: hinge,
          type: DisplayFeatureType.hinge,
          state: DisplayFeatureState.postureHalfOpened,
        );
        const contentKey = ValueKey('single-content');

        await tester.binding.setSurfaceSize(size);
        addTearDown(() => tester.binding.setSurfaceSize(null));

        // 1. AdaptiveSinglePane: soft keyboard shrinks lower half below
        // minVerticalPaneHeight -> content stays strictly in the taller top
        // half and never overlaps the horizontal hinge.
        bool? singleTabletop;
        await tester.pumpWidget(
          _wrap(
            size: size,
            features: const [feature],
            viewPadding: const EdgeInsets.only(top: 32),
            viewInsets: const EdgeInsets.only(bottom: 340),
            home: Scaffold(
              appBar: AppBar(title: const Text('Single')),
              body: SafeArea(
                child: AdaptiveSinglePane(
                  child: Builder(
                    builder: (context) {
                      singleTabletop = AdaptivePaneLayout.isTabletopOf(context);
                      return const SizedBox.expand(key: contentKey);
                    },
                  ),
                ),
              ),
            ),
          ),
        );
        await tester.pump();
        final topRect = tester.getRect(find.byKey(contentKey));
        expect(singleTabletop, isFalse);
        expect(topRect.overlaps(hinge), isFalse);
        expect(topRect.bottom, lessThanOrEqualTo(hinge.top + 0.1));

        // 2. AdaptiveSinglePane: top half shorter than minVerticalPaneHeight
        // (e.g. asymmetric high fold) -> content sits strictly in the taller
        // bottom half and never overlaps the hinge.
        const highHinge = Rect.fromLTWH(0, 180, 673, 12);
        const highFeature = DisplayFeature(
          bounds: highHinge,
          type: DisplayFeatureType.hinge,
          state: DisplayFeatureState.postureHalfOpened,
        );
        await tester.pumpWidget(
          _wrap(
            size: size,
            features: const [highFeature],
            viewPadding: const EdgeInsets.only(top: 32),
            home: Scaffold(
              appBar: AppBar(title: const Text('HighFold')),
              body: SafeArea(
                child: AdaptiveSinglePane(
                  child: const SizedBox.expand(key: contentKey),
                ),
              ),
            ),
          ),
        );
        await tester.pump();
        final bottomRect = tester.getRect(find.byKey(contentKey));
        expect(bottomRect.overlaps(highHinge), isFalse);
        expect(bottomRect.top, greaterThanOrEqualTo(highHinge.bottom - 0.1));

        // 3. AdaptivePaneLayout: soft keyboard collapses tabletop -> singlePane
        // stays strictly in the taller top half and never overlaps the hinge.
        const paneSingleKey = ValueKey('pane-single');
        await tester.pumpWidget(
          _wrap(
            size: size,
            features: const [feature],
            viewPadding: const EdgeInsets.only(top: 32),
            viewInsets: const EdgeInsets.only(bottom: 340),
            home: Scaffold(
              appBar: AppBar(title: const Text('Pane')),
              body: SafeArea(
                child: AdaptivePaneLayout(
                  primary: const SizedBox.expand(key: ValueKey('p')),
                  secondary: const SizedBox.expand(key: ValueKey('s')),
                  singlePane: const SizedBox.expand(key: paneSingleKey),
                ),
              ),
            ),
          ),
        );
        await tester.pump();
        final paneRect = tester.getRect(find.byKey(paneSingleKey));
        expect(paneRect.overlaps(hinge), isFalse);
        expect(paneRect.bottom, lessThanOrEqualTo(hinge.top + 0.1));
      },
    );

    testWidgets(
      'SendView keeps the Files / Text mode switcher in Book, Tabletop, and '
      'flat tablet layouts while preserving selection',
      (tester) async {
        final l = lookupAppLocalizations(const Locale('en'));
        final vm = SendViewModel(
          repository: FakeRepository(),
          fileSource: FakeFileSource(),
        );
        addTearDown(() => tester.binding.setSurfaceSize(null));

        for (final scenario in [
          (
            name: 'flat tablet',
            size: const Size(840, 600),
            features: const <DisplayFeature>[],
          ),
          (
            name: 'book hinge',
            size: const Size(840, 600),
            features: const [
              DisplayFeature(
                bounds: Rect.fromLTWH(406, 0, 28, 600),
                type: DisplayFeatureType.hinge,
                state: DisplayFeatureState.unknown,
              ),
            ],
          ),
          (
            name: 'tabletop fold',
            size: const Size(673, 841),
            features: const [
              DisplayFeature(
                bounds: Rect.fromLTWH(0, 420, 673, 12),
                type: DisplayFeatureType.fold,
                state: DisplayFeatureState.postureHalfOpened,
              ),
            ],
          ),
        ]) {
          await tester.binding.setSurfaceSize(scenario.size);
          vm.setTextMode(false);
          await tester.pumpWidget(
            _wrap(
              size: scenario.size,
              features: scenario.features,
              home: SendView(viewModel: vm),
            ),
          );
          await tester.pump();
          expect(
            find.byType(SegmentedButton<bool>),
            findsOneWidget,
            reason: '${scenario.name}: empty files mode shows switcher',
          );

          await tester.tap(
            find.descendant(
              of: find.byType(SegmentedButton<bool>),
              matching: find.text(l.sendText),
            ),
          );
          await tester.pump();
          await tester.pump(const Duration(milliseconds: 300));
          expect(vm.textMode, isTrue, reason: scenario.name);
          expect(
            find.byType(SegmentedButton<bool>),
            findsOneWidget,
            reason: '${scenario.name}: text mode shows switcher',
          );
        }
      },
    );
  });
}
