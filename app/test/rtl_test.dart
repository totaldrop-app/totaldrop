import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/ui/core/l10n.dart';

/// The interface mirrors in right-to-left languages (Arabic, Persian,
/// Hebrew, Urdu), so layout code names the start and end of a line, never
/// its left and right (ADR 0008). A left alignment once kept the home
/// screen's mark on the left while the Arabic title sat on the right.
void main() {
  final physical = <RegExp>[
    RegExp(r'\bAlignment\.(centerLeft|centerRight|topLeft|topRight|bottomLeft|bottomRight)\b'),
    RegExp(r'\bTextAlign\.(left|right)\b'),
    RegExp(r'\bEdgeInsets\.only\([^)]*\b(left|right):'),
    RegExp(r'\bPositioned\([^)]*\b(left|right):'),
    RegExp(r'\bBorderRadius\.only\([^)]*\b(topLeft|topRight|bottomLeft|bottomRight):'),
  ];
  // fromLTRB is fine when both sides are the same.
  final ltrb = RegExp(r'EdgeInsets\.fromLTRB\(\s*([^,]+),\s*[^,]+,\s*([^,]+),');

  test('layout uses start and end, not left and right', () {
    final problems = <String>[];
    final files = Directory('lib/ui')
        .listSync(recursive: true)
        .whereType<File>()
        .where((f) => f.path.endsWith('.dart'));
    for (final file in files) {
      final lines = file.readAsLinesSync();
      for (var i = 0; i < lines.length; i++) {
        final line = lines[i];
        for (final re in physical) {
          if (re.hasMatch(line)) problems.add('${file.path}:${i + 1}: ${line.trim()}');
        }
        for (final m in ltrb.allMatches(line)) {
          if (m.group(1)!.trim() != m.group(2)!.trim()) {
            problems.add('${file.path}:${i + 1}: ${line.trim()}');
          }
        }
      }
    }
    expect(problems, isEmpty, reason: problems.join('\n'));
  });

  test('values put into a sentence keep their own direction', () {
    // In Arabic, "Version 0.2.0-beta.2" once showed as "beta.2-0.2.0".
    expect(isolate('0.2.0-beta.2'), '\u20680.2.0-beta.2\u2069');
    expect(isolate('Download/TotalDrop'), '\u2068Download/TotalDrop\u2069');
    final settings = File('lib/ui/settings/settings_view.dart').readAsStringSync();
    final receive = File('lib/ui/receive/receive_view.dart').readAsStringSync();
    expect(settings, contains('l.settingsVersion(isolate('));
    expect(receive, contains('l.receiveSaveTo(isolate('));
  });
}
