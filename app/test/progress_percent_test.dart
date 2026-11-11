// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The transfer's percent follows the region, like the sizes beside it
// (docs/tasarim.md, "Dil ve bölge"; #483): Turkish writes the sign
// first, German puts a space before it.
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/ui/core/common.dart';

Future<void> _show(WidgetTester tester, Locale locale) async {
  tester.platformDispatcher.localeTestValue = locale;
  addTearDown(tester.platformDispatcher.clearLocaleTestValue);
  await tester.pumpWidget(
    MaterialApp(
      locale: locale,
      localizationsDelegates: const [
        AppLocalizations.delegate,
        ...GlobalMaterialLocalizations.delegates,
      ],
      supportedLocales: AppLocalizations.supportedLocales,
      home: const Scaffold(body: TransferProgress(done: 1, total: 2)),
    ),
  );
}

void main() {
  for (final (locale, percent) in [
    (const Locale('en', 'US'), '50%'),
    (const Locale('tr', 'TR'), '%50'),
    (const Locale('de', 'DE'), '50 %'),
  ]) {
    testWidgets('half done reads $percent in $locale', (tester) async {
      await _show(tester, locale);
      expect(find.text(percent), findsOneWidget);
    });
  }
}
