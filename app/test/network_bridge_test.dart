// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/data/network_bridge.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const channel = MethodChannel('app.totaldrop/network_test');

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, null);
  });

  test('seeds initial gateway without firing network-changed signal', () async {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, (call) async {
          expect(call.method, 'ready');
          return <String, Object?>{'gateway': '192.168.1.1'};
        });

    final gateways = <String?>[];
    var changes = 0;
    final bridge = AndroidNetworkBridge(
      channel: channel,
      onGateway: gateways.add,
      onNetworkChanged: () => changes += 1,
    );

    await bridge.attach();
    expect(gateways, ['192.168.1.1']);
    expect(changes, 0);
    bridge.detach();
  });

  test('applies live gateway updates and notifies only when changed', () async {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, (call) async {
          return <String, Object?>{'gateway': '192.168.1.1'};
        });

    final gateways = <String?>[];
    var changes = 0;
    final bridge = AndroidNetworkBridge(
      channel: channel,
      onGateway: gateways.add,
      onNetworkChanged: () => changes += 1,
    );
    await bridge.attach();

    Future<void> push(Object? arguments) async {
      final encoded = const StandardMethodCodec().encodeMethodCall(
        MethodCall('update', arguments),
      );
      await TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .handlePlatformMessage(channel.name, encoded, (_) {});
    }

    // Handoff to cellular: gateway cleared and active transfers notified.
    await push(<String, Object?>{'gateway': null, 'changed': true});
    expect(gateways.last, isNull);
    expect(changes, 1);

    // Gateway learned after switching to office Wi-Fi without second change.
    await push(<String, Object?>{'gateway': '10.0.0.1', 'changed': false});
    expect(gateways.last, '10.0.0.1');
    expect(changes, 1);

    // Malformed payload is ignored safely.
    await push('unexpected');
    expect(gateways.length, 3);
    expect(changes, 1);

    bridge.detach();
  });
}
