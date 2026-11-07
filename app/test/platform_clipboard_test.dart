// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Send clipboard from a launcher shortcut (#326): Android 10 and later
// give the clipboard only to the app whose window has focus, so the read
// waits for it.
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/ui/send/send_view_model.dart';

void main() {
  late List<AppLifecycleState?> readIn;

  setUp(() => readIn = []);

  void mockClipboard(WidgetTester tester) {
    tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
      SystemChannels.platform,
      (call) async {
        if (call.method != 'Clipboard.getData') return null;
        final state = tester.binding.lifecycleState;
        readIn.add(state);
        // Like Android: nothing for an app without focus.
        return state == AppLifecycleState.resumed
            ? <String, dynamic>{'text': 'hello'}
            : null;
      },
    );
    addTearDown(
      () => tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
        SystemChannels.platform,
        null,
      ),
    );
  }

  testWidgets('the read waits until the window has focus', (tester) async {
    mockClipboard(tester);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
    String? text;
    final read = const PlatformClipboard().read().then((t) => text = t);
    await tester.pump(const Duration(milliseconds: 500));
    expect(readIn, isEmpty);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();
    await read;
    expect(readIn, [AppLifecycleState.resumed]);
    expect(text, 'hello');
  });

  testWidgets('a window that has focus is read at once', (tester) async {
    mockClipboard(tester);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    expect(await const PlatformClipboard().read(), 'hello');
  });

  testWidgets('without focus the read gives up after a while', (tester) async {
    mockClipboard(tester);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
    String? text = 'unset';
    var done = false;
    final read = const PlatformClipboard(focusWait: Duration(seconds: 1))
        .read()
        .then((t) {
          text = t;
          done = true;
        });
    await tester.pump(const Duration(seconds: 2));
    await read;
    expect(done, isTrue);
    expect(text, isNull);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
  });
}
