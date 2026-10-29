import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:totaldrop/ui/core/mark.dart';

// The mark's shape and timing must match the SVG, the website and the
// Linux app (docs/tasarim.md, "İşaret" and "Hareket").
void main() {
  test('the loop lasts 3.2 s: four quarter turns of 0.6 s, each with a 0.2 s rest', () {
    expect(MarkMotion.loop, const Duration(milliseconds: 3200));
    expect(MarkMotion.turning * MarkMotion.loop.inMilliseconds / 4, 600);
  });

  test('it rests on each quarter and ends where it started', () {
    expect(MarkMotion.turn(0), 0);
    for (var q = 0; q < 4; q++) {
      // Turning ends 0.6 s into each 0.8 s quarter; the rest holds still.
      expect(MarkMotion.turn((q + 0.75) / 4), closeTo((q + 1) / 4, 1e-9));
      expect(MarkMotion.turn((q + 0.9) / 4), closeTo((q + 1) / 4, 1e-9));
    }
    expect(MarkMotion.turn(1), closeTo(1, 1e-9));
  });

  test('it only turns forward', () {
    var last = 0.0;
    for (var i = 0; i <= 1000; i++) {
      final at = MarkMotion.turn(i / 1000);
      expect(at, greaterThanOrEqualTo(last - 1e-12));
      last = at;
    }
  });

  test('the shape is the SVG: a circle of 42 around (55, 57) with its corner at top right', () {
    final b = MarkGeometry.shape().getBounds();
    expect(b.left, closeTo(13, 1e-6));
    expect(b.top, closeTo(15, 1e-6));
    expect(b.right, closeTo(97, 1e-6));
    expect(b.bottom, closeTo(99, 1e-6));
    final shape = MarkGeometry.shape();
    // Filled at the corner, empty just outside the circle at the other three.
    expect(shape.contains(const Offset(96, 16)), isTrue);
    expect(shape.contains(const Offset(15, 17)), isFalse);
    expect(shape.contains(const Offset(15, 97)), isFalse);
    expect(shape.contains(const Offset(95, 97)), isFalse);
    expect(shape.contains(MarkGeometry.centre), isTrue);
  });

  test('the turn is eased: halfway through a quarter it has turned half of it', () {
    // cubic-bezier(0.65, 0, 0.35, 1) is symmetric, so it passes (0.5, 0.5).
    expect(MarkMotion.turn(0.375 / 4), closeTo(0.125, 1e-3));
    // Slow at the start: a tenth of the time turns far less than a tenth.
    expect(MarkMotion.turn(0.075 / 4), lessThan(0.025 / 4));
  });

  // Every copy of the mark uses the same path and timing (docs/tasarim.md).
  const path = 'M55 15H97V57A42 42 0 1 1 55 15Z';
  test('the SVGs, the icon script and the site draw the same path', () {
    for (final f in [
      'assets/logo.svg',
      '../site/public/favicon.svg',
      '../site/public/index.html',
      '../packaging/linux/app.totaldrop.TotalDrop.svg',
      '../packaging/linux/app.totaldrop.TotalDrop-symbolic.svg',
      '../packaging/render-icons.sh',
    ]) {
      expect(File(f).readAsStringSync(), contains(path), reason: f);
    }
  });

  test('the Linux app and the site turn the same way, on the same timing', () {
    final qml = File('../linux/qml/Mark.qml').readAsStringSync().replaceAll(RegExp(r'\s+'), ' ');
    expect(qml, contains('origin.x: 55 * root.unit; origin.y: 57 * root.unit'));
    expect(qml, contains('startX: 55 * root.unit; startY: 15 * root.unit'));
    expect(qml, contains('PathLine { x: 97 * root.unit; y: 15 * root.unit }'));
    expect(qml, contains('PathLine { x: 97 * root.unit; y: 57 * root.unit }'));
    expect(qml, contains('duration: 600'));
    expect(RegExp(r'PauseAnimation \{ duration: 200 \}').allMatches(qml).length, 4);
    expect(qml, contains('[0.65, 0, 0.35, 1, 1, 1]'));
    final css = File('../site/public/styles.css').readAsStringSync();
    expect(css, contains('transform-origin: 55px 57px'));
    expect(css, contains('animation: mk-turn 3.2s cubic-bezier(0.65, 0, 0.35, 1) infinite'));
    for (final stop in ['18.75%, 25%', '43.75%, 50%', '68.75%, 75%', '93.75%, 100%']) {
      expect(css, contains(stop));
    }
  });

  testWidgets('it turns while sending', (tester) async {
    await tester.pumpWidget(
      const Directionality(
        textDirection: TextDirection.ltr,
        child: Center(child: TotalDropMark(sending: true)),
      ),
    );
    await tester.pump(const Duration(milliseconds: 300));
    expect(tester.hasRunningAnimations, isTrue);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('it stays still when animations are off', (tester) async {
    await tester.pumpWidget(
      const MediaQuery(
        data: MediaQueryData(disableAnimations: true),
        child: Directionality(
          textDirection: TextDirection.ltr,
          child: Center(child: TotalDropMark(sending: true)),
        ),
      ),
    );
    await tester.pump(const Duration(seconds: 1));
    expect(tester.hasRunningAnimations, isFalse);
  });
}
