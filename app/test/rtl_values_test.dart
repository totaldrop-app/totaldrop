// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:flutter/rendering.dart' show RenderParagraph;
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart'
    show PlatformInt64;
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/l10n.dart' show ShownText;
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/src/rust/frb_generated.dart';
import 'package:totaldrop/ui/core/common.dart';
import 'package:totaldrop/ui/core/l10n.dart';
import 'package:totaldrop/ui/core/theme.dart';
import 'package:totaldrop/ui/receive/receive_view.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

import 'fakes.dart';

class _Rust implements RustLibApi {
  @override
  bool crateApiL10NNeedsEnglish({required String tag, required String key}) =>
      false;

  @override
  String crateApiL10NRevealControls({required String name}) =>
      name.replaceAll('\u202E', '<U+202E>');

  @override
  ShownText crateApiL10NShownText({required String text}) =>
      ShownText(text: text, short: false, hidden: false);

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _NamedOffer extends FakeOffer {
  _NamedOffer(this._names);
  final List<String> _names;

  @override
  PlatformInt64 get files => _names.length;

  @override
  List<String> get names => _names;
}

Widget _wrap(Locale locale, Widget child, {double scale = 1.0}) => MaterialApp(
  theme: buildTheme(fallbackScheme(Brightness.light)),
  locale: locale,
  localizationsDelegates: const [
    AppLocalizations.delegate,
    ...GlobalMaterialLocalizations.delegates,
  ],
  supportedLocales: AppLocalizations.supportedLocales,
  builder: (context, inner) => MediaQuery(
    data: MediaQuery.of(context).copyWith(textScaler: TextScaler.linear(scale)),
    child: SystemMotion(child: inner!),
  ),
  home: Scaffold(body: child),
);

/// Bounding box of [substring] inside [paragraph] (first match from [from]).
TextBox _boxOf(RenderParagraph paragraph, String substring, [int from = 0]) {
  final full = paragraph.text.toPlainText();
  final start = full.indexOf(substring, from);
  expect(start, greaterThanOrEqualTo(0), reason: '"$substring" in "$full"');
  final boxes = paragraph.getBoxesForSelection(
    TextSelection(baseOffset: start, extentOffset: start + substring.length),
  );
  expect(boxes, isNotEmpty, reason: 'boxes for "$substring" in "$full"');
  return boxes.first;
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() => RustLib.initMock(api: _Rust()));

  const rtlRegions = [
    (Locale('ar'), Locale('ar', 'EG')),
    (Locale('fa'), Locale('fa', 'IR')),
    (Locale('he'), Locale('he', 'IL')),
    (Locale('ur'), Locale('ur', 'PK')),
  ];

  test(
    'firstStrongDirection follows UAX #9 P2-P3 across scripts and symbols',
    () {
      expect(firstStrongDirection('0 Rapport annuel.pdf'), TextDirection.ltr);
      expect(firstStrongDirection('٠ Rapport annuel.pdf'), TextDirection.ltr);
      expect(firstStrongDirection('۰ Rapport annuel.pdf'), TextDirection.ltr);
      expect(firstStrongDirection('٠٫٥ Report.pdf'), TextDirection.ltr);
      expect(firstStrongDirection('١٬٠٠٠ Report.pdf'), TextDirection.ltr);
      expect(firstStrongDirection('٪50 Report.pdf'), TextDirection.ltr);
      expect(firstStrongDirection('0 تقرير سنوي.pdf'), TextDirection.rtl);
      expect(firstStrongDirection('٠ تقرير سنوي.pdf'), TextDirection.rtl);
      expect(firstStrongDirection('📄 تقرير.pdf'), TextDirection.rtl);
      expect(firstStrongDirection('📷 صورة.jpg'), TextDirection.rtl);
      expect(firstStrongDirection('【تقرير】.pdf'), TextDirection.rtl);
      expect(
        firstStrongDirection('\u2068report\u2069 تقرير.pdf'),
        TextDirection.rtl,
      );
      expect(firstStrongDirection('𞤀𞤤𞤢𞤥.txt'), TextDirection.rtl);
      expect(
        firstStrongDirection('تقرير Final Draft 2026 v2.pdf'),
        TextDirection.rtl,
      );
      expect(firstStrongDirection('2026-11-14_001.jpg'), TextDirection.ltr);
      expect(firstStrongDirection('12345'), TextDirection.ltr);
    },
  );

  testWidgets(
    'progress row keeps "done / total" and units ordered in ar and fa',
    (tester) async {
      await tester.binding.setSurfaceSize(const Size(360, 740));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      addTearDown(tester.platformDispatcher.clearLocaleTestValue);

      for (final (locale, region) in rtlRegions) {
        tester.platformDispatcher.localeTestValue = region;
        final l = lookupAppLocalizations(locale);
        const done = 1200000000;
        const total = 3500000000;
        final doneStr = formatBytes(
          l,
          done,
          numberLocale: region.toLanguageTag(),
        );
        final totalStr = formatBytes(
          l,
          total,
          numberLocale: region.toLanguageTag(),
        );
        final doneNum = doneStr.split(' ').first;
        final totalNum = totalStr.split(' ').first;

        await tester.pumpWidget(
          _wrap(
            locale,
            const Padding(
              padding: EdgeInsets.all(24),
              child: TransferProgress(done: done, total: total),
            ),
          ),
        );

        final sizesFinder = find.textContaining('/', findRichText: true);
        final paragraph = tester.renderObject<RenderParagraph>(sizesFinder);
        final full = paragraph.text.toPlainText();

        final firstGbPos = full.indexOf('GB');
        final slashPos = full.indexOf('/');
        final secondGbPos = full.indexOf('GB', slashPos);

        final doneNumBox = _boxOf(paragraph, doneNum);
        final doneUnitBox = _boxOf(paragraph, 'GB', firstGbPos);
        final slashBox = _boxOf(paragraph, '/', slashPos);
        final totalNumBox = _boxOf(paragraph, totalNum, slashPos);
        final totalUnitBox = _boxOf(paragraph, 'GB', secondGbPos);

        // Inside each size, the number sits to the left of its Latin unit.
        expect(
          doneNumBox.right,
          lessThanOrEqualTo(doneUnitBox.left + 0.5),
          reason: '$region done number left of GB in "$full"',
        );
        expect(
          totalNumBox.right,
          lessThanOrEqualTo(totalUnitBox.left + 0.5),
          reason: '$region total number left of GB in "$full"',
        );
        // And "1.2 GB" sits to the left of "/", which sits to the left of "3.5 GB".
        expect(
          doneUnitBox.right,
          lessThanOrEqualTo(slashBox.left + 0.5),
          reason: '$region done size left of "/" in "$full"',
        );
        expect(
          slashBox.right,
          lessThanOrEqualTo(totalNumBox.left + 0.5),
          reason: '$region "/" left of total size in "$full"',
        );

        // In RTL, the percent sits on the right (start) and the sizes sit at
        // the left (end) edge of the row, not huddled next to the percent.
        final pctText = formatPercent(
          l,
          done,
          total,
          numberLocale: region.toLanguageTag(),
        );
        final pctRect = tester.getRect(find.text(pctText));
        final sizesLeftInWidget = doneNumBox.left;
        final sizesBoxOrigin = tester.getTopLeft(sizesFinder);
        final visualSizesLeft = sizesBoxOrigin.dx + sizesLeftInWidget;
        expect(
          visualSizesLeft,
          lessThan(60),
          reason:
              '$region sizes aligned to the left (end) edge of the 360px row',
        );
        expect(
          pctRect.left,
          greaterThan(visualSizesLeft + 80),
          reason: '$region percent on the right (start) side of the row',
        );
      }
    },
  );

  testWidgets(
    'byte sizes keep number before unit in file lists, buttons and offers',
    (tester) async {
      await tester.binding.setSurfaceSize(const Size(360, 740));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      addTearDown(tester.platformDispatcher.clearLocaleTestValue);

      for (final (locale, region) in rtlRegions) {
        tester.platformDispatcher.localeTestValue = region;
        final l = lookupAppLocalizations(locale);
        final oneB = formatBytes(l, 1, numberLocale: region.toLanguageTag());
        final tenB = formatBytes(l, 10, numberLocale: region.toLanguageTag());
        final oneNum = oneB.split(' ').first;
        final tenNum = tenB.split(' ').first;

        // 1. SendView with 10 files (1 B each -> row subtitle "1 B", button "10 B").
        final sendVm = SendViewModel(repository: FakeRepository());
        for (var i = 0; i < 10; i++) {
          sendVm.addFakeFileForTest('file_$i.txt');
        }
        await tester.pumpWidget(_wrap(locale, SendView(viewModel: sendVm)));
        await tester.pump();

        final rowSub = tester.renderObject<RenderParagraph>(
          find.textContaining(oneNum).first,
        );
        expect(
          _boxOf(rowSub, oneNum).right,
          lessThanOrEqualTo(_boxOf(rowSub, 'B').left + 0.5),
          reason: '$region file row subtitle "$oneB" keeps number before B',
        );

        final sendBtn = tester.renderObject<RenderParagraph>(
          find.descendant(
            of: find.byType(FilledButton),
            matching: find.textContaining(tenNum),
          ),
        );
        final btnText = sendBtn.text.toPlainText();
        final btnTenStart = btnText.lastIndexOf(tenNum);
        expect(
          _boxOf(sendBtn, tenNum, btnTenStart).right,
          lessThanOrEqualTo(_boxOf(sendBtn, 'B', btnTenStart).left + 0.5),
          reason: '$region send button "$btnText" keeps 10 before B',
        );

        // 2. SendView text mode button ("... · 10 B").
        sendVm
          ..setTextMode(true)
          ..setText('0123456789');
        await tester.pumpAndSettle();
        final textBtn = tester.renderObject<RenderParagraph>(
          find.descendant(
            of: find.byType(FilledButton),
            matching: find.textContaining(tenNum),
          ),
        );
        final textBtnPlain = textBtn.text.toPlainText();
        final textTenStart = textBtnPlain.lastIndexOf(tenNum);
        expect(
          _boxOf(textBtn, tenNum, textTenStart).right,
          lessThanOrEqualTo(_boxOf(textBtn, 'B', textTenStart).left + 0.5),
          reason: '$region text send button "$textBtnPlain" keeps 10 before B',
        );

        // 3. ReceiveView file offer summary and text offer size ("10 B").
        for (final isText in [false, true]) {
          final repo = FakeRepository();
          final rxVm = ReceiveViewModel(repository: repo)
            ..setInput('7-harbor-rocket');
          final c = rxVm.connect();
          repo.connected.complete(FakeOffer(isText: isText));
          await c;
          await tester.pumpWidget(_wrap(locale, ReceiveView(viewModel: rxVm)));
          await tester.pump();
          final offerSize = tester.renderObject<RenderParagraph>(
            find.textContaining(tenNum).first,
          );
          final plain = offerSize.text.toPlainText();
          final pos = plain.lastIndexOf(tenNum);
          expect(
            _boxOf(offerSize, tenNum, pos).right,
            lessThanOrEqualTo(_boxOf(offerSize, 'B', pos).left + 0.5),
            reason:
                '$region receive offer (isText=$isText) "$plain" keeps 10 before B',
          );
        }
      }
    },
  );

  testWidgets(
    'file names keep their own direction and start alignment in ar and fa',
    (tester) async {
      await tester.binding.setSurfaceSize(const Size(360, 740));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      const longLatin = '0 Rapport annuel 2026 (version finale).pdf';
      const shortLatin = '0 note.txt';
      const longArabic = '٠ تقرير سنوي ٢٠٢٦ (النسخة النهائية).pdf';

      for (final (locale, _) in rtlRegions) {
        // 1. SendView file list.
        final sendVm = SendViewModel(repository: FakeRepository())
          ..addFakeFileForTest(longLatin)
          ..addFakeFileForTest(shortLatin)
          ..addFakeFileForTest(longArabic);
        await tester.pumpWidget(_wrap(locale, SendView(viewModel: sendVm)));
        await tester.pump();

        void checkNames(String place, Finder leadingIconOfShort) {
          final longLatinPara = tester.renderObject<RenderParagraph>(
            find.text(longLatin),
          );
          expect(
            longLatinPara.textDirection,
            TextDirection.ltr,
            reason:
                '$locale $place: long Latin name uses LTR so ellipsis stays right',
          );
          expect(
            _boxOf(longLatinPara, '0').right,
            lessThanOrEqualTo(_boxOf(longLatinPara, 'Rapport').left + 0.5),
            reason: '$locale $place: leading "0" stays left of "Rapport"',
          );

          final shortLatinPara = tester.renderObject<RenderParagraph>(
            find.text(shortLatin),
          );
          expect(
            shortLatinPara.textDirection,
            TextDirection.ltr,
            reason: '$locale $place: short Latin name uses LTR',
          );
          expect(
            _boxOf(shortLatinPara, '0').right,
            lessThanOrEqualTo(_boxOf(shortLatinPara, 'note').left + 0.5),
            reason: '$locale $place: leading "0" stays left of "note"',
          );
          // Short LTR name still sits at the start (right) edge of the RTL row,
          // right beside the leading icon rather than on the far left.
          final iconRect = tester.getRect(leadingIconOfShort);
          final nameRect = tester.getRect(find.text(shortLatin));
          expect(
            iconRect.left - nameRect.right,
            inInclusiveRange(0, 28),
            reason:
                '$locale $place: short Latin name stays beside RTL start icon',
          );

          final arabicPara = tester.renderObject<RenderParagraph>(
            find.text(longArabic),
          );
          expect(
            arabicPara.textDirection,
            TextDirection.rtl,
            reason:
                '$locale $place: Arabic name uses RTL so ellipsis stays left',
          );
        }

        checkNames(
          'SendView',
          find
              .descendant(
                of: find.ancestor(
                  of: find.text(shortLatin),
                  matching: find.byType(ListTile),
                ),
                matching: find.byType(Icon),
              )
              .first,
        );

        // 2. ReceiveView incoming offer.
        final repo = FakeRepository();
        final rxVm = ReceiveViewModel(repository: repo)
          ..setInput('7-harbor-rocket');
        final connecting = rxVm.connect();
        repo.connected.complete(
          _NamedOffer([longLatin, shortLatin, longArabic]),
        );
        await connecting;
        await tester.pumpWidget(_wrap(locale, ReceiveView(viewModel: rxVm)));
        await tester.pump();

        checkNames(
          'ReceiveView offer',
          find
              .descendant(
                of: find.ancestor(
                  of: find.text(shortLatin),
                  matching: find.byType(Row),
                ),
                matching: find.byType(Icon),
              )
              .first,
        );

        // 3. ReceiveView finished saved files list.
        await rxVm.accept();
        repo.events.add(
          const TransferEvent(
            kind: EventKind.finished,
            words: [],
            done: 30,
            total: 30,
            summary: TransferSummary(
              filesOk: 3,
              filesFailed: 0,
              bytes: 30,
              saved: [
                '/storage/emulated/0/Download/$longLatin',
                '/storage/emulated/0/Download/$shortLatin',
                '/storage/emulated/0/Download/$longArabic',
              ],
              text: null,
            ),
            error: null,
          ),
        );
        await tester.pumpAndSettle();

        checkNames(
          'ReceiveView done',
          find
              .descendant(
                of: find.ancestor(
                  of: find.text(shortLatin),
                  matching: find.byType(ListTile),
                ),
                matching: find.byType(Icon),
              )
              .first,
        );
      }
    },
  );

  testWidgets(
    'short RTL file name stays beside leading icon in LTR and hidden overrides are revealed on SendView',
    (tester) async {
      await tester.binding.setSurfaceSize(const Size(360, 740));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      const shortArabic = '0 ملاحظة.txt';
      const spoofed = 'invoice_\u202Efdp.apk';
      const revealed = 'invoice_<U+202E>fdp.apk';

      final sendVm = SendViewModel(repository: FakeRepository())
        ..addFakeFileForTest(shortArabic)
        ..addFakeFileForTest(spoofed);
      await tester.pumpWidget(
        _wrap(const Locale('en'), SendView(viewModel: sendVm)),
      );
      await tester.pump();

      final arabicPara = tester.renderObject<RenderParagraph>(
        find.text(shortArabic),
      );
      expect(arabicPara.textDirection, TextDirection.rtl);
      final leadingIcon = find
          .descendant(
            of: find.ancestor(
              of: find.text(shortArabic),
              matching: find.byType(ListTile),
            ),
            matching: find.byType(Icon),
          )
          .first;
      expect(
        tester.getRect(find.text(shortArabic)).left -
            tester.getRect(leadingIcon).right,
        inInclusiveRange(0, 28),
      );
      expect(find.text(revealed), findsOneWidget);
      expect(find.text(spoofed), findsNothing);
    },
  );
}
