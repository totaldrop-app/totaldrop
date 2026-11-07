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
import 'package:totaldrop/data/background_access.dart';
import 'package:totaldrop/data/settings_repository.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/l10n.dart' show ShownText;
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

  // Both notes under a received text: the longest version of the screen.
  @override
  ShownText crateApiL10NShownText({required String text}) =>
      ShownText(text: text, short: true, hidden: true);

  @override
  String crateApiTransferDefaultServer() => 'wss://rendezvous.totaldrop.app';

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

const _longName = 'Rapport annuel 2026 (version finale).pdf';

/// An offer of one file with a long name.
class _LongOffer extends FakeOffer {
  @override
  List<String> get names => const [_longName];
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

/// An Android phone, so Settings shows its Background transfers section.
class _Limits implements BackgroundAccess {
  const _Limits();

  @override
  Future<BackgroundState> state() async =>
      const BackgroundState(exempt: false, maker: 'Xiaomi', autostart: true);

  @override
  Future<bool> open(BackgroundPage page) async => true;
}

/// Text each screen shows once it reaches its state.
const _shows = {
  'send with files': 'Rapport annuel',
  'send with the code': '7-harbor-rocket',
  'sending with the words': 'abstract',
  'sending, after the receiver accepted': 'abstract',
  'receive an offer': 'Rapport annuel',
  'send a text': 'Rapport annuel',
  'receive a text offer': 'one',
  'receive a text': 'Rapport annuel',
};

/// A finished receive of a text.
TransferEvent _textReceived() => const TransferEvent(
  kind: EventKind.finished,
  words: [],
  done: 0,
  total: 0,
  summary: TransferSummary(
    filesOk: 1,
    filesFailed: 0,
    bytes: 40,
    saved: [],
    text: _longName,
  ),
  error: null,
);

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
        'send with files': () => (
          SendView(
            viewModel: SendViewModel(
              repository: FakeRepository(),
              fileSource: FakeFileSource(),
            )..addFakeFileForTest(_longName),
          ),
          (_) async {},
        ),
        'send with the code': () {
          final repo = FakeRepository();
          final vm = SendViewModel(
            repository: repo,
            fileSource: FakeFileSource(),
          )..addFakeFileForTest(_longName);
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
          )..addFakeFileForTest(_longName);
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
        'sending, after the receiver accepted': () {
          final repo = FakeRepository();
          final vm = SendViewModel(
            repository: repo,
            fileSource: FakeFileSource(),
          )..addFakeFileForTest(_longName);
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
                  done: 1,
                  total: 10,
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
              repo.connected.complete(_LongOffer());
            },
          );
        },
        'send a text': () {
          final vm =
              SendViewModel(
                  repository: FakeRepository(),
                  fileSource: FakeFileSource(),
                  textClipboard: const FakeClipboard('other'),
                )
                ..setTextMode(true)
                ..setText(_longName);
          return (
            SendView(viewModel: vm),
            // The notice that the typed text was kept.
            (tester) => vm.pasteClipboard(),
          );
        },
        'receive a text offer': () {
          final repo = FakeRepository();
          final vm = ReceiveViewModel(repository: repo)
            ..setInput('7-harbor-rocket');
          return (
            ReceiveView(viewModel: vm),
            (tester) async {
              unawaited(vm.connect());
              repo.connected.complete(FakeOffer(isText: true));
            },
          );
        },
        'receive a text': () {
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
              repo.events.add(_textReceived());
            },
          );
        },
        'settings': () => (
          SettingsView(
            settings: settings,
            version: '0.3.0',
            background: const _Limits(),
          ),
          (_) async {},
        ),
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
    expect(
      tester.takeException(),
      isA<FlutterError>().having(
        (e) => e.toString(),
        'message',
        contains('overflowed'),
      ),
    );
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

          // A horizontal overflow is reported as an exception. Every screen
          // scrolls, so nothing can overflow vertically.
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
