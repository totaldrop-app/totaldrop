import 'package:flutter/services.dart';

import '../l10n/app_localizations.dart';

enum TransferDirection { sending, receiving }

/// Keeps a send or a receive going while the app is in the background.
///
/// Android cuts the network of an app the person switched away from. The
/// code then drops at the rendezvous server, often while the person shares
/// it in another app (#194). On Android the transfer runs as a
/// user-initiated data transfer job with an ongoing notification and a Stop
/// action (`TransferJob.kt`).
abstract class TransferKeepAlive {
  /// Start keeping the transfer alive. Call it while the app is visible.
  /// [onStop] runs when the person stops the transfer from the
  /// notification, or the system ends it. If the system refuses, the
  /// transfer still runs, but only while the app is visible.
  ///
  /// One transfer at a time: the send and receive screens never run
  /// together, so one instance serves both.
  Future<void> begin(TransferDirection direction, void Function() onStop);

  /// The transfer ended, for whatever reason.
  Future<void> end();
}

/// For platforms that keep a transfer running anyway, and for tests.
class NoKeepAlive implements TransferKeepAlive {
  const NoKeepAlive();

  @override
  Future<void> begin(TransferDirection direction, void Function() onStop) =>
      Future.value();

  @override
  Future<void> end() => Future.value();
}

class AndroidTransferKeepAlive implements TransferKeepAlive {
  AndroidTransferKeepAlive(this._strings) {
    _channel.setMethodCallHandler((call) async {
      if (call.method == 'stop') _onStop?.call();
    });
  }

  static const _channel = MethodChannel('app.totaldrop/transfer_job');

  /// The app's strings in the person's language; read at each [begin], so
  /// a language change applies to the next transfer.
  final AppLocalizations Function() _strings;
  void Function()? _onStop;

  @override
  Future<void> begin(
    TransferDirection direction,
    void Function() onStop,
  ) async {
    _onStop = onStop;
    final l = _strings();
    try {
      await _channel.invokeMethod<bool>('begin', {
        'channel': l.transferNoticeChannel,
        'title': switch (direction) {
          TransferDirection.sending => l.transferNoticeSending,
          TransferDirection.receiving => l.transferNoticeReceiving,
        },
        'text': l.transferNoticeBody,
        'stop': l.stop,
      });
    } on PlatformException {
      // The transfer still runs while the app is visible.
    }
  }

  @override
  Future<void> end() async {
    _onStop = null;
    try {
      await _channel.invokeMethod<void>('end');
    } on PlatformException {
      // Nothing is left to release.
    }
  }
}
