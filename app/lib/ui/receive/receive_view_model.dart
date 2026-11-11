// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'dart:async';

import 'package:flutter/foundation.dart';

import '../../data/transfer_keep_alive.dart';
import '../../data/transfer_repository.dart';
import '../../src/rust/api/transfer.dart';
import '../core/l10n.dart';

/// The code in pasted text. A shared message holds more than the code
/// (it may be the whole share text), so this takes its number and word
/// group, and falls back to the whole text. Direction marks are dropped:
/// the share text isolates the code, and they must not reach the field.
/// The number may use Arabic-Indic digits (ar, fa, ur keyboards); Rust
/// reads them as ASCII (`ascii_digits` in totaldrop-proto).
String codeFromPaste(String text) {
  final clean = stripInvisibleFormat(text).trim();
  final match = RegExp(
    r'[0-9\u0660-\u0669\u06F0-\u06F9]+(?:[-\s][a-zA-Z]+){2,3}',
  ).firstMatch(clean);
  return match?.group(0) ?? clean;
}

enum ReceiveStage {
  entering,
  connecting,
  offered,
  transferring,
  done,
  declined,
  failed,
}

class ReceiveViewModel extends ChangeNotifier {
  ReceiveViewModel({
    required TransferRepository repository,
    TransferKeepAlive keepAlive = const NoKeepAlive(),
  }) : _repo = repository,
       _keeper = keepAlive;

  final TransferRepository _repo;
  final TransferKeepAlive _keeper;
  bool _keptAlive = false;
  StreamSubscription<TransferEvent>? _sub;
  Stopper? _stopper;
  // Bumped by every connect and reset, so a call that returns after the
  // person moved on changes nothing.
  int _generation = 0;

  ReceiveStage stage = ReceiveStage.entering;
  String input = '';
  bool showInvalid = false;
  // The code as read, shown while connecting: what is typed or pasted can
  // look different (direction marks, Arabic-Indic digits, a whole message).
  String? connectingCode;
  Offer? offer;
  String? folder;
  int done = 0;
  int total = 0;
  TransferSummary? summary;
  TransferError? error;

  bool get busy =>
      stage == ReceiveStage.connecting || stage == ReceiveStage.transferring;

  /// The offer is a text to show, not files to save (#326).
  bool get isText => offer?.isText ?? false;

  /// The text received, exactly as it came; shown, never saved.
  String? get receivedText => summary?.text;

  String? get normalized => _repo.normalizeCode(input);

  void setInput(String v) {
    input = v;
    showInvalid = false;
    notifyListeners();
  }

  /// Fills in the code of a link that opened the app (#445), in place of
  /// what was typed. Not while a transfer runs or waits for an answer;
  /// a finished one is put away first. Nothing connects until the person
  /// does. Returns whether the link held a code.
  bool takeLink(String link) {
    if (busy || stage == ReceiveStage.offered) return false;
    final code = _repo.codeFromLink(link);
    if (code == null) return false;
    if (stage != ReceiveStage.entering) reset();
    setInput(code);
    return true;
  }

  Future<void> connect() async {
    final code = normalized;
    if (code == null) {
      showInvalid = true;
      notifyListeners();
      return;
    }
    stage = ReceiveStage.connecting;
    connectingCode = code;
    error = null;
    final generation = ++_generation;
    final stopper = _repo.newStopper();
    _stopper = stopper;
    _keepGoing();
    notifyListeners();
    try {
      final o = await _repo.connect(code, stopper);
      // A text is shown, not saved: it needs no folder, and a folder that
      // cannot be made does not stand in its way.
      final f = o.isText ? null : await _repo.receiveFolder();
      if (generation != _generation) {
        // Arrived just after a stop: tell the sender, do not keep it open.
        unawaited(_repo.decline(o).catchError((Object _) {}));
        return;
      }
      offer = o;
      folder = f;
      stage = ReceiveStage.offered;
    } on TransferError catch (e) {
      if (generation != _generation) return;
      error = e;
      stage = ReceiveStage.failed;
      _release();
    } catch (e) {
      // Not from the core (the folder cannot be made, a platform error):
      // still a failure, so the screen and the keep-alive do not hang.
      if (generation != _generation) return;
      error = TransferError(kind: ErrorKind.other, detail: '$e', cause: '');
      stage = ReceiveStage.failed;
      _release();
    }
    notifyListeners();
  }

  Future<void> accept() async {
    final o = offer;
    final dest = folder;
    final stopper = _stopper;
    // Only an offer waiting for an answer: a second tap, or a tap on a
    // finished or declined transfer, accepts nothing again.
    if (stage != ReceiveStage.offered) return;
    if (o == null || stopper == null) return;
    if (dest == null && !o.isText) return;
    stage = ReceiveStage.transferring;
    total = o.totalBytes;
    notifyListeners();
    final events = o.isText
        ? _repo.acceptText(o, stopper)
        : _repo.accept(o, dest!, stopper);
    _sub = events.listen(
      _onEvent,
      onError: (Object e) {
        _fail(
          e is TransferError
              ? e
              : TransferError(kind: ErrorKind.other, detail: '$e', cause: ''),
        );
      },
    );
  }

  /// Decline the offer shown. The screen says so at once, as on Linux,
  /// and nothing changes when the sender has been told: by then the person
  /// may have moved on (a reset, a new code), and that stays.
  Future<void> decline() async {
    final o = offer;
    if (stage != ReceiveStage.offered || o == null) return;
    stage = ReceiveStage.declined;
    _release();
    notifyListeners();
    try {
      await _repo.decline(o);
    } on TransferError {
      // Declining is best effort; the sender times out either way.
    }
  }

  void _onEvent(TransferEvent e) {
    switch (e.kind) {
      case EventKind.waiting:
      case EventKind.connected:
        break;
      case EventKind.progress:
        done = e.done;
        total = e.total;
      case EventKind.finished:
        summary = e.summary;
        stage = ReceiveStage.done;
        _sub?.cancel();
        _release();
      case EventKind.failed:
        _fail(
          e.error ??
              const TransferError(kind: ErrorKind.other, detail: '', cause: ''),
        );
        return;
    }
    notifyListeners();
  }

  void _fail(TransferError e) {
    error = e;
    stage = ReceiveStage.failed;
    _sub?.cancel();
    _release();
    notifyListeners();
  }

  /// Keep the receive going while the app is in the background (#194).
  /// The notification's Stop stops it.
  void _keepGoing() {
    _keptAlive = true;
    unawaited(_keeper.begin(TransferDirection.receiving, _stoppedFromNotice));
  }

  /// Stop on the notification. While an offer waits for an answer no
  /// transfer runs yet, so it is declined instead.
  void _stoppedFromNotice() {
    if (stage == ReceiveStage.offered) {
      unawaited(decline());
    } else {
      cancel();
    }
  }

  void _release() {
    if (!_keptAlive) return;
    _keptAlive = false;
    unawaited(_keeper.end());
  }

  /// Stop the transfer, at whatever stage it is in. The sender is told.
  void cancel() {
    final s = _stopper;
    _stopper = null;
    if (s != null) _repo.stop(s);
  }

  void reset() {
    cancel();
    _generation++;
    _sub?.cancel();
    _sub = null;
    _release();
    stage = ReceiveStage.entering;
    input = '';
    showInvalid = false;
    connectingCode = null;
    offer = null;
    done = 0;
    total = 0;
    summary = null;
    error = null;
    notifyListeners();
  }

  @override
  void dispose() {
    final o = offer;
    if (stage == ReceiveStage.offered && o != null) {
      unawaited(_repo.decline(o).catchError((Object _) {}));
    }
    cancel();
    _sub?.cancel();
    _release();
    super.dispose();
  }
}
