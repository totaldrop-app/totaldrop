// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:flutter/services.dart';

/// What the native camera QR scanner returned (#29).
sealed class QrScanResult {
  const QrScanResult();
}

/// A TotalDrop receive link (`https://totaldrop.app/r#…` or
/// `totaldrop://r/…`) decoded from a QR code; Rust validates the code.
class QrScanned extends QrScanResult {
  const QrScanned(this.link);
  final String link;
}

/// Camera permission was denied; typing or pasting the code stays available.
class QrCameraDenied extends QrScanResult {
  const QrCameraDenied();
}

/// No usable camera was found on the device.
class QrCameraUnavailable extends QrScanResult {
  const QrCameraUnavailable();
}

/// The person closed the camera screen without scanning a code.
class QrScanCancelled extends QrScanResult {
  const QrScanCancelled();
}

/// Opens the camera QR scanner on platforms that provide one (#29).
abstract class QrScanner {
  const QrScanner();

  bool get available;

  Future<QrScanResult> scan();
}

/// Default stand-in where no camera scanner is wired.
class NoQrScanner extends QrScanner {
  const NoQrScanner();

  @override
  bool get available => false;

  @override
  Future<QrScanResult> scan() async => const QrScanCancelled();
}

/// Launches `QrScanActivity` over `app.totaldrop/qr_scan` (#29).
class AndroidQrScanner extends QrScanner {
  const AndroidQrScanner();

  static const _channel = MethodChannel('app.totaldrop/qr_scan');

  @override
  bool get available => true;

  @override
  Future<QrScanResult> scan() async {
    try {
      final reply = await _channel.invokeMapMethod<String, Object?>('scan');
      final outcome = reply?['outcome'] as String?;
      final link = reply?['link'] as String?;
      return switch (outcome) {
        'scanned' when link != null => QrScanned(link),
        'denied' => const QrCameraDenied(),
        'unavailable' => const QrCameraUnavailable(),
        _ => const QrScanCancelled(),
      };
    } on PlatformException {
      return const QrCameraUnavailable();
    }
  }
}
