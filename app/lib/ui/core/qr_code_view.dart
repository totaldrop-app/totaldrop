// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'dart:async';

import 'package:flutter/semantics.dart';
import 'package:material_ui/material_ui.dart';

import '../../src/rust/api/transfer.dart' show QrGrid, qrForCode;
import 'theme.dart';

/// High-contrast QR code for `https://totaldrop.app/r#<code>` (#29).
///
/// Rendered dark-on-white with a four-module quiet zone in both light and
/// dark themes so phone cameras read it reliably without screen glare.
/// Tapping opens a larger, zero-elevation high-contrast dialog for scanning
/// across a desk or room; the dialog closes automatically as soon as the
/// waiting screen unmounts (for example when the receiver connects and the
/// four verification words appear).
class QrCodeView extends StatefulWidget {
  const QrCodeView({
    super.key,
    required this.code,
    this.size = 176,
    this.expandable = true,
    this.dismissListenable,
    this.shouldDismiss,
  });

  final String code;
  final double size;
  final bool expandable;
  final Listenable? dismissListenable;
  final bool Function()? shouldDismiss;

  @override
  State<QrCodeView> createState() => _QrCodeViewState();
}

class _QrCodeViewState extends State<QrCodeView> {
  DialogRoute<void>? _dialogRoute;
  NavigatorState? _navigator;

  @override
  void initState() {
    super.initState();
    widget.dismissListenable?.addListener(_onDismissSignal);
  }

  @override
  void didUpdateWidget(covariant QrCodeView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.dismissListenable != widget.dismissListenable) {
      oldWidget.dismissListenable?.removeListener(_onDismissSignal);
      widget.dismissListenable?.addListener(_onDismissSignal);
    }
    _onDismissSignal();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _navigator = Navigator.maybeOf(context);
  }

  void _onDismissSignal() {
    if (widget.shouldDismiss?.call() ?? false) {
      _closeDialogNow();
    }
  }

  void _closeDialogNow() {
    final route = _dialogRoute;
    _dialogRoute = null;
    final nav = _navigator;
    if (route != null && route.isActive && nav != null && nav.mounted) {
      nav.removeRoute(route);
    }
  }

  @override
  void dispose() {
    widget.dismissListenable?.removeListener(_onDismissSignal);
    final route = _dialogRoute;
    _dialogRoute = null;
    final nav = _navigator;
    if (route != null && route.isActive && nav != null) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (route.isActive && nav.mounted) {
          nav.removeRoute(route);
        }
      });
    }
    super.dispose();
  }

  void _showEnlargedQr(QrGrid grid) {
    if (_dialogRoute?.isActive ?? false) return;
    if (widget.shouldDismiss?.call() ?? false) return;
    final navigator = Navigator.maybeOf(context);
    if (navigator == null) return;
    final theme = Theme.of(context);
    final code = widget.code;
    late final DialogRoute<void> route;
    route = DialogRoute<void>(
      context: context,
      builder: (dialogContext) {
        final shortest = MediaQuery.sizeOf(dialogContext).shortestSide;
        final enlarged = (shortest - 96).clamp(176.0, 280.0);
        return Dialog(
          backgroundColor: theme.colorScheme.surface,
          surfaceTintColor: Colors.transparent,
          elevation: 0,
          shape: RoundedRectangleBorder(
            borderRadius: const BorderRadius.all(Radius.circular(28)),
            side: BorderSide(color: theme.colorScheme.outlineVariant),
          ),
          child: SingleChildScrollView(
            padding: const EdgeInsets.fromLTRB(24, 16, 24, 24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Align(
                  alignment: AlignmentDirectional.topEnd,
                  child: KeepIconButtonVariant(
                    child: CloseButton(
                      onPressed: () => Navigator.pop(dialogContext),
                    ),
                  ),
                ),
                ExcludeSemantics(
                  child: _QrFrame(grid: grid, size: enlarged),
                ),
                const SizedBox(height: 16),
                Semantics(
                  attributedLabel: AttributedString(
                    code,
                    attributes: [
                      LocaleStringAttribute(
                        range: TextRange(start: 0, end: code.length),
                        locale: const Locale('en'),
                      ),
                    ],
                  ),
                  excludeSemantics: true,
                  child: Directionality(
                    textDirection: TextDirection.ltr,
                    child: SelectableText(
                      code,
                      textAlign: TextAlign.center,
                      style: theme.textTheme.titleLarge,
                    ),
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
    _dialogRoute = route;
    unawaited(
      navigator.push<void>(route).whenComplete(() {
        if (identical(_dialogRoute, route)) {
          _dialogRoute = null;
        }
      }),
    );
  }

  @override
  Widget build(BuildContext context) {
    final code = widget.code;
    if (code.isEmpty) return const SizedBox.shrink();
    final grid = qrForCode(code: code);
    if (grid.width <= 0 || grid.modules.length != grid.width * grid.width) {
      return const SizedBox.shrink();
    }
    final frame = _QrFrame(grid: grid, size: widget.size);
    return ExcludeSemantics(
      child: widget.expandable
          ? GestureDetector(onTap: () => _showEnlargedQr(grid), child: frame)
          : frame,
    );
  }
}

class _QrFrame extends StatelessWidget {
  const _QrFrame({required this.grid, required this.size});

  final QrGrid grid;
  final double size;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return DecoratedBox(
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: const BorderRadius.all(Radius.circular(20)),
        border: Border.all(color: theme.colorScheme.outlineVariant),
      ),
      child: SizedBox.square(
        dimension: size,
        child: CustomPaint(painter: _QrPainter(grid)),
      ),
    );
  }
}

class _QrPainter extends CustomPainter {
  const _QrPainter(this.grid);

  final QrGrid grid;

  static const _quietZone = 4;
  static const _dark = Color(0xFF101418);

  @override
  void paint(Canvas canvas, Size size) {
    final n = grid.width;
    final total = n + _quietZone * 2;
    final cell = size.shortestSide / total;
    final originX = (size.width - cell * total) / 2;
    final originY = (size.height - cell * total) / 2;
    final paint = Paint()
      ..color = _dark
      ..style = PaintingStyle.fill
      ..isAntiAlias = false;

    final modules = grid.modules;
    for (var row = 0; row < n; row++) {
      final rowOffset = row * n;
      final top = originY + (row + _quietZone) * cell;
      final bottom = originY + (row + _quietZone + 1) * cell;
      var col = 0;
      while (col < n) {
        if (modules.codeUnitAt(rowOffset + col) != 0x31) {
          col++;
          continue;
        }
        var runEnd = col + 1;
        while (runEnd < n && modules.codeUnitAt(rowOffset + runEnd) == 0x31) {
          runEnd++;
        }
        canvas.drawRect(
          Rect.fromLTRB(
            originX + (col + _quietZone) * cell,
            top,
            originX + (runEnd + _quietZone) * cell,
            bottom,
          ),
          paint,
        );
        col = runEnd;
      }
    }
  }

  @override
  bool shouldRepaint(_QrPainter oldDelegate) => oldDelegate.grid != grid;
}
