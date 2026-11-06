// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import 'file_source.dart';

/// What the person asked for from outside the app (#53): a launcher
/// shortcut, or files shared to TotalDrop from another app.
sealed class Launch {
  const Launch();
}

/// A launcher shortcut: open Send or Receive.
final class OpenScreen extends Launch {
  const OpenScreen(this.screen);
  final String screen;
}

/// Files were shared; their copy into the app cache has started.
final class ShareCopying extends Launch {
  const ShareCopying();
}

/// The shared files, copied and ready to send.
final class SharedFiles extends Launch {
  const SharedFiles(this.files);
  final List<PickedFile> files;
}

/// The shared files could not be copied (no space, a grant that is gone,
/// nothing that may be read).
final class ShareFailed extends Launch {
  const ShareFailed();
}

/// Only text was shared, which TotalDrop does not send yet.
final class SharedTextOnly extends Launch {
  const SharedTextOnly();
}

/// The launches in one event from the platform, in order. Anything that is
/// not understood is left out: the event crosses a process boundary.
@visibleForTesting
List<Launch> parseLaunch(Object? event) {
  if (event is! Map) return const [];
  final out = <Launch>[];
  if (event['textOnly'] == true) return const [SharedTextOnly()];
  if (event['copying'] == true) out.add(const ShareCopying());
  if (event['failed'] == true) out.add(const ShareFailed());
  final files = event['files'];
  if (files is List) {
    final picked = <PickedFile>[
      for (final f in files)
        if (f is Map &&
            f['name'] is String &&
            f['path'] is String &&
            f['size'] is int &&
            (f['path'] as String).startsWith('/'))
          PickedFile(
            name: f['name'] as String,
            path: f['path'] as String,
            size: f['size'] as int,
          ),
    ];
    out.add(picked.isEmpty ? const ShareFailed() : SharedFiles(picked));
  }
  final screen = event['screen'];
  if (out.isEmpty && (screen == 'send' || screen == 'receive')) {
    out.add(OpenScreen(screen as String));
  }
  return out;
}

/// Where launches come from: the Android activity, nothing elsewhere.
abstract interface class LaunchSource {
  /// Every launch, those from before the app listened first.
  Stream<Launch> get launches;

  /// Deletes the copies of shared files.
  Future<void> clearShared();
}

class NoLaunches implements LaunchSource {
  const NoLaunches();
  @override
  Stream<Launch> get launches => const Stream.empty();
  @override
  Future<void> clearShared() async {}
}

class AndroidLaunchSource implements LaunchSource {
  AndroidLaunchSource() {
    _channel.setMethodCallHandler((call) async {
      if (call.method == 'launch') _add(call.arguments);
    });
    _channel.invokeMethod<List<Object?>>('ready').then((before) {
      for (final e in before ?? const <Object?>[]) {
        _add(e);
      }
    }, onError: (Object e) => debugPrint('launches: $e'));
  }

  static const _channel = MethodChannel('app.totaldrop/launch');
  final _controller = StreamController<Launch>.broadcast();
  final _early = <Launch>[];
  bool _listened = false;

  void _add(Object? event) {
    for (final l in parseLaunch(event)) {
      if (_listened) {
        _controller.add(l);
      } else {
        _early.add(l);
      }
    }
  }

  @override
  Stream<Launch> get launches {
    if (!_listened) {
      _listened = true;
      // Delivered once the caller has subscribed.
      scheduleMicrotask(() {
        for (final l in _early) {
          _controller.add(l);
        }
        _early.clear();
      });
    }
    return _controller.stream;
  }

  @override
  Future<void> clearShared() =>
      _channel.invokeMethod<void>('clear').then((_) {}, onError: (_) {});
}
