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
  static const muted = Color(0xFF463D72);
  static const night2 = Color(0xFF1E1648);
  static const glowMuted = Color(0xFFB9B1E0);
  static const ruleDark = Color(0xFF2E2566);
}

const _display = 'BricolageGrotesque';
const _body = 'AtkinsonHyperlegible';

/// The Aurora palette as a Material scheme, for when the system offers no
/// colours of its own. Values are the design system's tokens: `bg`,
/// `bg-raised`, `rule`, `outline`, `ink`, `ink-muted`, `accent`, `on-accent`.
ColorScheme fallbackScheme(Brightness brightness) {
  final base = ColorScheme.fromSeed(
    seedColor: Brand.accent,
    brightness: brightness,
  );
  return brightness == Brightness.light
      ? base.copyWith(
          primary: Brand.accent,
          onPrimary: const Color(0xFFFFFFFF),
          surface: Brand.paper,
          onSurface: Brand.ink,
          onSurfaceVariant: Brand.muted,
          surfaceContainerLowest: Brand.paper,
          surfaceContainerLow: Brand.paper,
          surfaceContainer: Brand.mist,
          surfaceContainerHigh: Brand.mist,
          surfaceContainerHighest: Brand.mist,
          secondaryContainer: Brand.mist,
          onSecondaryContainer: Brand.ink,
          outline: Brand.muted,
          outlineVariant: Brand.mist,
        )
      : base.copyWith(
          primary: Brand.glow,
          onPrimary: Brand.night,
          surface: Brand.night,
          onSurface: Brand.paper,
          onSurfaceVariant: Brand.glowMuted,
          surfaceContainerLowest: Brand.night,
          surfaceContainerLow: Brand.night,
          surfaceContainer: Brand.night2,
          surfaceContainerHigh: Brand.night2,
          surfaceContainerHighest: Brand.night2,
          secondaryContainer: Brand.night2,
          onSecondaryContainer: Brand.paper,
          outline: Brand.glowMuted,
          outlineVariant: Brand.ruleDark,
        );
}

TextTheme _text(TextTheme base) {
  // Display type is tracked tight only where it is large; titles keep air.
  TextStyle? display(TextStyle? s, FontWeight w, double tracking) =>
      s?.copyWith(
        fontFamily: _display,
        fontWeight: w,
        letterSpacing: tracking,
        height: 1.12,
        fontVariations: [FontVariation('wght', w.value.toDouble())],
      );
  // Atkinson Hyperlegible is already wide: drop Material's extra tracking.
  TextStyle? body(TextStyle? s, [FontWeight? w]) =>
      s?.copyWith(letterSpacing: 0.1, fontWeight: w);
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

/// Rams: surfaces are told apart by colour and a hairline, never by shadow.
/// Every elevation is 0 and no surface takes the primary tint.
ThemeData buildTheme(ColorScheme scheme) {
  final base = ThemeData(colorScheme: scheme, useMaterial3: true);
  return base.copyWith(
    textTheme: _text(base.textTheme),
    scaffoldBackgroundColor: scheme.surface,
    appBarTheme: AppBarTheme(
      centerTitle: false,
      elevation: 0,
      scrolledUnderElevation: 0,
      backgroundColor: scheme.surface,
      surfaceTintColor: Colors.transparent,
    ),
    cardTheme: const CardThemeData(
      elevation: 0,
      margin: EdgeInsets.zero,
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.all(Radius.circular(28)),
      ),
    ),
    dialogTheme: DialogThemeData(
      elevation: 0,
      backgroundColor: scheme.surface,
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(
        borderRadius: const BorderRadius.all(Radius.circular(28)),
        side: BorderSide(color: scheme.outlineVariant),
      ),
    ),
    snackBarTheme: SnackBarThemeData(
      elevation: 0,
      behavior: SnackBarBehavior.floating,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.all(Radius.circular(16)),
      ),
      backgroundColor: scheme.onSurface,
      contentTextStyle: TextStyle(
        fontFamily: _body,
        color: scheme.surface,
        fontSize: 14,
      ),
    ),
    dividerTheme: DividerThemeData(
      color: scheme.outlineVariant,
      thickness: 1,
      space: 1,
    ),
    listTileTheme: const ListTileThemeData(contentPadding: EdgeInsets.zero),
    progressIndicatorTheme: ProgressIndicatorThemeData(
      color: scheme.primary,
      linearTrackColor: scheme.surfaceContainerHigh,
    ),
    switchTheme: SwitchThemeData(
      trackOutlineColor: WidgetStateProperty.resolveWith(
        (states) => states.contains(WidgetState.selected)
            ? Colors.transparent
            : scheme.outline,
      ),
    ),
    filledButtonTheme: FilledButtonThemeData(
      style: FilledButton.styleFrom(
        minimumSize: const Size(64, 56),
        textStyle: const TextStyle(
          fontFamily: _body,
          fontWeight: FontWeight.w700,
          fontSize: 16,
        ),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        minimumSize: const Size(64, 56),
        side: BorderSide(color: scheme.outline),
        textStyle: const TextStyle(
          fontFamily: _body,
          fontWeight: FontWeight.w700,
          fontSize: 16,
        ),
      ),
    ),
    inputDecorationTheme: InputDecorationTheme(
      border: const OutlineInputBorder(
        borderRadius: BorderRadius.all(Radius.circular(16)),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: const BorderRadius.all(Radius.circular(16)),
        borderSide: BorderSide(color: scheme.outline),
      ),
    ),
    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(
        textStyle: const TextStyle(
          fontFamily: _body,
          fontWeight: FontWeight.w700,
          fontSize: 14,
        ),
      ),
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
