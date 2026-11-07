// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:material_ui/material_ui.dart';

import '../../data/background_access.dart';
import '../../data/settings_repository.dart';
import '../../data/transfer_keep_alive.dart';
import 'background_setup_view.dart';

/// Knows whether a transfer is running: the keep-alive both screens use,
/// passed through. Listeners hear when a transfer starts or ends.
class WatchedKeepAlive extends ChangeNotifier implements TransferKeepAlive {
  WatchedKeepAlive(this._inner);

  final TransferKeepAlive _inner;
  bool _active = false;

  bool get active => _active;

  void _set(bool on) {
    if (_active == on) return;
    _active = on;
    notifyListeners();
  }

  @override
  Future<void> begin(TransferDirection direction, void Function() onStop) {
    _set(true);
    return _inner.begin(direction, () {
      _set(false);
      onStop();
    });
  }

  @override
  Future<void> end() {
    _set(false);
    return _inner.end();
  }
}

/// Decides when the background page is shown (#338).
class BackgroundPrompts {
  BackgroundPrompts({required this.settings, required this.access});

  final SettingsRepository settings;
  final BackgroundAccess access;

  /// At first start: the page, unless it was shown before or nothing
  /// restricts TotalDrop. Marks it seen when nothing needs setting up.
  Future<bool> showAtStart() async {
    if (settings.backgroundSetupSeen) return false;
    final state = await access.state();
    if (state.settled) {
      await settings.setBackgroundSetupSeen(skipped: false);
      return false;
    }
    return true;
  }

  /// Back on screen after a transfer went to the background: the page once,
  /// when the person skipped it at first start or Android's battery
  /// optimisation is still on. Not before the first-run page was answered,
  /// which a share can open the app past. The caller marks it shown.
  Future<bool> shouldRemind() async {
    if (!settings.backgroundSetupSeen || settings.backgroundReminded) {
      return false;
    }
    final state = await access.state();
    if (state.settled) return false;
    return !state.exempt || settings.backgroundSkipped;
  }
}

/// Shows the first-run page before [child], once.
class FirstRunGate extends StatefulWidget {
  const FirstRunGate({super.key, required this.prompts, required this.child});

  final BackgroundPrompts prompts;
  final Widget child;

  @override
  State<FirstRunGate> createState() => _FirstRunGateState();
}

class _FirstRunGateState extends State<FirstRunGate> {
  bool? _setup;

  @override
  void initState() {
    super.initState();
    widget.prompts.showAtStart().then((show) {
      if (mounted) setState(() => _setup = show);
    });
  }

  @override
  Widget build(BuildContext context) => switch (_setup) {
    // A frame or two while the state is read.
    null => const Scaffold(),
    true => BackgroundSetupView(
      access: widget.prompts.access,
      firstRun: true,
      onDone: ({required skipped}) async {
        await widget.prompts.settings.setBackgroundSetupSeen(skipped: skipped);
        if (mounted) setState(() => _setup = false);
      },
    ),
    false => widget.child,
  };
}

/// Watches the app leave the screen during a transfer and, when it is
/// back, shows the background page once if the restrictions are still on.
class BackgroundReminder extends StatefulWidget {
  const BackgroundReminder({
    super.key,
    required this.prompts,
    required this.keepAlive,
    required this.navigator,
    required this.child,
  });

  final BackgroundPrompts prompts;
  final WatchedKeepAlive keepAlive;
  final GlobalKey<NavigatorState> navigator;
  final Widget child;

  @override
  State<BackgroundReminder> createState() => _BackgroundReminderState();
}

class _BackgroundReminderState extends State<BackgroundReminder>
    with WidgetsBindingObserver {
  /// A transfer was running while the app was off screen.
  bool _left = false;

  /// On screen again, waiting for that transfer to end: the page does not
  /// cover a transfer still running.
  bool _waiting = false;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    widget.keepAlive.addListener(_transferChanged);
  }

  @override
  void dispose() {
    widget.keepAlive.removeListener(_transferChanged);
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    switch (state) {
      case AppLifecycleState.hidden || AppLifecycleState.paused:
        if (widget.keepAlive.active) _left = true;
        _waiting = false;
      case AppLifecycleState.resumed:
        if (_left) {
          _left = false;
          _waiting = true;
          _transferChanged();
        }
      case AppLifecycleState.inactive || AppLifecycleState.detached:
        break;
    }
  }

  void _transferChanged() {
    if (_waiting && !widget.keepAlive.active) {
      _waiting = false;
      _remind();
    }
  }

  Future<void> _remind() async {
    if (_busy) return;
    _busy = true;
    try {
      if (!await widget.prompts.shouldRemind()) return;
      final navigator = widget.navigator.currentState;
      // Opened from Settings already: nothing to remind of.
      if (!mounted || navigator == null || BackgroundSetupView.shown > 0) {
        return;
      }
      await widget.prompts.settings.setBackgroundReminded();
      await navigator.push(
        MaterialPageRoute<void>(
          builder: (context) => BackgroundSetupView(
            access: widget.prompts.access,
            onDone: ({required skipped}) => Navigator.pop(context),
          ),
        ),
      );
    } finally {
      _busy = false;
    }
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
