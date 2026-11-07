// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../src/rust/api/transfer.dart' as bridge;

/// The few settings the app keeps on the device. Nothing here leaves it.
class SettingsRepository extends ChangeNotifier {
  SettingsRepository._(this._prefs);

  static const _serverKey = 'rendezvous_server';
  static const _privacyKey = 'privacy_mode';
  static const _setupSeenKey = 'background_setup_seen';
  static const _remindedKey = 'background_reminded';
  static const _skippedKey = 'background_skipped';

  final SharedPreferencesWithCache _prefs;

  static Future<SettingsRepository> load() async {
    final prefs = await SharedPreferencesWithCache.create(
      cacheOptions: const SharedPreferencesWithCacheOptions(
        allowList: {
          _serverKey,
          _privacyKey,
          _setupSeenKey,
          _remindedKey,
          _skippedKey,
        },
      ),
    );
    return SettingsRepository._(prefs);
  }

  String get defaultServer => bridge.defaultServer();

  String get server => _prefs.getString(_serverKey) ?? defaultServer;

  bool get usesDefaultServer => server == defaultServer;

  /// Only `wss://` servers are accepted from the app.
  static bool isValidServer(String url) {
    final uri = Uri.tryParse(url.trim());
    return uri != null &&
        uri.scheme == 'wss' &&
        uri.host.isNotEmpty &&
        uri.userInfo.isEmpty;
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

  /// Privacy mode: every transfer goes through the relay only, so the
  /// other device never learns this device's IP address. Off by default.
  bool get privacyMode => _prefs.getBool(_privacyKey) ?? false;

  Future<void> setPrivacyMode(bool on) async {
    if (on) {
      await _prefs.setBool(_privacyKey, true);
    } else {
      await _prefs.remove(_privacyKey);
    }
    notifyListeners();
  }

  /// The first-run page about background transfers was shown (#338),
  /// whatever the person chose on it.
  bool get backgroundSetupSeen => _prefs.getBool(_setupSeenKey) ?? false;

  /// [skipped]: the person chose "Not now" rather than "Done".
  Future<void> setBackgroundSetupSeen({required bool skipped}) async {
    await _prefs.setBool(_setupSeenKey, true);
    if (skipped) {
      await _prefs.setBool(_skippedKey, true);
    } else {
      await _prefs.remove(_skippedKey);
    }
  }

  bool get backgroundSkipped => _prefs.getBool(_skippedKey) ?? false;

  /// The one reminder, after a transfer first left the screen while the
  /// restrictions were still on, was shown.
  bool get backgroundReminded => _prefs.getBool(_remindedKey) ?? false;

  Future<void> setBackgroundReminded() => _prefs.setBool(_remindedKey, true);

  Future<void> resetServer() async {
    await _prefs.remove(_serverKey);
    notifyListeners();
  }
}
