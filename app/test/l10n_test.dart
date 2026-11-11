// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/ui/core/l10n.dart';

// Sizes must read the same as in the Rust programs (totaldrop-l10n's
// `sizes` test); plurals and counts follow Flutter's generated code.
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final en = lookupAppLocalizations(const Locale('en'));
  final tr = lookupAppLocalizations(const Locale('tr'));

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
}
