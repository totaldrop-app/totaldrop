// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'dart:io';

import 'package:path_provider/path_provider.dart';

import '../src/rust/api/transfer.dart' as bridge;
import 'downloads_storage.dart';
import 'settings_repository.dart';

/// The single way the UI reaches the Rust core.
class TransferRepository {
  TransferRepository(
    this._settings, {
    DownloadsStorage? storage,
    bool? isAndroid,
    Future<Directory?> Function()? externalStorageDirectory,
  }) : _isAndroid = isAndroid ?? Platform.isAndroid,
       _storage =
           storage ??
           ((isAndroid ?? Platform.isAndroid)
               ? const AndroidDownloadsStorage()
               : const NoDownloadsStorage()),
       _externalStorageDirectory =
           externalStorageDirectory ?? getExternalStorageDirectory;

  final SettingsRepository _settings;
  final DownloadsStorage _storage;
  final bool _isAndroid;
  final Future<Directory?> Function() _externalStorageDirectory;

  /// One per transfer: every call of that transfer takes it, and
  /// [stop] ends the transfer at whatever stage it is in.
  bridge.Stopper newStopper() => bridge.newStopper();

  void stop(bridge.Stopper stopper) => bridge.stopTransfer(stopper: stopper);

  Future<bridge.SendSession> prepareSend(
    List<String> paths, {
    required bool strong,
    required bridge.Stopper stopper,
  }) => bridge.prepareSend(
    server: _settings.server,
    paths: paths,
    strong: strong,
    privacy: _settings.privacyMode,
    stopper: stopper,
  );

  /// A text the receiver shows instead of saving (#326).
  Future<bridge.SendSession> prepareSendText(
    String text, {
    required bool strong,
    required bridge.Stopper stopper,
  }) => bridge.prepareSendText(
    server: _settings.server,
    text: text,
    strong: strong,
    privacy: _settings.privacyMode,
    stopper: stopper,
  );

  /// The most UTF-8 bytes a text may have.
  int get maxTextBytes => bridge.maxTextBytes();

  Stream<bridge.TransferEvent> runSend(
    bridge.SendSession session,
    bridge.Stopper stopper,
  ) => bridge.runSend(session: session, stopper: stopper);

  Future<bridge.Offer> connect(String code, bridge.Stopper stopper) =>
      bridge.connectReceive(
        server: _settings.server,
        code: code,
        privacy: _settings.privacyMode,
        stopper: stopper,
      );

  Stream<bridge.TransferEvent> accept(
    bridge.Offer offer,
    String destination,
    bridge.Stopper stopper,
  ) => bridge.acceptOffer(
    offer: offer,
    destination: destination,
    stopper: stopper,
  );

  /// Receives a text offer in memory; nothing is written (#326).
  Stream<bridge.TransferEvent> acceptText(
    bridge.Offer offer,
    bridge.Stopper stopper,
  ) => bridge.acceptText(offer: offer, stopper: stopper);

  Future<void> decline(bridge.Offer offer) => bridge.declineOffer(offer: offer);

  String? normalizeCode(String input) => bridge.normalizeCode(input: input);

  /// Classifies a TotalDrop receive link strictly in Rust (#445, #516):
  /// returns the normalized code, an empty string when nothing sits where
  /// the code goes, or `null` when the link is invalid.
  String? codeFromLink(String link) => bridge.codeFromLink(link: link);

  /// The code in a TotalDrop QR code (`https://totaldrop.app/r#…` or
  /// `totaldrop://r/…`), read strictly in Rust (#29).
  String? codeFromQr(String text) => bridge.codeFromQr(text: text);

  /// Where received files go: `<public Downloads>/TotalDrop` on Android,
  /// queried via the platform API so secondary users and Work Profiles save
  /// into their own public Downloads (#592). Falls back to the app's own
  /// folder if shared storage cannot be written.
  Future<String> receiveFolder() async {
    if (_isAndroid) {
      final publicDownloads =
          await _storage.downloadsDirectory() ?? '/storage/emulated/0/Download';
      final shared = Directory('$publicDownloads/TotalDrop');
      try {
        await shared.create(recursive: true);
        final probe = File('${shared.path}/.totaldrop-write-test');
        await probe.writeAsString('');
        await probe.delete();
        return shared.path;
      } on FileSystemException {
        // Fall through to the app folder.
      }
      final own = await _externalStorageDirectory();
      if (own != null) return '${own.path}/TotalDrop';
    }
    final downloads = await getDownloadsDirectory();
    final base = downloads ?? await getApplicationDocumentsDirectory();
    return '${base.path}/TotalDrop';
  }

  /// Indexes received files in Android's `MediaStore` so they appear right
  /// away in Gallery and the system Files app (#592).
  Future<void> scanSavedFiles(List<String> paths) => _storage.scanFiles(paths);
}
