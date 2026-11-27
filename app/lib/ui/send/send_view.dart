// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'dart:math' as math;
import 'dart:ui' show lerpDouble;

import 'package:flutter/foundation.dart' show clampDouble, visibleForTesting;
import 'package:material_ui/material_ui.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter/services.dart';
import 'package:share_plus/share_plus.dart';

import '../../l10n/app_localizations.dart';
import '../core/adaptive.dart';
import '../core/common.dart';
import '../core/l10n.dart';
import '../core/mark.dart';
import '../core/theme.dart';
import 'send_view_model.dart';

class SendView extends StatelessWidget {
  const SendView({super.key, required this.viewModel});

  final SendViewModel viewModel;

  Future<bool> _confirmStop(BuildContext context) async {
    final l = AppLocalizations.of(context);
    final stop = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(l.cancelTransferTitle),
        content: Text(l.cancelTransferBody),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(l.keepGoing),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(l.stop),
          ),
        ],
      ),
    );
    return stop ?? false;
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    return ListenableBuilder(
      listenable: viewModel,
      builder: (context, _) {
        final vm = viewModel;
        return PopScope(
          canPop: !vm.busy,
          onPopInvokedWithResult: (didPop, _) async {
            if (didPop) {
              if (vm.stage != SendStage.picking) {
                vm.reset();
              } else {
                // Leaving the wait ends it: the shares stop, and what
                // was chosen before stays. A text being written goes.
                vm.cancelShares();
                vm.leaveText();
              }
              return;
            }
            if (await _confirmStop(context)) {
              vm.reset();
              if (context.mounted) Navigator.pop(context);
            }
          },
          child: Scaffold(
            appBar: AppBar(title: Text(vm.textMode ? l.sendText : l.sendTitle)),
            body: SafeArea(
              child: AnimatedSwitcher(
                duration: const Duration(milliseconds: 250),
                child: KeyedSubtree(
                  key: ValueKey((vm.stage, vm.copying || vm.sharing)),
                  child: switch (vm.stage) {
                    // The picker copies the chosen files into the app
                    // cache before it hands them over (#269).
                    SendStage.picking when vm.copying || vm.sharing =>
                      _Centered(
                        logo: const TotalDropMark(size: 120, sending: true),
                        text: l.sendPreparing,
                        // A share's copy can be stopped; the picker's
                        // belongs to the picker.
                        action: vm.sharing && !vm.copying
                            ? TextButton(
                                onPressed: vm.cancelShares,
                                child: Text(l.cancel),
                              )
                            : null,
                      ),
                    SendStage.picking when vm.textMode => _TextPicking(vm: vm),
                    SendStage.picking => _Picking(vm: vm),
                    SendStage.preparing => _Centered(
                      logo: const TotalDropMark(size: 120, sending: true),
                      text: l.sendPreparing,
                    ),
                    SendStage.waiting => _Waiting(vm: vm),
                    SendStage.connected => _Connected(vm: vm),
                    SendStage.transferring => _Transferring(vm: vm),
                    SendStage.done => _Done(vm: vm),
                    SendStage.failed => AdaptiveErrorStage(
                      error: vm.error!,
                      onRetry: vm.start,
                    ),
                  },
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}

class _Centered extends StatelessWidget {
  const _Centered({required this.logo, required this.text, this.action});
  final Widget logo;
  final String text;
  final Widget? action;

  @override
  Widget build(BuildContext context) => CenteredScroll(
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        logo,
        Gap.l,
        Announced(
          Text(
            text,
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ),
        if (action case final a?) ...[Gap.l, a],
      ],
    ),
  );
}

/// Default M3 icon size in `material_ui 1.11.1` (`_SegmentedButtonDefaultsM3`
/// in `lib/src/generated/segmented_button_defaults_m3.g.dart:82`).
const _kSegmentedDefaultIconSize = 18.0;

/// Base font size divisor used by `material_ui 1.11.1` (`SegmentedButton` in
/// `lib/src/segmented_button.dart:600-603`, matching `_TextButtonWithIconChild`
/// in `lib/src/text_button.dart:480-483`):
/// `MediaQuery.textScalerOf(context).scale(defaultFontSize) / 14.0`.
const _kSegmentedBaseFontSize = 14.0;

/// Fallback minimum segment height when the theme omits `minimumSize`.
const _kSegmentedFallbackMinHeight = 48.0;

/// M3 `SegmentedButton` icon-segment padding before text-scale scaling in
/// `material_ui 1.11.1` (`lib/src/segmented_button.dart:604-611`).
const _kSegmentedPadding1x = EdgeInsetsDirectional.fromSTEB(12, 8, 16, 8);
const _kSegmentedPaddingScaled = EdgeInsets.symmetric(horizontal: 4);

/// M3 `SegmentedButton` gap between icon and label in `material_ui 1.11.1`
/// (`lib/src/segmented_button.dart:615-623`: `lerpDouble(8, 4, scale)`).
const _kSegmentedIconGap1x = 8.0;
const _kSegmentedIconGap2x = 4.0;

/// Two equal segments share outer left, middle divider, and outer right borders.
const _kSegmentedBordersPerSegment = 1.5;

/// Maximum label lines permitted inside each segment before falling back to
/// stacked full-width buttons.
const _kSegmentedMaxLines = 2;

/// Measured layout result for two equal `SegmentedButton` segments.
typedef _SegmentedLayoutMetrics = ({bool fits, double minHeight});

_SegmentedLayoutMetrics _measureSegmentedLayout({
  required ThemeData theme,
  required TextScaler textScaler,
  required TextDirection textDirection,
  required double availableWidth,
  required List<String> labels,
}) {
  final style = theme.segmentedButtonTheme.style;
  final textStyle =
      style?.textStyle?.resolve(const {}) ?? theme.textTheme.labelLarge;
  final defaultFontSize = textStyle?.fontSize ?? _kSegmentedBaseFontSize;
  // Matches `material_ui 1.11.1` `lib/src/segmented_button.dart:600-603`
  // verbatim (even when the theme's `labelLarge` is 15 sp, giving ~1.07 at
  // 1.0x). Guarded against future package drift by the `RenderParagraph`
  // threshold assertions in `app/test/responsive_polish_test.dart`.
  final effectiveTextScale =
      textScaler.scale(defaultFontSize) / _kSegmentedBaseFontSize;
  final scaledPadding = ButtonStyleButton.scaledPadding(
    _kSegmentedPadding1x,
    _kSegmentedPaddingScaled,
    _kSegmentedPaddingScaled,
    effectiveTextScale,
  ).resolve(textDirection);
  final iconSize =
      style?.iconSize?.resolve(const {}) ?? _kSegmentedDefaultIconSize;
  final scaleT = clampDouble(effectiveTextScale, 1.0, 2.0) - 1.0;
  final iconGap = lerpDouble(
    _kSegmentedIconGap1x,
    _kSegmentedIconGap2x,
    scaleT,
  )!;
  final borderWidth = style?.side?.resolve(const {})?.width ?? 1.0;
  final segmentChrome =
      scaledPadding.horizontal +
      iconSize +
      iconGap +
      borderWidth * _kSegmentedBordersPerSegment;
  final maxLabelWidth = (availableWidth / 2) - segmentChrome;
  final minHeight =
      style?.minimumSize?.resolve(const {})?.height ??
      _kSegmentedFallbackMinHeight;
  if (maxLabelWidth <= 0) return (fits: false, minHeight: minHeight);

  var maxRenderedLabelHeight = 0.0;
  for (final label in labels) {
    // For space-delimited scripts, require each word to fit so Flutter never
    // chops a word mid-word inside a segment.
    if (label.contains(RegExp(r'\s'))) {
      for (final word in label.split(RegExp(r'\s+'))) {
        if (word.isEmpty) continue;
        final wordPainter = TextPainter(
          text: TextSpan(text: word, style: textStyle),
          textDirection: textDirection,
          textScaler: textScaler,
          maxLines: 1,
        )..layout();
        final wordWidth = wordPainter.width;
        wordPainter.dispose();
        if (wordWidth > maxLabelWidth) {
          return (fits: false, minHeight: minHeight);
        }
      }
    }

    final painter = TextPainter(
      text: TextSpan(text: label, style: textStyle),
      textDirection: textDirection,
      textScaler: textScaler,
      maxLines: _kSegmentedMaxLines,
    )..layout(maxWidth: maxLabelWidth);
    final labelHeight = painter.height;
    // Two-line wrapping is intentional only near 1.0x so long translations
    // stay on `SegmentedButton` without inflating segments into oversized
    // double-height pills at large accessibility scales (where stacked
    // full-width single-line buttons are easier to scan and tap).
    final fits = !painter.didExceedMaxLines && labelHeight <= minHeight;
    painter.dispose();
    if (!fits) return (fits: false, minHeight: minHeight);
    maxRenderedLabelHeight = math.max(maxRenderedLabelHeight, labelHeight);
  }

  // `material_ui 1.11.1`'s `_RenderSegmentedButton.computeMaxIntrinsicHeight`
  // (`lib/src/segmented_button.dart:913-923`) passes the full width rather
  // than `width / childCount` to each segment, so a label that wraps to 2
  // lines under half width would under-report its intrinsic height to
  // `SliverFillRemaining(hasScrollBody: false)`. Reporting the wrapped minimum
  // height prevents bottom overflow near the viewport fold.
  final expectedHeight = math.max(
    minHeight,
    maxRenderedLabelHeight + scaledPadding.vertical + borderWidth * 2,
  );
  return (fits: true, minHeight: expectedHeight);
}

/// Returns whether two equal `SegmentedButton` segments inside [availableWidth]
/// can lay out [labels] using [theme]'s `segmentedButtonTheme` metrics without
/// mid-word breaks, exceeding [_kSegmentedMaxLines], or overflowing the
/// segment's minimum height budget.
@visibleForTesting
bool sendModeSwitchFitsSegmented({
  required ThemeData theme,
  required TextScaler textScaler,
  required TextDirection textDirection,
  required double availableWidth,
  required List<String> labels,
}) => _measureSegmentedLayout(
  theme: theme,
  textScaler: textScaler,
  textDirection: textDirection,
  availableWidth: availableWidth,
  labels: labels,
).fits;

/// Switches between sending files and sending text while keeping any
/// picked files and typed text intact (#545, ADR 0011). Allows up to two
/// wrapped label lines inside each segment so long translations stay on
/// `SegmentedButton` at 1.0x, and falls back to stacked full-width buttons
/// when large text scales or narrow viewports exceed that budget.
class _SendModeSwitch extends StatelessWidget {
  const _SendModeSwitch({required this.vm, required this.availableWidth});
  final SendViewModel vm;
  final double availableWidth;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final metrics = _measureSegmentedLayout(
      theme: Theme.of(context),
      textScaler: MediaQuery.textScalerOf(context),
      textDirection: Directionality.of(context),
      availableWidth: availableWidth,
      labels: [l.sendTitle, l.sendText],
    );

    if (metrics.fits) {
      return ConstrainedBox(
        constraints: BoxConstraints(minHeight: metrics.minHeight),
        child: SegmentedButton<bool>(
          expandedInsets: EdgeInsets.zero,
          showSelectedIcon: false,
          selected: {vm.textMode},
          onSelectionChanged: (selected) => vm.setTextMode(selected.first),
          segments: [
            ButtonSegment<bool>(
              value: false,
              icon: const Icon(Icons.attach_file),
              label: Text(
                l.sendTitle,
                maxLines: _kSegmentedMaxLines,
                softWrap: true,
                textAlign: TextAlign.center,
              ),
            ),
            ButtonSegment<bool>(
              value: true,
              icon: const Icon(Icons.notes),
              label: Text(
                l.sendText,
                maxLines: _kSegmentedMaxLines,
                softWrap: true,
                textAlign: TextAlign.center,
              ),
            ),
          ],
        ),
      );
    }

    Widget modeButton({
      required bool selected,
      required IconData icon,
      required String label,
      required VoidCallback onPressed,
    }) {
      final button = selected
          ? FilledButton.tonalIcon(
              onPressed: onPressed,
              icon: Icon(icon),
              label: Text(label, textAlign: TextAlign.center),
            )
          : OutlinedButton.icon(
              onPressed: onPressed,
              icon: Icon(icon),
              label: Text(label, textAlign: TextAlign.center),
            );
      return MergeSemantics(
        child: Semantics(
          selected: selected,
          inMutuallyExclusiveGroup: true,
          child: button,
        ),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        modeButton(
          selected: !vm.textMode,
          icon: Icons.attach_file,
          label: l.sendTitle,
          onPressed: () => vm.setTextMode(false),
        ),
        Gap.s,
        modeButton(
          selected: vm.textMode,
          icon: Icons.notes,
          label: l.sendText,
          onPressed: () => vm.setTextMode(true),
        ),
      ],
    );
  }
}

class _Picking extends StatelessWidget {
  const _Picking({required this.vm});
  final SendViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    if (vm.files.isEmpty) {
      final prompt = Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Icon(
            Icons.upload_file_outlined,
            size: 56,
            color: theme.colorScheme.onSurfaceVariant,
          ),
          Gap.m,
          Text(
            l.sendEmpty,
            style: theme.textTheme.headlineSmall,
            textAlign: TextAlign.center,
          ),
          Gap.s,
          Text(
            l.sendEmptyHint,
            textAlign: TextAlign.center,
            style: theme.textTheme.bodyLarge?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
          ),
        ],
      );
      Widget buildActions(double availableWidth) => Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _SendModeSwitch(vm: vm, availableWidth: availableWidth),
          Gap.l,
          if (vm.addFailed) ...[
            const _AddFailed(textAlign: TextAlign.center),
            Gap.m,
          ],
          FilledButton.icon(
            onPressed: vm.addFiles,
            icon: const Icon(Icons.attach_file),
            label: Text(l.sendAddFiles),
          ),
        ],
      );
      // One screen, one job: choosing files (with the mode switch). With the
      // largest text the page scrolls instead of running off (#322, #545).
      return AdaptivePaneLayout(
        primary: CenteredScroll(child: prompt),
        secondary: CenteredScroll(
          child: LayoutBuilder(
            builder: (context, constraints) =>
                buildActions(constraints.maxWidth),
          ),
        ),
        singlePane: LayoutBuilder(
          builder: (context, constraints) {
            final pad = columnPadding(constraints.maxWidth, top: 8, bottom: 24);
            final innerWidth = math.max(
              0.0,
              constraints.maxWidth - pad.horizontal,
            );
            return CustomScrollView(
              slivers: [
                SliverFillRemaining(
                  hasScrollBody: false,
                  child: Padding(
                    padding: pad,
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        _SendModeSwitch(vm: vm, availableWidth: innerWidth),
                        // Minimum clearance when Spacer collapses to 0 at 200%.
                        Gap.m,
                        const Spacer(),
                        prompt,
                        const Spacer(),
                        // Minimum clearance when Spacer collapses to 0 at 200%.
                        Gap.m,
                        if (vm.addFailed) ...[
                          const _AddFailed(textAlign: TextAlign.center),
                          Gap.m,
                        ],
                        FilledButton.icon(
                          onPressed: vm.addFiles,
                          icon: const Icon(Icons.attach_file),
                          label: Text(l.sendAddFiles),
                        ),
                      ],
                    ),
                  ),
                ),
              ],
            );
          },
        ),
      );
    }

    final fileList = LayoutBuilder(
      builder: (context, constraints) {
        final listPad = columnPadding(constraints.maxWidth, top: 8, bottom: 8);
        final innerWidth = math.max(
          0.0,
          constraints.maxWidth - listPad.horizontal,
        );
        return ListView.separated(
          padding: listPad,
          itemCount: vm.files.length + 2,
          separatorBuilder: (_, i) => i == 0 ? Gap.s : const Divider(),
          itemBuilder: (context, i) {
            if (i == 0) {
              return _SendModeSwitch(vm: vm, availableWidth: innerWidth);
            }
            if (i == vm.files.length + 1) {
              return Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TextButton.icon(
                    onPressed: vm.addFiles,
                    icon: const Icon(Icons.add),
                    label: Text(l.sendAddFiles),
                  ),
                  if (vm.addFailed)
                    const Padding(
                      padding: EdgeInsetsDirectional.fromSTEB(12, 0, 12, 8),
                      child: _AddFailed(textAlign: TextAlign.start),
                    ),
                ],
              );
            }
            final f = vm.files[i - 1];
            return ListTile(
              leading: Icon(
                Icons.insert_drive_file_outlined,
                color: theme.colorScheme.onSurfaceVariant,
              ),
              title: Text(f.name, maxLines: 1, overflow: TextOverflow.ellipsis),
              subtitle: Text(
                formatBytes(l, f.size),
                style: const TextStyle(
                  fontFeatures: [FontFeature.tabularFigures()],
                ),
              ),
              trailing: KeepIconButtonVariant(
                child: IconButton(
                  tooltip: l.sendRemove,
                  icon: const Icon(Icons.close),
                  onPressed: () => vm.remove(f),
                ),
              ),
            );
          },
        );
      },
    );

    final controls = Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SwitchListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(l.sendStrongCode),
          subtitle: Text(l.sendStrongCodeHint),
          value: vm.strong,
          onChanged: vm.setStrong,
        ),
        Gap.s,
        FilledButton.icon(
          onPressed: vm.start,
          icon: const Icon(Icons.key_outlined),
          // Read out when the list appears after the picker's copy
          // and when it changes, so a screen reader is not left
          // silent where "Add files" was (#271).
          label: Announced(
            Text(
              '${l.sendCreateCode} · ${l.itemsAndSize(vm.files.length, formatBytes(l, vm.totalSize))}',
            ),
          ),
        ),
      ],
    );

    return AdaptivePaneLayout(
      primary: fileList,
      secondary: CenteredScroll(child: controls),
      singlePane: Column(
        children: [
          Expanded(child: fileList),
          const Divider(),
          // Pinned under the list. With the largest text it takes at most
          // half the screen and scrolls itself, so the list keeps its room
          // (#322).
          ConstrainedBox(
            constraints: BoxConstraints(
              maxHeight: MediaQuery.sizeOf(context).height / 2,
            ),
            child: LayoutBuilder(
              builder: (context, constraints) => SingleChildScrollView(
                padding: columnPadding(
                  constraints.maxWidth,
                  top: 8,
                  bottom: 24,
                ),
                child: controls,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// A text to send instead of files (#326): typed, pasted, shared from
/// another app or selected there. What is sent is the text in the box,
/// as UTF-8; a text too long to edit is shown in part and sent whole.
class _TextPicking extends StatefulWidget {
  const _TextPicking({required this.vm});
  final SendViewModel vm;

  @override
  State<_TextPicking> createState() => _TextPickingState();
}

class _TextPickingState extends State<_TextPicking> {
  late final _controller = TextEditingController(text: _boxText);
  // Shared across AdaptivePaneLayout branches so fold/unfold and tabletop
  // keyboard collapse preserve focus, IME composing text, and selection (#632).
  final _fieldKey = GlobalKey();
  final _focusNode = FocusNode();

  String get _boxText {
    final vm = widget.vm;
    return vm.bigText ? vm.text.substring(0, _cut(vm.text)) : vm.text;
  }

  /// Not inside a character made of two UTF-16 units.
  static int _cut(String t) {
    final end = shownTextChars;
    final last = t.codeUnitAt(end - 1);
    return last >= 0xD800 && last <= 0xDBFF ? end - 1 : end;
  }

  @override
  void didUpdateWidget(_TextPicking old) {
    super.didUpdateWidget(old);
    _sync();
  }

  @override
  void dispose() {
    _focusNode.dispose();
    _controller.dispose();
    super.dispose();
  }

  /// The box follows a text that came from elsewhere (clipboard, share).
  void _sync() {
    final want = _boxText;
    if (_controller.text != want) {
      _controller.value = TextEditingValue(
        text: want,
        selection: TextSelection.collapsed(offset: want.length),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final vm = widget.vm;
    final notice = switch (vm.textNotice) {
      TextNotice.clipboardEmpty => l.sendClipboardEmpty,
      TextNotice.clipboardKept => l.sendClipboardKept,
      TextNotice.none when vm.textTooLong => l.sendTextTooLong(
        formatBytes(l, vm.maxTextBytes),
      ),
      TextNotice.none when vm.bigText => l.sendTextShortened,
      TextNotice.none => null,
    };
    final warn = vm.textTooLong || vm.textNotice != TextNotice.none;
    // The box shows hidden characters as the system draws them, which is
    // often not at all: the screen names them (#326).
    final hidden = vm.text.isNotEmpty && shownText(vm.text).hidden;
    Widget buildEditor({required double minHeight}) => Expanded(
      child: ConstrainedBox(
        constraints: BoxConstraints(minHeight: minHeight),
        child: TextField(
          key: _fieldKey,
          focusNode: _focusNode,
          controller: _controller,
          // A long text cannot be edited, only taken away.
          readOnly: vm.bigText,
          autofocus: vm.text.isEmpty,
          maxLines: null,
          expands: true,
          textAlignVertical: TextAlignVertical.top,
          keyboardType: TextInputType.multiline,
          decoration: InputDecoration(
            hintText: l.sendTextHint,
            border: const OutlineInputBorder(),
          ),
          onChanged: vm.setText,
        ),
      ),
    );

    final notices = <Widget>[
      if (notice != null) ...[
        Gap.s,
        Announced(
          Text(
            notice,
            style: theme.textTheme.bodyMedium?.copyWith(
              color: warn
                  ? theme.colorScheme.error
                  : theme.colorScheme.onSurfaceVariant,
            ),
          ),
        ),
      ],
      if (hidden) ...[
        Gap.s,
        Announced(
          Text(
            l.sendTextHidden,
            style: theme.textTheme.bodyMedium?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
          ),
        ),
      ],
    ];

    final controls = Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Wrap(
          spacing: 8,
          runSpacing: 8,
          crossAxisAlignment: WrapCrossAlignment.center,
          children: [
            if (vm.text.isEmpty)
              OutlinedButton.icon(
                onPressed: vm.pasteClipboard,
                icon: const Icon(Icons.content_paste),
                label: Text(l.receivePaste),
              )
            else
              OutlinedButton.icon(
                onPressed: () => vm.setText(''),
                icon: const Icon(Icons.clear),
                label: Text(l.sendRemove),
              ),
          ],
        ),
        const Divider(height: 24),
        SwitchListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(l.sendStrongCode),
          subtitle: Text(l.sendStrongCodeHint),
          value: vm.strong,
          onChanged: vm.setStrong,
        ),
        Gap.s,
        FilledButton.icon(
          onPressed: vm.canStart ? vm.start : null,
          icon: const Icon(Icons.key_outlined),
          label: Text(
            vm.text.isEmpty
                ? l.sendCreateCode
                : '${l.sendCreateCode} · ${formatBytes(l, vm.textBytes)}',
          ),
        ),
      ],
    );

    // The box takes the room left; with the keyboard up or large type,
    // the page scrolls instead of overflowing.
    return AdaptivePaneLayout(
      primary: Builder(
        builder: (context) {
          final isTabletop = AdaptivePaneLayout.isTabletopOf(context);
          return LayoutBuilder(
            builder: (context, constraints) {
              final pad = columnPadding(
                constraints.maxWidth,
                top: 8,
                bottom: 16,
              );
              final innerWidth = math.max(
                0.0,
                constraints.maxWidth - pad.horizontal,
              );
              return CustomScrollView(
                slivers: [
                  SliverFillRemaining(
                    hasScrollBody: false,
                    child: Padding(
                      padding: pad,
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          _SendModeSwitch(vm: vm, availableWidth: innerWidth),
                          Gap.m,
                          buildEditor(minHeight: isTabletop ? 100 : 160),
                          ...notices,
                        ],
                      ),
                    ),
                  ),
                ],
              );
            },
          );
        },
      ),
      secondary: CenteredScroll(child: controls),
      singlePane: LayoutBuilder(
        builder: (context, constraints) {
          final pad = columnPadding(constraints.maxWidth, top: 8, bottom: 24);
          final innerWidth = math.max(
            0.0,
            constraints.maxWidth - pad.horizontal,
          );
          return CustomScrollView(
            slivers: [
              SliverFillRemaining(
                hasScrollBody: false,
                child: Padding(
                  padding: pad,
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      _SendModeSwitch(vm: vm, availableWidth: innerWidth),
                      Gap.m,
                      buildEditor(minHeight: 160),
                      ...notices,
                      Gap.s,
                      controls,
                    ],
                  ),
                ),
              ),
            ],
          );
        },
      ),
    );
  }
}

/// Why the picker came back with nothing (#271).
class _AddFailed extends StatelessWidget {
  const _AddFailed({required this.textAlign});
  final TextAlign textAlign;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Announced(
      Text(
        AppLocalizations.of(context).sendAddFailed,
        textAlign: textAlign,
        style: theme.textTheme.bodyMedium?.copyWith(
          color: theme.colorScheme.error,
        ),
      ),
    );
  }
}

class _Waiting extends StatelessWidget {
  const _Waiting({required this.vm});
  final SendViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final code = vm.code ?? '';
    Future<void> copy() async {
      await Clipboard.setData(ClipboardData(text: code));
      if (context.mounted) {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text(l.sendCopied)));
      }
    }

    // Isolated, the code keeps its order in a right-to-left sentence;
    // pasted back, the receiver drops the marks.
    Future<void> share() async {
      await SharePlus.instance.share(
        ShareParams(text: l.sendShareText(isolate(code))),
      );
    }

    Widget buildStatus({required bool isTabletop}) => Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Center(
          child: TotalDropMark(size: isTabletop ? 96 : 120, sending: true),
        ),
        Gap.s,
        Announced(
          Text(
            l.sendWaiting,
            textAlign: TextAlign.center,
            style: theme.textTheme.bodyMedium?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
          ),
        ),
      ],
    );

    // The code is the content; the frame stays quiet.
    final codeCard = LayoutBuilder(
      builder: (context, constraints) => Card(
        color: theme.colorScheme.surface,
        shape: RoundedRectangleBorder(
          borderRadius: const BorderRadius.all(Radius.circular(28)),
          side: BorderSide(color: theme.colorScheme.outlineVariant),
        ),
        child: Padding(
          padding: EdgeInsets.all(constraints.maxWidth < 300 ? 16 : 24),
          child: Column(
            children: [
              Text(
                l.sendYourCode,
                style: theme.textTheme.labelLarge?.copyWith(
                  color: theme.colorScheme.onSurfaceVariant,
                ),
              ),
              Gap.s,
              // English words, left to right in every language.
              // Spoken with an English voice; the Copy button below copies it.
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
                    style: theme.textTheme.headlineLarge,
                  ),
                ),
              ),
              Gap.m,
              ButtonPair(
                spacing: 8,
                outlined: PairedButton(l.sendCopy, copy, icon: Icons.copy),
                filled: PairedButton(l.sendShare, share, icon: Icons.share),
              ),
            ],
          ),
        ),
      ),
    );

    final codeHint = Text(
      l.sendCodeHint,
      textAlign: TextAlign.center,
      style: theme.textTheme.bodyLarge,
    );

    return AdaptivePaneLayout(
      primary: Builder(
        builder: (context) {
          final isTabletop = AdaptivePaneLayout.isTabletopOf(context);
          return CenteredScroll(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                buildStatus(isTabletop: isTabletop),
                if (!isTabletop) ...[Gap.xl, const SameNetworkNote()],
              ],
            ),
          );
        },
      ),
      secondary: Builder(
        builder: (context) {
          final isTabletop = AdaptivePaneLayout.isTabletopOf(context);
          return CenteredScroll(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                codeCard,
                Gap.m,
                codeHint,
                if (isTabletop) ...[Gap.l, const SameNetworkNote()],
              ],
            ),
          );
        },
      ),
      singlePane: ColumnListView(
        children: [
          buildStatus(isTabletop: false),
          Gap.l,
          codeCard,
          Gap.m,
          codeHint,
          Gap.xl,
          const SameNetworkNote(),
        ],
      ),
    );
  }
}

/// The receiver came and decides: the words to compare, nothing sent yet.
class _Connected extends StatelessWidget {
  const _Connected({required this.vm});
  final SendViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final waitingText = Announced(
      Text(
        l.sendWaitingAccept,
        textAlign: TextAlign.center,
        style: Theme.of(context).textTheme.bodyMedium
            ?.copyWith(color: Theme.of(context).colorScheme.onSurfaceVariant),
      ),
    );
    final words = VerificationWords(words: vm.words);

    return AdaptivePaneLayout(
      primary: Builder(
        builder: (context) {
          final isTabletop = AdaptivePaneLayout.isTabletopOf(context);
          return CenteredScroll(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Center(
                  child: TotalDropMark(
                    size: isTabletop ? 96 : 120,
                    sending: true,
                  ),
                ),
                Gap.m,
                waitingText,
              ],
            ),
          );
        },
      ),
      secondary: CenteredScroll(child: words),
      singlePane: ColumnListView(
        children: [
          const Center(child: TotalDropMark(size: 120, sending: true)),
          Gap.l,
          words,
          Gap.m,
          waitingText,
        ],
      ),
    );
  }
}

class _Transferring extends StatelessWidget {
  const _Transferring({required this.vm});
  final SendViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    Widget buildStatus({required bool isTabletop}) => Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Center(
          child: TotalDropMark(size: isTabletop ? 96 : 140, sending: true),
        ),
        Gap.l,
        // Keep the live-region headline separate from the sibling
        // TransferProgress bar inside two-pane layouts.
        Announced(
          Text(
            l.sendSending,
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          container: true,
        ),
        Gap.m,
        TransferProgress(done: vm.done, total: vm.total),
      ],
    );
    if (vm.words.isEmpty) {
      return AdaptiveSinglePane(
        child: Builder(
          builder: (context) => ColumnListView(
            children: [
              buildStatus(isTabletop: AdaptivePaneLayout.isTabletopOf(context)),
            ],
          ),
        ),
      );
    }
    final words = VerificationWords(words: vm.words);
    return AdaptivePaneLayout(
      primary: Builder(
        builder: (context) => CenteredScroll(
          child: buildStatus(
            isTabletop: AdaptivePaneLayout.isTabletopOf(context),
          ),
        ),
      ),
      secondary: CenteredScroll(child: words),
      singlePane: ColumnListView(
        children: [buildStatus(isTabletop: false), Gap.l, words],
      ),
    );
  }
}

class _Done extends StatelessWidget {
  const _Done({required this.vm});
  final SendViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final s = vm.summary;
    final header = <Widget>[
      const Center(child: DoneBadge()),
      Gap.l,
      Announced(
        Text(
          l.sendDone,
          textAlign: TextAlign.center,
          style: theme.textTheme.headlineSmall,
        ),
      ),
      Gap.s,
      if (vm.textMode)
        Text(l.sendTextDone, textAlign: TextAlign.center)
      else if (s != null) ...[
        Text(l.sendDoneBody(s.filesOk), textAlign: TextAlign.center),
        if (s.filesFailed > 0) ...[
          Gap.s,
          Text(
            l.sendSomeFailed(s.filesFailed),
            textAlign: TextAlign.center,
            style: TextStyle(color: theme.colorScheme.error),
          ),
        ],
      ],
    ];
    final doneButton = FilledButton(
      onPressed: () {
        vm.reset();
        Navigator.pop(context);
      },
      child: Text(l.done),
    );
    return AdaptivePaneLayout(
      allowFlatTwoPane: false,
      primary: CenteredScroll(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: header,
        ),
      ),
      secondary: CenteredScroll(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [doneButton],
        ),
      ),
      singlePane: CenteredScroll(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [...header, Gap.xl, doneButton],
        ),
      ),
    );
  }
}
