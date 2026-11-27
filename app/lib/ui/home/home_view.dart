// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:material_ui/material_ui.dart';

import '../../l10n/app_localizations.dart';
import '../core/adaptive.dart';
import '../core/common.dart';
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

    Widget buildBrand({required bool compact}) => Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Align(
          alignment: AlignmentDirectional.centerStart,
          child: TotalDropMark(size: compact ? 72 : 88),
        ),
        Gap.m,
        FittedBox(
          fit: BoxFit.scaleDown,
          alignment: AlignmentDirectional.centerStart,
          child: Text('TotalDrop', style: theme.textTheme.displaySmall),
        ),
        Text(
          l.appTagline,
          style: theme.textTheme.bodyLarge?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
      ],
    );

    final sendCard = _ActionCard(
      icon: Icons.north_east_rounded,
      title: l.homeSendTitle,
      body: l.homeSendBody,
      background: theme.colorScheme.primary,
      foreground: theme.colorScheme.onPrimary,
      onTap: onSend,
    );

    final receiveCard = _ActionCard(
      icon: Icons.south_west_rounded,
      title: l.homeReceiveTitle,
      body: l.homeReceiveBody,
      // One accent per screen: the primary action carries it; receiving
      // sits on a quiet surface.
      background: theme.colorScheme.surfaceContainerHigh,
      foreground: theme.colorScheme.onSurface,
      onTap: onReceive,
    );

    final footer = Row(
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
    );

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
        child: AdaptivePaneLayout(
          primary: Builder(
            builder: (context) {
              final isTabletop = AdaptivePaneLayout.isTabletopOf(context);
              return CenteredScroll(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    buildBrand(compact: isTabletop),
                    if (!isTabletop) ...[Gap.xl, footer],
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
                    sendCard,
                    Gap.m,
                    receiveCard,
                    if (isTabletop) ...[Gap.l, footer],
                  ],
                ),
              );
            },
          ),
          singlePane: LayoutBuilder(
            builder: (context, constraints) => ColumnListView(
              top: 0,
              bottom: 24,
              children: [
                buildBrand(
                  compact: constraints.maxWidth < coverScreenThreshold,
                ),
                Gap.xl,
                sendCard,
                Gap.m,
                receiveCard,
                Gap.xl,
                footer,
              ],
            ),
          ),
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
    return LayoutBuilder(
      builder: (context, constraints) {
        // Standard 360 dp phones give 312 dp card width; tighter cover screens
        // and narrow split panes use compact card chrome so labels keep room.
        final compact = constraints.maxWidth < 300;
        final largeText = MediaQuery.textScalerOf(context).scale(16) >= 24;
        final boxSize = compact ? 48.0 : 64.0;
        final iconSize = compact ? 24.0 : 32.0;
        final cardPad = compact ? 16.0 : 24.0;
        final boxRadius = compact ? 16.0 : 20.0;
        final badge = Container(
          width: boxSize,
          height: boxSize,
          decoration: BoxDecoration(
            color: foreground.withValues(alpha: 0.12),
            borderRadius: BorderRadius.circular(boxRadius),
          ),
          child: Icon(icon, size: iconSize, color: foreground),
        );
        final copy = Column(
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
              style: theme.textTheme.bodyLarge?.copyWith(color: foreground),
            ),
          ],
        );
        final chevron = Icon(Icons.chevron_right, color: foreground);
        return Card(
          color: background,
          clipBehavior: Clip.antiAlias,
          child: InkWell(
            onTap: onTap,
            child: Padding(
              padding: EdgeInsets.all(cardPad),
              child: compact && largeText
                  ? Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [badge, chevron],
                        ),
                        Gap.s,
                        copy,
                      ],
                    )
                  : Row(
                      children: [
                        badge,
                        Gap.m,
                        Expanded(child: copy),
                        chevron,
                      ],
                    ),
            ),
          ),
        );
      },
    );
  }
}
