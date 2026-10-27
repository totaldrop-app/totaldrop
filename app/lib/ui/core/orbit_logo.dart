import 'dart:math' as math;

import 'package:material_ui/material_ui.dart';

import 'theme.dart';

/// The "Yörünge" logo, drawn in code so its satellite can orbit the planet
/// while a transfer runs. At rest it matches `assets/logo.svg` exactly.
///
/// The logo keeps its own colours in every theme: ink planet and violet
/// orbit on light backgrounds, paper planet and glow orbit on dark ones.
class OrbitLogo extends StatefulWidget {
  const OrbitLogo({super.key, this.size = 96, this.orbiting = false, this.fast = false});

  final double size;

  /// Whether the satellite circles the planet.
  final bool orbiting;

  /// Faster orbit, for while data is flowing.
  final bool fast;

  @override
  State<OrbitLogo> createState() => _OrbitLogoState();
}

class _OrbitLogoState extends State<OrbitLogo> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this);

  Duration get _period =>
      widget.fast ? const Duration(milliseconds: 1400) : const Duration(milliseconds: 3600);

  bool _shouldOrbit(BuildContext context) =>
      widget.orbiting && !(MediaQuery.maybeDisableAnimationsOf(context) ?? false);

  void _sync() {
    if (!mounted) return;
    if (_shouldOrbit(context)) {
      _c.duration = _period;
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
  void didUpdateWidget(OrbitLogo old) {
    super.didUpdateWidget(old);
    if (old.fast != widget.fast && _c.isAnimating) {
      // Keep the satellite where it is while changing speed.
      final at = _c.value;
      _c.duration = _period;
      _c.repeat();
      _c.value = at;
    }
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
    final background = Theme.of(context).colorScheme.surface;
    return Semantics(
      label: 'TotalDrop',
      image: true,
      child: RepaintBoundary(
        child: AnimatedBuilder(
          animation: _c,
          builder: (context, _) => CustomPaint(
            size: Size.square(widget.size),
            painter: _OrbitPainter(
              phase: _shouldOrbit(context) ? _c.value : null,
              dark: dark,
              background: background,
            ),
          ),
        ),
      ),
    );
  }
}

class _OrbitPainter extends CustomPainter {
  _OrbitPainter({required this.phase, required this.dark, required this.background});

  /// 0..1 around the orbit, or null for the resting logo.
  final double? phase;
  final bool dark;
  final Color background;

  // Geometry of the logo, in its SVG units (viewBox 0 6 112 112).
  static const _planet = Offset(56, 62);
  static const _planetR = 31.0;
  static const _restSatellite = Offset(89.07, 38.68);
  static const _satelliteR = 11.0;
  // The orbit is the ellipse the ring is drawn on: centred on the planet,
  // radii 50 and 15, tilted -22°. The resting satellite sits on it at 0.8944.
  static const _orbitCentre = Offset(56, 62);
  static const _rx = 50.0;
  static const _ry = 15.0;
  static final _tilt = -22 * math.pi / 180;
  static const _restPhase = 0.8944;

  Offset _onOrbit(double t) {
    final a = t * 2 * math.pi;
    final x = _rx * math.cos(a);
    final y = _ry * math.sin(a);
    return _orbitCentre +
        Offset(
          x * math.cos(_tilt) - y * math.sin(_tilt),
          x * math.sin(_tilt) + y * math.cos(_tilt),
        );
  }

  @override
  void paint(Canvas canvas, Size size) {
    final scale = size.width / 112;
    canvas.scale(scale);
    canvas.translate(0, -6);

    final planetColor = dark ? Brand.paper : Brand.ink;
    final orbitColor = dark ? Brand.glow : Brand.accent;
    final fill = Paint()..isAntiAlias = true;
    final ring = Paint()
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round
      ..isAntiAlias = true;

    Offset satellite = _restSatellite;
    var behind = false;
    if (phase != null) {
      // Start where the resting satellite sits (upper right) and go round.
      // Over the top of the planet it passes behind it.
      final t = (phase! + _restPhase) % 1.0;
      satellite = _onOrbit(t);
      behind = math.sin(t * 2 * math.pi) < -0.65;
    }

    if (phase != null && behind) {
      canvas.drawCircle(satellite, _satelliteR, fill..color = orbitColor);
    }

    canvas.drawCircle(_planet, _planetR, fill..color = planetColor);

    // The gap that lifts the ring off the planet, in the background colour.
    final gap = Path()
      ..moveTo(84.05, 64.68)
      ..arcToPoint(
        const Offset(34.01, 84.14),
        radius: const Radius.elliptical(50, 15),
        rotation: -22,
        clockwise: true,
      );
    canvas.drawPath(
      gap,
      ring
        ..color = background
        ..strokeWidth = 24,
    );

    final orbit = Path()
      ..moveTo(98.96, 52.73)
      ..arcToPoint(
        const Offset(61.62, 75.91),
        radius: const Radius.elliptical(50, 15),
        rotation: -22,
        clockwise: true,
      )
      ..arcToPoint(
        const Offset(9.64, 80.73),
        radius: const Radius.elliptical(50, 15),
        rotation: -22,
        clockwise: true,
      );
    canvas.drawPath(
      orbit,
      ring
        ..color = orbitColor
        ..strokeWidth = 13,
    );

    if (phase == null || !behind) {
      if (phase != null) {
        // A thin halo in the background colour keeps the satellite readable
        // where it crosses the ring.
        canvas.drawCircle(satellite, _satelliteR + 3, fill..color = background);
      }
      canvas.drawCircle(satellite, _satelliteR, fill..color = orbitColor);
    }
  }

  @override
  bool shouldRepaint(_OrbitPainter old) =>
      old.phase != phase || old.dark != dark || old.background != background;
}
