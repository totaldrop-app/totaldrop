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
    ErrorKind.server => l.errorServer,
    ErrorKind.connect => l.errorConnect,
    ErrorKind.invalidOffer => safeL10n(context, 'errorInvalidOffer').errorInvalidOffer,
    ErrorKind.storage => l.errorStorage,
    ErrorKind.other => l.errorOther,
  };
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
                Icon(Icons.verified_user_outlined, color: theme.colorScheme.primary),
                Gap.s,
                Text(
                  safeL10n(context, 'verificationTitle').verificationTitle,
                  style: theme.textTheme.titleMedium,
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
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Icon(
          wrongCode ? Icons.gpp_bad_outlined : Icons.error_outline,
          size: 56,
          color: theme.colorScheme.error,
        ),
        Gap.m,
        Text(
          errorMessage(context, error.kind),
          textAlign: TextAlign.center,
          style: theme.textTheme.titleMedium,
        ),
        if (error.detail.isNotEmpty) ...[
          Gap.s,
          ExpansionTile(
            title: Text(l.moreInfo, style: theme.textTheme.bodyMedium),
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
                // May hold the other side's cancel reason: hidden
                // characters are shown, not obeyed.
                child: SelectableText(
                  revealControls(error.detail),
                  style: theme.textTheme.bodySmall,
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
            style: theme.textTheme.bodySmall?.copyWith(color: theme.colorScheme.onSurfaceVariant),
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
  Widget build(BuildContext context) => const TotalDropMark(size: 140, done: true);
}
