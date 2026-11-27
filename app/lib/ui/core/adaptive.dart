// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'dart:math' as math;
import 'dart:ui' show DisplayFeature, DisplayFeatureState, DisplayFeatureType;

import 'package:material_ui/material_ui.dart';

/// Material 3 canonical width breakpoints (logical dp).
const double compactBreakpoint = 600;
const double expandedBreakpoint = 840;

/// Folded outer cover displays (320–344 dp) and narrow panes tighten outer
/// horizontal margins so controls and 200% text keep room.
const double coverScreenThreshold = 360;

/// Maximum combined width for flat side-by-side panes so ultra-wide tablets
/// keep the two panes visually connected.
const double maxTwoPaneWidth = 1160;

/// Minimum usable width on each side of a vertical fold or hinge so M3 cards
/// and verification-word chips never crush at large text sizes.
const double minHorizontalPaneWidth = 280;

/// Minimum usable height above and below a horizontal fold or hinge.
/// When the soft keyboard covers the lower half in tabletop posture, the
/// bottom pane drops below this floor and collapses cleanly to single-pane.
const double minVerticalPaneHeight = 140;

/// Above this text scale on a Medium (600–839 dp) flat screen, fall back to a
/// centered 560 dp single column so 200% text gets the full column width.
const double twoPaneMediumMaxTextScale = 1.5;

/// Canonical Material 3 window width classes.
enum WindowWidthClass {
  /// Under 600 dp: phones, cover screens, and narrow multi-window splits.
  compact,

  /// 600 to 839 dp: unfolded foldables in portrait, small tablets.
  medium,

  /// 840 dp and up: unfolded foldables in landscape, tablets, desktop windows.
  expanded,
}

/// Physical posture derived from active separating [DisplayFeature]s.
enum FoldPosture {
  /// Continuous flat screen, or a fold whose state is flat.
  flat,

  /// Half-opened vertical fold or physical vertical hinge splitting the
  /// viewport into start and end panes.
  book,

  /// Half-opened horizontal fold or physical horizontal hinge splitting the
  /// viewport into top (display) and bottom (controls) panes.
  tabletop,
}

/// Outer horizontal margin for a given available width: 16 dp on narrow cover
/// displays and narrow panes (< 360 dp), 24 dp elsewhere.
double adaptiveHorizontalMargin(double width) =>
    width < coverScreenThreshold ? 16 : 24;

/// Immutable adaptive layout metrics computed from viewport geometry and
/// `dart:ui` [DisplayFeature]s with zero external dependencies.
@immutable
class AdaptiveMetrics {
  const AdaptiveMetrics({
    required this.size,
    required this.widthClass,
    required this.posture,
    required this.hingeBounds,
    required this.isCoverScreen,
    required this.textScale,
  });

  /// Derives [AdaptiveMetrics] using targeted [MediaQuery] selectors so
  /// unrelated viewport updates (such as keyboard animation frames) do not
  /// trigger redundant rebuilds.
  factory AdaptiveMetrics.of(BuildContext context, {Size? viewportSize}) =>
      AdaptiveMetrics.fromViewport(
        size: viewportSize ?? MediaQuery.sizeOf(context),
        displayFeatures: MediaQuery.displayFeaturesOf(context),
        textScaler: MediaQuery.textScalerOf(context),
      );

  /// Pure factory for deterministic unit and widget tests.
  factory AdaptiveMetrics.fromMediaQuery(
    MediaQueryData data, {
    Size? viewportSize,
  }) => AdaptiveMetrics.fromViewport(
    size: viewportSize ?? data.size,
    displayFeatures: data.displayFeatures,
    textScaler: data.textScaler,
  );

  /// Computes metrics from explicit viewport dimensions, display features,
  /// and text scaler. When [featureOrigin] is non-zero (for example, inside a
  /// [Scaffold] with an [AppBar] or consumed [SafeArea]), display-feature
  /// bounds are evaluated in the viewport's local coordinate space.
  factory AdaptiveMetrics.fromViewport({
    required Size size,
    required List<DisplayFeature> displayFeatures,
    required TextScaler textScaler,
    Offset featureOrigin = Offset.zero,
  }) {
    final textScale = textScaler.scale(16) / 16;
    final widthClass = size.width >= expandedBreakpoint
        ? WindowWidthClass.expanded
        : size.width >= compactBreakpoint
        ? WindowWidthClass.medium
        : WindowWidthClass.compact;
    final isCoverScreen = size.width < coverScreenThreshold;
    final (posture, hingeBounds) = _detectPosture(
      displayFeatures,
      size,
      featureOrigin,
    );
    return AdaptiveMetrics(
      size: size,
      widthClass: widthClass,
      posture: posture,
      hingeBounds: hingeBounds,
      isCoverScreen: isCoverScreen,
      textScale: textScale,
    );
  }

  final Size size;
  final WindowWidthClass widthClass;
  final FoldPosture posture;

  /// Bounds of the separating fold or hinge in window logical pixels, or null
  /// when [posture] is [FoldPosture.flat].
  final Rect? hingeBounds;

  /// True on narrow cover displays (< 360 dp, e.g. 320–344 dp) and tight
  /// multi-window splits.
  final bool isCoverScreen;

  /// Effective text scale factor derived from [TextScaler].
  final double textScale;

  /// Outer horizontal margin: 16 dp on narrow cover screens, 24 dp elsewhere.
  double get horizontalMargin => adaptiveHorizontalMargin(size.width);

  /// Whether a screen should present side-by-side panes.
  ///
  /// Book posture always splits around the hinge; flat screens use two panes
  /// on Expanded widths (>= 840 dp) and on Medium widths (600–839 dp) while
  /// text scale remains <= 1.5x.
  bool get useTwoPane => switch (posture) {
    FoldPosture.book => true,
    FoldPosture.tabletop => false,
    FoldPosture.flat =>
      widthClass == WindowWidthClass.expanded ||
          (widthClass == WindowWidthClass.medium &&
              textScale <= twoPaneMediumMaxTextScale),
  };

  static (FoldPosture, Rect?) _detectPosture(
    List<DisplayFeature> features,
    Size size,
    Offset featureOrigin,
  ) {
    for (final f in features) {
      final separates =
          f.type == DisplayFeatureType.hinge ||
          (f.type == DisplayFeatureType.fold &&
              f.state == DisplayFeatureState.postureHalfOpened);
      if (!separates) continue;
      final bounds = f.bounds;
      if (bounds.width <= 0 && bounds.height <= 0) continue;
      final localLeft = bounds.left - featureOrigin.dx;
      final localRight = bounds.right - featureOrigin.dx;
      final localTop = bounds.top - featureOrigin.dy;
      final localBottom = bounds.bottom - featureOrigin.dy;
      // Ignore features outside the current viewport (e.g. single-pane split).
      if (localRight < 0 ||
          localLeft > size.width ||
          localBottom < 0 ||
          localTop > size.height) {
        continue;
      }
      if (bounds.height > bounds.width) {
        final leftWidth = localLeft;
        final rightWidth = size.width - localRight;
        if (leftWidth >= minHorizontalPaneWidth &&
            rightWidth >= minHorizontalPaneWidth) {
          return (FoldPosture.book, bounds);
        }
      } else if (bounds.width > bounds.height) {
        final topHeight = localTop;
        final bottomHeight = size.height - localBottom;
        if (topHeight >= minVerticalPaneHeight &&
            bottomHeight >= minVerticalPaneHeight) {
          return (FoldPosture.tabletop, bounds);
        }
      }
    }
    return (FoldPosture.flat, null);
  }
}

/// Tracks whether a subtree is already placed inside an [AdaptivePaneLayout]
/// pane and whether that layout is actively split in tabletop posture.
class _AdaptivePaneScope extends InheritedWidget {
  const _AdaptivePaneScope({
    required this.inSplitPane,
    required this.activeTabletop,
    required super.child,
  });

  final bool inSplitPane;
  final bool activeTabletop;

  static _AdaptivePaneScope? maybeOf(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<_AdaptivePaneScope>();

  @override
  bool updateShouldNotify(_AdaptivePaneScope oldWidget) =>
      inSplitPane != oldWidget.inSplitPane ||
      activeTabletop != oldWidget.activeTabletop;
}

/// Computes the synchronous first-frame horizontal and vertical origin of a
/// widget placed directly inside a [Scaffold] body (optionally wrapped in
/// [SafeArea]) relative to the root window. [_OriginAwareBuilder] then refines
/// this with [RenderBox.localToGlobal] after layout so an extra header or
/// banner inside the body still aligns cleanly with physical hinges.
Offset _bodyOrigin(BuildContext context) {
  final localPadding = MediaQuery.paddingOf(context);
  final localViewPadding = MediaQuery.viewPaddingOf(context);
  final scaffold = Scaffold.maybeOf(context);
  // SafeArea calls MediaQuery.removePadding, which zeros both padding and
  // viewPadding for descendants. Reading the Scaffold's incoming MediaQuery
  // padding (without registering an ancestor dependency) recovers the exact
  // consumed inset whether SafeArea is present or omitted.
  final outerPadding =
      scaffold?.context
          .getInheritedWidgetOfExactType<MediaQuery>()
          ?.data
          .padding ??
      localViewPadding;
  final hasAppBar =
      (scaffold?.hasAppBar ?? false) &&
      !(scaffold?.widget.extendBodyBehindAppBar ?? false);

  final consumedLeft = (outerPadding.left - localPadding.left).clamp(
    0.0,
    outerPadding.left,
  );
  final consumedTop = (outerPadding.top - localPadding.top).clamp(
    0.0,
    outerPadding.top,
  );
  final bodyTop = hasAppBar
      ? (scaffold!.appBarMaxHeight ?? (outerPadding.top + kToolbarHeight))
      : consumedTop;
  return Offset(consumedLeft, bodyTop);
}

/// Start, hinge, and end extents along the split axis in local coordinates.
@immutable
class _HingeSplit {
  const _HingeSplit({
    required this.start,
    required this.hinge,
    required this.end,
  });

  final double start;
  final double hinge;
  final double end;
}

/// Splits [extent] around `[hingeStart, hingeEnd]` translated by [origin] into
/// `(start, hinge, end)` extents. When [rtl] is true (Book posture in RTL
/// locales), `start` is the physical right pane and `end` is the physical left
/// pane so [Row] children map cleanly to both sides of an asymmetric hinge.
_HingeSplit _splitAroundHinge({
  required double extent,
  required double hingeStart,
  required double hingeEnd,
  required double origin,
  bool rtl = false,
}) {
  final localStart = (hingeStart - origin).clamp(0.0, extent);
  final localEnd = (hingeEnd - origin).clamp(localStart, extent);
  final leading = localStart;
  final hinge = localEnd - localStart;
  final trailing = extent - localEnd;
  return rtl
      ? _HingeSplit(start: trailing, hinge: hinge, end: leading)
      : _HingeSplit(start: leading, hinge: hinge, end: trailing);
}

/// Places [child] strictly on the wider side of a vertical hinge so content
/// never straddles a physical fold when one or both local panes fall below
/// [minHorizontalPaneWidth].
Widget _singleOnWiderSide({required _HingeSplit split, required Widget child}) {
  final useStart = split.start >= split.end;
  return _AdaptivePaneScope(
    inSplitPane: true,
    activeTabletop: false,
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SizedBox(width: split.start, child: useStart ? child : null),
        if (split.hinge > 0) SizedBox(width: split.hinge),
        SizedBox(width: split.end, child: useStart ? null : child),
      ],
    ),
  );
}

/// Places [child] strictly on the taller half of a horizontal fold so content
/// never straddles a physical hinge when one or both local halves fall below
/// [minVerticalPaneHeight].
Widget _singleOnTallerHalf({
  required _HingeSplit split,
  required Widget child,
}) {
  final useTop = split.start >= split.end;
  return _AdaptivePaneScope(
    inSplitPane: true,
    activeTabletop: false,
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SizedBox(height: split.start, child: useTop ? child : null),
        if (split.hinge > 0) SizedBox(height: split.hinge),
        SizedBox(height: split.end, child: useTop ? null : child),
      ],
    ),
  );
}

/// Supplies constraints plus the widget's root-window origin, starting from
/// [_bodyOrigin] on the first frame and refining via [RenderBox.localToGlobal]
/// post-frame whenever anything above the widget shifts its position.
class _OriginAwareBuilder extends StatefulWidget {
  const _OriginAwareBuilder({required this.builder});

  final Widget Function(
    BuildContext context,
    BoxConstraints constraints,
    Offset origin,
  )
  builder;

  @override
  State<_OriginAwareBuilder> createState() => _OriginAwareBuilderState();
}

class _OriginAwareBuilderState extends State<_OriginAwareBuilder> {
  Offset? _lastFallback;
  Offset? _measuredOrigin;
  bool _measureScheduled = false;
  Animation<double>? _routeAnim;
  Animation<double>? _secondaryRouteAnim;

  void _attachRouteAnimations(ModalRoute<dynamic>? route) {
    final primary = route?.animation;
    if (_routeAnim != primary) {
      _routeAnim?.removeStatusListener(_onPrimaryAnimationStatus);
      _routeAnim = primary;
      _routeAnim?.addStatusListener(_onPrimaryAnimationStatus);
    }
    final secondary = route?.secondaryAnimation;
    if (_secondaryRouteAnim != secondary) {
      _secondaryRouteAnim?.removeStatusListener(_onSecondaryAnimationStatus);
      _secondaryRouteAnim = secondary;
      _secondaryRouteAnim?.addStatusListener(_onSecondaryAnimationStatus);
    }
  }

  bool get _routeSettled {
    final primary = _routeAnim;
    if (primary != null && primary.status != AnimationStatus.completed) {
      return false;
    }
    final secondary = _secondaryRouteAnim;
    if (secondary != null && secondary.status != AnimationStatus.dismissed) {
      return false;
    }
    return true;
  }

  void _onPrimaryAnimationStatus(AnimationStatus status) {
    if (!mounted) return;
    if (status == AnimationStatus.completed && _routeSettled) {
      _scheduleMeasure();
      WidgetsBinding.instance.ensureVisualUpdate();
    }
  }

  void _onSecondaryAnimationStatus(AnimationStatus status) {
    if (!mounted) return;
    if (status == AnimationStatus.dismissed && _routeSettled) {
      _scheduleMeasure();
      WidgetsBinding.instance.ensureVisualUpdate();
    }
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _attachRouteAnimations(ModalRoute.of(context));
  }

  @override
  void dispose() {
    _attachRouteAnimations(null);
    super.dispose();
  }

  void _scheduleMeasure() {
    if (_measureScheduled) return;
    _measureScheduled = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _measureScheduled = false;
      if (!mounted || !_routeSettled) return;
      final box = context.findRenderObject();
      if (box is! RenderBox || !box.hasSize || !box.attached) return;
      final actual = box.localToGlobal(Offset.zero);
      if (!actual.dx.isFinite || !actual.dy.isFinite) return;
      final current = _measuredOrigin ?? _lastFallback ?? Offset.zero;
      if ((actual - current).distance > 0.5) {
        setState(() => _measuredOrigin = actual);
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final fallback = _bodyOrigin(context);
    if (_lastFallback != fallback) {
      _lastFallback = fallback;
      _measuredOrigin = null;
    }
    _scheduleMeasure();
    final origin = _measuredOrigin ?? fallback;
    return LayoutBuilder(
      builder: (context, constraints) =>
          widget.builder(context, constraints, origin),
    );
  }
}

/// Places a display-only single-pane stage without interactive action buttons
/// (such as Preparing, Connecting, or Transferring without verification words)
/// on a hinge-free sub-screen in Book or Tabletop posture so content never
/// straddles a physical hinge. Stages with interactive actions (Done, Declined,
/// Failed) use [AdaptivePaneLayout] instead so controls sit below the fold in
/// Tabletop posture. Becomes a pass-through when already nested inside an
/// [AdaptivePaneLayout] pane or when the screen is flat.
class AdaptiveSinglePane extends StatelessWidget {
  const AdaptiveSinglePane({super.key, required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    if (_AdaptivePaneScope.maybeOf(context)?.inSplitPane ?? false) {
      return child;
    }
    final displayFeatures = MediaQuery.displayFeaturesOf(context);
    if (displayFeatures.isEmpty) return child;

    final direction = Directionality.of(context);
    final windowSize = MediaQuery.sizeOf(context);
    final textScaler = MediaQuery.textScalerOf(context);
    final physical = AdaptiveMetrics.fromViewport(
      size: windowSize,
      displayFeatures: displayFeatures,
      textScaler: textScaler,
    );
    if (physical.posture == FoldPosture.flat || physical.hingeBounds == null) {
      return child;
    }

    return _OriginAwareBuilder(
      builder: (context, constraints, origin) {
        final hinge = physical.hingeBounds!;

        if (physical.posture == FoldPosture.book) {
          final split = _splitAroundHinge(
            extent: constraints.maxWidth,
            hingeStart: hinge.left,
            hingeEnd: hinge.right,
            origin: origin.dx,
            rtl: direction == TextDirection.rtl,
          );
          if (split.start >= minHorizontalPaneWidth &&
              split.end >= minHorizontalPaneWidth) {
            return _AdaptivePaneScope(
              inSplitPane: true,
              activeTabletop: false,
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  SizedBox(width: split.start, child: child),
                  if (split.hinge > 0) SizedBox(width: split.hinge),
                  SizedBox(width: split.end),
                ],
              ),
            );
          }
          if (math.max(split.start, split.end) > 0) {
            return _singleOnWiderSide(split: split, child: child);
          }
        }

        if (physical.posture == FoldPosture.tabletop) {
          final split = _splitAroundHinge(
            extent: constraints.maxHeight,
            hingeStart: hinge.top,
            hingeEnd: hinge.bottom,
            origin: origin.dy,
          );
          if (split.start >= minVerticalPaneHeight &&
              split.end >= minVerticalPaneHeight) {
            return _AdaptivePaneScope(
              inSplitPane: true,
              activeTabletop: true,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  SizedBox(height: split.start, child: child),
                  SizedBox(height: split.hinge + split.end),
                ],
              ),
            );
          }
          if (math.max(split.start, split.end) > 0) {
            return _singleOnTallerHalf(split: split, child: child);
          }
        }

        return child;
      },
    );
  }
}

/// Posture-aware two-pane container for foldable and large-screen devices.
///
/// - **Book posture:** places [primary] and [secondary] strictly on opposite
///   sides of the vertical hinge/fold, computing start and end pane widths
///   with full RTL awareness so asymmetric hinges never clip content. When
///   local insets shrink either pane below [minHorizontalPaneWidth], places
///   [singlePane] on the wider side rather than straddling the hinge.
/// - **Tabletop posture (when [allowTabletop] is true):** places [primary]
///   (display/status) above the horizontal fold and [secondary]
///   (controls/actions) below it, accounting for [AppBar], consumed [SafeArea]
///   insets, and any ancestor offset via [RenderBox.localToGlobal], placing
///   [singlePane] on the taller half rather than straddling the hinge if the
///   soft keyboard or local insets shrink either half below
///   [minVerticalPaneHeight].
/// - **Unfolded flat (`Medium`/`Expanded`, when [allowFlatTwoPane] is true):**
///   places [primary] and [secondary] side by side with a 1 dp hairline
///   divider capped at [maxTwoPaneWidth].
/// - **Compact / cover / high-scale fallback:** renders [singlePane].
class AdaptivePaneLayout extends StatelessWidget {
  const AdaptivePaneLayout({
    super.key,
    required this.primary,
    required this.secondary,
    required this.singlePane,
    this.allowFlatTwoPane = true,
    this.allowTabletop = true,
  });

  /// Display / context pane (start in Book/Flat, top in Tabletop).
  final Widget primary;

  /// Interactive / action pane (end in Book/Flat, bottom in Tabletop).
  final Widget secondary;

  /// Single-column layout used on phones, cover screens, and narrow fallbacks.
  final Widget singlePane;

  /// Set to false for screens (such as Settings or terminal Done/Error stages)
  /// that stay single-column on flat tablets and only split around a physical
  /// hinge.
  final bool allowFlatTwoPane;

  /// Set to false for form-heavy screens (such as Settings) whose interactive
  /// fields belong in a single scrollable column rather than split across a
  /// horizontal fold.
  final bool allowTabletop;

  /// True only while an enclosing [AdaptivePaneLayout] or [AdaptiveSinglePane]
  /// is actively split in tabletop posture (false once collapsed to single
  /// pane, for example when the soft keyboard opens).
  static bool isTabletopOf(BuildContext context) =>
      _AdaptivePaneScope.maybeOf(context)?.activeTabletop ?? false;

  @override
  Widget build(BuildContext context) {
    final windowSize = MediaQuery.sizeOf(context);
    final displayFeatures = MediaQuery.displayFeaturesOf(context);
    final textScaler = MediaQuery.textScalerOf(context);
    final direction = Directionality.of(context);
    // DisplayFeatures are reported in root window coordinates; evaluate
    // physical posture against the window so a tabletop keyboard resize
    // collapses to singlePane rather than falling into a flat side-by-side
    // split across a horizontal crease.
    final physical = AdaptiveMetrics.fromViewport(
      size: windowSize,
      displayFeatures: displayFeatures,
      textScaler: textScaler,
    );

    return _OriginAwareBuilder(
      builder: (context, constraints, origin) {
        if (physical.posture == FoldPosture.book &&
            physical.hingeBounds != null) {
          final hinge = physical.hingeBounds!;
          final split = _splitAroundHinge(
            extent: constraints.maxWidth,
            hingeStart: hinge.left,
            hingeEnd: hinge.right,
            origin: origin.dx,
            rtl: direction == TextDirection.rtl,
          );
          if (split.start >= minHorizontalPaneWidth &&
              split.end >= minHorizontalPaneWidth) {
            return _AdaptivePaneScope(
              inSplitPane: true,
              activeTabletop: false,
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  SizedBox(width: split.start, child: primary),
                  if (split.hinge > 0) SizedBox(width: split.hinge),
                  SizedBox(width: split.end, child: secondary),
                ],
              ),
            );
          }
          if (math.max(split.start, split.end) > 0) {
            return _singleOnWiderSide(split: split, child: singlePane);
          }
        }

        if (allowTabletop &&
            physical.posture == FoldPosture.tabletop &&
            physical.hingeBounds != null) {
          final hinge = physical.hingeBounds!;
          final split = _splitAroundHinge(
            extent: constraints.maxHeight,
            hingeStart: hinge.top,
            hingeEnd: hinge.bottom,
            origin: origin.dy,
          );
          if (split.start >= minVerticalPaneHeight &&
              split.end >= minVerticalPaneHeight) {
            return _AdaptivePaneScope(
              inSplitPane: true,
              activeTabletop: true,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  SizedBox(height: split.start, child: primary),
                  if (split.hinge > 0) SizedBox(height: split.hinge),
                  SizedBox(height: split.end, child: secondary),
                ],
              ),
            );
          }
          if (math.max(split.start, split.end) > 0) {
            return _singleOnTallerHalf(split: split, child: singlePane);
          }
        }

        if (allowFlatTwoPane && physical.posture == FoldPosture.flat) {
          final viewport = Size(
            constraints.maxWidth.isFinite
                ? constraints.maxWidth
                : windowSize.width,
            constraints.maxHeight.isFinite
                ? constraints.maxHeight
                : windowSize.height,
          );
          final viewportMetrics = AdaptiveMetrics.fromViewport(
            size: viewport,
            displayFeatures: const [],
            textScaler: textScaler,
          );
          if (viewportMetrics.useTwoPane) {
            return _AdaptivePaneScope(
              inSplitPane: true,
              activeTabletop: false,
              child: Center(
                child: ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: maxTwoPaneWidth),
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Expanded(child: primary),
                      const VerticalDivider(width: 1),
                      Expanded(child: secondary),
                    ],
                  ),
                ),
              ),
            );
          }
        }

        return _AdaptivePaneScope(
          inSplitPane: false,
          activeTabletop: false,
          child: singlePane,
        );
      },
    );
  }
}
