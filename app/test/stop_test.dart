// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Stopping a transfer from the view models, at every stage, with a fake
// repository in place of the Rust bridge.
import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

import 'fakes.dart';

void main() {
  group('send', () {
    test(
      'stopping while preparing stops the core and drops the late session',
      () async {
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
        expect(
          late.disposed,
          isTrue,
          reason: 'the reserved code is released at once',
        );
      },
    );

    test(
      'a new send after a reset uses a new stopper and ignores the old one',
      () async {
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
        expect(
          vm.stage,
          SendStage.preparing,
          reason: 'the old result changes nothing',
        );
        repo.prepared.complete(FakeSession());
        await pumpEventQueue();
        expect(vm.stage, SendStage.waiting);
        expect(repo.stoppers, hasLength(2));
        expect(repo.stoppers.first.stopped, isTrue);
        expect(repo.stoppers.last.stopped, isFalse);
      },
    );

    test('stopping while waiting or transferring stops the core', () async {
      final repo = FakeRepository();
      final vm = SendViewModel(repository: repo);
      vm.addFakeFileForTest('a.txt');
      unawaited(vm.start());
      repo.prepared.complete(FakeSession());
      await pumpEventQueue();
      repo.events.add(event(EventKind.connected));
      await pumpEventQueue();
      expect(vm.stage, SendStage.transferring);
      vm.cancel();
      vm.cancel(); // twice is fine
      expect(repo.stoppers.single.stopped, isTrue);
    });
  });

  group('receive', () {
    test(
      'stopping while connecting declines an offer that arrives late',
      () async {
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
      },
    );

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
