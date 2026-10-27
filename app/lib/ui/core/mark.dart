import 'dart:math' as math;

import 'package:material_ui/material_ui.dart';

import 'theme.dart';

/// The TotalDrop mark, "Dalga" (wave): a drop at the centre, two waves
/// spreading from it, a straight channel cut through them to the north-east
/// and the file again at the end of the channel, arrived.
///
/// Drawn in code so the file can travel the channel while a transfer runs.
/// At rest it matches `assets/logo.svg` exactly. The mark keeps its own
/// colours in every theme: ink waves and violet file on light backgrounds,
/// paper waves and glow file on dark ones.
class TotalDropMark extends StatefulWidget {
  const TotalDropMark({super.key, this.size = 96, this.sending = false});

  final double size;

  /// Whether the transfer animation runs: the file leaves the drop and
  /// travels the channel while a ring spreads from the centre, exactly as on
  /// totaldrop.app (see docs/tasarim.md).
  final bool sending;

  @override
  State<TotalDropMark> createState() => _TotalDropMarkState();
}

class _TotalDropMarkState extends State<TotalDropMark> with SingleTickerProviderStateMixin {
  late final AnimationController _c =
      AnimationController(vsync: this, duration: const Duration(milliseconds: 3200));

  bool _shouldMove(BuildContext context) =>
      widget.sending && !(MediaQuery.maybeDisableAnimationsOf(context) ?? false);

  void _sync() {
    if (!mounted) return;
    if (_shouldMove(context)) {
      if (!_c.isAnimating) _c.repeat();
    } else {
      _c.stop();
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
            painter: _MarkPainter(phase: _shouldMove(context) ? _c.value : null, dark: dark),
          ),
        ),
      ),
    );
  }
}

class _MarkPainter extends CustomPainter {
  _MarkPainter({required this.phase, required this.dark});

  /// 0..1 through one 3.2 s loop, or null for the resting mark.
  final double? phase;
  final bool dark;

  // Geometry, in the SVG's units (viewBox 0 0 112 112).
  static const _centre = Offset(56, 56);
  static const _dropR = 12.0;
  static const _waves = [(26.0, 8.0), (40.0, 6.0)];
  static const _channelHalfWidth = 9.0;
  static const _fileR = 8.0;
  static const _fileDistance = 52.0;
  static const _axis = -math.pi / 4; // north-east

  // The web's keyframes: travel 0..55 %, ripple 0..70 % of the loop.
  static const _travelEnd = 0.55;
  static const _rippleEnd = 0.70;
  static const _travelCurve = Cubic(0.65, 0, 0.35, 1);
  static const _rippleCurve = Cubic(0.2, 0.6, 0.3, 1);

  static double _distance(double t) {
    if (t >= _travelEnd) return _fileDistance;
    final u = _travelCurve.transform(t / _travelEnd);
    return _dropR + (_fileDistance - _dropR) * u;
  }

  @override
  void paint(Canvas canvas, Size size) {
    final unit = size.width / 112;
    canvas.scale(unit);

    final waveColor = dark ? Brand.paper : Brand.ink;
    final fileColor = dark ? Brand.glow : Brand.accent;
    final wave = Paint()
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.butt
      ..color = waveColor
      ..isAntiAlias = true;
    final fill = Paint()
      ..color = fileColor
      ..isAntiAlias = true;

    for (final (r, width) in _waves) {
      final h = math.asin(_channelHalfWidth / r);
      canvas.drawArc(
        Rect.fromCircle(center: _centre, radius: r),
        _axis + h,
        2 * math.pi - 2 * h,
        false,
        wave..strokeWidth = width,
      );
    }

    final t = phase;
    if (t != null && t < _rippleEnd) {
      final u = _rippleCurve.transform(t / _rippleEnd);
      final scale = 1 + 3.6 * u;
      final opacity = 0.7 * (1 - u);
      canvas.drawCircle(
        _centre,
        _dropR * scale,
        Paint()
          ..style = PaintingStyle.stroke
          // 2 logical pixels whatever the size, like the web's non-scaling stroke.
          ..strokeWidth = 2 / unit
          ..color = fileColor.withValues(alpha: opacity)
          ..isAntiAlias = true,
      );
    }

    canvas.drawCircle(_centre, _dropR, fill);

    final d = t == null ? _fileDistance : _distance(t);
    final file = _centre + Offset(math.cos(_axis), math.sin(_axis)) * d;
    canvas.drawCircle(file, _fileR, fill);
  }

  @override
  bool shouldRepaint(_MarkPainter old) => old.phase != phase || old.dark != dark;
}
