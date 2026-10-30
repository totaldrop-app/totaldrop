import 'dart:math' as math;

import 'package:material_ui/material_ui.dart';

import 'theme.dart';

/// The TotalDrop mark, "Köşe" (corner): a circle with one square corner.
/// The circle is the whole file, the corner makes it a drop and points the
/// way it goes. While a transfer runs it turns a quarter at a time, so the
/// corner points at every side in turn: any device to any device.
///
/// At rest it matches `assets/logo.svg` exactly. It keeps its own colour in
/// every theme: violet on light backgrounds, glow on dark ones.
class TotalDropMark extends StatefulWidget {
  const TotalDropMark({
    super.key,
    this.size = 96,
    this.sending = false,
    this.done = false,
  });

  final double size;

  /// Whether the transfer animation runs, exactly as on totaldrop.app (see
  /// docs/tasarim.md, "Hareket").
  final bool sending;

  /// The file arrived: a check inside the circle.
  final bool done;

  @override
  State<TotalDropMark> createState() => _TotalDropMarkState();
}

class _TotalDropMarkState extends State<TotalDropMark>
    with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(
    vsync: this,
    duration: MarkMotion.loop,
  );

  bool _shouldMove(BuildContext context) =>
      widget.sending &&
      !(MediaQuery.maybeDisableAnimationsOf(context) ?? false);

  void _sync() {
    if (!mounted) return;
    if (_shouldMove(context)) {
      if (!_c.isAnimating) _c.repeat();
    } else {
      // Back to rest, so the next transfer starts the loop from the beginning.
      _c
        ..stop()
        ..value = 0;
    }
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _sync();
  }

  @override
  void didUpdateWidget(TotalDropMark old) {
    super.didUpdateWidget(old);
    _sync();
  }

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final dark = Theme.of(context).brightness == Brightness.dark;
    return Semantics(
      label: 'TotalDrop',
      image: true,
      child: RepaintBoundary(
        child: AnimatedBuilder(
          animation: _c,
          builder: (context, _) => CustomPaint(
            size: Size.square(widget.size),
            painter: _MarkPainter(
              turn: _shouldMove(context) ? MarkMotion.turn(_c.value) : 0,
              dark: dark,
              done: widget.done,
            ),
          ),
        ),
      ),
    );
  }
}

/// The transfer animation's timing, shared with the website (the hero in
/// styles.css) and the Linux app (Mark.qml): four quarter turns, each eased
/// over 0.6 s and followed by a 0.2 s rest.
abstract final class MarkMotion {
  static const loop = Duration(milliseconds: 3200);

  /// The share of each quarter (0.8 s of 3.2 s) spent turning: 0.6 s.
  static const turning = 0.75;
  static const _ease = Cubic(0.65, 0, 0.35, 1);

  /// How far the mark has turned at [t] (0..1 through the loop), in whole
  /// turns: 0.25 is a quarter turn clockwise, 1 is back at rest.
  static double turn(double t) {
    final quarter = (t.clamp(0.0, 1.0) * 4).floor().clamp(0, 3);
    final within = (t * 4 - quarter) / turning;
    final eased = within >= 1 ? 1.0 : _ease.transform(within.clamp(0.0, 1.0));
    return (quarter + eased) / 4;
  }
}

/// The mark's geometry in the SVG's units (viewBox 0 0 112 112). The circle
/// sits one unit down and left of the box's centre, so the shape, heavier
/// at its corner, looks centred.
abstract final class MarkGeometry {
  static const centre = Offset(55, 57);
  static const radius = 42.0;

  /// The resting shape: corner at top right.
  static Path shape() => Path()
    ..moveTo(55, 15)
    ..lineTo(97, 15)
    ..lineTo(97, 57)
    ..arcToPoint(
      const Offset(55, 15),
      radius: const Radius.circular(radius),
      largeArc: true,
    )
    ..close();

  /// The check drawn inside the circle when a file has arrived.
  static Path check() => Path()
    ..moveTo(37, 58)
    ..lineTo(50, 71)
    ..lineTo(74, 45);
}

class _MarkPainter extends CustomPainter {
  _MarkPainter({required this.turn, required this.dark, required this.done});

  /// Whole turns from rest, clockwise.
  final double turn;
  final bool dark;
  final bool done;

  static final Path _shape = MarkGeometry.shape();
  static final Path _check = MarkGeometry.check();

  @override
  void paint(Canvas canvas, Size size) {
    canvas
      ..scale(size.width / 112)
      ..save()
      ..translate(MarkGeometry.centre.dx, MarkGeometry.centre.dy)
      ..rotate(turn * 2 * math.pi)
      ..translate(-MarkGeometry.centre.dx, -MarkGeometry.centre.dy)
      ..drawPath(
        _shape,
        Paint()
          ..color = dark ? Brand.glow : Brand.accent
          ..isAntiAlias = true,
      )
      ..restore();
    if (done) {
      canvas.drawPath(
        _check,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 9
          ..strokeCap = StrokeCap.round
          ..strokeJoin = StrokeJoin.round
          ..color = dark ? Brand.ink : Brand.paper
          ..isAntiAlias = true,
      );
    }
  }

  @override
  bool shouldRepaint(_MarkPainter old) =>
      old.turn != turn || old.dark != dark || old.done != done;
}
