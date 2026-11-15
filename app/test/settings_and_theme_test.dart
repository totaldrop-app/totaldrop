// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Android 13+ Predictive Back, edge-to-edge system bar styles, and the
// Settings screen's SafeArea, TalkBack section headings, and Save button
// state (#590).
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/data/background_access.dart';
import 'package:totaldrop/data/settings_repository.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/frb_generated.dart';
import 'package:totaldrop/ui/core/common.dart';
import 'package:totaldrop/ui/core/theme.dart';
import 'package:totaldrop/ui/settings/settings_view.dart';

class _Rust implements RustLibApi {
  @override
  bool crateApiL10NHasDraftTranslation({required String tag}) => false;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Settings extends ChangeNotifier implements SettingsRepository {
  @override
  String get defaultServer => 'wss://rendezvous.totaldrop.app';

  @override
  String server = 'wss://rendezvous.totaldrop.app';

  @override
  bool privacyMode = false;

  @override
  Future<void> setServer(String url) async {
    server = url.trim();
    notifyListeners();
  }

  @override
  Future<void> resetServer() async {
    server = defaultServer;
    notifyListeners();
  }

  @override
  Future<void> setPrivacyMode(bool on) async {
    privacyMode = on;
    notifyListeners();
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Background implements BackgroundAccess {
  const _Background();

  @override
  Future<BackgroundState> state() async =>
      const BackgroundState(exempt: false, maker: 'Samsung', autostart: false);

  @override
  Future<bool> open(BackgroundPage page) async => true;
}

Widget _app({
  required Widget child,
  Brightness brightness = Brightness.light,
  EdgeInsets padding = EdgeInsets.zero,
}) {
  final scheme = fallbackScheme(brightness);
  return MaterialApp(
    theme: buildTheme(scheme),
    localizationsDelegates: const [
      AppLocalizations.delegate,
      ...GlobalMaterialLocalizations.delegates,
    ],
    supportedLocales: AppLocalizations.supportedLocales,
    builder: (context, inner) => MediaQuery(
      data: MediaQuery.of(context)
          .copyWith(padding: padding, viewPadding: padding),
      child: SystemMotion(child: inner!),
    ),
    home: child,
  );
}

void main() {
  setUpAll(() => RustLib.initMock(api: _Rust()));

  test(
    'Android uses PredictiveBackPageTransitionsBuilder and matching bar icons',
    () {
      final light = buildTheme(fallbackScheme(Brightness.light));
      final dark = buildTheme(fallbackScheme(Brightness.dark));

      expect(
        light.pageTransitionsTheme.builders[TargetPlatform.android],
        isA<PredictiveBackPageTransitionsBuilder>(),
      );
      expect(
        dark.pageTransitionsTheme.builders[TargetPlatform.android],
        isA<PredictiveBackPageTransitionsBuilder>(),
      );

      final lightBars = light.appBarTheme.systemOverlayStyle!;
      expect(lightBars.statusBarColor, Colors.transparent);
      expect(lightBars.systemNavigationBarColor, Colors.transparent);
      expect(lightBars.statusBarIconBrightness, Brightness.dark);
      expect(lightBars.systemNavigationBarIconBrightness, Brightness.dark);
      expect(lightBars.systemNavigationBarContrastEnforced, isNull);

      final darkBars = dark.appBarTheme.systemOverlayStyle!;
      expect(darkBars.statusBarColor, Colors.transparent);
      expect(darkBars.systemNavigationBarColor, Colors.transparent);
      expect(darkBars.statusBarIconBrightness, Brightness.light);
      expect(darkBars.systemNavigationBarIconBrightness, Brightness.light);
      expect(darkBars.systemNavigationBarContrastEnforced, isNull);
    },
  );

  testWidgets(
    'SystemMotion applies edge-to-edge overlay style even without an AppBar',
    (tester) async {
      await tester.pumpWidget(
        _app(brightness: Brightness.dark, child: const Scaffold()),
      );
      final dark = tester.widget<AnnotatedRegion<SystemUiOverlayStyle>>(
        find.byType(AnnotatedRegion<SystemUiOverlayStyle>).first,
      );
      expect(dark.value.statusBarIconBrightness, Brightness.light);
      expect(dark.value.systemNavigationBarIconBrightness, Brightness.light);
      expect(dark.value.systemNavigationBarColor, Colors.transparent);
      expect(dark.value.systemNavigationBarContrastEnforced, isNull);

      await tester.pumpWidget(
        _app(brightness: Brightness.light, child: const Scaffold()),
      );
      await tester.pumpAndSettle();
      final light = tester.widget<AnnotatedRegion<SystemUiOverlayStyle>>(
        find.byType(AnnotatedRegion<SystemUiOverlayStyle>).first,
      );
      expect(light.value.statusBarIconBrightness, Brightness.dark);
      expect(light.value.systemNavigationBarIconBrightness, Brightness.dark);
      expect(light.value.systemNavigationBarColor, Colors.transparent);
      expect(light.value.systemNavigationBarContrastEnforced, isNull);
    },
  );

  testWidgets(
    'SettingsView pads for display cutouts and bottom system navigation bar',
    (tester) async {
      final settings = _Settings();
      await tester.pumpWidget(
        _app(
          padding: const EdgeInsets.fromLTRB(18, 28, 22, 48),
          child: SettingsView(
            settings: settings,
            version: '0.3.0-beta.7',
            background: const _Background(),
          ),
        ),
      );
      await tester.pumpAndSettle();

      final safeArea = tester.widget<SafeArea>(
        find.ancestor(
          of: find.byType(ListView),
          matching: find.byType(SafeArea),
        ),
      );
      expect(safeArea.bottom, isFalse);
      expect(tester.getTopLeft(find.byType(ListView)).dx, 18);
      expect(tester.getTopRight(find.byType(ListView)).dx, 800 - 22);
      final list = tester.widget<ListView>(find.byType(ListView));
      expect(list.padding, columnPadding(800 - 18 - 22, bottom: 24 + 48));
    },
  );

  testWidgets('SettingsView marks every section heading for TalkBack', (
    tester,
  ) async {
    final handle = tester.ensureSemantics();
    try {
      final settings = _Settings();
      await tester.pumpWidget(
        _app(
          child: SettingsView(
            settings: settings,
            version: '0.3.0-beta.7',
            background: const _Background(),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final l = await AppLocalizations.delegate.load(const Locale('en'));

      for (final heading in [
        l.settingsServer,
        l.settingsPrivacy,
        l.settingsBackground,
        l.settingsAbout,
      ]) {
        // Scroll far enough to bring lower sections into the viewport.
        await tester.scrollUntilVisible(
          find.text(heading),
          120,
          scrollable: find.byType(Scrollable).first,
        );
        final node = tester.getSemantics(find.text(heading));
        expect(
          node.flagsCollection.isHeader,
          isTrue,
          reason: '"$heading" should be marked as a header',
        );
      }
    } finally {
      handle.dispose();
    }
  });

  testWidgets(
    'Save is disabled until the server address changes, and disables once saved',
    (tester) async {
      final settings = _Settings();
      await tester.pumpWidget(
        _app(
          child: SettingsView(settings: settings, version: '0.3.0-beta.7'),
        ),
      );
      await tester.pumpAndSettle();
      final l = await AppLocalizations.delegate.load(const Locale('en'));

      FilledButton saveButton() => tester.widget<FilledButton>(
        find.widgetWithText(FilledButton, l.settingsSave),
      );

      expect(saveButton().onPressed, isNull);

      await tester.enterText(
        find.byType(TextField),
        'http://not-secure.example',
      );
      await tester.pump();
      expect(saveButton().onPressed, isNotNull);

      await tester.tap(find.text(l.settingsSave));
      await tester.pump();
      expect(find.text(l.settingsServerInvalid), findsOneWidget);
      expect(settings.server, settings.defaultServer);

      await tester.enterText(
        find.byType(TextField),
        '  wss://relay.example.org  ',
      );
      await tester.pump();
      expect(saveButton().onPressed, isNotNull);

      await tester.tap(find.text(l.settingsSave));
      await tester.pumpAndSettle();
      expect(settings.server, 'wss://relay.example.org');
      expect(saveButton().onPressed, isNull);

      await tester.tap(find.text(l.settingsServerReset));
      await tester.pumpAndSettle();
      expect(settings.server, settings.defaultServer);
      expect(saveButton().onPressed, isNull);

      await settings.setServer('wss://other.example.org');
      await tester.pump();
      expect(saveButton().onPressed, isNotNull);
    },
  );
}
