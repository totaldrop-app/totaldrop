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
}
