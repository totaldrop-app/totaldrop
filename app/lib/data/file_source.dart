// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:file_picker/file_picker.dart';

/// A file the person chose to send, as a path the core can read.
class PickedFile {
  const PickedFile({
    required this.name,
    required this.path,
    required this.size,
    this.sizeKnown = true,
  });
  final String name;
  final String path;
  final int size;

  /// Whether [size] came from the file rather than a fallback when the picker
  /// could not read its length (#593).
  final bool sizeKnown;
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
    final files = <PickedFile>[];
    for (final f in picked) {
      if (f.path case final path?) {
        final length = f.lengthSync() ?? await f.length();
        files.add(
          PickedFile(
            name: f.name,
            path: path,
            size: length ?? 0,
            sizeKnown: length != null,
          ),
        );
      }
    }
    return files;
  }

  @override
  Future<void> clearCopies() =>
      FilePicker.clearTemporaryFiles().then((_) {}, onError: (_) {});
}
