import 'dart:io';

import 'package:path_provider/path_provider.dart';

import '../src/rust/api/transfer.dart' as bridge;
import 'settings_repository.dart';

/// The single way the UI reaches the Rust core.
class TransferRepository {
  TransferRepository(this._settings);

  final SettingsRepository _settings;

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

  Future<void> decline(bridge.Offer offer) => bridge.declineOffer(offer: offer);

  String? normalizeCode(String input) => bridge.normalizeCode(input: input);

  /// Where received files go: Download/TotalDrop on Android, so they are
  /// easy to find in the Files app. Falls back to the app's own folder if
  /// shared storage cannot be written.
  Future<String> receiveFolder() async {
    if (Platform.isAndroid) {
      final shared = Directory('/storage/emulated/0/Download/TotalDrop');
      try {
        await shared.create(recursive: true);
        final probe = File('${shared.path}/.totaldrop-write-test');
        await probe.writeAsString('');
        await probe.delete();
        return shared.path;
      } on FileSystemException {
        // Fall through to the app folder.
      }
      final own = await getExternalStorageDirectory();
      if (own != null) return '${own.path}/TotalDrop';
    }
    final downloads = await getDownloadsDirectory();
    final base = downloads ?? await getApplicationDocumentsDirectory();
    return '${base.path}/TotalDrop';
  }
}
