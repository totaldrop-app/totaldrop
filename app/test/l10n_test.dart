// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:flutter/rendering.dart' show RenderParagraph;
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/src/rust/frb_generated.dart';
import 'package:totaldrop/ui/core/common.dart';
import 'package:totaldrop/ui/core/l10n.dart';
import 'package:totaldrop/ui/core/theme.dart';
import 'package:totaldrop/ui/receive/receive_view.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';

import 'fakes.dart';

class _DraftRust implements RustLibApi {
  _DraftRust({required this.draft});

  bool draft;

  @override
  bool crateApiL10NNeedsEnglish({required String tag, required String key}) =>
      draft && tag != 'en';

  @override
  String crateApiL10NRevealControls({required String name}) => name;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

// Sizes must read the same as in the Rust programs (totaldrop-l10n's
// `sizes` test); plurals and counts follow Flutter's generated code.
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final en = lookupAppLocalizations(const Locale('en'));
  final tr = lookupAppLocalizations(const Locale('tr'));
  final rust = _DraftRust(draft: true);

  setUpAll(() => RustLib.initMock(api: rust));

  test('sizes match the Rust programs', () {
    expect(formatBytes(en, 0, numberLocale: 'en'), '0 B');
    expect(formatBytes(en, 999, numberLocale: 'en'), '999 B');
    expect(formatBytes(en, 1000, numberLocale: 'en'), '1.0 kB');
    expect(formatBytes(en, 1549999, numberLocale: 'en'), '1.5 MB');
    expect(formatBytes(en, 9960000, numberLocale: 'en'), '10 MB');
    expect(formatBytes(en, 12450000, numberLocale: 'en'), '12 MB');
    expect(formatBytes(en, 12500000, numberLocale: 'en'), '13 MB');
    expect(formatBytes(en, 999950, numberLocale: 'en'), '1.0 MB');
    expect(formatBytes(en, 999999999, numberLocale: 'en'), '1.0 GB');
    expect(formatBytes(tr, 1500000, numberLocale: 'tr'), '1,5 MB');
    expect(formatBytes(en, 1500000, numberLocale: 'de'), '1,5 MB');
    expect(formatBytes(en, 999499, numberLocale: 'en'), '999 kB');
    expect(formatBytes(en, 12345678, numberLocale: 'en'), '12 MB');
    expect(
      formatBytes(en, 9223372036854775807, numberLocale: 'en'),
      '9,223 PB',
    );
  });

  test('counts are grouped and plurals follow the language', () {
    expect(en.itemsAndSize(1, '2 MB'), '1 item · 2 MB');
    expect(en.itemsAndSize(1234, '2 MB'), '1,234 items · 2 MB');
    expect(tr.itemsAndSize(1234, '2 MB'), '1.234 öğe · 2 MB');
  });

  // Settings says, in the person's own language, why the code and the
  // verification words stay English (#305, ADR 0008): a note left in
  // English would not reach the people it is for.
  test('every language explains that codes and words stay English', () {
    final english = en.settingsWordsEnglish;
    for (final locale in AppLocalizations.supportedLocales) {
      if (locale.languageCode == 'en') continue;
      final note = lookupAppLocalizations(locale).settingsWordsEnglish;
      expect(note, isNot(english), reason: '$locale');
      expect(note.trim(), isNotEmpty, reason: '$locale');
    }
  });

  test('the percent rounds down and follows the region', () {
    expect(formatPercent(en, 0, 0, numberLocale: 'en'), '0%');
    expect(formatPercent(en, 0, 7, numberLocale: 'en'), '0%');
    expect(formatPercent(en, 29, 100, numberLocale: 'en'), '29%');
    expect(formatPercent(en, 995, 1000, numberLocale: 'en'), '99%');
    expect(formatPercent(en, 999999, 1000000, numberLocale: 'en'), '99%');
    expect(formatPercent(en, 7, 7, numberLocale: 'en'), '100%');
    expect(formatPercent(en, 8, 7, numberLocale: 'en'), '100%');
    expect(formatPercent(en, 1, 2, numberLocale: 'tr'), '%50');
    expect(formatPercent(en, 1, 2, numberLocale: 'de'), '50\u00a0%');
  });

  // Safety-critical texts fall back to English while a language is an
  // unreviewed draft (#509). In Arabic, Persian, Hebrew and Urdu, that
  // English fallback must still lay out left to right so trailing
  // punctuation stays at the end instead of jumping to the start
  // (".differ, stop").
  testWidgets(
    'English safety fallbacks lay out left to right in RTL languages',
    (tester) async {
      await tester.binding.setSurfaceSize(const Size(360, 740));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      addTearDown(() => rust.draft = true);

      Widget wrap(Locale locale, Widget child) => MaterialApp(
        theme: buildTheme(fallbackScheme(Brightness.light)),
        locale: locale,
        localizationsDelegates: const [
          AppLocalizations.delegate,
          ...GlobalMaterialLocalizations.delegates,
        ],
        supportedLocales: AppLocalizations.supportedLocales,
        builder: (context, inner) => MediaQuery(
          data: MediaQuery.of(context)
              .copyWith(textScaler: const TextScaler.linear(2.0)),
          child: SystemMotion(child: inner!),
        ),
        home: Scaffold(body: child),
      );

      RenderParagraph paragraphOf(String text) =>
          tester.renderObject<RenderParagraph>(find.text(text));

      for (final code in const ['ar', 'fa', 'he', 'ur']) {
        final locale = Locale(code);
        rust.draft = true;

        // 1. Offered files screen: verificationTitle, verificationHint,
        //    receiveDecline, receiveAccept.
        final repo = FakeRepository();
        final vm = ReceiveViewModel(repository: repo)
          ..setInput('7-harbor-rocket');
        final connecting = vm.connect();
        repo.connected.complete(FakeOffer());
        await connecting;
        await tester.pumpWidget(wrap(locale, ReceiveView(viewModel: vm)));
        await tester.pump();

        for (final text in [en.verificationTitle, en.verificationHint]) {
          expect(
            paragraphOf(text).textDirection,
            TextDirection.ltr,
            reason: '$code draft: $text',
          );
        }

        // Heading icon sits on the left of the English heading even when
        // 2.0x text wraps it onto two lines.
        expect(
          tester.getRect(find.byIcon(Icons.verified_user_outlined)).right,
          lessThanOrEqualTo(
            tester.getRect(find.text(en.verificationTitle)).left + 0.5,
          ),
          reason: '$code draft icon left of English verificationTitle',
        );

        // The period at the end of verificationHint ("... differ, stop.")
        // sits to the right of the word before it, never at the start.
        final hint = paragraphOf(en.verificationHint);
        final lastChar = en.verificationHint.length - 1;
        final stopStart = en.verificationHint.lastIndexOf('stop');
        final wordBox = hint
            .getBoxesForSelection(
              TextSelection(baseOffset: stopStart, extentOffset: lastChar),
            )
            .single;
        final dotBox = hint
            .getBoxesForSelection(
              TextSelection(baseOffset: lastChar, extentOffset: lastChar + 1),
            )
            .single;
        expect(
          dotBox.left,
          greaterThanOrEqualTo(wordBox.right - 0.5),
          reason: '$code draft period after "stop"',
        );

        await tester.scrollUntilVisible(find.text(en.receiveDecline), 150);
        for (final text in [en.receiveDecline, en.receiveAccept]) {
          expect(
            paragraphOf(text).textDirection,
            TextDirection.ltr,
            reason: '$code draft: $text',
          );
        }

        // 2. Offered text screen: receiveDecline and receiveAccept there too.
        final textRepo = FakeRepository();
        final textVm = ReceiveViewModel(repository: textRepo)
          ..setInput('7-harbor-rocket');
        final textConnecting = textVm.connect();
        textRepo.connected.complete(FakeOffer(isText: true));
        await textConnecting;
        await tester.pumpWidget(wrap(locale, ReceiveView(viewModel: textVm)));
        await tester.pump();
        await tester.scrollUntilVisible(find.text(en.receiveDecline), 150);
        expect(
          paragraphOf(en.receiveDecline).textDirection,
          TextDirection.ltr,
          reason: '$code text offer decline',
        );
        expect(
          paragraphOf(en.receiveAccept).textDirection,
          TextDirection.ltr,
          reason: '$code text offer accept',
        );

        // 3. ErrorPanel: errorWrongCode and errorInvalidOffer.
        for (final (kind, text) in [
          (ErrorKind.wrongCode, en.errorWrongCode),
          (ErrorKind.invalidOffer, en.errorInvalidOffer),
        ]) {
          await tester.pumpWidget(
            wrap(
              locale,
              CenteredScroll(
                child: ErrorPanel(
                  error: TransferError(kind: kind, detail: '', cause: ''),
                  onRetry: () {},
                ),
              ),
            ),
          );
          expect(
            paragraphOf(text).textDirection,
            TextDirection.ltr,
            reason: '$code draft: $kind',
          );
        }

        // Once reviewed (`needsEnglish == false`), the language's own
        // strings lay out right to left again across all six critical keys.
        rust.draft = false;
        final native = lookupAppLocalizations(locale);
        await tester.pumpWidget(wrap(locale, ReceiveView(viewModel: vm)));
        await tester.pump();
        expect(
          paragraphOf(native.verificationTitle).textDirection,
          TextDirection.rtl,
          reason: '$code reviewed title',
        );
        expect(
          tester.getRect(find.byIcon(Icons.verified_user_outlined)).left,
          greaterThanOrEqualTo(
            tester.getRect(find.text(native.verificationTitle)).right - 0.5,
          ),
          reason: '$code reviewed icon right of RTL verificationTitle',
        );
        expect(
          paragraphOf(native.verificationHint).textDirection,
          TextDirection.rtl,
          reason: '$code reviewed hint',
        );
        await tester.scrollUntilVisible(find.text(native.receiveDecline), 150);
        expect(
          paragraphOf(native.receiveDecline).textDirection,
          TextDirection.rtl,
          reason: '$code reviewed decline',
        );
        expect(
          paragraphOf(native.receiveAccept).textDirection,
          TextDirection.rtl,
          reason: '$code reviewed accept',
        );
        await tester.pumpWidget(wrap(locale, ReceiveView(viewModel: textVm)));
        await tester.pump();
        await tester.scrollUntilVisible(find.text(native.receiveDecline), 150);
        expect(
          paragraphOf(native.receiveDecline).textDirection,
          TextDirection.rtl,
          reason: '$code reviewed text offer decline',
        );
        expect(
          paragraphOf(native.receiveAccept).textDirection,
          TextDirection.rtl,
          reason: '$code reviewed text offer accept',
        );
        for (final (kind, text) in [
          (ErrorKind.wrongCode, native.errorWrongCode),
          (ErrorKind.invalidOffer, native.errorInvalidOffer),
        ]) {
          await tester.pumpWidget(
            wrap(
              locale,
              CenteredScroll(
                child: ErrorPanel(
                  error: TransferError(kind: kind, detail: '', cause: ''),
                  onRetry: () {},
                ),
              ),
            ),
          );
          expect(
            paragraphOf(text).textDirection,
            TextDirection.rtl,
            reason: '$code reviewed $kind',
          );
        }
      }
    },
  );
}
