import 'dart:io';

import 'package:analyzer/dart/analysis/utilities.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/token.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/ui/core/l10n.dart';

/// The interface mirrors in right-to-left languages (Arabic, Persian,
/// Hebrew, Urdu), so layout code names the start and end of a line, never
/// its left and right (ADR 0008). A left alignment once kept the home
/// screen's mark on the left while the Arabic title sat on the right.
void main() {
  test('layout uses start and end, not left and right', () {
    final problems = <String>[];
    final files = Directory('lib')
        .listSync(recursive: true)
        .whereType<File>()
        .where((f) => f.path.endsWith('.dart'));
    for (final file in files) {
      problems.addAll(physicalSides(file.readAsStringSync(), file.path));
    }
    expect(problems, isEmpty, reason: problems.join('\n'));
  });

  test('the check reads whole calls, not single lines', () {
    // A line-by-line check missed all of these: the side sits on a later
    // line than the call it belongs to.
    const fixture = '''
Widget build() => Stack(children: [
  Padding(
    padding: const EdgeInsets.only(
      top: 8,
      left: 16,
    ),
  ),
  Positioned(
    top: 0,
    right: 0,
    child: x,
  ),
  Container(
    padding: EdgeInsets.fromLTRB(
      8,
      0,
      16,
      0,
    ),
    decoration: BoxDecoration(
      borderRadius: BorderRadius.only(
        topLeft: r,
      ),
    ),
  ),
  // TextAlign.left in a comment is not code.
  Text('Alignment.centerLeft', textAlign: TextAlign
      .right),
  AnimatedPositioned(
    duration: d,
    left: 0,
    child: x,
  ),
  Align(alignment: FractionalOffset
      .topRight),
]);
''';
    expect(physicalSides(fixture, 'fixture.dart'), [
      'fixture.dart:3: EdgeInsets.only',
      'fixture.dart:8: Positioned',
      'fixture.dart:14: EdgeInsets.fromLTRB',
      'fixture.dart:21: BorderRadius.only',
      'fixture.dart:27: TextAlign.right',
      'fixture.dart:29: AnimatedPositioned',
      'fixture.dart:34: FractionalOffset.topRight',
    ]);
  });

  test('start and end, and equal sides, pass', () {
    const fixture = '''
final a = [
  const EdgeInsetsDirectional.only(start: 16, end: 8),
  PositionedDirectional(start: 0, child: x),
  EdgeInsets.fromLTRB(16, 0, 16, 8),
  EdgeInsets.fromLTRB(gap * 2, 0, gap*2, 8),
  AnimatedPositionedDirectional(duration: d, end: 0, child: x),
  FractionalOffset.center,
  BorderRadiusDirectional.only(topStart: r),
  Alignment.center,
  AlignmentDirectional.centerStart,
  TextAlign.start,
  EdgeInsets.only(top: 4),
];
''';
    expect(physicalSides(fixture, 'fixture.dart'), isEmpty);
  });

  test('values put into a sentence keep their own direction', () {
    // In Arabic, "Version 0.2.0-beta.2" once showed as "beta.2-0.2.0".
    expect(isolate('0.2.0-beta.2'), '\u20680.2.0-beta.2\u2069');
    expect(isolate('Download/TotalDrop'), '\u2068Download/TotalDrop\u2069');
    final settings = File('lib/ui/settings/settings_view.dart')
        .readAsStringSync();
    final receive = File('lib/ui/receive/receive_view.dart').readAsStringSync();
    final send = File('lib/ui/send/send_view.dart').readAsStringSync();
    expect(settings, contains('l.settingsVersion(isolate('));
    expect(receive, contains('l.receiveSaveTo(isolate('));
    expect(send, contains('l.sendShareText(isolate(code))'));
  });
}

/// Every call or constant in [source] that names a physical side, as
/// `path:line: name`, where name is what is called or read, such as
/// `EdgeInsets.only` or `TextAlign.left`. The file is parsed, so a call
/// that spans several lines is checked as a whole, and comments and
/// strings are ignored.
List<String> physicalSides(String source, String path) {
  final unit = parseString(
    content: source,
    path: path,
    throwIfDiagnostics: false,
  ).unit;
  final visitor = _PhysicalSides();
  unit.accept(visitor);
  return [
    for (final (offset, name) in visitor.found)
      '$path:${unit.lineInfo.getLocation(offset).lineNumber}: $name',
  ];
}

/// Named arguments that are a physical side, by the call they belong to.
const _sideArguments = <String, Set<String>>{
  'EdgeInsets.only': {'left', 'right'},
  'Positioned': {'left', 'right'},
  'Positioned.fill': {'left', 'right'},
  'AnimatedPositioned': {'left', 'right'},
  'BorderRadius.only': {'topLeft', 'topRight', 'bottomLeft', 'bottomRight'},
  'BorderRadius.horizontal': {'left', 'right'},
  'Border': {'left', 'right'},
};

/// Calls whose first and third positional arguments are left and right:
/// fine only when both sides are the same.
const _leftTopRightBottom = {'EdgeInsets.fromLTRB', 'Positioned.fromLTRB'};

final _sideConstants = {
  for (final type in ['Alignment', 'FractionalOffset'])
    for (final side in [
      'centerLeft',
      'centerRight',
      'topLeft',
      'topRight',
      'bottomLeft',
      'bottomRight',
    ])
      '$type.$side',
  'TextAlign.left',
  'TextAlign.right',
};

class _PhysicalSides extends RecursiveAstVisitor<void> {
  /// The offset and the name of each physical side found.
  final found = <(int, String)>[];

  @override
  void visitInstanceCreationExpression(InstanceCreationExpression node) {
    // Unresolved, `const EdgeInsets.only(…)` reads as type `only` with the
    // prefix `EdgeInsets`, so the parts are joined in order.
    final c = node.constructorName;
    final callee = [
      ?c.type.importPrefix?.name.lexeme,
      c.type.name.lexeme,
      ?c.name?.name,
    ].join('.');
    _check(node.offset, callee, node.argumentList);
    super.visitInstanceCreationExpression(node);
  }

  // Without `const` or `new`, the parser cannot tell `EdgeInsets.only(…)`
  // or `Positioned(…)` from a method call.
  @override
  void visitMethodInvocation(MethodInvocation node) {
    final target = node.target;
    final method = node.methodName.name;
    if (target == null) {
      _check(node.offset, method, node.argumentList);
    } else if (target is SimpleIdentifier) {
      _check(node.offset, '${target.name}.$method', node.argumentList);
    }
    super.visitMethodInvocation(node);
  }

  @override
  void visitPrefixedIdentifier(PrefixedIdentifier node) {
    _constant(node.offset, '${node.prefix.name}.${node.identifier.name}');
    super.visitPrefixedIdentifier(node);
  }

  @override
  void visitPropertyAccess(PropertyAccess node) {
    final target = node.target;
    if (target is SimpleIdentifier) {
      _constant(node.offset, '${target.name}.${node.propertyName.name}');
    }
    super.visitPropertyAccess(node);
  }

  void _constant(int offset, String name) {
    if (_sideConstants.contains(name)) found.add((offset, name));
  }

  void _check(int offset, String callee, ArgumentList list) {
    final sides = _sideArguments[callee];
    if (sides != null &&
        list.arguments.any(
          (a) => a is NamedArgument && sides.contains(a.name.lexeme),
        )) {
      found.add((offset, callee));
    }
    if (_leftTopRightBottom.contains(callee)) {
      final positional = [
        for (final a in list.arguments)
          if (a is! NamedArgument) _lexemes(a),
      ];
      if (positional.length == 4 && positional[0] != positional[2]) {
        found.add((offset, callee));
      }
    }
  }

  /// The tokens of [node], joined, to compare two arguments.
  static String _lexemes(AstNode node) {
    final out = StringBuffer();
    for (Token? t = node.beginToken; t != null; t = t.next) {
      out.write(t.lexeme);
      if (t == node.endToken) break;
    }
    return out.toString();
  }
}
