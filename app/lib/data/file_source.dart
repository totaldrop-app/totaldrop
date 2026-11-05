// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:file_picker/file_picker.dart';

/// A file the person chose to send, as a path the core can read.
class PickedFile {
  const PickedFile({
    required this.name,
    required this.path,
    required this.size,
  });
  final String name;
  final String path;
  final int size;
}

/// Where the files to send come from: the system picker on a device, a
/// stand-in in tests.
abstract interface class FileSource {
  /// Lets the person choose files. Resolves to an empty list when they
  /// cancel.
  ///
  /// On Android the picker copies every chosen file into the app cache
  /// before it returns, which takes seconds for a large file (#269).
  /// [onCopying] is told `true` when that copy starts and `false` when it
  /// ends; the `false` may never come, so the caller must not wait for it.
  Future<List<PickedFile>> pick({
    required void Function(bool copying) onCopying,
  });

  /// Deletes the picker's copies in the app cache.
  Future<void> clearCopies();
}

class PlatformFileSource implements FileSource {
  const PlatformFileSource();

  @override
  Future<List<PickedFile>> pick({
    required void Function(bool copying) onCopying,
  }) async {
    // Multiple selection; files are copied to the app cache, never loaded
    // into memory.
    final picked = await FilePicker.pickFiles(
      onFileLoading: (status) => onCopying(status == FilePickerStatus.picking),
    );
    return [
      for (final f in picked)
        if (f.path case final path?)
          PickedFile(
            name: f.name,
            path: path,
            size: f.lengthSync() ?? await f.length() ?? 0,
          ),
    ];
  }

  @override
  Future<void> clearCopies() =>
      FilePicker.clearTemporaryFiles().then((_) {}, onError: (_) {});
}
