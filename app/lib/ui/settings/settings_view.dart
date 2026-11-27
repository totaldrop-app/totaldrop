// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:material_ui/material_ui.dart';

import '../../data/background_access.dart';
import '../../data/settings_repository.dart';
import '../../l10n/app_localizations.dart';
import '../core/adaptive.dart';
import '../core/common.dart';
import '../core/l10n.dart';
import '../core/theme.dart';
import '../setup/background_setup_view.dart';

class SettingsView extends StatefulWidget {
  const SettingsView({
    super.key,
    required this.settings,
    required this.version,
    this.background = const NoBackgroundLimits(),
  });

  final SettingsRepository settings;
  final String version;
  final BackgroundAccess background;

  @override
  State<SettingsView> createState() => _SettingsViewState();
}

class _SettingsViewState extends State<SettingsView> {
  late final _server = TextEditingController(text: widget.settings.server);
  final _serverFieldKey = GlobalKey();
  final _serverFocusNode = FocusNode();
  bool _invalid = false;

  @override
  void dispose() {
    _serverFocusNode.dispose();
    _server.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    final v = _server.text.trim();
    if (!SettingsRepository.isValidServer(v)) {
      setState(() => _invalid = true);
      return;
    }
    await widget.settings.setServer(v);
    if (!mounted) return;
    setState(() {
      if (_server.text != v) _server.text = v;
      _invalid = false;
    });
    FocusScope.of(context).unfocus();
  }

  /// Privacy mode is slower; say so before turning it on.
  Future<bool> _confirmPrivacy(BuildContext context) async {
    final l = AppLocalizations.of(context);
    return await showDialog<bool>(
          context: context,
          builder: (context) => AlertDialog(
            title: Text(l.privacyModeConfirmTitle),
            content: Text(l.privacyModeConfirmBody),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context, false),
                child: Text(l.cancel),
              ),
              FilledButton(
                onPressed: () => Navigator.pop(context, true),
                child: Text(l.privacyModeTurnOn),
              ),
            ],
          ),
        ) ??
        false;
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);

    final connectionAndPrivacy = <Widget>[
      Semantics(
        header: true,
        child: Text(l.settingsServer, style: theme.textTheme.titleLarge),
      ),
      Gap.s,
      Text(l.settingsServerHelp, style: theme.textTheme.bodyMedium),
      Gap.m,
      TextField(
        key: _serverFieldKey,
        focusNode: _serverFocusNode,
        controller: _server,
        autocorrect: false,
        keyboardType: TextInputType.url,
        // An address reads left to right in every language.
        textDirection: TextDirection.ltr,
        decoration: InputDecoration(
          errorText: _invalid ? l.settingsServerInvalid : null,
          prefixIcon: const Icon(Icons.dns_outlined),
        ),
        onChanged: (_) => setState(() => _invalid = false),
        onSubmitted: (_) => _save(),
      ),
      Gap.m,
      ListenableBuilder(
        listenable: widget.settings,
        builder: (context, _) {
          final dirty = _server.text.trim() != widget.settings.server;
          return FilledButton.tonal(
            onPressed: dirty ? _save : null,
            child: Text(l.settingsSave),
          );
        },
      ),
      Gap.s,
      Align(
        alignment: AlignmentDirectional.centerStart,
        child: TextButton(
          onPressed: () async {
            await widget.settings.resetServer();
            if (!mounted) return;
            setState(() {
              _server.text = widget.settings.server;
              _invalid = false;
            });
          },
          child: Text(l.settingsServerReset),
        ),
      ),
      Gap.l,
      const Divider(),
      Gap.l,
      Semantics(
        header: true,
        child: Text(l.settingsPrivacy, style: theme.textTheme.titleLarge),
      ),
      Gap.s,
      Text(l.settingsPrivacyBody, style: theme.textTheme.bodyMedium),
      Gap.m,
      ListenableBuilder(
        listenable: widget.settings,
        builder: (context, _) => SwitchListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(l.privacyMode),
          subtitle: Text(l.privacyModeHint),
          value: widget.settings.privacyMode,
          onChanged: (on) async {
            if (on && !await _confirmPrivacy(context)) return;
            try {
              await widget.settings.setPrivacyMode(on);
            } on Exception {
              // Not saved: the switch keeps showing the setting in force.
            }
          },
        ),
      ),
    ];

    final backgroundAndAbout = <Widget>[
      if (widget.background is! NoBackgroundLimits) ...[
        Semantics(
          header: true,
          child: Text(l.settingsBackground, style: theme.textTheme.titleLarge),
        ),
        Gap.s,
        Text(l.setupBackgroundBody, style: theme.textTheme.bodyMedium),
        Gap.m,
        Align(
          alignment: AlignmentDirectional.centerStart,
          child: FilledButton.tonal(
            onPressed: () => Navigator.push(
              context,
              MaterialPageRoute<void>(
                builder: (context) => BackgroundSetupView(
                  access: widget.background,
                  onDone: ({required skipped}) => Navigator.pop(context),
                ),
              ),
            ),
            child: Text(l.settingsBackgroundOpen),
          ),
        ),
        Gap.l,
        const Divider(),
        Gap.l,
      ],
      Semantics(
        header: true,
        child: Text(l.settingsAbout, style: theme.textTheme.titleLarge),
      ),
      Gap.s,
      Text(l.settingsAboutBody, style: theme.textTheme.bodyMedium),
      // The code and the verification words come from one English
      // word list, so devices in different languages match (ADR 0008).
      Gap.s,
      Text(
        l.settingsWordsEnglish,
        style: theme.textTheme.bodySmall?.copyWith(
          color: theme.colorScheme.onSurfaceVariant,
        ),
      ),
      if (hasDraftTranslation(context)) ...[
        Gap.s,
        Text(
          l.draftTranslation,
          style: theme.textTheme.bodySmall?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
      ],
      Gap.s,
      Text(
        l.settingsVersion(isolate(widget.version)),
        style: theme.textTheme.bodySmall?.copyWith(
          color: theme.colorScheme.onSurfaceVariant,
        ),
      ),
      Text(
        'totaldrop.app',
        style: theme.textTheme.bodySmall?.copyWith(
          color: theme.colorScheme.onSurfaceVariant,
        ),
      ),
    ];

    final bottomPadding = 24 + MediaQuery.paddingOf(context).bottom;

    return Scaffold(
      appBar: AppBar(title: Text(l.settings)),
      body: SafeArea(
        bottom: false,
        child: AdaptivePaneLayout(
          allowFlatTwoPane: false,
          allowTabletop: false,
          primary: ColumnListView(
            bottom: bottomPadding,
            children: connectionAndPrivacy,
          ),
          secondary: ColumnListView(
            bottom: bottomPadding,
            children: backgroundAndAbout,
          ),
          singlePane: ColumnListView(
            bottom: bottomPadding,
            children: [
              ...connectionAndPrivacy,
              Gap.l,
              const Divider(),
              Gap.l,
              ...backgroundAndAbout,
            ],
          ),
        ),
      ),
    );
  }
}
