import 'package:material_ui/material_ui.dart';

import '../../l10n/app_localizations.dart';
import '../core/mark.dart';
import '../core/theme.dart';

class HomeView extends StatelessWidget {
  const HomeView({
    super.key,
    required this.onSend,
    required this.onReceive,
    required this.onSettings,
  });

  final VoidCallback onSend;
  final VoidCallback onReceive;
  final VoidCallback onSettings;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    return Scaffold(
      appBar: AppBar(
        actions: [
          IconButton(
            tooltip: l.settings,
            icon: const Icon(Icons.settings_outlined),
            onPressed: onSettings,
          ),
        ],
      ),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.fromLTRB(24, 0, 24, 24),
          children: [
            const Align(
              alignment: AlignmentDirectional.centerStart,
              child: TotalDropMark(size: 88),
            ),
            Gap.m,
            Text('TotalDrop', style: theme.textTheme.displaySmall),
            Text(
              l.appTagline,
              style: theme.textTheme.bodyLarge?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            Gap.xl,
            _ActionCard(
              icon: Icons.north_east_rounded,
              title: l.homeSendTitle,
              body: l.homeSendBody,
              background: theme.colorScheme.primary,
              foreground: theme.colorScheme.onPrimary,
              onTap: onSend,
            ),
            Gap.m,
            _ActionCard(
              icon: Icons.south_west_rounded,
              title: l.homeReceiveTitle,
              body: l.homeReceiveBody,
              // One accent per screen: the primary action carries it; receiving
              // sits on a quiet surface.
              background: theme.colorScheme.surfaceContainerHigh,
              foreground: theme.colorScheme.onSurface,
              onTap: onReceive,
            ),
            Gap.xl,
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Icon(
                  Icons.lock_outline,
                  size: 20,
                  color: theme.colorScheme.onSurfaceVariant,
                ),
                Gap.s,
                Expanded(
                  child: Text(
                    l.homeFooter,
                    style: theme.textTheme.bodyMedium?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _ActionCard extends StatelessWidget {
  const _ActionCard({
    required this.icon,
    required this.title,
    required this.body,
    required this.background,
    required this.foreground,
    required this.onTap,
  });

  final IconData icon;
  final String title;
  final String body;
  final Color background;
  final Color foreground;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card(
      color: background,
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Row(
            children: [
              Container(
                width: 64,
                height: 64,
                decoration: BoxDecoration(
                  color: foreground.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(20),
                ),
                child: Icon(icon, size: 32, color: foreground),
              ),
              Gap.m,
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      title,
                      style: theme.textTheme.headlineMedium?.copyWith(
                        color: foreground,
                      ),
                    ),
                    Text(
                      body,
                      style: theme.textTheme.bodyLarge?.copyWith(
                        color: foreground,
                      ),
                    ),
                  ],
                ),
              ),
              Icon(Icons.chevron_right, color: foreground),
            ],
          ),
        ),
      ),
    );
  }
}
