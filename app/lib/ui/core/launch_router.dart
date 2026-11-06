// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../../data/launch_source.dart';
import '../../l10n/app_localizations.dart';
import '../receive/receive_view_model.dart';
import '../send/send_view_model.dart';

/// Route names, so a launch finds a screen that is already open.
const sendRoute = 'send';
const receiveRoute = 'receive';

/// Opens the screen a launch asks for (#53): Send with the shared files,
/// or the screen of a launcher shortcut. A running send or receive is never
/// cut short for it: the person is told instead.
class LaunchRouter extends StatefulWidget {
  const LaunchRouter({
    super.key,
    required this.launches,
    required this.navigator,
    required this.messenger,
    required this.send,
    required this.receive,
    required this.openSend,
    required this.openReceive,
    required this.child,
  });

  final LaunchSource launches;
  final GlobalKey<NavigatorState> navigator;
  final GlobalKey<ScaffoldMessengerState> messenger;
  final SendViewModel send;
  final ReceiveViewModel receive;
  final Route<void> Function() openSend;
  final Route<void> Function() openReceive;
  final Widget child;

  @override
  State<LaunchRouter> createState() => _LaunchRouterState();
}

class _LaunchRouterState extends State<LaunchRouter> {
  StreamSubscription<Launch>? _sub;

  @override
  void initState() {
    super.initState();
    _sub = widget.launches.launches.listen(_on);
  }

  @override
  void dispose() {
    _sub?.cancel();
    super.dispose();
  }

  /// A send or receive that leaving its screen would end, or a receive
  /// that waits for the person's answer.
  bool get _blocked =>
      widget.send.busy ||
      widget.receive.busy ||
      widget.receive.stage == ReceiveStage.offered;

  void _on(Launch launch) {
    switch (launch) {
      case OpenScreen(screen: final s):
        if (!_blocked) _show(s == receiveRoute ? receiveRoute : sendRoute);
      case ShareCopying():
        if (!_blocked && widget.send.beginShare()) {
          _show(sendRoute);
        } else {
          _tell((l) => l.shareBusy);
        }
      case SharedFiles(files: final files):
        widget.send.addShared(files);
      case ShareFailed():
        widget.send.shareFailed();
      case SharedTextOnly():
        if (!_blocked) _show(sendRoute);
        _tell((l) => l.shareTextSoon);
    }
  }

  /// Brings [name] to the top: closes the screens above it, or above the
  /// home screen and then opens it.
  void _show(String name) {
    final nav = widget.navigator.currentState;
    if (nav == null) return;
    var found = false;
    nav.popUntil((r) {
      if (r.settings.name == name) found = true;
      return r.isFirst || found;
    });
    if (!found) {
      nav.push(name == sendRoute ? widget.openSend() : widget.openReceive());
    }
  }

  void _tell(String Function(AppLocalizations) text) {
    // After the frame that shows the screen, so the message lands on it.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final messenger = widget.messenger.currentState;
      final context = widget.navigator.currentContext;
      if (messenger == null || context == null) return;
      messenger.showSnackBar(
        SnackBar(content: Text(text(AppLocalizations.of(context)))),
      );
    });
    // Nothing else may change on screen, so ask for that frame.
    WidgetsBinding.instance.scheduleFrame();
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
