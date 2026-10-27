import 'package:material_ui/material_ui.dart';

import '../../l10n/app_localizations.dart';
import '../../src/rust/api/transfer.dart';
import 'mark.dart';
import 'theme.dart';

/// "3.5 MB", in the person's locale.
String formatBytes(int bytes, String locale) {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  var value = bytes.toDouble();
  var unit = 0;
  while (value >= 1000 && unit < units.length - 1) {
    value /= 1000;
    unit++;
  }
  final text = unit == 0 ? bytes.toString() : value.toStringAsFixed(value < 10 ? 1 : 0);
  final localized = locale.startsWith('tr') ? text.replaceAll('.', ',') : text;
  return '$localized ${units[unit]}';
}

String errorMessage(AppLocalizations l, ErrorKind kind) => switch (kind) {
  ErrorKind.wrongCode => l.errorWrongCode,
  ErrorKind.peerLeft => l.errorPeerLeft,
  ErrorKind.declined => l.errorDeclined,
  ErrorKind.cancelled => l.errorCancelled,
  ErrorKind.invalidCode => l.errorInvalidCode,
  ErrorKind.server => l.errorServer,
  ErrorKind.connect => l.errorConnect,
  ErrorKind.invalidOffer => l.errorInvalidOffer,
  ErrorKind.storage => l.errorStorage,
  ErrorKind.other => l.errorOther,
};

/// The four verification words, with the instruction to compare them.
class VerificationWords extends StatelessWidget {
  const VerificationWords({super.key, required this.words});

  final List<String> words;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
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
                Text(l.verificationTitle, style: theme.textTheme.titleMedium),
              ],
            ),
            Gap.m,
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                for (final w in words)
                  Chip(
                    label: Text(w, style: theme.textTheme.titleMedium),
                    side: BorderSide.none,
                    backgroundColor: theme.colorScheme.secondaryContainer,
                  ),
              ],
            ),
            Gap.m,
            Text(
              l.verificationHint,
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
          errorMessage(l, error.kind),
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
                child: SelectableText(error.detail, style: theme.textTheme.bodySmall),
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
    final theme = Theme.of(context);
    final locale = Localizations.localeOf(context).languageCode;
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
              '${formatBytes(done, locale)} / ${formatBytes(total, locale)}',
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

/// The resting mark with a check: the file has arrived.
class DoneBadge extends StatelessWidget {
  const DoneBadge({super.key, required this.dark});

  final bool dark;

  @override
  Widget build(BuildContext context) {
    return SizedBox.square(
      dimension: 140,
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          const TotalDropMark(size: 140),
          Positioned(
            right: 4,
            bottom: 8,
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.surface,
                shape: BoxShape.circle,
              ),
              child: Icon(
                Icons.check_circle,
                size: 44,
                color: dark ? Brand.successDark : Brand.success,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
