import 'dart:async';

import 'package:flutter/foundation.dart';

import '../../data/transfer_repository.dart';
import '../../src/rust/api/transfer.dart';

enum ReceiveStage { entering, connecting, offered, transferring, done, declined, failed }

class ReceiveViewModel extends ChangeNotifier {
  ReceiveViewModel({required TransferRepository repository}) : _repo = repository;

  final TransferRepository _repo;
  StreamSubscription<TransferEvent>? _sub;
  Stopper? _stopper;
  // Bumped by every connect and reset, so a call that returns after the
  // person moved on changes nothing.
  int _generation = 0;

  ReceiveStage stage = ReceiveStage.entering;
  String input = '';
  bool showInvalid = false;
  Offer? offer;
  String? folder;
  int done = 0;
  int total = 0;
  TransferSummary? summary;
  TransferError? error;

  bool get busy => stage == ReceiveStage.connecting || stage == ReceiveStage.transferring;

  String? get normalized => _repo.normalizeCode(input);

  void setInput(String v) {
    input = v;
    showInvalid = false;
    notifyListeners();
  }

  Future<void> connect() async {
    final code = normalized;
    if (code == null) {
      showInvalid = true;
      notifyListeners();
      return;
    }
    stage = ReceiveStage.connecting;
    error = null;
    final generation = ++_generation;
    final stopper = _repo.newStopper();
    _stopper = stopper;
    notifyListeners();
    try {
      final o = await _repo.connect(code, stopper);
      final f = await _repo.receiveFolder();
      if (generation != _generation) return;
      offer = o;
      folder = f;
      stage = ReceiveStage.offered;
    } on TransferError catch (e) {
      if (generation != _generation) return;
      error = e;
      stage = ReceiveStage.failed;
    }
    notifyListeners();
  }

  Future<void> accept() async {
    final o = offer;
    final dest = folder;
    final stopper = _stopper;
    if (o == null || dest == null || stopper == null) return;
    stage = ReceiveStage.transferring;
    total = o.totalBytes;
    notifyListeners();
    _sub = _repo
        .accept(o, dest, stopper)
        .listen(
          _onEvent,
          onError: (Object e) {
            _fail(e is TransferError ? e : TransferError(kind: ErrorKind.other, detail: '$e'));
          },
        );
  }

  Future<void> decline() async {
    final o = offer;
    if (o == null) return;
    try {
      await _repo.decline(o);
    } on TransferError {
      // Declining is best effort; the sender times out either way.
    }
    stage = ReceiveStage.declined;
    notifyListeners();
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
      case EventKind.failed:
        _fail(e.error ?? const TransferError(kind: ErrorKind.other, detail: ''));
        return;
    }
    notifyListeners();
  }

  void _fail(TransferError e) {
    error = e;
    stage = ReceiveStage.failed;
    _sub?.cancel();
    notifyListeners();
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
    stage = ReceiveStage.entering;
    input = '';
    showInvalid = false;
    offer = null;
    done = 0;
    total = 0;
    summary = null;
    error = null;
    notifyListeners();
  }

  @override
  void dispose() {
    cancel();
    _sub?.cancel();
    super.dispose();
  }
}
