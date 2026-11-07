// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Stand-ins for the Rust bridge, shared by the view model tests.
import 'dart:async';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart'
    show PlatformInt64;
import 'package:totaldrop/data/file_source.dart';
import 'package:totaldrop/data/transfer_keep_alive.dart';
import 'package:totaldrop/data/transfer_repository.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/ui/send/send_view_model.dart' show TextClipboard;

class _Opaque {
  bool disposed = false;
  void dispose() => disposed = true;
  bool get isDisposed => disposed;
}

class FakeStopper extends _Opaque implements Stopper {
  bool stopped = false;
}

class FakeSession extends _Opaque implements SendSession {
  @override
  String get code => '7-harbor-rocket';
  @override
  PlatformInt64 get items => 1;
  @override
  PlatformInt64 get totalBytes => 10;
}

class FakeOffer extends _Opaque implements Offer {
  FakeOffer({this.isText = false});
  @override
  final bool isText;
  @override
  PlatformInt64 get files => 1;
  @override
  PlatformInt64 get folders => 0;
  @override
  List<String> get names => const ['a.txt'];
  @override
  PlatformInt64 get totalBytes => 10;
  @override
  List<String> get words => const ['one', 'two', 'three', 'four'];
}

class FakeRepository implements TransferRepository {
  final stoppers = <FakeStopper>[];
  final declined = <Offer>[];
  Completer<SendSession> prepared = Completer();
  Completer<Offer> connected = Completer();
  StreamController<TransferEvent> events = StreamController();

  @override
  Stopper newStopper() {
    final s = FakeStopper();
    stoppers.add(s);
    return s;
  }

  @override
  void stop(Stopper stopper) => (stopper as FakeStopper).stopped = true;

  @override
  Future<SendSession> prepareSend(
    List<String> paths, {
    required bool strong,
    required Stopper stopper,
  }) => prepared.future;

  /// The texts [prepareSendText] was asked to send.
  final sentTexts = <String>[];

  @override
  Future<SendSession> prepareSendText(
    String text, {
    required bool strong,
    required Stopper stopper,
  }) {
    sentTexts.add(text);
    return prepared.future;
  }

  @override
  int get maxTextBytes => 1 << 20;

  @override
  Stream<TransferEvent> runSend(SendSession session, Stopper stopper) =>
      events.stream;

  /// Which accept the receive took: 'files' or 'text'.
  final accepted = <String>[];

  @override
  Stream<TransferEvent> acceptText(Offer offer, Stopper stopper) {
    accepted.add('text');
    return events.stream;
  }

  @override
  Future<Offer> connect(String code, Stopper stopper) => connected.future;

  @override
  Stream<TransferEvent> accept(
    Offer offer,
    String destination,
    Stopper stopper,
  ) {
    accepted.add('files');
    return events.stream;
  }

  /// Holds [decline] open while set, so a test can act before it returns.
  Completer<void>? declining;

  @override
  Future<void> decline(Offer offer) async {
    declined.add(offer);
    await declining?.future;
  }

  @override
  String? normalizeCode(String input) => input;

  /// Thrown by [receiveFolder] when set.
  Object? folderError;

  @override
  Future<String> receiveFolder() async {
    final e = folderError;
    if (e != null) throw e;
    return '/tmp/TotalDrop';
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

TransferEvent event(EventKind kind) => TransferEvent(
  kind: kind,
  words: const [],
  done: 0,
  total: 0,
  summary: null,
  error: null,
);

/// Records what the view models ask of the keep-alive, and lets a test
/// press the notification's Stop.
class FakeKeepAlive implements TransferKeepAlive {
  final calls = <String>[];
  void Function()? onStop;

  bool get active => calls.isNotEmpty && calls.last.startsWith('begin');

  @override
  Future<void> begin(TransferDirection direction, void Function() onStop) {
    calls.add('begin ${direction.name}');
    this.onStop = onStop;
    return Future.value();
  }

  @override
  Future<void> end() {
    calls.add('end');
    onStop = null;
    return Future.value();
  }
}

/// The system picker, held open by the test: [startCopy] plays the copy
/// into the app cache that Android's picker does before it returns (#269),
/// [finish] closes it.
class FakeFileSource implements FileSource {
  int picks = 0;
  int clears = 0;
  void Function(bool copying)? _onCopying;
  Completer<List<PickedFile>>? _result;

  @override
  Future<List<PickedFile>> pick({
    required void Function(bool copying) onCopying,
  }) {
    picks++;
    _onCopying = onCopying;
    return (_result = Completer()).future;
  }

  void startCopy() => _onCopying!(true);

  /// Ends the pick with [files]. Like the plugin, it says that the copy
  /// ended just before it returns.
  void finish(List<PickedFile> files) {
    _onCopying!(false);
    _result!.complete(files);
  }

  void fail(Object error) => _result!.completeError(error);

  @override
  Future<void> clearCopies() {
    clears++;
    return Future.value();
  }
}

PickedFile pickedFile(String name, {int size = 1 << 30}) =>
    PickedFile(name: name, path: '/cache/file_picker/1/$name', size: size);

/// A clipboard that holds [text].
class FakeClipboard implements TextClipboard {
  const FakeClipboard(this.text);
  final String? text;
  @override
  Future<String?> read() async => text;
}
