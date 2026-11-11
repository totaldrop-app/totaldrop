// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Every screen in every language (#315, #52, ADR 0008): nothing
// overflows on a small phone, also with the largest text Android offers
// (200 %, #322), and right-to-left languages lay out right to left. The
// test font draws each letter as a full square: as wide as Japanese and
// wider than Latin or Arabic, so a line that fits here fits on a phone.
// How the text looks (shaping, fonts) is checked by hand on a phone in
// Arabic and Japanese.
import 'dart:async';

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

/// An offer of ten files with long names: six are listed, then "and 4
/// more".
class _ManyOffer extends FakeOffer {
  @override
  PlatformInt64 get files => 10;
  @override
  List<String> get names => [for (var i = 0; i < 10; i++) '$i $_longName'];
}

/// A repository that reads no code in what was typed.
class _NoCodeRepository extends FakeRepository {
  @override
  String? normalizeCode(String input) => null;
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

/// What each screen shows once it reaches its state: a text, or a text
/// of the language shown.
final _shows = <String, Object>{
  'send with files': 'Rapport annuel',
  'send with the code': '7-harbor-rocket',
  'sending with the words': 'abstract',
  'sending, after the receiver accepted': 'abstract',
  'receive an offer': 'Rapport annuel',
  'send a text': 'Rapport annuel',
  'receive a text offer': 'one',
  'receive a text': 'Rapport annuel',
  'send preparing': (AppLocalizations l) => l.sendPreparing,
  'send, a pick failed': (AppLocalizations l) => l.sendAddFailed,
  'send with files, a pick failed': (AppLocalizations l) => l.sendAddFailed,
  'sending a large file': '%',
  'sending, asked to stop': (AppLocalizations l) => l.cancelTransferTitle,
  'send done': (AppLocalizations l) => l.sendDone,
  'send failed, details open': 'wss://relay.totaldrop.app',
  'receive an invalid code': (AppLocalizations l) => l.receiveCodeInvalid,
  'receive connecting': '7-harbor-rocket',
  'receive an offer of many files': (AppLocalizations l) => l.receiveAndMore(4),
  'receiving a large file': '%',
  'receive done': (AppLocalizations l) => l.receiveDone,
  'receive declined': (AppLocalizations l) => l.receiveDeclined,
  'receive failed, details open': 'wss://relay.totaldrop.app',
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

/// A finished receive of files, one of which could not be saved.
TransferEvent _filesReceived() => const TransferEvent(
  kind: EventKind.finished,
  words: [],
  done: 0,
  total: 0,
  summary: TransferSummary(
    filesOk: 1,
    filesFailed: 1,
    bytes: 40,
    saved: ['/tmp/TotalDrop/$_longName'],
    text: null,
  ),
  error: null,
);

/// Halfway through a large file: the longest progress line.
const _halfway = TransferEvent(
  kind: EventKind.progress,
  words: [],
  done: 1234567890,
  total: 3456789012,
  summary: null,
  error: null,
);

/// A failure with a cause and a long technical text, the longest version
/// of the error screen once "More information" is open.
const _failed = TransferEvent(
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
        'wss://relay.totaldrop.app, none of them answered in time',
    cause: 'TimedOut',
  ),
);

/// Opens "More information" under an error.
Future<void> _openDetails(WidgetTester tester) async {
  await tester.pump();
  final l = AppLocalizations.of(tester.element(find.byType(Scaffold).first));
  // Below the fold with the largest text: scroll to it as a person would.
  await tester.ensureVisible(find.text(l.moreInfo));
  await tester.pump();
  await tester.tap(find.text(l.moreInfo));
  await tester.pump();
}

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
        'send preparing': () {
          final vm = SendViewModel(
            repository: FakeRepository(),
            fileSource: FakeFileSource(),
          )..addFakeFileForTest(_longName);
          return (
            SendView(viewModel: vm),
            (tester) async => unawaited(vm.start()),
          );
        },
        'send, a pick failed': () {
          final source = FakeFileSource();
          final vm = SendViewModel(
            repository: FakeRepository(),
            fileSource: source,
          );
          return (
            SendView(viewModel: vm),
            (tester) async {
              unawaited(vm.addFiles());
              await tester.pump();
              source.fail(Exception('unknown_path'));
            },
          );
        },
        'send with files, a pick failed': () {
          final source = FakeFileSource();
          final vm = SendViewModel(
            repository: FakeRepository(),
            fileSource: source,
          )..addFakeFileForTest(_longName);
          return (
            SendView(viewModel: vm),
            (tester) async {
              unawaited(vm.addFiles());
              await tester.pump();
              source.fail(Exception('unknown_path'));
            },
          );
        },
        'sending, asked to stop': () {
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
              repo.events.add(_halfway);
              await tester.pump();
              // Back during a transfer asks first.
              final navigator = tester.state<NavigatorState>(
                find.byType(Navigator),
              );
              unawaited(navigator.maybePop());
            },
          );
        },
        'receive an invalid code': () {
          final vm = ReceiveViewModel(repository: _NoCodeRepository())
            ..setInput('7-harbor');
          return (ReceiveView(viewModel: vm), (tester) => vm.connect());
        },
        'receive an offer of many files': () {
          final repo = FakeRepository();
          final vm = ReceiveViewModel(repository: repo)
            ..setInput('7-harbor-rocket');
          return (
            ReceiveView(viewModel: vm),
            (tester) async {
              unawaited(vm.connect());
              repo.connected.complete(_ManyOffer());
            },
          );
        },
        'sending a large file': () {
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
              repo.events.add(_halfway);
            },
          );
        },
        'send done': () {
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
              repo.events.add(_filesReceived());
            },
          );
        },
        'send failed, details open': () {
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
              repo.events.add(_failed);
              await _openDetails(tester);
            },
          );
        },
        'receive connecting': () {
          final vm = ReceiveViewModel(repository: FakeRepository())
            ..setInput('7-harbor-rocket');
          return (
            ReceiveView(viewModel: vm),
            (tester) async => unawaited(vm.connect()),
          );
        },
        'receiving a large file': () {
          final repo = FakeRepository();
          final vm = ReceiveViewModel(repository: repo)
            ..setInput('7-harbor-rocket');
          return (
            ReceiveView(viewModel: vm),
            (tester) async {
              unawaited(vm.connect());
              repo.connected.complete(_LongOffer());
              await tester.pump();
              await vm.accept();
              repo.events.add(_halfway);
            },
          );
        },
        'receive done': () {
          final repo = FakeRepository();
          final vm = ReceiveViewModel(repository: repo)
            ..setInput('7-harbor-rocket');
          return (
            ReceiveView(viewModel: vm),
            (tester) async {
              unawaited(vm.connect());
              repo.connected.complete(_LongOffer());
              await tester.pump();
              await vm.accept();
              repo.events.add(_filesReceived());
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
              repo.connected.complete(_LongOffer());
              await tester.pump();
              await vm.decline();
            },
          );
        },
        'receive failed, details open': () {
          final repo = FakeRepository();
          final vm = ReceiveViewModel(repository: repo)
            ..setInput('7-harbor-rocket');
          return (
            ReceiveView(viewModel: vm),
            (tester) async {
              unawaited(vm.connect());
              repo.connected.complete(_LongOffer());
              await tester.pump();
              await vm.accept();
              repo.events.add(_failed);
              await _openDetails(tester);
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

  // The check itself: the same steps report content that runs off the
  // phone, across and down. The app's screens no longer do, even on a
  // screen far too small, so a row and a column too large for the phone
  // stand in for them.
  for (final (name, tooLarge) in [
    ('across', const Row(children: [SizedBox(width: 400, height: 10)])),
    ('down', const Column(children: [SizedBox(width: 10, height: 800)])),
  ]) {
    testWidgets('an overflow $name fails the check', (tester) async {
      await tester.binding.setSurfaceSize(_phone);
      addTearDown(() => tester.binding.setSurfaceSize(null));
      await tester.pumpWidget(
        _app(const Locale('en'), 1.0, Scaffold(body: tooLarge)),
      );
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
  }

  for (final locale in AppLocalizations.supportedLocales) {
    for (final scale in [1.0, 1.3, 2.0]) {
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
          final l = AppLocalizations.of(
            tester.element(find.byType(Scaffold).first),
          );
          final shown = switch (_shows[name]) {
            final String text => text,
            final String Function(AppLocalizations) text => text(l),
            _ => null,
          };
          // Found at the top, or below the fold once scrolled to (a list
          // builds only what is near the screen).
          bool showing() {
            if (shown == null) return true;
            final found = find.textContaining(shown, findRichText: true);
            return found.evaluate().isNotEmpty;
          }

          var shownSomewhere = showing();

          // An overflow, across or down, is reported as an exception: a
          // screen whose content can be taller than the phone scrolls.
          expect(tester.takeException(), isNull);

          // A list lays out only what is near the screen: page through each
          // one to its end, half a screen at a time, as a person would, and
          // check everything that comes into view.
          final lists = tester.stateList<ScrollableState>(
            find.byType(Scrollable),
          );
          for (final list in lists.toList()) {
            for (var page = 0; page < 100; page++) {
              if (!list.mounted) break;
              final position = list.position;
              if (position.axis != Axis.vertical) break;
              if (position.pixels >= position.maxScrollExtent) break;
              final next = position.pixels + position.viewportDimension / 2;
              position.jumpTo(next.clamp(0.0, position.maxScrollExtent));
              await tester.pump();
              expect(tester.takeException(), isNull);
              shownSomewhere = shownSomewhere || showing();
            }
          }
          expect(shownSomewhere, isTrue, reason: 'shows $shown');
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
