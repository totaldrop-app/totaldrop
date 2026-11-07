// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:flutter/services.dart';

/// Whether a transfer can keep going while TotalDrop is not on screen
/// (#338).
///
/// Android's own battery optimisation can be read. Some makers, Xiaomi
/// among them, add a battery saver of their own that stops an app's
/// traffic as soon as it leaves the screen, even while its transfer job
/// runs; that one cannot be read, only opened.
class BackgroundState {
  const BackgroundState({
    required this.exempt,
    this.maker,
    this.autostart = false,
  });

  /// Android's battery optimisation is off for TotalDrop.
  final bool exempt;

  /// The maker, as people write it ("Xiaomi"), when its phones restrict
  /// background apps on their own; null otherwise.
  final String? maker;

  /// The maker has an Autostart page (Xiaomi).
  final bool autostart;

  /// Nothing to set up: Android does not restrict TotalDrop and no maker
  /// saver may.
  bool get settled => exempt && maker == null;
}

/// The settings pages that lift the restrictions.
enum BackgroundPage { battery, maker, autostart }

abstract class BackgroundAccess {
  Future<BackgroundState> state();

  /// Opens [page]; false when the phone has no such page.
  Future<bool> open(BackgroundPage page);
}

/// Platforms that do not restrict a running transfer, and tests.
class NoBackgroundLimits implements BackgroundAccess {
  const NoBackgroundLimits();

  @override
  Future<BackgroundState> state() =>
      Future.value(const BackgroundState(exempt: true));

  @override
  Future<bool> open(BackgroundPage page) => Future.value(false);
}

class AndroidBackgroundAccess implements BackgroundAccess {
  const AndroidBackgroundAccess();

  static const _channel = MethodChannel('app.totaldrop/background');

  @override
  Future<BackgroundState> state() async {
    try {
      final m = await _channel.invokeMapMethod<String, Object?>('state');
      final maker = m?['maker'];
      return BackgroundState(
        exempt: m?['exempt'] == true,
        maker: maker is String && maker.isNotEmpty ? maker : null,
        autostart: m?['autostart'] == true,
      );
    } on PlatformException {
      // Unknown: offer the settings rather than hide them.
      return const BackgroundState(exempt: false);
    }
  }

  @override
  Future<bool> open(BackgroundPage page) async {
    try {
      return await _channel.invokeMethod<bool>('open', page.name) ?? false;
    } on PlatformException {
      return false;
    }
  }
}
