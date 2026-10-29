import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/ui/core/mark.dart';

// The drop's timing must match the website and the Linux app
// (docs/tasarim.md, "Hareket").
void main() {
  test('the loop lasts 4.8 s', () {
    expect(MarkMotion.loop, const Duration(milliseconds: 4800));
  });

  test('the drop rests at each device and ends where it started', () {
    expect(MarkMotion.along(0), 0);
    expect(MarkMotion.along(0.125), closeTo(1 / 6, 1e-9));
    expect(MarkMotion.along(0.16), closeTo(1 / 6, 1e-9));
    expect(MarkMotion.along(0.45), closeTo(0.5, 1e-9));
    expect(MarkMotion.along(0.5), closeTo(0.5, 1e-9));
    expect(MarkMotion.along(0.775), closeTo(5 / 6, 1e-9));
    expect(MarkMotion.along(0.975), closeTo(1, 1e-9));
    expect(MarkMotion.along(0.99), closeTo(1, 1e-9));
  });

  test('it only moves forward', () {
    var last = 0.0;
    for (var i = 0; i <= 1000; i++) {
      final at = MarkMotion.along(i / 1000);
      expect(at, greaterThanOrEqualTo(last - 1e-12));
      last = at;
    }
  });

  test('the legs cover the whole way in order', () {
    var at = 0.0;
    var time = 0.0;
    for (final (t0, t1, p0, p1) in MarkMotion.legs) {
      expect(t0, greaterThanOrEqualTo(time));
      expect(p0, closeTo(at, 1e-9));
      at = p1;
      time = t1;
    }
    expect(at, 1.0);
    expect(time, lessThanOrEqualTo(1.0));
  });
}
