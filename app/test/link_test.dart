// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// A TotalDrop link that opens the app (#445) only fills in the code, and
// only when Receive can take a new one: never over a running transfer, an
// offer waiting for an answer or a received text, which is nowhere else.
import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';

import 'fakes.dart';

const _link = 'totaldrop://r/7-harbor-rocket';

TransferEvent _finished({String? text}) => TransferEvent(
  kind: EventKind.finished,
  words: const [],
  done: 0,
  total: 0,
  summary: TransferSummary(
    filesOk: text == null ? 1 : 0,
    filesFailed: 0,
    bytes: 1,
    saved: const [],
    text: text,
  ),
  error: null,
);

void main() {
  late FakeRepository repo;
  late ReceiveViewModel vm;

  setUp(() {
    repo = FakeRepository();
    vm = ReceiveViewModel(repository: repo);
  });

  /// Connects with a typed code and waits for the offer.
  Future<void> offered({bool text = false}) async {
    vm.setInput('3-old-code');
    final connected = vm.connect();
    repo.connected.complete(FakeOffer(isText: text));
    await connected;
    expect(vm.stage, ReceiveStage.offered);
  }

  test('a link fills in its code and does not connect', () {
    vm.setInput('3-old-code');
    expect(vm.takeLink(_link), isTrue);
    expect(vm.input, '7-harbor-rocket');
    expect(vm.stage, ReceiveStage.entering);
    expect(repo.stoppers, isEmpty, reason: 'nothing was started');
  });

  test('a link whose code is not a code empties the box and says so', () {
    vm.setInput('3-old-code');
    expect(vm.takeLink('totaldrop://r/x?c=7-harbor-rocket'), isFalse);
    expect(vm.input, isEmpty, reason: 'not the code typed before');
    expect(vm.showInvalid, isTrue);
  });

  test('a link with nothing where the code goes changes nothing', () {
    vm.setInput('3-old-code');
    for (final link in [
      'totaldrop://r',
      'totaldrop://r/',
      'https://totaldrop.app/r',
      'https://totaldrop.app/r#',
    ]) {
      expect(vm.takeLink(link), isFalse, reason: link);
      expect(vm.input, '3-old-code', reason: link);
      expect(vm.showInvalid, isFalse, reason: link);
    }
  });

  test('a link while connecting changes nothing', () async {
    vm.setInput('3-old-code');
    final connecting = vm.connect();
    expect(vm.stage, ReceiveStage.connecting);
    expect(vm.canTakeLink, isFalse);
    expect(vm.takeLink(_link), isFalse);
    expect(vm.input, '3-old-code');
    repo.connected.complete(FakeOffer());
    await connecting;
  });

  test('a link while an offer waits for an answer changes nothing', () async {
    await offered();
    expect(vm.canTakeLink, isFalse);
    expect(vm.takeLink(_link), isFalse);
    expect(vm.stage, ReceiveStage.offered);
    expect(vm.offer, isNotNull);
  });

  test('a link leaves a received text on the screen', () async {
    await offered(text: true);
    await vm.accept();
    repo.events.add(_finished(text: 'only here'));
    await pumpEventQueue();
    expect(vm.stage, ReceiveStage.done);
    expect(vm.canTakeLink, isFalse);
    expect(vm.takeLink(_link), isFalse);
    expect(vm.receivedText, 'only here');
  });

  test('after received files, a link starts over with its code', () async {
    await offered();
    await vm.accept();
    repo.events.add(_finished());
    await pumpEventQueue();
    expect(vm.stage, ReceiveStage.done);
    expect(vm.takeLink(_link), isTrue);
    expect(vm.stage, ReceiveStage.entering);
    expect(vm.input, '7-harbor-rocket');
    expect(vm.summary, isNull);
  });
}
