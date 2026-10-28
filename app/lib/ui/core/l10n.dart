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
    preferences: [for (final l in preferred ?? const <Locale>[]) l.toLanguageTag()],
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
bool hasDraftTranslation(BuildContext context) =>
    rust.hasDraftTranslation(tag: Localizations.localeOf(context).toLanguageTag());

/// A name from the other side with hidden direction and control characters
/// shown, not obeyed.
String revealControls(String name) => rust.revealControls(name: name);

/// The locale for numbers: the device's region (its first locale, so a
/// device set to en-DE writes "1,5 MB"), or the app language when intl
/// does not know the device's locale.
String _numberLocale(AppLocalizations l) {
  final device = WidgetsBinding.instance.platformDispatcher.locale.toLanguageTag();
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
