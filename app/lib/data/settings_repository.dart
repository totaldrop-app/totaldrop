import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../src/rust/api/transfer.dart' as bridge;

/// The few settings the app keeps on the device. Nothing here leaves it.
class SettingsRepository extends ChangeNotifier {
  SettingsRepository._(this._prefs);

  static const _serverKey = 'rendezvous_server';

  final SharedPreferencesWithCache _prefs;

  static Future<SettingsRepository> load() async {
    final prefs = await SharedPreferencesWithCache.create(
      cacheOptions: const SharedPreferencesWithCacheOptions(allowList: {_serverKey}),
    );
    return SettingsRepository._(prefs);
  }

  String get defaultServer => bridge.defaultServer();

  String get server => _prefs.getString(_serverKey) ?? defaultServer;

  bool get usesDefaultServer => server == defaultServer;

  /// Only `wss://` servers are accepted from the app.
  static bool isValidServer(String url) {
    final uri = Uri.tryParse(url.trim());
    return uri != null && uri.scheme == 'wss' && uri.host.isNotEmpty && uri.userInfo.isEmpty;
  }

  Future<void> setServer(String url) async {
    final v = url.trim();
    if (v == defaultServer) {
      await _prefs.remove(_serverKey);
    } else {
      await _prefs.setString(_serverKey, v);
    }
    notifyListeners();
  }

  Future<void> resetServer() async {
    await _prefs.remove(_serverKey);
    notifyListeners();
  }
}
