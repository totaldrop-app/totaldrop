import 'dart:io';

import 'package:analyzer/dart/analysis/utilities.dart';
import 'package:analyzer/dart/ast/ast.dart';
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
]);
''';
    expect(physicalSides(fixture, 'fixture.dart'), [
      'fixture.dart:3: const EdgeInsets.only(top: 8, left: 16)',
      'fixture.dart:8: Positioned(top: 0, right: 0, child: x)',
      'fixture.dart:14: EdgeInsets.fromLTRB(8, 0, 16, 0)',
      'fixture.dart:21: BorderRadius.only(topLeft: r)',
      'fixture.dart:27: TextAlign.right',
    ]);
  });

  test('start and end, and equal sides, pass', () {
    const fixture = '''
final a = [
  const EdgeInsetsDirectional.only(start: 16, end: 8),
  PositionedDirectional(start: 0, child: x),
  EdgeInsets.fromLTRB(16, 0, 16, 8),
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
/// `path:line: code`. The file is parsed, so a call that spans several
/// lines is checked as a whole, and comments and strings are ignored.
List<String> physicalSides(String source, String path) {
  final unit = parseString(
    content: source,
    path: path,
    throwIfDiagnostics: false,
  ).unit;
  final visitor = _PhysicalSides();
  unit.accept(visitor);
  return [
    for (final node in visitor.found)
      '$path:${unit.lineInfo.getLocation(node.offset).lineNumber}: '
          '${node.toSource()}',
  ];
}

/// Named arguments that are a physical side, by the call they belong to.
const _sideArguments = <String, Set<String>>{
  'EdgeInsets.only': {'left', 'right'},
  'Positioned': {'left', 'right'},
  'Positioned.fill': {'left', 'right'},
  'BorderRadius.only': {'topLeft', 'topRight', 'bottomLeft', 'bottomRight'},
  'BorderRadius.horizontal': {'left', 'right'},
  'Border': {'left', 'right'},
};

/// Calls whose first and third positional arguments are left and right:
/// fine only when both sides are the same.
const _leftTopRightBottom = {'EdgeInsets.fromLTRB', 'Positioned.fromLTRB'};

const _sideConstants = {
  'Alignment.centerLeft',
  'Alignment.centerRight',
  'Alignment.topLeft',
  'Alignment.topRight',
  'Alignment.bottomLeft',
  'Alignment.bottomRight',
  'TextAlign.left',
  'TextAlign.right',
};

class _PhysicalSides extends RecursiveAstVisitor<void> {
  final found = <AstNode>[];

  @override
  void visitInstanceCreationExpression(InstanceCreationExpression node) {
    _check(node, node.constructorName.toSource(), node.argumentList);
    super.visitInstanceCreationExpression(node);
  }

  // Without `const` or `new`, the parser cannot tell `EdgeInsets.only(…)`
  // or `Positioned(…)` from a method call.
  @override
  void visitMethodInvocation(MethodInvocation node) {
    final target = node.target;
    final callee = target == null
        ? node.methodName.name
        : '${target.toSource()}.${node.methodName.name}';
    _check(node, callee, node.argumentList);
    super.visitMethodInvocation(node);
  }

  @override
  void visitPrefixedIdentifier(PrefixedIdentifier node) {
    _constant(node);
    super.visitPrefixedIdentifier(node);
  }

  @override
  void visitPropertyAccess(PropertyAccess node) {
    _constant(node);
    super.visitPropertyAccess(node);
  }

  void _constant(Expression node) {
    if (_sideConstants.contains(node.toSource())) found.add(node);
  }

  void _check(AstNode node, String callee, ArgumentList list) {
    final sides = _sideArguments[callee];
    if (sides != null &&
        list.arguments.any(
          (a) => a is NamedArgument && sides.contains(a.name.lexeme),
        )) {
      found.add(node);
    }
    if (_leftTopRightBottom.contains(callee)) {
      final positional = [
        for (final a in list.arguments)
          if (a is! NamedArgument) a.argumentExpression.toSource(),
      ];
      if (positional.length == 4 && positional[0] != positional[2]) {
        found.add(node);
      }
    }
  }
}
