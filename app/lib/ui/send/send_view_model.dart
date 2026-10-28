import 'dart:async';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';

import '../../data/transfer_repository.dart';
import '../../src/rust/api/transfer.dart';

enum SendStage { picking, preparing, waiting, transferring, done, failed }

class PickedFile {
  const PickedFile({required this.name, required this.path, required this.size});
  final String name;
  final String path;
  final int size;
}

class SendViewModel extends ChangeNotifier {
  SendViewModel({required TransferRepository repository}) : _repo = repository;

  final TransferRepository _repo;
  StreamSubscription<TransferEvent>? _sub;
  Stopper? _stopper;
  // Bumped by every start and reset, so a call that returns after the
  // person moved on changes nothing.
  int _generation = 0;

  final List<PickedFile> _files = [];
  List<PickedFile> get files => List.unmodifiable(_files);
  int get totalSize => _files.fold(0, (a, f) => a + f.size);

  bool strong = false;
  SendStage stage = SendStage.picking;
  String? code;
  List<String> words = const [];
  int done = 0;
  int total = 0;
  TransferSummary? summary;
  TransferError? error;

  bool get busy =>
      stage == SendStage.preparing || stage == SendStage.waiting || stage == SendStage.transferring;

  Future<void> addFiles() async {
    // Multiple selection; files are copied to the app cache, never loaded
    // into memory.
    final picked = await FilePicker.pickFiles();
    for (final f in picked) {
      final path = f.path;
      if (path == null || _files.any((e) => e.path == path)) continue;
      final size = f.lengthSync() ?? await f.length() ?? 0;
      _files.add(PickedFile(name: f.name, path: path, size: size));
    }
    notifyListeners();
  }

  void remove(PickedFile f) {
    _files.remove(f);
    notifyListeners();
  }

  void setStrong(bool v) {
    strong = v;
    notifyListeners();
  }

  Future<void> start() async {
    if (_files.isEmpty || busy) return;
    stage = SendStage.preparing;
    error = null;
    final generation = ++_generation;
    final stopper = _repo.newStopper();
    _stopper = stopper;
    notifyListeners();
    final SendSession session;
    try {
      session = await _repo.prepareSend(
        _files.map((f) => f.path).toList(),
        strong: strong,
        stopper: stopper,
      );
    } on TransferError catch (e) {
      if (generation == _generation) _fail(e);
      return;
    }
    if (generation != _generation) return;
    code = session.code;
    total = session.totalBytes;
    stage = SendStage.waiting;
    notifyListeners();
    _sub = _repo
        .runSend(session, stopper)
        .listen(
          _onEvent,
          onError: (Object e) {
            _fail(e is TransferError ? e : TransferError(kind: ErrorKind.other, detail: '$e'));
          },
        );
  }

  void _onEvent(TransferEvent e) {
    switch (e.kind) {
      case EventKind.waiting:
        stage = SendStage.waiting;
      case EventKind.connected:
        words = e.words;
        stage = SendStage.transferring;
      case EventKind.progress:
        done = e.done;
        total = e.total;
        stage = SendStage.transferring;
      case EventKind.finished:
        summary = e.summary;
        stage = SendStage.done;
        _cleanUp();
      case EventKind.failed:
        _fail(e.error ?? const TransferError(kind: ErrorKind.other, detail: ''));
        return;
    }
    notifyListeners();
  }

  void _fail(TransferError e) {
    error = e;
    stage = SendStage.failed;
    _cleanUp();
    notifyListeners();
  }

  void _cleanUp() {
    _sub?.cancel();
    _sub = null;
  }

  /// Stop the transfer, at whatever stage it is in. The receiver is told.
  void cancel() {
    final s = _stopper;
    _stopper = null;
    if (s != null) _repo.stop(s);
  }

  /// Back to an empty selection.
  void reset() {
    cancel();
    _generation++;
    _sub?.cancel();
    _sub = null;
    _files.clear();
    // The picker keeps copies of picked files in the app cache; drop them.
    unawaited(FilePicker.clearTemporaryFiles().catchError((_) => false));
    stage = SendStage.picking;
    code = null;
    words = const [];
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
