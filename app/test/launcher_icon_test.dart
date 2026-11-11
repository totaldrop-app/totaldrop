// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'dart:io';
import 'dart:math';
import 'dart:ui' as ui;

import 'package:flutter_test/flutter_test.dart';

// The adaptive launcher icon's layers (docs/tasarim.md, "App icon";
// packaging/render-icons.sh). A layer is 108 dp; launchers show the middle
// 72 dp through a mask, and the tightest common one is the 72 dp circle
// (36 dp radius). The mark's corner, its point furthest from the centre,
// keeps clear of that circle so it never looks as if it touches the mask's
// edge (#261).
void main() {
  const densities = ['mdpi', 'hdpi', 'xhdpi', 'xxhdpi', 'xxxhdpi'];

  for (final density in densities) {
    for (final layer in ['foreground', 'monochrome']) {
      test('$density $layer: the corner stays 10 dp inside the mask', () async {
        final reach = await _reach(
          'android/app/src/main/res/mipmap-$density/ic_launcher_$layer.png',
        );
        // 0.44 of the mark's 58-unit reach is 25.5 dp; antialiasing and
        // pixel rounding move the measured edge by under a pixel.
        expect(
          reach,
          lessThanOrEqualTo(26),
          reason: 'corner too near the mask',
        );
        expect(
          reach,
          greaterThanOrEqualTo(24),
          reason: 'mark smaller than 0.44',
        );
      });
    }
  }
}

/// How far, in dp of the 108 dp layer, the furthest opaque pixel lies from
/// the layer's centre.
Future<double> _reach(String path) async {
  final codec = await ui.instantiateImageCodec(File(path).readAsBytesSync());
  final image = (await codec.getNextFrame()).image;
  final data = (await image.toByteData(
    format: ui.ImageByteFormat.rawStraightRgba,
  ))!;
  final w = image.width, h = image.height;
  final centre = w / 2;
  var furthest = 0.0;
  for (var y = 0; y < h; y++) {
    for (var x = 0; x < w; x++) {
      if (data.getUint8((y * w + x) * 4 + 3) < 128) continue;
      final d = sqrt(pow(x + 0.5 - centre, 2) + pow(y + 0.5 - centre, 2));
      if (d > furthest) furthest = d;
    }
  }
  image.dispose();
  codec.dispose();
  return furthest / w * 108;
}
