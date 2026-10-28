import 'package:dynamic_color/dynamic_color.dart';
import 'package:material_ui/material_ui.dart';

import 'data/settings_repository.dart';
import 'data/transfer_repository.dart';
import 'l10n/app_localizations.dart';
import 'src/rust/frb_generated.dart';
import 'ui/core/l10n.dart';
import 'ui/core/theme.dart';
import 'ui/home/home_view.dart';
import 'ui/receive/receive_view.dart';
import 'ui/receive/receive_view_model.dart';
import 'ui/send/send_view.dart';
import 'ui/send/send_view_model.dart';
import 'ui/settings/settings_view.dart';

/// Shown in Settings; kept in step with pubspec.yaml by the release script.
const appVersion = String.fromEnvironment('TOTALDROP_VERSION', defaultValue: '0.1.0-beta.5');

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  final settings = await SettingsRepository.load();
  final transfers = TransferRepository(settings);
  runApp(
    TotalDropApp(
      settings: settings,
      send: SendViewModel(repository: transfers),
      receive: ReceiveViewModel(repository: transfers),
    ),
  );
}

class TotalDropApp extends StatelessWidget {
  const TotalDropApp({
    super.key,
    required this.settings,
    required this.send,
    required this.receive,
  });

  final SettingsRepository settings;
  final SendViewModel send;
  final ReceiveViewModel receive;

  @override
  Widget build(BuildContext context) {
    // Follow the system's colours (Material You) when it has them; the
    // Aurora palette otherwise. Light and dark follow the system too.
    return DynamicColorBuilder(
      builder: (light, dark) {
        final lightScheme = light ?? fallbackScheme(Brightness.light);
        final darkScheme = dark ?? fallbackScheme(Brightness.dark);
        return MaterialApp(
          title: 'TotalDrop',
          debugShowCheckedModeBanner: false,
          theme: buildTheme(lightScheme),
          darkTheme: buildTheme(darkScheme),
          localizationsDelegates: const [
            AppLocalizations.delegate,
            ...GlobalMaterialLocalizations.delegates,
          ],
          supportedLocales: AppLocalizations.supportedLocales,
          localeListResolutionCallback: resolveLocale,
          home: Builder(
            builder: (context) => HomeView(
              onSend: () => Navigator.push(
                context,
                MaterialPageRoute<void>(builder: (_) => SendView(viewModel: send)),
              ),
              onReceive: () => Navigator.push(
                context,
                MaterialPageRoute<void>(builder: (_) => ReceiveView(viewModel: receive)),
              ),
              onSettings: () => Navigator.push(
                context,
                MaterialPageRoute<void>(
                  builder: (_) => SettingsView(settings: settings, version: appVersion),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}
