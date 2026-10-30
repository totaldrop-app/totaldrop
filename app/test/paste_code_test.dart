import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/ui/core/l10n.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';

void main() {
  test('a pasted message gives its code', () {
    expect(
      codeFromPaste('Get my files with TotalDrop. Code: 7-abandon-ability'),
      '7-abandon-ability',
    );
    expect(codeFromPaste('  12 zoo zone able  '), '12 zoo zone able');
    expect(codeFromPaste('not a code'), 'not a code');
  });

  test('direction marks around the code are dropped', () {
    // The share text isolates the code (U+2068 … U+2069), so a right-to-left
    // sentence keeps it in order. Pasted back, the marks must not reach the
    // field, whether the whole message or only the code was copied.
    expect(
      codeFromPaste(
        'استلم ملفاتي عبر TotalDrop. الرمز: \u20687-abandon-ability\u2069',
      ),
      '7-abandon-ability',
    );
    expect(codeFromPaste('\u20687-abandon-ability\u2069'), '7-abandon-ability');
    expect(
      codeFromPaste('\u200F\u20667 abandon\u2069 ability\u061C'),
      '7 abandon ability',
    );
    expect(
      codeFromPaste('\u202A7-abandon-\u202Eability\u202C'),
      '7-abandon-ability',
    );
    expect(codeFromPaste('\u2068half\u2069'), 'half');
    expect(codeFromPaste('\u2068\u2069'), '');
  });

  test('every bidirectional control is stripped', () {
    const controls = [
      0x061C,
      0x200E,
      0x200F,
      0x202A,
      0x202B,
      0x202C,
      0x202D,
      0x202E,
      0x2066,
      0x2067,
      0x2068,
      0x2069,
    ];
    for (final c in controls) {
      final mark = String.fromCharCode(c);
      expect(
        stripBidiControls('${mark}7-abandon$mark-ability$mark'),
        '7-abandon-ability',
        reason: c.toRadixString(16),
      );
    }
  });

  test('the share text gives its code back in every language', () {
    for (final locale in AppLocalizations.supportedLocales) {
      final l = lookupAppLocalizations(locale);
      for (final code in ['7-abandon-ability', '12-zoo-zone-able']) {
        expect(
          codeFromPaste(l.sendShareText(isolate(code))),
          code,
          reason: '$locale',
        );
      }
    }
  });
}
