import 'dart:io';

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

  test('a pasted message gives a code typed with Arabic-Indic digits', () {
    expect(
      codeFromPaste('the code is \u0667-abandon-ability, thanks'),
      '\u0667-abandon-ability',
    );
    expect(
      codeFromPaste('kod: \u06F1\u06F2 zoo zone able.'),
      '\u06F1\u06F2 zoo zone able',
    );
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

  test('every invisible format character is stripped, and nothing else', () {
    for (final (start, end) in invisibleFormat) {
      for (var c = start; c <= end; c++) {
        final mark = String.fromCharCode(c);
        expect(
          stripInvisibleFormat('${mark}7-abandon$mark-ability$mark'),
          '7-abandon-ability',
          reason: c.toRadixString(16),
        );
      }
    }
    // Neighbours of the ranges, and a Cyrillic look-alike, stay.
    final kept = String.fromCharCodes([
      0x00AC,
      0x00AE,
      0x061B,
      0x061D,
      0x200A,
      0x2010,
      0x2029,
      0x202F,
      0x205F,
      0x2061,
      0x2065,
      0x206A,
      0xFEFE,
      0x0430,
      0x20,
      0x37,
    ]);
    expect(stripInvisibleFormat(kept), kept);
  });

  test('the list matches the code parser in totaldrop-proto', () {
    final rust = File('../crates/totaldrop-proto/src/lib.rs')
        .readAsStringSync();
    const declaration = 'pub const INVISIBLE_FORMAT: &[(u32, u32)] = &[';
    final start = rust.indexOf(declaration);
    expect(start, isNot(-1), reason: 'declaration not found');
    final body = rust.substring(
      start + declaration.length,
      rust.indexOf('];', start),
    );
    // Every entry must be read: an entry the pattern misses (another
    // spelling, a constant) fails the count instead of passing unseen.
    final entries = '('.allMatches(body).length;
    final hex = RegExp(
      r'\(\s*0x([0-9a-f_]+)\s*,\s*0x([0-9a-f_]+)\s*\)',
      caseSensitive: false,
    );
    int value(String digits) =>
        int.parse(digits.replaceAll('_', ''), radix: 16);
    final ranges = [
      for (final m in hex.allMatches(body))
        (value(m.group(1)!), value(m.group(2)!)),
    ];
    expect(entries, greaterThan(0));
    expect(ranges, hasLength(entries));
    expect(ranges, invisibleFormat);
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
