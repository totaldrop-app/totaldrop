// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/ui/core/theme.dart';

/// Icon buttons follow Material 3 Expressive (ADR 0011, #257): round at
/// rest and a rounded square while pressed, as on Android 16's own
/// interface. Size, tap target and label stay those of Material 3.
void main() {
  for (final brightness in Brightness.values) {
    testWidgets('an icon button morphs when pressed ($brightness)', (
      tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: buildTheme(fallbackScheme(brightness)),
          home: Scaffold(
            body: Center(
              child: IconButton(
                tooltip: 'Settings',
                icon: const Icon(Icons.settings_outlined),
                onPressed: () {},
              ),
            ),
          ),
        ),
      );

      final button = find.byType(IconButton);
      expect(tester.getSize(button).width, greaterThanOrEqualTo(48));
      expect(tester.getSize(button).height, greaterThanOrEqualTo(48));
      expect(find.byTooltip('Settings'), findsOneWidget);
      expect(shapeOf(tester, button), isA<StadiumBorder>());

      // Held for less than a long press, which would show the tooltip
      // and end the pressed state.
      final gesture = await tester.startGesture(tester.getCenter(button));
      await tester.pump(const Duration(milliseconds: 200));
      final pressed = shapeOf(tester, button);
      expect(pressed, isA<RoundedRectangleBorder>());
      expect(
        (pressed as RoundedRectangleBorder).borderRadius,
        const BorderRadius.all(Radius.circular(8)),
      );

      await gesture.up();
      await tester.pumpAndSettle();
      expect(shapeOf(tester, button), isA<StadiumBorder>());
    });
  }

  for (final reduced in [false, true]) {
    testWidgets('the morph follows Remove animations ($reduced)', (
      tester,
    ) async {
      await tester.pumpWidget(
        MediaQuery(
          data: MediaQueryData(disableAnimations: reduced),
          child: MaterialApp(
            theme: buildTheme(fallbackScheme(Brightness.light)),
            builder: (context, child) => SystemMotion(child: child!),
            home: Scaffold(
              body: Center(
                child: IconButton(
                  tooltip: 'Remove',
                  icon: const Icon(Icons.close),
                  onPressed: () {},
                ),
              ),
            ),
          ),
        ),
      );

      final material = tester.widget<Material>(
        find
            .descendant(
              of: find.byType(IconButton),
              matching: find.byType(Material),
            )
            .first,
      );
      expect(
        material.animationDuration,
        reduced ? Duration.zero : kThemeChangeDuration,
      );
    });
  }

  test('only icon buttons take the Expressive style, not the whole app', () {
    final theme = buildTheme(fallbackScheme(Brightness.light));
    expect(theme.iconButtonTheme.variant, StyleVariant.material3Expressive);
  });
}

ShapeBorder? shapeOf(WidgetTester tester, Finder button) => tester
    .widget<Material>(
      find.descendant(of: button, matching: find.byType(Material)).first,
    )
    .shape;
