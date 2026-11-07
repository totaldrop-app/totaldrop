// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Sending and receiving a text (#326): what the send screen's text box
// holds, where the clipboard and a shared text go, and that a text offer
// is received as a text.
import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

import 'fakes.dart';

TransferEvent finished({int ok = 1, int failed = 0, String? text}) =>
    TransferEvent(
      kind: EventKind.finished,
      words: const [],
      done: 0,
      total: 0,
      summary: TransferSummary(
        filesOk: ok,
        filesFailed: failed,
        bytes: 1,
        saved: const [],
        text: text,
      ),
      error: null,
    );

void main() {
  group('send', () {
    late FakeRepository repo;

    SendViewModel model({String? clipboard}) => SendViewModel(
      repository: repo,
      textClipboard: FakeClipboard(clipboard),
    );

    setUp(() => repo = FakeRepository());

    test('the clipboard goes into an empty box exactly', () async {
      final vm = model(clipboard: 'a\r\nb c ');
      await vm.pasteClipboard();
      expect(vm.textMode, isTrue);
      expect(vm.text, 'a\r\nb c ');
      expect(vm.textNotice, TextNotice.none);
      expect(vm.canStart, isTrue);
    });

    test('typed text is kept, and the screen says so', () async {
      final vm = model(clipboard: 'other')
        ..setTextMode(true)
        ..setText('mine');
      await vm.pasteClipboard();
      expect(vm.text, 'mine');
      expect(vm.textNotice, TextNotice.clipboardKept);
    });

    test('an empty clipboard is said', () async {
      final vm = model();
      await vm.pasteClipboard();
      expect(vm.text, isEmpty);
      expect(vm.textNotice, TextNotice.clipboardEmpty);
      expect(vm.canStart, isFalse);
    });

    test('a shared text takes the place of the text in the box', () {
      final vm = model();
      expect(vm.takeText('one'), isTrue);
      expect(vm.takeText('two'), isTrue);
      expect(vm.text, 'two');
    });

    test('a shared text stops files still being copied from a share', () {
      final vm = model();
      expect(vm.beginShare(1), isTrue);
      expect(vm.takeText('hi'), isTrue);
      expect(vm.sharing, isFalse);
      expect(vm.addShared(1, [pickedFile('late.jpg')]), isFalse);
      expect(vm.textMode, isTrue);
      expect(vm.files, isEmpty);
    });

    test('shared files are shown and sent, not the text being written', () {
      final vm = model()..takeText('draft');
      expect(vm.beginShare(1), isTrue);
      expect(vm.textMode, isFalse);
      expect(vm.addShared(1, [pickedFile('a.jpg')]), isTrue);
      expect(vm.textMode, isFalse);
      expect(vm.files.single.name, 'a.jpg');
      // The text is still there for when the person goes back to it.
      expect(vm.text, 'draft');
    });

    test('leaving the screen while choosing forgets the text only', () {
      final vm = model()
        ..addFakeFileForTest('a.txt')
        ..takeText('secret');
      vm.leaveText();
      expect(vm.text, isEmpty);
      expect(vm.textMode, isFalse);
      expect(vm.files, hasLength(1));
      // Nothing old hides in front of the next shared text.
      vm.takeText('new');
      expect(vm.text, 'new');
    });

    test('a text over the limit cannot be sent', () {
      final vm = model()..setTextMode(true);
      vm.setText('é' * (repo.maxTextBytes ~/ 2));
      expect(vm.textTooLong, isFalse);
      vm.setText('${vm.text}x');
      expect(vm.textTooLong, isTrue);
      expect(vm.canStart, isFalse);
      expect(vm.bigText, isTrue);
    });

    test('the text is sent as a text, not the files listed', () async {
      final vm = model()
        ..addFakeFileForTest('a.txt')
        ..setTextMode(true)
        ..setText('hello');
      final started = vm.start();
      repo.prepared.complete(FakeSession());
      await started;
      expect(repo.sentTexts, ['hello']);
      repo.events.add(finished());
      await pumpEventQueue();
      expect(vm.stage, SendStage.done);
    });

    test('a text the receiver found damaged is a failure', () async {
      final vm = model()
        ..setTextMode(true)
        ..setText('hello');
      final started = vm.start();
      repo.prepared.complete(FakeSession());
      await started;
      repo.events.add(finished(ok: 0, failed: 1));
      await pumpEventQueue();
      expect(vm.stage, SendStage.failed);
      expect(vm.error?.kind, ErrorKind.textDamaged);
    });

    test('a reset forgets the text', () {
      final vm = model()..takeText('secret');
      vm.reset();
      expect(vm.text, isEmpty);
      expect(vm.textMode, isFalse);
    });
  });

  group('receive', () {
    test('a text offer is received as a text and kept in memory', () async {
      final repo = FakeRepository();
      final vm = ReceiveViewModel(repository: repo)..setInput('7-a-b');
      final connected = vm.connect();
      repo.connected.complete(FakeOffer(isText: true));
      await connected;
      expect(vm.isText, isTrue);
      await vm.accept();
      expect(repo.accepted, ['text']);
      repo.events.add(finished(text: 'hi\u202E'));
      await pumpEventQueue();
      expect(vm.stage, ReceiveStage.done);
      expect(vm.receivedText, 'hi\u202E');
      vm.reset();
      expect(vm.receivedText, isNull);
    });

    test('a text offer needs no folder', () async {
      final repo = FakeRepository()..folderError = Exception('no folder');
      final vm = ReceiveViewModel(repository: repo)..setInput('7-a-b');
      final connected = vm.connect();
      repo.connected.complete(FakeOffer(isText: true));
      await connected;
      expect(vm.stage, ReceiveStage.offered);
      expect(vm.folder, isNull);
      await vm.accept();
      expect(repo.accepted, ['text']);
    });

    test('a file offer is saved as files', () async {
      final repo = FakeRepository();
      final vm = ReceiveViewModel(repository: repo)..setInput('7-a-b');
      final connected = vm.connect();
      repo.connected.complete(FakeOffer());
      await connected;
      await vm.accept();
      expect(repo.accepted, ['files']);
    });
  });
}
