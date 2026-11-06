// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'dart:async';

import 'package:flutter/foundation.dart';

import '../../data/file_source.dart';
import '../../data/launch_source.dart';
import '../../data/transfer_keep_alive.dart';
import '../../data/transfer_repository.dart';
import '../../src/rust/api/transfer.dart';

export '../../data/file_source.dart' show PickedFile;

enum SendStage { picking, preparing, waiting, transferring, done, failed }

class SendViewModel extends ChangeNotifier {
  SendViewModel({
    required TransferRepository repository,
    TransferKeepAlive keepAlive = const NoKeepAlive(),
    FileSource fileSource = const PlatformFileSource(),
    LaunchSource launches = const NoLaunches(),
  }) : _repo = repository,
       _keeper = keepAlive,
       _source = fileSource,
       _shares = launches;

  final LaunchSource _shares;

  final TransferRepository _repo;
  final FileSource _source;
  final TransferKeepAlive _keeper;
  bool _keptAlive = false;
  StreamSubscription<TransferEvent>? _sub;
  Stopper? _stopper;
  // Bumped by every start and reset, so a call that returns after the
  // person moved on changes nothing.
  int _generation = 0;
  // Bumped by every reset only: a pick that returns after one belongs to
  // no selection.
  int _resets = 0;

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
      stage == SendStage.preparing ||
      stage == SendStage.waiting ||
      stage == SendStage.transferring;

  /// Whether the picker is copying the chosen files into the app cache.
  /// For a large file that takes seconds, and the screen says so (#269).
  bool copying = false;
  bool _picking = false;

  /// Whether the last pick failed, so the screen can say so (#271).
  bool addFailed = false;

  /// Whether files shared from another app are being copied into the
  /// app's storage (#53). The screen shows the same wait as for the picker.
  bool get sharing => _pendingShares.isNotEmpty;
  // Shares taken whose copy is still being written, and shares whose files
  // were added: their copies are dropped when the selection is cleared.
  final Set<int> _pendingShares = {};
  final Set<int> _addedShares = {};

  /// Files shared from another app (#53) start copying as share [id].
  /// False while a transfer runs: then the share is not taken, and the
  /// caller says so and calls [refuseShare]. A share may come while the
  /// picker is open, or while another share copies: all their files are
  /// added.
  bool beginShare(int id) {
    if (busy) return false;
    // The last transfer's result gives way to the new selection.
    if (stage != SendStage.picking) reset();
    _pendingShares.add(id);
    addFailed = false;
    notifyListeners();
    return true;
  }

  /// Share [id] is not taken: its copy stops and is deleted.
  void refuseShare(int id) => unawaited(_shares.drop([id]));

  /// The files of share [id], copied. False, and the copy is deleted, unless
  /// [beginShare] took the share and no reset came since.
  bool addShared(int id, List<PickedFile> files) {
    if (!_pendingShares.remove(id)) {
      refuseShare(id);
      return false;
    }
    _addedShares.add(id);
    for (final f in files) {
      if (_files.any((e) => e.path == f.path)) continue;
      _files.add(f);
    }
    notifyListeners();
    return true;
  }

  /// The person stopped waiting for the shares being copied: they stop
  /// and their copies are deleted. What was chosen before stays.
  void cancelShares() {
    if (_pendingShares.isEmpty) return;
    unawaited(_shares.drop(_pendingShares.toList()));
    _pendingShares.clear();
    notifyListeners();
  }

  /// The files of share [id] could not be copied: the screen says so as it
  /// does for the picker (#271).
  void shareFailed(int id) {
    if (!_pendingShares.remove(id)) return;
    addFailed = true;
    notifyListeners();
  }

  Future<void> addFiles() async {
    // One picker at a time: the plugin refuses a second one anyway. Not
    // while a share copies either.
    if (_picking || sharing) return;
    _picking = true;
    final resets = _resets;
    if (addFailed) {
      addFailed = false;
      notifyListeners();
    }
    try {
      final picked = await _source.pick(
        onCopying: (c) {
          if (copying == c) return;
          copying = c;
          notifyListeners();
        },
      );
      if (resets != _resets) {
        // The selection was cleared while the picker copied: these copies
        // belong to no selection, and the reset's clean-up may have missed
        // them. Nothing else is in the cache after a reset.
        unawaited(_source.clearCopies());
        return;
      }
      for (final f in picked) {
        if (_files.any((e) => e.path == f.path)) continue;
        _files.add(f);
      }
    } on Exception catch (e) {
      // The picker could not hand the files over: a path it cannot read,
      // too little memory, or no space left for its copy. The screen
      // says so; a pick that returns after a reset belongs to no
      // selection and says nothing (#271). A copy cut short by a full
      // disk may leave a partial file in the cache; clearing it here
      // would also delete the files already chosen, so a reset does it.
      debugPrint('file picker failed: $e');
      if (resets == _resets) addFailed = true;
    } finally {
      // The plugin says the copy ended just before it returns, but only
      // while someone listens; end it here too.
      _picking = false;
      copying = false;
      notifyListeners();
    }
  }

  @visibleForTesting
  void addFakeFileForTest(String name) =>
      _files.add(PickedFile(name: name, path: '/tmp/$name', size: 1));

  void remove(PickedFile f) {
    _files.remove(f);
    // The message was about the selection as it was.
    addFailed = false;
    notifyListeners();
  }

  void setStrong(bool v) {
    strong = v;
    notifyListeners();
  }

  Future<void> start() async {
    // Not while the picker or a share copies: the list is about to change
    // (#269).
    if (_files.isEmpty || busy || _picking || sharing) return;
    stage = SendStage.preparing;
    error = null;
    addFailed = false;
    final generation = ++_generation;
    final stopper = _repo.newStopper();
    _stopper = stopper;
    _keepGoing();
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
    } catch (e) {
      // Not from the core (a file that went away, a platform error): still
      // a failure, so the screen and the keep-alive do not hang.
      if (generation == _generation) {
        _fail(TransferError(kind: ErrorKind.other, detail: '$e', cause: ''));
      }
      return;
    }
    if (generation != _generation) {
      // Prepared just before a stop: release the code now, not at GC.
      session.dispose();
      return;
    }
    code = session.code;
    total = session.totalBytes;
    stage = SendStage.waiting;
    notifyListeners();
    _sub = _repo
        .runSend(session, stopper)
        .listen(
          _onEvent,
          onError: (Object e) {
            _fail(
              e is TransferError
                  ? e
                  : TransferError(
                      kind: ErrorKind.other,
                      detail: '$e',
                      cause: '',
                    ),
            );
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
    stage = SendStage.failed;
    _cleanUp();
    notifyListeners();
  }

  void _cleanUp() {
    _sub?.cancel();
    _sub = null;
    _release();
  }

  /// Keep the send going while the person shares the code in another app
  /// (#194). The notification's Stop stops it.
  void _keepGoing() {
    _keptAlive = true;
    unawaited(_keeper.begin(TransferDirection.sending, cancel));
  }

  void _release() {
    if (!_keptAlive) return;
    _keptAlive = false;
    unawaited(_keeper.end());
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
    _resets++;
    _sub?.cancel();
    _sub = null;
    _release();
    _files.clear();
    addFailed = false;
    // The picker keeps copies of picked files in the app cache, and shares
    // in the app's storage; drop them. A share still copying stops.
    unawaited(_source.clearCopies());
    unawaited(_shares.drop({..._pendingShares, ..._addedShares}));
    _pendingShares.clear();
    _addedShares.clear();
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
    _release();
    super.dispose();
  }
}
