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
  });
}
