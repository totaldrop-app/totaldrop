// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/ui/core/theme.dart';
import 'package:totaldrop/ui/home/home_view.dart';
import 'package:totaldrop/ui/receive/receive_view.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

import 'fakes.dart';

/// Icon buttons follow Material 3 Expressive (ADR 0011, #257): round at
/// rest and a rounded square while pressed, as on Android 16's own
/// interface on Pixel. Size, tap target and label stay those of Material 3.
/// Each of the app's icon buttons is pressed where it really sits: the
/// app bar, the code field's suffix and a file row's trailing slot. The
/// last two rebuild the icon button theme in material_ui 1.8.0.
void main() {
  Widget app(Widget home, {bool reduced = false, Brightness? brightness}) =>
      MediaQuery(
        data: MediaQueryData(disableAnimations: reduced),
        child: MaterialApp(
          theme: buildTheme(fallbackScheme(brightness ?? Brightness.light)),
          localizationsDelegates: const [
            AppLocalizations.delegate,
            ...GlobalMaterialLocalizations.delegates,
          ],
          supportedLocales: AppLocalizations.supportedLocales,
          builder: (context, child) => SystemMotion(child: child!),
          home: home,
        ),
      );

  final views = <String, Widget Function()>{
    'Settings': () =>
        HomeView(onSend: () {}, onReceive: () {}, onSettings: () {}),
    'Paste': () =>
        ReceiveView(viewModel: ReceiveViewModel(repository: FakeRepository())),
    'Remove': () => SendView(
      viewModel: SendViewModel(repository: FakeRepository())
        ..addFakeFileForTest('trip.jpg'),
    ),
  };

  for (final MapEntry(key: label, value: view) in views.entries) {
    for (final brightness in Brightness.values) {
      testWidgets('$label morphs when pressed ($brightness)', (tester) async {
        await tester.pumpWidget(app(view(), brightness: brightness));
        await tester.pumpAndSettle();
        final button = find.ancestor(
          of: find.byTooltip(label),
          matching: find.byType(IconButton),
        );

        expect(variantOf(tester, button), StyleVariant.material3Expressive);
        expect(tester.getSize(button).width, greaterThanOrEqualTo(48));
        expect(tester.getSize(button).height, greaterThanOrEqualTo(48));
        expect(shapeOf(tester, button), isA<StadiumBorder>());

        // Held for less than a long press, which would show the tooltip
        // and end the pressed state.
        final gesture = await tester.startGesture(tester.getCenter(button));
        await tester.pump(const Duration(milliseconds: 200));
        expect(shapeOf(tester, button), pressedShape);

        await gesture.cancel(); // No tap: Remove would take the row away.
        await tester.pumpAndSettle();
        expect(shapeOf(tester, button), isA<StadiumBorder>());
      });
    }

    testWidgets('$label changes shape at once with Remove animations', (
      tester,
    ) async {
      await tester.pumpWidget(app(view(), reduced: true));
      await tester.pumpAndSettle();
      final button = find.ancestor(
        of: find.byTooltip(label),
        matching: find.byType(IconButton),
      );
      expect(variantOf(tester, button), StyleVariant.material3Expressive);
      expect(materialOf(tester, button).animationDuration, Duration.zero);

      final gesture = await tester.startGesture(tester.getCenter(button));
      await tester.pump(const Duration(milliseconds: 200));
      // The shape has already arrived; there is no frame in between.
      expect(shapeOf(tester, button), pressedShape);
      await gesture.cancel(); // No tap: Remove would take the row away.
      await tester.pump();
      expect(shapeOf(tester, button), isA<StadiumBorder>());
    });
  }

  testWidgets('with animations on, the morph takes the theme duration', (
    tester,
  ) async {
    await tester.pumpWidget(app(views['Settings']!()));
    final button = find.ancestor(
      of: find.byTooltip('Settings'),
      matching: find.byType(IconButton),
    );
    expect(materialOf(tester, button).animationDuration, kThemeChangeDuration);
  });

  test('the theme opts icon buttons in; nothing else has a variant', () {
    // In material_ui 1.8.0 only IconButtonThemeData takes a StyleVariant,
    // so the rest of the app stays on Material 3 (ADR 0011).
    final theme = buildTheme(fallbackScheme(Brightness.light));
    expect(theme.iconButtonTheme.variant, StyleVariant.material3Expressive);
  });
}

const pressedShape = RoundedRectangleBorder(
  borderRadius: BorderRadius.all(Radius.circular(8)),
);

Material materialOf(WidgetTester tester, Finder button) =>
    tester.widget<Material>(
      find.descendant(of: button, matching: find.byType(Material)).first,
    );

ShapeBorder? shapeOf(WidgetTester tester, Finder button) =>
    materialOf(tester, button).shape;

StyleVariant? variantOf(WidgetTester tester, Finder button) =>
    IconButtonTheme.of(tester.element(button)).variant;
