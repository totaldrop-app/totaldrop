import 'package:material_ui/material_ui.dart';

import '../../data/settings_repository.dart';
import '../../l10n/app_localizations.dart';
import '../core/l10n.dart';
import '../core/theme.dart';

class SettingsView extends StatefulWidget {
  const SettingsView({
    super.key,
    required this.settings,
    required this.version,
  });

  final SettingsRepository settings;
  final String version;

  @override
  State<SettingsView> createState() => _SettingsViewState();
}

class _SettingsViewState extends State<SettingsView> {
  late final _server = TextEditingController(text: widget.settings.server);
  bool _invalid = false;

  @override
  void dispose() {
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
    if (mounted) FocusScope.of(context).unfocus();
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
    return Scaffold(
      appBar: AppBar(title: Text(l.settings)),
      body: ListView(
        padding: const EdgeInsets.all(24),
        children: [
          Text(l.settingsServer, style: theme.textTheme.titleLarge),
          Gap.s,
          Text(l.settingsServerHelp, style: theme.textTheme.bodyMedium),
          Gap.m,
          TextField(
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
          FilledButton.tonal(onPressed: _save, child: Text(l.settingsSave)),
          Gap.s,
          Align(
            alignment: AlignmentDirectional.centerStart,
            child: TextButton(
              onPressed: () async {
                await widget.settings.resetServer();
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
          Text(l.settingsPrivacy, style: theme.textTheme.titleLarge),
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
          Gap.l,
          const Divider(),
          Gap.l,
          Text(l.settingsAbout, style: theme.textTheme.titleLarge),
          Gap.s,
          Text(l.settingsAboutBody, style: theme.textTheme.bodyMedium),
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
        ],
      ),
    );
  }
}
