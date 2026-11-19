// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The screens move only from the stage an action belongs to: a late
// answer, a second tap or an action on a finished transfer changes
// nothing.
import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

import 'fakes.dart';

class _HeldClipboard implements TextClipboard {
  final pending = Completer<String?>();

  @override
  Future<String?> read() => pending.future;
}

class _HeldFolderRepository extends FakeRepository {
  final folder = Completer<String>();

  @override
  Future<String> receiveFolder() => folder.future;
}

/// A receive view model showing [offer].
Future<ReceiveViewModel> offered(FakeRepository repo, FakeOffer offer) async {
  final vm = ReceiveViewModel(repository: repo);
  vm.setInput('7-harbor-rocket');
  unawaited(vm.connect());
  repo.connected.complete(offer);
  await pumpEventQueue();
  expect(vm.stage, ReceiveStage.offered);
  return vm;
}

TransferEvent finished() => const TransferEvent(
  kind: EventKind.finished,
  words: [],
  done: 10,
  total: 10,
  summary: TransferSummary(
    filesOk: 1,
    filesFailed: 0,
    bytes: 10,
    saved: [],
    text: null,
  ),
  error: null,
);

void main() {
  group('receive', () {
    test('a reset while declining stays reset', () async {
      final repo = FakeRepository();
      final vm = await offered(repo, FakeOffer());
      repo.declining = Completer();
      unawaited(vm.decline());
      vm.reset();
      expect(vm.stage, ReceiveStage.entering);
      repo.declining!.complete();
      await pumpEventQueue();
      expect(vm.stage, ReceiveStage.entering, reason: 'the decline was late');
    });

    test('declining twice declines once', () async {
      final repo = FakeRepository();
      final offer = FakeOffer();
      final vm = await offered(repo, offer);
      repo.declining = Completer();
      unawaited(vm.decline());
      unawaited(vm.decline());
      repo.declining!.complete();
      await pumpEventQueue();
      expect(vm.stage, ReceiveStage.declined);
      expect(repo.declined, [offer]);
    });

    test('accepting twice accepts once', () async {
      final repo = FakeRepository();
      final vm = await offered(repo, FakeOffer());
      await vm.accept();
      await vm.accept();
      expect(repo.accepted, ['files']);
    });

    test('a finished offer is not accepted or declined again', () async {
      final repo = FakeRepository();
      final vm = await offered(repo, FakeOffer());
      await vm.accept();
      repo.events.add(finished());
      await pumpEventQueue();
      expect(vm.stage, ReceiveStage.done);
      await vm.accept();
      await vm.decline();
      expect(vm.stage, ReceiveStage.done);
      expect(repo.accepted, ['files']);
      expect(repo.declined, isEmpty);
    });

    test('a declined offer is not accepted', () async {
      final repo = FakeRepository();
      final vm = await offered(repo, FakeOffer());
      await vm.decline();
      await vm.accept();
      expect(vm.stage, ReceiveStage.declined);
      expect(repo.accepted, isEmpty);
    });

    test(
      'disposing while connecting declines a late offer without notifying',
      () async {
        final repo = FakeRepository();
        final vm = ReceiveViewModel(repository: repo);
        vm.setInput('7-harbor-rocket');
        final connecting = vm.connect();
        expect(vm.stage, ReceiveStage.connecting);
        vm.dispose();
        expect(repo.stoppers.single.stopped, isTrue);
        final late = FakeOffer();
        repo.connected.complete(late);
        await connecting;
        await pumpEventQueue();
        expect(repo.declined, [late]);
        expect(vm.offer, isNull);
      },
    );

    test(
      'disposing while connecting ignores a late failure without notifying',
      () async {
        final repo = FakeRepository();
        final vm = ReceiveViewModel(repository: repo);
        vm.setInput('7-harbor-rocket');
        final connecting = vm.connect();
        vm.dispose();
        repo.connected.completeError(
          const TransferError(
            kind: ErrorKind.cancelled,
            detail: 'stopped',
            cause: '',
          ),
        );
        await connecting;
        expect(vm.error, isNull);
      },
    );

    test(
      'disposing while preparing the receive folder declines the offer',
      () async {
        final repo = _HeldFolderRepository();
        final vm = ReceiveViewModel(repository: repo);
        vm.setInput('7-harbor-rocket');
        final connecting = vm.connect();
        final offer = FakeOffer();
        repo.connected.complete(offer);
        await pumpEventQueue();
        vm.dispose();
        repo.folder.complete('/tmp/TotalDrop');
        await connecting;
        await pumpEventQueue();
        expect(repo.declined, [offer]);
        expect(vm.offer, isNull);
      },
    );
  });

  group('send', () {
    test('connected waits for the receiver to accept, as on Linux', () async {
      final repo = FakeRepository();
      final vm = SendViewModel(repository: repo);
      vm.addFakeFileForTest('a.txt');
      unawaited(vm.start());
      repo.prepared.complete(FakeSession());
      await pumpEventQueue();
      expect(vm.stage, SendStage.waiting);
      repo.events.add(
        const TransferEvent(
          kind: EventKind.connected,
          words: ['one', 'two', 'three', 'four'],
          done: 0,
          total: 0,
          summary: null,
          error: null,
        ),
      );
      await pumpEventQueue();
      expect(vm.stage, SendStage.connected);
      expect(vm.words, hasLength(4));
      expect(vm.busy, isTrue, reason: 'leaving asks first, as while sending');
      repo.events.add(
        const TransferEvent(
          kind: EventKind.progress,
          words: [],
          done: 1,
          total: 10,
          summary: null,
          error: null,
        ),
      );
      await pumpEventQueue();
      expect(vm.stage, SendStage.transferring);
    });

    test('stopping while the receiver decides stops the core', () async {
      final repo = FakeRepository();
      final vm = SendViewModel(repository: repo);
      vm.addFakeFileForTest('a.txt');
      unawaited(vm.start());
      repo.prepared.complete(FakeSession());
      await pumpEventQueue();
      repo.events.add(event(EventKind.connected));
      await pumpEventQueue();
      expect(vm.stage, SendStage.connected);
      vm.reset();
      expect(repo.stoppers.single.stopped, isTrue);
      expect(vm.stage, SendStage.picking);
    });

    test(
      'disposing while preparing releases the late session without notifying',
      () async {
        final repo = FakeRepository();
        final vm = SendViewModel(repository: repo);
        vm.addFakeFileForTest('a.txt');
        final starting = vm.start();
        expect(vm.stage, SendStage.preparing);
        vm.dispose();
        expect(repo.stoppers.single.stopped, isTrue);
        final late = FakeSession();
        repo.prepared.complete(late);
        await starting;
        expect(late.disposed, isTrue);
        expect(vm.code, isNull);
      },
    );

    test(
      'disposing while preparing ignores a late failure without notifying',
      () async {
        final repo = FakeRepository();
        final vm = SendViewModel(repository: repo);
        vm.addFakeFileForTest('a.txt');
        final starting = vm.start();
        vm.dispose();
        repo.prepared.completeError(
          const TransferError(
            kind: ErrorKind.cancelled,
            detail: 'stopped',
            cause: '',
          ),
        );
        await starting;
        expect(vm.error, isNull);
      },
    );

    test(
      'disposing while picking clears late copies without notifying',
      () async {
        final source = FakeFileSource();
        final vm = SendViewModel(
          repository: FakeRepository(),
          fileSource: source,
        );
        final adding = vm.addFiles();
        source.startCopy();
        vm.dispose();
        source.finish([pickedFile('film.mkv')]);
        await adding;
        await pumpEventQueue();
        expect(source.clears, 1);
        expect(vm.files, isEmpty);
      },
    );

    test(
      'resetting and re-entering text mode ignores an earlier clipboard read',
      () async {
        final clipboard = _HeldClipboard();
        final vm = SendViewModel(
          repository: FakeRepository(),
          textClipboard: clipboard,
        );
        final pasting = vm.pasteClipboard();
        expect(vm.textMode, isTrue);
        vm.reset();
        vm.setTextMode(true);
        clipboard.pending.complete('stale');
        await pasting;
        expect(vm.text, isEmpty);
        expect(vm.textNotice, TextNotice.none);
      },
    );

    test('disposing while reading the clipboard does not notify', () async {
      final clipboard = _HeldClipboard();
      final vm = SendViewModel(
        repository: FakeRepository(),
        textClipboard: clipboard,
      );
      final pasting = vm.pasteClipboard();
      vm.dispose();
      clipboard.pending.complete('late');
      await pasting;
      expect(vm.text, isEmpty);
    });
  });
}
