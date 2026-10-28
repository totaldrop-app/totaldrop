// Stopping a transfer from the view models, at every stage, with a fake
// repository in place of the Rust bridge.
import 'dart:async';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart' show PlatformInt64;
import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/data/transfer_repository.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

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

  @override
  Stream<TransferEvent> runSend(SendSession session, Stopper stopper) => events.stream;

  @override
  Future<Offer> connect(String code, Stopper stopper) => connected.future;

  @override
  Stream<TransferEvent> accept(Offer offer, String destination, Stopper stopper) => events.stream;

  @override
  Future<void> decline(Offer offer) async => declined.add(offer);

  @override
  String? normalizeCode(String input) => input;

  @override
  Future<String> receiveFolder() async => '/tmp/TotalDrop';

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

TransferEvent _event(EventKind kind) =>
    TransferEvent(kind: kind, words: const [], done: 0, total: 0, summary: null, error: null);

void main() {
  group('send', () {
    test('stopping while preparing stops the core and drops the late session', () async {
      final repo = FakeRepository();
      final vm = SendViewModel(repository: repo);
      vm.addFakeFileForTest('a.txt');
      unawaited(vm.start());
      expect(vm.stage, SendStage.preparing);
      vm.reset();
      expect(repo.stoppers.single.stopped, isTrue);
      final late = FakeSession();
      repo.prepared.complete(late);
      await pumpEventQueue();
      expect(vm.stage, SendStage.picking);
      expect(vm.code, isNull);
      expect(late.disposed, isTrue, reason: 'the reserved code is released at once');
    });

    test('a new send after a reset uses a new stopper and ignores the old one', () async {
      final repo = FakeRepository();
      final vm = SendViewModel(repository: repo);
      vm.addFakeFileForTest('a.txt');
      unawaited(vm.start());
      final first = repo.prepared;
      vm.reset();
      vm.addFakeFileForTest('b.txt');
      repo.prepared = Completer();
      unawaited(vm.start());
      first.complete(FakeSession());
      await pumpEventQueue();
      expect(vm.stage, SendStage.preparing, reason: 'the old result changes nothing');
      repo.prepared.complete(FakeSession());
      await pumpEventQueue();
      expect(vm.stage, SendStage.waiting);
      expect(repo.stoppers, hasLength(2));
      expect(repo.stoppers.first.stopped, isTrue);
      expect(repo.stoppers.last.stopped, isFalse);
    });

    test('stopping while waiting or transferring stops the core', () async {
      final repo = FakeRepository();
      final vm = SendViewModel(repository: repo);
      vm.addFakeFileForTest('a.txt');
      unawaited(vm.start());
      repo.prepared.complete(FakeSession());
      await pumpEventQueue();
      repo.events.add(_event(EventKind.connected));
      await pumpEventQueue();
      expect(vm.stage, SendStage.transferring);
      vm.cancel();
      vm.cancel(); // twice is fine
      expect(repo.stoppers.single.stopped, isTrue);
    });
  });

  group('receive', () {
    test('stopping while connecting declines an offer that arrives late', () async {
      final repo = FakeRepository();
      final vm = ReceiveViewModel(repository: repo);
      vm.setInput('7-harbor-rocket');
      unawaited(vm.connect());
      expect(vm.stage, ReceiveStage.connecting);
      vm.reset();
      expect(repo.stoppers.single.stopped, isTrue);
      final late = FakeOffer();
      repo.connected.complete(late);
      await pumpEventQueue();
      expect(vm.stage, ReceiveStage.entering);
      expect(vm.offer, isNull);
      expect(repo.declined, [late]);
    });

    test('accepting uses the same stopper as connecting', () async {
      final repo = FakeRepository();
      final vm = ReceiveViewModel(repository: repo);
      vm.setInput('7-harbor-rocket');
      unawaited(vm.connect());
      repo.connected.complete(FakeOffer());
      await pumpEventQueue();
      expect(vm.stage, ReceiveStage.offered);
      await vm.accept();
      expect(vm.stage, ReceiveStage.transferring);
      vm.cancel();
      expect(repo.stoppers.single.stopped, isTrue);
    });

    test('closing the screen on an offer declines it', () async {
      final repo = FakeRepository();
      final vm = ReceiveViewModel(repository: repo);
      vm.setInput('7-harbor-rocket');
      unawaited(vm.connect());
      final offer = FakeOffer();
      repo.connected.complete(offer);
      await pumpEventQueue();
      vm.dispose();
      await pumpEventQueue();
      expect(repo.declined, [offer]);
    });
  });
}
