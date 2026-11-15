// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:flutter/services.dart';

import '../src/rust/api/transfer.dart' as rust;

/// Forwards Android's default-network IPv4 gateway (#406) and active-network
/// handoff signals (#32) from `MainActivity` (`app.totaldrop/network`) into
/// the Rust core.
class AndroidNetworkBridge {
  AndroidNetworkBridge({
    this.channel = _defaultChannel,
    void Function(String? gateway)? onGateway,
    void Function()? onNetworkChanged,
  }) : _onGateway = onGateway ?? _setRustGateway,
       _onNetworkChanged = onNetworkChanged ?? rust.notifyNetworkChanged;

  static const _defaultChannel = MethodChannel('app.totaldrop/network');

  final MethodChannel channel;
  final void Function(String? gateway) _onGateway;
  final void Function() _onNetworkChanged;

  static void _setRustGateway(String? gateway) {
    rust.setNetworkGateway(gateway: gateway);
  }

  /// Attach the channel listener and fetch the initial default gateway.
  static Future<AndroidNetworkBridge> start() async {
    final bridge = AndroidNetworkBridge();
    await bridge.attach();
    return bridge;
  }

  Future<void> attach() async {
    channel.setMethodCallHandler(_handleCall);
    try {
      final initial = await channel.invokeMapMethod<String, Object?>('ready');
      _applyGateway(initial);
    } on PlatformException {
      _onGateway(null);
    } on MissingPluginException {
      _onGateway(null);
    }
  }

  void detach() {
    channel.setMethodCallHandler(null);
  }

  Future<Object?> _handleCall(MethodCall call) async {
    if (call.method != 'update') return null;
    final args = call.arguments;
    if (args is! Map) return null;
    _applyGateway(args);
    if (args['changed'] == true) {
      _onNetworkChanged();
    }
    return null;
  }

  void _applyGateway(Map<Object?, Object?>? map) {
    final raw = map?['gateway'];
    _onGateway(raw is String && raw.isNotEmpty ? raw : null);
  }
}
