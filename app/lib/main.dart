// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'dart:io';

import 'package:dynamic_color/dynamic_color.dart';
import 'package:material_ui/material_ui.dart';

import 'data/background_access.dart';
import 'data/launch_source.dart';
import 'data/settings_repository.dart';
import 'data/transfer_keep_alive.dart';
import 'data/transfer_repository.dart';
import 'l10n/app_localizations.dart';
import 'src/rust/frb_generated.dart';
import 'ui/core/l10n.dart';
import 'ui/core/launch_router.dart';
import 'ui/core/theme.dart';
import 'ui/home/home_view.dart';
import 'ui/receive/receive_view.dart';
import 'ui/receive/receive_view_model.dart';
import 'ui/send/send_view.dart';
import 'ui/send/send_view_model.dart';
import 'ui/settings/settings_view.dart';
import 'ui/setup/background_reminder.dart';

/// Shown in Settings; kept in step with pubspec.yaml by the release script.
const appVersion = String.fromEnvironment(
  'TOTALDROP_VERSION',
  defaultValue: '0.3.0-beta.7',
);

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  final settings = await SettingsRepository.load();
  final transfers = TransferRepository(settings);
  // The notification that keeps a transfer going in the background speaks
  // the language the app shows.
  final keepAlive = WatchedKeepAlive(
    Platform.isAndroid
        ? AndroidTransferKeepAlive(
            () => lookupAppLocalizations(
              resolveLocale(
                WidgetsBinding.instance.platformDispatcher.locales,
                AppLocalizations.supportedLocales,
              ),
            ),
          )
        : const NoKeepAlive(),
  );
  // Battery restrictions that stop a transfer in the background (#338).
  final BackgroundAccess background = Platform.isAndroid
      ? const AndroidBackgroundAccess()
      : const NoBackgroundLimits();
  // Shares and launcher shortcuts (#53).
  final LaunchSource launches = Platform.isAndroid
      ? AndroidLaunchSource()
      : const NoLaunches();
  runApp(
    TotalDropApp(
      settings: settings,
      launches: launches,
      keepAlive: keepAlive,
      background: background,
      send: SendViewModel(
        repository: transfers,
        keepAlive: keepAlive,
        launches: launches,
      ),
      receive: ReceiveViewModel(repository: transfers, keepAlive: keepAlive),
    ),
  );
}

class TotalDropApp extends StatelessWidget {
  TotalDropApp({
    super.key,
    required this.settings,
    required this.send,
    required this.receive,
    required this.keepAlive,
    this.launches = const NoLaunches(),
    this.background = const NoBackgroundLimits(),
  });

  final SettingsRepository settings;
  final SendViewModel send;
  final ReceiveViewModel receive;
  final LaunchSource launches;

  /// The same keep-alive the view models use, watched so that a transfer
  /// that leaves the screen is known (#338).
  final WatchedKeepAlive keepAlive;
  final BackgroundAccess background;
  late final _prompts = BackgroundPrompts(
    settings: settings,
    access: background,
  );
  final _navigator = GlobalKey<NavigatorState>();
  final _messenger = GlobalKey<ScaffoldMessengerState>();

  Route<void> _sendRoute() => MaterialPageRoute<void>(
    settings: const RouteSettings(name: sendRoute),
    builder: (_) => SendView(viewModel: send),
  );

  Route<void> _receiveRoute() => MaterialPageRoute<void>(
    settings: const RouteSettings(name: receiveRoute),
    builder: (_) => ReceiveView(viewModel: receive),
  );

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
          navigatorKey: _navigator,
          scaffoldMessengerKey: _messenger,
          builder: (context, child) => LaunchRouter(
            launches: launches,
            navigator: _navigator,
            messenger: _messenger,
            send: send,
            receive: receive,
            openSend: _sendRoute,
            openReceive: _receiveRoute,
            child: BackgroundReminder(
              prompts: _prompts,
              keepAlive: keepAlive,
              navigator: _navigator,
              child: SystemMotion(child: child!),
            ),
          ),
          localizationsDelegates: const [
            AppLocalizations.delegate,
            ...GlobalMaterialLocalizations.delegates,
          ],
          supportedLocales: AppLocalizations.supportedLocales,
          localeListResolutionCallback: resolveLocale,
          home: FirstRunGate(
            prompts: _prompts,
            child: Builder(
              builder: (context) => HomeView(
                onSend: () => Navigator.push(context, _sendRoute()),
                onReceive: () => Navigator.push(context, _receiveRoute()),
                onSettings: () => Navigator.push(
                  context,
                  MaterialPageRoute<void>(
                    builder: (_) => SettingsView(
                      settings: settings,
                      version: appVersion,
                      background: background,
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}
