// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:flutter/services.dart';

/// Resolves Android's public `Downloads` directory for the active user / work
/// profile and notifies `MediaStore` when received files are written (#592).
abstract interface class DownloadsStorage {
  /// Absolute path to the active user's public `Downloads` folder, or null
  /// when the platform does not expose one through this channel.
  Future<String?> downloadsDirectory();

  /// Asks the platform media scanner to index [paths] so saved files appear
  /// right away in Gallery and the system Files app.
  Future<void> scanFiles(List<String> paths);
}

/// For non-Android platforms and unit tests that do not test storage channels.
class NoDownloadsStorage implements DownloadsStorage {
  const NoDownloadsStorage();

  @override
  Future<String?> downloadsDirectory() => Future.value(null);

  @override
  Future<void> scanFiles(List<String> paths) => Future.value();
}

/// Reaches `DownloadsStorage.kt` on Android over `app.totaldrop/storage`.
class AndroidDownloadsStorage implements DownloadsStorage {
  const AndroidDownloadsStorage();

  static const _channel = MethodChannel('app.totaldrop/storage');

  @override
  Future<String?> downloadsDirectory() async {
    try {
      final path = await _channel.invokeMethod<String>('downloadsDir');
      return (path == null || path.isEmpty) ? null : path;
    } on PlatformException {
      return null;
    } on MissingPluginException {
      return null;
    }
  }

  @override
  Future<void> scanFiles(List<String> paths) async {
    if (paths.isEmpty) return;
    try {
      await _channel.invokeMethod<void>('scanFiles', paths);
    } on PlatformException {
      // Indexing is best effort; the files themselves are already saved.
    } on MissingPluginException {
      // Nothing listens outside Android.
    }
  }
}
