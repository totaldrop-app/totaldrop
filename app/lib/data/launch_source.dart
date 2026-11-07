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

/// A launcher shortcut: open Send or Receive, or Send with the text on
/// the clipboard ([clipboardScreen]).
final class OpenScreen extends Launch {
  const OpenScreen(this.screen);
  final String screen;
}

/// Files were shared; their copy into the app's storage has started. [id]
/// names the share in the events that follow, and in
/// [LaunchSource.drop].
final class ShareCopying extends Launch {
  const ShareCopying(this.id);
  final int id;
}

/// The shared files of share [id], copied and ready to send. [skipped]
/// items were left out: they could not, or may not, be read.
final class SharedFiles extends Launch {
  const SharedFiles(this.id, this.files, {this.skipped = 0});
  final int id;
  final List<PickedFile> files;
  final int skipped;
}

/// The files of share [id] could not be copied (no space, a grant that is
/// gone, nothing that may be read).
final class ShareFailed extends Launch {
  const ShareFailed(this.id);
  final int id;
}

/// The launcher shortcut that opens Send with the clipboard's text
/// (#326).
const clipboardScreen = 'clipboard';

/// A text was shared, or selected and sent from another app's text
/// selection menu (#326): it opens Send with the text filled in.
final class SharedText extends Launch {
  const SharedText(this.text);
  final String text;
}

/// The launches in one event from the platform, in order. Anything that is
/// not understood is left out: the event crosses a process boundary.
@visibleForTesting
List<Launch> parseLaunch(Object? event) {
  if (event is! Map) return const [];
  final text = event['text'];
  if (text is String) return text.isEmpty ? const [] : [SharedText(text)];
  final id = event['id'];
  if (id is int) {
    if (event['copying'] == true) return [ShareCopying(id)];
    if (event['failed'] == true) return [ShareFailed(id)];
    final files = event['files'];
    if (files is! List) return const [];
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
    if (picked.isEmpty) return [ShareFailed(id)];
    final skipped = event['skipped'];
    return [
      SharedFiles(
        id,
        picked,
        skipped:
            (skipped is int && skipped > 0 ? skipped : 0) +
            files.length -
            picked.length,
      ),
    ];
  }
  final screen = event['screen'];
  if (screen == 'send' || screen == 'receive' || screen == clipboardScreen) {
    return [OpenScreen(screen as String)];
  }
  return const [];
}

/// Where launches come from: the Android activity, nothing elsewhere.
abstract interface class LaunchSource {
  /// Every launch, those from before the app listened first.
  Stream<Launch> get launches;

  /// Stops the copies of these shares and deletes them: the person cannot
  /// take them, or cleared them.
  Future<void> drop(Iterable<int> ids);
}

class NoLaunches implements LaunchSource {
  const NoLaunches();
  @override
  Stream<Launch> get launches => const Stream.empty();
  @override
  Future<void> drop(Iterable<int> ids) async {}
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
  Future<void> drop(Iterable<int> ids) {
    if (ids.isEmpty) return Future.value();
    return _channel
        .invokeMethod<void>('drop', ids.toList())
        .then((_) {}, onError: (Object e) => debugPrint('launches: $e'));
  }
}
