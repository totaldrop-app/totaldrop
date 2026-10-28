import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/ui/core/l10n.dart';

// Sizes must read the same as in the Rust programs (totaldrop-l10n's
// `sizes` test); plurals and counts follow Flutter's generated code.
void main() {
  final en = lookupAppLocalizations(const Locale('en'));
  final tr = lookupAppLocalizations(const Locale('tr'));

  test('sizes match the Rust programs', () {
    expect(formatBytes(en, 0), '0 B');
    expect(formatBytes(en, 999), '999 B');
    expect(formatBytes(en, 1000), '1.0 kB');
    expect(formatBytes(en, 1549999), '1.5 MB');
    expect(formatBytes(en, 9960000), '10 MB');
    expect(formatBytes(en, 12450000), '12 MB');
    expect(formatBytes(en, 12500000), '13 MB');
    expect(formatBytes(en, 999950), '1.0 MB');
    expect(formatBytes(en, 999999999), '1.0 GB');
    expect(formatBytes(tr, 1500000), '1,5 MB');
  });

  test('counts are grouped and plurals follow the language', () {
    expect(en.itemsAndSize(1, '2 MB'), '1 item · 2 MB');
    expect(en.itemsAndSize(1234, '2 MB'), '1,234 items · 2 MB');
    expect(tr.itemsAndSize(1234, '2 MB'), '1.234 öğe · 2 MB');
  });
}
