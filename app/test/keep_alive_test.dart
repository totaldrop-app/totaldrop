// A transfer keeps going in the background for exactly as long as it runs
// (#194): the view models ask for it when a send or a receive starts,
// release it whenever it ends, and the notification's Stop stops it.
import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

import 'fakes.dart';

TransferEvent finished() => TransferEvent(
  kind: EventKind.finished,
  words: const [],
  done: 0,
  total: 0,
  summary: const TransferSummary(
    filesOk: 1,
    filesFailed: 0,
    bytes: 1,
    saved: [],
  ),
  error: null,
);

TransferEvent stoppedEvent() => TransferEvent(
  kind: EventKind.failed,
  words: const [],
  done: 0,
  total: 0,
  summary: null,
  error: const TransferError(kind: ErrorKind.stopped, detail: ''),
);

void main() {
  group('send', () {
    late FakeRepository repo;
    late FakeKeepAlive keep;
    late SendViewModel vm;

    setUp(() {
      repo = FakeRepository();
      keep = FakeKeepAlive();
      vm = SendViewModel(repository: repo, keepAlive: keep);
      vm.addFakeFileForTest('a.txt');
    });

    test('is kept alive from the start, while the code waits', () async {
      unawaited(vm.start());
      expect(keep.calls, ['begin sending']);
      repo.prepared.complete(FakeSession());
      await pumpEventQueue();
      expect(vm.stage, SendStage.waiting);
      expect(keep.active, isTrue);
    });

    test('is released when the send finishes', () async {
      unawaited(vm.start());
      repo.prepared.complete(FakeSession());
      await pumpEventQueue();
      repo.events.add(finished());
      await pumpEventQueue();
      expect(vm.stage, SendStage.done);
      expect(keep.calls, ['begin sending', 'end']);
    });

    test('is released when the send fails', () async {
      unawaited(vm.start());
      repo.prepared.completeError(
        const TransferError(kind: ErrorKind.other, detail: 'x'),
      );
      await pumpEventQueue();
      expect(vm.stage, SendStage.failed);
      expect(keep.calls, ['begin sending', 'end']);
    });

    test('is released when the person goes back', () async {
      unawaited(vm.start());
      vm.reset();
      expect(keep.calls, ['begin sending', 'end']);
    });

    test('Stop on the notification stops the core', () async {
      unawaited(vm.start());
      repo.prepared.complete(FakeSession());
      await pumpEventQueue();
      keep.onStop!();
      expect(repo.stoppers.single.stopped, isTrue);
      // The core then reports the stop, which releases the keep-alive.
      repo.events.add(stoppedEvent());
      await pumpEventQueue();
      expect(vm.stage, SendStage.failed);
      expect(keep.calls, ['begin sending', 'end']);
    });

    test('is released when the screen goes away', () async {
      unawaited(vm.start());
      vm.dispose();
      expect(keep.calls, ['begin sending', 'end']);
    });
  });

  group('receive', () {
    late FakeRepository repo;
    late FakeKeepAlive keep;
    late ReceiveViewModel vm;

    setUp(() {
      repo = FakeRepository();
      keep = FakeKeepAlive();
      vm = ReceiveViewModel(repository: repo, keepAlive: keep);
      vm.setInput('7-harbor-rocket');
    });

    test('is kept alive from connecting to the end of the transfer', () async {
      unawaited(vm.connect());
      expect(keep.calls, ['begin receiving']);
      repo.connected.complete(FakeOffer());
      await pumpEventQueue();
      expect(vm.stage, ReceiveStage.offered);
      unawaited(vm.accept());
      expect(keep.active, isTrue);
      repo.events.add(finished());
      await pumpEventQueue();
      expect(vm.stage, ReceiveStage.done);
      expect(keep.calls, ['begin receiving', 'end']);
    });

    test('is released when the person declines', () async {
      unawaited(vm.connect());
      repo.connected.complete(FakeOffer());
      await pumpEventQueue();
      await vm.decline();
      expect(keep.calls, ['begin receiving', 'end']);
    });

    test('is released when connecting fails', () async {
      unawaited(vm.connect());
      repo.connected.completeError(
        const TransferError(kind: ErrorKind.other, detail: 'x'),
      );
      await pumpEventQueue();
      expect(vm.stage, ReceiveStage.failed);
      expect(keep.calls, ['begin receiving', 'end']);
    });

    test('Stop on the notification stops the core', () async {
      unawaited(vm.connect());
      keep.onStop!();
      expect(repo.stoppers.single.stopped, isTrue);
    });

    test('Stop on the notification declines an offer', () async {
      unawaited(vm.connect());
      final offer = FakeOffer();
      repo.connected.complete(offer);
      await pumpEventQueue();
      keep.onStop!();
      await pumpEventQueue();
      expect(repo.declined, [offer]);
      expect(vm.stage, ReceiveStage.declined);
      expect(keep.calls, ['begin receiving', 'end']);
    });

    test('is released when the folder cannot be made', () async {
      repo.folderError = const FileSystemException('read-only');
      unawaited(vm.connect());
      repo.connected.complete(FakeOffer());
      await pumpEventQueue();
      expect(vm.stage, ReceiveStage.failed);
      expect(keep.calls, ['begin receiving', 'end']);
    });
  });
}
