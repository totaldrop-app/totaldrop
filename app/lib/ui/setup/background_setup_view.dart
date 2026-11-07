// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:material_ui/material_ui.dart';

import '../../data/background_access.dart';
import '../../l10n/app_localizations.dart';
import '../core/theme.dart';

/// Asks the person to lift the battery restrictions that stop a transfer
/// once TotalDrop leaves the screen (#338). Shown once at first start, once
/// more after the first transfer that went to the background while they
/// were still on, and from Settings.
///
/// Android does not let an app ask at install time, and Google Play allows
/// the direct "ignore battery optimisation" dialog only for a few kinds of
/// app, so this page opens the system's settings and the person decides
/// there.
class BackgroundSetupView extends StatefulWidget {
  const BackgroundSetupView({
    super.key,
    required this.access,
    required this.onDone,
    this.firstRun = false,
  });

  final BackgroundAccess access;

  /// [skipped] is true for "Not now".
  final void Function({required bool skipped}) onDone;

  /// At first start: no way back, and a "Not now" next to "Done".
  final bool firstRun;

  @override
  State<BackgroundSetupView> createState() => _BackgroundSetupViewState();
}

class _BackgroundSetupViewState extends State<BackgroundSetupView>
    with WidgetsBindingObserver {
  BackgroundState? _state;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _refresh();
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  /// Back from the system's settings: show what the person chose there.
  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) _refresh();
  }

  Future<void> _refresh() async {
    final s = await widget.access.state();
    if (mounted) setState(() => _state = s);
  }

  Future<void> _open(BackgroundPage page) async {
    await widget.access.open(page);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final state = _state;
    final muted = theme.textTheme.bodyMedium?.copyWith(
      color: theme.colorScheme.onSurfaceVariant,
    );
    return PopScope(
      canPop: !widget.firstRun,
      child: Scaffold(
        appBar: AppBar(automaticallyImplyLeading: !widget.firstRun),
        body: SafeArea(
          child: ListView(
            padding: const EdgeInsets.fromLTRB(24, 0, 24, 24),
            children: [
              Icon(
                Icons.battery_charging_full_rounded,
                size: 48,
                color: theme.colorScheme.primary,
              ),
              Gap.m,
              Semantics(
                header: true,
                child: Text(
                  l.setupBackgroundTitle,
                  style: theme.textTheme.headlineMedium,
                ),
              ),
              Gap.s,
              Text(l.setupBackgroundBody, style: theme.textTheme.bodyLarge),
              Gap.l,
              if (state != null) ...[
                Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Icon(
                      state.exempt
                          ? Icons.check_circle_outline_rounded
                          : Icons.info_outline_rounded,
                      size: 20,
                      color: state.exempt
                          ? theme.colorScheme.primary
                          : theme.colorScheme.onSurfaceVariant,
                    ),
                    Gap.s,
                    Expanded(
                      child: Text(
                        state.exempt
                            ? l.setupBackgroundAllowed
                            : l.setupBackgroundRestricted,
                        style: muted,
                      ),
                    ),
                  ],
                ),
                Gap.m,
              ],
              FilledButton(
                onPressed: () => _open(BackgroundPage.battery),
                child: Text(l.setupBackgroundBattery),
              ),
              Gap.s,
              Text(l.setupBackgroundBatteryHint, style: muted),
              if (state?.maker case final maker?) ...[
                Gap.l,
                const Divider(),
                Gap.l,
                Semantics(
                  header: true,
                  child: Text(
                    l.setupBackgroundMaker(maker),
                    style: theme.textTheme.titleLarge,
                  ),
                ),
                Gap.s,
                Text(
                  l.setupBackgroundMakerBody,
                  style: theme.textTheme.bodyMedium,
                ),
                Gap.m,
                FilledButton.tonal(
                  onPressed: () => _open(BackgroundPage.maker),
                  child: Text(l.setupBackgroundMakerOpen),
                ),
                if (state?.autostart ?? false) ...[
                  Gap.s,
                  OutlinedButton(
                    onPressed: () => _open(BackgroundPage.autostart),
                    child: Text(l.setupBackgroundAutostart),
                  ),
                ],
              ],
              Gap.xl,
              Wrap(
                alignment: WrapAlignment.end,
                spacing: 8,
                runSpacing: 8,
                children: [
                  if (widget.firstRun)
                    TextButton(
                      onPressed: () => widget.onDone(skipped: true),
                      child: Text(l.setupBackgroundSkip),
                    ),
                  FilledButton.tonal(
                    onPressed: () => widget.onDone(skipped: false),
                    child: Text(l.done),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
