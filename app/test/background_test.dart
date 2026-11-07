// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The page that asks to lift the battery restrictions (#338): shown once
// at first start when something restricts TotalDrop, once more after a
// transfer went to the background while they were still on, and never
// again on its own.
import 'package:material_ui/material_ui.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/data/background_access.dart';
import 'package:totaldrop/data/settings_repository.dart';
import 'package:totaldrop/data/transfer_keep_alive.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/ui/setup/background_reminder.dart';
import 'package:totaldrop/ui/setup/background_setup_view.dart';

class _Settings extends ChangeNotifier implements SettingsRepository {
  @override
  bool backgroundSetupSeen = false;
  @override
  bool backgroundSkipped = false;
  @override
  bool backgroundReminded = false;

  @override
  Future<void> setBackgroundSetupSeen({required bool skipped}) async {
    backgroundSetupSeen = true;
    backgroundSkipped = skipped;
  }

  @override
  Future<void> setBackgroundReminded() async => backgroundReminded = true;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Access implements BackgroundAccess {
  _Access(this.current);

  BackgroundState current;
  final opened = <BackgroundPage>[];

  @override
  Future<BackgroundState> state() async => current;

  @override
  Future<bool> open(BackgroundPage page) async {
    opened.add(page);
    return true;
  }
}

class _KeepAlive implements TransferKeepAlive {
  void Function()? stop;
  var ended = 0;

  @override
  Future<void> begin(
    TransferDirection direction,
    void Function() onStop,
  ) async {
    stop = onStop;
  }

  @override
  Future<void> end() async => ended++;
}

const _android = BackgroundState(exempt: false);
const _exempt = BackgroundState(exempt: true);
const _xiaomi = BackgroundState(
  exempt: false,
  maker: 'Xiaomi',
  autostart: true,
);
const _xiaomiExempt = BackgroundState(
  exempt: true,
  maker: 'Xiaomi',
  autostart: true,
);

Widget _app(Widget home, {Locale locale = const Locale('en')}) => MaterialApp(
  locale: locale,
  localizationsDelegates: const [
    AppLocalizations.delegate,
    ...GlobalMaterialLocalizations.delegates,
  ],
  supportedLocales: AppLocalizations.supportedLocales,
  home: home,
);

/// Tall enough that the whole page is built without scrolling.
void _tall(WidgetTester tester) {
  tester.view.physicalSize = const Size(800, 3000);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
}

void main() {
  group('BackgroundState', () {
    test('is settled only without Android or maker restrictions', () {
      expect(_exempt.settled, isTrue);
      expect(_android.settled, isFalse);
      expect(_xiaomiExempt.settled, isFalse);
      expect(_xiaomi.settled, isFalse);
    });
  });

  group('at first start', () {
    test('shows the page when Android restricts TotalDrop', () async {
      final settings = _Settings();
      final prompts = BackgroundPrompts(
        settings: settings,
        access: _Access(_android),
      );
      expect(await prompts.showAtStart(), isTrue);
      expect(settings.backgroundSetupSeen, isFalse);
    });

    test('shows the page on a maker that may restrict on its own', () async {
      final prompts = BackgroundPrompts(
        settings: _Settings(),
        access: _Access(_xiaomiExempt),
      );
      expect(await prompts.showAtStart(), isTrue);
    });

    test('skips it and marks it seen when nothing restricts', () async {
      final settings = _Settings();
      final prompts = BackgroundPrompts(
        settings: settings,
        access: _Access(_exempt),
      );
      expect(await prompts.showAtStart(), isFalse);
      expect(settings.backgroundSetupSeen, isTrue);
      expect(settings.backgroundSkipped, isFalse);
    });

    test('never shows it twice', () async {
      final settings = _Settings()..backgroundSetupSeen = true;
      final prompts = BackgroundPrompts(
        settings: settings,
        access: _Access(_xiaomi),
      );
      expect(await prompts.showAtStart(), isFalse);
    });
  });

  group('after a transfer went to the background', () {
    test('reminds while the restrictions are on, until marked', () async {
      final settings = _Settings()..backgroundSetupSeen = true;
      final prompts = BackgroundPrompts(
        settings: settings,
        access: _Access(_android),
      );
      expect(await prompts.shouldRemind(), isTrue);
      // Asking does not use the reminder up; showing it does.
      expect(settings.backgroundReminded, isFalse);
      await settings.setBackgroundReminded();
      expect(await prompts.shouldRemind(), isFalse);
    });

    test('not before the first-run page was answered', () async {
      // A share can open the app past the first-run page.
      final prompts = BackgroundPrompts(
        settings: _Settings(),
        access: _Access(_android),
      );
      expect(await prompts.shouldRemind(), isFalse);
    });

    test('does not remind when nothing restricts', () async {
      final settings = _Settings()..backgroundSetupSeen = true;
      final prompts = BackgroundPrompts(
        settings: settings,
        access: _Access(_exempt),
      );
      expect(await prompts.shouldRemind(), isFalse);
    });

    test('does not remind on an exempt Xiaomi phone after Done', () async {
      // The maker's saver cannot be read: after Done, trust the person.
      final settings = _Settings()..backgroundSetupSeen = true;
      final prompts = BackgroundPrompts(
        settings: settings,
        access: _Access(_xiaomiExempt),
      );
      expect(await prompts.shouldRemind(), isFalse);
    });

    test('reminds on an exempt Xiaomi phone after Not now', () async {
      final settings = _Settings()
        ..backgroundSetupSeen = true
        ..backgroundSkipped = true;
      final prompts = BackgroundPrompts(
        settings: settings,
        access: _Access(_xiaomiExempt),
      );
      expect(await prompts.shouldRemind(), isTrue);
    });
  });

  group('WatchedKeepAlive', () {
    test('is active from begin until end', () async {
      final inner = _KeepAlive();
      final watched = WatchedKeepAlive(inner);
      expect(watched.active, isFalse);
      await watched.begin(TransferDirection.sending, () {});
      expect(watched.active, isTrue);
      await watched.end();
      expect(watched.active, isFalse);
      expect(inner.ended, 1);
    });

    test('is inactive once the notification stops the transfer', () async {
      final inner = _KeepAlive();
      final watched = WatchedKeepAlive(inner);
      var stopped = false;
      await watched.begin(TransferDirection.receiving, () => stopped = true);
      inner.stop!();
      expect(stopped, isTrue);
      expect(watched.active, isFalse);
    });
  });

  group('the page', () {
    testWidgets('on a Xiaomi phone offers all three settings', (tester) async {
      _tall(tester);
      final access = _Access(_xiaomi);
      await tester.pumpWidget(
        _app(
          BackgroundSetupView(
            access: access,
            firstRun: true,
            onDone: ({required skipped}) {},
          ),
        ),
      );
      await tester.pumpAndSettle();
      final l = lookupAppLocalizations(const Locale('en'));
      expect(find.text(l.setupBackgroundRestricted), findsOneWidget);
      expect(find.text(l.setupBackgroundMaker('Xiaomi')), findsOneWidget);

      await tester.tap(find.text(l.setupBackgroundBattery));
      await tester.ensureVisible(find.text(l.setupBackgroundMakerOpen));
      await tester.tap(find.text(l.setupBackgroundMakerOpen));
      await tester.ensureVisible(find.text(l.setupBackgroundAutostart));
      await tester.tap(find.text(l.setupBackgroundAutostart));
      expect(access.opened, [
        BackgroundPage.battery,
        BackgroundPage.maker,
        BackgroundPage.autostart,
      ]);
    });

    testWidgets('shows the new state when the person comes back', (
      tester,
    ) async {
      _tall(tester);
      final access = _Access(_android);
      await tester.pumpWidget(
        _app(
          BackgroundSetupView(access: access, onDone: ({required skipped}) {}),
        ),
      );
      await tester.pumpAndSettle();
      final l = lookupAppLocalizations(const Locale('en'));
      expect(find.text(l.setupBackgroundRestricted), findsOneWidget);
      expect(find.text(l.setupBackgroundMakerOpen), findsNothing);

      access.current = _exempt;
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpAndSettle();
      expect(find.text(l.setupBackgroundAllowed), findsOneWidget);
    });

    testWidgets('at first start: Not now and Done, no way back', (
      tester,
    ) async {
      _tall(tester);
      bool? chose;
      await tester.pumpWidget(
        _app(
          BackgroundSetupView(
            access: _Access(_android),
            firstRun: true,
            onDone: ({required skipped}) => chose = skipped,
          ),
        ),
      );
      await tester.pumpAndSettle();
      final l = lookupAppLocalizations(const Locale('en'));
      expect(find.byType(BackButton), findsNothing);
      await tester.ensureVisible(find.text(l.setupBackgroundSkip));
      await tester.tap(find.text(l.setupBackgroundSkip));
      expect(chose, isTrue);
      await tester.tap(find.text(l.done));
      expect(chose, isFalse);
    });

    testWidgets('from Settings: no Not now', (tester) async {
      _tall(tester);
      await tester.pumpWidget(
        _app(
          BackgroundSetupView(
            access: _Access(_android),
            onDone: ({required skipped}) {},
          ),
        ),
      );
      await tester.pumpAndSettle();
      final l = lookupAppLocalizations(const Locale('en'));
      expect(find.text(l.setupBackgroundSkip), findsNothing);
    });

    testWidgets('fits every language at twice the text size', (tester) async {
      tester.view.physicalSize = const Size(360, 740);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      for (final locale in AppLocalizations.supportedLocales) {
        await tester.pumpWidget(
          MediaQuery(
            data: const MediaQueryData(
              size: Size(360, 740),
              textScaler: TextScaler.linear(2),
            ),
            child: _app(
              BackgroundSetupView(
                access: _Access(_xiaomi),
                firstRun: true,
                onDone: ({required skipped}) {},
              ),
              locale: locale,
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull, reason: '$locale');
      }
    });
  });

  group('FirstRunGate', () {
    testWidgets('shows the page, then the app after Not now', (tester) async {
      _tall(tester);
      final settings = _Settings();
      final prompts = BackgroundPrompts(
        settings: settings,
        access: _Access(_android),
      );
      await tester.pumpWidget(
        _app(FirstRunGate(prompts: prompts, child: const Text('home'))),
      );
      await tester.pumpAndSettle();
      final l = lookupAppLocalizations(const Locale('en'));
      expect(find.text('home'), findsNothing);
      await tester.ensureVisible(find.text(l.setupBackgroundSkip));
      await tester.tap(find.text(l.setupBackgroundSkip));
      await tester.pumpAndSettle();
      expect(find.text('home'), findsOneWidget);
      expect(settings.backgroundSetupSeen, isTrue);
      expect(settings.backgroundSkipped, isTrue);
    });

    testWidgets('goes straight to the app when nothing restricts', (
      tester,
    ) async {
      _tall(tester);
      final prompts = BackgroundPrompts(
        settings: _Settings(),
        access: _Access(_exempt),
      );
      await tester.pumpWidget(
        _app(FirstRunGate(prompts: prompts, child: const Text('home'))),
      );
      await tester.pumpAndSettle();
      expect(find.text('home'), findsOneWidget);
    });
  });

  group('BackgroundReminder', () {
    testWidgets('shows the page once after a transfer left the screen', (
      tester,
    ) async {
      _tall(tester);
      final settings = _Settings()..backgroundSetupSeen = true;
      final prompts = BackgroundPrompts(
        settings: settings,
        access: _Access(_android),
      );
      final keepAlive = WatchedKeepAlive(_KeepAlive());
      final navigator = GlobalKey<NavigatorState>();
      await tester.pumpWidget(
        MaterialApp(
          navigatorKey: navigator,
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          builder: (context, child) => BackgroundReminder(
            prompts: prompts,
            keepAlive: keepAlive,
            navigator: navigator,
            child: child!,
          ),
          home: const Text('home'),
        ),
      );
      final l = lookupAppLocalizations(const Locale('en'));

      // Leaving the screen without a transfer: nothing.
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpAndSettle();
      expect(find.text(l.setupBackgroundTitle), findsNothing);

      await keepAlive.begin(TransferDirection.sending, () {});
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpAndSettle();
      // The transfer is still running: its screen stays in view.
      expect(find.text(l.setupBackgroundTitle), findsNothing);
      expect(settings.backgroundReminded, isFalse);

      await keepAlive.end();
      await tester.pumpAndSettle();
      expect(find.text(l.setupBackgroundTitle), findsOneWidget);
      expect(settings.backgroundReminded, isTrue);

      await tester.ensureVisible(find.text(l.done));
      await tester.tap(find.text(l.done));
      await tester.pumpAndSettle();
      expect(find.text('home'), findsOneWidget);

      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpAndSettle();
      expect(find.text(l.setupBackgroundTitle), findsNothing);
    });
    testWidgets('does not open a second page over one already open', (
      tester,
    ) async {
      _tall(tester);
      final settings = _Settings()..backgroundSetupSeen = true;
      final prompts = BackgroundPrompts(
        settings: settings,
        access: _Access(_android),
      );
      final keepAlive = WatchedKeepAlive(_KeepAlive());
      final navigator = GlobalKey<NavigatorState>();
      await tester.pumpWidget(
        MaterialApp(
          navigatorKey: navigator,
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          builder: (context, child) => BackgroundReminder(
            prompts: prompts,
            keepAlive: keepAlive,
            navigator: navigator,
            child: child!,
          ),
          // As when opened from Settings.
          home: BackgroundSetupView(
            access: _Access(_android),
            onDone: ({required skipped}) {},
          ),
        ),
      );
      await tester.pumpAndSettle();
      await keepAlive.begin(TransferDirection.receiving, () {});
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      await keepAlive.end();
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpAndSettle();
      expect(find.byType(BackgroundSetupView), findsOneWidget);
      expect(settings.backgroundReminded, isFalse);
    });
  });
}
