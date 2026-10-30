import 'package:flutter/semantics.dart';
import 'package:intl/intl.dart' as intl;
import 'package:material_ui/material_ui.dart';

import '../../l10n/app_localizations.dart';
import '../../src/rust/api/l10n.dart' as rust;

/// The language for the device's preferences, chosen the same way as the
/// Linux app and the command line (ADR 0008): CLDR fallback chains and
/// likely scripts, so `zh-HK` gets Traditional Chinese and `pt-AO` gets
/// `pt-PT`. Flutter's own choice would match on the language code alone.
Locale resolveLocale(List<Locale>? preferred, Iterable<Locale> supported) {
  final tags = {for (final l in supported) l.toLanguageTag(): l};
  final chosen = rust.negotiateLocale(
    preferences: [
      for (final l in preferred ?? const <Locale>[]) l.toLanguageTag(),
    ],
    available: tags.keys.toList(),
  );
  return tags[chosen] ?? const Locale('en');
}

/// The app's strings, except that a safety-critical string (verification,
/// a wrong code, accept or decline) comes in English while its translation
/// in this language is an unreviewed draft.
AppLocalizations safeL10n(BuildContext context, String key) {
  final tag = Localizations.localeOf(context).toLanguageTag();
  if (rust.needsEnglish(tag: tag, key: key)) {
    return lookupAppLocalizations(const Locale('en'));
  }
  return AppLocalizations.of(context);
}

/// Whether some text in the current language is an unreviewed draft.
bool hasDraftTranslation(BuildContext context) => rust.hasDraftTranslation(
  tag: Localizations.localeOf(context).toLanguageTag(),
);

/// A name from the other side with hidden direction and control characters
/// shown, not obeyed.
String revealControls(String name) => rust.revealControls(name: name);

/// The locale for numbers: the device's region (its first locale, so a
/// device set to en-DE writes "1,5 MB"), or the app language when intl
/// does not know the device's locale.
String _numberLocale(AppLocalizations l) {
  final device = WidgetsBinding.instance.platformDispatcher.locale
      .toLanguageTag();
  for (final candidate in [device, l.localeName]) {
    try {
      intl.NumberFormat.decimalPattern(candidate);
      return candidate;
    } catch (_) {
      // Not a locale intl has data for: try the next one.
    }
  }
  return 'en';
}

/// "1.5 MB" in the region's format: one decimal below 10 of a unit, whole
/// numbers above, the next unit once the rounded value reaches 1000. The
/// same rules as the Rust programs. `numberLocale` is for tests.
String formatBytes(AppLocalizations l, int bytes, {String? numberLocale}) {
  final locale = numberLocale ?? _numberLocale(l);
  if (bytes < 1000) {
    return l.sizeBytes(intl.NumberFormat.decimalPattern(locale).format(bytes));
  }
  final units = [l.sizeKB, l.sizeMB, l.sizeGB, l.sizeTB, l.sizePB];
  var divisor = BigInt.from(1000);
  final n = BigInt.from(bytes);
  for (var unit = 0; unit < units.length; unit++) {
    final half = divisor ~/ BigInt.two;
    final tenths = (n * BigInt.from(10) + half) ~/ divisor;
    final whole = (n + half) ~/ divisor;
    if (whole < BigInt.from(1000) || unit == units.length - 1) {
      final String value;
      if (tenths < BigInt.from(100)) {
        final f = intl.NumberFormat.decimalPattern(locale)
          ..minimumFractionDigits = 1
          ..maximumFractionDigits = 1;
        value = f.format(tenths.toInt() / 10);
      } else {
        value = intl.NumberFormat.decimalPattern(locale).format(whole.toInt());
      }
      return units[unit](value);
    }
    divisor *= BigInt.from(1000);
  }
  return l.sizePB(bytes.toString());
}

/// A value from outside the translation (a version, a folder) placed in a
/// sentence. Isolated, it keeps its own direction in a right-to-left
/// language: "0.2.0-beta.2" would otherwise show as "beta.2-0.2.0".
/// Translations never contain these characters; only the code adds them.
String isolate(String value) => '\u2068$value\u2069';

/// Invisible format characters a code can pick up when it is copied, as
/// inclusive ranges: the soft hyphen, the Arabic letter mark, the zero-width
/// space, non-joiner and joiner, the left-to-right and right-to-left marks,
/// the embeddings and overrides with their pop, the word joiner, the
/// isolates with theirs, and the byte order mark. The same list as
/// `INVISIBLE_FORMAT` in crates/totaldrop-proto/src/lib.rs, which the code
/// parser strips; test/paste_code_test.dart checks that they match.
const invisibleFormat = <(int, int)>[
  (0x00AD, 0x00AD),
  (0x061C, 0x061C),
  (0x200B, 0x200F),
  (0x202A, 0x202E),
  (0x2060, 0x2060),
  (0x2066, 0x2069),
  (0xFEFF, 0xFEFF),
];

final _invisibleFormat = RegExp(
  '[${invisibleFormat.map((r) => '${_char(r.$1)}-${_char(r.$2)}').join()}]',
);

String _char(int code) => String.fromCharCode(code);

/// [value] without [invisibleFormat] characters: the reverse of [isolate],
/// for text that comes back in, such as a pasted message holding a code.
String stripInvisibleFormat(String value) =>
    value.replaceAll(_invisibleFormat, '');

/// Text that must read left to right in every language (the transfer code,
/// the verification words), spoken by screen readers with an English voice.
class LtrEnglish extends StatelessWidget {
  const LtrEnglish(this.text, {super.key, this.style, this.textAlign});

  final String text;
  final TextStyle? style;
  final TextAlign? textAlign;

  @override
  Widget build(BuildContext context) {
    return Directionality(
      textDirection: TextDirection.ltr,
      child: Semantics(
        attributedLabel: AttributedString(
          text,
          attributes: [
            LocaleStringAttribute(
              range: TextRange(start: 0, end: text.length),
              locale: const Locale('en'),
            ),
          ],
        ),
        excludeSemantics: true,
        child: Text(text, style: style, textAlign: textAlign),
      ),
    );
  }
}
