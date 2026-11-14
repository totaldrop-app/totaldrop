// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'dart:math' as math;

import 'package:material_ui/material_ui.dart';

import '../../l10n/app_localizations.dart';
import '../../src/rust/api/transfer.dart';
import 'adaptive.dart';
import 'l10n.dart';
import 'mark.dart';
import 'theme.dart';

String errorMessage(BuildContext context, ErrorKind kind) {
  final l = AppLocalizations.of(context);
  return switch (kind) {
    ErrorKind.wrongCode => safeL10n(context, 'errorWrongCode').errorWrongCode,
    ErrorKind.peerLeft => l.errorPeerLeft,
    ErrorKind.declined => l.errorDeclined,
    ErrorKind.cancelled => l.errorCancelled,
    ErrorKind.stopped => l.errorStopped,
    ErrorKind.invalidCode => l.errorInvalidCode,
    ErrorKind.codeNotInUse => l.errorCodeNotInUse,
    ErrorKind.codeExpired => l.errorCodeExpired,
    ErrorKind.badServer => l.errorBadServer,
    ErrorKind.rateLimited => l.errorRateLimited,
    ErrorKind.server => l.errorServer,
    ErrorKind.connect => l.errorConnect,
    ErrorKind.invalidOffer => safeL10n(
      context,
      'errorInvalidOffer',
    ).errorInvalidOffer,
    ErrorKind.storage => l.errorStorage,
    ErrorKind.textDamaged => l.errorTextDamaged,
    ErrorKind.other => l.errorOther,
  };
}

/// Why a transfer failed, in the person's language (#212), or null when
/// the headline says it all. [cause] is the core's name for it.
String? causeMessage(BuildContext context, String cause) =>
    causeText(AppLocalizations.of(context), cause);

/// The sentence for a cause name from the core (`Cause::as_str`), or null
/// when there is nothing more to say than the technical text.
String? causeText(AppLocalizations l, String cause) {
  return switch (cause) {
    'TimedOut' => l.causeTimedOut,
    'ConnectionLost' => l.causeConnectionLost,
    'NoRoute' => l.causeNoRoute,
    'Relay' => l.causeRelay,
    'Protocol' => l.causeProtocol,
    'DiskFull' => l.causeDiskFull,
    'NoPermission' => l.causeNoPermission,
    'NotFound' => l.causeNotFound,
    'ReadOnly' => l.causeReadOnly,
    'FileChanged' => l.causeFileChanged,
    _ => null,
  };
}

/// A stage headline that a screen reader reads out when it appears.
///
/// When a screen changes stage, the button that had focus goes away and
/// TalkBack says nothing; a live region makes it read the new headline
/// (#271). Only for text that changes with the stage, never for text that
/// changes by itself, such as progress, or it would talk without pause.
/// Pass [container]: true only where a sibling progress widget would otherwise
/// merge into the live region node.
class Announced extends StatelessWidget {
  const Announced(this.child, {super.key, this.container = false});
  final Widget child;
  final bool container;

  @override
  Widget build(BuildContext context) =>
      Semantics(container: container, liveRegion: true, child: child);
}

/// The four verification words, with the instruction to compare them.
class VerificationWords extends StatelessWidget {
  const VerificationWords({super.key, required this.words});

  final List<String> words;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    // A calm outlined frame: the words are the content, not the box.
    return Card(
      color: theme.colorScheme.surface,
      shape: RoundedRectangleBorder(
        borderRadius: const BorderRadius.all(Radius.circular(28)),
        side: BorderSide(color: theme.colorScheme.outlineVariant),
      ),
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // A heading on two lines keeps the icon beside its first line.
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Icon(
                  Icons.verified_user_outlined,
                  color: theme.colorScheme.primary,
                ),
                Gap.s,
                // Wraps on a narrow screen, larger text or a long language.
                Flexible(
                  child: Text(
                    safeL10n(context, 'verificationTitle').verificationTitle,
                    style: theme.textTheme.titleMedium,
                  ),
                ),
              ],
            ),
            Gap.m,
            // English words in the same order on both sides, whatever the
            // language's direction.
            Wrap(
              textDirection: TextDirection.ltr,
              spacing: 8,
              runSpacing: 8,
              children: [
                for (final w in words)
                  Chip(
                    label: LtrEnglish(w, style: theme.textTheme.titleMedium),
                    side: BorderSide.none,
                    backgroundColor: theme.colorScheme.secondaryContainer,
                  ),
              ],
            ),
            Gap.m,
            Text(
              safeL10n(context, 'verificationHint').verificationHint,
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// A full-width message for a failed transfer, with details on request.
class ErrorPanel extends StatelessWidget {
  const ErrorPanel({super.key, required this.error, required this.onRetry});

  final TransferError error;
  final VoidCallback onRetry;

  static Widget _header(BuildContext context, TransferError error) {
    final theme = Theme.of(context);
    final wrongCode = error.kind == ErrorKind.wrongCode;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Icon(
          wrongCode ? Icons.gpp_bad_outlined : Icons.error_outline,
          size: 56,
          color: theme.colorScheme.error,
        ),
        Gap.m,
        Announced(
          Text(
            errorMessage(context, error.kind),
            textAlign: TextAlign.center,
            style: theme.textTheme.titleMedium,
          ),
        ),
      ],
    );
  }

  static Widget _controls(
    BuildContext context,
    TransferError error,
    VoidCallback onRetry,
  ) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final cause = causeMessage(context, error.cause);
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (error.detail.isNotEmpty) ...[
          ExpansionTile(
            title: Text(l.moreInfo, style: theme.textTheme.bodyMedium),
            expandedCrossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Why, in the person's language; the technical text
                    // below stays English, for problem reports.
                    if (cause != null) ...[
                      Text(cause, style: theme.textTheme.bodyMedium),
                      Gap.m,
                    ],
                    Text(
                      l.technicalDetail,
                      style: theme.textTheme.labelMedium?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                    Gap.s,
                    // May hold the other side's cancel reason: hidden
                    // characters are shown, not obeyed.
                    // English, so left to right even in Arabic or Hebrew.
                    SelectableText(
                      revealControls(error.detail),
                      style: theme.textTheme.bodySmall,
                      textDirection: TextDirection.ltr,
                    ),
                  ],
                ),
              ),
            ],
          ),
          Gap.l,
        ],
        FilledButton(onPressed: onRetry, child: Text(l.tryAgain)),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _header(context, error),
        if (error.detail.isNotEmpty) Gap.s else Gap.l,
        _controls(context, error, onRetry),
      ],
    );
  }
}

/// Full-screen failed-transfer stage: keeps the error headline above the fold
/// and interactive details + Try again button below the fold in Tabletop
/// posture (or across the vertical hinge in Book posture), while staying a
/// centered single column on flat displays (#632).
class AdaptiveErrorStage extends StatelessWidget {
  const AdaptiveErrorStage({
    super.key,
    required this.error,
    required this.onRetry,
  });

  final TransferError error;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return AdaptivePaneLayout(
      allowFlatTwoPane: false,
      primary: CenteredScroll(child: ErrorPanel._header(context, error)),
      secondary: CenteredScroll(
        child: ErrorPanel._controls(context, error, onRetry),
      ),
      singlePane: CenteredScroll(
        child: ErrorPanel(error: error, onRetry: onRetry),
      ),
    );
  }
}

/// A determinate progress bar with "1.2 MB / 3.5 MB".
class TransferProgress extends StatelessWidget {
  const TransferProgress({super.key, required this.done, required this.total});

  final int done;
  final int total;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final fraction = total == 0 ? null : (done / total).clamp(0.0, 1.0);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        ClipRRect(
          borderRadius: BorderRadius.circular(8),
          child: LinearProgressIndicator(
            value: fraction,
            minHeight: 10,
            // The same whole percent as the text: the bar's own rounds up.
            semanticsValue: fraction == null
                ? null
                : '${wholePercent(done, total)}',
          ),
        ),
        Gap.s,
        // The sizes wrap under large text instead of running off the
        // screen (#322).
        Row(
          crossAxisAlignment: CrossAxisAlignment.baseline,
          textBaseline: TextBaseline.alphabetic,
          children: [
            Text(
              fraction == null ? '' : formatPercent(l, done, total),
              style: theme.textTheme.titleMedium,
            ),
            Gap.m,
            Expanded(
              child: Text(
                '${formatBytes(l, done)} / ${formatBytes(l, total)}',
                textAlign: TextAlign.end,
                style: theme.textTheme.bodyMedium,
              ),
            ),
          ],
        ),
      ],
    );
  }
}

/// Maximum width of a screen's main column on tablets, foldables and
/// wide landscape windows, matching Linux's `Theme.columnWidth` (#545).
const double maxColumnWidth = 560;

/// Equal horizontal padding that keeps at least [minHorizontal] on each
/// side (defaulting to [adaptiveHorizontalMargin]: 16 dp on narrow cover
/// displays under 360 dp, 24 dp on standard phones and wider screens) and
/// centres a [maxWidth] column on wider screens while keeping the full
/// viewport scrollable (#545, #632).
EdgeInsets columnPadding(
  double availableWidth, {
  double? minHorizontal,
  double top = 24,
  double bottom = 24,
  double maxWidth = maxColumnWidth,
}) {
  final margin = minHorizontal ?? adaptiveHorizontalMargin(availableWidth);
  final extra = math.max(0.0, (availableWidth - maxWidth - margin * 2) / 2);
  final h = margin + extra;
  return EdgeInsets.fromLTRB(h, top, h, bottom);
}

/// A scrollable screen column constrained to [maxWidth] on wide screens
/// while keeping full-viewport scroll gestures (#545, #632).
class ColumnListView extends StatelessWidget {
  const ColumnListView({
    super.key,
    required this.children,
    this.top = 24,
    this.bottom = 24,
    this.maxWidth = maxColumnWidth,
  });

  final List<Widget> children;
  final double top;
  final double bottom;
  final double maxWidth;

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) => ListView(
      padding: columnPadding(
        constraints.maxWidth,
        top: top,
        bottom: bottom,
        maxWidth: maxWidth,
      ),
      children: children,
    ),
  );
}

/// A screen's content, centred, that scrolls when it is taller than the
/// screen: with the largest text, a long language or "More information"
/// open, nothing runs off the bottom (#322). Keeps content within
/// [maxColumnWidth] on tablets and foldables while preserving full-viewport
/// scrolling and tighter margins on narrow cover displays (#545, #632).
class CenteredScroll extends StatelessWidget {
  const CenteredScroll({super.key, required this.child, this.padding});

  final Widget child;
  final EdgeInsets? padding;

  @override
  Widget build(BuildContext context) {
    return AdaptiveSinglePane(
      child: LayoutBuilder(
        builder: (context, constraints) {
          final effective =
              padding ??
              EdgeInsets.all(adaptiveHorizontalMargin(constraints.maxWidth));
          return SingleChildScrollView(
            padding: effective,
            child: ConstrainedBox(
              constraints: BoxConstraints(
                minHeight: math.max(
                  0,
                  constraints.maxHeight - effective.vertical,
                ),
              ),
              child: Center(
                child: ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: maxColumnWidth),
                  child: child,
                ),
              ),
            ),
          );
        },
      ),
    );
  }
}

/// One button of a [ButtonPair].
class PairedButton {
  const PairedButton(this.label, this.onPressed, {this.icon});

  final String label;
  final VoidCallback? onPressed;
  final IconData? icon;
}

/// An outlined and a filled button side by side, the same width; the
/// filled one above the other when a label would not fit on one line
/// beside the other. A label on two lines runs into the button's round
/// ends: 受け取る at 200 % in Japanese (#322).
class ButtonPair extends StatelessWidget {
  const ButtonPair({
    super.key,
    required this.outlined,
    required this.filled,
    this.spacing = 16,
  });

  final PairedButton outlined;
  final PairedButton filled;
  final double spacing;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final scaler = MediaQuery.textScalerOf(context);
    final direction = Directionality.of(context);

    // The label's width on one line, the widest padding a button takes
    // (24 on each side) and an icon with its gap.
    double width(PairedButton b, ButtonStyle? style) {
      final textStyle = style?.textStyle?.resolve(const {});
      final painter = TextPainter(
        text: TextSpan(
          text: b.label,
          style: textStyle ?? theme.textTheme.labelLarge,
        ),
        textDirection: direction,
        textScaler: scaler,
        maxLines: 1,
      )..layout();
      final label = painter.width;
      painter.dispose();
      return label + 48 + (b.icon == null ? 0 : 32);
    }

    Widget button(PairedButton b, {required bool filled}) {
      final label = Text(b.label, textAlign: TextAlign.center);
      final icon = b.icon;
      if (icon == null) {
        if (filled) return FilledButton(onPressed: b.onPressed, child: label);
        return OutlinedButton(onPressed: b.onPressed, child: label);
      }
      if (filled) {
        return FilledButton.icon(
          onPressed: b.onPressed,
          icon: Icon(icon),
          label: label,
        );
      }
      return OutlinedButton.icon(
        onPressed: b.onPressed,
        icon: Icon(icon),
        label: label,
      );
    }

    final needed = math.max(
      width(outlined, theme.outlinedButtonTheme.style),
      width(filled, theme.filledButtonTheme.style),
    );
    final first = button(outlined, filled: false);
    final second = button(filled, filled: true);
    return LayoutBuilder(
      builder: (context, constraints) {
        if ((constraints.maxWidth - spacing) / 2 >= needed) {
          return Row(
            children: [
              Expanded(child: first),
              SizedBox(width: spacing),
              Expanded(child: second),
            ],
          );
        }
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [second, Gap.s, first],
        );
      },
    );
  }
}

/// The honest note about the current limitation.
class SameNetworkNote extends StatelessWidget {
  const SameNetworkNote({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Icon(Icons.wifi, size: 20, color: theme.colorScheme.onSurfaceVariant),
        Gap.s,
        Expanded(
          child: Text(
            AppLocalizations.of(context).sameNetworkNote,
            style: theme.textTheme.bodySmall?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
          ),
        ),
      ],
    );
  }
}

/// The resting mark with a check inside its circle: the file has arrived.
class DoneBadge extends StatelessWidget {
  const DoneBadge({super.key});

  @override
  Widget build(BuildContext context) => TotalDropMark(
    size: AdaptivePaneLayout.isTabletopOf(context) ? 96 : 140,
    done: true,
  );
}
