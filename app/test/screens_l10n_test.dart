// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Every screen in every language (#315, #52, ADR 0008): nothing
// overflows on a small phone, also with larger text, and right-to-left
// languages lay out right to left. The test font draws each letter as a
// full square: as wide as Japanese and wider than Latin or Arabic, so a
// line that fits here fits on a phone. How the text looks (shaping,
// fonts) is checked by hand on a phone in Arabic and Japanese.
import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/data/settings_repository.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/src/rust/frb_generated.dart';
import 'package:totaldrop/ui/core/theme.dart';
import 'package:totaldrop/ui/home/home_view.dart';
import 'package:totaldrop/ui/receive/receive_view.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';
import 'package:totaldrop/ui/settings/settings_view.dart';

import 'fakes.dart';

/// Stands in for the Rust library on the screens tested here.
class _Rust implements RustLibApi {
  @override
  String crateApiL10NRevealControls({required String name}) => name;

  // Every text in its own language: the English fallback for unreviewed
  // critical keys is shorter than most translations.
  @override
  bool crateApiL10NNeedsEnglish({required String tag, required String key}) =>
      false;

  // Shows the draft note too, the longest version of Settings.
  @override
  bool crateApiL10NHasDraftTranslation({required String tag}) => true;

  @override
  String crateApiTransferDefaultServer() => 'wss://rendezvous.totaldrop.app';

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

/// Settings as on a new install, without the device's storage.
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
}

/// Text each screen shows once it reaches its state.
const _shows = {
  'send with the code': '7-harbor-rocket',
  'sending with the words': 'abstract',
  'receive an offer': 'a.txt',
};

const _rtl = {'ar', 'fa', 'he', 'ur'};

/// A small phone: 360 × 740 dp.
const _phone = Size(360, 740);

Widget _app(Locale locale, double textScale, Widget home) => MaterialApp(
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

void main() {
  final settings = _Settings();

  setUpAll(() => RustLib.initMock(api: _Rust()));

  // Each screen, and what brings it to the state shown.
  final screens =
      <String, (Widget, Future<void> Function(WidgetTester)) Function()>{
        'home': () => (
          HomeView(onSend: () {}, onReceive: () {}, onSettings: () {}),
          (_) async {},
        ),
        'send': () => (
          SendView(
            viewModel: SendViewModel(
              repository: FakeRepository(),
              fileSource: FakeFileSource(),
            ),
          ),
          (_) async {},
        ),
        'send with the code': () {
          final repo = FakeRepository();
          final vm = SendViewModel(
            repository: repo,
            fileSource: FakeFileSource(),
          )..addFakeFileForTest('Rapport annuel 2026 (version finale).pdf');
          return (
            SendView(viewModel: vm),
            (tester) async {
              unawaited(vm.start());
              repo.prepared.complete(FakeSession());
            },
          );
        },
        'sending with the words': () {
          final repo = FakeRepository();
          final vm = SendViewModel(
            repository: repo,
            fileSource: FakeFileSource(),
          )..addFakeFileForTest('Rapport annuel 2026 (version finale).pdf');
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
            },
          );
        },
        'receive': () => (
          ReceiveView(
            viewModel: ReceiveViewModel(repository: FakeRepository()),
          ),
          (_) async {},
        ),
        'receive an offer': () {
          final repo = FakeRepository();
          final vm = ReceiveViewModel(repository: repo)
            ..setInput('7-harbor-rocket');
          return (
            ReceiveView(viewModel: vm),
            (tester) async {
              unawaited(vm.connect());
              repo.connected.complete(FakeOffer());
            },
          );
        },
        'settings': () =>
            (SettingsView(settings: settings, version: '0.3.0'), (_) async {}),
      };

  // The check itself: on a screen too narrow for the code, the same steps
  // report an overflow.
  testWidgets('an overflow fails the check', (tester) async {
    await tester.binding.setSurfaceSize(const Size(160, 740));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    final (widget, reach) = screens['send with the code']!();
    await tester.pumpWidget(_app(const Locale('en'), 1.0, widget));
    await reach(tester);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));
    expect(tester.takeException(), isNotNull);
  });

  for (final locale in AppLocalizations.supportedLocales) {
    for (final scale in [1.0, 1.3]) {
      for (final MapEntry(key: name, value: screen) in screens.entries) {
        testWidgets('$name in $locale at ${scale}x', (tester) async {
          await tester.binding.setSurfaceSize(_phone);
          addTearDown(() => tester.binding.setSurfaceSize(null));
          final (widget, reach) = screen();
          await tester.pumpWidget(_app(locale, scale, widget));
          await reach(tester);
          // The turning mark never settles: let the first frames pass.
          await tester.pump();
          await tester.pump(const Duration(milliseconds: 300));
          // The screen is in the state named.
          final shown = _shows[name];
          if (shown != null) {
            expect(
              find.textContaining(shown, findRichText: true),
              findsWidgets,
            );
          }

          // An overflow is reported as an exception.
          expect(tester.takeException(), isNull);
          final direction = Directionality.of(
            tester.element(find.byType(Scaffold).first),
          );
          expect(
            direction,
            _rtl.contains(locale.languageCode)
                ? TextDirection.rtl
                : TextDirection.ltr,
          );
        });
      }
    }
  }
}
