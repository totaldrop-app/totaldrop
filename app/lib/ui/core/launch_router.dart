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

/// Opens the screen a launch asks for (#53): Send with the shared files or
/// text, Receive with the code of a link (#445), or the screen of a
/// launcher shortcut. A running send or receive is never
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
      case OpenScreen(screen: clipboardScreen):
        if (_blocked) {
          _showRunning();
          _tell((l) => l.shareBusy);
        } else {
          _show(sendRoute);
          // Android gives the clipboard only to the app whose window has
          // focus; the read waits for it (PlatformClipboard).
          WidgetsBinding.instance.addPostFrameCallback(
            (_) => unawaited(widget.send.pasteClipboard()),
          );
        }
      case OpenScreen(screen: final s):
        // Not away from a running transfer: its screen comes up instead.
        if (_blocked) {
          _showRunning();
        } else {
          _show(s == receiveRoute ? receiveRoute : sendRoute);
        }
      case ShareCopying(id: final id):
        if (!_blocked && widget.send.beginShare(id)) {
          _show(sendRoute);
        } else {
          widget.send.refuseShare(id);
          _showRunning();
          _tell((l) => l.shareBusy);
        }
      case SharedFiles(id: final id, files: final files, skipped: final n):
        if (widget.send.addShared(id, files) && n > 0) {
          _tell((l) => l.shareSkipped(n));
        }
      case ShareFailed(id: final id):
        widget.send.shareFailed(id);
      case OpenLink(link: final link):
        // A link without a code still opens Receive, to type one in.
        if (_blocked) {
          _showRunning();
          _tell((l) => l.linkBusy);
        } else {
          widget.receive.takeLink(link);
          _show(receiveRoute);
        }
      case SharedText(text: final t):
        if (!_blocked && widget.send.takeText(t)) {
          _show(sendRoute);
        } else {
          _showRunning();
          _tell((l) => l.shareBusy);
        }
    }
  }

  /// The screen of the transfer that blocks a launch.
  void _showRunning() {
    if (widget.send.busy) {
      _show(sendRoute);
    } else {
      _show(receiveRoute);
    }
  }

  /// Brings [name] to the top: closes the screens above it, or above the
  /// home screen and then opens it. A screen closed this way loses what
  /// was typed into it, such as a receive code not yet sent; a screen with
  /// a running transfer is never closed (see [_blocked]).
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
