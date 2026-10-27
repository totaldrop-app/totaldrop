import 'package:material_ui/material_ui.dart';

/// TotalDrop's brand colours (the Aurora palette). The app follows the
/// system's colours when the system offers them (Material You on Android
/// 12+); these are the fallback and the colours of the logo itself, which
/// never changes.
abstract final class Brand {
  static const paper = Color(0xFFF4F2FB);
  static const ink = Color(0xFF22184F);
  static const accent = Color(0xFF6D4AE0);
  static const night = Color(0xFF140E33);
  static const glow = Color(0xFFA68BFF);
  static const mist = Color(0xFFE2DCF6);
  static const success = Color(0xFF0A7357);
  static const successDark = Color(0xFF7ED9B5);
}

const _display = 'BricolageGrotesque';
const _body = 'AtkinsonHyperlegible';

ColorScheme fallbackScheme(Brightness brightness) {
  final base = ColorScheme.fromSeed(seedColor: Brand.accent, brightness: brightness);
  return brightness == Brightness.light
      ? base.copyWith(primary: Brand.accent, surface: Brand.paper, onSurface: Brand.ink)
      : base.copyWith(primary: Brand.glow, onPrimary: Brand.night, surface: Brand.night);
}

TextTheme _text(TextTheme base) {
  // Display type is tracked tight only where it is large; titles keep air.
  TextStyle? display(TextStyle? s, FontWeight w, double tracking) => s?.copyWith(
    fontFamily: _display,
    fontWeight: w,
    letterSpacing: tracking,
    height: 1.12,
    fontVariations: [FontVariation('wght', w.value.toDouble())],
  );
  // Atkinson Hyperlegible is already wide: drop Material's extra tracking.
  TextStyle? body(TextStyle? s, [FontWeight? w]) => s?.copyWith(letterSpacing: 0.1, fontWeight: w);
  final t = base.apply(fontFamily: _body);
  return t.copyWith(
    displayLarge: display(t.displayLarge, FontWeight.w800, -1.0),
    displayMedium: display(t.displayMedium, FontWeight.w800, -0.8),
    displaySmall: display(t.displaySmall, FontWeight.w800, -0.6),
    headlineLarge: display(t.headlineLarge, FontWeight.w800, -0.4),
    headlineMedium: display(t.headlineMedium, FontWeight.w700, -0.2),
    headlineSmall: display(t.headlineSmall, FontWeight.w700, 0),
    titleLarge: display(t.titleLarge, FontWeight.w700, 0),
    titleMedium: body(t.titleMedium, FontWeight.w700),
    titleSmall: body(t.titleSmall, FontWeight.w700),
    bodyLarge: body(t.bodyLarge),
    bodyMedium: body(t.bodyMedium),
    bodySmall: body(t.bodySmall),
    labelLarge: body(t.labelLarge, FontWeight.w700),
  );
}

ThemeData buildTheme(ColorScheme scheme) {
  final base = ThemeData(colorScheme: scheme, useMaterial3: true);
  return base.copyWith(
    textTheme: _text(base.textTheme),
    appBarTheme: const AppBarTheme(centerTitle: false),
    cardTheme: const CardThemeData(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.all(Radius.circular(28))),
    ),
    filledButtonTheme: FilledButtonThemeData(
      style: FilledButton.styleFrom(
        minimumSize: const Size.fromHeight(56),
        textStyle: const TextStyle(fontFamily: _body, fontWeight: FontWeight.w700, fontSize: 16),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        minimumSize: const Size.fromHeight(56),
        textStyle: const TextStyle(fontFamily: _body, fontWeight: FontWeight.w700, fontSize: 16),
      ),
    ),
    inputDecorationTheme: const InputDecorationTheme(
      border: OutlineInputBorder(borderRadius: BorderRadius.all(Radius.circular(16))),
    ),
  );
}

/// Shared spacing.
abstract final class Gap {
  static const s = SizedBox(height: 8, width: 8);
  static const m = SizedBox(height: 16, width: 16);
  static const l = SizedBox(height: 24, width: 24);
  static const xl = SizedBox(height: 40, width: 40);
}
