import 'package:material_ui/material_ui.dart';

import '../../data/settings_repository.dart';
import '../../l10n/app_localizations.dart';
import '../core/theme.dart';

class SettingsView extends StatefulWidget {
  const SettingsView({super.key, required this.settings, required this.version});

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
            alignment: Alignment.centerLeft,
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
          Gap.l,
          const Divider(),
          Gap.l,
          Text(l.settingsAbout, style: theme.textTheme.titleLarge),
          Gap.s,
          Text(l.settingsAboutBody, style: theme.textTheme.bodyMedium),
          Gap.s,
          Text(
            l.settingsVersion(widget.version),
            style: theme.textTheme.bodySmall?.copyWith(color: theme.colorScheme.onSurfaceVariant),
          ),
          Text(
            'totaldrop.app',
            style: theme.textTheme.bodySmall?.copyWith(color: theme.colorScheme.onSurfaceVariant),
          ),
        ],
      ),
    );
  }
}
