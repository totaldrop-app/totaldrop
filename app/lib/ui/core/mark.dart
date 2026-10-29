import 'package:material_ui/material_ui.dart';

import 'theme.dart';

/// The TotalDrop mark, "Ağ" (network): three equal devices on a circle, each
/// joined to the other two, and a drop on the bottom link pointing the way
/// it travels.
///
/// Drawn in code so the drop can visit the devices while a transfer runs.
/// At rest it matches `assets/logo.svg` exactly. The mark keeps its own
/// colours in every theme: ink links and violet devices on light
/// backgrounds, paper links and glow devices on dark ones.
class TotalDropMark extends StatefulWidget {
  const TotalDropMark({super.key, this.size = 96, this.sending = false});

  final double size;

  /// Whether the transfer animation runs: the drop travels the links and
  /// stops briefly at each device, exactly as on totaldrop.app (see
  /// docs/tasarim.md).
  final bool sending;

  @override
  State<TotalDropMark> createState() => _TotalDropMarkState();
}

class _TotalDropMarkState extends State<TotalDropMark> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: MarkMotion.loop);

  bool _shouldMove(BuildContext context) =>
      widget.sending && !(MediaQuery.maybeDisableAnimationsOf(context) ?? false);

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
              along: _shouldMove(context) ? MarkMotion.along(_c.value) : 0,
              dark: dark,
            ),
          ),
        ),
      ),
    );
  }
}

/// The transfer animation's timing, shared with the website (the hero's
/// `animateMotion` in index.html) and the Linux app (Mark.qml).
abstract final class MarkMotion {
  static const loop = Duration(milliseconds: 4800);

  /// Each leg: start and end of the loop (0..1), start and end along the
  /// motion path (0..1). Between legs the drop rests on a device.
  static const legs = [
    (0.0, 0.125, 0.0, 1 / 6),
    (0.2, 0.45, 1 / 6, 0.5),
    (0.525, 0.775, 0.5, 5 / 6),
    (0.85, 0.975, 5 / 6, 1.0),
  ];
  static const _ease = Cubic(0.65, 0, 0.35, 1);

  /// How far along the motion path the drop is at [t] (0..1 through the loop).
  static double along(double t) {
    var at = 0.0;
    for (final (t0, t1, p0, p1) in legs) {
      if (t < t0) return at;
      if (t <= t1) return p0 + (p1 - p0) * _ease.transform((t - t0) / (t1 - t0));
      at = p1;
    }
    return at;
  }
}

class _MarkPainter extends CustomPainter {
  _MarkPainter({required this.along, required this.dark});

  /// 0..1 along the motion path; 0 (and 1) is the resting place.
  final double along;
  final bool dark;

  // Geometry, in the SVG's units (viewBox 0 0 112 112).
  static const _nodes = [Offset(56, 24), Offset(88.91, 81), Offset(23.09, 81)];
  static const _nodeR = 11.0;
  static const _linkR = Radius.circular(60);
  static const _linkWidth = 7.0;

  /// The links: each pair of devices joined by an arc bowing outwards.
  static final Path _links = Path()
    ..moveTo(_nodes[0].dx, _nodes[0].dy)
    ..arcToPoint(_nodes[1], radius: _linkR)
    ..arcToPoint(_nodes[2], radius: _linkR)
    ..arcToPoint(_nodes[0], radius: _linkR)
    ..close();

  /// The drop's way round: from its resting place in the middle of the
  /// bottom link, to the right device, the top one, the left one and back.
  static final _motion =
      (Path()
            ..moveTo(56, 90.83)
            ..arcToPoint(_nodes[1], radius: _linkR, clockwise: false)
            ..arcToPoint(_nodes[0], radius: _linkR, clockwise: false)
            ..arcToPoint(_nodes[2], radius: _linkR, clockwise: false)
            ..arcToPoint(const Offset(56, 90.83), radius: _linkR, clockwise: false))
          .computeMetrics()
          .first;

  /// The drop, body centred on the origin and pointing along +x.
  static final Path _drop = Path()
    ..moveTo(11, 0)
    ..lineTo(3.84, -5.24)
    ..arcToPoint(
      const Offset(3.84, 5.24),
      radius: const Radius.circular(6.5),
      largeArc: true,
      clockwise: false,
    )
    ..close();

  @override
  void paint(Canvas canvas, Size size) {
    canvas.scale(size.width / 112);

    final fill = Paint()
      ..color = dark ? Brand.glow : Brand.accent
      ..isAntiAlias = true;
    canvas.drawPath(
      _links,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = _linkWidth
        ..color = dark ? Brand.paper : Brand.ink
        ..isAntiAlias = true,
    );
    for (final n in _nodes) {
      canvas.drawCircle(n, _nodeR, fill);
    }

    final at = _motion.getTangentForOffset(_motion.length * (along % 1.0))!;
    canvas
      ..save()
      ..translate(at.position.dx, at.position.dy)
      ..rotate(-at.angle)
      ..drawPath(_drop, fill)
      ..restore();
  }

  @override
  bool shouldRepaint(_MarkPainter old) => old.along != along || old.dark != dark;
}
