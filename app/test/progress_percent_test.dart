// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The transfer's percent follows the device's region, like the sizes
// beside it, not the app's language (docs/kararlar/0008, "Dil ve bölge
// seçimi"; #483). The cases that set the two apart fail if the percent
// comes from the language.
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/ui/core/common.dart';

Future<void> _show(
  WidgetTester tester, {
  required Locale app,
  required Locale device,
  int done = 1,
  int total = 2,
}) async {
  tester.platformDispatcher.localeTestValue = device;
  addTearDown(tester.platformDispatcher.clearLocaleTestValue);
  await tester.pumpWidget(
    MaterialApp(
      locale: app,
      localizationsDelegates: const [
        AppLocalizations.delegate,
        ...GlobalMaterialLocalizations.delegates,
      ],
      supportedLocales: AppLocalizations.supportedLocales,
      home: Scaffold(body: TransferProgress(done: done, total: total)),
    ),
  );
}

void main() {
  for (final (app, device, percent) in [
    (const Locale('en'), const Locale('en', 'US'), '50%'),
    (const Locale('en'), const Locale('tr', 'TR'), '%50'),
    (const Locale('tr'), const Locale('en', 'US'), '50%'),
    (const Locale('en'), const Locale('de', 'DE'), '50\u00a0%'),
    (const Locale('ja'), const Locale('ja', 'JP'), '50%'),
    // Right to left: the region's own sign and marks, from CLDR.
    (const Locale('ar'), const Locale('ar'), '50\u200e%\u200e'),
    (const Locale('ar'), const Locale('ar', 'EG'), '\u0665\u0660\u066a\u061c'),
    (const Locale('fa'), const Locale('fa', 'IR'), '\u06f5\u06f0\u066a'),
  ]) {
    testWidgets('half reads $percent for $app on $device', (tester) async {
      await _show(tester, app: app, device: device);
      expect(find.text(percent), findsOneWidget);
    });
  }

  // Rounded up, 995 of 1000 bytes read "100%" with bytes still to come.
  testWidgets('almost done reads 99%, not 100%', (tester) async {
    const en = Locale('en', 'US');
    await _show(tester, app: en, device: en, done: 995, total: 1000);
    expect(find.text('99%'), findsOneWidget);
  });
}
