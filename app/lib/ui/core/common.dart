// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:material_ui/material_ui.dart';

import '../../l10n/app_localizations.dart';
import '../../src/rust/api/transfer.dart';
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
    ErrorKind.server => l.errorServer,
    ErrorKind.connect => l.errorConnect,
    ErrorKind.invalidOffer => safeL10n(
      context,
      'errorInvalidOffer',
    ).errorInvalidOffer,
    ErrorKind.storage => l.errorStorage,
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
class Announced extends StatelessWidget {
  const Announced(this.child, {super.key});
  final Widget child;

  @override
  Widget build(BuildContext context) =>
      Semantics(liveRegion: true, child: child);
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
            Row(
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

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final wrongCode = error.kind == ErrorKind.wrongCode;
    final cause = causeMessage(context, error.cause);
    return Column(
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
        if (error.detail.isNotEmpty) ...[
          Gap.s,
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
        ],
        Gap.l,
        FilledButton(onPressed: onRetry, child: Text(l.tryAgain)),
      ],
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
          child: LinearProgressIndicator(value: fraction, minHeight: 10),
        ),
        Gap.s,
        Row(
          children: [
            Text(
              fraction == null ? '' : '${(fraction * 100).round()}%',
              style: theme.textTheme.titleMedium,
            ),
            const Spacer(),
            Text(
              '${formatBytes(l, done)} / ${formatBytes(l, total)}',
              style: theme.textTheme.bodyMedium,
            ),
          ],
        ),
      ],
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
  Widget build(BuildContext context) =>
      const TotalDropMark(size: 140, done: true);
}
