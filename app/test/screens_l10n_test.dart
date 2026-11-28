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
import 'package:flutter/rendering.dart' show RenderParagraph;
import 'package:flutter_test/flutter_test.dart';
import 'package:intl/intl.dart' as intl;
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
  String? crateApiL10NSingleWebUrl({required String text}) =>
      text.startsWith('https://') ? text : null;

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
  'receive a link': (AppLocalizations l) => l.openLink,
  'receive a link, asked to open': (AppLocalizations l) => l.openLinkTitle,
  'send preparing': (AppLocalizations l) => l.sendPreparing,
  'send, a pick failed': (AppLocalizations l) => l.sendAddFailed,
  'send with files, a pick failed': (AppLocalizations l) => l.sendAddFailed,
  'send with many files': '30 Rapport annuel',
  'send, a share being copied': (AppLocalizations l) => l.cancel,
  'sending a large file': _expectedHalfwayPercent,
  'sending, asked to stop': (AppLocalizations l) => l.cancelTransferTitle,
  'send done': (AppLocalizations l) => l.sendDone,
  'send failed, details open': 'wss://relay.totaldrop.app',
  'receive an invalid code': (AppLocalizations l) => l.receiveCodeInvalid,
  'receive connecting': '7-harbor-rocket',
  'receive an offer of many files': (AppLocalizations l) => l.receiveAndMore(4),
  'receiving a large file': _expectedHalfwayPercent,
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

/// A finished receive of a single web link (#53).
TransferEvent _linkReceived() => const TransferEvent(
  kind: EventKind.finished,
  words: [],
  done: 0,
  total: 0,
  summary: TransferSummary(
    filesOk: 1,
    filesFailed: 0,
    bytes: 52,
    saved: [],
    text: 'https://totaldrop.app/documentation/getting-started',
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

/// Representative device region for each supported app language (#515), so
/// region-sensitive numbers and percentages (`50 %`, `٥٠٪`, `۵۰٪`, `৫০%`)
/// are laid out and checked at every text scale instead of `en-US` everywhere.
Locale _regionOf(Locale locale) =>
    switch ((locale.languageCode, locale.scriptCode, locale.countryCode)) {
      ('ar', _, _) => const Locale('ar', 'EG'),
      ('bn', _, _) => const Locale('bn', 'BD'),
      ('cs', _, _) => const Locale('cs', 'CZ'),
      ('da', _, _) => const Locale('da', 'DK'),
      ('de', _, _) => const Locale('de', 'DE'),
      ('el', _, _) => const Locale('el', 'GR'),
      ('en', _, _) => const Locale('en', 'US'),
      ('es', _, _) => const Locale('es', 'ES'),
      ('fa', _, _) => const Locale('fa', 'IR'),
      ('fi', _, _) => const Locale('fi', 'FI'),
      ('fr', _, _) => const Locale('fr', 'FR'),
      ('he', _, _) => const Locale('he', 'IL'),
      ('hi', _, _) => const Locale('hi', 'IN'),
      ('hu', _, _) => const Locale('hu', 'HU'),
      ('id', _, _) => const Locale('id', 'ID'),
      ('it', _, _) => const Locale('it', 'IT'),
      ('ja', _, _) => const Locale('ja', 'JP'),
      ('ko', _, _) => const Locale('ko', 'KR'),
      ('ms', _, _) => const Locale('ms', 'MY'),
      ('nb', _, _) => const Locale('nb', 'NO'),
      ('nl', _, _) => const Locale('nl', 'NL'),
      ('pl', _, _) => const Locale('pl', 'PL'),
      ('pt', _, 'PT') => const Locale('pt', 'PT'),
      ('pt', _, _) => const Locale('pt', 'BR'),
      ('ro', _, _) => const Locale('ro', 'RO'),
      ('ru', _, _) => const Locale('ru', 'RU'),
      ('sv', _, _) => const Locale('sv', 'SE'),
      ('sw', _, _) => const Locale('sw', 'KE'),
      ('th', _, _) => const Locale('th', 'TH'),
      ('tr', _, _) => const Locale('tr', 'TR'),
      ('uk', _, _) => const Locale('uk', 'UA'),
      ('ur', _, _) => const Locale('ur', 'PK'),
      ('vi', _, _) => const Locale('vi', 'VN'),
      ('zh', 'Hant', _) => const Locale('zh', 'TW'),
      ('zh', _, _) => const Locale('zh', 'CN'),
      _ => throw StateError('No test region configured for $locale'),
    };

/// `intl` keeps separate regional tables only where a language has distinct
/// regional number formats (for example `ar_EG` vs `ar`, `pt_BR` vs `pt_PT`);
/// for single-default languages such as `de_DE` or `tr_TR`, `intl` resolves
/// the canonical tag to its language table (`de`, `tr`).
const _regionalNumberLocales = {
  'ar_EG',
  'en_US',
  'es_ES',
  'pt_BR',
  'pt_PT',
  'zh_CN',
  'zh_TW',
};

/// Expected percent text for [_halfway] (`1234567890 / 3456789012`, 35%)
/// under each language's representative region from [_regionOf].
///
/// Kept as literal strings rather than calling `formatPercent` so the screen
/// tests verify the formatted output independently of the code under test.
String _expectedHalfwayPercent(AppLocalizations l) => switch (l.localeName) {
  'ar' => '\u0663\u0665\u066a\u061c',
  'bn' => '\u09e9\u09eb%',
  'cs' => '35\u00a0%',
  'da' => '35\u00a0%',
  'de' => '35\u00a0%',
  'el' => '35%',
  'en' => '35%',
  'es' => '35\u00a0%',
  'fa' => '\u06f3\u06f5\u066a',
  'fi' => '35\u00a0%',
  'fr' => '35\u00a0%',
  'he' => '35%',
  'hi' => '35%',
  'hu' => '35%',
  'id' => '35%',
  'it' => '35%',
  'ja' => '35%',
  'ko' => '35%',
  'ms' => '35%',
  'nb' => '35\u00a0%',
  'nl' => '35%',
  'pl' => '35%',
  'pt' => '35%',
  'pt_BR' => '35%',
  'pt_PT' => '35%',
  'ro' => '35\u00a0%',
  'ru' => '35\u00a0%',
  'sv' => '35\u00a0%',
  'sw' => '35%',
  'th' => '35%',
  'tr' => '%35',
  'uk' => '35%',
  'ur' => '35%',
  'vi' => '35%',
  'zh' => '35%',
  'zh_Hans' => '35%',
  'zh_Hant' => '35%',
  _ => throw StateError('No expected percent for ${l.localeName}'),
};

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

/// A button's label on two lines runs into the button's round ends:
/// 受け取る at 200 % in a half-width button (#480). Checked for buttons
/// side by side in a row; the test font draws every letter as a square,
/// so a full-width label wraps here long before it does on a phone. A
/// label that wraps is narrower than the same label on one line.
void _expectButtonLabelsOnOneLine(WidgetTester tester) {
  final labels = find.descendant(
    of: find.descendant(
      of: find.byType(Row),
      matching: find.byWidgetPredicate((w) => w is ButtonStyleButton),
    ),
    matching: find.byType(RichText),
  );
  for (final element in labels.evaluate()) {
    final box = element.renderObject! as RenderBox;
    final oneLine = box.getMaxIntrinsicWidth(double.infinity);
    final label = (element.widget as RichText).text.toPlainText();
    expect(
      box.size.width,
      greaterThanOrEqualTo(oneLine - 0.5),
      reason: 'the button "$label" wraps',
    );
  }
}

/// A message given room for several lines must show all of it: the
/// not-a-code message at 200 % in Japanese stopped after three lines with
/// "…" (#480), and no overflow is reported for it. Texts kept to one line
/// (file names, the code, titles) are cut on purpose; the test font, wider
/// than any real one, cuts them far sooner than a phone does.
void _expectNoTextCutShort(WidgetTester tester) {
  final paragraphs = tester.allRenderObjects.whereType<RenderParagraph>();
  for (final paragraph in paragraphs) {
    if ((paragraph.maxLines ?? 0) < 2) continue;
    expect(
      paragraph.didExceedMaxLines,
      isFalse,
      reason: 'the text "${paragraph.text.toPlainText()}" is cut short',
    );
  }
}

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
        'receive a link': () {
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
              repo.events.add(_linkReceived());
            },
          );
        },
        'receive a link, asked to open': () {
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
              repo.events.add(_linkReceived());
              await tester.pump();
              await tester.pump(const Duration(milliseconds: 300));
              final l = AppLocalizations.of(
                tester.element(find.byType(Scaffold).first),
              );
              await tester.scrollUntilVisible(
                find.text(l.openLink),
                200,
                scrollable: find.byType(Scrollable).first,
              );
              await tester.pump();
              await tester.tap(find.text(l.openLink));
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
        // The list scrolls above the pinned box, which takes at most half
        // the screen; the last file is reached by scrolling.
        'send with many files': () {
          final vm = SendViewModel(
            repository: FakeRepository(),
            fileSource: FakeFileSource(),
          );
          for (var i = 1; i <= 30; i++) {
            vm.addFakeFileForTest('$i $_longName');
          }
          return (SendView(viewModel: vm), (_) async {});
        },
        // Files shared from another app are copied first; that copy can be
        // stopped, so the wait has a Cancel button.
        'send, a share being copied': () => (
          SendView(
            viewModel: SendViewModel(
              repository: FakeRepository(),
              fileSource: FakeFileSource(),
            )..beginShare(1),
          ),
          (_) async {},
        ),
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

  testWidgets('a button label on two lines fails the check', (tester) async {
    await tester.binding.setSurfaceSize(_phone);
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _app(
        const Locale('en'),
        1.0,
        Scaffold(
          body: Row(
            children: [
              Expanded(
                child: FilledButton(
                  onPressed: () {},
                  child: const Text('one two three four five six'),
                ),
              ),
              const Spacer(),
            ],
          ),
        ),
      ),
    );
    expect(
      () => _expectButtonLabelsOnOneLine(tester),
      throwsA(isA<TestFailure>()),
    );
  });

  testWidgets('a text cut short fails the check', (tester) async {
    await tester.binding.setSurfaceSize(_phone);
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _app(
        const Locale('en'),
        1.0,
        Scaffold(
          body: TextField(
            decoration: InputDecoration(
              errorText: List.filled(40, 'word').join(' '),
              errorMaxLines: 2,
            ),
          ),
        ),
      ),
    );
    expect(() => _expectNoTextCutShort(tester), throwsA(isA<TestFailure>()));
  });

  test('every supported locale has a known regional number format', () {
    expect(_halfway.done * 100 ~/ _halfway.total, 35);
    for (final locale in AppLocalizations.supportedLocales) {
      final region = _regionOf(locale);
      expect(region.countryCode, isNotEmpty, reason: '$locale');
      final tag = region.toLanguageTag();
      final canonical = intl.Intl.canonicalizedLocale(tag);
      final expectedLocale = _regionalNumberLocales.contains(canonical)
          ? canonical
          : region.languageCode;
      final dec = intl.NumberFormat.decimalPattern(tag);
      final pct = intl.NumberFormat.percentPattern(tag);
      expect(dec.locale, expectedLocale, reason: '$locale -> $tag');
      expect(pct.locale, expectedLocale, reason: '$locale -> $tag');
      expect(
        pct.format(0.35),
        _expectedHalfwayPercent(lookupAppLocalizations(locale)),
        reason: '$locale -> $tag',
      );
    }

    const zhHans = Locale.fromSubtags(languageCode: 'zh', scriptCode: 'Hans');
    const zhHant = Locale.fromSubtags(languageCode: 'zh', scriptCode: 'Hant');
    const samples = [
      (Locale('en'), Locale('en', 'US'), '1,234.5', '50%'),
      (Locale('de'), Locale('de', 'DE'), '1.234,5', '50\u00a0%'),
      (Locale('fr'), Locale('fr', 'FR'), '1\u202f234,5', '50\u00a0%'),
      (Locale('tr'), Locale('tr', 'TR'), '1.234,5', '%50'),
      (
        Locale('ar'),
        Locale('ar', 'EG'),
        '\u0661\u066c\u0662\u0663\u0664\u066b\u0665',
        '\u0665\u0660\u066a\u061c',
      ),
      (
        Locale('fa'),
        Locale('fa', 'IR'),
        '\u06f1\u066c\u06f2\u06f3\u06f4\u066b\u06f5',
        '\u06f5\u06f0\u066a',
      ),
      (
        Locale('bn'),
        Locale('bn', 'BD'),
        '\u09e7,\u09e8\u09e9\u09ea.\u09eb',
        '\u09eb\u09e6%',
      ),
      (Locale('pt'), Locale('pt', 'BR'), '1.234,5', '50%'),
      (Locale('pt', 'BR'), Locale('pt', 'BR'), '1.234,5', '50%'),
      (Locale('pt', 'PT'), Locale('pt', 'PT'), '1\u00a0234,5', '50%'),
      (Locale('zh'), Locale('zh', 'CN'), '1,234.5', '50%'),
      (zhHans, Locale('zh', 'CN'), '1,234.5', '50%'),
      (zhHant, Locale('zh', 'TW'), '1,234.5', '50%'),
    ];
    for (final (locale, expectedRegion, decimal, percent) in samples) {
      final region = _regionOf(locale);
      expect(region, expectedRegion, reason: '$locale');
      final tag = region.toLanguageTag();
      expect(
        intl.NumberFormat.decimalPattern(tag).format(1234.5),
        decimal,
        reason: '$locale -> $tag',
      );
      expect(
        intl.NumberFormat.percentPattern(tag).format(0.5),
        percent,
        reason: '$locale -> $tag',
      );
    }
  });

  for (final locale in AppLocalizations.supportedLocales) {
    for (final scale in [1.0, 1.3, 2.0]) {
      for (final MapEntry(key: name, value: screen) in screens.entries) {
        testWidgets('$name in $locale at ${scale}x', (tester) async {
          tester.platformDispatcher.localeTestValue = _regionOf(locale);
          addTearDown(tester.platformDispatcher.clearLocaleTestValue);
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
          _expectButtonLabelsOnOneLine(tester);
          _expectNoTextCutShort(tester);

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
              _expectButtonLabelsOnOneLine(tester);
              _expectNoTextCutShort(tester);
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
