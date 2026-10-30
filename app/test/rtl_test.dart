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

  test('import prefixes and dot shorthands are checked too', () {
    const fixture = '''
final a = [
  m.EdgeInsets.only(left: 8),
  const m.EdgeInsets.only(right: 8),
  Text(x, textAlign: m.TextAlign.left),
  Align(alignment: m.Alignment.centerRight),
  Text(x, textAlign: .right),
  Padding(padding: .only(top: 4, left: 8)),
  Padding(padding: const .only(right: 8)),
  Padding(padding: .fromLTRB(8, 0, 16, 0)),
  Container(decoration: BoxDecoration(borderRadius: .only(topLeft: r))),
  Align(alignment: .bottomLeft),
];
''';
    expect(physicalSides(fixture, 'fixture.dart'), [
      'fixture.dart:2: m.EdgeInsets.only',
      'fixture.dart:3: m.EdgeInsets.only',
      'fixture.dart:4: m.TextAlign.left',
      'fixture.dart:5: m.Alignment.centerRight',
      'fixture.dart:6: .right',
      'fixture.dart:7: .only',
      'fixture.dart:8: .only',
      'fixture.dart:9: .fromLTRB',
      'fixture.dart:10: .only',
      'fixture.dart:11: .bottomLeft',
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
  m.EdgeInsetsDirectional.only(start: 8),
  Text(x, textAlign: .start),
  Padding(padding: .only(top: 4)),
  Padding(padding: .fromLTRB(8, 0, 8, 0)),
  Align(alignment: .centerStart),
];
''';
    expect(physicalSides(fixture, 'fixture.dart'), isEmpty);
  });

  test('values put into a sentence keep their own direction', () {
    // In Arabic, "Version 0.2.0-beta.2" once showed as "beta.2-0.2.0".
    expect(isolate('0.2.0-beta.2'), '\u20680.2.0-beta.2\u2069');
    expect(isolate('Download/TotalDrop'), '\u2068Download/TotalDrop\u2069');
    // Read without whitespace, so the formatter's line breaks don't matter.
    String code(String path) =>
        File(path).readAsStringSync().replaceAll(RegExp(r'\s'), '');
    final settings = code('lib/ui/settings/settings_view.dart');
    final receive = code('lib/ui/receive/receive_view.dart');
    final send = code('lib/ui/send/send_view.dart');
    expect(settings, contains('l.settingsVersion(isolate('));
    expect(receive, contains('l.receiveSaveTo(isolate('));
    expect(send, contains('l.sendShareText(isolate(code))'));
  });
}

/// Every call or constant in [source] that names a physical side, as
/// `path:line: name`, where name is what is called or read as written,
/// such as `EdgeInsets.only`, `m.TextAlign.left` or `.left`. The file is
/// parsed, so a call that spans several lines is checked as a whole, and
/// comments and strings are ignored.
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
  'BorderRadius.only': _corners,
  'BorderRadius.horizontal': {'left', 'right'},
  'Border': {'left', 'right'},
};

const _corners = {'topLeft', 'topRight', 'bottomLeft', 'bottomRight'};

/// Calls whose first and third positional arguments are left and right:
/// fine only when both sides are the same.
const _leftTopRightBottom = {'EdgeInsets.fromLTRB', 'Positioned.fromLTRB'};

const _sideMembers = {
  'centerLeft',
  'centerRight',
  'topLeft',
  'topRight',
  'bottomLeft',
  'bottomRight',
};

final _sideConstants = {
  for (final type in ['Alignment', 'FractionalOffset'])
    for (final side in _sideMembers) '$type.$side',
  'TextAlign.left',
  'TextAlign.right',
};

/// A dot shorthand (`.only(…)`, `.left`) has no type in an unresolved
/// parse, so it is judged by its name alone: the constructors and
/// constants above, without their type.
const _shorthandArguments = <String, Set<String>>{
  'only': {'left', 'right', ..._corners},
  'horizontal': {'left', 'right'},
  'fill': {'left', 'right'},
};
const _shorthandLeftTopRightBottom = {'fromLTRB'};
const _shorthandConstants = {'left', 'right', ..._sideMembers};

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
    _call(node.offset, callee, node.argumentList);
    super.visitInstanceCreationExpression(node);
  }

  // Without `const` or `new`, the parser cannot tell `EdgeInsets.only(…)`
  // or `Positioned(…)` from a method call.
  @override
  void visitMethodInvocation(MethodInvocation node) {
    final target = node.target;
    final method = node.methodName.name;
    if (target == null) {
      _call(node.offset, method, node.argumentList);
    } else if (_dotted(target) case final prefix?) {
      _call(node.offset, '$prefix.$method', node.argumentList);
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
    if (_dotted(node) case final name?) _constant(node.offset, name);
    super.visitPropertyAccess(node);
  }

  @override
  void visitDotShorthandInvocation(DotShorthandInvocation node) {
    _shorthand(node.offset, node.memberName.name, node.argumentList);
    super.visitDotShorthandInvocation(node);
  }

  @override
  void visitDotShorthandConstructorInvocation(
    DotShorthandConstructorInvocation node,
  ) {
    _shorthand(node.offset, node.constructorName.name, node.argumentList);
    super.visitDotShorthandConstructorInvocation(node);
  }

  @override
  void visitDotShorthandPropertyAccess(DotShorthandPropertyAccess node) {
    final name = node.propertyName.name;
    if (_shorthandConstants.contains(name)) found.add((node.offset, '.$name'));
    super.visitDotShorthandPropertyAccess(node);
  }

  /// `a`, `a.b` or `a.b.c` as written, or null for anything else.
  static String? _dotted(Expression e) => switch (e) {
    SimpleIdentifier(:final name) => name,
    PrefixedIdentifier(:final prefix, :final identifier) =>
      '${prefix.name}.${identifier.name}',
    PropertyAccess(:final target?, :final propertyName) => switch (_dotted(
      target,
    )) {
      final t? => '$t.${propertyName.name}',
      null => null,
    },
    _ => null,
  };

  /// [name] without an import prefix (`m.TextAlign.left` is
  /// `TextAlign.left`), as a key of the tables above.
  static String? _known(String name, Iterable<String> keys) {
    for (final k in keys) {
      if (name == k || name.endsWith('.$k')) return k;
    }
    return null;
  }

  void _constant(int offset, String name) {
    if (_known(name, _sideConstants) != null) found.add((offset, name));
  }

  void _call(int offset, String callee, ArgumentList list) {
    final key = _known(callee, [
      ..._sideArguments.keys,
      ..._leftTopRightBottom,
    ]);
    if (key == null) return;
    _check(
      offset,
      callee,
      list,
      _sideArguments[key],
      _leftTopRightBottom.contains(key),
    );
  }

  void _shorthand(int offset, String member, ArgumentList list) => _check(
    offset,
    '.$member',
    list,
    _shorthandArguments[member],
    _shorthandLeftTopRightBottom.contains(member),
  );

  void _check(
    int offset,
    String name,
    ArgumentList list,
    Set<String>? sides,
    bool leftTopRightBottom,
  ) {
    if (sides != null &&
        list.arguments.any(
          (a) => a is NamedArgument && sides.contains(a.name.lexeme),
        )) {
      found.add((offset, name));
    }
    if (leftTopRightBottom) {
      final positional = [
        for (final a in list.arguments)
          if (a is! NamedArgument) _lexemes(a),
      ];
      if (positional.length == 4 && positional[0] != positional[2]) {
        found.add((offset, name));
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
