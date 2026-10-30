// While connecting, the receive screen shows the code as Rust read it,
// not the text that was typed or pasted (#149).
import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/data/transfer_repository.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';

class _Stopper implements Stopper {
  @override
  dynamic noSuchMethod(Invocation invocation) => null;
}

class _Repository implements TransferRepository {
  final connected = Completer<Offer>();
  final codes = <String>[];

  // Stands in for the Rust parser. What it returns differs from the typed
  // text on purpose: the screen must show the parser's output, whatever
  // the input looks like.
  @override
  String? normalizeCode(String input) =>
      input.contains('rocket') ? '7-harbor-rocket' : null;

  @override
  Stopper newStopper() => _Stopper();

  @override
  void stop(Stopper stopper) {}

  @override
  Future<Offer> connect(String code, Stopper stopper) {
    codes.add(code);
    return connected.future;
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  test('connecting shows the code that is used, not the typed text', () {
    final repo = _Repository();
    final vm = ReceiveViewModel(repository: repo);
    vm.setInput('\u202E7-rocket-harbor');
    expect(vm.connectingCode, isNull);
    unawaited(vm.connect());
    expect(vm.stage, ReceiveStage.connecting);
    expect(vm.connectingCode, '7-harbor-rocket');
    expect(repo.codes, ['7-harbor-rocket']);
    vm.reset();
    expect(vm.connectingCode, isNull);
  });

  test('an invalid code shows nothing and does not connect', () {
    final repo = _Repository();
    final vm = ReceiveViewModel(repository: repo);
    vm.setInput('7-harbor');
    unawaited(vm.connect());
    expect(vm.stage, ReceiveStage.entering);
    expect(vm.showInvalid, isTrue);
    expect(vm.connectingCode, isNull);
    expect(repo.codes, isEmpty);
  });
}
